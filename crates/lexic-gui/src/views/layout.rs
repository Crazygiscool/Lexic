use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide::*;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let due_count: i64 = self.deck_counts.values().map(|(_, _, due)| *due).sum();
        let transpiler_status = match (self.gset_status.as_deref(), self.gset_result.as_ref()) {
            (Some(status), _) if status.contains("Running") => "Transpiler: Running",
            (_, Some(result)) if result.success => "Transpiler: Idle (AST Clean)",
            (_, Some(_)) => "Transpiler: Failed",
            _ => "Transpiler: Idle",
        };
        let transpiler_dot_color = match transpiler_status {
            "Transpiler: Failed" => theme::ERROR,
            "Transpiler: Running" => theme::WARNING,
            _ => theme::TERTIARY,
        };

        let active_file = if !self.gset_editor_path.is_empty() {
            std::path::Path::new(&self.gset_editor_path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("source.ts")
                .to_string()
        } else {
            String::from("source.ts")
        };

        let db_size = self
            .database_path
            .as_ref()
            .and_then(|path| std::fs::metadata(path).ok().map(|meta| meta.len()))
            .unwrap_or(0);
        let db_path = self
            .database_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| String::from("database unavailable"));
        let sqlite_version = self
            .storage_stats
            .as_ref()
            .map(|stats| stats.sqlite_version.clone())
            .unwrap_or_else(|| String::from("unknown"));
        let journal_mode = self
            .storage_stats
            .as_ref()
            .map(|stats| stats.journal_mode.clone())
            .unwrap_or_else(|| String::from("unknown"));
        let version_chip = option_env!("CARGO_PKG_VERSION").unwrap_or("dev");

        // Sidebar
        let sidebar_header = row![
            container(
                text("\u{03bb}")
                    .size(16)
                    .color(theme::ON_PRIMARY_CONTAINER)
                    .font(theme::font_code())
            )
            .width(28)
            .height(28)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(theme::panel_flat(
                theme::PRIMARY_CONTAINER,
                theme::PRIMARY_CONTAINER,
                6.0,
            )),
            column![
                text("LEXIC")
                    .size(13)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                text("GSET Studio")
                    .size(10)
                    .color(theme::PRIMARY)
                    .font(theme::font_code()),
            ]
            .spacing(0),
            Space::new().width(Length::Fill),
            container(
                row![
                    container(row![
                        components::status_dot(theme::TERTIARY),
                        text(version_chip).size(10).color(theme::TERTIARY)
                    ])
                    .padding([1, 6]),
                ]
                .spacing(0),
            )
            .style(theme::panel_flat(
                theme::BG_CONTAINER_HIGH,
                theme::BORDER_SUBTLE,
                6.0
            )),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let spec_pill = container(
            row![
                code().size(13).color(theme::TERTIARY),
                text("Grammar Spec")
                    .size(11)
                    .color(theme::TEXT_SECONDARY)
                    .font(theme::font_code()),
                Space::new().width(Length::Fill),
                chip_text(self.gset_bridge.version.clone()),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .style(theme::panel_flat(
            theme::BG_CONTAINER,
            theme::BORDER_SUBTLE,
            6.0,
        ));

        let nav = column![
            nav_link(
                self,
                Screen::Home,
                layout_dashboard(),
                "Home",
                "\u{2325}1",
                None
            ),
            nav_link(
                self,
                Screen::Project,
                code(),
                "Transpiler IDE",
                "\u{2325}2",
                None
            ),
            nav_link(
                self,
                Screen::Review,
                library(),
                "Review Deck",
                "Due",
                Some(due_count),
            ),
            nav_link(
                self,
                Screen::Search,
                book_open(),
                "Grammar Dict",
                "\u{2325}4",
                None
            ),
            nav_link(
                self,
                Screen::Settings,
                settings(),
                "Engine & Interop",
                "\u{2325}5",
                None
            ),
        ]
        .spacing(4);

        let queue_text = format!("{due_count} SRS pending");
        let bridge_label = if self.gset_bridge.available {
            self.gset_bridge
                .version
                .clone()
                .unwrap_or_else(|| String::from("available"))
        } else {
            String::from("unavailable")
        };

        let footer = column![
            column![
                text("Engine Daemons")
                    .size(10)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code()),
                row![
                    text("2 OK")
                        .size(10)
                        .color(theme::TERTIARY)
                        .font(theme::font_code()),
                ],
            ]
            .spacing(2),
            panel_line("rusqlite", &sqlite_version, "WAL mode", &db_path),
            panel_line(
                "IPC Daemon",
                &bridge_label,
                "GSET bridge",
                &self.gset_bridge.path.display().to_string()
            ),
            container(
                row![
                    text("Queue Load")
                        .size(11)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    Space::new().width(Length::Fill),
                    text(queue_text)
                        .size(11)
                        .color(theme::PRIMARY)
                        .font(theme::font_code()),
                ]
                .align_y(Alignment::Center),
            )
            .padding([6, 8])
            .style(theme::panel_flat(
                theme::BG_CONTAINER,
                theme::BORDER_SUBTLE,
                6.0
            )),
            row![
                text(format_bytes(db_size))
                    .size(10)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code()),
                Space::new().width(Length::Fill),
                text(journal_mode)
                    .size(10)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code()),
            ],
        ]
        .spacing(8);

        let sidebar = container(
            column![
                column![sidebar_header, spec_pill, nav].spacing(14),
                Space::new().height(Length::Fill),
                footer,
            ]
            .spacing(12),
        )
        .width(Length::Fixed(288.0))
        .height(Length::Fill)
        .padding([10, 12])
        .style(theme::panel_shell());

        // Top bar
        let top_bar = container(
            row![
                container(
                    row![
                        file_text().size(13).color(theme::PRIMARY),
                        text(active_file)
                            .size(12)
                            .color(theme::TEXT_PRIMARY)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([4, 10])
                .style(theme::panel_flat(theme::BG_CONTAINER, theme::PRIMARY, 6.0)),
                container(
                    row![
                        git_branch().size(12).color(theme::SECONDARY),
                        text(self.gset_target_language.to_uppercase())
                            .size(11)
                            .color(theme::TEXT_SECONDARY)
                            .font(theme::font_code()),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .padding([4, 8])
                .style(theme::panel_flat(
                    theme::BG_CONTAINER_HIGH,
                    theme::BORDER_SUBTLE,
                    6.0
                )),
                Space::new().width(8),
                row![
                    container(Space::new())
                        .width(1)
                        .height(16)
                        .style(theme::panel_flat(
                            theme::BG_CONTAINER_HIGHEST,
                            theme::BG_CONTAINER_HIGHEST,
                            0.0
                        )),
                ],
                Space::new().width(8),
                row![
                    components::status_dot(transpiler_dot_color),
                    text(transpiler_status.trim_start_matches("Transpiler: "))
                        .size(11)
                        .color(theme::TEXT_SECONDARY)
                        .font(theme::font_code()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
                Space::new().width(Length::Fill),
                button(
                    row![
                        search().size(13).color(theme::TEXT_MUTED),
                        text("Find Token")
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code()),
                        components::kbd("^K"),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(theme::ghost_button())
                .on_press(Message::FindToken),
                button(
                    row![
                        text("+").size(13).color(theme::ON_PRIMARY_CONTAINER),
                        text("Quick Card")
                            .size(11)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code()),
                        container(
                            text("^N")
                                .size(10)
                                .color(theme::ON_PRIMARY_CONTAINER)
                                .font(theme::font_code())
                        )
                        .padding([1, 5])
                        .style(theme::panel_flat(
                            theme::PRIMARY_CONTAINER,
                            theme::PRIMARY_CONTAINER,
                            4.0
                        )),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(theme::primary_button())
                .on_press(Message::Navigate(Screen::Search)),
                button(
                    row![
                        upload().size(13).color(theme::TEXT_MUTED),
                        text("Export")
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(theme::ghost_button())
                .on_press(Message::BeginExport),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .height(56)
        .padding([0, 20])
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BG_CONTAINER_HIGHEST,
            0.0,
        ));

        // Content
        let content: Element<'_, Message> = match self.screen {
            Screen::Home => self.home_view(),
            Screen::Project => self.project_view(),
            Screen::Review => self.review_view(),
            Screen::Search => self.search_view(),
            Screen::Settings => self.settings_view(),
            _ => self.home_view(),
        };

        let body = container(content).width(Length::Fill).height(Length::Fill);

        row![
            sidebar,
            column![top_bar, body]
                .width(Length::Fill)
                .height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn nav_link(
    app: &App,
    screen: Screen,
    icon: iced::widget::Text<'static>,
    label: &'static str,
    badge: &'static str,
    count: Option<i64>,
) -> Element<'static, Message> {
    let active = app.screen == screen;
    let right: Element<'static, Message> = if let Some(count) = count {
        if count > 0 {
            container(
                text(format!("{}", count))
                    .size(10)
                    .color(theme::ON_SECONDARY_CONTAINER)
                    .font(theme::font_code()),
            )
            .padding([1, 6])
            .style(theme::panel_flat(
                theme::SECONDARY_CONTAINER,
                theme::SECONDARY_CONTAINER,
                6.0,
            ))
            .into()
        } else {
            text(badge)
                .size(10)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code())
                .into()
        }
    } else {
        text(badge)
            .size(10)
            .color(theme::TEXT_MUTED)
            .font(theme::font_code())
            .into()
    };

    button(
        row![
            icon.size(16).color(if active {
                theme::PRIMARY
            } else {
                theme::TEXT_SECONDARY
            }),
            text(label)
                .size(13)
                .color(if active {
                    theme::PRIMARY
                } else {
                    theme::TEXT_SECONDARY
                })
                .font(theme::font_code()),
            Space::new().width(Length::Fill),
            right
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(theme::nav_item(active))
    .on_press(Message::Navigate(screen))
    .into()
}

fn panel_line(
    title: &'static str,
    value: &str,
    tag: &'static str,
    detail: &str,
) -> Element<'static, Message> {
    let value = value.to_string();
    let detail = detail.to_string();
    container(
        column![
            row![
                row![
                    components::status_dot(theme::TERTIARY),
                    text(title)
                        .size(11)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
                Space::new().width(Length::Fill),
                container(
                    text(tag)
                        .size(9)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code())
                )
                .padding([1, 5])
                .style(theme::panel_flat(
                    theme::BG_CONTAINER_HIGH,
                    theme::BORDER_SUBTLE,
                    4.0
                )),
            ]
            .align_y(Alignment::Center),
            text(detail).size(9).color(theme::TEXT_MUTED),
            text(value)
                .size(9)
                .color(theme::TEXT_SECONDARY)
                .font(theme::font_code()),
        ]
        .spacing(2),
    )
    .padding([6, 8])
    .style(theme::panel_flat(
        theme::BG_CONTAINER_LOW,
        theme::BORDER_SUBTLE,
        6.0,
    ))
    .into()
}

fn chip_text(value: Option<String>) -> Element<'static, Message> {
    let value = value.unwrap_or_else(|| String::from("GSET"));
    container(
        text(value)
            .size(10)
            .color(theme::TERTIARY)
            .font(theme::font_code()),
    )
    .padding([1, 6])
    .style(theme::panel_flat(
        theme::BG_CONTAINER_HIGH,
        theme::BORDER_SUBTLE,
        6.0,
    ))
    .into()
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
