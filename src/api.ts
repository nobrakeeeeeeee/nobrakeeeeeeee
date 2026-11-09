import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Hold {
  place_code: number;
  race_date: string;
  place_key: string;
  place_name: string;
  period_start: string;
  period_end: string;
  paragraphday: number;
  grade_name: string;
  title: string;
  title_csv: string;
  nighter_name: string | null;
  race_name: string;
  race_class: string;
  distance: number | null;
  live_start_time: string | null;
  live_end_time: string | null;
  race_start_time: string | null;
  now_race_no: number | null;
  result_race_no: number | null;
}

export interface RaceInfo {
  race_no: number;
  race_name: string;
  race_class: string;
  distance: number | null;
  weather: string;
  race_weather: string;
  temp: string;
  humid: string;
  roadtemp: string;
  situation_code: number | null;
  race_start_time: string;
  final_race_no: number | null;
  grade_name: string;
  title: string;
  paragraphday: number | null;
}

export interface Entry {
  car_no: number;
  player_code: string;
  player_name: string;
  player_name_kana: string;
  age: number | null;
  rank: string;
  handicap: number | null;
  bike_class: number | null;
  bike_name: string;
  graduation_code: string;
  place_name: string;
  trial_run_time: string;
  absent: number;
}

export interface RaceResultRow {
  ord: number;
  car_no: number | null;
  player_code: string | null;
  player_name: string | null;
  player_name_en: string | null;
  motorcycle_name: string | null;
  handicap: number | null;
  trial_time: string | null;
  race_time: string | null;
  st: string | null;
  accident_name: string | null;
  foul_code: string | null;
}

export interface RefundRow {
  bet_type: string;
  seq: number;
  combo: string | null;
  finish_order: number | null;
  refund: number | null;
  popularity: number | null;
  votes: string | null;
  type_name: string | null;
}

export interface OddsRow {
  bet_type: string;
  seq: number;
  combo: string;
  odds: string;
  sort_odds: number;
}

export interface OddsMeta {
  sales: any;
  expect: any;
  pop: any;
  player_stats: any;
  comment: string | null;
}

export interface RaceDetail {
  hold: Hold | null;
  infos: RaceInfo[];
  entries: Entry[];
  results: RaceResultRow[];
  refunds: RefundRow[];
  odds: OddsRow[];
  odds_meta: OddsMeta | null;
}

export interface PlayerRow {
  player_code: string;
  player_name: string;
  player_name_kana: string;
  place_code: number | null;
  place_key: string;
}

export interface PlayerHistoryRow {
  race_date: string;
  place_code: number;
  race_no: number;
  ord: number | null;
  car_no: number | null;
  race_time: string | null;
  st: string | null;
  trial_time: string | null;
  accident_name: string | null;
}

export interface PlayerStatGroup {
  label: string;
  races: number;
  first: number;
  top2: number;
  top3: number;
}

export interface PlayerStats {
  races: number;
  first: number;
  second: number;
  third: number;
  top2: number;
  top3: number;
  avg_st: number | null;
  avg_time: number | null;
  by_place: PlayerStatGroup[];
  by_car: PlayerStatGroup[];
}

export interface RaceSearchRow {
  place_code: number;
  place_name: string;
  race_date: string;
  race_no: number;
  title: string;
  grade_name: string;
  ord: number;
  car_no: number | null;
  player_code: string;
  player_name: string;
  race_time: string | null;
  st: string | null;
}

export interface RaceSearchQuery {
  player?: string;
  place_code?: number;
  date_from?: string;
  date_to?: string;
  ord_min?: number;
  ord_max?: number;
  bet_type?: string;
  min_refund?: number;
}

export interface CollectReport {
  place_code: number;
  race_date: string;
  races_attempted: number;
  programs_saved: number;
  entries_saved: number;
  results_saved: number;
  infos_saved: number;
  odds_saved: number;
  errors: string[];
}

export const api = {
  todayHolds: () => invoke<Hold[]>("today_holds"),
  recentHolds: (placeCode: number) => invoke<Hold[]>("recent_holds", { placeCode }),
  listHolds: (limit = 300) => invoke<Hold[]>("list_holds", { limit }),
  raceNumbers: (placeCode: number, raceDate: string) =>
    invoke<number[]>("race_numbers", { placeCode, raceDate }),
  raceDetail: (placeCode: number, raceDate: string, raceNo: number) =>
    invoke<RaceDetail>("race_detail", { placeCode, raceDate, raceNo }),
  collectDay: (placeCode: number, raceDate: string) =>
    invoke<CollectReport>("collect_day", { placeCode, raceDate }),
  collectRecent: (placeCode: number | null) =>
    invoke<{ days_total: number; days_done: number; cancelled: boolean; errors: string[] }>(
      "collect_recent",
      { placeCode }
    ),
  collectRange: (placeCode: number, startDate: string, endDate: string) =>
    invoke<{ days_total: number; days_done: number; days_skipped: number; cancelled: boolean; errors: string[] }>(
      "collect_range",
      { placeCode, startDate, endDate }
    ),
  cancelCollection: () => invoke<void>("cancel_collection"),
  syncPlayers: () => invoke<number>("sync_players"),
  searchPlayers: (query: string) => invoke<PlayerRow[]>("search_players", { query }),
  playerDetail: (playerCode: string) =>
    invoke<{ profile: any; history: PlayerHistoryRow[]; stats: PlayerStats }>("player_detail", { playerCode }),
  searchRaces: (query: RaceSearchQuery) =>
    invoke<RaceSearchRow[]>("search_races", { query }),
  dbStats: () => invoke<{ holds: number; players: number }>("db_stats"),
  onCollectLog: (cb: (msg: string) => void): Promise<UnlistenFn> =>
    listen<string>("collect-log", (e) => cb(e.payload)),
};
