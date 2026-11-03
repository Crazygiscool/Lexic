#![allow(clippy::collapsible_if)]

use lexic_core::model::{Card, Deck, Entry, Sense};
use lexic_core::schedule::{CardSnapshot, Scheduler, Sm2};
use lexic_core::time::now_utc;
use lexic_dict::{DictionaryClient, LookupResult, normalize};
use lexic_store::Store;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use iced::keyboard;
use iced::{
    Alignment, Color, Element, Length, Task, Theme,
    widget::{button as iced_button, column, container, row, scrollable, slider, text, text_input},
};

fn monochrome_button(
    selected: bool,
) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_theme, status| {
        let (background, text_color, border) = match status {
            iced::widget::button::Status::Hovered => (
                Color::from_rgb8(34, 34, 34),
                Color::from_rgb8(245, 245, 245),
                Color::from_rgb8(70, 70, 70),
            ),
            iced::widget::button::Status::Pressed => (
                Color::from_rgb8(28, 28, 28),
                Color::from_rgb8(245, 245, 245),
                Color::from_rgb8(70, 70, 70),
            ),
            iced::widget::button::Status::Disabled => (
                Color::from_rgb8(16, 16, 16),
                Color::from_rgb8(105, 105, 105),
                Color::from_rgb8(38, 38, 38),
            ),
            iced::widget::button::Status::Active => {
                if selected {
                    (
                        Color::from_rgb8(52, 52, 52),
                        Color::from_rgb8(245, 245, 245),
                        Color::from_rgb8(82, 82, 82),
                    )
                } else {
                    (
                        Color::from_rgb8(24, 24, 24),
                        Color::from_rgb8(230, 230, 230),
                        Color::from_rgb8(54, 54, 54),
                    )
                }
            }
        };
        iced::widget::button::Style {
            snap: false,
            background: Some(background.into()),
            text_color,
            border: iced::Border {
                radius: 10.0.into(),
                width: 1.0,
                color: border,
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 10.0,
            },
        }
    }
}

fn button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Button<'a, Message> {
    iced_button(content).style(monochrome_button(false))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Home,
    Search,
    Decks,
    DeckDetail(lexic_core::model::Id),
    Review,
    Stats,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
    System,
}

pub struct App {
    screen: Screen,
    sidebar_collapsed: bool,
    store: Option<Arc<Store>>,
    dictionary: DictionaryClient,
    decks: Vec<Deck>,
    deck_counts: BTreeMap<lexic_core::model::Id, (i64, i64, i64)>,
    search_input: String,
    lookup_generation: u64,
    search_status: SearchStatus,
    active_entry: Option<Entry>,
    active_senses: Vec<Sense>,
    selected_save_deck: Option<lexic_core::model::Id>,
    save_status: Option<String>,

    new_deck_name: String,
    selected_deck: Option<lexic_core::model::Id>,
    rename_deck_input: String,
    deck_new_limit_input: String,
    deck_review_limit_input: String,
    deck_management_status: Option<String>,
    deck_cards: Vec<DeckCard>,
    deck_card_status: Option<String>,
    #[allow(dead_code)]
    theme_mode: ThemeMode,
    include_review_history: bool,
    settings_status: Option<String>,
    stats_range: u32,
    stats_data: Option<StatsSnapshot>,

    review: Option<ReviewSession>,
    review_status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeckCard {
    pub card: Card,
    pub word: String,
}

#[derive(Debug, Clone)]
pub struct ReviewSession {
    cards: Vec<ReviewCard>,
    queue: VecDeque<usize>,
    requeue: Vec<usize>,
    current: Option<usize>,
    revealed: bool,
    reviewed: usize,
    correct: usize,
    last_graded: Option<usize>,
    last_undo: Option<UndoSnapshot>,
    last_rating: Option<lexic_core::model::Rating>,
    ended: bool,
}

#[derive(Debug, Clone)]
pub struct ReviewCard {
    pub card: Card,
    pub entry: Option<Entry>,
    pub senses: Vec<Sense>,
}

#[derive(Debug, Clone)]
pub struct UndoSnapshot {
    pub review_id: lexic_core::model::Id,
    pub previous_card: Card,
    pub rating: lexic_core::model::Rating,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    Navigate(Screen),
    ToggleSidebar,
    Event(iced::Event),
    SearchInputChanged(String),
    Lookup,
    LookupFinished(u64, Result<LookupResult, String>),
    SelectSaveDeck(lexic_core::model::Id),
    SaveActiveEntry,
    SaveFinished(Result<lexic_core::model::Id, String>),
    LoadSettings,
    SettingsLoaded(Option<String>),
    ToggleIncludeReviewHistory,
    BeginExport,
    ExportPathChosen(Option<std::path::PathBuf>),
    ExportFinished(Result<(), String>),
    BeginImport,
    ImportPathChosen(Option<std::path::PathBuf>),
    ImportFinished(Result<lexic_store::ImportSummary, String>),

    NewDeckNameChanged(String),
    CreateDeck,
    DeckCreated(Result<lexic_core::model::Id, String>),
    SelectManageDeck(lexic_core::model::Id),
    RenameDeckNameChanged(String),
    DeckNewLimitChanged(String),
    DeckReviewLimitChanged(String),
    SaveDeckDetails,
    DeckDetailsSaved(Result<(), String>),
    DeleteDeck(lexic_core::model::Id),
    ArchiveDeck(lexic_core::model::Id),
    DeckActionFinished(Result<(), String>),
    ReloadDecks,
    DecksReloaded(Vec<Deck>, Vec<(lexic_core::model::Id, (i64, i64, i64))>),

    OpenDeckDetail(lexic_core::model::Id),
    LoadStats,
    StatsLoaded(StatsSnapshot),
    SetStatsRange(u32),
    DeckDetailLoaded(Vec<DeckCard>),
    RemoveCard(lexic_core::model::Id),
    SuspendCard(lexic_core::model::Id),
    ResetCard(lexic_core::model::Id),
    CardActionFinished(Result<(), String>),

