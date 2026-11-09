use crate::model::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use std::path::Path;

pub struct Db {
    conn: Connection,
}

fn sv(v: &Value, key: &str) -> String {
    v.get(key)
        .map(|x| match x {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        })
        .unwrap_or_default()
}

fn iv(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0)
}

fn ov(v: &Value, key: &str) -> Option<String> {
    let s = sv(v, key);
    if s.is_empty() { None } else { Some(s) }
}

/// Recursively flatten a nested odds object into (combo, odds_text, sort_value).
/// Leaf values are either a scalar odds string or a {min, max} range object.
fn flatten_odds(node: &Value, prefix: &mut Vec<String>, out: &mut Vec<(String, String, f64)>) {
    match node {
        Value::Object(map) => {
            if map.contains_key("min") || map.contains_key("max") {
                let min = map.get("min").and_then(|v| v.as_str()).unwrap_or("0");
                let max = map.get("max").and_then(|v| v.as_str()).unwrap_or("0");
                let min_f = min.parse::<f64>().unwrap_or(0.0);
                let max_f = max.parse::<f64>().unwrap_or(0.0);
                let sort = if min_f > 0.0 { min_f } else { max_f };
                out.push((prefix.join("-"), format!("{min}-{max}"), sort));
            } else {
                for (k, v) in map {
                    prefix.push(k.clone());
                    flatten_odds(v, prefix, out);
                    prefix.pop();
                }
            }
        }
        Value::String(s) => {
            let num = s.parse::<f64>().unwrap_or(0.0);
            out.push((prefix.join("-"), s.clone(), num));
        }
        Value::Number(n) => {
            let s = n.to_string();
            out.push((prefix.join("-"), s.clone(), n.as_f64().unwrap_or(0.0)));
        }
        _ => {}
    }
}

/// Normalize combo ordering for unordered bet types (rfw, wid).
fn normalize_combo(bet_type: &str, combo: &str) -> String {
    if matches!(bet_type, "rfw" | "wid") {
        let mut parts: Vec<i64> = combo
            .split('-')
            .filter_map(|p| p.parse().ok())
            .collect();
        parts.sort_unstable();
        parts.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("-")
    } else {
        combo.to_string()
    }
}

