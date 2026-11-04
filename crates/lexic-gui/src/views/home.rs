#![allow(clippy::collapsible_if)]
#![allow(clippy::single_match)]

use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::canvas;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Font, Length, Point, Rectangle};
use iced_fonts::lucide::*;
use lexic_core::model::Entry;

const STREAM_SLOT: f32 = 180.0;
const STREAM_HEIGHT: f32 = 64.0;

impl App {
    pub fn home_view(&self) -> Element<'_, Message> {
        let due_count: i64 = self.deck_counts.values().map(|(_, _, due)| *due).sum();
        let new_count: i64 = self.deck_counts.values().map(|(_, new, _)| *new).sum();
        let storage = self.storage_stats.clone().unwrap_or_default();
        let stats = self.stats_data.clone().unwrap_or_default();

        let due_total = due_count + new_count;

        let stream_panel = self.word_stream_panel();

        let tiles = column![
            row![
                components::stat_tile("Due now", due_count.to_string(), theme::ERROR),
                components::stat_tile("New cards", new_count.to_string(), theme::PRIMARY),
                components::stat_tile(
                    "Reviews (30d)",
                    stats.total_reviews.to_string(),
                    theme::TERTIARY
                ),
            ]
            .spacing(8),
            row![
                components::stat_tile(
                    "Retention",
                    format!("{:.0}%", stats.retention * 100.0),
                    theme::SECONDARY
                ),
                components::stat_tile("Streak", stats.current_streak.to_string(), theme::WARNING),
                components::stat_tile(
                    "Dictionary",
                    storage.entry_count.to_string(),
                    theme::STRUCTURAL
                ),
            ]
            .spacing(8),
        ]
        .spacing(8);

        // Due now list
        let mut due_rows = column![].spacing(6);
        if self.home_due.is_empty() {
            due_rows = due_rows.push(components::empty_state(
                "All caught up",
                String::from("No cards are due right now. Start a review to study new cards."),
            ));
        } else {
            for preview in &self.home_due {
                let due_label = due_label(preview, sched_time());
                let state_color = match preview.card.state {
                    lexic_core::model::CardState::New => theme::PRIMARY,
                    lexic_core::model::CardState::Learning => theme::WARNING,
                    lexic_core::model::CardState::Review => theme::TERTIARY,
                    lexic_core::model::CardState::Relearning => theme::ERROR,
                    lexic_core::model::CardState::Suspended => theme::TEXT_MUTED,
                };
                let state_label = format!("{:?}", preview.card.state).to_lowercase();
                due_rows = due_rows.push(
                    container(
                        row![
                            text(&preview.word)
                                .size(14)
                                .color(theme::TEXT_PRIMARY)
                                .font(theme::font_code()),
                            Space::new().width(Length::Fill),
                            text(&preview.deck_name)
                                .size(11)
                                .color(theme::TEXT_MUTED)
                                .font(theme::font_code()),
                            components::chip(
                                format!("{state_label} · {due_label}"),
                                state_color,
                                theme::BG_CONTAINER_HIGH
                            ),
                            button(text("Open").size(11).font(theme::font_code()))
                                .padding([4, 10])
                                .style(theme::ghost_button())
                                .on_press(Message::OpenWord(preview.card.entry_id)),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 12])
                    .style(theme::panel_flat(
                        theme::BG_CONTAINER,
                        theme::BORDER_SUBTLE,
                        8.0,
                    )),
                );
            }
        }