    StartReview,
    ReviewStarted(Vec<ReviewCard>),
    RevealAnswer,
    Grade(lexic_core::model::Rating),
    GradeSaved(Result<(), String>),
    UndoReview,
    UndoFinished(Result<(), String>),
    EndReview,
}

#[derive(Debug, Clone, Default)]
pub struct StatsSnapshot {
    pub reviews_by_day: std::collections::BTreeMap<String, DailyStat>,
    pub due_forecast: Vec<(String, i64)>,
    pub total_reviews: usize,
    pub retention: f64,
    pub total_duration_ms: i64,
    pub current_streak: usize,
    pub best_streak: usize,
}

#[derive(Debug, Clone, Default)]
pub struct DailyStat {
    pub count: usize,
    pub successful: usize,
    pub duration_ms: i64,
}

impl App {
    pub fn new(store: Option<Arc<Store>>, decks: Vec<Deck>) -> (Self, Task<Message>) {
        let dictionary = DictionaryClient::default();
        let app = Self {
            screen: Screen::Home,
            sidebar_collapsed: false,
            store,
            dictionary,
            decks,
            deck_counts: BTreeMap::new(),
            search_input: String::new(),
            lookup_generation: 0,
            search_status: SearchStatus::Idle,
            active_entry: None,
            active_senses: Vec::new(),
            selected_save_deck: None,
            save_status: None,
            new_deck_name: String::new(),
            selected_deck: None,
            rename_deck_input: String::new(),
            deck_new_limit_input: String::new(),
            deck_review_limit_input: String::new(),
            deck_management_status: None,
            deck_cards: Vec::new(),
            deck_card_status: None,
            theme_mode: ThemeMode::Light,
            include_review_history: false,
            settings_status: None,
            stats_range: 30,
            stats_data: None,
            review: None,
            review_status: None,
        };
        let task = if let Some(store) = app.store.clone() {
            Task::perform(
                async move {
                    let now = now_utc();
                    let decks = store.list_decks().unwrap_or_default();
                    let counts = decks
                        .iter()
                        .map(|deck| {
                            let counts = store.deck_counts(deck.id, now).unwrap_or((0, 0, 0));
                            (deck.id, counts)
                        })
                        .collect();
                    (decks, counts)
                },
                |value| Message::DecksReloaded(value.0, value.1),
            )
        } else {
            Task::none()
        };
        (app, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadSettings => {
                let Some(store) = self.store.clone() else {
                    return Task::none();
                };
                Task::perform(
                    async move { store.get_setting("theme").ok().flatten() },
                    Message::SettingsLoaded,
                )
            }
            Message::SettingsLoaded(theme) => {
                self.theme_mode = match theme.as_deref() {
                    Some("dark") => ThemeMode::Dark,
                    Some("system") => ThemeMode::System,
                    _ => ThemeMode::Light,
                };
                Task::none()
            }
            Message::ToggleIncludeReviewHistory => {
                self.include_review_history = !self.include_review_history;
                Task::none()
            }
            Message::BeginExport => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_file_name("lexic-backup.json")
                        .save_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                Message::ExportPathChosen,
            ),
            Message::ExportPathChosen(Some(path)) => {
                let Some(store) = self.store.clone() else {
                    self.settings_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                let include = self.include_review_history;
                Task::perform(
                    async move {
                        lexic_store::export_backup(&store, &path, include)
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    },
                    Message::ExportFinished,
                )
            }
            Message::ExportPathChosen(None) => Task::none(),
            Message::ExportFinished(result) => {
                self.settings_status = Some(match result {
                    Ok(_) => String::from("Export saved"),
                    Err(error) => error,
                });
                Task::none()
            }
            Message::BeginImport => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Lexic backup", &["json"])
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                Message::ImportPathChosen,
            ),
            Message::ImportPathChosen(Some(path)) => {
                let Some(store) = self.store.clone() else {
                    self.settings_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                Task::perform(
                    async move {
                        lexic_store::import_backup(&store, &path, lexic_store::ImportMode::Merge)
                            .map_err(|error| error.to_string())
                    },
                    Message::ImportFinished,
                )
            }
            Message::ImportPathChosen(None) => Task::none(),
            Message::ImportFinished(result) => {
                self.settings_status = Some(match result {
                    Ok(_) => String::from("Import complete"),
                    Err(error) => error,
                });
                Task::none()
            }
            Message::Navigate(screen) => {
                self.screen = screen;
                if screen == Screen::Settings {
                    return self.update(Message::LoadSettings);
                }
                if screen == Screen::Stats {
                    return self.update(Message::LoadStats);
                }
                Task::none()
            }
            Message::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
                Task::none()
            }
            Message::Event(event) => {
                if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
                    if let Screen::Review = self.screen {
                        if let Some(session) = &mut self.review {
                            if let Some(_current) = session.current {
                                match key {
                                    keyboard::Key::Named(keyboard::key::Named::Space) => {
                                        if !session.revealed {
                                            session.revealed = true;
                                        }
                                    }
                                    keyboard::Key::Character(value) => match value.as_ref() {
                                        "1" => {
                                            return self.update(Message::Grade(
                                                lexic_core::model::Rating::Again,
                                            ));
                                        }
                                        "2" => {
                                            return self.update(Message::Grade(
                                                lexic_core::model::Rating::Hard,
                                            ));
                                        }
                                        "3" => {
                                            return self.update(Message::Grade(
                                                lexic_core::model::Rating::Good,
                                            ));
                                        }
                                        "4" => {
                                            return self.update(Message::Grade(
                                                lexic_core::model::Rating::Easy,
                                            ));
                                        }
                                        _ => {}
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::SearchInputChanged(value) => {
                self.search_input = value;
                Task::none()
            }
            Message::Lookup => self.start_lookup(),
            Message::LookupFinished(id, result) => {
                if id != self.lookup_generation {
                    return Task::none();
                }
                match result {
                    Ok(lookup) => {
                        self.active_entry = Some(lookup.entry);
                        self.active_senses = lookup.senses;
                        self.search_status = SearchStatus::Loaded;
                        self.screen = Screen::Search;
                    }
                    Err(error) => {
                        self.search_status = SearchStatus::Error;
                        self.save_status = Some(error);
                    }
                }
                Task::none()
            }
            Message::SelectSaveDeck(deck_id) => {
                self.selected_save_deck = Some(deck_id);
                Task::none()
            }
            Message::SaveActiveEntry => self.save_active_entry(),
            Message::SaveFinished(result) => {
                self.save_status = Some(match result {
                    Ok(_) => String::from("Saved"),
                    Err(error) => error,
                });
                Task::none()
            }

            Message::NewDeckNameChanged(value) => {
                self.new_deck_name = value;
                Task::none()
            }
            Message::CreateDeck => {
                let name = self.new_deck_name.trim().to_string();
                if name.is_empty() {
                    self.deck_management_status = Some(String::from("Deck name cannot be empty"));
                    return Task::none();
                }
                let Some(store) = self.store.clone() else {
                    self.deck_management_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                self.deck_management_status = None;
                Task::perform(
                    async move {
                        store
                            .create_deck(&name, None)
                            .map_err(|error| error.to_string())
                    },
                    Message::DeckCreated,
                )
            }
            Message::DeckCreated(result) => {
                match result {
                    Ok(_) => {
                        self.new_deck_name.clear();
                        self.deck_management_status = Some(String::from("Deck created"));
                    }
                    Err(error) => self.deck_management_status = Some(error),
                }
                self.update(Message::ReloadDecks)
            }
            Message::SelectManageDeck(deck_id) => {
                self.selected_deck = Some(deck_id);
                if let Some(deck) = self.decks.iter().find(|deck| deck.id == deck_id) {
                    self.rename_deck_input = deck.name.clone();
                    self.deck_new_limit_input = deck
                        .new_per_day
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                    self.deck_review_limit_input = deck
                        .reviews_per_day
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                }
                Task::none()
            }
            Message::RenameDeckNameChanged(value) => {
                self.rename_deck_input = value;
                Task::none()
            }
            Message::DeckNewLimitChanged(value) => {
                self.deck_new_limit_input = value;
                Task::none()
            }
            Message::DeckReviewLimitChanged(value) => {
                self.deck_review_limit_input = value;
                Task::none()
            }
            Message::SaveDeckDetails => {
                let Some(deck_id) = self.selected_deck else {
                    self.deck_management_status = Some(String::from("Select a deck first"));
                    return Task::none();
                };
                let Some(store) = self.store.clone() else {
                    self.deck_management_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                let name = self.rename_deck_input.trim().to_string();
                let new_limit = if self.deck_new_limit_input.trim().is_empty() {
                    None
                } else {
                    match self.deck_new_limit_input.trim().parse::<i32>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            self.deck_management_status =
                                Some(String::from("Invalid new-card limit"));
                            return Task::none();
                        }
                    }
                };
                let review_limit = if self.deck_review_limit_input.trim().is_empty() {
                    None
                } else {
                    match self.deck_review_limit_input.trim().parse::<i32>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            self.deck_management_status =
                                Some(String::from("Invalid review limit"));
                            return Task::none();
                        }
                    }
                };
                Task::perform(
                    async move {
                        if !name.is_empty() {
                            store
                                .rename_deck(deck_id, &name)
                                .map_err(|error| error.to_string())?;
                        }
                        store
                            .update_deck_limits(deck_id, new_limit, review_limit)
                            .map_err(|error| error.to_string())?;
                        Ok(())
                    },
                    Message::DeckDetailsSaved,
                )
            }
            Message::DeckDetailsSaved(result) => {
                self.deck_management_status = Some(match result {
                    Ok(_) => String::from("Deck saved"),
                    Err(error) => error,
                });
                self.update(Message::ReloadDecks)
            }
            Message::DeleteDeck(deck_id) => {
                let Some(store) = self.store.clone() else {
                    self.deck_management_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                Task::perform(
                    async move {
                        store
                            .delete_deck(deck_id)
                            .map_err(|error| error.to_string())
                    },
                    Message::DeckActionFinished,
                )
            }
            Message::ArchiveDeck(deck_id) => {
                let Some(store) = self.store.clone() else {
                    self.deck_management_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                Task::perform(
                    async move {
                        store
                            .archive_deck(deck_id)
                            .map_err(|error| error.to_string())
                    },
                    Message::DeckActionFinished,
                )
            }
            Message::DeckActionFinished(result) => {
                self.deck_management_status = Some(match result {
                    Ok(_) => String::from("Deck updated"),
                    Err(error) => error,
                });
                if self
                    .selected_deck
                    .is_some_and(|id| !self.decks.iter().any(|deck| deck.id == id))
                {
                    self.selected_deck = None;
                }
                self.update(Message::ReloadDecks)
            }
            Message::LoadStats => {
                let Some(store) = self.store.clone() else {
                    return Task::none();
                };
                let range = self.stats_range;
                Task::perform(
                    async move {
                        let now = now_utc();
                        let days = range as i64;
                        let since = (now - chrono::Duration::days(days))
                            .format("%Y-%m-%d")
                            .to_string();
                        let rows = store.review_history_rows(&since).unwrap_or_default();
                        let mut reviews_by_day: std::collections::BTreeMap<String, DailyStat> =
                            std::collections::BTreeMap::new();
                        let mut ratings = Vec::new();
                        let mut total_duration = 0_i64;
                        for (day, rating_num, duration) in rows {
                            let entry = reviews_by_day.entry(day).or_default();
                            entry.count += 1;
                            if rating_num >= 3 {
                                entry.successful += 1;
                            }
                            if let Some(value) = duration {
                                entry.duration_ms += value;
                                total_duration += value;
                            }
                            match rating_num {
                                1 => ratings.push(lexic_core::model::Rating::Again),
                                2 => ratings.push(lexic_core::model::Rating::Hard),
                                3 => ratings.push(lexic_core::model::Rating::Good),
                                _ => ratings.push(lexic_core::model::Rating::Easy),
                            }
                        }
                        let retention = lexic_core::stats::retention(&ratings);
                        let day_values: Vec<chrono::NaiveDate> = reviews_by_day
                            .keys()
                            .filter_map(|day| {
                                chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").ok()
                            })
                            .collect();
                        let (current_streak, best_streak) =
                            lexic_core::stats::best_and_current_streak(&day_values);
                        let total_reviews = reviews_by_day.values().map(|value| value.count).sum();
                        let due_forecast = store.due_forecast_counts(30, now).unwrap_or_default();
                        StatsSnapshot {
                            reviews_by_day,
                            due_forecast,
                            total_reviews,
                            retention,
                            total_duration_ms: total_duration,
                            current_streak,
                            best_streak,
                        }
                    },
                    Message::StatsLoaded,
                )
            }
            Message::StatsLoaded(data) => {
                self.stats_data = Some(data);
                Task::none()
            }
            Message::SetStatsRange(range) => {
                self.stats_range = range;
                self.update(Message::LoadStats)
            }
            Message::ReloadDecks => {
                let Some(store) = self.store.clone() else {
                    return Task::none();
                };
                Task::perform(
                    async move {
                        let now = now_utc();
                        let decks = store.list_decks().unwrap_or_default();
                        let counts = decks
                            .iter()
                            .map(|deck| {
                                let counts = store.deck_counts(deck.id, now).unwrap_or((0, 0, 0));
                                (deck.id, counts)
                            })
                            .collect();
                        (decks, counts)
                    },
                    |value| Message::DecksReloaded(value.0, value.1),
                )
            }
            Message::DecksReloaded(decks, counts) => {
                self.decks = decks;
                self.deck_counts = counts.into_iter().collect();
                if let Some(selected) = self.selected_deck {
                    if !self.decks.iter().any(|deck| deck.id == selected) {
                        self.selected_deck = None;
                    }
                }
                if let Some(selected_save) = self.selected_save_deck {
                    if !self.decks.iter().any(|deck| deck.id == selected_save) {
                        self.selected_save_deck = self.decks.first().map(|deck| deck.id);
                    }
                } else {
                    self.selected_save_deck = self.decks.first().map(|deck| deck.id);
                }
                Task::none()
            }
            Message::OpenDeckDetail(deck_id) => {
                let Some(store) = self.store.clone() else {
                    self.deck_card_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                self.selected_deck = Some(deck_id);
                self.screen = Screen::DeckDetail(deck_id);
                self.deck_card_status = None;
                Task::perform(
                    async move {
                        let cards = store.list_cards_by_deck(deck_id).unwrap_or_default();
                        cards
                            .into_iter()
                            .map(|card| {
                                let word = store
                                    .get_entry(card.entry_id)
                                    .ok()
                                    .flatten()
                                    .map(|entry| entry.word)
                                    .unwrap_or_else(|| String::from("Unknown word"));
                                DeckCard { card, word }
                            })
                            .collect()
                    },
                    Message::DeckDetailLoaded,
                )
            }
            Message::DeckDetailLoaded(rows) => {
                self.deck_cards = rows;
                Task::none()
            }
            Message::RemoveCard(card_id) => {
                let Some(store) = self.store.clone() else {
                    self.deck_card_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                Task::perform(
                    async move {
                        store
                            .remove_card(card_id)
                            .map_err(|error| error.to_string())
                    },
                    Message::CardActionFinished,
                )
            }
            Message::SuspendCard(card_id) => {
                let Some(store) = self.store.clone() else {
                    self.deck_card_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                Task::perform(
                    async move {
                        store
                            .suspend_card(card_id, true)
                            .map_err(|error| error.to_string())
                    },
                    Message::CardActionFinished,
                )
            }
            Message::ResetCard(card_id) => {
                let Some(store) = self.store.clone() else {
                    self.deck_card_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                Task::perform(
                    async move { store.reset_card(card_id).map_err(|error| error.to_string()) },
                    Message::CardActionFinished,
                )
            }
            Message::CardActionFinished(result) => {
                self.deck_card_status = Some(match result {
                    Ok(_) => String::from("Card updated"),
                    Err(error) => error,
                });
                if let Screen::DeckDetail(deck_id) = self.screen {
                    self.update(Message::OpenDeckDetail(deck_id))
                } else {
                    Task::none()
                }
            }

            Message::StartReview => {
                let Some(store) = self.store.clone() else {
                    self.review_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };
                self.review_status = None;
                self.screen = Screen::Review;
                Task::perform(
                    async move {
                        let now = now_utc();
                        let cards = store.list_cards_due(now).unwrap_or_default();
                        cards
                            .into_iter()
                            .map(|card| {
                                let entry = store.get_entry(card.entry_id).ok().flatten();
                                let senses = entry
                                    .as_ref()
                                    .map(|entry| store.get_senses(entry.id).unwrap_or_default())
                                    .unwrap_or_default();
                                ReviewCard {
                                    card,
                                    entry,
                                    senses,
                                }
                            })
                            .collect()
                    },
                    Message::ReviewStarted,
                )
            }
            Message::ReviewStarted(cards) => {
                if cards.is_empty() {
                    self.review = None;
                    self.review_status = Some(String::from("No cards due"));
                    return Task::none();
                }
                let now = now_utc();
                let queue_entries: Vec<lexic_core::queue::QueueEntry> = cards
                    .iter()
                    .map(|review_card| lexic_core::queue::QueueEntry {
                        id: review_card.card.id,
                        state: review_card.card.state,
                        due_at: review_card.card.due_at,
                        created_at: review_card.card.created_at,
                    })
                    .collect();
                let ordered = lexic_core::queue::build_queue(
                    &queue_entries,
                    lexic_core::queue::QueueOptions {
                        now,
                        new_limit: 20,
                        review_limit: 200,
                        interleave_every: 5,
                    },
                );
                let queue = ordered
                    .iter()
                    .filter_map(|entry| cards.iter().position(|card| card.card.id == entry.id))
                    .collect();
                self.review = Some(ReviewSession {
                    cards,
                    queue,
                    requeue: Vec::new(),
                    current: Some(0),
                    revealed: false,
                    reviewed: 0,
                    correct: 0,
                    last_graded: None,
                    last_undo: None,
                    last_rating: None,
                    ended: false,
                });
                if let Some(session) = &mut self.review {
                    session.current = session.queue.pop_front();
                }
                self.review_status = Some(String::from("Review session ready"));
                Task::none()
            }
            Message::RevealAnswer => {
                if let Some(session) = &mut self.review {
                    session.revealed = true;
                }
                Task::none()
            }
            Message::Grade(rating) => {
                let now = now_utc();
                let (_card, previous_card, outcome, review_id, preview_success) = {
                    let session = match &mut self.review {
                        Some(value) => value,
                        None => return Task::none(),
                    };
                    let Some(current_index) = session.current else {
                        return Task::none();
                    };
                    let previous_card = session.cards[current_index].card.clone();
                    let snapshot = CardSnapshot::from_card(&previous_card);
                    let outcome = Sm2::default().grade(&snapshot, rating, now);
                    let review_id = lexic_core::model::Id::from_u128(rand_u128());

                    let mut new_card = previous_card.clone();
                    new_card.state = outcome.state;
                    new_card.due_at = Some(outcome.due_at);
                    new_card.interval_days = outcome.interval_days;
                    new_card.ease_factor = outcome.ease_factor;
                    new_card.repetitions = outcome.repetitions;
                    new_card.lapses = outcome.lapses;
                    new_card.last_reviewed_at = Some(now);
                    if new_card.first_reviewed_at.is_none() {
                        new_card.first_reviewed_at = Some(now);
                    }
                    new_card.updated_at = now;

                    let review = lexic_core::model::ReviewLog {
                        id: review_id,
                        card_id: previous_card.id,
                        reviewed_at: now,
                        day: lexic_core::time::local_day_string(now),
                        rating,
                        prev_state: previous_card.state,
                        prev_interval_days: previous_card.interval_days,
                        prev_ease: previous_card.ease_factor,
                        new_state: outcome.state,
                        new_interval_days: outcome.interval_days,
                        new_ease: outcome.ease_factor,
                        duration_ms: None,
                    };
                    let _ = review;

                    session.cards[current_index].card = new_card.clone();
                    session.reviewed += 1;
                    if rating != lexic_core::model::Rating::Again {
                        session.correct += 1;
                    }
                    session.last_graded = Some(current_index);
                    session.last_rating = Some(rating);
                    session.last_undo = Some(UndoSnapshot {
                        review_id,
                        previous_card: previous_card.clone(),
                        rating,
                    });
                    if rating == lexic_core::model::Rating::Again {
                        session.requeue.push(current_index);
                    }
                    session.revealed = false;
                    let mut next = session.queue.pop_front();
                    if next.is_none() && !session.requeue.is_empty() {
                        next = Some(session.requeue.remove(0));
                    }
                    session.current = next;
                    if session.current.is_none() {
                        session.ended = true;
                    }

                    (
                        previous_card.clone(),
                        previous_card,
                        outcome,
                        review_id,
                        true,
                    )
                };
                let _ = preview_success;
                let Some(store) = self.store.clone() else {
                    self.review_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };

                Task::perform(
                    async move {
                        store
                            .update_card_after_review(
                                previous_card.id,
                                outcome.state,
                                outcome.due_at,
                                outcome.interval_days,
                                outcome.ease_factor,
                                outcome.repetitions,
                                outcome.lapses,
                            )
                            .map_err(|error| error.to_string())?;
                        store
                            .append_review(&lexic_core::model::ReviewLog {
                                id: review_id,
                                card_id: previous_card.id,
                                reviewed_at: now,
                                day: lexic_core::time::local_day_string(now),
                                rating,
                                prev_state: previous_card.state,
                                prev_interval_days: previous_card.interval_days,
                                prev_ease: previous_card.ease_factor,
                                new_state: outcome.state,
                                new_interval_days: outcome.interval_days,
                                new_ease: outcome.ease_factor,
                                duration_ms: None,
                            })
                            .map_err(|error| error.to_string())?;
                        Ok(())
                    },
                    Message::GradeSaved,
                )
            }
            Message::GradeSaved(result) => {
                self.review_status = Some(match result {
                    Ok(_) => String::from("Grade saved"),
                    Err(error) => error,
                });
                Task::none()
            }
            Message::UndoReview => {
                let now = now_utc();
                let Some(session) = &mut self.review else {
                    return Task::none();
                };
                let Some(snapshot) = session.last_undo.clone() else {
                    return Task::none();
                };
                let restored_position = session.last_graded;
                let previous_card = snapshot.previous_card.clone();
                let review_id = snapshot.review_id;
                let rating = snapshot.rating;
                let Some(store) = self.store.clone() else {
                    self.review_status = Some(String::from("Database is not ready"));
                    return Task::none();
                };

                if let Some(position) = restored_position {
                    if let Some(review_session) = &mut self.review {
                        if let Some(card) = review_session.cards.get_mut(position) {
                            card.card = previous_card.clone();
                        }
                        review_session.current = Some(position);
                        review_session.revealed = true;
                        review_session.reviewed = review_session.reviewed.saturating_sub(1);
                        if rating != lexic_core::model::Rating::Again {
                            review_session.correct = review_session.correct.saturating_sub(1);
                        }
                        review_session.last_undo = None;
                        review_session.last_graded = None;
                        review_session.last_rating = None;
                    }
                }

                let _ = now;
                Task::perform(
                    async move {
                        store
                            .restore_card(&previous_card)
                            .map_err(|error| error.to_string())?;
                        store
                            .delete_review(review_id)
                            .map_err(|error| error.to_string())?;
                        Ok(())
                    },
                    Message::UndoFinished,
                )
            }
            Message::UndoFinished(result) => {
                self.review_status = Some(match result {
                    Ok(_) => String::from("Undone"),
                    Err(error) => error,
                });
                Task::none()
            }
            Message::EndReview => {
                self.review = None;
                self.review_status = Some(String::from("Review session ended"));
                self.screen = Screen::Home;
                Task::none()
            }
        }
    }
    fn start_lookup(&mut self) -> Task<Message> {
        let word = self.search_input.trim().to_string();
        if word.is_empty() {
            self.search_status = SearchStatus::Error;
            self.save_status = Some(String::from("Enter a word to look up"));
            return Task::none();
        }

        let normalized = match normalize(&word) {
            Ok(value) => value,
            Err(error) => {
                self.search_status = SearchStatus::Error;
                self.save_status = Some(error.to_string());
                return Task::none();
            }
        };

        self.lookup_generation += 1;
        let id = self.lookup_generation;
        self.search_status = SearchStatus::Loading;
        self.save_status = None;
        self.screen = Screen::Search;

        let client = self.dictionary.clone();
        let store = self.store.clone();
        let now = now_utc();
        let query = normalized;

        Task::perform(
            async move {
                let mut lookup = client
                    .lookup(&query, now)
                    .await
                    .map_err(|error| error.to_string())?;
                if let Some(store) = store.as_ref() {
                    let entry = lookup.entry.clone();
                    let senses = lookup.senses.clone();
                    let upsert_store = store.clone();
                    tokio::task::spawn_blocking(move || {
                        upsert_store
                            .upsert_entry(&entry, &senses)
                            .map_err(|error| error.to_string())
                    })
                    .await
                    .map_err(|error| error.to_string())??;

                    let search_store = store.clone();
                    let normalized_query = query.clone();
                    if let Ok(canonical) = tokio::task::spawn_blocking(move || {
                        search_store
                            .search_entries(&normalized_query, 1)
                            .ok()
                            .and_then(|mut entries| entries.pop())
                    })
                    .await
                    {
                        if let Some(canonical) = canonical {
                            let old_id = lookup.entry.id;
                            lookup.entry = canonical.clone();
                            for sense in &mut lookup.senses {
                                if sense.entry_id == old_id {
                                    sense.entry_id = canonical.id;
                                }
                            }
                        }
                    }
                }
                Ok(lookup)
            },
            move |result| Message::LookupFinished(id, result),
        )
    }

    fn save_active_entry(&mut self) -> Task<Message> {
        let entry = match &self.active_entry {
            Some(value) => value.id,
            None => return Task::none(),
        };
        let deck_id = self
            .selected_save_deck
            .or_else(|| self.decks.first().map(|deck| deck.id));
        let Some(deck_id) = deck_id else {
            self.save_status = Some(String::from("Create a deck first"));
            return Task::none();
        };
        let Some(store) = self.store.clone() else {
            self.save_status = Some(String::from("Database is not ready"));
            return Task::none();
        };

        self.save_status = None;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || store.add_card(entry, deck_id))
                    .await
                    .map_err(|error| error.to_string())?
                    .map_err(|error| error.to_string())
            },
            Message::SaveFinished,
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let labels = [
            ("Home", Screen::Home),
            ("Dictionary", Screen::Search),
            ("Decks", Screen::Decks),
            ("Review", Screen::Review),
            ("Stats", Screen::Stats),
            ("Settings", Screen::Settings),
        ];

        let sidebar_width = if self.sidebar_collapsed {
            Length::Fixed(82.0)
        } else {
            Length::Fixed(236.0)
        };

        let mut nav_items = column![].spacing(6);
        let brand = if self.sidebar_collapsed {
            text("L").size(24).color(Color::from_rgb8(245, 245, 245))
        } else {
            text("Lexic")
                .size(22)
                .color(Color::from_rgb8(245, 245, 245))
        };
        let sidebar_toggle = button(text(">").size(16).color(Color::from_rgb8(245, 245, 245)))
            .padding([6, 10])
            .on_press(Message::ToggleSidebar)
            .style(monochrome_button(false));
        nav_items = nav_items.push(
            row![
                brand,
                iced::widget::Space::new().width(Length::Fill),
                sidebar_toggle
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );

        for (label, screen) in labels {
            let is_active = screen == self.screen;
            let collapsed = self.sidebar_collapsed;
            let nav_label = if collapsed {
                label.chars().next().unwrap().to_string()
            } else {
                label.to_string()
            };
            let mut nav_button = button(
                text(nav_label)
                    .size(15)
                    .color(Color::from_rgb8(245, 245, 245)),
            )
            .width(Length::Fill)
            .padding([10, 12])
            .on_press(Message::Navigate(screen));
            nav_button = nav_button.style(move |_theme: &Theme, status| {
                let hovered = matches!(status, iced::widget::button::Status::Hovered);
                let pressed = matches!(status, iced::widget::button::Status::Pressed);
                let (background, text_color, border) = if is_active {
                    (
                        Color::from_rgb8(54, 54, 54),
                        Color::from_rgb8(245, 245, 245),
                        Color::from_rgb8(86, 86, 86),
                    )
                } else if hovered || pressed {
                    (
                        Color::from_rgb8(34, 34, 34),
                        Color::from_rgb8(245, 245, 245),
                        Color::from_rgb8(74, 74, 74),
                    )
                } else {
                    (
                        Color::from_rgb8(18, 18, 18),
                        Color::from_rgb8(225, 225, 225),
                        Color::from_rgb8(44, 44, 44),
                    )
                };
                iced::widget::button::Style {
                    snap: false,
                    background: Some(background.into()),
                    text_color,
                    border: iced::Border {
                        radius: 10.0.into(),
                        width: 1.0,
                        color: border,
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 14.0,
                    },
                }
            });
            nav_items = nav_items.push(nav_button);
        }

        let sidebar = container(column![nav_items].spacing(10).height(Length::Fill))
            .padding(16)
            .width(sidebar_width)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                snap: false,
                background: Some(Color::from_rgb8(10, 10, 10).into()),
                border: iced::Border {
                    radius: 0.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(30, 30, 30),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
                text_color: Some(Color::from_rgb8(245, 245, 245)),
            });

        let search_input = text_input("Search the dictionary…", &self.search_input)
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::Lookup)
            .padding([10, 14])
            .width(Length::Fill);

        let search_input = search_input.style(|_theme, _status| text_input::Style {
            background: iced::Background::Color(Color::from_rgb8(15, 15, 15)),
            border: iced::Border {
                radius: 10.0.into(),
                width: 1.0,
                color: Color::from_rgb8(44, 44, 44),
            },
            icon: Color::from_rgb8(163, 163, 163),
            placeholder: Color::from_rgb8(115, 115, 115),
            value: Color::from_rgb8(245, 245, 245),
            selection: Color::from_rgb8(50, 50, 50),
        });

        let lookup_button = button(text("Look up").size(14).color(Color::BLACK))
            .padding([10, 16])
            .on_press(Message::Lookup);
        let lookup_button = lookup_button.style(|_theme, status| {
            let (background, text_color) = match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                    (Color::from_rgb8(225, 225, 225), Color::BLACK)
                }
                iced::widget::button::Status::Disabled => (
                    Color::from_rgb8(60, 60, 60),
                    Color::from_rgb8(160, 160, 160),
                ),
                _ => (Color::from_rgb8(236, 236, 236), Color::BLACK),
            };
            iced::widget::button::Style {
                snap: false,
                background: Some(background.into()),
                text_color,
                border: iced::Border {
                    radius: 10.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
            }
        });

        let topbar = container(
            row![search_input, lookup_button,]
                .spacing(12)
                .align_y(Alignment::Center),
        )
        .padding([14, 18])
        .height(Length::Fixed(76.0))
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            snap: false,
            background: Some(Color::from_rgb8(10, 10, 10).into()),
            border: iced::Border {
                radius: 0.0.into(),
                width: 1.0,
                color: Color::from_rgb8(30, 30, 30),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 14.0,
            },
            text_color: Some(Color::from_rgb8(245, 245, 245)),
        });

        let body: Element<'_, Message> = match self.screen {
            Screen::Home => self.home_view(),
            Screen::Search => self.search_view(),
            Screen::Decks => self.decks_view(),
            Screen::DeckDetail(deck_id) => self.deck_detail_view(deck_id),
            Screen::Review => self.review_view(),
            Screen::Stats => self.stats_view(),
            Screen::Settings => self.settings_view(),
        };

        let content =
            scrollable(container(body).padding(28).width(Length::Fill)).height(Length::Fill);

        let main = column![topbar, content].spacing(0).width(Length::Fill);

        let root = row![sidebar, main].height(Length::Fill);

        container(root)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                snap: false,
                background: Some(Color::BLACK.into()),
                border: iced::Border::default(),
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
                text_color: Some(Color::from_rgb8(245, 245, 245)),
            })
            .into()
    }

