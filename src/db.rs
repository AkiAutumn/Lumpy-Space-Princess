use sqlx::{SqlitePool, query, Row};
use chrono::{NaiveDateTime, Utc};
use std::error::Error;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    // Initialize the database connection
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let database_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL"); // Path to the SQLite file
        let pool = SqlitePool::connect(&database_url).await?;

        // Create the table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS suspensions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                moderator_id INTEGER NOT NULL,
                previous_roles TEXT NOT NULL,
                from_datetime TEXT NOT NULL,
                until_datetime TEXT NOT NULL,
                reason TEXT
            )",
        )
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    // Log a suspension to the database
    pub async fn log_suspension(
        &self,
        user_id: i64,
        moderator_id: i64,
        previous_roles: &[String],
        from_datetime: &str,
        until_datetime: &str,
        reason: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO suspensions (user_id, moderator_id, previous_roles, from_datetime, until_datetime, reason)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
            .bind(user_id)
            .bind(moderator_id)
            .bind(previous_roles.join(",")) // Convert Vec<String> to a single comma-separated string
            .bind(from_datetime)
            .bind(until_datetime)
            .bind(reason)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Retrieve active suspensions for a specific user
    pub async fn get_active_suspensions(&self, user_id: i64) -> Result<Vec<Suspension>, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let rows = sqlx::query(
            "SELECT id, user_id, moderator_id, previous_roles, from_datetime, until_datetime, reason
             FROM suspensions WHERE until_datetime > ? AND user_id = ?",
        )
            .bind(&now)
            .bind(&user_id)
            .fetch_all(&self.pool)
            .await?;

        let suspensions = rows
            .into_iter()
            .map(|row| Suspension {
                id: row.get("id"),
                user_id: row.get("user_id"),
                moderator_id: row.get("moderator_id"),
                previous_roles: row.get::<String, _>("previous_roles").split(',').map(String::from).collect(),
                from_datetime: row.get("from_datetime"),
                until_datetime: row.get("until_datetime"),
                reason: row.get("reason"),
            })
            .collect();

        Ok(suspensions)
    }
}

// Struct to map database rows to
#[derive(Debug)]
pub struct Suspension {
    pub id: i64,
    pub user_id: i64,
    pub moderator_id: i64,
    pub previous_roles: Vec<String>,
    pub from_datetime: String,
    pub until_datetime: String,
    pub reason: Option<String>,
}
