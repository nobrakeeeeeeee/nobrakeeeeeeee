use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize)]
pub struct RaceDetail {
    pub hold: Option<Hold>,
    pub infos: Vec<RaceInfo>,
    pub entries: Vec<Entry>,
    pub results: Vec<RaceResultRow>,
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

#[derive(Debug, Clone, Serialize)]
pub struct CollectReport {
    pub place_code: i64,
    pub race_date: String,
    pub races_attempted: i64,
    pub programs_saved: i64,
    pub entries_saved: i64,
    pub results_saved: i64,
    pub infos_saved: i64,
    pub errors: Vec<String>,
}
