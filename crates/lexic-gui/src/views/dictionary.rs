use crate::app::*;
use iced::widget::{button, column, container, text};
use iced::{Element, Length};

impl App {
    pub fn search_view(&self) -> Element<'_, Message> {
        let mut content = column![].spacing(16);

        match self.search_status {
            SearchStatus::Loading => content = content.push(text("Loading definition...")),
            SearchStatus::Error => {}
            _ => {}
        }

        if let Some(status) = &self.save_status {
            content = content.push(text(status).size(14));
        }

        if let Some(entry) = &self.active_entry {
            let title: Element<'_, Message> = text(&entry.word).size(44).into();
            let phonetic = entry
                .phonetic
                .as_ref()
                .map(|value| text(value).size(18))
                .unwrap_or_else(|| text("").size(1));
            let mut body_lines = Vec::new();
            for sense in &self.active_senses {
                body_lines.push(text(&sense.definition).size(16).into());
                if let Some(example) = &sense.example {
                    body_lines.push(text(example).size(14).into());
                }
            }
            let body_text = column(body_lines).spacing(8);
            let details = container(
                column![title, phonetic, text(&entry.source).size(12), body_text].spacing(12),
            )
            .padding(24)
            .width(Length::Fill);

            let mut deck_buttons = column![text("Save to deck").size(16)].spacing(8);
            let raw_buttons = self
                .decks
                .iter()
                .fold(column![].spacing(8), |mut column, deck| {
                    column = column.push(
                        button(text(&deck.name).size(14))
                            .width(Length::Fill)
                            .on_press(Message::SelectSaveDeck(deck.id)),
                    );
                    column
                });
            deck_buttons = deck_buttons.push(raw_buttons);
            if self.decks.is_empty() {
                deck_buttons = deck_buttons
                    .push(text("Create a deck in the Decks tab to save words").size(12));
            }
            deck_buttons = deck_buttons
                .push(button("Save to selected deck").on_press(Message::SaveActiveEntry));

            content = content.push(details);
            content = content.push(container(deck_buttons).width(Length::Fill));
        }

        container(content).padding(32).into()
    }
}
