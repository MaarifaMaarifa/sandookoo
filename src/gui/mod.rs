use iced::widget::row;
use iced::{Element, Task};

mod connections_panel;
mod query_panel;
mod state;
pub mod style;

pub struct Gui {
    state: state::GuiState,
    connections: connections_panel::State,
    query: query_panel::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Connections(connections_panel::Message),
    Query(query_panel::Message),
}

impl Gui {
    pub fn new() -> Self {
        Self {
            state: state::GuiState::new(),
            connections: connections_panel::State::new(),
            query: query_panel::State::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connections(message) => self
                .connections
                .update(message, &mut self.state)
                .map(Message::Connections),
            Message::Query(message) => self.query.update(message, &self.state).map(Message::Query),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![
            self.connections.view(&self.state).map(Message::Connections),
            self.query.view().map(Message::Query),
        ]
        .spacing(style::space::MD)
        .padding(style::space::LG)
        .into()
    }
}
