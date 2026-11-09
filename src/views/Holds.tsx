import { useEffect, useState } from "react";
import { api, type Hold, type OddsMeta, type OddsRow, type RaceDetail as RaceDetailData, type RefundRow } from "../api";
import type { RaceSelection } from "../App";

const PLACES: [number, string][] = [
  [2, "川口"],
  [3, "伊勢崎"],
  [4, "浜松"],
  [5, "飯塚"],
  [6, "山陽"],
];

interface Props {
  initial: RaceSelection | null;
}

export default function Holds({ initial }: Props) {
  const [holds, setHolds] = useState<Hold[]>([]);
  const [selected, setSelected] = useState<RaceSelection | null>(initial);
  const [raceNos, setRaceNos] = useState<number[]>([]);
  const [raceNo, setRaceNo] = useState<number | null>(null);
  const [detail, setDetail] = useState<RaceDetailData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const loadHolds = async () => {
    try {
      setHolds(await api.listHolds(500));
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    loadHolds();
  }, []);

  useEffect(() => {
    if (!selected) return;
    setRaceNo(selected.raceNo ?? null);
    setDetail(null);
    api
      .raceNumbers(selected.placeCode, selected.raceDate)
      .then(setRaceNos)
      .catch((e) => setError(String(e)));
  }, [selected]);

  useEffect(() => {
    if (!selected || raceNo == null) return;
    setLoading(true);
    api
      .raceDetail(selected.placeCode, selected.raceDate, raceNo)
      .then(setDetail)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [selected, raceNo]);

  const fetchRecent = async (placeCode: number) => {
    setError(null);
    try {
      await api.recentHolds(placeCode);
      await loadHolds();
    } catch (e) {
      setError(String(e));
    }
  };

  // Group holds by date
  const byDate = new Map<string, Hold[]>();
  for (const h of holds) {
    const arr = byDate.get(h.race_date) ?? [];
    arr.push(h);
    byDate.set(h.race_date, arr);
  }

  return (
    <section className="split">
      <div className="sidebar">
        <div className="toolbar">
          <h2>開催一覧</h2>
          <button onClick={loadHolds}>更新</button>
        </div>
        <div className="place-fetch">
          {PLACES.map(([code, name]) => (
            <button key={code} className="chip" onClick={() => fetchRecent(code)}>
              {name}の最近の開催を取得
            </button>
          ))}
        </div>
        {error && <p className="error">{error}</p>}
        {[...byDate.entries()].map(([date, list]) => (
          <div key={date}>
            <div className="date-header">{date}</div>
            {list.map((h) => {
              const active =
                selected?.placeCode === h.place_code && selected?.raceDate === h.race_date;
              return (
                <button
                  key={`${h.place_code}-${h.race_date}`}
                  className={active ? "hold-item active" : "hold-item"}
                  onClick={() =>
                    setSelected({
                      placeCode: h.place_code,
                      placeName: h.place_name,
                      raceDate: h.race_date,
                      title: `${h.place_name} ${h.race_date} ${h.title}`,
                    })
                  }
                >
                  <span className="hold-place">{h.place_name}</span>
                  <span className="muted">{h.title_csv || h.title}</span>
                  {h.result_race_no ? (
                    <span className="badge">結果{h.result_race_no}Rまで</span>
                  ) : null}
                </button>
              );
            })}
          </div>
        ))}
        {holds.length === 0 && <p className="muted">データがありません。右の「データ収集」または各場の「最近の開催を取得」でデータを集めてください。</p>}
      </div>
      <div className="content">
        {!selected && <p className="muted">左の一覧から開催日を選択してください。</p>}
        {selected && (
          <>
            <h2>{selected.title}</h2>
            {raceNos.length === 0 && !loading && (
              <p className="muted">この日のデータは未収集です。「データ収集」タブから収集してください。</p>
            )}
            <div className="race-tabs">
              {raceNos.map((n) => (
                <button
                  key={n}
                  className={raceNo === n ? "chip active" : "chip"}
                  onClick={() => setRaceNo(n)}
                >
                  {n}R
                </button>
              ))}
            </div>
            {loading && <p>読み込み中...</p>}
            {detail && raceNo != null && <RaceDetailView detail={detail} raceNo={raceNo} />}
          </>
        )}
      </div>
    </section>
  );
}

function RaceDetailView({ detail, raceNo }: { detail: RaceDetailData; raceNo: number }) {
  const info = detail.infos.find((i) => i.race_no === raceNo);
  return (
    <div>
      {info && (
        <div className="card-meta" style={{ marginBottom: 8 }}>
          {info.race_name} / {info.grade_name} / {info.distance}m / {info.race_start_time}発走
          {info.weather ? ` / 天候:${info.weather}` : ""}
        </div>
      )}
      {detail.results.length > 0 && (
        <>
          <h3>結果</h3>
          <table>
            <thead>
              <tr>
                <th>着</th>
                <th>車</th>
                <th>選手</th>
                <th>車名</th>
                <th>H</th>
                <th>試走</th>
                <th>タイム</th>
                <th>ST</th>
                <th>備考</th>
              </tr>
            </thead>
            <tbody>
              {detail.results.map((r) => (
                <tr key={r.ord} className={r.ord <= 3 ? "top3" : ""}>
                  <td>{r.ord}</td>
                  <td>{r.car_no}</td>
                  <td>{r.player_name}</td>
                  <td>{r.motorcycle_name}</td>
                  <td>{r.handicap}</td>
                  <td>{r.trial_time}</td>
                  <td>{r.race_time}</td>
                  <td>{r.st}</td>
                  <td>{r.accident_name || r.foul_code || ""}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
      {detail.refunds.length > 0 && <RefundsView refunds={detail.refunds} />}
      {detail.odds.length > 0 && <OddsView odds={detail.odds} meta={detail.odds_meta} />}
      <h3>出走表</h3>
      {detail.entries.length === 0 ? (
        <p className="muted">出走表データなし</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>車</th>
              <th>選手</th>
              <th>年齢</th>
              <th>ランク</th>
              <th>H</th>
              <th>級</th>
              <th>車名</th>
              <th>期</th>
              <th>所属</th>
              <th>試走</th>
            </tr>
          </thead>
          <tbody>
            {detail.entries.map((e) => (
              <tr key={e.car_no} className={e.absent ? "absent" : ""}>
                <td>{e.car_no}</td>
                <td>{e.player_name}</td>
                <td>{e.age}</td>
                <td>{e.rank}</td>
                <td>{e.handicap}</td>
                <td>{e.bike_class}</td>
                <td>{e.bike_name}</td>
                <td>{e.graduation_code}</td>
                <td>{e.place_name}</td>
                <td>{e.trial_run_time}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

const BET_LABELS: [string, string][] = [
  ["tns", "単勝"],
  ["fns", "複勝"],
  ["rtw", "2連単"],
  ["rfw", "2連複"],
  ["rt3", "3連単"],
  ["rf3", "3連複"],
  ["wid", "ワイド"],
];

function RefundsView({ refunds }: { refunds: RefundRow[] }) {
  const groups = new Map<string, RefundRow[]>();
  for (const r of refunds) {
    const list = groups.get(r.bet_type) ?? [];
    list.push(r);
    groups.set(r.bet_type, list);
  }
  const ordered = [
    ...BET_LABELS.map(([k]) => k).filter((k) => groups.has(k)),
    ...[...groups.keys()].filter((k) => !BET_LABELS.some(([bk]) => bk === k)),
  ];
  const labelOf = (k: string) => BET_LABELS.find(([bk]) => bk === k)?.[1] ?? k;
  return (
    <>
      <h3>払戻金</h3>
      <table>
        <thead>
          <tr>
            <th>賭式</th>
            <th>買い目</th>
            <th>着順</th>
            <th>払戻</th>
            <th>人気</th>
            <th>票数</th>
            <th>区分</th>
          </tr>
        </thead>
        <tbody>
          {ordered.flatMap((k) =>
            (groups.get(k) ?? []).map((r, i) => (
              <tr key={`${k}-${r.seq}-${i}`}>
                {i === 0 && (
                  <td rowSpan={(groups.get(k) ?? []).length}>{labelOf(k)}</td>
                )}
                <td>{r.combo ?? "-"}</td>
                <td>{r.finish_order ?? ""}</td>
                <td>{r.refund != null ? `${r.refund}円` : "-"}</td>
                <td>{r.popularity ?? "-"}番人気</td>
                <td>{r.votes ?? "-"}</td>
                <td>{r.type_name && r.type_name !== "通常" ? r.type_name : ""}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </>
  );
}

const EXPECT_LABELS: [string, string][] = [
  ["sunny", "晴天"],
  ["rain", "雨天"],
  ["st", "ST"],
  ["ai", "AI"],
];

function OddsView({ odds, meta }: { odds: OddsRow[]; meta: OddsMeta | null }) {
  const groups = new Map<string, OddsRow[]>();
  for (const o of odds) {
    const list = groups.get(o.bet_type) ?? [];
    list.push(o);
    groups.set(o.bet_type, list);
  }
  const ordered = [
    ...BET_LABELS.map(([k]) => k).filter((k) => groups.has(k)),
    ...[...groups.keys()].filter((k) => !BET_LABELS.some(([bk]) => bk === k)),
  ];
  const labelOf = (k: string) => BET_LABELS.find(([bk]) => bk === k)?.[1] ?? k;
  const [tab, setTab] = useState(ordered[0] ?? "tns");
  const rows = groups.get(tab) ?? [];

  const expects = meta?.expect && typeof meta.expect === "object" ? meta.expect : null;
  const sales = meta?.sales && typeof meta.sales === "object" ? meta.sales : null;

  return (
    <>
      <h3>オッズ</h3>
      {meta?.comment && <p className="muted" style={{ whiteSpace: "pre-wrap" }}>{meta.comment}</p>}
      {expects && (
        <div className="card-meta" style={{ marginBottom: 8 }}>
          {EXPECT_LABELS.map(([key, label]) => {
            const v = expects[key];
            const text = Array.isArray(v) ? v.join("、") : typeof v === "string" ? v : "";
            return text ? <span key={key} style={{ marginRight: 12 }}>{label}: {text}</span> : null;
          })}
        </div>
      )}
      <div className="race-tabs">
        {ordered.map((k) => (
          <button
            key={k}
            className={tab === k ? "chip active" : "chip"}
            onClick={() => setTab(k)}
          >
            {labelOf(k)}
          </button>
        ))}
      </div>
      <div className="odds-table">
        <table>
          <thead>
            <tr>
              <th>人気</th>
              <th>買い目</th>
              <th>オッズ</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((o, i) => (
              <tr key={`${o.bet_type}-${o.combo}-${i}`}>
                <td>{i + 1}</td>
                <td>{o.combo.split("-").join(" → ")}</td>
                <td>{o.odds}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {sales && (
        <p className="muted" style={{ fontSize: "0.85em" }}>
          {typeof sales.updateDate === "string" && `更新: ${sales.updateDate} / `}
          {BET_LABELS.map(([k]) => {
            const s = sales[k];
            const n = s && typeof s === "object" ? s.salesCount : null;
            return n != null ? `${labelOf(k)} ${Number(n).toLocaleString()}票 ` : null;
          })}
        </p>
      )}
    </>
  );
}
