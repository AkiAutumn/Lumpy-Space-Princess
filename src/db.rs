use sqlx::{SqlitePool, Row};
use chrono::Utc;
use std::error::Error;

pub struct Database {
    pub(crate) pool: SqlitePool,
}

impl Database {
    // Initialize the database connection
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        
        let db_url = "sqlite://database.db?mode=rwc";
        let pool = SqlitePool::connect(db_url).await.expect("Database connection failed");
        
        // Create the table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS suspensions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                guild_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                moderator_id INTEGER NOT NULL,
                previous_roles TEXT NOT NULL,
                from_datetime TEXT NOT NULL,
                until_datetime TEXT NOT NULL,
                reason TEXT,
                active BOOLEAN NOT NULL
            )",
        )
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    // Log a suspension to the database
    pub async fn log_suspension(
        &self,
        suspension: Suspension,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO suspensions (guild_id, user_id, moderator_id, previous_roles, from_datetime, until_datetime, reason, active)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(suspension.guild_id)
            .bind(suspension.user_id)
            .bind(suspension.moderator_id)
            .bind(suspension.previous_roles.join(",")) // Convert Vec<String> to a single comma-separated string
            .bind(suspension.from_datetime)
            .bind(suspension.until_datetime)
            .bind(suspension.reason)
            .bind(true)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Retrieve all suspensions for a specific user
    pub async fn get_suspensions(&self, guild_id: i64, user_id: i64) -> Result<Vec<Suspension>, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let rows = sqlx::query(
            "SELECT id, guild_id, user_id, moderator_id, previous_roles, from_datetime, until_datetime, reason, active
             FROM suspensions WHERE until_datetime > ? AND guild_id = ? AND user_id = ?",
        )
            .bind(&now)
            .bind(guild_id)
            .bind(&user_id)
            .fetch_all(&self.pool)
            .await?;

        let suspensions = rows
            .into_iter()
            .map(|row| Suspension {
                id: row.get("id"),
                guild_id: row.get("guild_id"),
                user_id: row.get("user_id"),
                moderator_id: row.get("moderator_id"),
                previous_roles: row.get::<String, _>("previous_roles").split(',').map(String::from).collect(),
                from_datetime: row.get("from_datetime"),
                until_datetime: row.get("until_datetime"),
                reason: row.get("reason"),
                active: row.get("active"),
            })
            .collect();

        Ok(suspensions)
    }

    // Retrieve all suspensions for a specific user
    pub async fn set_suspension_inactive(&self, suspension_id: i64) {
        sqlx::query("UPDATE suspensions SET ACTIVE = FALSE WHERE id = ?")
            .bind(suspension_id)
            .execute(&self.pool)
            .await
            .ok();
    }

    // Retrieve all active suspensions for a specific user
    pub async fn get_active_suspensions(&self, guild_id: i64, user_id: i64) -> Result<Vec<Suspension>, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let rows = sqlx::query(
            "SELECT id, guild_id, user_id, moderator_id, previous_roles, from_datetime, until_datetime, reason
             FROM suspensions WHERE until_datetime > ? AND guild_id = ? AND user_id = ? AND active = TRUE",
        )
            .bind(&now)
            .bind(guild_id)
            .bind(&user_id)
            .fetch_all(&self.pool)
            .await?;

        let suspensions = rows
            .into_iter()
            .map(|row| Suspension {
                id: row.get("id"),
                guild_id: row.get("guild_id"),
                user_id: row.get("user_id"),
                moderator_id: row.get("moderator_id"),
                previous_roles: row.get::<String, _>("previous_roles").split(',').map(String::from).collect(),
                from_datetime: row.get("from_datetime"),
                until_datetime: row.get("until_datetime"),
                reason: row.get("reason"),
                active: Some(true),
            })
            .collect();

        Ok(suspensions)
    }
}

// Struct to map database rows to
#[derive(Debug)]
pub struct Suspension {
    pub id: i64,
    pub guild_id: i64,
    pub user_id: i64,
    pub moderator_id: i64,
    pub previous_roles: Vec<String>,
    pub from_datetime: String,
    pub until_datetime: String,
    pub reason: Option<String>,
    pub active: Option<bool>,
}
