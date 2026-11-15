use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::text_editor;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide::*;
use lexic_core::model::{Id, Note};

impl App {
    pub fn notes_view(&self) -> Element<'_, Message> {
        let subjects = &self.note_subjects;
        let notes = &self.notes;

        // Filter notes by search
        let filtered_notes: Vec<&Note> = if self.note_search_input.is_empty() {
            notes.iter().collect()
        } else {
            let query = self.note_search_input.to_lowercase();
            notes
                .iter()
                .filter(|n| {
                    n.title.to_lowercase().contains(&query)
                        || n.content.to_lowercase().contains(&query)
                        || n.subject.to_lowercase().contains(&query)
                })
                .collect()
        };

        let header = row![
            column![
                text("Notes")
                    .size(20)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                text("Your humanities knowledge base")
                    .size(12)
                    .color(theme::TEXT_MUTED),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            row![
                button(
                    row![
                        network().size(13).color(theme::ON_PRIMARY_CONTAINER),
                        text("Graph")
                            .size(11)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::secondary_button())
                .on_press(Message::OpenNoteGraph),
                button(
                    row![
                        plus().size(13).color(theme::ON_PRIMARY_CONTAINER),
                        text("New Note")
                            .size(11)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::primary_button())
                .on_press(Message::NewNote),
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
            button(
                text("All")
                    .size(11)
                    .font(theme::font_code())
                    .color(theme::ON_PRIMARY_CONTAINER),
            )
            .padding([4, 10])
            .style(theme::primary_button())
            .on_press(Message::Navigate(Screen::Notes)),
        );
        for subject in subjects {
            subject_chips = subject_chips.push(
                button(
                    text(subject.as_str())
                        .size(11)
                        .font(theme::font_code())
                        .color(theme::TEXT_MUTED),
                )
                .padding([4, 10])
                .style(theme::secondary_button())
                .on_press(Message::Navigate(Screen::Notes)),
            );
        }

        let search_input = text_input("Search notes...", &self.note_search_input)
            .on_input(Message::NoteSearchChanged)
            .padding([8, 12])
            .width(Length::FillPortion(1))
            .style(theme::themed_input());

        let controls = row![
            subject_chips,
            Space::new().width(Length::Fill),
            search_input
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Notes list
        let notes_list: Element<'_, Message> = if filtered_notes.is_empty() {
            components::empty_state(
                "No notes yet",
                String::from("Create your first note to start building your knowledge base."),
            )
        } else {
            let mut list = column![].spacing(6);
            for note in filtered_notes {
                let selected = self.note_selected_id == Some(note.id);
                let tag_chips: Element<'_, Message> = if note.tags.is_empty() {
                    text("").size(1).into()
                } else {
                    let mut chips_row = row![].spacing(4);
                    for tag in &note.tags {
                        chips_row = chips_row.push(components::chip(
                            format!("#{tag}"),
                            theme::TEXT_SECONDARY,
                            theme::BG_CONTAINER_HIGH,
                        ));
                    }
                    chips_row.into()
                };
                list = list.push(
                    button(
                        row![
                            column![
                                text(&note.title)
                                    .size(13)
                                    .color(if selected {
                                        theme::PRIMARY
                                    } else {
                                        theme::TEXT_PRIMARY
                                    })
                                    .font(theme::font_code()),
                                row![
                                    text(&note.subject)
                                        .size(10)
                                        .color(theme::TEXT_MUTED)
                                        .font(theme::font_code()),
                                    Space::new().width(6),
                                    tag_chips,
                                ]
                                .spacing(2),
                            ]
                            .spacing(2),
                            Space::new().width(Length::Fill),
                            text(note.updated_at.format("%b %d").to_string())
                                .size(10)
                                .color(theme::TEXT_MUTED)
                                .font(theme::font_code()),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding([10, 14])
                    .style(theme::button_style(selected))
                    .on_press(Message::NoteSelected(note.id)),
                );
            }
            column![controls, list].spacing(10).into()
        };

        let content = column![header, notes_list].spacing(16);

        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn note_detail_view(&self) -> Element<'_, Message> {
        let is_new = self.active_note.is_none();

        let title_input = text_input("Note title", &self.note_editor_title)
            .on_input(Message::NoteTitleChanged)
            .padding([8, 12])
            .width(Length::Fill)
            .size(18)
            .style(theme::themed_input());

        let subject_input =
            text_input("Subject (e.g. History, English)", &self.note_editor_subject)
                .on_input(Message::NoteSubjectChanged)
                .padding([8, 12])
                .width(Length::FillPortion(1))
                .style(theme::themed_input());

        let tags_input = text_input("tags, comma-separated", &self.note_editor_tags)
            .on_input(Message::NoteTagsChanged)
            .padding([8, 12])
            .width(Length::FillPortion(1))
            .style(theme::themed_input());

        let editor = container(
            text_editor(&self.note_editor_content)
                .on_action(Message::NoteEditorAction)
                .height(Length::Fill)
                .style(theme::text_editor_style()),
        )
        .width(Length::Fill);

        let header = row![
            column![
                text(if is_new { "New Note" } else { "Edit Note" })
                    .size(20)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                text("Write in markdown. Your knowledge grows with every note.")
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
                .on_press(Message::CancelNoteEdit),
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
                .on_press(Message::SaveNote),
            ]
            .spacing(8),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let meta_row = row![subject_input, tags_input].spacing(10);

        let status_text = if let Some(status) = &self.note_editor_status {
            text(status)
                .size(11)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code())
        } else {
            text("").size(1)
        };

        // Links panels (owned data to satisfy borrow checker)
        let links_panel: Element<'_, Message> = if let Some(note) = &self.active_note {
            let note_id = note.id;
            let store = self.store.as_ref();
            let outgoing = store
                .and_then(|s| s.get_outgoing_links(note_id).ok())
                .unwrap_or_default();
            let incoming = store
                .and_then(|s| s.get_incoming_links(note_id).ok())
                .unwrap_or_default();

            // Clone owned data for button closures
            let outgoing_items: Vec<(String, Id)> = outgoing
                .into_iter()
                .map(|(_link, target)| (target.title.clone(), target.id))
                .collect();
            let incoming_items: Vec<(String, Id)> = incoming
                .into_iter()
                .map(|(_link, source)| (source.title.clone(), source.id))
                .collect();

            let outgoing_list: Element<'_, Message> = if outgoing_items.is_empty() {
                text("No outgoing links")
                    .size(11)
                    .color(theme::TEXT_MUTED)
                    .into()
            } else {
                let mut list: iced::widget::Column<'_, Message> = column![].spacing(4);
                for (title, id) in &outgoing_items {
                    let title = title.clone();
                    let id = *id;
                    list = list.push(
                        button(
                            row![
                                text(title)
                                    .size(12)
                                    .color(theme::TEXT_PRIMARY)
                                    .font(theme::font_code()),
                                Space::new().width(Length::Fill),
                                text("→")
                                    .size(12)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code()),
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                        )
                        .padding([6, 10])
                        .style(theme::ghost_button())
                        .on_press(Message::OpenNoteGraphCentered(id)),
                    );
                }
                container(list).into()
            };

            let incoming_list: Element<'_, Message> = if incoming_items.is_empty() {
                text("No incoming links")
                    .size(11)
                    .color(theme::TEXT_MUTED)
                    .into()
            } else {
                let mut list: iced::widget::Column<'_, Message> = column![].spacing(4);
                for (title, id) in &incoming_items {
                    let title = title.clone();
                    let id = *id;
                    list = list.push(
                        button(
                            row![
                                text(title)
                                    .size(12)
                                    .color(theme::TEXT_PRIMARY)
                                    .font(theme::font_code()),
                                Space::new().width(Length::Fill),
                                text("←")
                                    .size(12)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code()),
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                        )
                        .padding([6, 10])
                        .style(theme::ghost_button())
                        .on_press(Message::OpenNoteGraphCentered(id)),
                    );
                }
                container(list).into()
            };

            container(
                column![
                    text("Links")
                        .size(14)
                        .color(theme::TEXT_PRIMARY)
                        .font(theme::font_code()),
                    row![
                        container(
                            column![
                                text("Outgoing")
                                    .size(11)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code()),
                                outgoing_list,
                            ]
                            .spacing(6),
                        )
                        .padding(10)
                        .width(Length::FillPortion(1))
                        .style(theme::panel_flat(
                            theme::BG_CONTAINER,
                            theme::BORDER_SUBTLE,
                            8.0
                        )),
                        container(
                            column![
                                text("Incoming")
                                    .size(11)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code()),
                                incoming_list,
                            ]
                            .spacing(6),
                        )
                        .padding(10)
                        .width(Length::FillPortion(1))
                        .style(theme::panel_flat(
                            theme::BG_CONTAINER,
                            theme::BORDER_SUBTLE,
                            8.0
                        )),
                    ]
                    .spacing(10),
                    button(
                        row![
                            network().size(13).color(theme::ON_PRIMARY_CONTAINER),
                            text("View Graph")
                                .size(11)
                                .color(theme::ON_PRIMARY_CONTAINER)
                                .font(theme::font_code()),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 14])
                    .style(theme::secondary_button())
                    .on_press(Message::OpenNoteGraphCentered(note_id)),
                ]
                .spacing(10),
            )
            .padding(12)
            .style(theme::panel_flat(
                theme::BG_CONTAINER_LOW,
                theme::BORDER_SUBTLE,
                10.0,
            ))
            .into()
        } else {
            container(text("").size(1)).into()
        };

        let content = column![
            header,
            meta_row,
            title_input,
            status_text,
            editor,
            links_panel
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
