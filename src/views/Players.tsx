import { useEffect, useRef, useState } from "react";
import { api, type PlayerRow, type PlayerHistoryRow } from "../api";

export default function Players() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<PlayerRow[]>([]);
  const [selected, setSelected] = useState<PlayerRow | null>(null);
  const [profile, setProfile] = useState<any>(null);
  const [history, setHistory] = useState<PlayerHistoryRow[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [syncMsg, setSyncMsg] = useState("");
  const timer = useRef<number | null>(null);

  const search = async (q: string) => {
    try {
      setResults(await api.searchPlayers(q.trim()));
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    if (timer.current) window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => search(query), 300);
    return () => {
      if (timer.current) window.clearTimeout(timer.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query]);

  const openPlayer = async (p: PlayerRow) => {
    setSelected(p);
    setProfile(null);
    setHistory([]);
    setError(null);
    try {
      const d = await api.playerDetail(p.player_code);
      setProfile(d.profile);
      setHistory(d.history);
    } catch (e) {
      setError(String(e));
    }
  };

  const sync = async () => {
    setSyncing(true);
    setSyncMsg("");
    try {
      const n = await api.syncPlayers();
      setSyncMsg(`${n} 名の選手マスタを同期しました`);
    } catch (e) {
      setSyncMsg(`同期失敗: ${e}`);
    } finally {
      setSyncing(false);
    }
  };

  return (
    <section className="split">
      <div className="sidebar">
        <div className="toolbar">
          <h2>選手検索</h2>
          <button onClick={sync} disabled={syncing}>
            {syncing ? "同期中..." : "マスタ同期"}
          </button>
        </div>
        {syncMsg && <p className="muted">{syncMsg}</p>}
        <input
          className="search"
          placeholder="選手コード(数字) / 名前・カナ(コピペ)"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <p className="muted" style={{ fontSize: 12, margin: "0 0 8px" }}>
          WSLg環境では日本語IMEが使えません。空欄のままにすると全選手をコード順に一覧表示します。
        </p>
        {error && <p className="error">{error}</p>}
        <div className="player-list">
          {results.map((p) => (
            <button
              key={p.player_code}
              className={
                selected?.player_code === p.player_code ? "hold-item active" : "hold-item"
              }
              onClick={() => openPlayer(p)}
            >
              <span className="hold-place">{p.player_code}</span>
              {p.player_name} <span className="muted">{p.player_name_kana}</span>
            </button>
          ))}
        </div>
      </div>
      <div className="content">
        {!selected && <p className="muted">選手を選択してください（まず「マスタ同期」で選手一覧を取得）。</p>}
        {selected && (
          <>
            <h2>
              {selected.player_name}{" "}
              <span className="muted">
                {selected.player_code} / {selected.player_name_kana}
              </span>
            </h2>
            {profile == null && <p>読み込み中...</p>}
            {profile != null && <ProfileView profile={profile} />}
            <h3>収集済み戦績</h3>
            {history.length === 0 ? (
              <p className="muted">収集済みデータなし</p>
            ) : (
              <table>
                <thead>
                  <tr>
                    <th>日付</th>
                    <th>場</th>
                    <th>R</th>
                    <th>着</th>
                    <th>車</th>
                    <th>タイム</th>
                    <th>ST</th>
                    <th>試走</th>
                    <th>備考</th>
                  </tr>
                </thead>
                <tbody>
                  {history.map((h, i) => (
                    <tr key={i}>
                      <td>{h.race_date}</td>
                      <td>{h.place_code}</td>
                      <td>{h.race_no}</td>
                      <td>{h.ord}</td>
                      <td>{h.car_no}</td>
                      <td>{h.race_time}</td>
                      <td>{h.st}</td>
                      <td>{h.trial_time}</td>
                      <td>{h.accident_name || ""}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </>
        )}
      </div>
    </section>
  );
}

function ProfileView({ profile }: { profile: any }) {
  const p = profile?.profile ?? {};
  const tr = profile?.totalResult ?? {};
  const ch = profile?.characteristics ?? {};
  const entries: [string, any][] = Object.entries(p);
  return (
    <div>
      <div className="card-meta" style={{ marginBottom: 8 }}>
        {p.playerName} / {p.playerNameKana} / {p.age}歳 / {p.placeName}
      </div>
      {entries.length > 0 && (
        <table>
          <tbody>
            {entries.map(([k, v]) => (
              <tr key={k}>
                <td className="muted">{k}</td>
                <td>{typeof v === "object" ? JSON.stringify(v) : String(v ?? "")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {(Object.keys(tr).length > 0 || Object.keys(ch).length > 0) && (
        <>
          <h3>通算成績</h3>
          <pre className="json">{JSON.stringify({ totalResult: tr, characteristics: ch }, null, 2)}</pre>
        </>
      )}
    </div>
  );
}
