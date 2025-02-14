use chrono::{DateTime, NaiveDateTime, Utc};

pub fn date_string_to_discord_timestamp(date_string: &str) -> String {
    let datetime = NaiveDateTime::parse_from_str(date_string, "%Y-%m-%d %H:%M:%S%.9f").expect("Failed to parse datetime");
    let datetime_utc: DateTime<Utc> = DateTime::from_naive_utc_and_offset(datetime, Utc);
    format!("<t:{}>", datetime_utc.timestamp())
}