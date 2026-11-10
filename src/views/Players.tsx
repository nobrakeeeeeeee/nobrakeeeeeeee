import { useEffect, useRef, useState, type ReactNode } from "react";
import { api, type PlayerRow, type PlayerHistoryRow, type PlayerStats, type PlayerStatGroup } from "../api";

export default function Players() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<PlayerRow[]>([]);
  const [selected, setSelected] = useState<PlayerRow | null>(null);
  const [profile, setProfile] = useState<any>(null);
  const [history, setHistory] = useState<PlayerHistoryRow[]>([]);
  const [stats, setStats] = useState<PlayerStats | null>(null);
  const [favs, setFavs] = useState<Set<string>>(new Set());
  const [favOnly, setFavOnly] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [syncMsg, setSyncMsg] = useState("");
  const timer = useRef<number | null>(null);

  const search = async (q: string) => {
    try {
      setResults(
        favOnly ? await api.favoritePlayers() : await api.searchPlayers(q.trim())
      );
    } catch (e) {
      setError(String(e));
    }
  };

  const loadFavs = async () => {
    try {
      const ps = await api.favoritePlayers();
      setFavs(new Set(ps.map((p) => p.player_code)));
    } catch {
      /* ignore */
    }
  };

  const toggleFav = async (code: string) => {
    try {
      const now = await api.toggleFavorite(code);
      setFavs((prev) => {
        const s = new Set(prev);
        if (now) s.add(code);
        else s.delete(code);
        return s;
      });
      if (favOnly) search("");
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    loadFavs();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (timer.current) window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => search(query), 300);
    return () => {
      if (timer.current) window.clearTimeout(timer.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query, favOnly]);

  const openPlayer = async (p: PlayerRow) => {
    setSelected(p);
    setProfile(null);
    setHistory([]);
    setStats(null);
    setError(null);
    try {
      const d = await api.playerDetail(p.player_code);
      setProfile(d.profile);
      setHistory(d.history);
      setStats(d.stats);
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
          placeholder="選手コード / 名前・カナで検索"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <label className="fav-filter">
          <input
            type="checkbox"
            checked={favOnly}
            onChange={(e) => setFavOnly(e.target.checked)}
          />
          お気に入りのみ
        </label>
        <p className="muted" style={{ fontSize: 12, margin: "0 0 8px" }}>
          {favOnly
            ? "お気に入り登録した選手を表示しています。"
            : "空欄のままにすると全選手をコード順に一覧表示します。"}
        </p>
        {error && <p className="error">{error}</p>}
        <div className="player-list">
          {results.map((p) => (
            <div key={p.player_code} className="hold-row">
              <button
                className={
                  selected?.player_code === p.player_code ? "hold-item active" : "hold-item"
                }
                onClick={() => openPlayer(p)}
              >
                <span className="hold-place">{p.player_code}</span>
                {p.player_name} <span className="muted">{p.player_name_kana}</span>
              </button>
              <button
                className={favs.has(p.player_code) ? "chip fav on" : "chip fav"}
                title="お気に入り"
                onClick={() => toggleFav(p.player_code)}
              >
                {favs.has(p.player_code) ? "★" : "☆"}
              </button>
            </div>
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
              </span>{" "}
              <button
                className={favs.has(selected.player_code) ? "chip fav on" : "chip fav"}
                title="お気に入り"
                onClick={() => toggleFav(selected.player_code)}
              >
                {favs.has(selected.player_code) ? "★" : "☆"}
              </button>
            </h2>
            {profile == null && <p>読み込み中...</p>}
            {profile != null && <ProfileView profile={profile} />}
            <h3>収集データ成績</h3>
            {stats == null ? (
              <p className="muted">読み込み中...</p>
            ) : stats.races === 0 ? (
              <p className="muted">収集済みデータなし</p>
            ) : (
              <StatsView stats={stats} />
            )}
            {history.length >= 2 && <OrdTrendChart history={history} />}
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

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <>
      <h3>{title}</h3>
      {children}
    </>
  );
}

function InfoTable({ rows }: { rows: [string, ReactNode][] }) {
  const visible = rows.filter(([, v]) => v !== null && v !== undefined && v !== "");
  if (visible.length === 0) return null;
  return (
    <table>
      <tbody>
        {visible.map(([k, v]) => (
          <tr key={k}>
            <td className="muted" style={{ width: 140 }}>{k}</td>
            <td>{v}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

const dash = (v: any) => (v === null || v === undefined || v === "" ? "-" : String(v));
const pct = (v: any) => (v === null || v === undefined || v === "" ? "-" : `${v}%`);
const joinList = (v: any) => (Array.isArray(v) ? v.filter(Boolean).join("、") : dash(v));

const statRate = (n: number, d: number) =>
  d > 0 ? `${((n / d) * 100).toFixed(1)}%` : "-";

function StatGroupTable({ title, rows }: { title: string; rows: PlayerStatGroup[] }) {
  if (rows.length === 0) return null;
  return (
    <>
      <h4 style={{ margin: "12px 0 4px" }}>{title}</h4>
      <table>
        <thead>
          <tr>
            <th></th>
            <th>出走</th>
            <th>1着率</th>
            <th>2連対率</th>
            <th>3連対率</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((g) => (
            <tr key={g.label}>
              <td>{g.label}</td>
              <td>{g.races}</td>
              <td>{statRate(g.first, g.races)}</td>
              <td>{statRate(g.top2, g.races)}</td>
              <td>{statRate(g.top3, g.races)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

function OrdTrendChart({ history }: { history: PlayerHistoryRow[] }) {
  const pts = [...history].reverse().filter((h) => h.ord != null && h.ord > 0);
  if (pts.length < 2) return null;
  const W = 640;
  const H = 120;
  const padL = 26;
  const padR = 10;
  const padT = 10;
  const padB = 20;
  const maxOrd = 9;
  const x = (i: number) =>
    padL + (pts.length === 1 ? 0 : (i * (W - padL - padR)) / (pts.length - 1));
  const y = (ord: number) => padT + ((ord - 1) / (maxOrd - 1)) * (H - padT - padB);
  const line = pts
    .map((p, i) => `${i === 0 ? "M" : "L"}${x(i).toFixed(1)},${y(p.ord!).toFixed(1)}`)
    .join(" ");
  const first = pts[0].race_date.slice(5);
  const last = pts[pts.length - 1].race_date.slice(5);
  return (
    <div style={{ margin: "8px 0 16px" }}>
      <h4 style={{ margin: "8px 0 4px" }}>着順推移（収集データ・古い順）</h4>
      <svg viewBox={`0 0 ${W} ${H}`} className="ord-chart">
        {[1, 3, 6, 9].map((o) => (
          <g key={o}>
            <line
              x1={padL}
              x2={W - padR}
              y1={y(o)}
              y2={y(o)}
              stroke="#2c3240"
              strokeWidth={1}
            />
            <text x={4} y={y(o) + 3} fontSize={9} fill="#9aa3b2">
              {o}
            </text>
          </g>
        ))}
        <path d={line} fill="none" stroke="#6ea8fe" strokeWidth={1.5} />
        {pts.map((p, i) => (
          <circle
            key={i}
            cx={x(i)}
            cy={y(p.ord!)}
            r={p.ord! <= 3 ? 3.5 : 2.5}
            fill={p.ord! <= 3 ? "#7ee2a8" : "#6ea8fe"}
          >
            <title>{`${p.race_date} ${p.place_code} ${p.race_no}R ${p.ord}着`}</title>
          </circle>
        ))}
        <text x={padL} y={H - 4} fontSize={9} fill="#9aa3b2">
          {first}
        </text>
        <text x={W - padR} y={H - 4} fontSize={9} fill="#9aa3b2" textAnchor="end">
          {last}
        </text>
      </svg>
    </div>
  );
}

function StatsView({ stats }: { stats: PlayerStats }) {
  return (
    <div>
      <p className="muted" style={{ fontSize: 12 }}>
        収集済みレース（{stats.races}走）のみの集計です。
      </p>
      <InfoTable
        rows={[
          ["出走", stats.races],
          ["1着", stats.first],
          ["2着", stats.second],
          ["3着", stats.third],
          ["単勝率", statRate(stats.first, stats.races)],
          ["2連対率", statRate(stats.top2, stats.races)],
          ["3連対率", statRate(stats.top3, stats.races)],
          ["平均ST", stats.avg_st != null ? stats.avg_st.toFixed(2) : null],
          ["平均タイム", stats.avg_time != null ? stats.avg_time.toFixed(3) : null],
        ]}
      />
      <StatGroupTable title="場別成績" rows={stats.by_place} />
      <StatGroupTable title="車番別成績" rows={stats.by_car} />
    </div>
  );
}

function ProfileView({ profile }: { profile: any }) {
  const p = profile?.profile ?? {};
  const tr = profile?.totalResult ?? {};
  const ch = profile?.characteristics ?? {};
  const l10 = profile?.latest10Result ?? {};
  const l10g = profile?.latest10GoodTrackResult ?? {};
  const yr = profile?.thisYearResult ?? {};
  const l90 = profile?.latest90Result ?? {};
  const l180 = profile?.latest180Result ?? {};
  const titles = profile?.winTitleList ?? [];
  const schedule = profile?.attendScheduleList ?? [];
  const recent = profile?.latest5ResultList ?? [];

  return (
    <div>
      <Section title="基本情報">
        <InfoTable
          rows={[
            ["氏名", p.playerName],
            ["フリガナ", p.playerNameKana],
            ["ローマ字", p.playerNameEn],
            ["所属", p.placeName],
            ["出身地", p.hometown],
            ["年齢", p.age != null ? `${p.age}歳` : null],
            ["生年月日", p.birthday],
            ["期別", p.graduation],
            ["選手登録", p.registDate],
            ["登録番号", p.registNo],
            ["所有車", joinList(p.motorcycleList)],
            ["身長", p.height != null ? `${p.height}cm` : null],
            ["体重", p.weight != null ? `${p.weight}kg` : null],
            ["血液型", p.blood],
            ["星座", p.constellation],
            ["趣味", joinList(p.hobbyList)],
          ]}
        />
      </Section>

      <Section title="ランク・ポイント">
        <InfoTable
          rows={[
            ["現行ランク", p.currentRank],
            ["前期ランク", p.lastRank],
            ["審査ポイント", p.point],
          ]}
        />
      </Section>

      <Section title="通算成績">
        <InfoTable
          rows={[
            ["通算V回数", tr.winCount],
            ["1着", tr.countOf1th],
            ["2着", tr.countOf2th],
            ["3着", tr.countOf3th],
            ["単勝率", pct(tr.rate1)],
            ["2連対率", pct(tr.rate2)],
            ["3連対率", pct(tr.rate3)],
          ]}
        />
      </Section>

      <Section title="選手の特徴">
        <InfoTable
          rows={[
            ["Ｓ力", ch.startPower],
            ["独走力", ch.runAlonePower],
            ["追込み力", ch.catchUpPower],
            ["雨巧拙", ch.wetTrackSkill],
            ["コース", ch.course],
          ]}
        />
      </Section>

      <Section title="近10走">
        <table>
          <thead>
            <tr>
              <th>1着</th>
              <th>2着</th>
              <th>3着</th>
              <th>着外</th>
              <th>2連対率</th>
              <th>3連対率</th>
              <th>平均試走T</th>
              <th>平均競走T</th>
              <th>最高競走T</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>{dash(l10.countOf1th)}</td>
              <td>{dash(l10.countOf2th)}</td>
              <td>{dash(l10.countOf3th)}</td>
              <td>{dash(l10.countOfOther)}</td>
              <td>{pct(l10.rate2)}</td>
              <td>{pct(l10.rate3)}</td>
              <td>{dash(l10g.traialAveTime)}</td>
              <td>{dash(l10g.raceAveTime)}</td>
              <td>{dash(l10g.raceBestTime)}</td>
            </tr>
          </tbody>
        </table>
        <InfoTable
          rows={[
            ["今年V", yr.winCount],
            ["今年優出", yr.advanceFinalCount],
          ]}
        />
      </Section>

      <Section title="近90日成績">
        <InfoTable
          rows={[
            ["出走", l90.runCount],
            ["優出", l90.advanceFinalCount],
            ["優勝", l90.winCount],
            ["直近優勝", l90.latestWinDate ? `${l90.latestWinDate} ${l90.latestWinPlace ?? ""}` : null],
          ]}
        />
        <table>
          <thead>
            <tr>
              <th></th>
              <th>平均試走T</th>
              <th>平均競走T</th>
              <th>平均ST</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td className="muted">良</td>
              <td>{dash(l90.goodTrackTime?.traialAveTime)}</td>
              <td>{dash(l90.goodTrackTime?.raceAveTime)}</td>
              <td>{dash(l90.goodTrackTime?.stAve)}</td>
            </tr>
            <tr>
              <td className="muted">湿</td>
              <td>{dash(l90.wetTrackTime?.traialAveTime)}</td>
              <td>{dash(l90.wetTrackTime?.raceAveTime)}</td>
              <td>{dash(l90.wetTrackTime?.stAve)}</td>
            </tr>
          </tbody>
        </table>
      </Section>

      <Section title="勝率(180日)">
        <table>
          <thead>
            <tr>
              <th></th>
              <th>単勝率</th>
              <th>2連対率</th>
              <th>3連対率</th>
            </tr>
          </thead>
          <tbody>
            {(["goodTrack", "wetTrack"] as const).map((k) => {
              const g = l180[k] ?? {};
              const cell = (r: any) =>
                r ? `${r.winRate}% (${r.winRatio})` : "-";
              return (
                <tr key={k}>
                  <td className="muted">{k === "goodTrack" ? "良" : "湿"}</td>
                  <td>{cell(g.rate1)}</td>
                  <td>{cell(g.rate2)}</td>
                  <td>{cell(g.rate3)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </Section>

      <Section title="直近成績">
        {recent.length === 0 ? (
          <p className="muted">データなし</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>グレード</th>
                <th>場</th>
                <th>日付</th>
                <th>R</th>
                <th>種別</th>
                <th>着</th>
                <th>試走T</th>
                <th>競走T</th>
                <th>ST</th>
              </tr>
            </thead>
            <tbody>
              {recent.map((r: any, i: number) => (
                <tr key={i}>
                  <td>{r.gradeName}</td>
                  <td>{r.placeName}</td>
                  <td>{r.raceDate}</td>
                  <td>{r.raceNo}</td>
                  <td>{r.raceType}</td>
                  <td>{r.order}</td>
                  <td>{r.traialTime}</td>
                  <td>{r.raceTime}</td>
                  <td>{r.st}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>

      <Section title="獲得タイトル">
        {titles.length === 0 ? (
          <p className="muted">データなし</p>
        ) : (
          titles.map((g: any, i: number) => (
            <div key={i} style={{ marginBottom: 8 }}>
              <strong>{g.gradeName}</strong>（{g.gradeWinCount}回）
              <ul style={{ margin: "4px 0 0", paddingLeft: 20 }}>
                {(g.titleList ?? []).map((t: any, j: number) => (
                  <li key={j}>
                    {t.paragraphName} {t.yearList?.join("・")}（{t.paragraphWinCount}回）
                  </li>
                ))}
              </ul>
            </div>
          ))
        )}
      </Section>

      <Section title="今後の出場予定">
        {schedule.length === 0 ? (
          <p className="muted">データなし</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>グレード</th>
                <th>場</th>
                <th>期間</th>
                <th>タイトル</th>
              </tr>
            </thead>
            <tbody>
              {schedule.map((s: any, i: number) => (
                <tr key={i}>
                  <td>{s.gradeName}</td>
                  <td>{s.placeName}</td>
                  <td>
                    {s.startDate}〜{s.endDate}
                    {s.status ? `（${s.status}）` : ""}
                  </td>
                  <td>{s.title}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>
    </div>
  );
}