    fn review_view(&self) -> Element<'_, Message> {
        if let Some(status) = &self.review_status {
            let _ = status;
        }

        let Some(session) = &self.review else {
            return container(
                column![
                    text("Review").size(32),
                    button("Start review session").on_press(Message::StartReview),
                ]
                .spacing(16),
            )
            .padding(32)
            .into();
        };

        if session.current.is_none() {
            let accuracy = if session.reviewed == 0 {
                0.0
            } else {
                session.correct as f64 / session.reviewed as f64 * 100.0
            };
            return container(
                column![
                    text("Session complete").size(32),
                    text(format!("Reviewed: {}", session.reviewed)),
                    text(format!("Accuracy: {accuracy:.0}%")),
                    button("Done").on_press(Message::EndReview),
                ]
                .spacing(12),
            )
            .padding(32)
            .into();
        }

        let Some(current_index) = session.current else {
            return container(text("No card").size(24)).padding(32).into();
        };
        let review_card = &session.cards[current_index];
        let now = now_utc();
        let previews = [
            lexic_core::model::Rating::Again,
            lexic_core::model::Rating::Hard,
            lexic_core::model::Rating::Good,
            lexic_core::model::Rating::Easy,
        ]
        .map(|rating| {
            let outcome =
                Sm2::default().grade(&CardSnapshot::from_card(&review_card.card), rating, now);
            let minutes = (outcome.due_at - now).num_minutes();
            let label = if minutes < 60 {
                format!("{rating:?} {minutes}m")
            } else if minutes < 60 * 24 {
                format!("{rating:?} {}h", minutes / 60)
            } else {
                format!("{rating:?} {}d", (minutes / 60 + 23) / 24)
            };
            (rating, label)
        });

