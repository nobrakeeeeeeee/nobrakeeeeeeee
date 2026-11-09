use crate::api::ApiError;
use crate::db::Db;
use crate::model::*;
use chrono::NaiveDate;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, State};

pub struct AppState {
    pub api: crate::api::ApiClient,
    pub db: Mutex<Db>,
    pub cancel: AtomicBool,
}

fn sval(v: &Value, key: &str) -> String {
    v.get(key)
        .map(|x| match x {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        })
        .unwrap_or_default()
}

fn ival(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0)
}

fn oval(v: &Value, key: &str) -> Option<String> {
    let s = sval(v, key);
    if s.is_empty() { None } else { Some(s) }
}

fn date_plus(date: &str, days: i64) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_signed(chrono::Duration::days(days)))
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| date.to_string())
}

fn hold_from_today(v: &Value) -> Option<Hold> {
    let period_start = sval(v, "periodStartDate");
    let paragraphday = ival(v, "paragraphday");
    if period_start.is_empty() || paragraphday == 0 {
        return None;
    }
    let race_date = date_plus(&period_start, paragraphday - 1);
    Some(Hold {
        place_code: ival(v, "placeCode"),
        race_date,
        place_key: sval(v, "placeKey"),
        place_name: sval(v, "placeName"),
        period_start,
        period_end: sval(v, "periodEndDate"),
        paragraphday,
        grade_name: sval(v, "gradeName"),
        title: sval(v, "title"),
        title_csv: sval(v, "titleCsv"),
        nighter_name: oval(v, "nighterName"),
        race_name: sval(v, "raceName"),
        race_class: sval(v, "raceClass"),
        distance: v.get("distance").and_then(|d| d.as_i64()),
        live_start_time: oval(v, "liveStartTime"),
        live_end_time: oval(v, "liveEndTime"),
        race_start_time: oval(v, "raceStartTime"),
        now_race_no: v.get("nowRaceNo").and_then(|d| d.as_i64()),
        result_race_no: v.get("resultRaceNo").and_then(|d| d.as_i64()),
    })
}

/// Build a Hold row from an OtherRaceInfo response body.
fn hold_from_race_info(body: &Value, race_date: &str) -> Option<Hold> {
    let period_start = sval(body, "periodStartDate");
    if period_start.is_empty() {
        return None;
    }
    let paragraphday = ival(body, "paragraphday");
    Some(Hold {
        place_code: ival(body, "placeCode"),
        race_date: race_date.to_string(),
        place_key: sval(body, "placeKey"),
        place_name: sval(body, "placeName"),
        period_start: period_start.clone(),
        period_end: sval(body, "periodEndDate"),
        paragraphday,
        grade_name: sval(body, "gradeName"),
        title: sval(body, "title"),
        title_csv: String::new(),
        nighter_name: None,
        race_name: sval(body, "raceName"),
        race_class: sval(body, "classCode"),
        distance: body.get("distance").and_then(|d| d.as_i64()),
        live_start_time: None,
        live_end_time: None,
        race_start_time: oval(body, "raceStartTime"),
        now_race_no: None,
        result_race_no: None,
    })
}

#[tauri::command]
pub async fn today_holds(state: State<'_, AppState>) -> Result<Vec<Hold>, String> {
    let body = state.api.get("/race_info/XML/Hold/Today").await.map_err(|e| e.to_string())?;
    let mut holds = Vec::new();
    if let Some(list) = body.get("today").and_then(|v| v.as_array()) {
        for item in list {
            if let Some(h) = hold_from_today(item) {
                let _ = state.db.lock().unwrap().upsert_hold(&h);
                holds.push(h);
            }
        }
    }
    Ok(holds)
}

