#![allow(dead_code)]
use iced::widget::{button, container};
use iced::{Color, Theme};

pub const BG_APP: Color = Color::from_rgb8(9, 9, 11);
pub const BG_SIDEBAR: Color = Color::from_rgb8(17, 17, 19);
pub const BG_SURFACE: Color = Color::from_rgb8(24, 24, 27);
pub const BG_ELEVATED: Color = Color::from_rgb8(39, 39, 42);
pub const BORDER_SUBTLE: Color = Color::from_rgb8(46, 46, 50);
pub const BORDER_STRONG: Color = Color::from_rgb8(63, 63, 70);
pub const TEXT_PRIMARY: Color = Color::from_rgb8(250, 250, 250);
pub const TEXT_SECONDARY: Color = Color::from_rgb8(161, 161, 170);
pub const TEXT_MUTED: Color = Color::from_rgb8(113, 113, 122);
pub const ACCENT: Color = Color::from_rgb8(99, 102, 241);
pub const ACCENT_HOVER: Color = Color::from_rgb8(79, 70, 229);

#[allow(dead_code)]
pub fn shadow() -> iced::Shadow {
    iced::Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
        offset: iced::Vector::new(0.0, 6.0),
        blur_radius: 14.0,
    }
}

#[allow(dead_code)]
pub fn surface_container(
    background: Color,
    border: Color,
    radius: f32,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        snap: false,
        background: Some(background.into()),
        border: iced::Border {
            radius: radius.into(),
            width: 1.0,
            color: border,
        },
        shadow: shadow(),
        text_color: Some(TEXT_PRIMARY),
    }
}

#[allow(dead_code)]
pub fn button_style(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (background, text_color, border) = match status {
            button::Status::Hovered => (BG_ELEVATED, TEXT_PRIMARY, BORDER_STRONG),
            button::Status::Pressed => (BG_SURFACE, TEXT_PRIMARY, BORDER_STRONG),
            button::Status::Disabled => (BG_SURFACE, TEXT_MUTED, BORDER_SUBTLE),
            button::Status::Active => {
                if selected {
                    (ACCENT, TEXT_PRIMARY, ACCENT_HOVER)
                } else {
                    (BG_SURFACE, TEXT_PRIMARY, BORDER_SUBTLE)
                }
            }
        };
        button::Style {
            snap: false,
            background: Some(background.into()),
            text_color,
            border: iced::Border {
                radius: 10.0.into(),
                width: 1.0,
                color: border,
            },
            shadow: shadow(),
        }
    }
}

#[allow(dead_code)]
pub fn themed_input()
-> impl Fn(&Theme, iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    move |_theme, _status| iced::widget::text_input::Style {
        background: iced::Background::Color(BG_SURFACE),
        border: iced::Border {
            radius: 10.0.into(),
            width: 1.0,
            color: BORDER_SUBTLE,
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: BG_ELEVATED,
    }
}
