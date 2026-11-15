use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::{
    Space, button, column, container, progress_bar as widget_progress_bar, row, scrollable, text,
};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide::*;
use lexic_core::schedule::{CardSnapshot, Scheduler, Sm2};
use lexic_core::time::now_utc;

impl App {
    pub fn review_view(&self) -> Element<'_, Message> {
        let due_count: i64 = self.deck_counts.values().map(|(_, _, due)| *due).sum();
        let new_count: i64 = self.deck_counts.values().map(|(_, _, new)| *new).sum();

        let header = column![
            row![
                column![
                    text("Review Deck")
                        .size(20)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    text("Spaced repetition flashcard quiz")
                        .size(12)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                row![
                    text("Active Deck")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
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
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([4, 8])
                .style(theme::panel_flat(
                    theme::PRIMARY_CONTAINER,
                    theme::PRIMARY_CONTAINER,
                    6.0,
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

        // If no active batch, show deck selection / start screen
        if self.review_batch.is_empty() && self.review.is_none() {
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
                                        .unwrap_or_else(|| String::from("No description")),
                                )
                                .size(11)
                                .color(theme::TEXT_MUTED),
                            ]
                            .spacing(2),
                            Space::new().width(Length::Fill),
                            components::chip(
                                format!("{} total", counts.0),
                                theme::TEXT_SECONDARY,
                                theme::BG_CONTAINER_HIGH,
                            ),
                            components::chip(
                                format!("{} due", counts.2),
                                theme::TERTIARY,
                                theme::BG_CONTAINER_HIGH,
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
            .on_press(Message::LoadNextBatch);

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

        // Quiz in progress
        let batch_len = self.review_batch.len();
        let current_idx = self.review_batch_index.min(batch_len.saturating_sub(1));
        let current_card = self.review_batch.get(current_idx);

        // Progress
        let progress = if batch_len > 0 {
            (self.review_batch_index as f32) / (batch_len as f32)
        } else {
            0.0
        };

        let progress_panel = container(
            column![
                row![
                    text(format!("Card {}", current_idx + 1))
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    Space::new().width(Length::Fill),
                    text(format!("of {}", batch_len))
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                ]
                .align_y(Alignment::Center),
                widget_progress_bar(0.0..=1.0, progress).girth(6),
            ]
            .spacing(8),
        )
        .padding(12)
        .style(theme::panel_flat(
            theme::BG_CONTAINER,
            theme::BORDER_SUBTLE,
            8.0,
        ));

        // Card content
        let card_content = if let Some(card) = current_card {
            self.card_front_view(card)
        } else {
            // Batch complete
            let accuracy = if self.review_batch_total_count > 0 {
                self.review_batch_correct_count as f64 / self.review_batch_total_count as f64
                    * 100.0
            } else {
                0.0
            };
            container(
                column![
                    text("Batch Complete")
                        .size(22)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    components::stat_tile(
                        "Cards",
                        self.review_batch_total_count.to_string(),
                        theme::PRIMARY
                    ),
                    components::stat_tile(
                        "Correct",
                        self.review_batch_correct_count.to_string(),
                        theme::TERTIARY
                    ),
                    components::stat_tile("Accuracy", format!("{accuracy:.0}%"), theme::SECONDARY),
                    button(text("Start New Batch").size(12).font(theme::font_code()))
                        .padding([8, 16])
                        .style(theme::primary_button())
                        .on_press(Message::LoadNextBatch),
                ]
                .spacing(10),
            )
            .padding(18)
            .style(theme::panel_flat(
                theme::BG_CONTAINER,
                theme::BORDER_SUBTLE,
                10.0,
            ))
            .into()
        };

        let content = column![header, deck_header_card, progress_panel, card_content].spacing(12);

        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn card_front_view<'a>(&'a self, card: &'a ReviewCard) -> Element<'a, Message> {
        let word = card
            .entry
            .as_ref()
            .map(|e| e.word.clone())
            .unwrap_or_default();
        let phonetic = card.entry.as_ref().and_then(|e| e.phonetic.clone());
        let correct_definition = card
            .senses
            .first()
            .map(|s| s.definition.clone())
            .unwrap_or_default();

        // Word display
        let word_display = column![
            text(word)
                .size(34)
                .color(theme::TEXT_PRIMARY)
                .font(theme::font_code()),
            phonetic.map(|p| {
                text(p)
                    .size(16)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code())
            }),
        ]
        .spacing(4);

        // Question
        let question = text("What is the definition?")
            .size(14)
            .color(theme::TEXT_SECONDARY)
            .font(theme::font_code());

        // Answer options
        let options: Element<'_, Message> = if self.review_batch_revealed {
            // Show correct answer highlighted
            let is_correct = self.review_batch_correct;
            let color = if is_correct {
                theme::TERTIARY
            } else {
                theme::ERROR
            };
            let icon = if is_correct {
                check().size(16)
            } else {
                x().size(16)
            };

            column![
                row![
                    icon.color(color),
                    text(correct_definition)
                        .size(14)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(if is_correct { "Correct!" } else { "Incorrect" })
                    .size(12)
                    .color(color)
                    .font(theme::font_code()),
            ]
            .spacing(8)
            .into()
        } else {
            let mut buttons = column![].spacing(8);
            for (idx, option) in self.review_batch_options.iter().enumerate() {
                let is_selected = self.review_batch_selected == Some(idx);
                let btn = button(
                    row![
                        text(option)
                            .size(13)
                            .color(theme::TEXT_PRIMARY)
                            .font(theme::font_code()),
                        Space::new().width(Length::Fill),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([10, 14])
                .width(Length::Fill)
                .style(theme::button_style(is_selected))
                .on_press(Message::SelectAnswer(idx));
                buttons = buttons.push(btn);
            }
            buttons.into()
        };

        // Rating buttons (shown after revealing)
        let ratings: Element<'_, Message> = if self.review_batch_revealed {
            let now = now_utc();
            let card_snapshot = CardSnapshot::from_card(&card.card);
            let previews = [
                (lexic_core::model::Rating::Again, theme::ERROR),
                (lexic_core::model::Rating::Hard, theme::WARNING),
                (lexic_core::model::Rating::Good, theme::TERTIARY),
                (lexic_core::model::Rating::Easy, theme::PRIMARY),
            ];

            let mut rating_buttons: Vec<Element<'_, Message>> = Vec::new();
            for (rating, color) in previews.iter() {
                let outcome = Sm2::default().grade(&card_snapshot, *rating, now);
                let minutes = (outcome.due_at - now).num_minutes();
                let interval = if minutes < 60 {
                    format!("{} m", minutes.max(1))
                } else if minutes < 60 * 24 {
                    format!("{} h", minutes / 60)
                } else {
                    format!("{} d", (minutes + 60 * 24 - 1) / (60 * 24))
                };

                let btn = button(
                    column![
                        text(match rating {
                            lexic_core::model::Rating::Again => "Again",
                            lexic_core::model::Rating::Hard => "Hard",
                            lexic_core::model::Rating::Good => "Good",
                            lexic_core::model::Rating::Easy => "Easy",
                        })
                        .size(12)
                        .color(*color)
                        .font(theme::font_code()),
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
                .on_press(Message::Grade(*rating));
                rating_buttons.push(btn.into());
            }
            let mut rating_row = row![].spacing(8);
            for btn in rating_buttons {
                rating_row = rating_row.push(btn);
            }
            rating_row.into()
        } else {
            text("").into()
        };

        // Navigation buttons
        let mut nav_items: Vec<Element<'_, Message>> = Vec::new();
        if self.review_batch_index > 0 {
            nav_items.push(
                button(
                    row![
                        rotate_ccw().size(13).color(theme::ON_PRIMARY_CONTAINER),
                        text("Previous")
                            .size(11)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::primary_button())
                .on_press(Message::PrevCard)
                .into(),
            );
        }
        nav_items.push(Space::new().width(Length::Fill).into());
        nav_items.push(
            button(
                row![
                    text("Next Card")
                        .size(11)
                        .color(theme::ON_PRIMARY_CONTAINER)
                        .font(theme::font_code()),
                    chevron_right().size(13).color(theme::ON_PRIMARY_CONTAINER),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([8, 14])
            .style(theme::primary_button())
            .on_press(Message::NextCard)
            .into(),
        );
        let mut nav = row![].spacing(8);
        for item in nav_items {
            nav = nav.push(item);
        }
        let nav = nav;

        // Senses detail (shown after revealing)
        let senses_detail = if self.review_batch_revealed && !card.senses.is_empty() {
            let mut senses_lines: Vec<Element<'_, Message>> = Vec::new();
            for sense in &card.senses {
                senses_lines.push(
                    container(
                        column![
                            text(sense.part_of_speech.clone().unwrap_or_default())
                                .size(10)
                                .color(theme::PRIMARY)
                                .font(theme::font_code()),
                            text(&sense.definition).size(14).color(theme::TEXT_PRIMARY),
                            sense.example.as_ref().map(|example| {
                                text(format!("\"{example}\""))
                                    .size(12)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code())
                            }),
                        ]
                        .spacing(4),
                    )
                    .padding(10)
                    .style(theme::panel_flat(
                        theme::BG_CONTAINER,
                        theme::BORDER_SUBTLE,
                        8.0,
                    ))
                    .into(),
                );
            }
            let mut senses_column = column![
                text("Definitions")
                    .size(12)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code()),
            ]
            .spacing(6);
            for line in senses_lines {
                senses_column = senses_column.push(line);
            }
            let senses_column: Element<'_, Message> = senses_column.into();
            senses_column
        } else {
            text("").into()
        };

        let middle: Element<'_, Message> = if self.review_batch_revealed {
            column![senses_detail, ratings].spacing(12).into()
        } else {
            text("").into()
        };

        container(column![word_display, question, options, middle, nav].spacing(16))
            .padding(18)
            .style(theme::panel_flat(
                theme::BG_CONTAINER_LOW,
                theme::BORDER_SUBTLE,
                10.0,
            ))
            .into()
    }
}
