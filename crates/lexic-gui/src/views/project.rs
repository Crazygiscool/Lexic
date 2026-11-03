use crate::app::*;

use crate::theme::{self, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Theme};
use iced::widget::text_editor;

impl App {
    pub fn project_view(&self) -> Element<'_, Message> {
        let path_input = text_input("Project path", &self.gset_project_path)
            .on_input(Message::GsetProjectPathChanged)
            .padding([10, 14])
            .width(Length::Fill)
            .style(|_theme, _status| iced::widget::text_input::Style {
                background: iced::Background::Color(Color::from_rgb8(30, 41, 59)),
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(71, 85, 105),
                },
                icon: Color::from_rgb8(148, 163, 184),
                placeholder: Color::from_rgb8(148, 163, 184),
                value: Color::from_rgb8(218, 226, 253),
                selection: Color::from_rgb8(51, 65, 85),
            });

        let target_input = text_input("Target language", &self.gset_target_language)
            .on_input(Message::GsetTargetLanguageChanged)
            .padding([10, 14])
            .width(Length::Fixed(180.0))
            .style(|_theme, _status| iced::widget::text_input::Style {
                background: iced::Background::Color(Color::from_rgb8(30, 41, 59)),
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(71, 85, 105),
                },
                icon: Color::from_rgb8(148, 163, 184),
                placeholder: Color::from_rgb8(148, 163, 184),
                value: Color::from_rgb8(218, 226, 253),
                selection: Color::from_rgb8(51, 65, 85),
            });

        let run_button = button(text("Transpile project").size(14).color(Color::WHITE))
            .padding([10, 16])
            .on_press(Message::RunGsetTranspile)
            .style(|_theme, status| {
                let background = match status {
                    iced::widget::button::Status::Hovered => Color::from_rgb8(37, 99, 235),
                    _ => Color::from_rgb8(59, 130, 246),
                };
                iced::widget::button::Style {
                    background: Some(background.into()),
                    text_color: Color::WHITE,
                    border: iced::Border {
                        radius: 8.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: theme::shadow(),
                    snap: false,
                }
            });

        let mut project_buttons = column![].spacing(4);
        for project in &self.gset_projects {
            let is_selected = self.gset_selected_project == Some(project.id);
            project_buttons = project_buttons.push(
                row![
                    button(text(&project.name).size(14).color(Color::from_rgb8(218, 226, 253)))
                        .padding([8, 12])
                        .on_press(Message::SelectGsetProject(project.id))
                        .style(move |_theme, status| iced::widget::button::Style {
                            background: Some(
                                if is_selected
                                    || matches!(status, iced::widget::button::Status::Hovered)
                                {
                                    Color::from_rgb8(51, 65, 85)
                                } else {
                                    Color::from_rgb8(30, 41, 59)
                                }
                                .into(),
                            ),
                            text_color: Color::from_rgb8(218, 226, 253),
                            border: iced::Border {
                                radius: 8.0.into(),
                                width: 1.0,
                                color: Color::from_rgb8(71, 85, 105),
                            },
                            shadow: theme::shadow(),
                            snap: false,
                        }),
                    text(&project.root_path)
                        .size(12)
                        .color(Color::from_rgb8(148, 163, 184)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }

        let saved_projects: Element<'_, Message> = if self.gset_projects.is_empty() {
            text("No saved projects yet. Run a transpilation to create one.")
                .size(14)
                .color(TEXT_MUTED)
                .into()
        } else {
            column![
                text("Saved projects").size(14).color(TEXT_SECONDARY),
                project_buttons,
            ]
            .spacing(4)
            .into()
        };

        let mut content = column![
            text("GSET Transpilation").size(28).color(TEXT_PRIMARY),
            saved_projects,
            row![path_input, target_input, run_button]
                .spacing(10)
                .align_y(Alignment::Center),
        ]
        .spacing(16);

        if let Some(status) = &self.gset_status {
            content = content.push(text(status).size(14).color(TEXT_SECONDARY));
        }

        if let Some(result) = &self.gset_result {
            let diagnostics: Element<'_, Message> = if result.diagnostics.is_empty() {
                text("No diagnostics reported.")
                    .size(14)
                    .color(TEXT_MUTED)
                    .into()
            } else {
                let lines: Vec<Element<'_, Message>> = result
                    .diagnostics
                    .iter()
                    .map(|item| text(&item.raw).size(12).color(TEXT_SECONDARY).into())
                    .collect();
                column(lines).spacing(4).into()
            };

            content = content.push(
                container(
                    column![
                        text(if result.success {
                            "Transpilation successful"
                        } else {
                            "Transpilation failed"
                        })
                        .size(16)
                        .color(if result.success {
                            Color::from_rgb8(16, 185, 129)
                        } else {
                            Color::from_rgb8(244, 63, 94)
                        }),
                        text("Diagnostics").size(14).color(TEXT_SECONDARY),
                        diagnostics,
                        text("Output").size(14).color(TEXT_SECONDARY),
                        scrollable(text(&result.stdout).size(12).color(TEXT_PRIMARY)),
                    ]
                    .spacing(8),
                )
                .padding(16)
                .width(Length::Fill)
                .style(|_theme| iced::widget::container::Style {
                    background: Some(Color::from_rgb8(30, 41, 59).into()),
                    border: iced::Border {
                        radius: 12.0.into(),
                        width: 1.0,
                        color: Color::from_rgb8(71, 85, 105),
                    },
                    shadow: theme::shadow(),
                    text_color: Some(TEXT_PRIMARY),
                    snap: false,
                }),
            );
        }

        let editor_path = text_input("File to edit", &self.gset_editor_path)
            .on_input(Message::GsetEditorPathChanged)
            .padding([10, 14])
            .width(Length::Fill);

        let editor = text_editor(&self.gset_editor_content)
            .on_action(Message::GsetEditorAction)
            .height(Length::Fixed(320.0))
            .width(1200.0);

        let editor_panel = container(
            column![
                text("Raw File Editor").size(18).color(TEXT_PRIMARY),
                row![editor_path]
                    .spacing(10)
                    .push(button("Load").on_press(Message::LoadGsetEditorFile))
                    .push(button("Save").on_press(Message::SaveGsetEditorFile)),
                editor,
                if let Some(status) = &self.gset_editor_status {
                    text(status).size(14).color(TEXT_SECONDARY)
                } else {
                    text("").size(1)
                },
            ]
            .spacing(8),
        )
        .padding(16)
        .width(Length::Fill)
        .style(|_theme: &Theme| iced::widget::container::Style {
            background: Some(Color::from_rgb8(30, 41, 59).into()),
            border: iced::Border {
                radius: 12.0.into(),
                width: 1.0,
                color: Color::from_rgb8(71, 85, 105),
            },
            shadow: theme::shadow(),
            text_color: Some(TEXT_PRIMARY),
            snap: false,
        });

        content = content.push(editor_panel);

        scrollable(container(content).padding(24).width(Length::Fill))
            .height(Length::Fill)
            .into()
    }
}