/// Fetch Hold/Recent for a place and upsert one hold row per race day.
async fn fetch_recent_holds(
    api: &crate::api::ApiClient,
    db: &Mutex<Db>,
    place_code: i64,
) -> Result<Vec<Hold>, String> {
    let body = api
        .post("/race_info/XML/Hold/Recent", &json!({ "placeCode": place_code }))
        .await
        .map_err(|e| e.to_string())?;
    let mut holds = Vec::new();
    if let Some(list) = body.as_array() {
        for h in list {
            let period_start = sval(h, "periodStartDate");
            let period_end = sval(h, "periodEndDate");
            if let Some(days) = h.get("raceList").and_then(|v| v.as_array()) {
                for d in days {
                    let race_date = sval(d, "raceDate");
                    if race_date.is_empty() {
                        continue;
                    }
                    let paragraphday = NaiveDate::parse_from_str(&race_date, "%Y-%m-%d")
                        .ok()
                        .zip(NaiveDate::parse_from_str(&period_start, "%Y-%m-%d").ok())
                        .map(|(a, b)| (a - b).num_days() + 1)
                        .unwrap_or(0);
                    let hold = Hold {
                        place_code: ival(h, "placeCode"),
                        race_date,
                        place_key: sval(h, "placeKey"),
                        place_name: sval(h, "placeName"),
                        period_start: period_start.clone(),
                        period_end: period_end.clone(),
                        paragraphday,
                        grade_name: String::new(),
                        title: sval(h, "title"),
                        title_csv: String::new(),
                        nighter_name: None,
                        race_name: String::new(),
                        race_class: String::new(),
                        distance: None,
                        live_start_time: None,
                        live_end_time: None,
                        race_start_time: None,
                        now_race_no: None,
                        result_race_no: d.get("finalRaceNo").and_then(|v| v.as_i64()),
                    };
                    let _ = db.lock().unwrap().upsert_hold(&hold);
                    holds.push(hold);
                }
            }
        }
    }
    Ok(holds)
}

#[tauri::command]
pub async fn recent_holds(
    state: State<'_, AppState>,
    place_code: i64,
) -> Result<Vec<Hold>, String> {
    fetch_recent_holds(&state.api, &state.db, place_code).await
}

#[tauri::command]
pub async fn list_holds(state: State<'_, AppState>, limit: i64) -> Result<Vec<Hold>, String> {
    state.db.lock().unwrap().list_holds(limit)
}

#[tauri::command]
pub async fn race_numbers(
    state: State<'_, AppState>,
    place_code: i64,
    race_date: String,
) -> Result<Vec<i64>, String> {
    state.db.lock().unwrap().race_numbers_for_day(place_code, &race_date)
}

#[tauri::command]
pub async fn race_detail(
    state: State<'_, AppState>,
    place_code: i64,
    race_date: String,
    race_no: i64,
) -> Result<RaceDetail, String> {
    let db = state.db.lock().unwrap();
    Ok(RaceDetail {
        hold: db.hold_for_day(place_code, &race_date)?,
        infos: db.race_infos_for_day(place_code, &race_date)?,
        entries: db.entries_for_race(place_code, &race_date, race_no)?,
        results: db.results_for_race(place_code, &race_date, race_no)?,
        refunds: db.refunds_for_race(place_code, &race_date, race_no)?,
        odds: db.odds_for_race(place_code, &race_date, race_no)?,
        odds_meta: db.odds_meta_for_race(place_code, &race_date, race_no)?,
    })
}

