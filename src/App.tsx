import { useState } from "react";
import Today from "./views/Today";
import Holds from "./views/Holds";
import Players from "./views/Players";
import Collect from "./views/Collect";

type Tab = "today" | "holds" | "players" | "collect";

export interface RaceSelection {
  placeCode: number;
  placeName: string;
  raceDate: string;
  title: string;
}

export default function App() {
  const [tab, setTab] = useState<Tab>("today");
  const [sel, setSel] = useState<RaceSelection | null>(null);

  const openRace = (s: RaceSelection) => {
    setSel(s);
    setTab("holds");
  };

  return (
    <div className="app">
      <header className="header">
        <h1>Nobrake</h1>
        <nav>
          {(
            [
              ["today", "本日の開催"],
              ["holds", "開催・レース"],
              ["players", "選手"],
              ["collect", "データ収集"],
            ] as [Tab, string][]
          ).map(([key, label]) => (
            <button
              key={key}
              className={tab === key ? "tab active" : "tab"}
              onClick={() => setTab(key)}
            >
              {label}
            </button>
          ))}
        </nav>
      </header>
      <main>
        {tab === "today" && <Today onOpenRace={openRace} />}
        {tab === "holds" && <Holds initial={sel} />}
        {tab === "players" && <Players />}
        {tab === "collect" && <Collect />}
      </main>
    </div>
  );
}
