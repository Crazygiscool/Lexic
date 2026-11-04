#![allow(dead_code)]
use iced::widget::{button, container};
use iced::{Color, Theme};

pub const BG_APP: Color = Color::from_rgb8(11, 19, 38);
pub const BG_SIDEBAR: Color = Color::from_rgb8(19, 27, 46);
pub const BG_SURFACE: Color = Color::from_rgb8(23, 31, 51);
pub const BG_ELEVATED: Color = Color::from_rgb8(34, 42, 61);
pub const BORDER_SUBTLE: Color = Color::from_rgb8(66, 71, 84);
pub const BORDER_STRONG: Color = Color::from_rgb8(71, 85, 105);
pub const TEXT_PRIMARY: Color = Color::from_rgb8(218, 226, 253);
pub const TEXT_SECONDARY: Color = Color::from_rgb8(194, 198, 214);
pub const TEXT_MUTED: Color = Color::from_rgb8(140, 144, 159);
pub const BG_SURFACE_HIGHEST: Color = Color::from_rgb8(45, 52, 73);
pub const SECONDARY: Color = Color::from_rgb8(164, 201, 255);
pub const SECONDARY_CONTAINER: Color = Color::from_rgb8(2, 103, 184);
pub const TERTIARY: Color = Color::from_rgb8(78, 222, 163);
pub const STRUCTURAL: Color = Color::from_rgb8(168, 85, 247);
pub const SUCCESS: Color = Color::from_rgb8(16, 185, 129);
pub const WARNING: Color = Color::from_rgb8(245, 158, 11);
pub const ERROR: Color = Color::from_rgb8(255, 180, 171);
pub const ACCENT: Color = Color::from_rgb8(77, 142, 255);
pub const ACCENT_HOVER: Color = Color::from_rgb8(59, 130, 246);

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
