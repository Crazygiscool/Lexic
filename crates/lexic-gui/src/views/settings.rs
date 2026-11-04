use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide::*;

impl App {
    pub fn settings_view(&self) -> Element<'_, Message> {
        let stats = self.storage_stats.clone().unwrap_or_default();
        let db_path = self
            .database_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| String::from("database unavailable"));
        let db_size = self
            .database_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok().map(|m| m.len()))
            .unwrap_or(0);

        let header = container(
            row![
                column![
                    text("Engine & Interop")
                        .size(20)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    row![
                        text("sys::engine")
                            .size(11)
                            .color(theme::TERTIARY)
                            .font(theme::font_code()),
                        text("/").size(11).color(theme::TEXT_MUTED),
                        text("storage-interop")
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code()),
                    ]
                    .spacing(4),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                container(
                    text(format!(
                        "{} {}",
                        std::env::consts::OS,
                        std::env::consts::ARCH
                    ))
                    .size(10)
                    .color(theme::PRIMARY)
                    .font(theme::font_code()),
                )
                .padding([3, 8])
                .style(theme::panel_flat(
                    theme::BG_CONTAINER_HIGH,
                    theme::BORDER_SUBTLE,
                    6.0
                )),
            ]
            .align_y(Alignment::Center),
        )
        .padding(8);

        // Storage card
        let storage_card = container(
            column![
                text("SQLite storage")
                    .size(14)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                components::stat_tile("Entries", stats.entry_count.to_string(), theme::PRIMARY),
                components::stat_tile("Senses", stats.sense_count.to_string(), theme::SECONDARY),
                components::stat_tile("Decks", stats.deck_count.to_string(), theme::TERTIARY),
                components::stat_tile("Cards", stats.card_count.to_string(), theme::WARNING),
                components::stat_tile("Reviews", stats.review_count.to_string(), theme::STRUCTURAL),
                components::stat_tile("Due", stats.due_count.to_string(), theme::ERROR),
                row![
                    text("Path")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text(db_path)
                        .size(11)
                        .color(theme::TEXT_SECONDARY)
                        .font(theme::font_code()),
                ]
                .spacing(8),
                row![
                    text("SQLite")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text(stats.sqlite_version.clone())
                        .size(11)
                        .color(theme::TEXT_SECONDARY)
                        .font(theme::font_code()),
                    text("Journal")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text(stats.journal_mode.clone())
                        .size(11)
                        .color(theme::TEXT_SECONDARY)
                        .font(theme::font_code()),
                    text("FK")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text(if stats.foreign_keys { "on" } else { "off" })
                        .size(11)
                        .color(theme::TEXT_SECONDARY)
                        .font(theme::font_code()),
                    text("Size")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text(format_bytes(db_size))
                        .size(11)
                        .color(theme::TEXT_SECONDARY)
                        .font(theme::font_code()),
                    text("Integrity")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text(stats.integrity.clone())
                        .size(11)
                        .color(if stats.integrity == "ok" {
                            theme::TERTIARY
                        } else {
                            theme::ERROR
                        })
                        .font(theme::font_code()),
                ]
                .spacing(8),
                row![
                    button(text("Verify").size(11).font(theme::font_code()))
                        .padding([6, 10])
                        .style(theme::secondary_button())
                        .on_press(Message::VerifyDatabase),
                    button(text("Vacuum").size(11).font(theme::font_code()))
                        .padding([6, 10])
                        .style(theme::secondary_button())
                        .on_press(Message::VacuumDatabase),
                ]
                .spacing(8),
            ]
            .spacing(8),
        )
        .padding(14)
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            10.0,
        ));

        // Backup card
        let mode_row = row![
            button(text("Merge").size(11).font(theme::font_code()).color(
                if self.import_mode == lexic_store::ImportMode::Merge {
                    theme::ON_PRIMARY_CONTAINER
                } else {
                    theme::TEXT_MUTED
                }
            ))
            .padding([6, 10])
            .style(theme::button_style(
                self.import_mode == lexic_store::ImportMode::Merge
            ))
            .on_press(Message::SetImportMode(lexic_store::ImportMode::Merge)),
            button(text("Replace").size(11).font(theme::font_code()).color(
                if self.import_mode == lexic_store::ImportMode::Replace {
                    theme::ON_PRIMARY_CONTAINER
                } else {
                    theme::TEXT_MUTED
                }
            ))
            .padding([6, 10])
            .style(theme::button_style(
                self.import_mode == lexic_store::ImportMode::Replace
            ))
            .on_press(Message::SetImportMode(lexic_store::ImportMode::Replace)),
        ]
        .spacing(6);

        let include_button = button(
            text(if self.include_review_history {
                "Include review history: on"
            } else {
                "Include review history: off"
            })
            .size(11)
            .font(theme::font_code())
            .color(if self.include_review_history {
                theme::ON_PRIMARY_CONTAINER
            } else {
                theme::TEXT_MUTED
            }),
        )
        .padding([6, 10])
        .style(theme::button_style(self.include_review_history))
        .on_press(Message::ToggleIncludeReviewHistory);

        let backup_card = container(
            column![
                text("Backup & restore")
                    .size(14)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                row![
                    button(
                        row![
                            download().size(13).color(theme::ON_PRIMARY_CONTAINER),
                            text("Export backup")
                                .size(12)
                                .color(theme::ON_PRIMARY_CONTAINER)
                                .font(theme::font_code())
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center)
                    )
                    .padding([8, 14])
                    .style(theme::primary_button())
                    .on_press(Message::BeginExport),
                    include_button,
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text("Import mode")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    mode_row,
                    button(
                        row![
                            upload().size(13).color(theme::TEXT_MUTED),
                            text("Import backup")
                                .size(12)
                                .color(theme::TEXT_MUTED)
                                .font(theme::font_code())
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center)
                    )
                    .padding([8, 14])
                    .style(theme::secondary_button())
                    .on_press(Message::BeginImport),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                if let Some(status) = &self.settings_status {
                    text(status)
                        .size(12)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code())
                } else {
                    text("").size(1)
                },
            ]
            .spacing(8),
        )
        .padding(14)
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            10.0,
        ));

        let content = column![header, storage_card, backup_card].spacing(12);
        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
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
