use crate::config::Category;
use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rusqlite::{Connection, params};
use std::collections::HashMap;

pub struct Store {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct ActivityEntry {
    #[allow(dead_code)]
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub app_name: String,
    pub window_title: String,
    pub category: String,
    pub duration_secs: u64,
}

#[derive(Debug, Clone)]
pub struct SwitchEntry {
    #[allow(dead_code)]
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub from_category: String,
    pub to_category: String,
    pub cost_mins: u64,
}

#[derive(Debug, Clone)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub total_tracked: u64,
    pub switch_count: u64,
    pub focus_loss: u64,
    pub focus_score: u64,
    pub by_category: Vec<(String, u64)>,
    pub top_switches: Vec<SwitchEntry>,
}

impl Store {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS activities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                app_name TEXT NOT NULL,
                window_title TEXT NOT NULL,
                category TEXT NOT NULL,
                duration_secs INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_activities_date ON activities(timestamp);
            CREATE INDEX IF NOT EXISTS idx_activities_category ON activities(category);

            CREATE TABLE IF NOT EXISTS switches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                from_category TEXT NOT NULL,
                to_category TEXT NOT NULL,
                cost_mins INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_switches_date ON switches(timestamp);

            CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_ts TEXT NOT NULL,
                end_ts TEXT,
                planned_mins INTEGER NOT NULL,
                interrupted INTEGER NOT NULL DEFAULT 0,
                switch_count INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_focus_date ON focus_sessions(start_ts);
            ",
        )?;
        Ok(Self { conn })
    }

    pub fn insert_activity(
        &self,
        timestamp: NaiveDateTime,
        app_name: &str,
        window_title: &str,
        category: Category,
        duration_secs: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO activities (timestamp, app_name, window_title, category, duration_secs)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                app_name,
                window_title,
                category.as_str(),
                duration_secs
            ],
        )?;
        Ok(())
    }

    pub fn insert_switch(
        &self,
        timestamp: NaiveDateTime,
        from: Category,
        to: Category,
        cost_mins: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO switches (timestamp, from_category, to_category, cost_mins)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                from.as_str(),
                to.as_str(),
                cost_mins
            ],
        )?;
        Ok(())
    }

    pub fn start_focus_session(&self, planned_mins: u32) -> Result<i64> {
        let now = chrono::Local::now().naive_local();
        self.conn.execute(
            "INSERT INTO focus_sessions (start_ts, planned_mins) VALUES (?1, ?2)",
            params![now.format("%Y-%m-%d %H:%M:%S").to_string(), planned_mins],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn end_focus_session(&self, id: i64, interrupted: bool, switch_count: u32) -> Result<()> {
        let now = chrono::Local::now().naive_local();
        self.conn.execute(
            "UPDATE focus_sessions SET end_ts = ?1, interrupted = ?2, switch_count = ?3 WHERE id = ?4",
            params![
                now.format("%Y-%m-%d %H:%M:%S").to_string(),
                interrupted as i32,
                switch_count,
                id
            ],
        )?;
        Ok(())
    }

    pub fn activities_for_date(&self, date: NaiveDate) -> Result<Vec<ActivityEntry>> {
        let start = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end = date.and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap());
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, app_name, window_title, category, duration_secs
             FROM activities
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC",
        )?;
        let rows = stmt.query_map(
            params![
                start.format("%Y-%m-%d %H:%M:%S").to_string(),
                end.format("%Y-%m-%d %H:%M:%S").to_string()
            ],
            |row| {
                let ts: String = row.get(1)?;
                Ok(ActivityEntry {
                    id: row.get(0)?,
                    timestamp: NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")
                        .unwrap_or(start),
                    app_name: row.get(2)?,
                    window_title: row.get(3)?,
                    category: row.get(4)?,
                    duration_secs: row.get(5)?,
                })
            },
        )?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    pub fn switches_for_date(&self, date: NaiveDate) -> Result<Vec<SwitchEntry>> {
        let start = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end = date.and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap());
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, from_category, to_category, cost_mins
             FROM switches
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC",
        )?;
        let rows = stmt.query_map(
            params![
                start.format("%Y-%m-%d %H:%M:%S").to_string(),
                end.format("%Y-%m-%d %H:%M:%S").to_string()
            ],
            |row| {
                let ts: String = row.get(1)?;
                Ok(SwitchEntry {
                    id: row.get(0)?,
                    timestamp: NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")
                        .unwrap_or(start),
                    from_category: row.get(2)?,
                    to_category: row.get(3)?,
                    cost_mins: row.get(4)?,
                })
            },
        )?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    #[allow(dead_code)]
    pub fn last_activity(&self) -> Result<Option<ActivityEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, app_name, window_title, category, duration_secs
             FROM activities
             ORDER BY timestamp DESC
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], |row| {
            let ts: String = row.get(1)?;
            Ok(ActivityEntry {
                id: row.get(0)?,
                timestamp: NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")
                    .unwrap_or(chrono::Local::now().naive_local()),
                app_name: row.get(2)?,
                window_title: row.get(3)?,
                category: row.get(4)?,
                duration_secs: row.get(5)?,
            })
        })?;
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }
}

