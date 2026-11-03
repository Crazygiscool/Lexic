use chrono::{DateTime, Duration, Utc};

pub fn is_fresh(fetched_at: Option<i64>, ttl_days: i64, now: DateTime<Utc>) -> bool {
    match fetched_at {
        Some(value) => {
            let fetched = DateTime::from_timestamp(value, 0).unwrap_or_else(Utc::now);
            now - fetched < Duration::days(ttl_days)
        }
        None => false,
    }
}

pub fn stale_after(fetched_at: i64, ttl_days: i64) -> DateTime<Utc> {
    let fetched = DateTime::from_timestamp(fetched_at, 0).unwrap_or_else(Utc::now);
    fetched + Duration::days(ttl_days)
}

pub fn negative_cache_ttl_days() -> i64 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn fresh_within_ttl() {
        let now = Utc::now();
        let fetched = (now - Duration::days(1)).timestamp();
        assert!(is_fresh(Some(fetched), 30, now));
    }

    #[test]
    fn stale_after_ttl() {
        let now = Utc::now();
        let fetched = (now - Duration::days(31)).timestamp();
        assert!(!is_fresh(Some(fetched), 30, now));
    }

    #[test]
    fn missing_is_not_fresh() {
        assert!(!is_fresh(None, 30, Utc::now()));
    }
}
