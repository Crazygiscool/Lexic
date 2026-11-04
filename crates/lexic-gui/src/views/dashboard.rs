use crate::app::*;
use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length, Theme};

impl App {
    pub fn home_view(&self) -> Element<'_, Message> {
        let due_cards: i64 = self.deck_counts.values().map(|(_, _, due)| *due).sum();
        let new_cards: i64 = self.deck_counts.values().map(|(_, new, _)| *new).sum();

        let stat_card = |title: &'static str, value: String| -> Element<'_, Message> {
            container(
                column![
                    text(title).size(13).color(Color::from_rgb8(194, 198, 214)),
                    text(value).size(28).color(Color::from_rgb8(218, 226, 253))
                ]
                .spacing(4),
            )
            .padding(18)
            .width(Length::Fixed(180.0))
            .style(|_theme: &Theme| container::Style {
                snap: false,
                background: Some(Color::from_rgb8(23, 31, 51).into()),
                border: iced::Border {
                    radius: 16.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(42, 42, 42),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
                text_color: Some(Color::from_rgb8(218, 226, 253)),
            })
            .into()
        };

        let mut content = column![
            text("Welcome to Lexic")
                .size(36)
                .color(Color::from_rgb8(218, 226, 253)),
            text("Search from the bar above or jump back into your study flow.")
                .size(15)
                .color(Color::from_rgb8(194, 198, 214)),
            row![
                stat_card("Decks", self.decks.len().to_string()),
                stat_card("Due cards", due_cards.to_string()),
                stat_card("New cards", new_cards.to_string()),
            ]
            .spacing(12),
            row![
                {
                    let button = button(text("Start review").size(15).color(Color::BLACK))
                        .on_press(Message::StartReview)
                        .padding([10, 18]);
                    button.style(|_theme, status| {
                        let (background, text_color) = match status {
                            iced::widget::button::Status::Hovered
                            | iced::widget::button::Status::Pressed => {
                                (Color::from_rgb8(194, 198, 214), Color::BLACK)
                            }
                            _ => (Color::from_rgb8(236, 236, 236), Color::BLACK),
                        };
                        iced::widget::button::Style {
                            snap: false,
                            background: Some(background.into()),
                            text_color,
                            border: iced::Border {
                                radius: 10.0.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: iced::Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                                offset: iced::Vector::new(0.0, 6.0),
                                blur_radius: 14.0,
                            },
                        }
                    })
                },
                {
                    let button = button(text("Browse decks").size(15).color(Color::BLACK))
                        .on_press(Message::Navigate(Screen::Decks))
                        .padding([10, 18]);
                    button.style(|_theme, status| {
                        let (background, text_color) = match status {
                            iced::widget::button::Status::Hovered
                            | iced::widget::button::Status::Pressed => {
                                (Color::from_rgb8(194, 198, 214), Color::BLACK)
                            }
                            _ => (Color::from_rgb8(236, 236, 236), Color::BLACK),
                        };
                        iced::widget::button::Style {
                            snap: false,
                            background: Some(background.into()),
                            text_color,
                            border: iced::Border {
                                radius: 10.0.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: iced::Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                                offset: iced::Vector::new(0.0, 6.0),
                                blur_radius: 14.0,
                            },
                        }
                    })
                },
            ]
            .spacing(12),
        ]
        .spacing(18);

        if let Some(entry) = &self.active_entry {
            content = content.push(
                container(
                    column![
                        text("Last looked up")
                            .size(13)
                            .color(Color::from_rgb8(194, 198, 214)),
                        text(&entry.word)
                            .size(24)
                            .color(Color::from_rgb8(218, 226, 253)),
                        text(&entry.source)
                            .size(12)
                            .color(Color::from_rgb8(140, 144, 159)),
                    ]
                    .spacing(4),
                )
                .padding(18)
                .width(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    snap: false,
                    background: Some(Color::from_rgb8(23, 31, 51).into()),
                    border: iced::Border {
                        radius: 16.0.into(),
                        width: 1.0,
                        color: Color::from_rgb8(42, 42, 42),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 14.0,
                    },
                    text_color: Some(Color::from_rgb8(218, 226, 253)),
                }),
            );
        }

        container(content).padding(8).into()
    }
}
