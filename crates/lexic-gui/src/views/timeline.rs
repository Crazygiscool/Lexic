use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide::*;

impl App {
    pub fn timeline_view(&self) -> Element<'_, Message> {
        let events = &self.timeline_events;
        let subjects = &self.timeline_subjects;

        let header = row![
            column![
                text("Timelines")
                    .size(20)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                text("Historical events and chronologies")
                    .size(12)
                    .color(theme::TEXT_MUTED),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            row![
                button(
                    row![
                        plus().size(13).color(theme::ON_PRIMARY_CONTAINER),
                        text("New Event")
                            .size(11)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::primary_button())
                .on_press(Message::NewTimelineEvent),
                button(
                    row![
                        external_link().size(13).color(theme::TEXT_MUTED),
                        text("View All")
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::ghost_button())
                .on_press(Message::LoadTimeline),
            ]
            .spacing(8),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        // Subject filter chips
        let mut subject_chips = row![
            text("Subject:")
                .size(11)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code()),
        ]
        .spacing(6)
        .align_y(Alignment::Center);
        subject_chips = subject_chips.push(
            button(text("All").size(11).font(theme::font_code()).color(
                if self.timeline_subject_filter.is_none() {
                    theme::ON_PRIMARY_CONTAINER
                } else {
                    theme::TEXT_MUTED
                },
            ))
            .padding([4, 10])
            .style(theme::button_style(self.timeline_subject_filter.is_none()))
            .on_press(Message::TimelineSubjectFilterChanged(None)),
        );
        for subject in subjects {
            let active = self.timeline_subject_filter.as_deref() == Some(subject.as_str());
            subject_chips = subject_chips.push(
                button(
                    text(subject.as_str())
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
                .on_press(Message::TimelineSubjectFilterChanged(Some(subject.clone()))),
            );
        }

        // Timeline list
        let events_list: Element<'_, Message> = if events.is_empty() {
            components::empty_state(
                "No timeline events yet",
                String::from("Create your first event to start building your historical timeline."),
            )
        } else {
            let mut list = column![].spacing(6);
            for event in events {
                let selected = self.timeline_selected_id == Some(event.id);
                let date_range = match (&event.start_date, &event.end_date) {
                    (start, Some(end)) => format!(
                        "{} – {}",
                        start.format("%b %d, %Y"),
                        end.format("%b %d, %Y")
                    ),
                    (start, None) => start.format("%b %d, %Y").to_string(),
                };
                let subject_color = match event.subject.as_str() {
                    "History" => theme::TERTIARY,
                    "English" => theme::PRIMARY,
                    "Philosophy" => theme::STRUCTURAL,
                    "Art" => theme::ERROR,
                    "Music" => theme::WARNING,
                    _ => theme::SECONDARY,
                };
                list = list.push(
                    button(
                        row![
                            column![
                                text(event.title.clone())
                                    .size(13)
                                    .color(if selected {
                                        theme::PRIMARY
                                    } else {
                                        theme::TEXT_PRIMARY
                                    })
                                    .font(theme::font_code()),
                                row![
                                    text(date_range.clone())
                                        .size(10)
                                        .color(theme::TEXT_MUTED)
                                        .font(theme::font_code()),
                                    Space::new().width(6),
                                    components::chip(
                                        event.subject.clone(),
                                        subject_color,
                                        theme::BG_CONTAINER_HIGH,
                                    ),
                                ]
                                .spacing(2),
                            ]
                            .spacing(2),
                            Space::new().width(Length::Fill),
                            if let Some(_note_id) = event.note_id {
                                text("📝").size(12).color(theme::TEXT_MUTED)
                            } else {
                                text("").size(12)
                            },
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding([10, 14])
                    .style(theme::button_style(selected))
                    .on_press(Message::TimelineSelected(event.id)),
                );
            }
            column![subject_chips, list].spacing(10).into()
        };

        let content = column![header, events_list].spacing(16);

        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn timeline_detail_view(&self) -> Element<'_, Message> {
        let is_new = self.active_timeline_event.is_none();

        let title_input = text_input("Event title", &self.timeline_editor_title)
            .on_input(Message::TimelineTitleChanged)
            .padding([8, 12])
            .width(Length::Fill)
            .size(18)
            .style(theme::themed_input());

        let description_input = text_input("Description", &self.timeline_editor_description)
            .on_input(Message::TimelineDescriptionChanged)
            .padding([8, 12])
            .width(Length::Fill)
            .style(theme::themed_input());

        let subject_input = text_input(
            "Subject (e.g. History, English)",
            &self.timeline_editor_subject,
        )
        .on_input(Message::TimelineSubjectChanged)
        .padding([8, 12])
        .width(Length::FillPortion(1))
        .style(theme::themed_input());

        let start_date_input =
            text_input("Start date (YYYY-MM-DD)", &self.timeline_editor_start_date)
                .on_input(Message::TimelineStartDateChanged)
                .padding([8, 12])
                .width(Length::FillPortion(1))
                .style(theme::themed_input());

        let end_date_input = text_input(
            "End date (YYYY-MM-DD, optional)",
            &self.timeline_editor_end_date,
        )
        .on_input(Message::TimelineEndDateChanged)
        .padding([8, 12])
        .width(Length::FillPortion(1))
        .style(theme::themed_input());

        let header = row![
            column![
                text(if is_new {
                    "New Timeline Event"
                } else {
                    "Edit Timeline Event"
                })
                .size(20)
                .color(theme::TEXT_PRIMARY)
                .font(theme::font_code()),
                text("Place events on your historical timeline.")
                    .size(12)
                    .color(theme::TEXT_MUTED),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            row![
                button(
                    row![
                        x().size(13).color(theme::TEXT_MUTED),
                        text("Cancel")
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::ghost_button())
                .on_press(Message::CancelTimelineEdit),
                button(
                    row![
                        save().size(13).color(theme::ON_PRIMARY_CONTAINER),
                        text("Save")
                            .size(11)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::primary_button())
                .on_press(Message::SaveTimelineEvent),
            ]
            .spacing(8),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let date_row = row![start_date_input, end_date_input].spacing(10);

        let status_text = if let Some(status) = &self.timeline_editor_status {
            text(status)
                .size(11)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code())
        } else {
            text("").size(1)
        };

        let content = column![
            header,
            subject_input,
            title_input,
            description_input,
            date_row,
            status_text,
        ]
        .spacing(10);

        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
