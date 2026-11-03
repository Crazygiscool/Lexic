use crate::app::*;
use iced::widget::{button, column, container, row, slider, text};
use iced::{Color, Element, Length};

impl App {
    pub fn stats_view(&self) -> Element<'_, Message> {
        let Some(data) = &self.stats_data else {
            return container(
                column![
                    text("Stats").size(32),
                    button("Load stats").on_press(Message::LoadStats),
                ]
                .spacing(12),
            )
            .padding(32)
            .into();
        };

        let retention_text = format!("Retention: {:.0}%", data.retention * 100.0);
        let duration_seconds = data.total_duration_ms / 1000;
        let time_text = format!(
            "Study time: {}m {}s · Reviews: {}",
            duration_seconds / 60,
            duration_seconds % 60,
            data.total_reviews
        );
        let streak_text = format!(
            "Current streak: {} days · Best streak: {} days",
            data.current_streak, data.best_streak
        );

        let range_slider = slider(30.0..=365.0, self.stats_range as f32, |value| {
            Message::SetStatsRange(value as u32)
        })
        .step(5.0_f32)
        .width(Length::Fill);

        let max_forecast = data
            .due_forecast
            .iter()
            .map(|entry| entry.1)
            .max()
            .unwrap_or(1)
            .max(1);
        let mut forecast_bars = row![].spacing(2);
        for (_day, count) in &data.due_forecast {
            let height = 4.0 + 96.0 * (*count as f32 / max_forecast as f32);
            forecast_bars = forecast_bars.push(
                container(text(count.to_string()).size(8))
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(height))
                    .style(move |_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb8(
                            112, 112, 112,
                        ))),
                        border: iced::Border::default(),
                        shadow: iced::Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 14.0,
                        },
                        snap: false,
                        text_color: None,
                    }),
            );
        }

        let max_heat = data
            .reviews_by_day
            .values()
            .map(|entry| entry.count)
            .max()
            .unwrap_or(1)
            .max(1);
        let mut heatmap = column![].spacing(2);
        let days: Vec<&String> = data.reviews_by_day.keys().collect();
        for chunk in days.chunks(14) {
            let mut row_view = row![].spacing(2);
            for day in chunk {
                let count = data
                    .reviews_by_day
                    .get(*day)
                    .map(|entry| entry.count)
                    .unwrap_or(0);
                let intensity = count as f32 / max_heat as f32;
                let channel = (232.0 - 180.0 * intensity) as u8;
                row_view = row_view.push(
                    container(
                        text(if count == 0 {
                            String::new()
                        } else {
                            count.to_string()
                        })
                        .size(8),
                    )
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .style(move |_theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb8(
                            channel, channel, channel,
                        ))),
                        border: iced::Border::default(),
                        snap: false,
                        shadow: iced::Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 14.0,
                        },
                        text_color: Some(iced::Color::from_rgb8(245, 245, 245)),
                    }),
                );
            }
            heatmap = heatmap.push(row_view);
        }

        let content = column![
            text("Stats").size(32),
            range_slider,
            text(retention_text).size(18),
            text(time_text).size(18),
            text(streak_text).size(18),
            text("Due forecast (30 days)").size(18),
            container(forecast_bars).padding(8).width(Length::Fill),
            text("Review heatmap").size(18),
            container(heatmap).padding(8).width(Length::Fill),
        ]
        .spacing(16);

        container(content)
            .padding(32)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Color::from_rgb8(30, 41, 59).into()),
                border: iced::Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: iced::Color::from_rgb8(71, 85, 105),
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
