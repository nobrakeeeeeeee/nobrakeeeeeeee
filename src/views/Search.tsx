import { useState } from "react";
import { api, type RaceSearchQuery, type RaceSearchRow } from "../api";
import type { RaceSelection } from "../App";

const PLACES: [number, string][] = [
  [2, "川口"],
  [3, "伊勢崎"],
  [4, "浜松"],
  [5, "飯塚"],
  [6, "山陽"],
];

const BET_TYPES: [string, string][] = [
  ["tns", "単勝"],
  ["fns", "複勝"],
  ["rtw", "2連単"],
  ["rfw", "2連複"],
  ["rt3", "3連単"],
  ["rf3", "3連複"],
  ["wid", "ワイド"],
];

interface Props {
  onOpenRace: (s: RaceSelection) => void;
}

export default function Search({ onOpenRace }: Props) {
  const [player, setPlayer] = useState("");
  const [placeCode, setPlaceCode] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [ordMin, setOrdMin] = useState("");
  const [ordMax, setOrdMax] = useState("");
  const [betType, setBetType] = useState("");
  const [minRefund, setMinRefund] = useState("");
  const [rows, setRows] = useState<RaceSearchRow[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const search = async () => {
    setLoading(true);
    setError(null);
    const q: RaceSearchQuery = {};
    if (player.trim()) q.player = player.trim();
    if (placeCode) q.place_code = Number(placeCode);
    if (dateFrom) q.date_from = dateFrom;
    if (dateTo) q.date_to = dateTo;
    if (ordMin) q.ord_min = Number(ordMin);
    if (ordMax) q.ord_max = Number(ordMax);
    if (betType && minRefund) {
      q.bet_type = betType;
      q.min_refund = Number(minRefund);
    }
    try {
      setRows(await api.searchRaces(q));
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const open = (r: RaceSearchRow) =>
    onOpenRace({
      placeCode: r.place_code,
      placeName: r.place_name,
      raceDate: r.race_date,
      title: `${r.place_name} ${r.race_date} ${r.race_no}R ${r.player_name}`,
      raceNo: r.race_no,
    });

  return (
    <section>
      <div className="toolbar">
        <h2>レース検索</h2>
        <button onClick={search} disabled={loading}>
          {loading ? "検索中..." : "検索"}
        </button>
      </div>
      <div className="search-form">
        <label>
          選手
          <input
            placeholder="名前・カナ・コード"
            value={player}
            onChange={(e) => setPlayer(e.target.value)}
          />
        </label>
        <label>
          場
          <select value={placeCode} onChange={(e) => setPlaceCode(e.target.value)}>
            <option value="">全場</option>
            {PLACES.map(([c, n]) => (
              <option key={c} value={c}>
                {n}
              </option>
            ))}
          </select>
        </label>
        <label>
          開始日
          <input type="date" value={dateFrom} onChange={(e) => setDateFrom(e.target.value)} />
        </label>
        <label>
          終了日
          <input type="date" value={dateTo} onChange={(e) => setDateTo(e.target.value)} />
        </label>
        <label>
          着順
          <select value={ordMin} onChange={(e) => setOrdMin(e.target.value)}>
            <option value="">〜</option>
            {[1, 2, 3, 4, 5, 6, 7, 8].map((n) => (
              <option key={n} value={n}>
                {n}着以上
              </option>
            ))}
          </select>
          〜
          <select value={ordMax} onChange={(e) => setOrdMax(e.target.value)}>
            <option value="">〜</option>
            {[1, 2, 3, 4, 5, 6, 7, 8].map((n) => (
              <option key={n} value={n}>
                {n}着以内
              </option>
            ))}
          </select>
        </label>
        <label>
          高配当
          <select value={betType} onChange={(e) => setBetType(e.target.value)}>
            <option value="">指定なし</option>
            {BET_TYPES.map(([k, n]) => (
              <option key={k} value={k}>
                {n}
              </option>
            ))}
          </select>
          <input
            type="number"
            placeholder="最低払戻(円)"
            value={minRefund}
            onChange={(e) => setMinRefund(e.target.value)}
            style={{ width: 110 }}
          />
          以上
        </label>
      </div>
      <p className="muted" style={{ fontSize: 12 }}>
        条件を空にすると収集済みの全レース結果を表示します（最大1000件）。行をクリックするとレース詳細を開きます。
      </p>
      {error && <p className="error">{error}</p>}
      {rows != null && (
        <>
          <p className="muted">{rows.length} 件</p>
          <div className="search-results">
            <table>
              <thead>
                <tr>
                  <th>日付</th>
                  <th>場</th>
                  <th>R</th>
                  <th>タイトル</th>
                  <th>着</th>
                  <th>車</th>
                  <th>選手</th>
                  <th>タイム</th>
                  <th>ST</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r, i) => (
                  <tr
                    key={i}
                    className="clickable"
                    onClick={() => open(r)}
                  >
                    <td>{r.race_date}</td>
                    <td>{r.place_name}</td>
                    <td>{r.race_no}</td>
                    <td className="muted">{r.title || r.grade_name}</td>
                    <td className={r.ord <= 3 ? "top3" : ""}>{r.ord}</td>
                    <td>{r.car_no}</td>
                    <td>{r.player_name}</td>
                    <td>{r.race_time}</td>
                    <td>{r.st}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </section>
  );
}
