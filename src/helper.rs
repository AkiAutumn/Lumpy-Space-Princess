use chrono::{Local, NaiveDateTime};

pub fn date_string_to_discord_timestamp(date_string: &str) -> String {
    let datetime = NaiveDateTime::parse_from_str(date_string, "%Y-%m-%d %H:%M:%S%.9f").expect("Failed to parse datetime");
    format!("<t:{}>", datetime.and_local_timezone(Local).unwrap().timestamp())
}