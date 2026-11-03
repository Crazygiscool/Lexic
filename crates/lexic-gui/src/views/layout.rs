use crate::app::*;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Theme};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let labels = [
            ("Home", Screen::Home),
            ("Dictionary", Screen::Search),
            ("Decks", Screen::Decks),
            ("Review", Screen::Review),
            ("Stats", Screen::Stats),
            ("Settings", Screen::Settings),
        ];

        let sidebar_width = if self.sidebar_collapsed {
            Length::Fixed(82.0)
        } else {
            Length::Fixed(236.0)
        };

        let mut nav_items = column![].spacing(6);
        let brand = if self.sidebar_collapsed {
            text("L").size(24).color(Color::from_rgb8(245, 245, 245))
        } else {
            text("Lexic")
                .size(22)
                .color(Color::from_rgb8(245, 245, 245))
        };
        let sidebar_toggle = button(text(">").size(16).color(Color::from_rgb8(245, 245, 245)))
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
                    .color(Color::from_rgb8(245, 245, 245)),
            )
            .width(Length::Fill)
            .padding([10, 12])
            .on_press(Message::Navigate(screen));
            nav_button = nav_button.style(move |_theme: &Theme, status| {
                let hovered = matches!(status, iced::widget::button::Status::Hovered);
                let pressed = matches!(status, iced::widget::button::Status::Pressed);
                let (background, text_color, border) = if is_active {
                    (
                        Color::from_rgb8(54, 54, 54),
                        Color::from_rgb8(245, 245, 245),
                        Color::from_rgb8(86, 86, 86),
                    )
                } else if hovered || pressed {
                    (
                        Color::from_rgb8(34, 34, 34),
                        Color::from_rgb8(245, 245, 245),
                        Color::from_rgb8(74, 74, 74),
                    )
                } else {
                    (
                        Color::from_rgb8(18, 18, 18),
                        Color::from_rgb8(225, 225, 225),
                        Color::from_rgb8(44, 44, 44),
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
                background: Some(Color::from_rgb8(10, 10, 10).into()),
                border: iced::Border {
                    radius: 0.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(30, 30, 30),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
                text_color: Some(Color::from_rgb8(245, 245, 245)),
            });

        let search_input = text_input("Search the dictionary…", &self.search_input)
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::Lookup)
            .padding([10, 14])
            .width(Length::Fill);

        let search_input = search_input.style(|_theme, _status| text_input::Style {
            background: iced::Background::Color(Color::from_rgb8(15, 15, 15)),
            border: iced::Border {
                radius: 10.0.into(),
                width: 1.0,
                color: Color::from_rgb8(44, 44, 44),
            },
            icon: Color::from_rgb8(163, 163, 163),
            placeholder: Color::from_rgb8(115, 115, 115),
            value: Color::from_rgb8(245, 245, 245),
            selection: Color::from_rgb8(50, 50, 50),
        });

        let lookup_button = button(text("Look up").size(14).color(Color::BLACK))
            .padding([10, 16])
            .on_press(Message::Lookup);
        let lookup_button = lookup_button.style(|_theme, status| {
            let (background, text_color) = match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                    (Color::from_rgb8(225, 225, 225), Color::BLACK)
                }
                iced::widget::button::Status::Disabled => (
                    Color::from_rgb8(60, 60, 60),
                    Color::from_rgb8(160, 160, 160),
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
            background: Some(Color::from_rgb8(10, 10, 10).into()),
            border: iced::Border {
                radius: 0.0.into(),
                width: 1.0,
                color: Color::from_rgb8(30, 30, 30),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 14.0,
            },
            text_color: Some(Color::from_rgb8(245, 245, 245)),
        });

        let body: Element<'_, Message> = match self.screen {
            Screen::Home => self.home_view(),
            Screen::Search => self.search_view(),
            Screen::Decks => self.decks_view(),
            Screen::DeckDetail(deck_id) => self.deck_detail_view(deck_id),
            Screen::Review => self.review_view(),
            Screen::Stats => self.stats_view(),
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
                background: Some(Color::BLACK.into()),
                border: iced::Border::default(),
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 14.0,
                },
                text_color: Some(Color::from_rgb8(245, 245, 245)),
            })
            .into()
    }
}
