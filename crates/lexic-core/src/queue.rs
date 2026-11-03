use crate::model::{Card, CardState};
use crate::schedule::CardSnapshot;

impl CardSnapshot {
    pub fn from_card(card: &Card) -> Self {
        Self {
            state: card.state,
            interval_days: card.interval_days,
            ease_factor: card.ease_factor,
            repetitions: card.repetitions,
            lapses: card.lapses,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueueEntry {
    pub id: crate::model::Id,
    pub state: CardState,
    pub due_at: Option<crate::model::Timestamp>,
    pub created_at: crate::model::Timestamp,
}

#[derive(Clone, Copy, Debug)]
pub struct QueueOptions {
    pub now: crate::model::Timestamp,
    pub new_limit: usize,
    pub review_limit: usize,
    pub interleave_every: usize,
}

pub fn build_queue(entries: &[QueueEntry], options: QueueOptions) -> Vec<QueueEntry> {
    let now = options.now;
    let mut learning_due = Vec::new();
    let mut review_due = Vec::new();
    let mut new_cards = Vec::new();

    for entry in entries {
        match entry.state {
            CardState::New => new_cards.push(entry.clone()),
            CardState::Learning | CardState::Relearning => {
                if entry.due_at.map(|due| due <= now).unwrap_or(true) {
                    learning_due.push(entry.clone());
                }
            }
            CardState::Review => {
                if entry.due_at.map(|due| due <= now).unwrap_or(false) {
                    review_due.push(entry.clone());
                }
            }
            CardState::Suspended => {}
        }
    }

    learning_due.sort_by_key(|entry| (entry.due_at, entry.created_at));
    review_due.sort_by_key(|entry| (entry.due_at, entry.created_at));
    new_cards.sort_by_key(|entry| entry.created_at);

    let mut review_queue = Vec::new();
    review_queue.extend(learning_due);
    review_queue.extend(review_due);
    review_queue.truncate(options.review_limit);

    let new_queue: Vec<_> = new_cards.into_iter().take(options.new_limit).collect();

    let mut result = Vec::new();
    let mut new_index = 0;
    let mut review_seen = 0_usize;

    for entry in review_queue {
        result.push(entry);
        review_seen += 1;
        if options.interleave_every > 0
            && review_seen.is_multiple_of(options.interleave_every)
            && new_index < new_queue.len()
        {
            result.push(new_queue[new_index].clone());
            new_index += 1;
        }
    }

    while new_index < new_queue.len() {
        result.push(new_queue[new_index].clone());
        new_index += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn new_cards_respect_limit() {
        let now = Utc::now();
        let entries = vec![
            QueueEntry {
                id: uuid::Uuid::new_v4(),
                state: crate::model::CardState::New,
                due_at: None,
                created_at: now,
            },
            QueueEntry {
                id: uuid::Uuid::new_v4(),
                state: crate::model::CardState::New,
                due_at: None,
                created_at: now,
            },
        ];
        let queue = build_queue(
            &entries,
            QueueOptions {
                now,
                new_limit: 1,
                review_limit: 10,
                interleave_every: 5,
            },
        );
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn interleaves_new_cards_after_reviews() {
        let now = Utc::now();
        let review = QueueEntry {
            id: uuid::Uuid::new_v4(),
            state: crate::model::CardState::Review,
            due_at: Some(now),
            created_at: now,
        };
        let new_card = QueueEntry {
            id: uuid::Uuid::new_v4(),
            state: crate::model::CardState::New,
            due_at: None,
            created_at: now,
        };
        let queue = build_queue(
            &[review.clone(), new_card.clone()],
            QueueOptions {
                now,
                new_limit: 1,
                review_limit: 10,
                interleave_every: 1,
            },
        );
        assert_eq!(queue, vec![review, new_card]);
    }
}
