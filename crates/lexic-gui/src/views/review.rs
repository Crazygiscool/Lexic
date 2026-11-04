use crate::app::*;
use iced::Element;
use iced::widget::{button, column, container, row, text};
use lexic_core::schedule::{CardSnapshot, Scheduler, Sm2};
use lexic_core::time::now_utc;

impl App {
    pub fn review_view(&self) -> Element<'_, Message> {
        if let Some(status) = &self.review_status {
            let _ = status;
        }

        let Some(session) = &self.review else {
            return container(
                column![
                    text("Review").size(32),
                    button("Start review session").on_press(Message::StartReview),
                ]
                .spacing(16),
            )
            .padding(32)
            .into();
        };

        if session.current.is_none() {
            let accuracy = if session.reviewed == 0 {
                0.0
            } else {
                session.correct as f64 / session.reviewed as f64 * 100.0
            };
            return container(
                column![
                    text("Session complete").size(32),
                    text(format!("Reviewed: {}", session.reviewed)),
                    text(format!("Accuracy: {accuracy:.0}%")),
                    button("Done").on_press(Message::EndReview),
                ]
                .spacing(12),
            )
            .padding(32)
            .into();
        }

        let Some(current_index) = session.current else {
            return container(text("No card").size(24))
                .padding(32)
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb8(23, 31, 51).into()),
                    border: iced::Border {
                        radius: 12.0.into(),
                        width: 1.0,
                        color: iced::Color::from_rgb8(66, 71, 84),
                    },
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 14.0,
                    },
                    text_color: Some(iced::Color::from_rgb8(218, 226, 253)),
                    snap: false,
                })
                .into();
        };
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
            let label = if minutes < 60 {
                format!("{rating:?} {minutes}m")
            } else if minutes < 60 * 24 {
                format!("{rating:?} {}h", minutes / 60)
            } else {
                format!("{rating:?} {}d", (minutes / 60 + 23) / 24)
            };
            (rating, label)
        });

        let front = if let Some(entry) = &review_card.entry {
            column![
                text(&entry.word).size(44),
                entry
                    .phonetic
                    .as_ref()
                    .map(|value| text(value).size(18))
                    .unwrap_or_else(|| text("").size(1)),
            ]
            .spacing(8)
        } else {
            column![text("Unknown word").size(32)].spacing(8)
        };

        if session.revealed {
            let mut back_lines = Vec::new();
            for sense in &review_card.senses {
                back_lines.push(text(&sense.definition).size(18).into());
                if let Some(example) = &sense.example {
                    back_lines.push(text(example).size(14).into());
                }
            }
            let back = column(back_lines).spacing(8);
            let mut content = column![front, back].spacing(24);
            content = content.push(
                row![
                    button(text(previews[0].1.clone()).size(13))
                        .on_press(Message::Grade(previews[0].0)),
                    button(text(previews[1].1.clone()).size(13))
                        .on_press(Message::Grade(previews[1].0)),
                    button(text(previews[2].1.clone()).size(13))
                        .on_press(Message::Grade(previews[2].0)),
                    button(text(previews[3].1.clone()).size(13))
                        .on_press(Message::Grade(previews[3].0)),
                ]
                .spacing(8),
            );
            content = content.push(button("Undo").on_press(Message::UndoReview));
            container(content)
                .padding(32)
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb8(23, 31, 51).into()),
                    border: iced::Border {
                        radius: 12.0.into(),
                        width: 1.0,
                        color: iced::Color::from_rgb8(66, 71, 84),
                    },
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 14.0,
                    },
                    text_color: Some(iced::Color::from_rgb8(218, 226, 253)),
                    snap: false,
                })
                .into()
        } else {
            let content = column![
                front,
                button("Show answer").on_press(Message::RevealAnswer),
                text("Press Space to reveal").size(12),
            ]
            .spacing(16);
            container(content)
                .padding(32)
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb8(23, 31, 51).into()),
                    border: iced::Border {
                        radius: 12.0.into(),
                        width: 1.0,
                        color: iced::Color::from_rgb8(66, 71, 84),
                    },
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 14.0,
                    },
                    text_color: Some(iced::Color::from_rgb8(218, 226, 253)),
                    snap: false,
                })
                .into()
        }
    }
}
