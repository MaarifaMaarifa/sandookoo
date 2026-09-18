use iced::widget::{
    center, column, container, mouse_area, opaque, pick_list, row, stack, text, text_input,
};
use iced::{Alignment, Color, Element, Task, Theme};

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
    ui_font_input: String,
    editor_font_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Connections(connections_panel::Message),
    Query(query_panel::Message),
    ThemeSelected(Theme),
    UiFontChanged(String),
    UiFontSubmitted,
    EditorFontChanged(String),
    EditorFontSubmitted,
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
        let ui_font_input = settings.ui_font.clone().unwrap_or_default();
        let editor_font_input = settings.editor_font.clone().unwrap_or_default();

        (
            Self {
                state: state::GuiState::new(settings),
                connections,
                query: query_panel::State::new(),
                theme,
                ui_font_input,
                editor_font_input,
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
            Message::UiFontChanged(value) => {
                self.ui_font_input = value;
                Task::none()
            }
            Message::UiFontSubmitted => {
                self.state.set_ui_font(non_empty(&self.ui_font_input));
                Task::none()
            }
            Message::EditorFontChanged(value) => {
                self.editor_font_input = value;
                Task::none()
            }
            Message::EditorFontSubmitted => {
                self.state
                    .set_editor_font(non_empty(&self.editor_font_input));
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let toolbar = row![
            text("Theme"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeSelected),
            text("UI font (restart to apply)"),
            text_input("System default", &self.ui_font_input)
                .width(160)
                .style(style::field)
                .on_input(Message::UiFontChanged)
                .on_submit(Message::UiFontSubmitted),
            text("Editor font"),
            text_input("Monospace", &self.editor_font_input)
                .width(160)
                .style(style::field)
                .on_input(Message::EditorFontChanged)
                .on_submit(Message::EditorFontSubmitted),
        ]
        .spacing(style::space::SM)
        .align_y(Alignment::Center);

        let panels = row![
            self.connections.view(&self.state).map(Message::Connections),
            self.query
                .view(&self.theme, self.state.editor_font())
                .map(Message::Query),
        ]
        .spacing(style::space::MD);

        let base: Element<'_, Message> = column![toolbar, panels]
            .spacing(style::space::MD)
            .padding(style::space::LG)
            .into();

        match self.connections.modal() {
            Some(dialog) => modal(
                base,
                dialog.map(Message::Connections),
                Message::Connections(connections_panel::Message::CancelNewConnectionForm),
            ),
            None => base,
        }
    }
}

/// Trims `value` and turns it into `Some` unless it's empty, for the font
/// inputs: an empty field means "use the default", not a font named "".
fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
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
