mod database;
mod gui;
mod settings;

/// The app's name, read once from `Cargo.toml` so the window title,
/// settings directory, and keychain service all agree on it.
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(format!("warn,{APP_NAME}=info"))
            }),
        )
        .init();

    // The UI's default font is fixed for the life of the renderer, so it
    // has to be read before the application boots; changing it takes
    // effect on next launch. The editor font, by contrast, is applied
    // per-render from `GuiState` and updates live.
    let ui_font = gui::style::resolve_font(
        settings::Settings::load().ui_font.as_deref(),
        iced::Font::DEFAULT,
    );

    iced::application(gui::Gui::new, gui::Gui::update, gui::Gui::view)
        .title(APP_NAME)
        .theme(gui::style::theme)
        .default_font(ui_font)
        .executor::<iced_futures::backend::native::tokio::Executor>()
        .run()
}
