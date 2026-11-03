use chrono::{DateTime, Duration, FixedOffset, Local, NaiveDate, Timelike, Utc};

pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

pub fn local_day(value: DateTime<Utc>) -> NaiveDate {
    value.with_timezone(&Local).date_naive()
}

pub fn local_day_string(value: DateTime<Utc>) -> String {
    local_day(value).format("%Y-%m-%d").to_string()
}

pub fn start_of_local_day(value: DateTime<Utc>, offset: FixedOffset) -> DateTime<Utc> {
    let local = value.with_timezone(&offset);
    let date = local.date_naive();
    date.and_time(
        local
            .time()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap(),
    )
    .and_local_timezone(offset)
    .earliest()
    .unwrap()
    .with_timezone(&Utc)
}

pub fn add_days(value: DateTime<Utc>, days: i64) -> DateTime<Utc> {
    value + Duration::days(days)
}

pub fn add_minutes(value: DateTime<Utc>, minutes: i64) -> DateTime<Utc> {
    value + Duration::minutes(minutes)
}

pub fn parse_day(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}
