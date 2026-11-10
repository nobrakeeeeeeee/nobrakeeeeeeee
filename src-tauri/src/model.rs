use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hold {
    pub place_code: i64,
    pub race_date: String,
    pub place_key: String,
    pub place_name: String,
    pub period_start: String,
    pub period_end: String,
    pub paragraphday: i64,
    pub grade_name: String,
    pub title: String,
    pub title_csv: String,
    pub nighter_name: Option<String>,
    pub race_name: String,
    pub race_class: String,
    pub distance: Option<i64>,
    pub live_start_time: Option<String>,
    pub live_end_time: Option<String>,
    pub race_start_time: Option<String>,
    pub now_race_no: Option<i64>,
    pub result_race_no: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceInfo {
    pub race_no: i64,
    pub race_name: String,
    pub race_class: String,
    pub distance: Option<i64>,
    pub weather: String,
    pub race_weather: String,
    pub temp: String,
    pub humid: String,
    pub roadtemp: String,
    pub situation_code: Option<i64>,
    pub race_start_time: String,
    pub final_race_no: Option<i64>,
    pub grade_name: String,
    pub title: String,
    pub paragraphday: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub car_no: i64,
    pub player_code: String,
    pub player_name: String,
    pub player_name_kana: String,
    pub age: Option<i64>,
    pub rank: String,
    pub handicap: Option<i64>,
    pub bike_class: Option<i64>,
    pub bike_name: String,
    pub graduation_code: String,
    pub place_name: String,
    pub trial_run_time: String,
    pub absent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResultRow {
    pub ord: i64,
    pub car_no: Option<i64>,
    pub player_code: Option<String>,
    pub player_name: Option<String>,
    pub player_name_en: Option<String>,
    pub motorcycle_name: Option<String>,
    pub handicap: Option<i64>,
    pub trial_time: Option<String>,
    pub race_time: Option<String>,
    pub st: Option<String>,
    pub accident_name: Option<String>,
    pub foul_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRow {
    pub bet_type: String,
    pub seq: i64,
    pub combo: Option<String>,
    pub finish_order: Option<i64>,
    pub refund: Option<i64>,
    pub popularity: Option<i64>,
    pub votes: Option<String>,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OddsRow {
    pub bet_type: String,
    pub seq: i64,
    pub combo: String,
    pub odds: String,
    pub sort_odds: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OddsMeta {
    pub sales: Value,
    pub expect: Value,
    pub pop: Value,
    pub player_stats: Value,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RaceDetail {
    pub hold: Option<Hold>,
    pub infos: Vec<RaceInfo>,
    pub entries: Vec<Entry>,
    pub results: Vec<RaceResultRow>,
    pub refunds: Vec<RefundRow>,
    pub odds: Vec<OddsRow>,
    pub odds_meta: Option<OddsMeta>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerStatGroup {
    pub label: String,
    pub races: i64,
    pub first: i64,
    pub top2: i64,
    pub top3: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PlayerStats {
    pub races: i64,
    pub first: i64,
    pub second: i64,
    pub third: i64,
    pub top2: i64,
    pub top3: i64,
    pub avg_st: Option<f64>,
    pub avg_time: Option<f64>,
    pub by_place: Vec<PlayerStatGroup>,
    pub by_car: Vec<PlayerStatGroup>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerRow {
    pub player_code: String,
    pub player_name: String,
    pub player_name_kana: String,
    pub place_code: Option<i64>,
    pub place_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerHistoryRow {
    pub race_date: String,
    pub place_code: i64,
    pub race_no: i64,
    pub ord: Option<i64>,
    pub car_no: Option<i64>,
    pub race_time: Option<String>,
    pub st: Option<String>,
    pub trial_time: Option<String>,
    pub accident_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RaceSearchQuery {
    pub player: Option<String>,
    pub place_code: Option<i64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub ord_min: Option<i64>,
    pub ord_max: Option<i64>,
    pub bet_type: Option<String>,
    pub min_refund: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RaceSearchRow {
    pub place_code: i64,
    pub place_name: String,
    pub race_date: String,
    pub race_no: i64,
    pub title: String,
    pub grade_name: String,
    pub ord: i64,
    pub car_no: Option<i64>,
    pub player_code: String,
    pub player_name: String,
    pub race_time: Option<String>,
    pub st: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectReport {
    pub place_code: i64,
    pub race_date: String,
    pub races_attempted: i64,
    pub programs_saved: i64,
    pub entries_saved: i64,
    pub results_saved: i64,
    pub infos_saved: i64,
    pub odds_saved: i64,
    pub errors: Vec<String>,
}
