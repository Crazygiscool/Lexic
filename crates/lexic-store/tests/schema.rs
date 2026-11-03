use chrono::Utc;
use lexic_core::model::{Entry, Sense};
use lexic_store::Store;
use uuid::Uuid;

#[test]
fn opens_in_memory_and_round_trips_entry() {
    let store = Store::in_memory().unwrap();
    let now = Utc::now();
    let entry_id = Uuid::new_v4();
    let entry = Entry {
        id: entry_id,
        word: "example".to_string(),
        normalized_word: "example".to_string(),
        phonetic: Some("/ig-zam-pul/".to_string()),
        audio_url: None,
        origin: None,
        source: "manual".to_string(),
        source_url: None,
        license: None,
        created_at: now,
        updated_at: now,
    };
    let sense = Sense {
        id: Uuid::new_v4(),
        entry_id,
        part_of_speech: Some("noun".to_string()),
        definition: "a sample".to_string(),
        example: Some("for example".to_string()),
        synonyms: vec!["sample".to_string()],
        antonyms: Vec::new(),
        sort_order: 0,
    };
    store.upsert_entry(&entry, &[sense]).unwrap();
    let found = store.search_entries("exa", 10).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, entry_id);
    assert_eq!(store.get_senses(entry_id).unwrap().len(), 1);
}

#[test]
fn deck_and_card_creation_work() {
    let store = Store::in_memory().unwrap();
    let deck_id = store.create_deck("My Words", None).unwrap();
    let entry_id = Uuid::new_v4();
    let now = Utc::now();
    let entry = Entry {
        id: entry_id,
        word: "lexicon".to_string(),
        normalized_word: "lexicon".to_string(),
        phonetic: None,
        audio_url: None,
        origin: None,
        source: "manual".to_string(),
        source_url: None,
        license: None,
        created_at: now,
        updated_at: now,
    };
    store.upsert_entry(&entry, &[]).unwrap();
    let card_id = store.add_card(entry_id, deck_id).unwrap();
    let cards = store.list_cards_by_deck(deck_id).unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].id, card_id);
    assert_eq!(cards[0].state, lexic_core::model::CardState::New);
}