impl Store {
    /// Longest consecutive focus streak (code + research) for a given date.
    /// Returns seconds.
    pub fn longest_streak_for_date(&self, date: NaiveDate) -> Result<u64> {
        let activities = self.activities_for_date(date)?;
        let switches = self.switches_for_date(date)?;

        if activities.is_empty() {
            return Ok(0);
        }

        // Switch timestamps where focus was broken (to distraction/communication)
        let break_times: Vec<NaiveDateTime> = switches
            .iter()
            .filter(|s| {
                let to = Category::from_str(&s.to_category);
                to.is_focus_breaking()
            })
            .map(|s| s.timestamp)
            .collect();

        let mut max_streak = 0u64;
        let mut current_streak = 0u64;

        for a in &activities {
            let is_focus = a.category == "code" || a.category == "research";
            let was_broken = break_times.iter().any(|t| *t <= a.timestamp);

            if is_focus && !was_broken {
                current_streak += a.duration_secs;
                if current_streak > max_streak {
                    max_streak = current_streak;
                }
            } else {
                current_streak = 0;
            }
        }

        Ok(max_streak)
    }

    /// Streak history for the last N days.
    /// Returns Vec<(date, longest_streak_seconds)>.
    pub fn streak_history(&self, days: u32) -> Result<Vec<(NaiveDate, u64)>> {
        let today = chrono::Local::now().date_naive();
        let mut result = Vec::new();

        for i in (0..days).rev() {
            let date = today - chrono::Duration::days(i as i64);
            let streak = self.longest_streak_for_date(date)?;
            result.push((date, streak));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_store() -> (Store, tempfile::NamedTempFile) {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let store = Store::open(tmp.path()).unwrap();
        (store, tmp)
    }

    #[test]
    fn test_insert_and_query_activity() {
        let (store, _tmp) = tmp_store();
        let date = chrono::Local::now().date_naive();
        let ts = date.and_hms_opt(10, 0, 0).unwrap();
        store
            .insert_activity(ts, "code", "main.rs", Category::Code, 120)
            .unwrap();
        let activities = store.activities_for_date(date).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].app_name, "code");
        assert_eq!(activities[0].duration_secs, 120);
    }

    #[test]
    fn test_insert_and_query_switch() {
        let (store, _tmp) = tmp_store();
        let date = chrono::Local::now().date_naive();
        let ts = date.and_hms_opt(10, 0, 0).unwrap();
        store
            .insert_switch(ts, Category::Code, Category::Distraction, 23)
            .unwrap();
        let switches = store.switches_for_date(date).unwrap();
        assert_eq!(switches.len(), 1);
        assert_eq!(switches[0].from_category, "code");
        assert_eq!(switches[0].to_category, "distraction");
        assert_eq!(switches[0].cost_mins, 23);
    }

    #[test]
    fn test_empty_date_returns_empty() {
        let (store, _tmp) = tmp_store();
        let date = chrono::Local::now().date_naive();
        let activities = store.activities_for_date(date).unwrap();
        assert!(activities.is_empty());
        let switches = store.switches_for_date(date).unwrap();
        assert!(switches.is_empty());
    }

    #[test]
    fn test_focus_session_lifecycle() {
        let (store, _tmp) = tmp_store();
        let id = store.start_focus_session(25).unwrap();
        store.end_focus_session(id, false, 2).unwrap();
    }

    #[test]
    fn test_longest_streak_no_data() {
        let (store, _tmp) = tmp_store();
        let date = chrono::Local::now().date_naive();
        let streak = store.longest_streak_for_date(date).unwrap();
        assert_eq!(streak, 0);
    }
}

impl DailySummary {
    pub fn for_date(store: &Store, date: NaiveDate) -> Result<Self> {
        let activities = store.activities_for_date(date)?;
        let switches = store.switches_for_date(date)?;

        let total_tracked: u64 = activities.iter().map(|a| a.duration_secs).sum();
        let switch_count = switches.len() as u64;
        let focus_loss: u64 = switches.iter().map(|s| s.cost_mins * 60).sum();

        // Focus score: 100 = perfect focus, 0 = constant switching
        // Based on: switch count, focus loss ratio, distraction time ratio
        let focus_score = if total_tracked > 0 {
            let distraction_secs: u64 = activities
                .iter()
                .filter(|a| a.category == "distraction")
                .map(|a| a.duration_secs)
                .sum();
            let distraction_ratio = distraction_secs as f64 / total_tracked as f64;
            let loss_ratio = (focus_loss as f64 / total_tracked as f64).min(1.0);
            let switch_penalty = (switch_count as f64 / 50.0).min(1.0); // 50+ switches = 0

            let score =
                100.0 * (1.0 - 0.4 * distraction_ratio - 0.3 * loss_ratio - 0.3 * switch_penalty);
            score.round() as u64
        } else {
            0
        };

        // By category
        let mut cat_map: HashMap<String, u64> = HashMap::new();
        for a in &activities {
            *cat_map.entry(a.category.clone()).or_insert(0) += a.duration_secs;
        }
        let mut by_category: Vec<(String, u64)> = cat_map.into_iter().collect();
        by_category.sort_by_key(|b| std::cmp::Reverse(b.1));

        // Top switches (most costly)
        let mut top_switches = switches.clone();
        top_switches.sort_by_key(|b| std::cmp::Reverse(b.cost_mins));
        top_switches.truncate(10);

        Ok(Self {
            date,
            total_tracked,
            switch_count,
            focus_loss,
            focus_score,
            by_category,
            top_switches,
        })
    }
}
