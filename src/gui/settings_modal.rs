use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Element, Length, Task, Theme};

use super::state::GuiState;
use super::style;

/// A section of the settings dialog. Only one exists today, but the
/// sidebar/content split exists so more (e.g. a "General" or
/// "Connections" section) can be added without restructuring the modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Appearance,
}

impl Category {
    const ALL: &'static [Self] = &[Self::Appearance];

    fn label(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
        }
    }
}

pub struct State {
    open: bool,
    category: Category,
    theme: Theme,
    ui_font_input: String,
    editor_font_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open,
    Close,
    SelectCategory(Category),
    ThemeSelected(Theme),
    UiFontChanged(String),
    UiFontSubmitted,
    EditorFontChanged(String),
    EditorFontSubmitted,
}

impl State {
    pub fn new(theme: Theme, ui_font: Option<String>, editor_font: Option<String>) -> Self {
        Self {
            open: false,
            category: Category::Appearance,
            theme,
            ui_font_input: ui_font.unwrap_or_default(),
            editor_font_input: editor_font.unwrap_or_default(),
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    /// `shared` is where font and theme choices get persisted; this panel
    /// owns the dialog's open/closed state and its draft input values.
    pub fn update(&mut self, message: Message, shared: &mut GuiState) -> Task<Message> {
        match message {
            Message::Open => self.open = true,
            Message::Close => self.open = false,
            Message::SelectCategory(category) => self.category = category,
            Message::ThemeSelected(theme) => {
                self.theme = theme.clone();
                shared.set_theme(&theme);
            }
            Message::UiFontChanged(value) => self.ui_font_input = value,
            Message::UiFontSubmitted => shared.set_ui_font(non_empty(&self.ui_font_input)),
            Message::EditorFontChanged(value) => self.editor_font_input = value,
            Message::EditorFontSubmitted => {
                shared.set_editor_font(non_empty(&self.editor_font_input));
            }
        }

        Task::none()
    }

    /// The dialog itself, to be shown as a modal over the whole app while
    /// it's open.
    pub fn modal(&self) -> Option<Element<'_, Message>> {
        if !self.open {
            return None;
        }

        let sidebar = column(Category::ALL.iter().map(|&category| {
            let is_selected = category == self.category;
            button(text(category.label()))
                .width(Length::Fill)
                .padding([style::space::SM, style::space::MD])
                .style(style::list_item(is_selected))
                .on_press(Message::SelectCategory(category))
                .into()
        }))
        .spacing(style::space::XS)
        .width(Length::Fixed(140.0));

        let content = match self.category {
            Category::Appearance => self.appearance_view(),
        };

        let header = row![
            text("Settings").size(18).width(Length::Fill),
            button(text("Close"))
                .padding([style::space::XS, style::space::SM])
                .style(style::ghost_button)
                .on_press(Message::Close),
        ];

        Some(
            container(
                column![header, row![sidebar, content].spacing(style::space::MD)]
                    .spacing(style::space::MD),
            )
            .width(Length::Fixed(560.0))
            .padding(style::space::MD)
            .style(style::panel)
            .into(),
        )
    }

    fn appearance_view(&self) -> Element<'_, Message> {
        column![
            text("Theme").size(14),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeSelected).width(Length::Fill),
            text("UI font (restart to apply)").size(14),
            text_input("System default", &self.ui_font_input)
                .style(style::field)
                .on_input(Message::UiFontChanged)
                .on_submit(Message::UiFontSubmitted),
            text("Editor font").size(14),
            text_input("Monospace", &self.editor_font_input)
                .style(style::field)
                .on_input(Message::EditorFontChanged)
                .on_submit(Message::EditorFontSubmitted),
        ]
        .spacing(style::space::SM)
        .width(Length::Fill)
        .into()
    }
}

/// Trims `value` and turns it into `Some` unless it's empty, for the font
/// inputs: an empty field means "use the default", not a font named "".
fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
