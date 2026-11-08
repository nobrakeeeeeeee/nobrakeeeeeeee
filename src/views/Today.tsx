import { useEffect, useState } from "react";
import { api, type Hold } from "../api";
import type { RaceSelection } from "../App";

interface Props {
  onOpenRace: (s: RaceSelection) => void;
}

export default function Today({ onOpenRace }: Props) {
  const [holds, setHolds] = useState<Hold[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      setHolds(await api.todayHolds());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <section>
      <div className="toolbar">
        <h2>本日の開催</h2>
        <button onClick={load} disabled={loading}>
          {loading ? "取得中..." : "更新"}
        </button>
      </div>
      {error && <p className="error">{error}</p>}
      {holds && holds.length === 0 && <p>本日の開催はありません。</p>}
      <div className="hold-grid">
        {holds?.map((h) => (
          <div className="card" key={`${h.place_code}-${h.race_date}`}>
            <div className="card-title">
              {h.place_name} <span className="muted">{h.grade_name}</span>
            </div>
            <div className="card-sub">{h.title}</div>
            <div className="card-meta">
              {h.race_date}（{h.paragraphday}日目） {h.race_name} / {h.distance}m
              {h.nighter_name ? ` / ${h.nighter_name}` : ""}
            </div>
            <div className="card-times">
              発走 {h.race_start_time ?? "-"} / ライブ {h.live_start_time ?? "-"}〜
              {h.live_end_time ?? "-"}
            </div>
            <button
              className="link"
              onClick={() =>
                onOpenRace({
                  placeCode: h.place_code,
                  placeName: h.place_name,
                  raceDate: h.race_date,
                  title: `${h.place_name} ${h.race_date}`,
                })
              }
            >
              レースを表示
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}
