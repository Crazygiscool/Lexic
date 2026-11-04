use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::{
    Space, button, column, container, progress_bar as widget_progress_bar, row, scrollable, text,
};
use iced::{Alignment, Color, Element, Length};
use iced_fonts::lucide::*;
use lexic_core::schedule::{CardSnapshot, Scheduler, Sm2};
use lexic_core::time::now_utc;

impl App {
    pub fn review_view(&self) -> Element<'_, Message> {
        let due_count: i64 = self.deck_counts.values().map(|(_, _, due)| *due).sum();
        let new_count: i64 = self.deck_counts.values().map(|(_, _, new)| *new).sum();
        let session_progress = self.review.as_ref().map(|session| {
            let total = session.cards.len().max(1) as f32;
            let reviewed = session.reviewed.min(session.cards.len()) as f32;
            (reviewed, total)
        });

        let header = column![
            row![
                column![
                    text("Review Deck")
                        .size(20)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    text("Most modern implementations utilize spaced repetition intervals...")
                        .size(12)
                        .color(theme::TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                row![
                    text("Active Deck")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    // deck count
                ]
                .spacing(6),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(8);

        let deck_header_card = container(
            row![
                container(
                    row![
                        layers().size(16).color(theme::ON_PRIMARY_CONTAINER),
                        text("SRS Collection")
                            .size(11)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code())
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .padding([4, 8])
                .style(theme::panel_flat(
                    theme::PRIMARY_CONTAINER,
                    theme::PRIMARY_CONTAINER,
                    6.0
                )),
                Space::new().width(Length::Fill),
                row![
                    components::stat_tile("Due", due_count.to_string(), theme::TERTIARY),
                    components::stat_tile("New", new_count.to_string(), theme::PRIMARY),
                ]
                .spacing(8),
            ]
            .align_y(Alignment::Center),
        )
        .padding(12)
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            10.0,
        ));

        if self.review.is_none() {
            // start screen
            let mut deck_list = column![].spacing(8);
            for deck in &self.decks {
                let counts = self.deck_counts.get(&deck.id).copied().unwrap_or((0, 0, 0));
                deck_list = deck_list.push(
                    container(
                        row![
                            column![
                                text(&deck.name)
                                    .size(14)
                                    .color(theme::TEXT_PRIMARY)
                                    .font(theme::font_code()),
                                text(
                                    deck.description
                                        .clone()
                                        .unwrap_or_else(|| String::from("No description"))
                                )
                                .size(11)
                                .color(theme::TEXT_MUTED),
                            ]
                            .spacing(2),
                            Space::new().width(Length::Fill),
                            components::chip(
                                format!("{} total", counts.0),
                                theme::TEXT_SECONDARY,
                                theme::BG_CONTAINER_HIGH
                            ),
                            components::chip(
                                format!("{} due", counts.2),
                                theme::TERTIARY,
                                theme::BG_CONTAINER_HIGH
                            ),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding(12)
                    .style(theme::panel_flat(
                        theme::BG_CONTAINER,
                        theme::BORDER_SUBTLE,
                        8.0,
                    )),
                );
            }
            if self.decks.is_empty() {
                deck_list = deck_list.push(components::empty_state(
                    "No decks",
                    String::from("Create a deck in the data store to begin."),
                ));
            }

            let banner = self.review_status.as_ref().map(|status| {
                container(
                    text(status)
                        .size(12)
                        .color(theme::WARNING)
                        .font(theme::font_code()),
                )
                .padding(8)
                .style(theme::panel_flat(theme::BG_CONTAINER, theme::WARNING, 6.0))
            });

            let start_button = button(
                row![
                    play().size(14).color(theme::ON_PRIMARY_CONTAINER),
                    text("Start review session")
                        .size(12)
                        .color(theme::ON_PRIMARY_CONTAINER)
                        .font(theme::font_code()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([10, 16])
            .style(theme::primary_button())
            .on_press(Message::StartReview);

            let mut col = column![deck_header_card, deck_list].spacing(12);
            if let Some(banner) = banner {
                col = col.push(banner);
            }
            let mut with_button = col;
            with_button = with_button.push(row![start_button].align_y(Alignment::Center));
            let scroll = scrollable(with_button).height(Length::Fill);
            return container(column![header, scroll].spacing(12))
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let session = self.review.as_ref().unwrap();

        if session.current.is_none() {
            let accuracy = if session.reviewed == 0 {
                0.0
            } else {
                session.correct as f64 / session.reviewed as f64 * 100.0
            };
            let summary = container(
                column![
                    text("Session complete")
                        .size(22)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    components::stat_tile("Reviewed", session.reviewed.to_string(), theme::PRIMARY),
                    components::stat_tile("Accuracy", format!("{accuracy:.0}%"), theme::TERTIARY),
                    button(text("Done").size(12).font(theme::font_code()))
                        .padding([8, 16])
                        .style(theme::primary_button())
                        .on_press(Message::EndReview),
                ]
                .spacing(10),
            )
            .padding(18)
            .style(theme::panel_flat(
                theme::BG_CONTAINER,
                theme::BORDER_SUBTLE,
                10.0,
            ));
            let scroll = scrollable(column![header, deck_header_card, summary].spacing(12))
                .height(Length::Fill);
            return container(scroll)
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let current_index = session.current.unwrap();
        let review_card = &session.cards[current_index];
        let now = now_utc();
        let previews = [
            lexic_core::model::Rating::Again,
            lexic_core::model::Rating::Hard,
            lexic_core::model::Rating::Good,
            lexic_core::model::Rating::Easy,
        ]
        .map(|rating| {
            let outcome =
                Sm2::default().grade(&CardSnapshot::from_card(&review_card.card), rating, now);
            let minutes = (outcome.due_at - now).num_minutes();
            let interval = if minutes < 60 {
                format!("{} m", minutes.max(1))
            } else if minutes < 60 * 24 {
                format!("{} h", minutes / 60)
            } else {
                format!("{} d", (minutes + 60 * 24 - 1) / (60 * 24))
            };
            (rating, interval)
        });

        let front = if let Some(entry) = &review_card.entry {
            column![
                text(&entry.word)
                    .size(40)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                entry
                    .phonetic
                    .as_ref()
                    .map(|value| text(value)
                        .size(16)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()))
                    .unwrap_or_else(|| text("").size(1)),
            ]
            .spacing(6)
        } else {
            column![text("Unknown word").size(30).color(theme::TEXT_PRIMARY)].spacing(6)
        };

        let (reviewed, total) = session_progress.unwrap_or((0.0, 1.0));
        let progress_panel = container(
            column![
                row![
                    text(format!(
                        "Progress {}/{}",
                        session.reviewed,
                        session.cards.len()
                    ))
                    .size(11)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code()),
                    Space::new().width(Length::Fill),
                    text(format!("EF {:.2}", review_card.card.ease_factor))
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                ]
                .align_y(Alignment::Center),
                widget_progress_bar(0.0..=total, reviewed).girth(6),
            ]
            .spacing(6),
        )
        .padding(12)
        .style(theme::panel_flat(
            theme::BG_CONTAINER,
            theme::BORDER_SUBTLE,
            8.0,
        ));

        let card_container = if session.revealed {
            let mut back_lines = column![].spacing(8);
            for sense in &review_card.senses {
                back_lines = back_lines.push(
                    container(
                        column![
                            text(sense.part_of_speech.clone().unwrap_or_default())
                                .size(10)
                                .color(theme::PRIMARY)
                                .font(theme::font_code()),
                            text(&sense.definition).size(15).color(theme::TEXT_PRIMARY),
                            sense
                                .example
                                .as_ref()
                                .map(|example| text(example).size(12).color(theme::TEXT_MUTED))
                                .unwrap_or_else(|| text("").size(1)),
                        ]
                        .spacing(4),
                    )
                    .padding(10)
                    .style(theme::panel_flat(
                        theme::BG_CONTAINER,
                        theme::BORDER_SUBTLE,
                        8.0,
                    )),
                );
            }
            if review_card.senses.is_empty() {
                back_lines = back_lines.push(
                    text("No definitions available")
                        .size(12)
                        .color(theme::TEXT_MUTED),
                );
            }

            let ratings = container(
                row![
                    rating_button(
                        "Again",
                        theme::ERROR,
                        previews[0].1.clone(),
                        Message::Grade(previews[0].0)
                    ),
                    rating_button(
                        "Hard",
                        theme::WARNING,
                        previews[1].1.clone(),
                        Message::Grade(previews[1].0)
                    ),
                    rating_button(
                        "Good",
                        theme::TERTIARY,
                        previews[2].1.clone(),
                        Message::Grade(previews[2].0)
                    ),
                    rating_button(
                        "Easy",
                        theme::PRIMARY,
                        previews[3].1.clone(),
                        Message::Grade(previews[3].0)
                    ),
                ]
                .spacing(8),
            )
            .width(Length::Fill);

            let undo_row = row![
                button(
                    row![
                        rotate_ccw().size(13).color(theme::TEXT_MUTED),
                        text("Undo")
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code())
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .padding([6, 10])
                .style(theme::ghost_button())
                .on_press(Message::UndoReview),
                Space::new().width(Length::Fill),
                button(text("End session").size(11).font(theme::font_code()))
                    .padding([6, 10])
                    .style(theme::ghost_button())
                    .on_press(Message::EndReview),
            ]
            .align_y(Alignment::Center);

            container(
                column![front, back_lines, ratings, undo_row]
                    .spacing(16)
                    .width(Length::Fill),
            )
            .padding(18)
            .style(theme::panel_flat(
                theme::BG_CONTAINER_LOW,
                theme::BORDER_SUBTLE,
                10.0,
            ))
        } else {
            container(
                column![
                    front,
                    button(
                        row![
                            eye().size(13).color(theme::ON_PRIMARY_CONTAINER),
                            text("Show answer")
                                .size(12)
                                .color(theme::ON_PRIMARY_CONTAINER)
                                .font(theme::font_code())
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center)
                    )
                    .padding([10, 16])
                    .style(theme::primary_button())
                    .on_press(Message::RevealAnswer),
                    text("Press Space to reveal")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                ]
                .spacing(16)
                .width(Length::Fill),
            )
            .padding(18)
            .style(theme::panel_flat(
                theme::BG_CONTAINER_LOW,
                theme::BORDER_SUBTLE,
                10.0,
            ))
        };

        let scroll = scrollable(
            column![header, deck_header_card, progress_panel, card_container].spacing(12),
        )
        .height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn rating_button(
    label: &'static str,
    color: Color,
    interval: String,
    message: Message,
) -> Element<'static, Message> {
    button(
        column![
            text(label).size(12).color(color).font(theme::font_code()),
            text(interval)
                .size(10)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code()),
        ]
        .spacing(2)
        .align_x(iced::Alignment::Center),
    )
    .padding([8, 10])
    .width(Length::Fill)
    .style(theme::secondary_button())
    .on_press(message)
    .into()
}
