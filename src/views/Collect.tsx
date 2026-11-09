import { useEffect, useRef, useState } from "react";
import { api, type CollectReport } from "../api";

const PLACES: [number, string][] = [
  [2, "川口"],
  [3, "伊勢崎"],
  [4, "浜松"],
  [5, "飯塚"],
  [6, "山陽"],
];

export default function Collect() {
  const [placeCode, setPlaceCode] = useState(2);
  const [startDate, setStartDate] = useState("");
  const [endDate, setEndDate] = useState("");
  const [logs, setLogs] = useState<string[]>([]);
  const [report, setReport] = useState<CollectReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [stats, setStats] = useState<{ holds: number; players: number } | null>(null);
  const unlisten = useRef<(() => void) | null>(null);

  useEffect(() => {
    api.onCollectLog((msg) => setLogs((l) => [...l, msg])).then((u) => {
      unlisten.current = u;
    });
    api.dbStats().then(setStats).catch(() => {});
    return () => unlisten.current?.();
  }, []);

  const collect = async () => {
    if (!startDate) return;
    setBusy(true);
    setLogs([]);
    setReport(null);
    try {
      const r = await api.collectRange(placeCode, startDate, endDate || startDate);
      setLogs((l) => [
        ...l,
        `=== 期間収集完了: ${r.days_done} 日分収集 / ${r.days_skipped} 日スキップ / 全${r.days_total}日${r.cancelled ? "（キャンセル）" : ""} ===`,
        ...(r.errors.length ? [`エラー ${r.errors.length} 件:`, ...r.errors.slice(0, 10)] : []),
      ]);
    } catch (e) {
      setLogs((l) => [...l, `エラー: ${e}`]);
    } finally {
      setBusy(false);
      api.dbStats().then(setStats).catch(() => {});
    }
  };

  const collectRecent = async (pc: number | null) => {
    setBusy(true);
    setLogs([]);
    setReport(null);
    try {
      const r = await api.collectRecent(pc);
      setLogs((l) => [
        ...l,
        `=== 一括収集完了: ${r.days_done}/${r.days_total} 日${r.cancelled ? "（キャンセル）" : ""} ===`,
        ...(r.errors.length ? [`エラー ${r.errors.length} 件:`, ...r.errors.slice(0, 10)] : []),
      ]);
    } catch (e) {
      setLogs((l) => [...l, `エラー: ${e}`]);
    } finally {
      setBusy(false);
      api.dbStats().then(setStats).catch(() => {});
    }
  };

  const syncPlayers = async () => {
    setBusy(true);
    try {
      const n = await api.syncPlayers();
      setLogs((l) => [...l, `選手マスタ ${n} 件を同期`]);
    } catch (e) {
      setLogs((l) => [...l, `選手同期エラー: ${e}`]);
    } finally {
      setBusy(false);
      api.dbStats().then(setStats).catch(() => {});
    }
  };

  return (
    <section>
      <h2>データ収集</h2>
      <p className="muted">
        公式サイトの内部APIから、人間が閲覧する程度の速度（約1.2秒間隔）でデータを取得します。
      </p>
      {stats && (
        <p className="muted">
          DB: 開催日 {stats.holds} 件 / 選手 {stats.players} 名
        </p>
      )}

      <div className="collect-form">
        <label>
          場:{" "}
          <select value={placeCode} onChange={(e) => setPlaceCode(Number(e.target.value))}>
            {PLACES.map(([c, n]) => (
              <option key={c} value={c}>
                {n}
              </option>
            ))}
          </select>
        </label>
        <label>
          開始日:{" "}
          <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
        </label>
        <label>
          終了日:{" "}
          <input type="date" value={endDate} onChange={(e) => setEndDate(e.target.value)} />
        </label>
        <button onClick={collect} disabled={busy || !startDate}>
          {busy ? "収集中..." : "期間を収集"}
        </button>
        <button onClick={syncPlayers} disabled={busy}>
          選手マスタを同期
        </button>
        {busy ? (
          <button onClick={() => api.cancelCollection()}>キャンセル</button>
        ) : (
          <>
            <button onClick={() => collectRecent(placeCode)}>
              この場の最近の開催を一括収集
            </button>
            <button onClick={() => collectRecent(null)}>全場の最近の開催を一括収集</button>
          </>
        )}
      </div>
      <p className="muted" style={{ marginTop: 6, fontSize: 12 }}>
        期間収集は開始日〜終了日を1日ずつ処理します（終了日が空なら開始日のみ、当日以降は自動で除外）。開催のない日は1リクエストでスキップ、開催日は最大36リクエスト×約1.2秒かかります。
      </p>

      {report && (
        <div className="card" style={{ marginTop: 12 }}>
          <div>
            {report.race_date} (place {report.place_code}): R{report.races_attempted}試行 /
            プログラム {report.programs_saved} / 出走 {report.entries_saved} /
            結果 {report.results_saved} / レース情報 {report.infos_saved} /
            オッズ {report.odds_saved}
          </div>
          {report.errors.length > 0 && (
            <ul className="error-list">
              {report.errors.map((e, i) => (
                <li key={i}>{e}</li>
              ))}
            </ul>
          )}
        </div>
      )}

      <h3>ログ</h3>
      <div className="log">
        {logs.map((l, i) => (
          <div key={i}>{l}</div>
        ))}
      </div>
    </section>
  );
}