        let front = if let Some(entry) = &review_card.entry {
            column![
                text(&entry.word).size(44),
                entry
                    .phonetic
                    .as_ref()
                    .map(|value| text(value).size(18))
                    .unwrap_or_else(|| text("").size(1)),
            ]
            .spacing(8)
        } else {
            column![text("Unknown word").size(32)].spacing(8)
        };

        if session.revealed {
            let mut back_lines = Vec::new();
            for sense in &review_card.senses {
                back_lines.push(text(&sense.definition).size(18).into());
                if let Some(example) = &sense.example {
                    back_lines.push(text(example).size(14).into());
                }
            }
            let back = column(back_lines).spacing(8);
            let mut content = column![front, back].spacing(24);
            content = content.push(
                row![
                    button(text(previews[0].1.clone()).size(13))
                        .on_press(Message::Grade(previews[0].0)),
                    button(text(previews[1].1.clone()).size(13))
                        .on_press(Message::Grade(previews[1].0)),
                    button(text(previews[2].1.clone()).size(13))
                        .on_press(Message::Grade(previews[2].0)),
                    button(text(previews[3].1.clone()).size(13))
                        .on_press(Message::Grade(previews[3].0)),
                ]
                .spacing(8),
            );
            content = content.push(button("Undo").on_press(Message::UndoReview));
            container(content).padding(32).into()
        } else {
            let content = column![
                front,
                button("Show answer").on_press(Message::RevealAnswer),
                text("Press Space to reveal").size(12),
            ]
            .spacing(16);
            container(content).padding(32).into()
        }
    }

