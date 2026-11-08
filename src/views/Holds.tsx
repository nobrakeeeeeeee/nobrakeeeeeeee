import { useEffect, useState } from "react";
import { api, type Hold, type RaceDetail as RaceDetailData } from "../api";
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
    setRaceNo(null);
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
