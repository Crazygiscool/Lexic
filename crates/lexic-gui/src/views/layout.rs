use crate::app::*;

use iced::widget::Space;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Theme};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let collapsed = self.sidebar_collapsed;
        let sidebar_width = if collapsed {
            Length::Fixed(82.0)
        } else {
            Length::Fixed(288.0)
        };

        let due_count: i64 = self.deck_counts.values().map(|(_, _, due)| *due).sum();
        let missing_count = self
            .gset_result
            .as_ref()
            .map(|result| result.diagnostics.len())
            .unwrap_or(0);
        let transpiler_status = match (self.gset_status.as_deref(), self.gset_result.as_ref()) {
            (Some(status), _) if status.contains("Running") => "Transpiler: Running",
            (_, Some(result)) if result.success => "Transpiler: Idle (AST Clean)",
            (_, Some(_)) => "Transpiler: Failed",
            _ => "Transpiler: Idle",
        };
        let transpiler_dot = match (self.gset_status.as_deref(), self.gset_result.as_ref()) {
            (Some(status), _) if status.contains("Running") => Color::from_rgb8(78, 222, 163),
            (_, Some(result)) if result.success => Color::from_rgb8(78, 222, 163),
            (_, Some(_)) => Color::from_rgb8(255, 180, 171),
            _ => Color::from_rgb8(194, 198, 214),
        };

        let file_context = if !self.gset_project_path.trim().is_empty() {
            let path = std::path::Path::new(self.gset_project_path.trim());
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("project.gset")
                .to_string()
        } else if let Some(entry) = &self.active_entry {
            format!("{}.gset", entry.word)
        } else {
            String::from("typescached.gset")
        };

        struct NavItem {
            label: &'static str,
            shortcut: &'static str,
            icon: &'static str,
            screen: Screen,
            badge: Option<String>,
        }
        let nav_items_spec = [
            NavItem {
                label: "Transpiler IDE",
                shortcut: "⌥1",
                icon: " code_blocks ",
                screen: Screen::Project,
                badge: (if missing_count > 0 {
                    Some(missing_count.to_string())
                } else {
                    None
                }),
            },
            NavItem {
                label: "Review Deck",
                shortcut: "⌥2",
                icon: " psychology ",
                screen: Screen::Review,
                badge: (if due_count > 0 {
                    Some(due_count.to_string())
                } else {
                    None
                }),
            },
            NavItem {
                label: "Grammar Dict",
                shortcut: "⌥3",
                icon: " menu_book ",
                screen: Screen::Search,
                badge: None,
            },
            NavItem {
                label: "Engine & Interop",
                shortcut: "⌥4",
                icon: " terminal ",
                screen: Screen::Settings,
                badge: None,
            },
        ];

        let mut nav_column = column![].spacing(4);
        let brand = if collapsed {
            text("λ").size(24).color(Color::from_rgb8(173, 198, 255))
        } else {
            text("λ").size(22).color(Color::from_rgb8(173, 198, 255))
        };
        // We cannot easily build rich row without closures referencing nav_items_spec elements?
        // Build simple nav here.
        let version_badge = container(
            row![
                container(text("").size(8))
                    .width(Length::Fixed(6.0))
                    .height(Length::Fixed(6.0)),
                text(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .size(10)
                    .color(Color::from_rgb8(78, 222, 163)),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([2, 6])
        .style(|_theme: &Theme| iced::widget::container::Style {
            background: Some(Color::from_rgb8(34, 42, 61).into()),
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            text_color: Some(Color::from_rgb8(218, 226, 253)),
            snap: false,
        });

        let brand_block = if collapsed {
            column![brand].spacing(4)
        } else {
            column![
                row![
                    container(text("λ").size(20).color(Color::from_rgb8(0, 46, 106)))
                        .width(Length::Fixed(28.0))
                        .height(Length::Fixed(28.0))
                        .style(|_theme: &Theme| iced::widget::container::Style {
                            background: Some(Color::from_rgb8(77, 142, 255).into()),
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                            text_color: Some(Color::from_rgb8(0, 46, 106)),
                            snap: false,
                        }),
                    column![
                        text("LEXIC")
                            .size(18)
                            .color(Color::from_rgb8(218, 226, 253)),
                        text("GSET Studio")
                            .size(10)
                            .color(Color::from_rgb8(173, 198, 255)),
                    ]
                    .spacing(0),
                    Space::new().width(Length::Fill),
                    version_badge,
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                container(
                    row![
                        text(" data_object ")
                            .size(12)
                            .color(Color::from_rgb8(78, 222, 163)),
                        text("Grammar Spec")
                            .size(12)
                            .color(Color::from_rgb8(194, 198, 214)),
                        Space::new().width(Length::Fill),
                        container(
                            text("PARSER_SYNCED")
                                .size(9)
                                .color(Color::from_rgb8(164, 201, 255))
                        )
                        .padding([2, 6])
                        .style(|_theme: &Theme| {
                            iced::widget::container::Style {
                                background: Some(Color::from_rgb8(45, 52, 73).into()),
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                                text_color: Some(Color::from_rgb8(164, 201, 255)),
                                snap: false,
                            }
                        }),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .padding([6, 8])
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(Color::from_rgb8(23, 31, 51).into()),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    text_color: Some(Color::from_rgb8(218, 226, 253)),
                    snap: false,
                }),
            ]
            .spacing(8)
        };
        let sidebar_toggle = button(text(">").size(14).color(Color::from_rgb8(194, 198, 214)))
            .padding([4, 8])
            .on_press(Message::ToggleSidebar)
            .style(|_theme, status| iced::widget::button::Style {
                snap: false,
                background: Some(
                    if matches!(status, iced::widget::button::Status::Hovered) {
                        Color::from_rgb8(34, 42, 61)
                    } else {
                        Color::from_rgb8(23, 31, 51)
                    }
                    .into(),
                ),
                text_color: Color::from_rgb8(194, 198, 214),
                border: iced::Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(66, 71, 84),
                },
                shadow: iced::Shadow::default(),
            });
        nav_column = nav_column.push(
            row![brand_block, sidebar_toggle]
                .spacing(8)
                .align_y(Alignment::Center),
        );

        for spec in nav_items_spec {
            let is_active = spec.screen == self.screen;
            let nav_label = if collapsed {
                spec.label.chars().next().unwrap().to_string()
            } else {
                spec.label.to_string()
            };
            let shortcut = if collapsed {
                String::new()
            } else {
                spec.shortcut.to_string()
            };
            let badge = spec.badge.clone();
            let icon = spec.icon.to_string();
            let mut nav_button = button(
                row![
                    container(text(icon.trim().to_string()).size(14).color(if is_active {
                        Color::from_rgb8(0, 46, 106)
                    } else {
                        Color::from_rgb8(140, 144, 159)
                    }))
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .style(move |_theme: &Theme| {
                        iced::widget::container::Style {
                            background: Some(
                                if is_active {
                                    Color::from_rgb8(77, 142, 255)
                                } else {
                                    Color::from_rgb8(34, 42, 61)
                                }
                                .into(),
                            ),
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                            text_color: Some(if is_active {
                                Color::from_rgb8(0, 46, 106)
                            } else {
                                Color::from_rgb8(140, 144, 159)
                            }),
                            snap: false,
                        }
                    }),
                    text(nav_label).size(14).color(if is_active {
                        Color::from_rgb8(218, 226, 253)
                    } else {
                        Color::from_rgb8(194, 198, 214)
                    }),
                    Space::new().width(Length::Fill),
                    {
                        let badge_element: Element<'_, Message> = if let Some(badge) = badge {
                            container(text(badge).size(10).color(Color::from_rgb8(2, 103, 184)))
                                .padding([2, 6])
                                .style(|_theme: &Theme| iced::widget::container::Style {
                                    background: Some(Color::from_rgb8(194, 198, 214).into()),
                                    border: iced::Border::default(),
                                    shadow: iced::Shadow::default(),
                                    text_color: Some(Color::from_rgb8(2, 103, 184)),
                                    snap: false,
                                })
                                .into()
                        } else if shortcut.is_empty() {
                            text("").size(10).into()
                        } else {
                            text(shortcut)
                                .size(10)
                                .color(Color::from_rgb8(140, 144, 159))
                                .into()
                        };
                        badge_element
                    },
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8, 12])
            .on_press(Message::Navigate(spec.screen));
            nav_button = nav_button.style(move |_theme: &Theme, status| {
                let hovered = matches!(status, iced::widget::button::Status::Hovered);
                let pressed = matches!(status, iced::widget::button::Status::Pressed);
                let (background, border) = if is_active {
                    (Color::from_rgb8(34, 42, 61), Color::from_rgb8(96, 165, 250))
                } else if hovered || pressed {
                    (Color::from_rgb8(34, 42, 61), Color::from_rgb8(71, 85, 105))
                } else {
                    (Color::TRANSPARENT, Color::TRANSPARENT)
                };
                iced::widget::button::Style {
                    snap: false,
                    background: Some(background.into()),
                    text_color: Color::from_rgb8(218, 226, 253),
                    border: iced::Border {
                        radius: 8.0.into(),
                        width: if is_active || hovered { 1.0 } else { 0.0 },
                        color: border,
                    },
                    shadow: iced::Shadow::default(),
                }
            });
            nav_column = nav_column.push(nav_button);
        }

        // Clean footer with real data only.
        let engine_ok_count = 1 + usize::from(
            self.gset_result
                .as_ref()
                .map(|result| result.success)
                .unwrap_or(false),
        );
        let footer = if collapsed {
            column![
                text(format!("{}", due_count))
                    .size(10)
                    .color(Color::from_rgb8(78, 222, 163)),
            ]
            .spacing(4)
        } else {
            column![
                row![
                    text("Engine Daemons")
                        .size(10)
                        .color(Color::from_rgb8(194, 198, 214)),
                    Space::new().width(Length::Fill),
                    text(format!("{} OK", engine_ok_count))
                        .size(10)
                        .color(Color::from_rgb8(78, 222, 163)),
                ]
                .align_y(Alignment::Center),
                container(
                    column![
                        row![
                            text("• rusqlite")
                                .size(11)
                                .color(Color::from_rgb8(218, 226, 253)),
                            Space::new().width(Length::Fill),
                            text("WAL mode")
                                .size(10)
                                .color(Color::from_rgb8(140, 144, 159)),
                        ]
                        .align_y(Alignment::Center),
                        text("~/.local/share/lexic/lexic.db")
                            .size(10)
                            .color(Color::from_rgb8(140, 144, 159)),
                    ]
                    .spacing(2),
                )
                .padding(6)
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(Color::from_rgb8(23, 31, 51).into()),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    text_color: Some(Color::from_rgb8(218, 226, 253)),
                    snap: false,
                }),
                row![
                    text("Queue Load")
                        .size(11)
                        .color(Color::from_rgb8(218, 226, 253)),
                    Space::new().width(Length::Fill),
                    text(format!("{} pending", due_count))
                        .size(10)
                        .color(Color::from_rgb8(173, 198, 255)),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(8)
        };

        let sidebar = container(
            column![nav_column, Space::new().height(Length::Fill), footer]
                .spacing(12)
                .height(Length::Fill),
        )
        .padding(16)
        .width(sidebar_width)
        .height(Length::Fill)
        .style(|_theme: &Theme| iced::widget::container::Style {
            background: Some(Color::from_rgb8(19, 27, 46).into()),
            border: iced::Border {
                radius: 0.0.into(),
                width: 1.0,
                color: Color::from_rgb8(66, 71, 84),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 8.0,
            },
            text_color: Some(Color::from_rgb8(218, 226, 253)),
            snap: false,
        });

        let search_input: iced::widget::TextInput<'_, Message, Theme, iced::Renderer> =
            text_input("Search the dictionary…", &self.search_input)
                .on_input(Message::SearchInputChanged)
                .on_submit(Message::Lookup)
                .padding([10, 14])
                .width(Length::Fill);

        let search_input: iced::widget::TextInput<'_, Message, Theme, iced::Renderer> =
            search_input.style(|_theme: &Theme, _status| iced::widget::text_input::Style {
                background: iced::Background::Color(Color::from_rgb8(23, 31, 51)),
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(66, 71, 84),
                },
                icon: Color::from_rgb8(194, 198, 214),
                placeholder: Color::from_rgb8(140, 144, 159),
                value: Color::from_rgb8(218, 226, 253),
                selection: Color::from_rgb8(34, 42, 61),
            });

        let lookup_button: iced::widget::Button<'_, Message, Theme, iced::Renderer> =
            button(text("Look up").size(14).color(Color::from_rgb8(15, 23, 42)))
                .padding([8, 12])
                .on_press(Message::Lookup);
        let lookup_button: iced::widget::Button<'_, Message, Theme, iced::Renderer> = lookup_button
            .style(|_theme: &Theme, status| {
                let (background, text_color) = match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => {
                        (Color::from_rgb8(96, 165, 250), Color::from_rgb8(15, 23, 42))
                    }
                    iced::widget::button::Status::Disabled => (
                        Color::from_rgb8(51, 65, 85),
                        Color::from_rgb8(140, 144, 159),
                    ),
                    _ => (Color::from_rgb8(77, 142, 255), Color::from_rgb8(15, 23, 42)),
                };
                iced::widget::button::Style {
                    snap: false,
                    background: Some(background.into()),
                    text_color,
                    border: iced::Border {
                        radius: 8.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: iced::Shadow::default(),
                }
            });

        let file_chip_label = if file_context.trim().is_empty() {
            String::from("typescript-extended.gset")
        } else {
            file_context
        };
        let target_badge = if self.gset_target_language.trim().is_empty() {
            String::from("go")
        } else {
            self.gset_target_language.trim().to_string()
        };

        let top_bar = container(
            row![
                container(
                    row![
                        text(" description ")
                            .size(14)
                            .color(Color::from_rgb8(77, 142, 255)),
                        text(file_chip_label)
                            .size(13)
                            .color(Color::from_rgb8(218, 226, 253)),
                        text(" (v1.0.0)")
                            .size(10)
                            .color(Color::from_rgb8(194, 198, 214)),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([4, 8])
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(Color::from_rgb8(23, 31, 51).into()),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    text_color: Some(Color::from_rgb8(218, 226, 253)),
                    snap: false,
                }),
                container(
                    row![
                        text(" fork_right ")
                            .size(12)
                            .color(Color::from_rgb8(164, 201, 255)),
                        text(target_badge)
                            .size(11)
                            .color(Color::from_rgb8(194, 198, 214)),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .padding([3, 6])
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(Color::from_rgb8(51, 65, 85).into()),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    text_color: Some(Color::from_rgb8(218, 226, 253)),
                    snap: false,
                }),
                container(
                    row![
                        container(text(" ").size(8))
                            .width(Length::Fixed(8.0))
                            .height(Length::Fixed(8.0)),
                        text("Transpiler:")
                            .size(11)
                            .color(Color::from_rgb8(194, 198, 214)),
                        text(transpiler_status.trim_start_matches("Transpiler: "))
                            .size(11)
                            .color(transpiler_dot),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([2, 8]),
                search_input,
                lookup_button,
                iced::widget::Space::new().width(Length::Fill),
                button(
                    text("Find Token")
                        .size(11)
                        .color(Color::from_rgb8(194, 198, 214))
                )
                .padding([6, 10])
                .style(|_theme, status| crate::theme::button_style(false)(_theme, status)),
                button(
                    text("Quick Card")
                        .size(11)
                        .color(Color::from_rgb8(0, 46, 106))
                )
                .padding([6, 10])
                .style(|_theme, _status| iced::widget::button::Style {
                    snap: false,
                    background: Some(Color::from_rgb8(77, 142, 255).into()),
                    text_color: Color::from_rgb8(0, 46, 106),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                }),
                button(
                    text("Export")
                        .size(11)
                        .color(Color::from_rgb8(194, 198, 214))
                )
                .padding([6, 10])
                .on_press(Message::BeginExport)
                .style(|_theme, status| crate::theme::button_style(false)(_theme, status)),
                container(
                    row![
                        button(
                            text("bug_report")
                                .size(14)
                                .color(Color::from_rgb8(140, 144, 159))
                        )
                        .padding([4, 6])
                        .style(|_theme, status| crate::theme::button_style(false)(_theme, status)),
                        button(
                            text("terminal")
                                .size(14)
                                .color(Color::from_rgb8(140, 144, 159))
                        )
                        .padding([4, 6])
                        .style(|_theme, status| crate::theme::button_style(false)(_theme, status)),
                    ]
                    .spacing(4),
                )
                .padding([0, 4]),
                container(
                    text("account_circle")
                        .size(16)
                        .color(Color::from_rgb8(173, 198, 255))
                )
                .width(Length::Fixed(32.0))
                .height(Length::Fixed(32.0))
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: Some(Color::from_rgb8(77, 142, 255).into()),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    text_color: Some(Color::from_rgb8(0, 46, 106)),
                    snap: false,
                }),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 24])
        .height(Length::Fixed(56.0))
        .width(Length::Fill)
        .style(|_theme: &Theme| iced::widget::container::Style {
            background: Some(Color::from_rgb8(23, 31, 51).into()),
            border: iced::Border {
                radius: 0.0.into(),
                width: 1.0,
                color: Color::from_rgb8(66, 71, 84),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 8.0,
            },
            text_color: Some(Color::from_rgb8(218, 226, 253)),
            snap: false,
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
            scrollable(container(body).padding(24).width(Length::Fill)).height(Length::Fill);

        let main = column![top_bar, content].spacing(0).width(Length::Fill);

        let root = row![sidebar, main].height(Length::Fill);

        container(root)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| iced::widget::container::Style {
                background: Some(Color::from_rgb8(11, 19, 38).into()),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                text_color: Some(Color::from_rgb8(218, 226, 253)),
                snap: false,
            })
            .into()
    }
}
