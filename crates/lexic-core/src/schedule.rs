use chrono::{DateTime, Utc};

use crate::model::{CardState, Rating};

#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub state: CardState,
    pub due_at: DateTime<Utc>,
    pub interval_days: f64,
    pub ease_factor: f64,
    pub repetitions: i32,
    pub lapses: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sm2Config {
    pub learning_minutes: i64,
    pub first_review_days: i64,
    pub second_review_days: i64,
    pub easy_graduating_days: i64,
    pub max_interval_days: i64,
    pub maximum_ease: f64,
    pub minimum_ease: f64,
}

impl Default for Sm2Config {
    fn default() -> Self {
        Self {
            learning_minutes: 10,
            first_review_days: 1,
            second_review_days: 6,
            easy_graduating_days: 4,
            max_interval_days: 36500,
            maximum_ease: 3.0,
            minimum_ease: 1.3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CardSnapshot {
    pub state: CardState,
    pub interval_days: f64,
    pub ease_factor: f64,
    pub repetitions: i32,
    pub lapses: i32,
}

pub trait Scheduler {
    fn new_card(&self, now: DateTime<Utc>) -> Outcome;
    fn grade(&self, state: &CardSnapshot, rating: Rating, now: DateTime<Utc>) -> Outcome;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sm2 {
    pub config: Sm2Config,
}

fn ease_update(ease: f64, quality: u8, minimum: f64, maximum: f64) -> f64 {
    let q = quality as f64;
    let value = ease + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02));
    value.clamp(minimum, maximum)
}

fn review_interval(repetitions: i32, previous_interval: f64, ease: f64, maximum: i64) -> f64 {
    let value = if repetitions <= 0 {
        1.0
    } else if repetitions == 1 {
        6.0
    } else {
        (previous_interval * ease).round()
    };
    value.min(maximum as f64)
}

impl Scheduler for Sm2 {
    fn new_card(&self, now: DateTime<Utc>) -> Outcome {
        Outcome {
            state: CardState::Learning,
            due_at: now + chrono::Duration::minutes(self.config.learning_minutes),
            interval_days: 0.0,
            ease_factor: 2.5,
            repetitions: 0,
            lapses: 0,
        }
    }

    fn grade(&self, card: &CardSnapshot, rating: Rating, now: DateTime<Utc>) -> Outcome {
        let ease_factor = ease_update(
            card.ease_factor,
            rating.quality(),
            self.config.minimum_ease,
            self.config.maximum_ease,
        );

        match card.state {
            CardState::New | CardState::Learning | CardState::Relearning => match rating {
                Rating::Again | Rating::Hard => Outcome {
                    state: CardState::Learning,
                    due_at: now + chrono::Duration::minutes(self.config.learning_minutes),
                    interval_days: 0.0,
                    ease_factor,
                    repetitions: 0,
                    lapses: card.lapses,
                },
                Rating::Good => Outcome {
                    state: CardState::Review,
                    due_at: now + chrono::Duration::days(self.config.first_review_days),
                    interval_days: self.config.first_review_days as f64,
                    ease_factor,
                    repetitions: 1,
                    lapses: card.lapses,
                },
                Rating::Easy => Outcome {
                    state: CardState::Review,
                    due_at: now + chrono::Duration::days(self.config.easy_graduating_days),
                    interval_days: self.config.easy_graduating_days as f64,
                    ease_factor,
                    repetitions: 2,
                    lapses: card.lapses,
                },
            },
            CardState::Suspended => self.grade(
                &CardSnapshot {
                    state: CardState::New,
                    ..card.clone()
                },
                rating,
                now,
            ),
            CardState::Review => match rating {
                Rating::Again => Outcome {
                    state: CardState::Relearning,
                    due_at: now + chrono::Duration::minutes(self.config.learning_minutes),
                    interval_days: 0.0,
                    ease_factor,
                    repetitions: 0,
                    lapses: card.lapses + 1,
                },
                Rating::Hard | Rating::Good | Rating::Easy => {
                    let interval = review_interval(
                        card.repetitions,
                        card.interval_days,
                        ease_factor,
                        self.config.max_interval_days,
                    );
                    Outcome {
                        state: CardState::Review,
                        due_at: now + chrono::Duration::days(interval.round() as i64),
                        interval_days: interval,
                        ease_factor,
                        repetitions: card.repetitions + 1,
                        lapses: card.lapses,
                    }
                }
            },
        }
    }
}

pub fn snapshot_from_card(card: &crate::model::Card) -> CardSnapshot {
    CardSnapshot {
        state: card.state,
        interval_days: card.interval_days,
        ease_factor: card.ease_factor,
        repetitions: card.repetitions,
        lapses: card.lapses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn new_card_is_learning() {
        let scheduler = Sm2::default();
        let now = Utc::now();
        let outcome = scheduler.new_card(now);
        assert_eq!(outcome.state, CardState::Learning);
        assert_eq!(outcome.repetitions, 0);
        assert_eq!(outcome.lapses, 0);
    }

    #[test]
    fn good_twice_uses_one_then_six_day_intervals() {
        let scheduler = Sm2::default();
        let now = Utc::now();
        let first = CardSnapshot {
            state: CardState::New,
            interval_days: 0.0,
            ease_factor: 2.5,
            repetitions: 0,
            lapses: 0,
        };
        let first_outcome = scheduler.grade(&first, Rating::Good, now);
        assert_eq!(first_outcome.state, CardState::Review);
        assert_eq!(first_outcome.interval_days, 1.0);

        let second = scheduler.grade(
            &CardSnapshot {
                state: CardState::Review,
                interval_days: first_outcome.interval_days,
                ease_factor: first_outcome.ease_factor,
                repetitions: first_outcome.repetitions,
                lapses: first_outcome.lapses,
            },
            Rating::Good,
            now + Duration::days(1),
        );
        assert_eq!(second.interval_days, 6.0);
    }

    #[test]
    fn again_resets_progress_and_increments_lapses() {
        let scheduler = Sm2::default();
        let now = Utc::now();
        let card = CardSnapshot {
            state: CardState::Review,
            interval_days: 6.0,
            ease_factor: 2.5,
            repetitions: 2,
            lapses: 1,
        };
        let outcome = scheduler.grade(&card, Rating::Again, now);
        assert_eq!(outcome.state, CardState::Relearning);
        assert_eq!(outcome.repetitions, 0);
        assert_eq!(outcome.interval_days, 0.0);
        assert_eq!(outcome.lapses, 2);
    }

    #[test]
    fn ease_factor_is_clamped() {
        let scheduler = Sm2::default();
        let now = Utc::now();
        let outcome = scheduler.grade(
            &CardSnapshot {
                state: CardState::Review,
                interval_days: 6.0,
                ease_factor: 1.3,
                repetitions: 2,
                lapses: 0,
            },
            Rating::Again,
            now,
        );
        assert!(outcome.ease_factor >= 1.3);
        assert!(outcome.ease_factor <= 3.0);
    }
}