/// Collect one day (all races) of program/result/race-info data.
async fn collect_one_day(
    app: &tauri::AppHandle,
    state: &AppState,
    place_code: i64,
    race_date: &str,
) -> Result<CollectReport, String> {
    let race_date = race_date.to_string();
    let mut report = CollectReport {
        place_code,
        race_date: race_date.clone(),
        races_attempted: 0,
        programs_saved: 0,
        entries_saved: 0,
        results_saved: 0,
        infos_saved: 0,
        odds_saved: 0,
        errors: Vec::new(),
    };

    let emit = |msg: String| {
        let _ = app.emit("collect-log", msg.clone());
        msg
    };

    // First fetch OtherRaceInfo for race 1 to learn finalRaceNo and hold meta.
    let mut final_race_no = 12i64;
    match state
        .api
        .post(
            "/race_info/OtherRaceInfo",
            &json!({ "placeCode": place_code, "raceDate": race_date, "raceNo": 1 }),
        )
        .await
    {
        Ok(info) => {
            if let Some(f) = info.get("finalRaceNo").and_then(|v| v.as_i64()) {
                if f > 0 {
                    final_race_no = f;
                }
            }
            let _ = state.db.lock().unwrap().save_race_info(place_code, &race_date, 1, &info);
            report.infos_saved += 1;
            if let Some(h) = hold_from_race_info(&info, &race_date) {
                let _ = state.db.lock().unwrap().upsert_hold(&h);
            }
        }
        Err(ApiError::Api { code, .. }) if code == "4101" => {
            emit(format!("{race_date}: no race info (not held?)"));
            return Ok(report);
        }
        Err(e) => {
            report.errors.push(format!("OtherRaceInfo R1: {e}"));
        }
    }

    for race_no in 1..=final_race_no {
        if state.cancel.load(Ordering::Relaxed) {
            emit("collection cancelled".to_string());
            break;
        }
        report.races_attempted += 1;

        // Program (entries + stats)
        match state
            .api
            .post(
                "/race_info/Program",
                &json!({ "placeCode": place_code, "raceDate": race_date, "raceNo": race_no }),
            )
            .await
        {
            Ok(body) => {
                let n = state
                    .db
                    .lock()
                    .unwrap()
                    .save_program(place_code, &race_date, race_no, &body)?;
                report.programs_saved += 1;
                report.entries_saved += n as i64;
                emit(format!("R{race_no}: program ok ({n} entries)"));
            }
            Err(ApiError::Api { code, message }) if code == "4101" => {
                emit(format!("R{race_no}: no program ({message}), stop"));
                break;
            }
            Err(e) => {
                report.errors.push(format!("Program R{race_no}: {e}"));
                emit(format!("R{race_no}: program error {e}"));
            }
        }

        // RaceInfo for races > 1
        if race_no > 1 {
            match state
                .api
                .post(
                    "/race_info/OtherRaceInfo",
                    &json!({ "placeCode": place_code, "raceDate": race_date, "raceNo": race_no }),
                )
                .await
            {
                Ok(info) => {
                    state
                        .db
                        .lock()
                        .unwrap()
                        .save_race_info(place_code, &race_date, race_no, &info)?;
                    report.infos_saved += 1;
                }
                Err(ApiError::Api { code, .. }) if code == "4101" => {}
                Err(e) => report.errors.push(format!("OtherRaceInfo R{race_no}: {e}")),
            }
        }

        // RaceResult (may not exist yet for upcoming races)
        match state
            .api
            .post(
                "/race_info/RaceResult",
                &json!({ "placeCode": place_code, "raceDate": race_date, "raceNo": race_no }),
            )
            .await
        {
            Ok(body) => {
                let n = state
                    .db
                    .lock()
                    .unwrap()
                    .save_race_result(place_code, &race_date, race_no, &body)?;
                state
                    .db
                    .lock()
                    .unwrap()
                    .save_race_refunds(place_code, &race_date, race_no, &body)?;
                if n > 0 {
                    report.results_saved += n as i64;
                    emit(format!("R{race_no}: result ok ({n} rows)"));
                }
            }
            Err(ApiError::Api { code, .. }) if code == "4101" => {
                emit(format!("R{race_no}: no result yet"));
            }
            Err(e) => report.errors.push(format!("RaceResult R{race_no}: {e}")),
        }

        // Odds (available for ~1 month after the race; 4101 = not held / expired)
        match state
            .api
            .post(
                "/race_info/Odds",
                &json!({ "placeCode": place_code, "raceDate": race_date, "raceNo": race_no }),
            )
            .await
        {
            Ok(body) => {
                let n = state
                    .db
                    .lock()
                    .unwrap()
                    .save_race_odds(place_code, &race_date, race_no, &body)?;
                report.odds_saved += n as i64;
                emit(format!("R{race_no}: odds ok ({n} combos)"));
            }
            Err(ApiError::Api { code, .. }) if code == "4101" => {
                emit(format!("R{race_no}: no odds"));
            }
            Err(e) => report.errors.push(format!("Odds R{race_no}: {e}")),
        }
    }

    Ok(report)
}

#[tauri::command]
pub async fn collect_day(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    place_code: i64,
    race_date: String,
) -> Result<CollectReport, String> {
    state.cancel.store(false, Ordering::Relaxed);
    collect_one_day(&app, state.inner(), place_code, &race_date).await
}

/// Collect every calendar day in [start_date, end_date] for a place.
/// Days without a hold cost a single OtherRaceInfo probe request.
#[tauri::command]
pub async fn collect_range(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    place_code: i64,
    start_date: String,
    end_date: String,
) -> Result<Value, String> {
    state.cancel.store(false, Ordering::Relaxed);
    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|_| format!("invalid start_date: {start_date}"))?;
    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|_| format!("invalid end_date: {end_date}"))?;
    if end < start {
        return Err("end_date is before start_date".to_string());
    }
    let today = chrono::Local::now().date_naive();
    let last = end.min(today - chrono::Duration::days(1));

    let mut days_total = 0i64;
    let mut days_done = 0i64;
    let mut days_skipped = 0i64;
    let mut errors: Vec<String> = Vec::new();

    let mut d = start;
    while d <= last {
        if state.cancel.load(Ordering::Relaxed) {
            let _ = app.emit("collect-log", "collection cancelled by user".to_string());
            break;
        }
        let date = d.format("%Y-%m-%d").to_string();
        days_total += 1;
        let _ = app.emit("collect-log", format!("=== {date} ==="));

        match collect_one_day(&app, state.inner(), place_code, &date).await {
            Ok(r) => {
                if r.programs_saved == 0 && r.entries_saved == 0 {
                    days_skipped += 1;
                } else {
                    days_done += 1;
                }
                let _ = app.emit(
                    "collect-log",
                    format!(
                        "--- {date} done: {} programs, {} entries, {} results ---",
                        r.programs_saved, r.entries_saved, r.results_saved
                    ),
                );
            }
            Err(e) => {
                errors.push(format!("{date}: {e}"));
                let _ = app.emit("collect-log", format!("{date}: error {e}"));
            }
        }
        d += chrono::Duration::days(1);
    }

    Ok(json!({
        "days_total": days_total,
        "days_done": days_done,
        "days_skipped": days_skipped,
        "cancelled": state.cancel.load(Ordering::Relaxed),
        "errors": errors,
    }))
}

