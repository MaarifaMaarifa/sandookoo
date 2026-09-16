use iced::widget::{pick_list, row, text};
use iced::{Alignment, Element, Task, Theme};

mod connections_panel;
mod query_panel;
mod state;
pub mod style;

use crate::settings::Settings;

pub struct Gui {
    state: state::GuiState,
    connections: connections_panel::State,
    query: query_panel::State,
    theme: Theme,
}

#[derive(Debug, Clone)]
pub enum Message {
    Connections(connections_panel::Message),
    Query(query_panel::Message),
    ThemeSelected(Theme),
}

impl Gui {
    pub fn new() -> (Self, Task<Message>) {
        let settings = Settings::load();
        let theme = Theme::ALL
            .iter()
            .find(|theme| theme.to_string() == settings.theme)
            .cloned()
            .unwrap_or(Theme::CatppuccinMocha);
        let (connections, reconnect_task) =
            connections_panel::State::new(settings.connections.clone());

        (
            Self {
                state: state::GuiState::new(settings),
                connections,
                query: query_panel::State::new(),
                theme,
            },
            reconnect_task.map(Message::Connections),
        )
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connections(message) => self
                .connections
                .update(message, &mut self.state)
                .map(Message::Connections),
            Message::Query(message) => self.query.update(message, &self.state).map(Message::Query),
            Message::ThemeSelected(theme) => {
                self.theme = theme.clone();
                self.state.set_theme(&theme);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let toolbar = row![
            text("Theme"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeSelected),
        ]
        .spacing(style::space::SM)
        .align_y(Alignment::Center);

        let panels = row![
            self.connections.view(&self.state).map(Message::Connections),
            self.query.view().map(Message::Query),
        ]
        .spacing(style::space::MD);

        iced::widget::column![toolbar, panels]
            .spacing(style::space::MD)
            .padding(style::space::LG)
            .into()
    }
}
