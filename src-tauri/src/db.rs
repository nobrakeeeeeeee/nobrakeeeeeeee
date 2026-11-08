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
                 WHERE player_code LIKE ?1 OR player_name LIKE ?1 OR player_name_kana LIKE ?1
                 ORDER BY player_code LIMIT 200",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![like], |r| {
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
}
