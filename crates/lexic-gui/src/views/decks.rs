#![allow(clippy::collapsible_if)]
use crate::app::*;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Color, Element, Length};

impl App {
    pub fn decks_view(&self) -> Element<'_, Message> {
        let mut content = column![
            text("Decks").size(32),
            row![
                text_input("New deck name", &self.new_deck_name)
                    .on_input(Message::NewDeckNameChanged)
                    .padding([10, 14])
                    .width(Length::Fill)
                    .style(|_theme, _status| text_input::Style {
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
                    }),
                button("Create deck").on_press(Message::CreateDeck),
            ]
            .spacing(8),
        ]
        .spacing(16);

        if let Some(status) = &self.deck_management_status {
            content = content.push(text(status).size(14));
        }

        for deck in &self.decks {
            let counts = self.deck_counts.get(&deck.id).copied().unwrap_or((0, 0, 0));
            let row_view = row![
                column![
                    text(&deck.name).size(20),
                    text(format!(
                        "Cards: {} · New: {} · Due: {}",
                        counts.0, counts.1, counts.2
                    ))
                    .size(12),
                ]
                .spacing(4)
                .width(Length::Fill),
                button("Cards").on_press(Message::OpenDeckDetail(deck.id)),
                button("Edit").on_press(Message::SelectManageDeck(deck.id)),
                button("Archive").on_press(Message::ArchiveDeck(deck.id)),
                button("Delete").on_press(Message::DeleteDeck(deck.id)),
            ]
            .spacing(8);

            content = content.push(container(row_view).padding(12).width(Length::Fill));
        }

        if let Some(selected_id) = self.selected_deck {
            if let Some(_deck) = self.decks.iter().find(|deck| deck.id == selected_id) {
                let editor = column![
                    text("Edit selected deck").size(18),
                    text_input("Deck name", &self.rename_deck_input)
                        .on_input(Message::RenameDeckNameChanged)
                        .width(Length::Fill),
                    row![
                        text_input("New cards per day", &self.deck_new_limit_input)
                            .on_input(Message::DeckNewLimitChanged)
                            .width(Length::Fill),
                        text_input("Reviews per day", &self.deck_review_limit_input)
                            .on_input(Message::DeckReviewLimitChanged)
                            .width(Length::Fill),
                        button("Save details").on_press(Message::SaveDeckDetails),
                    ]
                    .spacing(8),
                ]
                .spacing(8);

                content = content.push(container(editor).padding(16).width(Length::Fill));
            }
        }

        container(content).padding(32).into()
    }

    pub fn deck_detail_view(&self, deck_id: lexic_core::model::Id) -> Element<'_, Message> {
        let deck_name = self
            .decks
            .iter()
            .find(|deck| deck.id == deck_id)
            .map(|deck| deck.name.clone())
            .unwrap_or_else(|| String::from("Deck"));

        let mut content = column![
            text(format!("Deck: {deck_name}")).size(32),
            button("Back to decks").on_press(Message::Navigate(Screen::Decks)),
        ]
        .spacing(16);

        if let Some(status) = &self.deck_card_status {
            content = content.push(text(status).size(14));
        }

        for row_data in &self.deck_cards {
            let card_row = row![
                text(&row_data.word).size(18).width(Length::Fill),
                text(row_data.card.state.to_string()).size(12),
                button("Suspend").on_press(Message::SuspendCard(row_data.card.id)),
                button("Reset").on_press(Message::ResetCard(row_data.card.id)),
                button("Remove").on_press(Message::RemoveCard(row_data.card.id)),
            ]
            .spacing(8);

            content = content.push(container(card_row).padding(12).width(Length::Fill));
        }

        if self.deck_cards.is_empty() {
            content = content.push(text("No cards in this deck"));
        }

        container(content).padding(32).into()
    }
}