    fn home_view(&self) -> Element<'_, Message> {
        let due_cards: i64 = self.deck_counts.values().map(|(_, _, due)| *due).sum();
        let new_cards: i64 = self.deck_counts.values().map(|(_, new, _)| *new).sum();

        let stat_card = |title: &'static str, value: String| -> Element<'_, Message> {
            container(
                column![
                    text(title).size(13).color(Color::from_rgb8(163, 163, 163)),
                    text(value).size(28).color(Color::from_rgb8(245, 245, 245))
                ]
                .spacing(4),
            )
            .padding(18)
            .width(Length::Fixed(180.0))
            .style(|_theme: &Theme| container::Style {
                snap: false,
                background: Some(Color::from_rgb8(18, 18, 18).into()),
                border: iced::Border {
                    radius: 16.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(42, 42, 42),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
                text_color: Some(Color::from_rgb8(245, 245, 245)),
            })
            .into()
        };

        let mut content = column![
            text("Welcome to Lexic")
                .size(36)
                .color(Color::from_rgb8(245, 245, 245)),
            text("Search from the bar above or jump back into your study flow.")
                .size(15)
                .color(Color::from_rgb8(163, 163, 163)),
            row![
                stat_card("Decks", self.decks.len().to_string()),
                stat_card("Due cards", due_cards.to_string()),
                stat_card("New cards", new_cards.to_string()),
            ]
            .spacing(12),
            row![
                {
                    let button = button(text("Start review").size(15).color(Color::BLACK))
                        .on_press(Message::StartReview)
                        .padding([10, 18]);
                    button.style(|_theme, status| {
                        let (background, text_color) = match status {
                            iced::widget::button::Status::Hovered
                            | iced::widget::button::Status::Pressed => {
                                (Color::from_rgb8(225, 225, 225), Color::BLACK)
                            }
                            _ => (Color::from_rgb8(236, 236, 236), Color::BLACK),
                        };
                        iced::widget::button::Style {
                            snap: false,
                            background: Some(background.into()),
                            text_color,
                            border: iced::Border {
                                radius: 10.0.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: iced::Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                                offset: iced::Vector::new(0.0, 6.0),
                                blur_radius: 14.0,
                            },
                        }
                    })
                },
                {
                    let button = button(text("Browse decks").size(15).color(Color::BLACK))
                        .on_press(Message::Navigate(Screen::Decks))
                        .padding([10, 18]);
                    button.style(|_theme, status| {
                        let (background, text_color) = match status {
                            iced::widget::button::Status::Hovered
                            | iced::widget::button::Status::Pressed => {
                                (Color::from_rgb8(225, 225, 225), Color::BLACK)
                            }
                            _ => (Color::from_rgb8(236, 236, 236), Color::BLACK),
                        };
                        iced::widget::button::Style {
                            snap: false,
                            background: Some(background.into()),
                            text_color,
                            border: iced::Border {
                                radius: 10.0.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: iced::Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                                offset: iced::Vector::new(0.0, 6.0),
                                blur_radius: 14.0,
                            },
                        }
                    })
                },
            ]
            .spacing(12),
        ]
        .spacing(18);

        if let Some(entry) = &self.active_entry {
            content = content.push(
                container(
                    column![
                        text("Last looked up")
                            .size(13)
                            .color(Color::from_rgb8(163, 163, 163)),
                        text(&entry.word)
                            .size(24)
                            .color(Color::from_rgb8(245, 245, 245)),
                        text(&entry.source)
                            .size(12)
                            .color(Color::from_rgb8(115, 115, 115)),
                    ]
                    .spacing(4),
                )
                .padding(18)
                .width(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    snap: false,
                    background: Some(Color::from_rgb8(18, 18, 18).into()),
                    border: iced::Border {
                        radius: 16.0.into(),
                        width: 1.0,
                        color: Color::from_rgb8(42, 42, 42),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 14.0,
                    },
                    text_color: Some(Color::from_rgb8(245, 245, 245)),
                }),
            );
        }

