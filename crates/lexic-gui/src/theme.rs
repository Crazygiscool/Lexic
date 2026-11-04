#![allow(dead_code)]
use iced::widget::{button, container, text_input};
use iced::{Color, Font, Shadow, Theme};

pub const BG_APP: Color = Color::from_rgb8(11, 19, 38);
pub const BG_SIDEBAR: Color = Color::from_rgb8(19, 27, 46);
pub const BG_SURFACE: Color = Color::from_rgb8(23, 31, 51);
pub const BG_ELEVATED: Color = Color::from_rgb8(34, 42, 61);
pub const BG_SURFACE_DIM: Color = Color::from_rgb8(11, 19, 38);
pub const BG_CONTAINER_LOWEST: Color = Color::from_rgb8(6, 14, 32);
pub const BG_CONTAINER_LOW: Color = Color::from_rgb8(19, 27, 46);
pub const BG_CONTAINER: Color = Color::from_rgb8(23, 31, 51);
pub const BG_CONTAINER_HIGH: Color = Color::from_rgb8(34, 42, 61);
pub const BG_CONTAINER_HIGHEST: Color = Color::from_rgb8(45, 52, 73);
pub const BG_SURFACE_BRIGHT: Color = Color::from_rgb8(49, 57, 77);
pub const BORDER_SUBTLE: Color = Color::from_rgb8(66, 71, 84);
pub const BORDER_STRONG: Color = Color::from_rgb8(71, 85, 105);
pub const OUTLINE: Color = Color::from_rgb8(140, 144, 159);
pub const OUTLINE_VARIANT: Color = Color::from_rgb8(66, 71, 84);
pub const TEXT_PRIMARY: Color = Color::from_rgb8(218, 226, 253);
pub const TEXT_SECONDARY: Color = Color::from_rgb8(194, 198, 214);
pub const TEXT_MUTED: Color = Color::from_rgb8(140, 144, 159);
pub const TEXT_ON_SURFACE_VARIANT: Color = Color::from_rgb8(194, 198, 214);
pub const PRIMARY: Color = Color::from_rgb8(173, 198, 255);
pub const PRIMARY_CONTAINER: Color = Color::from_rgb8(77, 142, 255);
pub const ON_PRIMARY: Color = Color::from_rgb8(0, 46, 106);
pub const ON_PRIMARY_CONTAINER: Color = Color::from_rgb8(0, 40, 93);
pub const SECONDARY: Color = Color::from_rgb8(164, 201, 255);
pub const SECONDARY_CONTAINER: Color = Color::from_rgb8(2, 103, 184);
pub const ON_SECONDARY_CONTAINER: Color = Color::from_rgb8(214, 229, 255);
pub const TERTIARY: Color = Color::from_rgb8(78, 222, 163);
pub const TERTIARY_CONTAINER: Color = Color::from_rgb8(0, 165, 114);
pub const ON_TERTIARY_CONTAINER: Color = Color::from_rgb8(0, 49, 31);
pub const ERROR: Color = Color::from_rgb8(255, 180, 171);
pub const ERROR_CONTAINER: Color = Color::from_rgb8(147, 0, 10);
pub const ON_ERROR_CONTAINER: Color = Color::from_rgb8(255, 218, 214);
pub const SUCCESS: Color = Color::from_rgb8(16, 185, 129);
pub const WARNING: Color = Color::from_rgb8(245, 158, 11);
pub const STRUCTURAL: Color = Color::from_rgb8(168, 85, 247);
pub const ACCENT: Color = Color::from_rgb8(77, 142, 255);
pub const ACCENT_HOVER: Color = Color::from_rgb8(59, 130, 246);

pub fn font_body() -> Font {
    Font::with_name("Inter")
}

pub fn font_code() -> Font {
    Font::with_name("JetBrains Mono")
}

pub fn shadow_sm() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.24),
        offset: iced::Vector::new(0.0, 2.0),
        blur_radius: 6.0,
    }
}

pub fn shadow_md() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
        offset: iced::Vector::new(0.0, 6.0),
        blur_radius: 14.0,
    }
}

pub fn shadow_shell() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.40),
        offset: iced::Vector::new(0.0, 1.0),
        blur_radius: 8.0,
    }
}

