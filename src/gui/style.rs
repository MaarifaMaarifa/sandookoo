use iced::widget::{button, container, text_editor, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// The app-wide theme. Kept in one place so every panel renders
/// consistently.
pub fn theme(_gui: &super::Gui) -> Theme {
    Theme::CatppuccinMocha
}

/// A consistent spacing scale, used instead of scattering magic numbers
/// through each panel's layout code.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
}

/// A consistent corner-radius scale.
pub mod radius {
    pub const SM: f32 = 6.0;
    pub const MD: f32 = 10.0;
    pub const LG: f32 = 14.0;
}

/// The raised "card" look shared by the sidebar and the editor/results
/// boxes: a soft background, a faint border, and a drop shadow for a bit
/// of depth.
pub fn panel(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color.scale_alpha(0.35),
            width: 1.0,
            radius: radius::LG.into(),
        },
        shadow: Shadow {
            color: Color::BLACK.scale_alpha(0.28),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 16.0,
        },
        ..container::Style::default()
    }
}

/// A rounded, accent-colored button for a panel's main action (Run,
/// Connect, + New).
pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        background: Some(Background::Color(palette.primary.base.color)),
        text_color: palette.primary.base.text,
        border: Border::default().rounded(radius::SM),
        ..button::Style::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.primary.weak.color)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: base.background.map(|b| b.scale_alpha(0.5)),
            text_color: base.text_color.scale_alpha(0.5),
            ..base
        },
    }
}

/// An outlined, transparent button for a panel's secondary action (Cancel).
pub fn ghost_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        background: None,
        text_color: palette.background.base.text,
        border: Border::default()
            .rounded(radius::SM)
            .width(1.0)
            .color(palette.background.strong.color),
        ..button::Style::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.background.weak.color)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.background.strong.color)),
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: base.text_color.scale_alpha(0.5),
            ..base
        },
    }
}

/// A connection entry in the sidebar list: a filled pill when selected,
/// otherwise a plain row that only highlights on hover.
pub fn connection_item(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();

        if selected {
            let base = button::Style {
                background: Some(Background::Color(palette.primary.base.color)),
                text_color: palette.primary.base.text,
                border: Border::default().rounded(radius::SM),
                ..button::Style::default()
            };

            return match status {
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(palette.primary.strong.color)),
                    ..base
                },
                _ => base,
            };
        }

        let base = button::Style {
            background: None,
            text_color: palette.background.base.text,
            border: Border::default().rounded(radius::SM),
            ..button::Style::default()
        };

        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(palette.background.weak.color)),
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(palette.background.strong.color)),
                ..base
            },
            _ => base,
        }
    }
}

/// A rounded text field, used by the new-connection form.
pub fn field(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();

    let active = text_input::Style {
        background: Background::Color(palette.background.base.color),
        border: Border::default()
            .rounded(radius::SM)
            .width(1.0)
            .color(palette.background.strong.color),
        icon: palette.background.weak.text,
        placeholder: palette.secondary.base.color,
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    };

    match status {
        text_input::Status::Active => active,
        text_input::Status::Hovered => text_input::Style {
            border: active.border.color(palette.background.base.text),
            ..active
        },
        text_input::Status::Focused { .. } => text_input::Style {
            border: active.border.color(palette.primary.strong.color).width(1.5),
            ..active
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(palette.background.weak.color),
            value: active.placeholder,
            ..active
        },
    }
}

/// A rounded, slightly sunken surface for the SQL editor, set apart from
/// the panel background it sits in.
pub fn editor(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let palette = theme.extended_palette();

    let active = text_editor::Style {
        background: Background::Color(palette.background.base.color),
        border: Border::default()
            .rounded(radius::MD)
            .width(1.0)
            .color(palette.background.strong.color.scale_alpha(0.6)),
        placeholder: palette.secondary.base.color,
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    };

    match status {
        text_editor::Status::Active => active,
        text_editor::Status::Hovered => text_editor::Style {
            border: active.border.color(palette.background.base.text),
            ..active
        },
        text_editor::Status::Focused { .. } => text_editor::Style {
            border: active.border.color(palette.primary.strong.color).width(1.5),
            ..active
        },
        text_editor::Status::Disabled => active,
    }
}