        container(content).padding(8).into()
    }

    fn search_view(&self) -> Element<'_, Message> {
        let mut content = column![].spacing(16);

        match self.search_status {
            SearchStatus::Loading => content = content.push(text("Loading definition...")),
            SearchStatus::Error => {}
            _ => {}
        }

        if let Some(status) = &self.save_status {
            content = content.push(text(status).size(14));
        }

        if let Some(entry) = &self.active_entry {
            let title: Element<'_, Message> = text(&entry.word).size(44).into();
            let phonetic = entry
                .phonetic
                .as_ref()
                .map(|value| text(value).size(18))
                .unwrap_or_else(|| text("").size(1));
            let mut body_lines = Vec::new();
            for sense in &self.active_senses {
                body_lines.push(text(&sense.definition).size(16).into());
                if let Some(example) = &sense.example {
                    body_lines.push(text(example).size(14).into());
                }
            }
            let body_text = column(body_lines).spacing(8);
            let details = container(
                column![title, phonetic, text(&entry.source).size(12), body_text].spacing(12),
            )
            .padding(24)
            .width(Length::Fill);

            let mut deck_buttons = column![text("Save to deck").size(16)].spacing(8);
            let raw_buttons = self
                .decks
                .iter()
                .fold(column![].spacing(8), |mut column, deck| {
                    column = column.push(
                        button(text(&deck.name).size(14))
                            .width(Length::Fill)
                            .on_press(Message::SelectSaveDeck(deck.id)),
                    );
                    column
                });
            deck_buttons = deck_buttons.push(raw_buttons);
            if self.decks.is_empty() {
                deck_buttons = deck_buttons
                    .push(text("Create a deck in the Decks tab to save words").size(12));
            }
            deck_buttons = deck_buttons
                .push(button("Save to selected deck").on_press(Message::SaveActiveEntry));

            content = content.push(details);
            content = content.push(container(deck_buttons).width(Length::Fill));
        }

        container(content).padding(32).into()
    }

    fn decks_view(&self) -> Element<'_, Message> {
        let mut content = column![
            text("Decks").size(32),
            row![
                text_input("New deck name", &self.new_deck_name)
                    .on_input(Message::NewDeckNameChanged)
                    .padding([10, 14])
                    .width(Length::Fill)
                    .style(|_theme, _status| text_input::Style {
                        background: iced::Background::Color(Color::from_rgb8(15, 15, 15)),
                        border: iced::Border {
                            radius: 10.0.into(),
                            width: 1.0,
                            color: Color::from_rgb8(44, 44, 44),
                        },
                        icon: Color::from_rgb8(163, 163, 163),
                        placeholder: Color::from_rgb8(115, 115, 115),
                        value: Color::from_rgb8(245, 245, 245),
                        selection: Color::from_rgb8(50, 50, 50),
                    }),
                button("Create deck").on_press(Message::CreateDeck),
            ]
            .spacing(8),
        ]
        .spacing(16);

        if let Some(status) = &self.deck_management_status {
            content = content.push(text(status).size(14));
        }

        for deck in &self.decks {
            let counts = self.deck_counts.get(&deck.id).copied().unwrap_or((0, 0, 0));
            let row_view = row![
                column![
                    text(&deck.name).size(20),
                    text(format!(
                        "Cards: {} · New: {} · Due: {}",
                        counts.0, counts.1, counts.2
                    ))
                    .size(12),
                ]
                .spacing(4)
                .width(Length::Fill),
                button("Cards").on_press(Message::OpenDeckDetail(deck.id)),
                button("Edit").on_press(Message::SelectManageDeck(deck.id)),
                button("Archive").on_press(Message::ArchiveDeck(deck.id)),
                button("Delete").on_press(Message::DeleteDeck(deck.id)),
            ]
            .spacing(8);

            content = content.push(container(row_view).padding(12).width(Length::Fill));
        }

        if let Some(selected_id) = self.selected_deck {
            if let Some(_deck) = self.decks.iter().find(|deck| deck.id == selected_id) {
                let editor = column![
                    text("Edit selected deck").size(18),
                    text_input("Deck name", &self.rename_deck_input)
                        .on_input(Message::RenameDeckNameChanged)
                        .width(Length::Fill),
                    row![
                        text_input("New cards per day", &self.deck_new_limit_input)
                            .on_input(Message::DeckNewLimitChanged)
                            .width(Length::Fill),
                        text_input("Reviews per day", &self.deck_review_limit_input)
                            .on_input(Message::DeckReviewLimitChanged)
                            .width(Length::Fill),
                        button("Save details").on_press(Message::SaveDeckDetails),
                    ]
                    .spacing(8),
                ]
                .spacing(8);

                content = content.push(container(editor).padding(16).width(Length::Fill));
            }
        }

        container(content).padding(32).into()
    }

    fn deck_detail_view(&self, deck_id: lexic_core::model::Id) -> Element<'_, Message> {
        let deck_name = self
            .decks
            .iter()
            .find(|deck| deck.id == deck_id)
            .map(|deck| deck.name.clone())
            .unwrap_or_else(|| String::from("Deck"));

        let mut content = column![
            text(format!("Deck: {deck_name}")).size(32),
            button("Back to decks").on_press(Message::Navigate(Screen::Decks)),
        ]
        .spacing(16);

        if let Some(status) = &self.deck_card_status {
            content = content.push(text(status).size(14));
        }

        for row_data in &self.deck_cards {
            let card_row = row![
                text(&row_data.word).size(18).width(Length::Fill),
                text(row_data.card.state.to_string()).size(12),
                button("Suspend").on_press(Message::SuspendCard(row_data.card.id)),
                button("Reset").on_press(Message::ResetCard(row_data.card.id)),
                button("Remove").on_press(Message::RemoveCard(row_data.card.id)),
            ]
            .spacing(8);

            content = content.push(container(card_row).padding(12).width(Length::Fill));
        }

        if self.deck_cards.is_empty() {
            content = content.push(text("No cards in this deck"));
        }

        container(content).padding(32).into()
    }

    fn stats_view(&self) -> Element<'_, Message> {
        let Some(data) = &self.stats_data else {
            return container(
                column![
                    text("Stats").size(32),
                    button("Load stats").on_press(Message::LoadStats),
                ]
                .spacing(12),
            )
            .padding(32)
            .into();
        };

        let retention_text = format!("Retention: {:.0}%", data.retention * 100.0);
        let duration_seconds = data.total_duration_ms / 1000;
        let time_text = format!(
            "Study time: {}m {}s · Reviews: {}",
            duration_seconds / 60,
            duration_seconds % 60,
            data.total_reviews
        );
        let streak_text = format!(
            "Current streak: {} days · Best streak: {} days",
            data.current_streak, data.best_streak
        );

        let range_slider = slider(30.0..=365.0, self.stats_range as f32, |value| {
            Message::SetStatsRange(value as u32)
        })
        .step(5.0_f32)
        .width(Length::Fill);

        let max_forecast = data
            .due_forecast
            .iter()
            .map(|entry| entry.1)
            .max()
            .unwrap_or(1)
            .max(1);
        let mut forecast_bars = row![].spacing(2);
        for (_day, count) in &data.due_forecast {
            let height = 4.0 + 96.0 * (*count as f32 / max_forecast as f32);
            forecast_bars = forecast_bars.push(
                container(text(count.to_string()).size(8))
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(height))
                    .style(move |_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb8(
                            112, 112, 112,
                        ))),
                        border: iced::Border::default(),
                        shadow: iced::Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 14.0,
                        },
                        snap: false,
                        text_color: None,
                    }),
            );
        }

        let max_heat = data
            .reviews_by_day
            .values()
            .map(|entry| entry.count)
            .max()
            .unwrap_or(1)
            .max(1);
        let mut heatmap = column![].spacing(2);
        let days: Vec<&String> = data.reviews_by_day.keys().collect();
        for chunk in days.chunks(14) {
            let mut row_view = row![].spacing(2);
            for day in chunk {
                let count = data
                    .reviews_by_day
                    .get(*day)
                    .map(|entry| entry.count)
                    .unwrap_or(0);
                let intensity = count as f32 / max_heat as f32;
                let channel = (232.0 - 180.0 * intensity) as u8;
                row_view = row_view.push(
                    container(
                        text(if count == 0 {
                            String::new()
                        } else {
                            count.to_string()
                        })
                        .size(8),
                    )
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .style(move |_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb8(
                            channel, channel, channel,
                        ))),
                        border: iced::Border::default(),
                        snap: false,
                        shadow: iced::Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 14.0,
                        },
                        text_color: Some(iced::Color::from_rgb8(245, 245, 245)),
                    }),
                );
            }
            heatmap = heatmap.push(row_view);
        }

        let content = column![
            text("Stats").size(32),
            range_slider,
            text(retention_text).size(18),
            text(time_text).size(18),
            text(streak_text).size(18),
            text("Due forecast (30 days)").size(18),
            container(forecast_bars).padding(8).width(Length::Fill),
            text("Review heatmap").size(18),
            container(heatmap).padding(8).width(Length::Fill),
        ]
        .spacing(16);

        container(content).padding(32).into()
    }

    fn settings_view(&self) -> Element<'_, Message> {
        let backup_row = row![
            button("Export backup").on_press(Message::BeginExport),
            button("Import backup").on_press(Message::BeginImport),
        ]
        .spacing(8);

        let mut content = column![
            text("Settings").size(32),
            text(format!(
                "Include review history in exports: {}",
                self.include_review_history
            ))
            .size(16),
            button("Toggle review history").on_press(Message::ToggleIncludeReviewHistory),
            backup_row,
        ]
        .spacing(16);

        if let Some(status) = &self.settings_status {
            content = content.push(text(status).size(14));
        }

        container(content).padding(32).into()
    }
    pub fn title(&self) -> String {
        String::from("Lexic")
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn style(&self, _theme: &Theme) -> iced::theme::Style {
        iced::theme::Style {
            background_color: Color::BLACK,
            text_color: Color::from_rgb8(245, 245, 245),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen().map(Message::Event)
    }
}

fn rand_u128() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
