use crate::app::*;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Theme};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let labels = [
            ("Transpiler IDE", Screen::Project),
            ("Review Deck", Screen::Review),
            ("Grammar Dict", Screen::Search),
            ("Engine & Interop", Screen::Settings),
        ];

        let sidebar_width = if self.sidebar_collapsed {
            Length::Fixed(82.0)
        } else {
            Length::Fixed(236.0)
        };

        let mut nav_items = column![].spacing(6);
        let brand = if self.sidebar_collapsed {
            text("L").size(24).color(Color::from_rgb8(218, 226, 253))
        } else {
            text("Lexic")
                .size(22)
                .color(Color::from_rgb8(218, 226, 253))
        };
        let sidebar_toggle = button(text(">").size(16).color(Color::from_rgb8(218, 226, 253)))
            .padding([6, 10])
            .on_press(Message::ToggleSidebar)
            .style(monochrome_button(false));
        nav_items = nav_items.push(
            row![
                brand,
                iced::widget::Space::new().width(Length::Fill),
                sidebar_toggle
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );

        for (label, screen) in labels {
            let is_active = screen == self.screen;
            let collapsed = self.sidebar_collapsed;
            let nav_label = if collapsed {
                label.chars().next().unwrap().to_string()
            } else {
                label.to_string()
            };
            let mut nav_button = button(
                text(nav_label)
                    .size(15)
                    .color(Color::from_rgb8(218, 226, 253)),
            )
            .width(Length::Fill)
            .padding([10, 12])
            .on_press(Message::Navigate(screen));
            nav_button = nav_button.style(move |_theme: &Theme, status| {
                let hovered = matches!(status, iced::widget::button::Status::Hovered);
                let pressed = matches!(status, iced::widget::button::Status::Pressed);
                let (background, text_color, border) = if is_active {
                    (
                        Color::from_rgb8(77, 142, 255),
                        Color::from_rgb8(11, 19, 38),
                        Color::from_rgb8(96, 165, 250),
                    )
                } else if hovered || pressed {
                    (
                        Color::from_rgb8(51, 65, 85),
                        Color::from_rgb8(218, 226, 253),
                        Color::from_rgb8(66, 71, 84),
                    )
                } else {
                    (
                        Color::from_rgb8(23, 31, 51),
                        Color::from_rgb8(194, 198, 214),
                        Color::from_rgb8(66, 71, 84),
                    )
                };
                iced::widget::button::Style {
                    snap: false,
                    background: Some(background.into()),
                    text_color,
                    border: iced::Border {
                        radius: 10.0.into(),
                        width: 1.0,
                        color: border,
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 14.0,
                    },
                }
            });
            nav_items = nav_items.push(nav_button);
        }

        let sidebar = container(column![nav_items].spacing(10).height(Length::Fill))
            .padding(16)
            .width(sidebar_width)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                snap: false,
                background: Some(Color::from_rgb8(19, 27, 46).into()),
                border: iced::Border {
                    radius: 0.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(34, 42, 61),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
                text_color: Some(Color::from_rgb8(218, 226, 253)),
            });

        let search_input = text_input("Search the dictionary…", &self.search_input)
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::Lookup)
            .padding([10, 14])
            .width(Length::Fill);

        let search_input = search_input.style(|_theme, _status| text_input::Style {
            background: iced::Background::Color(Color::from_rgb8(23, 31, 51)),
            border: iced::Border {
                radius: 10.0.into(),
                width: 1.0,
                color: Color::from_rgb8(66, 71, 84),
            },
            icon: Color::from_rgb8(194, 198, 214),
            placeholder: Color::from_rgb8(140, 144, 159),
            value: Color::from_rgb8(218, 226, 253),
            selection: Color::from_rgb8(34, 42, 61),
        });

        let lookup_button = button(text("Look up").size(14).color(Color::BLACK))
            .padding([10, 16])
            .on_press(Message::Lookup);
        let lookup_button = lookup_button.style(|_theme, status| {
            let (background, text_color) = match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                    (Color::from_rgb8(194, 198, 214), Color::BLACK)
                }
                iced::widget::button::Status::Disabled => (
                    Color::from_rgb8(51, 65, 85),
                    Color::from_rgb8(140, 144, 159),
                ),
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
        });

        let topbar = container(
            row![search_input, lookup_button,]
                .spacing(12)
                .align_y(Alignment::Center),
        )
        .padding([14, 18])
        .height(Length::Fixed(76.0))
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            snap: false,
            background: Some(Color::from_rgb8(19, 27, 46).into()),
            border: iced::Border {
                radius: 0.0.into(),
                width: 1.0,
                color: Color::from_rgb8(34, 42, 61),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 14.0,
            },
            text_color: Some(Color::from_rgb8(218, 226, 253)),
        });

        let body: Element<'_, Message> = match self.screen {
            Screen::Home => self.home_view(),
            Screen::Search => self.search_view(),
            Screen::Decks => self.decks_view(),
            Screen::DeckDetail(deck_id) => self.deck_detail_view(deck_id),
            Screen::Review => self.review_view(),
            Screen::Stats => self.stats_view(),
            Screen::Project => self.project_view(),
            Screen::Settings => self.settings_view(),
        };

        let content =
            scrollable(container(body).padding(28).width(Length::Fill)).height(Length::Fill);

        let main = column![topbar, content].spacing(0).width(Length::Fill);

        let root = row![sidebar, main].height(Length::Fill);

        container(root)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                snap: false,
                background: Some(Color::from_rgb8(11, 19, 38).into()),
                border: iced::Border::default(),
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
                text_color: Some(Color::from_rgb8(218, 226, 253)),
            })
            .into()
    }
}
