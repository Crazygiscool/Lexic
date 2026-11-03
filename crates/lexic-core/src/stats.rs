use crate::model::Rating;
use chrono::NaiveDate;

pub fn retention(ratings: &[Rating]) -> f64 {
    if ratings.is_empty() {
        return 0.0;
    }
    let successful = ratings
        .iter()
        .filter(|rating| matches!(rating, Rating::Hard | Rating::Good | Rating::Easy))
        .count();
    successful as f64 / ratings.len() as f64
}

pub fn best_and_current_streak(days: &[NaiveDate]) -> (usize, usize) {
    if days.is_empty() {
        return (0, 0);
    }

    let mut sorted = days.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    let mut best = 0;
    let mut current_run = 0;
    let mut best_run = 0;
    let mut previous: Option<NaiveDate> = None;

    for day in &sorted {
        if previous
            .map(|prev| *day == prev + chrono::Duration::days(1))
            .unwrap_or(false)
        {
            current_run += 1;
        } else {
            current_run = 1;
        }
        if current_run > best_run {
            best_run = current_run;
        }
        previous = Some(*day);
        best = current_run;
    }

    (best, best_run)
}

pub fn due_forecast_bucket(due: chrono::NaiveDate, today: chrono::NaiveDate) -> i64 {
    (due - today).num_days()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn retention_uses_successful_reviews() {
        let ratings = [Rating::Good, Rating::Easy, Rating::Again];
        assert_eq!(retention(&ratings), 2.0 / 3.0);
    }

    #[test]
    fn empty_retention_is_zero() {
        let ratings: [Rating; 0] = [];
        assert_eq!(retention(&ratings), 0.0);
    }

    #[test]
    fn streak_counts_consecutive_days() {
        let days = vec![
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
        ];
        assert_eq!(best_and_current_streak(&days), (1, 3));
    }
}