        let due_panel = container(
            column![
                row![
                    text("Due now")
                        .size(14)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    Space::new().width(Length::Fill),
                    components::chip(
                        format!("{due_total} pending"),
                        theme::TEXT_MUTED,
                        theme::BG_CONTAINER_HIGH
                    ),
                    button(
                        row![
                            play().size(13).color(theme::ON_PRIMARY_CONTAINER),
                            text("Review all")
                                .size(11)
                                .color(theme::ON_PRIMARY_CONTAINER)
                                .font(theme::font_code())
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .padding([6, 12])
                    .style(theme::primary_button())
                    .on_press(Message::StartReview),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                due_rows,
            ]
            .spacing(8),
        )
        .padding(12)
        .width(Length::FillPortion(2))
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            10.0,
        ));

        // Activity panel
        let activity_panel = container(
            column![
                text("Activity · 7 days")
                    .size(14)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                bar_chart(stats.reviews_by_day.clone()),
                row![
                    text(format!("Retention {:.0}%", stats.retention * 100.0))
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    Space::new().width(Length::Fill),
                    text(format!("Streak {}", stats.current_streak))
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                ],
                if stats.total_reviews == 0 {
                    text("Start reviewing to build your streak.")
                        .size(12)
                        .color(theme::TEXT_MUTED)
                } else {
                    text(format!(
                        "{} reviews in the last 30 days",
                        stats.total_reviews
                    ))
                    .size(11)
                    .color(theme::TEXT_SECONDARY)
                    .font(theme::font_code())
                },
            ]
            .spacing(8),
        )
        .padding(12)
        .width(Length::FillPortion(1))
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            10.0,
        ));

        let lower = row![due_panel, activity_panel].spacing(10);

        let content = column![
            row![
                column![
                    text("Home")
                        .size(20)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    text("Your study snapshot and a window into the dictionary.")
                        .size(12)
                        .color(theme::TEXT_MUTED),
                ]
                .spacing(4),
                Space::new().width(Length::Fill),
                components::chip(
                    storage.sqlite_version.clone(),
                    theme::TERTIARY,
                    theme::BG_CONTAINER_HIGH
                ),
            ]
            .align_y(Alignment::Center),
            stream_panel,
            tiles,
            lower,
        ]
        .spacing(12);

        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn word_stream_panel(&self) -> Element<'_, Message> {
        let words = self.home_words.as_slice();
        let canvas_widget: Element<'_, Message> = if words.is_empty() {
            container(
                text("No dictionary entries yet — look one up in Grammar Dict.")
                    .size(12)
                    .color(theme::TEXT_MUTED),
            )
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fixed(STREAM_HEIGHT))
            .style(theme::panel_flat(
                theme::BG_CONTAINER,
                theme::BORDER_SUBTLE,
                8.0,
            ))
            .into()
        } else {
            iced::widget::canvas(WordStream {
                words,
                offset: self.home_ticker_offset,
            })
            .width(Length::Fill)
            .height(Length::Fixed(STREAM_HEIGHT))
            .into()
        };
        container(
            column![
                row![
                    text("WORD STREAM")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    Space::new().width(Length::Fill),
                    button(text("Shuffle").size(11).font(theme::font_code()))
                        .padding([4, 10])
                        .style(theme::ghost_button())
                        .on_press(Message::ShuffleHomeWords),
                ]
                .align_y(Alignment::Center),
                container(canvas_widget)
                    .width(Length::Fill)
                    .height(Length::Fixed(STREAM_HEIGHT))
                    .style(theme::panel_flat(
                        theme::BG_CONTAINER,
                        theme::BORDER_SUBTLE,
                        8.0
                    )),
            ]
            .spacing(8),
        )
        .padding(12)
        .width(Length::Fill)
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            10.0,
        ))
        .into()
    }
}

struct WordStream<'a> {
    words: &'a [Entry],
    offset: f32,
}

