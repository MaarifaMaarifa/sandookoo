mod database;
mod gui;
mod settings;

/// The app's name, read once from `Cargo.toml` so the window title,
/// settings directory, and keychain service all agree on it.
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> iced::Result {
    iced::application(gui::Gui::new, gui::Gui::update, gui::Gui::view)
        .title(APP_NAME)
        .theme(gui::style::theme)
        .executor::<iced_futures::backend::native::tokio::Executor>()
        .run()
}