/// Collect every race day returned by Hold/Recent for one place, or all
/// five autorace venues when `place_code` is None.
#[tauri::command]
pub async fn collect_recent(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    place_code: Option<i64>,
) -> Result<Value, String> {
    state.cancel.store(false, Ordering::Relaxed);
    let places: Vec<i64> = place_code.map(|c| vec![c]).unwrap_or_else(|| vec![2, 3, 4, 5, 6]);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut days_total = 0i64;
    let mut days_done = 0i64;
    let mut errors: Vec<String> = Vec::new();

    'places: for pc in places {
        let holds = match fetch_recent_holds(&state.api, &state.db, pc).await {
            Ok(h) => h,
            Err(e) => {
                errors.push(format!("place {pc} recent holds: {e}"));
                let _ = app.emit("collect-log", format!("place {pc}: failed to list holds ({e})"));
                continue;
            }
        };
        let mut seen = std::collections::HashSet::new();
        for h in holds {
            // Skip today and future dates; programs may not exist yet.
            if h.race_date >= today || !seen.insert(h.race_date.clone()) {
                continue;
            }
            days_total += 1;
            let label = format!("{}/{} {}", h.place_name, h.race_date, h.title);
            let _ = app.emit("collect-log", format!("=== {label} ==="));

            if state.cancel.load(Ordering::Relaxed) {
                let _ = app.emit("collect-log", "collection cancelled by user".to_string());
                break 'places;
            }

            match collect_one_day(&app, state.inner(), h.place_code, &h.race_date).await {
                Ok(r) => {
                    days_done += 1;
                    let _ = app.emit(
                        "collect-log",
                        format!(
                            "--- {} done: {} programs, {} entries, {} results ---",
                            r.race_date, r.programs_saved, r.entries_saved, r.results_saved
                        ),
                    );
                }
                Err(e) => {
                    errors.push(format!("{}: {e}", h.race_date));
                    let _ = app.emit("collect-log", format!("{}: error {e}", h.race_date));
                }
            }
        }
    }

    Ok(json!({
        "days_total": days_total,
        "days_done": days_done,
        "cancelled": state.cancel.load(Ordering::Relaxed),
        "errors": errors,
    }))
}

#[tauri::command]
pub fn cancel_collection(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub async fn sync_players(state: State<'_, AppState>) -> Result<usize, String> {
    let body = state.api.get("/race_info/Players").await.map_err(|e| e.to_string())?;
    state.db.lock().unwrap().save_players(&body)
}

#[tauri::command]
pub async fn search_players(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<PlayerRow>, String> {
    state.db.lock().unwrap().search_players(&query)
}

#[tauri::command]
pub async fn player_detail(
    state: State<'_, AppState>,
    player_code: String,
) -> Result<Value, String> {
    let profile = state
        .api
        .post("/race_info/Profile", &json!({ "playerCode": player_code }))
        .await
        .map_err(|e| e.to_string())?;
    let history = state
        .db
        .lock()
        .unwrap()
        .player_history(&player_code, 50)
        .unwrap_or_default();
    let stats = state
        .db
        .lock()
        .unwrap()
        .player_stats(&player_code)
        .unwrap_or_default();
    Ok(json!({ "profile": profile, "history": history, "stats": stats }))
}

#[tauri::command]
pub async fn db_stats(state: State<'_, AppState>) -> Result<Value, String> {
    let db = state.db.lock().unwrap();
    let holds = db.list_holds(10000)?.len() as i64;
    let players = db.player_count()?;
    Ok(json!({ "holds": holds, "players": players }))
}