impl Db {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| e.to_string())?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS holds (
                    place_code INTEGER NOT NULL,
                    race_date TEXT NOT NULL,
                    place_key TEXT NOT NULL,
                    place_name TEXT,
                    period_start TEXT,
                    period_end TEXT,
                    paragraphday INTEGER,
                    grade_name TEXT,
                    title TEXT,
                    title_csv TEXT,
                    nighter_name TEXT,
                    race_name TEXT,
                    race_class TEXT,
                    distance INTEGER,
                    live_start_time TEXT,
                    live_end_time TEXT,
                    race_start_time TEXT,
                    now_race_no INTEGER,
                    result_race_no INTEGER,
                    fetched_at TEXT NOT NULL,
                    PRIMARY KEY (place_code, race_date)
                );
                CREATE TABLE IF NOT EXISTS programs (
                    place_code INTEGER NOT NULL,
                    race_date TEXT NOT NULL,
                    race_no INTEGER NOT NULL,
                    json TEXT NOT NULL,
                    fetched_at TEXT NOT NULL,
                    PRIMARY KEY (place_code, race_date, race_no)
                );
                CREATE TABLE IF NOT EXISTS entries (
                    place_code INTEGER NOT NULL,
                    race_date TEXT NOT NULL,
                    race_no INTEGER NOT NULL,
                    car_no INTEGER NOT NULL,
                    player_code TEXT NOT NULL,
                    player_name TEXT,
                    player_name_kana TEXT,
                    age INTEGER,
                    rank TEXT,
                    handicap INTEGER,
                    bike_class INTEGER,
                    bike_name TEXT,
                    graduation_code TEXT,
                    place_name TEXT,
                    trial_run_time TEXT,
                    absent INTEGER,
                    fetched_at TEXT NOT NULL,
                    PRIMARY KEY (place_code, race_date, race_no, car_no)
                );
                CREATE TABLE IF NOT EXISTS race_results (
                    place_code INTEGER NOT NULL,
                    race_date TEXT NOT NULL,
                    race_no INTEGER NOT NULL,
                    ord INTEGER NOT NULL,
                    car_no INTEGER,
                    player_code TEXT,
                    player_name TEXT,
                    player_name_en TEXT,
                    motorcycle_name TEXT,
                    handicap INTEGER,
                    trial_time TEXT,
                    race_time TEXT,
                    st TEXT,
                    accident_name TEXT,
                    foul_code TEXT,
                    fetched_at TEXT NOT NULL,
                    PRIMARY KEY (place_code, race_date, race_no, ord)
                );
                CREATE TABLE IF NOT EXISTS race_refunds (
                    place_code INTEGER NOT NULL,
                    race_date TEXT NOT NULL,
                    race_no INTEGER NOT NULL,
                    bet_type TEXT NOT NULL,
                    seq INTEGER NOT NULL,
                    combo TEXT,
                    finish_order INTEGER,
                    refund INTEGER,
                    popularity INTEGER,
                    votes TEXT,
                    type_code INTEGER,
                    type_name TEXT,
                    fetched_at TEXT NOT NULL,
                    PRIMARY KEY (place_code, race_date, race_no, bet_type, seq)
                );
                CREATE TABLE IF NOT EXISTS race_infos (
                    place_code INTEGER NOT NULL,
                    race_date TEXT NOT NULL,
                    race_no INTEGER NOT NULL,
                    race_name TEXT,
                    race_class TEXT,
                    distance INTEGER,
                    weather TEXT,
                    race_weather TEXT,
                    temp TEXT,
                    humid TEXT,
                    roadtemp TEXT,
                    situation_code INTEGER,
                    race_start_time TEXT,
                    final_race_no INTEGER,
                    grade_name TEXT,
                    title TEXT,
                    paragraphday INTEGER,
                    fetched_at TEXT NOT NULL,
                    PRIMARY KEY (place_code, race_date, race_no)
                );
                CREATE TABLE IF NOT EXISTS race_odds (
                    place_code INTEGER NOT NULL,
                    race_date TEXT NOT NULL,
                    race_no INTEGER NOT NULL,
                    bet_type TEXT NOT NULL,
                    seq INTEGER NOT NULL,
                    combo TEXT NOT NULL,
                    odds TEXT NOT NULL,
                    sort_odds REAL NOT NULL,
                    fetched_at TEXT NOT NULL,
                    PRIMARY KEY (place_code, race_date, race_no, bet_type, combo)
                );
                CREATE TABLE IF NOT EXISTS race_odds_meta (
                    place_code INTEGER NOT NULL,
                    race_date TEXT NOT NULL,
                    race_no INTEGER NOT NULL,
                    sales_json TEXT,
                    expect_json TEXT,
                    pop_json TEXT,
                    player_stats_json TEXT,
                    comment TEXT,
                    fetched_at TEXT NOT NULL,
                    PRIMARY KEY (place_code, race_date, race_no)
                );
                CREATE TABLE IF NOT EXISTS players (
                    player_code TEXT PRIMARY KEY,
                    player_name TEXT,
                    player_name_kana TEXT,
                    place_code INTEGER,
                    place_key TEXT,
                    group_key TEXT,
                    fetched_at TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_entries_player ON entries(player_code);
                CREATE INDEX IF NOT EXISTS idx_results_player ON race_results(player_code);
                CREATE INDEX IF NOT EXISTS idx_holds_date ON holds(race_date);
                ",
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn now() -> String {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    // ---------- holds ----------

    pub fn upsert_hold(&self, h: &Hold) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO holds (place_code, race_date, place_key, place_name,
                    period_start, period_end, paragraphday, grade_name, title, title_csv,
                    nighter_name, race_name, race_class, distance, live_start_time,
                    live_end_time, race_start_time, now_race_no, result_race_no, fetched_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)
                 ON CONFLICT(place_code, race_date) DO UPDATE SET
                    place_key=excluded.place_key, place_name=excluded.place_name,
                    period_start=excluded.period_start, period_end=excluded.period_end,
                    paragraphday=excluded.paragraphday, grade_name=excluded.grade_name,
                    title=excluded.title, title_csv=excluded.title_csv,
                    nighter_name=excluded.nighter_name, race_name=excluded.race_name,
                    race_class=excluded.race_class, distance=excluded.distance,
                    live_start_time=excluded.live_start_time, live_end_time=excluded.live_end_time,
                    race_start_time=excluded.race_start_time, now_race_no=excluded.now_race_no,
                    result_race_no=excluded.result_race_no, fetched_at=excluded.fetched_at",
                params![
                    h.place_code, h.race_date, h.place_key, h.place_name,
                    h.period_start, h.period_end, h.paragraphday, h.grade_name, h.title,
                    h.title_csv, h.nighter_name, h.race_name, h.race_class, h.distance,
                    h.live_start_time, h.live_end_time, h.race_start_time,
                    h.now_race_no, h.result_race_no, Self::now(),
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_holds(&self, limit: i64) -> Result<Vec<Hold>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT place_code, race_date, place_key, place_name, period_start, period_end,
                        paragraphday, grade_name, title, title_csv, nighter_name, race_name,
                        race_class, distance, live_start_time, live_end_time, race_start_time,
                        now_race_no, result_race_no
                 FROM holds ORDER BY race_date DESC, place_code LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit], |r| {
                Ok(Hold {
                    place_code: r.get(0)?,
                    race_date: r.get(1)?,
                    place_key: r.get(2)?,
                    place_name: r.get(3)?,
                    period_start: r.get(4)?,
                    period_end: r.get(5)?,
                    paragraphday: r.get::<_, Option<i64>>(6)?.unwrap_or(0),
                    grade_name: r.get(7)?,
                    title: r.get(8)?,
                    title_csv: r.get(9)?,
                    nighter_name: r.get(10)?,
                    race_name: r.get(11)?,
                    race_class: r.get(12)?,
                    distance: r.get(13)?,
                    live_start_time: r.get(14)?,
                    live_end_time: r.get(15)?,
                    race_start_time: r.get(16)?,
                    now_race_no: r.get(17)?,
                    result_race_no: r.get(18)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    // ---------- programs / entries ----------

    /// Store the raw Program API response and its playerList as entries.
    /// Returns number of entries inserted.
    pub fn save_program(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
        body: &Value,
    ) -> Result<usize, String> {
        self.conn
            .execute(
                "INSERT INTO programs (place_code, race_date, race_no, json, fetched_at)
                 VALUES (?1,?2,?3,?4,?5)
                 ON CONFLICT(place_code, race_date, race_no) DO UPDATE SET
                    json=excluded.json, fetched_at=excluded.fetched_at",
                params![place_code, race_date, race_no, body.to_string(), Self::now()],
            )
            .map_err(|e| e.to_string())?;

        let mut count = 0;
        if let Some(list) = body.get("playerList").and_then(|v| v.as_array()) {
            for p in list {
                self.conn
                    .execute(
                        "INSERT INTO entries (place_code, race_date, race_no, car_no,
                            player_code, player_name, player_name_kana, age, rank, handicap,
                            bike_class, bike_name, graduation_code, place_name,
                            trial_run_time, absent, fetched_at)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)
                         ON CONFLICT(place_code, race_date, race_no, car_no) DO UPDATE SET
                            player_code=excluded.player_code, player_name=excluded.player_name,
                            player_name_kana=excluded.player_name_kana, age=excluded.age,
                            rank=excluded.rank, handicap=excluded.handicap,
                            bike_class=excluded.bike_class, bike_name=excluded.bike_name,
                            graduation_code=excluded.graduation_code,
                            place_name=excluded.place_name,
                            trial_run_time=excluded.trial_run_time, absent=excluded.absent,
                            fetched_at=excluded.fetched_at",
                        params![
                            place_code,
                            race_date,
                            race_no,
                            iv(p, "carNo"),
                            sv(p, "playerCode"),
                            sv(p, "playerName"),
                            String::new(),
                            iv(p, "age"),
                            sv(p, "rank"),
                            iv(p, "handicap"),
                            iv(p, "bikeClass"),
                            sv(p, "bikeName"),
                            sv(p, "graduationCode"),
                            sv(p, "placeName"),
                            sv(p, "trialRunTime"),
                            p.get("absent").map(|a| if a.is_null() { 0 } else { 1 }).unwrap_or(0),
                            Self::now(),
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
        Ok(count)
    }

    // ---------- race results ----------

    pub fn save_race_result(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
        body: &Value,
    ) -> Result<usize, String> {
        let mut count = 0;
        if let Some(list) = body.get("raceResult").and_then(|v| v.as_array()) {
            for r in list {
                // API uses misspelled "traialTime"; accept both.
                let trial = ov(r, "trialTime").or_else(|| ov(r, "traialTime")).unwrap_or_default();
                self.conn
                    .execute(
                        "INSERT INTO race_results (place_code, race_date, race_no, ord,
                            car_no, player_code, player_name, player_name_en,
                            motorcycle_name, handicap, trial_time, race_time, st,
                            accident_name, foul_code, fetched_at)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
                         ON CONFLICT(place_code, race_date, race_no, ord) DO UPDATE SET
                            car_no=excluded.car_no, player_code=excluded.player_code,
                            player_name=excluded.player_name, player_name_en=excluded.player_name_en,
                            motorcycle_name=excluded.motorcycle_name, handicap=excluded.handicap,
                            trial_time=excluded.trial_time, race_time=excluded.race_time,
                            st=excluded.st, accident_name=excluded.accident_name,
                            foul_code=excluded.foul_code, fetched_at=excluded.fetched_at",
                        params![
                            place_code,
                            race_date,
                            race_no,
                            iv(r, "order"),
                            iv(r, "carNo"),
                            sv(r, "playerCode"),
                            sv(r, "playerName"),
                            sv(r, "playerNameEn"),
                            sv(r, "motorcycleName"),
                            iv(r, "handicap"),
                            trial,
                            sv(r, "raceTime"),
                            sv(r, "st"),
                            ov(r, "accidentName"),
                            sv(r, "foulCode"),
                            Self::now(),
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Persist refundInfo (payouts for every bet type) from a RaceResult body.
    pub fn save_race_refunds(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
        body: &Value,
    ) -> Result<usize, String> {
        let mut count = 0;
        let Some(info) = body.get("refundInfo").and_then(|v| v.as_object()) else {
            return Ok(0);
        };
        for (bet_type, entry) in info {
            let (type_code, type_name, items) = match entry {
                Value::Object(o) => (
                    o.get("typeCode").and_then(|v| v.as_i64()),
                    o.get("typeName").and_then(|v| v.as_str()).map(String::from),
                    o.get("list").and_then(|v| v.as_array()),
                ),
                Value::Array(a) => (None, None, Some(a)),
                _ => (None, None, None),
            };
            let Some(items) = items else { continue };
            for (seq, item) in items.iter().enumerate() {
                let mut combo_parts: Vec<String> = Vec::new();
                for key in ["carNo", "1thCarNo", "2thCarNo", "3thCarNo"] {
                    if let Some(v) = item.get(key) {
                        let s = match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => continue,
                        };
                        if !s.is_empty() {
                            combo_parts.push(s);
                        }
                    }
                }
                let combo = if combo_parts.is_empty() {
                    None
                } else {
                    Some(combo_parts.join("-"))
                };
                let finish_order = item
                    .get("order")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
                self.conn
                    .execute(
                        "INSERT INTO race_refunds (place_code, race_date, race_no,
                            bet_type, seq, combo, finish_order, refund, popularity,
                            votes, type_code, type_name, fetched_at)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
                         ON CONFLICT(place_code, race_date, race_no, bet_type, seq)
                         DO UPDATE SET
                            combo=excluded.combo, finish_order=excluded.finish_order,
                            refund=excluded.refund, popularity=excluded.popularity,
                            votes=excluded.votes, type_code=excluded.type_code,
                            type_name=excluded.type_name, fetched_at=excluded.fetched_at",
                        params![
                            place_code,
                            race_date,
                            race_no,
                            bet_type,
                            seq as i64,
                            combo,
                            finish_order,
                            item.get("refund").and_then(|v| v.as_i64()),
                            item.get("pop").and_then(|v| v.as_i64()),
                            ov(item, "refundVotes"),
                            type_code,
                            type_name,
                            Self::now(),
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn refunds_for_race(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
    ) -> Result<Vec<RefundRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT bet_type, seq, combo, finish_order, refund, popularity,
                        votes, type_name
                 FROM race_refunds
                 WHERE place_code=?1 AND race_date=?2 AND race_no=?3
                 ORDER BY bet_type, seq",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![place_code, race_date, race_no], |r| {
                Ok(RefundRow {
                    bet_type: r.get(0)?,
                    seq: r.get(1)?,
                    combo: r.get(2)?,
                    finish_order: r.get(3)?,
                    refund: r.get(4)?,
                    popularity: r.get(5)?,
                    votes: r.get(6)?,
                    type_name: r.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    // ---------- race odds ----------

    /// Persist odds for all bet types plus per-race meta (sales/expect/pop/player stats).
    /// Returns the number of odds rows written.
    pub fn save_race_odds(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
        body: &Value,
    ) -> Result<usize, String> {
        const BET_TYPES: [&str; 7] = ["tns", "fns", "rtw", "rfw", "rt3", "rf3", "wid"];
        let mut count = 0;
        for bet_type in BET_TYPES {
            let key = format!("{}OddsList", bet_type);
            let Some(list) = body.get(&key) else { continue };
            let mut flat = Vec::new();
            flatten_odds(list, &mut Vec::new(), &mut flat);
            flat.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
            for (seq, (combo, odds, sort)) in flat.iter().enumerate() {
                let combo = normalize_combo(bet_type, combo);
                self.conn
                    .execute(
                        "INSERT INTO race_odds (place_code, race_date, race_no, bet_type,
                            seq, combo, odds, sort_odds, fetched_at)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                         ON CONFLICT(place_code, race_date, race_no, bet_type, combo)
                         DO UPDATE SET seq=excluded.seq, odds=excluded.odds,
                            sort_odds=excluded.sort_odds, fetched_at=excluded.fetched_at",
                        params![
                            place_code, race_date, race_no, bet_type,
                            seq as i64, combo, odds, sort, Self::now(),
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                count += 1;
            }
        }

        let json_or_null = |k: &str| -> Option<String> {
            body.get(k).map(|v| v.to_string())
        };
        self.conn
            .execute(
                "INSERT INTO race_odds_meta (place_code, race_date, race_no,
                    sales_json, expect_json, pop_json, player_stats_json, comment, fetched_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                 ON CONFLICT(place_code, race_date, race_no) DO UPDATE SET
                    sales_json=excluded.sales_json, expect_json=excluded.expect_json,
                    pop_json=excluded.pop_json, player_stats_json=excluded.player_stats_json,
                    comment=excluded.comment, fetched_at=excluded.fetched_at",
                params![
                    place_code,
                    race_date,
                    race_no,
                    json_or_null("salesInfo"),
                    json_or_null("expectInfo"),
                    json_or_null("popInfo"),
                    json_or_null("playerList"),
                    ov(body, "comment"),
                    Self::now(),
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(count)
    }

    pub fn odds_for_race(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
    ) -> Result<Vec<OddsRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT bet_type, seq, combo, odds, sort_odds
                 FROM race_odds
                 WHERE place_code=?1 AND race_date=?2 AND race_no=?3
                 ORDER BY bet_type, sort_odds, combo",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![place_code, race_date, race_no], |r| {
                Ok(OddsRow {
                    bet_type: r.get(0)?,
                    seq: r.get(1)?,
                    combo: r.get(2)?,
                    odds: r.get(3)?,
                    sort_odds: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn odds_meta_for_race(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
    ) -> Result<Option<OddsMeta>, String> {
        let row = self
            .conn
            .query_row(
                "SELECT sales_json, expect_json, pop_json, player_stats_json, comment
                 FROM race_odds_meta
                 WHERE place_code=?1 AND race_date=?2 AND race_no=?3",
                params![place_code, race_date, race_no],
                |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((sales, expect, pop, stats, comment)) = row else {
            return Ok(None);
        };
        let parse = |s: Option<String>| -> Value {
            s.and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null)
        };
        Ok(Some(OddsMeta {
            sales: parse(sales),
            expect: parse(expect),
            pop: parse(pop),
            player_stats: parse(stats),
            comment,
        }))
    }

    // ---------- race info ----------

    pub fn save_race_info(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
        body: &Value,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO race_infos (place_code, race_date, race_no, race_name, race_class,
                    distance, weather, race_weather, temp, humid, roadtemp, situation_code,
                    race_start_time, final_race_no, grade_name, title, paragraphday, fetched_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)
                 ON CONFLICT(place_code, race_date, race_no) DO UPDATE SET
                    race_name=excluded.race_name, race_class=excluded.race_class,
                    distance=excluded.distance, weather=excluded.weather,
                    race_weather=excluded.race_weather, temp=excluded.temp,
                    humid=excluded.humid, roadtemp=excluded.roadtemp,
                    situation_code=excluded.situation_code,
                    race_start_time=excluded.race_start_time,
                    final_race_no=excluded.final_race_no, grade_name=excluded.grade_name,
                    title=excluded.title, paragraphday=excluded.paragraphday,
                    fetched_at=excluded.fetched_at",
                params![
                    place_code,
                    race_date,
                    race_no,
                    sv(body, "raceName"),
                    sv(body, "classCode"),
                    iv(body, "distance"),
                    sv(body, "weather"),
                    sv(body, "raceWeather"),
                    sv(body, "temp"),
                    sv(body, "humid"),
                    sv(body, "roadtemp"),
                    iv(body, "situationCode"),
                    sv(body, "raceStartTime"),
                    iv(body, "finalRaceNo"),
                    sv(body, "gradeName"),
                    sv(body, "title"),
                    iv(body, "paragraphday"),
                    Self::now(),
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn race_infos_for_day(
        &self,
        place_code: i64,
        race_date: &str,
    ) -> Result<Vec<RaceInfo>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT race_no, race_name, race_class, distance, weather, race_weather,
                        temp, humid, roadtemp, situation_code, race_start_time,
                        final_race_no, grade_name, title, paragraphday
                 FROM race_infos WHERE place_code=?1 AND race_date=?2 ORDER BY race_no",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![place_code, race_date], |r| {
                Ok(RaceInfo {
                    race_no: r.get(0)?,
                    race_name: r.get(1)?,
                    race_class: r.get(2)?,
                    distance: r.get(3)?,
                    weather: r.get(4)?,
                    race_weather: r.get(5)?,
                    temp: r.get(6)?,
                    humid: r.get(7)?,
                    roadtemp: r.get(8)?,
                    situation_code: r.get(9)?,
                    race_start_time: r.get(10)?,
                    final_race_no: r.get(11)?,
                    grade_name: r.get(12)?,
                    title: r.get(13)?,
                    paragraphday: r.get(14)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    // ---------- reads for UI ----------

    pub fn entries_for_race(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
    ) -> Result<Vec<Entry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT car_no, player_code, player_name, player_name_kana, age, rank,
                        handicap, bike_class, bike_name, graduation_code, place_name,
                        trial_run_time, absent
                 FROM entries WHERE place_code=?1 AND race_date=?2 AND race_no=?3
                 ORDER BY car_no",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![place_code, race_date, race_no], |r| {
                Ok(Entry {
                    car_no: r.get(0)?,
                    player_code: r.get(1)?,
                    player_name: r.get(2)?,
                    player_name_kana: r.get(3)?,
                    age: r.get(4)?,
                    rank: r.get(5)?,
                    handicap: r.get(6)?,
                    bike_class: r.get(7)?,
                    bike_name: r.get(8)?,
                    graduation_code: r.get(9)?,
                    place_name: r.get(10)?,
                    trial_run_time: r.get(11)?,
                    absent: r.get(12)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn results_for_race(
        &self,
        place_code: i64,
        race_date: &str,
        race_no: i64,
    ) -> Result<Vec<RaceResultRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT ord, car_no, player_code, player_name, player_name_en,
                        motorcycle_name, handicap, trial_time, race_time, st,
                        accident_name, foul_code
                 FROM race_results WHERE place_code=?1 AND race_date=?2 AND race_no=?3
                 ORDER BY ord",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![place_code, race_date, race_no], |r| {
                Ok(RaceResultRow {
                    ord: r.get(0)?,
                    car_no: r.get(1)?,
                    player_code: r.get(2)?,
                    player_name: r.get(3)?,
                    player_name_en: r.get(4)?,
                    motorcycle_name: r.get(5)?,
                    handicap: r.get(6)?,
                    trial_time: r.get(7)?,
                    race_time: r.get(8)?,
                    st: r.get(9)?,
                    accident_name: r.get(10)?,
                    foul_code: r.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    /// All race numbers that have data for a day.
    pub fn race_numbers_for_day(
        &self,
        place_code: i64,
        race_date: &str,
    ) -> Result<Vec<i64>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT race_no FROM (
                    SELECT race_no FROM entries WHERE place_code=?1 AND race_date=?2
                    UNION SELECT race_no FROM race_results WHERE place_code=?1 AND race_date=?2
                    UNION SELECT race_no FROM race_infos WHERE place_code=?1 AND race_date=?2
                 ) ORDER BY race_no",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![place_code, race_date], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    // ---------- players ----------

    /// Upsert players from the /race_info/Players master list.
    /// `body` is the array of {key, playerList:[{playerCode,playerName,playerNameKana,placeCode,placeKey}]}.
    pub fn save_players(&self, body: &Value) -> Result<usize, String> {
        let mut count = 0;
        if let Some(groups) = body.as_array() {
            for g in groups {
                let group_key = sv(g, "key");
                if let Some(list) = g.get("playerList").and_then(|v| v.as_array()) {
                    for p in list {
                        self.conn
                            .execute(
                                "INSERT INTO players (player_code, player_name, player_name_kana,
                                    place_code, place_key, group_key, fetched_at)
                                 VALUES (?1,?2,?3,?4,?5,?6,?7)
                                 ON CONFLICT(player_code) DO UPDATE SET
                                    player_name=excluded.player_name,
                                    player_name_kana=excluded.player_name_kana,
                                    place_code=excluded.place_code, place_key=excluded.place_key,
                                    group_key=excluded.group_key, fetched_at=excluded.fetched_at",
                                params![
                                    sv(p, "playerCode"),
                                    sv(p, "playerName"),
                                    sv(p, "playerNameKana"),
                                    iv(p, "placeCode"),
                                    sv(p, "placeKey"),
                                    group_key,
                                    Self::now(),
                                ],
                            )
                            .map_err(|e| e.to_string())?;
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    pub fn search_players(&self, query: &str) -> Result<Vec<PlayerRow>, String> {
        let like = format!("%{query}%");
        let mut stmt = self
            .conn
            .prepare(
                "SELECT player_code, player_name, player_name_kana, place_code, place_key
                 FROM players
                 WHERE ?1 = '' OR player_code LIKE ?2 OR player_name LIKE ?2 OR player_name_kana LIKE ?2
                 ORDER BY player_code LIMIT 500",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![query, like], |r| {
                Ok(PlayerRow {
                    player_code: r.get(0)?,
                    player_name: r.get(1)?,
                    player_name_kana: r.get(2)?,
                    place_code: r.get(3)?,
                    place_key: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    /// Race history for a player from collected results, newest first.
    pub fn player_history(&self, player_code: &str, limit: i64) -> Result<Vec<PlayerHistoryRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT race_date, place_code, race_no, ord, car_no, race_time, st,
                        trial_time, accident_name
                 FROM race_results WHERE player_code=?1
                 ORDER BY race_date DESC, race_no LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![player_code, limit], |r| {
                Ok(PlayerHistoryRow {
                    race_date: r.get(0)?,
                    place_code: r.get(1)?,
                    race_no: r.get(2)?,
                    ord: r.get(3)?,
                    car_no: r.get(4)?,
                    race_time: r.get(5)?,
                    st: r.get(6)?,
                    trial_time: r.get(7)?,
                    accident_name: r.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn player_count(&self) -> Result<i64, String> {
        self.conn
            .query_row("SELECT COUNT(*) FROM players", [], |r| r.get(0))
            .map_err(|e| e.to_string())
    }

    pub fn hold_for_day(&self, place_code: i64, race_date: &str) -> Result<Option<Hold>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT place_code, race_date, place_key, place_name, period_start, period_end,
                        paragraphday, grade_name, title, title_csv, nighter_name, race_name,
                        race_class, distance, live_start_time, live_end_time, race_start_time,
                        now_race_no, result_race_no
                 FROM holds WHERE place_code=?1 AND race_date=?2",
            )
            .map_err(|e| e.to_string())?;
        stmt.query_row(params![place_code, race_date], |r| {
            Ok(Hold {
                place_code: r.get(0)?,
                race_date: r.get(1)?,
                place_key: r.get(2)?,
                place_name: r.get(3)?,
                period_start: r.get(4)?,
                period_end: r.get(5)?,
                paragraphday: r.get::<_, Option<i64>>(6)?.unwrap_or(0),
                grade_name: r.get(7)?,
                title: r.get(8)?,
                title_csv: r.get(9)?,
                nighter_name: r.get(10)?,
                race_name: r.get(11)?,
                race_class: r.get(12)?,
                distance: r.get(13)?,
                live_start_time: r.get(14)?,
                live_end_time: r.get(15)?,
                race_start_time: r.get(16)?,
                now_race_no: r.get(17)?,
                result_race_no: r.get(18)?,
            })
        })
        .optional()
        .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn program_result_roundtrip() {
        let path = std::env::temp_dir().join(format!("nobrake_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = Db::open(&path).unwrap();

        let program = json!({"playerList": [
            {"carNo": 1, "playerCode": "9037", "playerName": "九門　結衣", "age": 18,
             "rank": "B-110", "handicap": 0, "bikeClass": 2, "bikeName": "リレーション",
             "graduationCode": "39", "placeName": "川　口", "trialRunTime": "", "absent": null}
        ]});
        assert_eq!(db.save_program(2, "2026-11-09", 1, &program).unwrap(), 1);

        let result = json!({"raceResult": [
            {"order": 1, "carNo": 4, "playerCode": "2530", "playerName": "谷島　俊行",
             "playerNameEn": "T", "motorcycleName": "ブリザール", "handicap": 20,
             "traialTime": "3.37", "raceTime": "3.449", "st": "0.11"}
        ]});
        assert_eq!(db.save_race_result(2, "2026-11-09", 1, &result).unwrap(), 1);

        db.save_race_info(2, "2026-11-09", 1, &json!({
            "raceName": "一般戦", "classCode": "1", "distance": 3100, "finalRaceNo": 12
        })).unwrap();

        let entries = db.entries_for_race(2, "2026-11-09", 1).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].player_code, "9037");

        let results = db.results_for_race(2, "2026-11-09", 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].trial_time.as_deref(), Some("3.37"));

        let nos = db.race_numbers_for_day(2, "2026-11-09").unwrap();
        assert_eq!(nos, vec![1]);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn refund_roundtrip() {
        let path = std::env::temp_dir().join(format!("nobrake_refund_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = Db::open(&path).unwrap();

        let body = json!({"refundInfo": {
            "tns": {"typeCode": 0, "typeName": "通常", "list": [
                {"carNo": 4, "order": 1, "refund": 300, "pop": 2, "refundVotes": "188"}
            ]},
            "rtw": {"typeCode": 0, "typeName": "通常", "list": [
                {"1thCarNo": 4, "2thCarNo": 7, "refund": 340, "pop": 2, "refundVotes": "928"}
            ]},
            "wid": {"typeCode": 0, "typeName": "通常", "list": [
                {"1thCarNo": 4, "2thCarNo": 7, "refund": 120, "pop": 1, "refundVotes": "403"},
                {"1thCarNo": 3, "2thCarNo": 4, "refund": 1770, "pop": 17, "refundVotes": "13"}
            ]}
        }});
        assert_eq!(db.save_race_refunds(2, "2026-11-08", 1, &body).unwrap(), 4);

        let refunds = db.refunds_for_race(2, "2026-11-08", 1).unwrap();
        assert_eq!(refunds.len(), 4);
        let rtw = refunds.iter().find(|r| r.bet_type == "rtw").unwrap();
        assert_eq!(rtw.combo.as_deref(), Some("4-7"));
        assert_eq!(rtw.refund, Some(340));
        let tns = refunds.iter().find(|r| r.bet_type == "tns").unwrap();
        assert_eq!(tns.finish_order, Some(1));
        assert_eq!(tns.type_name.as_deref(), Some("通常"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn odds_roundtrip() {
        let path = std::env::temp_dir().join(format!("nobrake_odds_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = Db::open(&path).unwrap();

        let body = json!({
            "tnsOddsList": {"1": "43.9", "4": "12.3"},
            "fnsOddsList": {"1": {"min": "1.0", "max": "1.2"}},
            "rtwOddsList": {"1": {"2": "791.3", "4": "1582"}},
            "rfwOddsList": {"4": {"1": "85.2"}, "1": {"4": "85.2"}},
            "rt3OddsList": {"1": {"2": {"3": "5361"}}},
            "widOddsList": {"4": {"1": {"min": "25.7", "max": "35.9"}}},
            "salesInfo": {"updateDate": "2026-11-10 02:43:24", "rtw": {"salesCount": 4522}},
            "expectInfo": {"sunny": ["7=4-8"]},
            "popInfo": {"rt3": [{"no1": "7", "no2": "4", "no3": "8", "odds": "8.8"}]},
            "playerList": [{"carNo": 1, "stAve": "0.23"}],
            "comment": "テストコメント"
        });
        let n = db.save_race_odds(2, "2026-11-08", 1, &body).unwrap();
        assert_eq!(n, 9);

        let odds = db.odds_for_race(2, "2026-11-08", 1).unwrap();
        // rfw の "1-4"/"4-1" は正規化で同一PKにupsertされるため8行
        assert_eq!(odds.len(), 8);

        // 単勝: sort_odds昇順で "4"(12.3) が "1"(43.9) より先
        let tns: Vec<_> = odds.iter().filter(|o| o.bet_type == "tns").collect();
        assert_eq!(tns[0].combo, "4");
        assert_eq!(tns[0].odds, "12.3");

        // 2連複は無順序正規化で "1-4" に統一される
        let rfw: Vec<_> = odds.iter().filter(|o| o.bet_type == "rfw").collect();
        assert_eq!(rfw.len(), 1);
        assert_eq!(rfw[0].combo, "1-4");

        // ワイドは min-max 表記
        let wid = odds.iter().find(|o| o.bet_type == "wid").unwrap();
        assert_eq!(wid.combo, "1-4");
        assert_eq!(wid.odds, "25.7-35.9");

        // 3連単は順序保持
        let rt3 = odds.iter().find(|o| o.bet_type == "rt3").unwrap();
        assert_eq!(rt3.combo, "1-2-3");

        let meta = db.odds_meta_for_race(2, "2026-11-08", 1).unwrap().unwrap();
        assert_eq!(meta.comment.as_deref(), Some("テストコメント"));
        assert_eq!(meta.sales["rtw"]["salesCount"], json!(4522));
        assert_eq!(meta.expect["sunny"][0], json!("7=4-8"));
        assert_eq!(meta.player_stats[0]["stAve"], json!("0.23"));

        // upsertで重複しない
        db.save_race_odds(2, "2026-11-08", 1, &body).unwrap();
        assert_eq!(db.odds_for_race(2, "2026-11-08", 1).unwrap().len(), 8);

        let _ = std::fs::remove_file(&path);
    }
}
