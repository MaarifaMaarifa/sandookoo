use iced::Element;
use iced::widget::{center, text};

mod gui_state;

pub struct Gui {
    state: gui_state::GuiState,
}

pub enum Message {}

impl Gui {
    pub fn new() -> Self {
        Self {
            state: gui_state::GuiState::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {}
    }

    pub fn view(&self) -> Element<'_, Message> {
        center(text("hello world")).into()
    }
}
