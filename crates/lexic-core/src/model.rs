use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Id = Uuid;
pub type Timestamp = DateTime<Utc>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    Again,
    Hard,
    Good,
    Easy,
}

impl Rating {
    pub fn quality(self) -> u8 {
        match self {
            Rating::Again => 2,
            Rating::Hard => 3,
            Rating::Good => 4,
            Rating::Easy => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
    Suspended,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub id: Id,
    pub word: String,
    pub normalized_word: String,
    pub phonetic: Option<String>,
    pub audio_url: Option<String>,
    pub origin: Option<String>,
    pub source: String,
    pub source_url: Option<String>,
    pub license: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sense {
    pub id: Id,
    pub entry_id: Id,
    pub part_of_speech: Option<String>,
    pub definition: String,
    pub example: Option<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Deck {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub new_per_day: Option<i32>,
    pub reviews_per_day: Option<i32>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub archived_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub id: Id,
    pub entry_id: Id,
    pub deck_id: Id,
    pub direction: String,
    pub front_note: Option<String>,
    pub back_note: Option<String>,
    pub state: CardState,
    pub due_at: Option<Timestamp>,
    pub interval_days: f64,
    pub ease_factor: f64,
    pub repetitions: i32,
    pub lapses: i32,
    pub first_reviewed_at: Option<Timestamp>,
    pub last_reviewed_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewLog {
    pub id: Id,
    pub card_id: Id,
    pub reviewed_at: Timestamp,
    pub day: String,
    pub rating: Rating,
    pub prev_state: CardState,
    pub prev_interval_days: f64,
    pub prev_ease: f64,
    pub new_state: CardState,
    pub new_interval_days: f64,
    pub new_ease: f64,
    pub duration_ms: Option<i64>,
}

impl std::fmt::Display for CardState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardState::New => write!(formatter, "new"),
            CardState::Learning => write!(formatter, "learning"),
            CardState::Review => write!(formatter, "review"),
            CardState::Relearning => write!(formatter, "relearning"),
            CardState::Suspended => write!(formatter, "suspended"),
        }
    }
}
