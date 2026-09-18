use iced::widget::{button, center, container, mouse_area, opaque, row, stack, text};
use iced::{Color, Element, Length, Task, Theme};

mod connections_panel;
mod query_panel;
mod settings_modal;
mod state;
pub mod style;

use crate::settings::Settings;

pub struct Gui {
    state: state::GuiState,
    connections: connections_panel::State,
    query: query_panel::State,
    settings_modal: settings_modal::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Connections(connections_panel::Message),
    Query(query_panel::Message),
    Settings(settings_modal::Message),
}

impl Gui {
    pub fn new() -> (Self, Task<Message>) {
        let settings = Settings::load();
        let theme = Theme::ALL
            .iter()
            .find(|theme| theme.to_string() == settings.theme)
            .cloned()
            .unwrap_or(Theme::CatppuccinMocha);
        let settings_modal = settings_modal::State::new(
            theme,
            settings.ui_font.clone(),
            settings.editor_font.clone(),
        );
        let (connections, reconnect_task) =
            connections_panel::State::new(settings.connections.clone());

        (
            Self {
                state: state::GuiState::new(settings),
                connections,
                query: query_panel::State::new(),
                settings_modal,
            },
            reconnect_task.map(Message::Connections),
        )
    }

    pub fn theme(&self) -> Theme {
        self.settings_modal.theme()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connections(message) => self
                .connections
                .update(message, &mut self.state)
                .map(Message::Connections),
            Message::Query(message) => self.query.update(message, &self.state).map(Message::Query),
            Message::Settings(message) => self
                .settings_modal
                .update(message, &mut self.state)
                .map(Message::Settings),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme = self.settings_modal.theme();

        let toolbar = container(
            button(text("⚙ Settings"))
                .padding([style::space::XS, style::space::SM])
                .style(style::ghost_button)
                .on_press(Message::Settings(settings_modal::Message::Open)),
        )
        .align_right(Length::Fill);

        let panels = row![
            self.connections.view(&self.state).map(Message::Connections),
            self.query
                .view(&theme, self.state.editor_font())
                .map(Message::Query),
        ]
        .spacing(style::space::MD);

        let base: Element<'_, Message> = iced::widget::column![toolbar, panels]
            .spacing(style::space::MD)
            .padding(style::space::LG)
            .into();

        if let Some(dialog) = self.connections.modal() {
            return modal(
                base,
                dialog.map(Message::Connections),
                Message::Connections(connections_panel::Message::CancelNewConnectionForm),
            );
        }

        if let Some(dialog) = self.settings_modal.modal() {
            return modal(
                base,
                dialog.map(Message::Settings),
                Message::Settings(settings_modal::Message::Close),
            );
        }

        base
    }
}

/// Layers `content` centered over `base`, dimmed by a click-to-dismiss
/// backdrop that sends `on_dismiss`.
fn modal<'a>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_dismiss: Message,
) -> Element<'a, Message> {
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_dismiss)
        )
    ]
    .into()
}
