import { useEffect, useState } from "react";
import Today from "./views/Today";
import Holds from "./views/Holds";
import Players from "./views/Players";
import Collect from "./views/Collect";
import Search from "./views/Search";
import { checkForUpdate, installAndRelaunch } from "./updater";
import type { Update } from "@tauri-apps/plugin-updater";

type Tab = "today" | "holds" | "players" | "search" | "collect";

export interface RaceSelection {
  placeCode: number;
  placeName: string;
  raceDate: string;
  title: string;
  raceNo?: number;
}

export default function App() {
  const [tab, setTab] = useState<Tab>("today");
  const [sel, setSel] = useState<RaceSelection | null>(null);
  const [update, setUpdate] = useState<Update | null>(null);
  const [updating, setUpdating] = useState(false);

  useEffect(() => {
    checkForUpdate().then((u) => {
      if (u) setUpdate(u);
    });
  }, []);

  const applyUpdate = async () => {
    if (!update) return;
    setUpdating(true);
    try {
      await installAndRelaunch(update);
    } catch (e) {
      alert(`更新に失敗しました: ${e}`);
      setUpdating(false);
    }
  };

  const openRace = (s: RaceSelection) => {
    setSel(s);
    setTab("holds");
  };

  return (
    <div className="app">
      <header className="header">
        <h1>No Brakeeeeeeee</h1>
        <nav>
          {(
            [
              ["today", "本日の開催"],
              ["holds", "開催・レース"],
              ["players", "選手"],
              ["search", "検索"],
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
      {update && (
        <div className="update-banner">
          新しいバージョン v{update.version} が利用可能です
          <button onClick={applyUpdate} disabled={updating}>
            {updating ? "更新中..." : "今すぐ更新"}
          </button>
        </div>
      )}
      <main>
        {tab === "today" && <Today onOpenRace={openRace} />}
        {tab === "holds" && <Holds initial={sel} />}
        {tab === "players" && <Players />}
        {tab === "search" && <Search onOpenRace={openRace} />}
        {tab === "collect" && <Collect />}
      </main>
    </div>
  );
}
