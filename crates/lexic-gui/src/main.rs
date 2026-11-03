#[cfg(feature = "ui")]
mod app;

#[cfg(feature = "ui")]
fn prepare() -> (
    Option<std::sync::Arc<lexic_store::Store>>,
    Vec<lexic_core::model::Deck>,
) {
    let store = (|| {
        let project = directories::ProjectDirs::from("dev", "Lexic", "Lexic")?;
        let data_dir = project.data_dir();
        std::fs::create_dir_all(data_dir).ok()?;
        lexic_store::Store::open(data_dir.join("lexic.db"))
            .ok()
            .map(std::sync::Arc::new)
    })();
    let decks = store
        .as_ref()
        .map(|value| value.list_decks().unwrap_or_default())
        .unwrap_or_default();
    (store, decks)
}

#[cfg(feature = "ui")]
#[cfg(feature = "ui")]
fn init_logging() {
    let _ = tracing_subscriber::fmt().try_init();
}
#[cfg(feature = "ui")]
fn main() -> iced::Result {
    init_logging();
    let (store, decks) = prepare();
    let boot = move || app::App::new(store.clone(), decks.clone());
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
        .run()
}

#[cfg(not(feature = "ui"))]
fn main() {
    println!("Lexic GUI feature is disabled; build with --features ui");
}