impl canvas::Program<Message> for WordStream<'_> {
    type State = ();
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let mut frame = iced::widget::canvas::Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::TRANSPARENT);
        let slot = STREAM_SLOT;
        let total = self.words.len() as f32 * slot;
        let visible = (bounds.width / slot).ceil() as usize + 2;
        let start = (self.offset / slot).floor() as usize;
        for k in 0..visible {
            let idx = (start + k) % self.words.len();
            let entry = &self.words[idx];
            let x = (start + k) as f32 * slot - self.offset;
            if x + slot < 0.0 || x > bounds.width {
                continue;
            }
            let color = match idx % 4 {
                0 => theme::TEXT_PRIMARY,
                1 => theme::PRIMARY,
                2 => theme::SECONDARY,
                _ => theme::TERTIARY,
            };
            let word = truncate(&entry.word, 18);
            frame.fill_text(canvas::Text {
                content: word,
                position: Point::new(x + 14.0, STREAM_HEIGHT * 0.30),
                color,
                size: iced::Pixels(16.0),
                font: Font::with_name("Inter"),
                align_x: iced::widget::text::Alignment::Left,
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
            let sub = entry
                .phonetic
                .clone()
                .unwrap_or_else(|| entry.source.clone());
            frame.fill_text(canvas::Text {
                content: truncate(&sub, 24),
                position: Point::new(x + 14.0, STREAM_HEIGHT * 0.66),
                color: theme::TEXT_MUTED,
                size: iced::Pixels(10.0),
                font: Font::with_name("JetBrains Mono"),
                align_x: iced::widget::text::Alignment::Left,
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }
        let _ = total;
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) =
            event
        {
            if let Some(position) = cursor.position_in(bounds) {
                let slot = STREAM_SLOT;
                let index = ((position.x + self.offset) / slot).floor() as usize % self.words.len();
                if !self.words.is_empty() {
                    return Some(canvas::Action::publish(Message::OpenWord(
                        self.words[index].id,
                    )));
                }
            }
        }
        None
    }
}

fn due_label(preview: &lexic_store::DueCardPreview, now: chrono::DateTime<chrono::Utc>) -> String {
    if preview.card.state == lexic_core::model::CardState::New {
        return String::from("new");
    }
    match preview.card.due_at {
        Some(due) if due <= now => String::from("now"),
        Some(due) => {
            let minutes = (due - now).num_minutes();
            if minutes < 60 {
                format!("{}m", minutes.max(1))
            } else if minutes < 60 * 24 {
                format!("{}h", minutes / 60)
            } else {
                format!("{}d", (minutes + 60 * 24 - 1) / (60 * 24))
            }
        }
        None => String::from("now"),
    }
}

fn sched_time() -> chrono::DateTime<chrono::Utc> {
    lexic_core::time::now_utc()
}

fn bar_chart(
    reviews_by_day: std::collections::BTreeMap<String, DailyStat>,
) -> Element<'static, Message> {
    use chrono::{Duration, Local};
    let today = Local::now().date_naive();
    let mut days: Vec<(String, u16, i64)> = Vec::with_capacity(7);
    let mut max_count: i64 = 0;
    for offset in (0..7).rev() {
        let day = today - Duration::days(offset);
        let key = day.format("%Y-%m-%d").to_string();
        let count = reviews_by_day
            .get(&key)
            .map(|d| d.count as i64)
            .unwrap_or(0);
        if count > max_count {
            max_count = count;
        }
        days.push((day.format("%a").to_string(), count as u16, count));
    }
    let max = (max_count.max(1)) as f32;
    let mut bars = row![]
        .spacing(6)
        .height(Length::Fixed(120.0))
        .align_y(Alignment::Center);
    for (label, _count_u16, count) in &days {
        let height = ((*count as f32) / max * 90.0).max(if *count > 0 { 6.0 } else { 3.0 });
        bars = bars.push(
            column![
                text(count.to_string())
                    .size(9)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code()),
                Space::new().height(Length::Fill),
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(height))
                    .style(theme::panel_flat(
                        theme::PRIMARY_CONTAINER,
                        theme::PRIMARY_CONTAINER,
                        4.0
                    )),
                text(label.clone())
                    .size(9)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code()),
            ]
            .spacing(3)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        );
    }
    container(bars)
        .padding([6, 0])
        .width(Length::Fill)
        .style(theme::panel_flat(
            theme::BG_CONTAINER,
            theme::BORDER_SUBTLE,
            6.0,
        ))
        .into()
}

fn truncate(value: &str, max: usize) -> String {
    let mut chars: Vec<char> = value.chars().collect();
    if chars.len() <= max {
        value.to_string()
    } else {
        chars.truncate(max);
        format!("{}…", chars.into_iter().collect::<String>())
    }
}
