#[cfg(feature = "ui")]
mod app;
#[cfg(feature = "ui")]
mod components;
#[cfg(feature = "ui")]
#[cfg(feature = "ui")]
mod theme;
#[cfg(feature = "ui")]
mod views;

#[cfg(feature = "ui")]
fn prepare() -> (
    Option<std::sync::Arc<lexic_store::Store>>,
    Option<std::path::PathBuf>,
    Vec<lexic_core::model::Deck>,
) {
    let project = directories::ProjectDirs::from("dev", "Lexic", "Lexic");
    let (store, path) = if let Some(project) = project {
        let data_dir = project.data_dir().to_path_buf();
        let db_path = data_dir.join("lexic.db");
        let store = if std::fs::create_dir_all(&data_dir).is_ok() {
            lexic_store::Store::open(&db_path)
                .ok()
                .map(std::sync::Arc::new)
        } else {
            None
        };
        (store, Some(db_path))
    } else {
        (None, None)
    };
    let decks = store
        .as_ref()
        .map(|value| value.list_decks().unwrap_or_default())
        .unwrap_or_default();
    (store, path, decks)
}

#[cfg(feature = "ui")]
#[cfg(feature = "ui")]
fn init_logging() {
    let _ = tracing_subscriber::fmt().try_init();
}
#[cfg(feature = "ui")]
fn main() -> iced::Result {
    init_logging();
    let (store, database_path, decks) = prepare();
    let boot = move || app::App::new(store.clone(), database_path.clone(), decks.clone());
    let window = iced::window::Settings {
        size: (1100.0, 760.0).into(),
        ..Default::default()
    };
    iced::application(boot, app::App::update, app::App::view)
        .title(app::App::title)
        .theme(app::App::theme)
        .window(window)
        .style(app::App::style)
        .subscription(app::App::subscription)
        .font(include_bytes!("../../../assets/fonts/Inter-Regular.ttf"))
        .font(include_bytes!("../../../assets/fonts/Inter-Medium.ttf"))
        .font(include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf"))
        .font(include_bytes!("../../../assets/fonts/Inter-Bold.ttf"))
        .font(include_bytes!(
            "../../../assets/fonts/JetBrainsMono-Regular.ttf"
        ))
        .font(include_bytes!(
            "../../../assets/fonts/JetBrainsMono-Medium.ttf"
        ))
        .font(iced_fonts::LUCIDE_FONT_BYTES)
        .default_font(iced::Font::with_name("Inter"))
        .run()
}

#[cfg(not(feature = "ui"))]
fn main() {
    println!("Lexic GUI feature is disabled; build with --features ui");
}
