#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_match)]

use lexic_core::model::{Card, Deck, Entry, Sense};
use lexic_core::schedule::{CardSnapshot, Scheduler, Sm2};
use lexic_core::time::now_utc;
use lexic_dict::{DictionaryClient, LookupResult, normalize};
use lexic_store::Store;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use iced::keyboard;
use iced::{Color, Element, Task, Theme, widget::button as iced_button};

pub(crate) fn monochrome_button(
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

#[allow(dead_code)]
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
    Project,
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
    pub(crate) screen: Screen,
    pub(crate) sidebar_collapsed: bool,
    pub(crate) store: Option<Arc<Store>>,
    pub(crate) dictionary: DictionaryClient,
    pub(crate) decks: Vec<Deck>,
    pub(crate) deck_counts: BTreeMap<lexic_core::model::Id, (i64, i64, i64)>,
    pub(crate) search_input: String,
    pub(crate) lookup_generation: u64,
    pub(crate) search_status: SearchStatus,
    pub(crate) active_entry: Option<Entry>,
    pub(crate) active_senses: Vec<Sense>,
    pub(crate) selected_save_deck: Option<lexic_core::model::Id>,
    pub(crate) save_status: Option<String>,

    pub(crate) new_deck_name: String,
    pub(crate) selected_deck: Option<lexic_core::model::Id>,
    pub(crate) rename_deck_input: String,
    pub(crate) deck_new_limit_input: String,
    pub(crate) deck_review_limit_input: String,
    pub(crate) deck_management_status: Option<String>,
    pub(crate) deck_cards: Vec<DeckCard>,
    pub(crate) deck_card_status: Option<String>,
    #[allow(dead_code)]
    pub(crate) theme_mode: ThemeMode,
    pub(crate) include_review_history: bool,
    pub(crate) settings_status: Option<String>,
    pub(crate) stats_range: u32,
    pub(crate) stats_data: Option<StatsSnapshot>,

    pub(crate) gset_project_path: String,
    pub(crate) gset_target_language: String,
    pub(crate) gset_status: Option<String>,
    pub(crate) gset_result: Option<crate::gset::GsetRunResult>,

    pub(crate) gset_editor_path: String,
    pub(crate) gset_editor_content: iced::widget::text_editor::Content,
    pub(crate) gset_editor_status: Option<String>,

    pub(crate) review: Option<ReviewSession>,
    pub(crate) review_status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeckCard {
    pub card: Card,
    pub word: String,
}

#[derive(Debug, Clone)]
pub struct ReviewSession {
    pub(crate) cards: Vec<ReviewCard>,
    pub(crate) queue: VecDeque<usize>,
    pub(crate) requeue: Vec<usize>,
    pub(crate) current: Option<usize>,
    pub(crate) revealed: bool,
    pub(crate) reviewed: usize,
    pub(crate) correct: usize,
    pub(crate) last_graded: Option<usize>,
    pub(crate) last_undo: Option<UndoSnapshot>,
    pub(crate) last_rating: Option<lexic_core::model::Rating>,
    pub(crate) ended: bool,
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
    GsetProjectPathChanged(String),
    GsetTargetLanguageChanged(String),
    RunGsetTranspile,
    GsetTranspileFinished(Result<crate::gset::GsetRunResult, String>),
    GsetEditorPathChanged(String),
    LoadGsetEditorFile,
    GsetEditorFileLoaded(Result<String, String>),
    GsetEditorAction(iced::widget::text_editor::Action),
    SaveGsetEditorFile,
    SaveGsetEditorFinished(Result<(), String>),
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

            gset_project_path: String::new(),
            gset_target_language: String::from("go"),
            gset_status: None,
            gset_result: None,

            gset_editor_path: String::new(),
            gset_editor_content: iced::widget::text_editor::Content::new(),
            gset_editor_status: None,
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
            Message::GsetProjectPathChanged(value) => {
                self.gset_project_path = value;
                Task::none()
            }
            Message::GsetTargetLanguageChanged(value) => {
                self.gset_target_language = value;
                Task::none()
            }
            Message::RunGsetTranspile => {
                let path = self.gset_project_path.trim().to_string();
                if path.is_empty() {
                    self.gset_status = Some(String::from("Enter a project path"));
                    return Task::none();
                }
                let target = self.gset_target_language.trim().to_string();
                self.gset_status = Some(String::from("Running GSET transpilation..."));
                let store = self.store.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let target = if target.is_empty() {
                                String::from("go")
                            } else {
                                target
                            };
                            let project_id = if let Some(store) = store.as_ref() {
                                Some(store.ensure_project(&path, &target).map_err(|e| e.to_string())?)
                            } else {
                                None
                            };
                            let run_id = if let (Some(store), Some(project_id)) =
                                (store.as_ref(), project_id)
                            {
                                Some(store.start_transpile_run(project_id, &target).map_err(|e| e.to_string())?)
                            } else {
                                None
                            };

                            let result = crate::gset::transpile(
                                std::path::Path::new(&path),
                                &target,
                                None,
                                None,
                                true,
                                false,
                            )?;

                            if let (Some(store), Some(run_id)) = (store.as_ref(), run_id) {
                                let status = if result.success { "success" } else { "failed" };
                                let summary = if result.success {
                                    Some(format!(
                                        "GSET transpilation completed with {} diagnostics",
                                        result.diagnostics.len()
                                    ))
                                } else {
                                    Some(String::from("GSET transpilation failed"))
                                };
                                store
                                    .finish_transpile_run(
                                        run_id,
                                        status,
                                        result.exit_code,
                                        result.diagnostics.len() as i32,
                                        summary.as_deref(),
                                    )
                                    .map_err(|e| e.to_string())?;
                            }

                            Ok(result)
                        })
                        .await
                        .map_err(|error| error.to_string())?
                    },
                    Message::GsetTranspileFinished,
                )
            }
            Message::GsetTranspileFinished(result) => {
                match result {
                    Ok(result) => {
                        self.gset_status = Some(if result.success {
                            String::from("GSET transpilation completed")
                        } else {
                            String::from("GSET transpilation failed")
                        });
                        self.gset_result = Some(result);
                    }
                    Err(error) => {
                        self.gset_status = Some(error);
                    }
                }
                Task::none()
            }
            Message::GsetEditorPathChanged(value) => {
                self.gset_editor_path = value;
                Task::none()
            }
            Message::LoadGsetEditorFile => {
                let path = self.gset_editor_path.trim().to_string();
                if path.is_empty() {
                    self.gset_editor_status = Some(String::from("Enter a file path"));
                    return Task::none();
                }
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            std::fs::read_to_string(&path).map_err(|error| error.to_string())
                        })
                        .await
                        .map_err(|error| error.to_string())?
                    },
                    Message::GsetEditorFileLoaded,
                )
            }
            Message::GsetEditorFileLoaded(result) => {
                match result {
                    Ok(content) => {
                        self.gset_editor_content = iced::widget::text_editor::Content::with_text(&content);
                        self.gset_editor_status = Some(String::from("File loaded"));
                    }
                    Err(error) => self.gset_editor_status = Some(error),
                }
                Task::none()
            }
            Message::GsetEditorAction(action) => {
                self.gset_editor_content.perform(action);
                Task::none()
            }
            Message::SaveGsetEditorFile => {
                let path = self.gset_editor_path.trim().to_string();
                if path.is_empty() {
                    self.gset_editor_status = Some(String::from("Enter a file path"));
                    return Task::none();
                }
                let content = self.gset_editor_content.text();
                let store = self.store.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            std::fs::write(&path, &content).map_err(|error| error.to_string())?;
                            if let Some(store) = store {
                                let root = std::path::Path::new(&path)
                                    .parent()
                                    .and_then(|value| value.to_str())
                                    .filter(|value| !value.trim().is_empty())
                                    .unwrap_or(".")
                                    .to_string();
                                let project_id = store.ensure_project(&root, "go").map_err(|error| error.to_string())?;
                                store.add_manual_edit(project_id, &path, &content).map_err(|error| error.to_string())?;
                            }
                            Ok(())
                        })
                        .await
                        .map_err(|error| error.to_string())?
                    },
                    Message::SaveGsetEditorFinished,
                )
            }
            Message::SaveGsetEditorFinished(result) => {
                match result {
                    Ok(()) => self.gset_editor_status = Some(String::from("File saved")),
                    Err(error) => self.gset_editor_status = Some(error),
                }
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
}

impl App {
    pub fn title(&self) -> String {
        String::from("Lexic")
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn style(&self, _theme: &Theme) -> iced::theme::Style {
        iced::theme::Style {
            background_color: Color::from_rgb8(15, 23, 42),
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
