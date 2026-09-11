mod database;
mod gui;

fn main() -> iced::Result {
    iced::application(gui::Gui::new, gui::Gui::update, gui::Gui::view)
        .title("Sandookoo")
        .executor::<iced_futures::backend::native::tokio::Executor>()
        .run()
}
