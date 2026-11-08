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
  const [raceDate, setRaceDate] = useState("");
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
    if (!raceDate) return;
    setBusy(true);
    setLogs([]);
    setReport(null);
    try {
      const r = await api.collectDay(placeCode, raceDate);
      setReport(r);
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
          日付:{" "}
          <input type="date" value={raceDate} onChange={(e) => setRaceDate(e.target.value)} />
        </label>
        <button onClick={collect} disabled={busy || !raceDate}>
          {busy ? "収集中..." : "この日を収集"}
        </button>
        <button onClick={syncPlayers} disabled={busy}>
          選手マスタを同期
        </button>
      </div>

      {report && (
        <div className="card" style={{ marginTop: 12 }}>
          <div>
            {report.race_date} (place {report.place_code}): R{report.races_attempted}試行 /
            プログラム {report.programs_saved} / 出走 {report.entries_saved} /
            結果 {report.results_saved} / レース情報 {report.infos_saved}
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
