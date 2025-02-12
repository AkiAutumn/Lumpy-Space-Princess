use sqlx::{SqlitePool, query};
use chrono::{NaiveDateTime, Utc};
use std::error::Error;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    // Initialize the database connection
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let database_url = "sqlite://suspensions.db"; // Path to the SQLite file
        let pool = SqlitePool::connect(database_url).await?;

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
        reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO suspensions (user_id, moderator_id, previous_roles, from_datetime, until_datetime, reason)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
            .bind(user_id)
            .bind(moderator_id)
            .bind(previous_roles.join(","))
            .bind(from_datetime)
            .bind(until_datetime)
            .bind(reason)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Retrieve active suspensions
    pub async fn get_active_suspensions(&self) -> Result<Vec<Suspension>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, user_id, moderator_id, previous_roles, from_datetime, until_datetime, reason
             FROM suspensions WHERE until_datetime > ?",
            Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
        )
            .fetch_all(&self.pool)
            .await?;

        let suspensions = rows
            .into_iter()
            .map(|row| Suspension {
                id: row.id,
                user_id: row.user_id,
                moderator_id: row.moderator_id,
                previous_roles: row.previous_roles.split(',').map(String::from).collect(),
                from_datetime: row.from_datetime,
                until_datetime: row.until_datetime,
                reason: row.reason,
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
