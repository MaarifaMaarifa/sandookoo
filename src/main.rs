mod database;
mod gui;
mod settings;

fn main() -> iced::Result {
    iced::application(gui::Gui::new, gui::Gui::update, gui::Gui::view)
        .title("Sandookoo")
        .theme(gui::style::theme)
        .executor::<iced_futures::backend::native::tokio::Executor>()
        .run()
}
