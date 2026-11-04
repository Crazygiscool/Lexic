use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide::*;

impl App {
    pub fn search_view(&self) -> Element<'_, Message> {
        let search_input = text_input("query vocabulary or symbol...", &self.search_input)
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::Lookup)
            .padding([10, 14])
            .width(Length::Fill)
            .style(theme::themed_input());

        let lookup_button = button(
            row![
                search().size(14).color(theme::ON_PRIMARY_CONTAINER),
                text("Look up")
                    .size(12)
                    .color(theme::ON_PRIMARY_CONTAINER)
                    .font(theme::font_code()),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([10, 16])
        .style(theme::primary_button())
        .on_press(Message::Lookup);

        let status_line = match self.search_status {
            SearchStatus::Loading => text("Loading definition...")
                .size(12)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code()),
            SearchStatus::Error => text("Lookup failed")
                .size(12)
                .color(theme::ERROR)
                .font(theme::font_code()),
            SearchStatus::Loaded => text("Definition loaded")
                .size(12)
                .color(theme::TERTIARY)
                .font(theme::font_code()),
            SearchStatus::Idle => text("Ready")
                .size(12)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code()),
        };

        let header = column![
            text("Grammar Dictionary")
                .size(20)
                .color(theme::TEXT_PRIMARY)
                .font(theme::font_code()),
            text("Search the dictionary, inspect senses, and save cards to a deck.")
                .size(12)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(4);

        // POS chips from active senses
        let mut pos_values: Vec<String> = self
            .active_senses
            .iter()
            .filter_map(|sense| sense.part_of_speech.clone())
            .collect();
        pos_values.sort();
        pos_values.dedup();
        let mut chips_row = row![
            text("Parts of speech:")
                .size(11)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code())
        ]
        .spacing(6)
        .align_y(Alignment::Center);
        chips_row = chips_row.push(
            button(text("all").size(11).font(theme::font_code()).color(
                if self.dictionary_pos_filter.is_none() {
                    theme::ON_PRIMARY_CONTAINER
                } else {
                    theme::TEXT_MUTED
                },
            ))
            .padding([4, 10])
            .style(theme::button_style(self.dictionary_pos_filter.is_none()))
            .on_press(Message::DictionaryPosFilterChanged(None)),
        );
        for pos in &pos_values {
            let active = self.dictionary_pos_filter.as_deref() == Some(pos.as_str());
            chips_row = chips_row.push(
                button(
                    text(pos.clone())
                        .size(11)
                        .font(theme::font_code())
                        .color(if active {
                            theme::ON_PRIMARY_CONTAINER
                        } else {
                            theme::TEXT_MUTED
                        }),
                )
                .padding([4, 10])
                .style(theme::button_style(active))
                .on_press(Message::DictionaryPosFilterChanged(Some(pos.clone()))),
            );
        }

        let search_bar = container(
            row![search_input, lookup_button, status_line]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .padding(12)
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            10.0,
        ));

        // Active entry card
        let entry_card: Element<'_, Message> = if let Some(entry) = &self.active_entry {
            let filtered_senses: Vec<&lexic_core::model::Sense> = self
                .active_senses
                .iter()
                .filter(|sense| {
                    self.dictionary_pos_filter
                        .as_ref()
                        .map(|pos| sense.part_of_speech.as_deref() == Some(pos.as_str()))
                        .unwrap_or(true)
                })
                .collect();

            let mut senses_column = column![].spacing(8);
            for sense in &filtered_senses {
                let mut chips = row![].spacing(6);
                for synonym in &sense.synonyms {
                    chips = chips.push(components::chip(
                        format!("syn: {synonym}"),
                        theme::SECONDARY,
                        theme::BG_CONTAINER_HIGH,
                    ));
                }
                for antonym in &sense.antonyms {
                    chips = chips.push(components::chip(
                        format!("ant: {antonym}"),
                        theme::ERROR,
                        theme::BG_CONTAINER_HIGH,
                    ));
                }
                let mut sense_body =
                    column![text(&sense.definition).size(14).color(theme::TEXT_PRIMARY),]
                        .spacing(4);
                if let Some(example) = &sense.example {
                    sense_body = sense_body.push(
                        text(format!("\"{example}\""))
                            .size(12)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code()),
                    );
                }
                if !sense.synonyms.is_empty() || !sense.antonyms.is_empty() {
                    sense_body = sense_body.push(chips);
                }
                senses_column = senses_column.push(
                    container(
                        column![
                            text(
                                sense
                                    .part_of_speech
                                    .clone()
                                    .unwrap_or_else(|| String::from("sense"))
                            )
                            .size(10)
                            .color(theme::PRIMARY)
                            .font(theme::font_code()),
                            sense_body,
                        ]
                        .spacing(6),
                    )
                    .padding(10)
                    .style(theme::panel_flat(
                        theme::BG_CONTAINER,
                        theme::BORDER_SUBTLE,
                        8.0,
                    )),
                );
            }
            if filtered_senses.is_empty() {
                senses_column = senses_column.push(
                    text("No senses for the selected part of speech.")
                        .size(12)
                        .color(theme::TEXT_MUTED),
                );
            }

            // deck selector
            let mut deck_buttons = row![].spacing(6);
            for deck in &self.decks {
                let selected = self.selected_save_deck == Some(deck.id);
                deck_buttons =
                    deck_buttons.push(
                        button(text(&deck.name).size(11).font(theme::font_code()).color(
                            if selected {
                                theme::ON_PRIMARY_CONTAINER
                            } else {
                                theme::TEXT_MUTED
                            },
                        ))
                        .padding([4, 10])
                        .style(theme::button_style(selected))
                        .on_press(Message::SelectSaveDeck(deck.id)),
                    );
            }
            if self.decks.is_empty() {
                deck_buttons =
                    deck_buttons.push(text("No decks").size(11).color(theme::TEXT_MUTED));
            }

            let save_row = row![
                deck_buttons,
                Space::new().width(Length::Fill),
                button(
                    row![
                        bookmark().size(13).color(theme::ON_PRIMARY_CONTAINER),
                        text("Save card")
                            .size(12)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code())
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .padding([8, 14])
                .style(theme::primary_button())
                .on_press(Message::SaveActiveEntry),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let save_panel = container(
                column![
                    text("Save to deck")
                        .size(12)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    save_row,
                    if let Some(status) = &self.save_status {
                        text(status)
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code())
                    } else {
                        text("").size(1)
                    },
                ]
                .spacing(8),
            )
            .padding(12)
            .style(theme::panel_flat(
                theme::BG_CONTAINER_LOW,
                theme::BORDER_SUBTLE,
                8.0,
            ));

            container(
                column![
                    row![
                        column![
                            text(&entry.word)
                                .size(34)
                                .color(theme::TEXT_PRIMARY)
                                .font(theme::font_code()),
                            entry
                                .phonetic
                                .as_ref()
                                .map(|value| text(value)
                                    .size(15)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code()))
                                .unwrap_or_else(|| text("").size(1)),
                            text(
                                entry
                                    .origin
                                    .clone()
                                    .unwrap_or_else(|| String::from("Etymology unknown"))
                            )
                            .size(12)
                            .color(theme::TEXT_SECONDARY),
                        ]
                        .spacing(4),
                        Space::new().width(Length::Fill),
                        components::chip(
                            entry.source.clone(),
                            theme::SECONDARY,
                            theme::BG_CONTAINER_HIGH
                        ),
                    ]
                    .align_y(Alignment::Center),
                    senses_column,
                    save_panel,
                ]
                .spacing(12),
            )
            .padding(16)
            .style(theme::panel_flat(
                theme::BG_CONTAINER,
                theme::BORDER_SUBTLE,
                10.0,
            ))
            .into()
        } else {
            components::empty_state(
                "Search a word",
                String::from("Enter a term and press Look up to load its definitions."),
            )
        };

        let mut body = column![header, search_bar].spacing(12);
        if self.search_status == SearchStatus::Loading {
            // no-op
        }
        body = body.push(chips_row);
        body = body.push(entry_card);

        let scroll = scrollable(body).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