pub fn shadow() -> Shadow {
    shadow_md()
}

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
        shadow: shadow_md(),
        text_color: Some(TEXT_PRIMARY),
    }
}

pub fn panel_flat(
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
        shadow: Shadow::default(),
        text_color: Some(TEXT_PRIMARY),
    }
}

pub fn panel_shell() -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        snap: false,
        background: Some(BG_CONTAINER_LOW.into()),
        border: iced::Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: shadow_shell(),
        text_color: Some(TEXT_PRIMARY),
    }
}

pub fn nav_item(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered);
        let (background, text_color) = if active {
            (BG_CONTAINER_HIGH, PRIMARY)
        } else if hovered {
            (BG_CONTAINER, TEXT_PRIMARY)
        } else {
            (Color::TRANSPARENT, TEXT_SECONDARY)
        };
        button::Style {
            snap: false,
            background: Some(background.into()),
            text_color,
            border: iced::Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn primary_button() -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (background, text_color, border) = match status {
            button::Status::Hovered => (PRIMARY_CONTAINER, ON_PRIMARY_CONTAINER, PRIMARY_CONTAINER),
            button::Status::Pressed => (PRIMARY, ON_PRIMARY, PRIMARY),
            button::Status::Disabled => (BG_CONTAINER_HIGH, TEXT_MUTED, BORDER_SUBTLE),
            button::Status::Active => (PRIMARY_CONTAINER, ON_PRIMARY_CONTAINER, PRIMARY_CONTAINER),
        };
        button::Style {
            snap: false,
            background: Some(background.into()),
            text_color,
            border: iced::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: border,
            },
            shadow: shadow_sm(),
        }
    }
}

pub fn secondary_button() -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (background, text_color, border) = match status {
            button::Status::Hovered => (BG_CONTAINER_HIGHEST, TEXT_PRIMARY, BORDER_STRONG),
            button::Status::Pressed => (BG_ELEVATED, TEXT_PRIMARY, BORDER_STRONG),
            button::Status::Disabled => (BG_CONTAINER, TEXT_MUTED, BORDER_SUBTLE),
            button::Status::Active => (BG_CONTAINER, TEXT_PRIMARY, BORDER_SUBTLE),
        };
        button::Style {
            snap: false,
            background: Some(background.into()),
            text_color,
            border: iced::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: border,
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn ghost_button() -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (background, text_color, border) = match status {
            button::Status::Hovered => (BG_CONTAINER, TEXT_PRIMARY, BORDER_SUBTLE),
            button::Status::Pressed => (BG_ELEVATED, TEXT_PRIMARY, BORDER_STRONG),
            button::Status::Disabled => (Color::TRANSPARENT, TEXT_MUTED, Color::TRANSPARENT),
            button::Status::Active => (Color::TRANSPARENT, TEXT_SECONDARY, Color::TRANSPARENT),
        };
        button::Style {
            snap: false,
            background: Some(background.into()),
            text_color,
            border: iced::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: border,
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn icon_button() -> impl Fn(&Theme, button::Status) -> button::Style {
    ghost_button()
}

pub fn button_style(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (background, text_color, border) = match status {
            button::Status::Hovered => (BG_ELEVATED, TEXT_PRIMARY, BORDER_STRONG),
            button::Status::Pressed => (BG_SURFACE, TEXT_PRIMARY, BORDER_STRONG),
            button::Status::Disabled => (BG_SURFACE, TEXT_MUTED, BORDER_SUBTLE),
            button::Status::Active => {
                if selected {
                    (PRIMARY_CONTAINER, ON_PRIMARY_CONTAINER, PRIMARY_CONTAINER)
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
                radius: 8.0.into(),
                width: 1.0,
                color: border,
            },
            shadow: shadow_sm(),
        }
    }
}

pub fn themed_input() -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_theme, _status| text_input::Style {
        background: iced::Background::Color(BG_CONTAINER),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: BORDER_SUBTLE,
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: BG_ELEVATED,
    }
}

pub fn text_editor_style()
-> impl Fn(&Theme, iced::widget::text_editor::Status) -> iced::widget::text_editor::Style {
    move |_theme, _status| iced::widget::text_editor::Style {
        background: iced::Background::Color(BG_CONTAINER_LOWEST),
        border: iced::Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: PRIMARY_CONTAINER,
    }
}
