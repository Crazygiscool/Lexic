use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::text_editor;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use iced_fonts::lucide::*;

impl App {
    pub fn project_view(&self) -> Element<'_, Message> {
        let project_input =
            text_input("Project path or file to transpile", &self.gset_project_path)
                .on_input(Message::GsetProjectPathChanged)
                .padding([8, 12])
                .width(Length::Fill)
                .style(theme::themed_input());

        let source_input = text_input("from (optional)", &self.gset_source_language)
            .on_input(Message::GsetSourceLanguageChanged)
            .padding([8, 12])
            .width(Length::Fixed(150.0))
            .style(theme::themed_input());

        let target_input = text_input("to", &self.gset_target_language)
            .on_input(Message::GsetTargetLanguageChanged)
            .padding([8, 12])
            .width(Length::Fixed(120.0))
            .style(theme::themed_input());

        let output_input = text_input("output path (optional)", &self.gset_output_path)
            .on_input(Message::GsetOutputPathChanged)
            .padding([8, 12])
            .width(Length::Fixed(260.0))
            .style(theme::themed_input());

        let verbose_button = button(
            text(if self.gset_verbose {
                "verbose: on"
            } else {
                "verbose: off"
            })
            .size(11)
            .font(theme::font_code())
            .color(if self.gset_verbose {
                theme::ON_PRIMARY_CONTAINER
            } else {
                theme::TEXT_MUTED
            }),
        )
        .padding([8, 12])
        .style(theme::button_style(self.gset_verbose))
        .on_press(Message::GsetVerboseToggled);

        let experimental_button = button(
            text(if self.gset_experimental_api_map {
                "experimental api map: on"
            } else {
                "experimental api map: off"
            })
            .size(11)
            .font(theme::font_code())
            .color(if self.gset_experimental_api_map {
                theme::ON_PRIMARY_CONTAINER
            } else {
                theme::TEXT_MUTED
            }),
        )
        .padding([8, 12])
        .style(theme::button_style(self.gset_experimental_api_map))
        .on_press(Message::GsetExperimentalToggled);

        let run_button = button(
            row![
                play().size(14).color(theme::ON_PRIMARY_CONTAINER),
                text("Run Transpiler")
                    .size(12)
                    .color(theme::ON_PRIMARY_CONTAINER)
                    .font(theme::font_code()),
                components::kbd("F5"),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        )
        .padding([8, 14])
        .style(theme::primary_button())
        .on_press(Message::RunGsetTranspile);

        let status_line = row![
            components::status_dot(
                if self.gset_status.as_deref().unwrap_or("").contains("failed") {
                    theme::ERROR
                } else {
                    theme::TERTIARY
                }
            ),
            text(
                self.gset_status
                    .clone()
                    .unwrap_or_else(|| String::from("GSET ready"))
            )
            .size(11)
            .color(theme::TEXT_SECONDARY)
            .font(theme::font_code()),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        let toolbar = container(
            column![
                row![project_input].spacing(8),
                row![
                    source_input,
                    target_input,
                    output_input,
                    verbose_button,
                    experimental_button,
                    run_button,
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![status_line, Space::new().width(Length::Fill)]
                    .align_y(iced::Alignment::Center),
            ]
            .spacing(8),
        )
        .padding([10, 12])
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            8.0,
        ));

        // Saved projects row
        let mut projects_row = row![
            text("Projects:")
                .size(11)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code())
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);
        for project in &self.gset_projects {
            let selected = self.gset_selected_project == Some(project.id);
            projects_row =
                projects_row.push(
                    button(text(&project.name).size(11).font(theme::font_code()).color(
                        if selected {
                            theme::ON_PRIMARY_CONTAINER
                        } else {
                            theme::TEXT_SECONDARY
                        },
                    ))
                    .padding([4, 10])
                    .style(theme::button_style(selected))
                    .on_press(Message::SelectGsetProject(project.id)),
                );
        }
        if self.gset_projects.is_empty() {
            projects_row = projects_row.push(text("none yet").size(11).color(theme::TEXT_MUTED));
        }
        let projects_bar = container(projects_row)
            .padding([6, 12])
            .style(theme::panel_flat(
                theme::BG_CONTAINER_LOWEST,
                theme::BORDER_SUBTLE,
                8.0,
            ));

        // Source editor with header
        let source_pane = container(
            column![
                container(
                    row![
                        text("SOURCE EDITOR")
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code()),
                        Space::new().width(Length::Fill),
                        text(format!(
                            "{} chars",
                            self.gset_editor_content.text().chars().count()
                        ))
                        .size(10)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .padding([6, 12])
                .style(theme::panel_flat(
                    theme::BG_CONTAINER_LOW,
                    theme::BORDER_SUBTLE,
                    0.0
                )),
                text_editor(&self.gset_editor_content)
                    .on_action(Message::GsetEditorAction)
                    .height(Length::Fill)
                    .style(theme::text_editor_style()),
                container(
                    row![
                        text_input("File path", &self.gset_editor_path)
                            .on_input(Message::GsetEditorPathChanged)
                            .padding([6, 10])
                            .width(Length::Fill)
                            .style(theme::themed_input()),
                        button(text("Load").size(11).font(theme::font_code()))
                            .padding([6, 10])
                            .style(theme::secondary_button())
                            .on_press(Message::LoadGsetEditorFile),
                        button(text("Save").size(11).font(theme::font_code()))
                            .padding([6, 10])
                            .style(theme::primary_button())
                            .on_press(Message::SaveGsetEditorFile),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
                )
                .padding([6, 12])
                .style(theme::panel_flat(
                    theme::BG_CONTAINER_LOW,
                    theme::BORDER_SUBTLE,
                    0.0
                )),
                {
                    let status_element: Element<'_, Message> =
                        if let Some(status) = &self.gset_editor_status {
                            container(
                                text(status)
                                    .size(11)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code()),
                            )
                            .padding([4, 12])
                            .into()
                        } else {
                            container(Space::new().height(0)).into()
                        };
                    status_element
                },
            ]
            .spacing(0),
        )
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(theme::panel_flat(
            theme::BG_CONTAINER,
            theme::BORDER_SUBTLE,
            8.0,
        ));

        // Output pane
        let mut output_col = column![
            container(
                row![
                    text("TRANSPILER OUTPUT")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    Space::new().width(Length::Fill),
                ]
                .align_y(iced::Alignment::Center),
            )
            .padding([6, 12])
            .style(theme::panel_flat(
                theme::BG_CONTAINER_LOW,
                theme::BORDER_SUBTLE,
                0.0
            )),
        ]
        .spacing(0);

        if let Some(result) = &self.gset_result {
            output_col = output_col.push(
                container(
                    text(if result.stdout.is_empty() {
                        "<no output>"
                    } else {
                        &result.stdout
                    })
                    .size(12)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                )
                .padding(12)
                .width(Length::Fill)
                .height(Length::FillPortion(2)),
            );
            // diagnostics drawer
            let diag_header = container(
                row![
                    text(format!("DIAGNOSTICS ({})", result.diagnostics.len()))
                        .size(11)
                        .color(if result.success {
                            theme::TERTIARY
                        } else {
                            theme::ERROR
                        })
                        .font(theme::font_code()),
                    Space::new().width(Length::Fill),
                    text(format!(
                        "exit {}",
                        result
                            .exit_code
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "?".into())
                    ))
                    .size(10)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_code()),
                ]
                .align_y(iced::Alignment::Center),
            )
            .padding([6, 12])
            .style(theme::panel_flat(
                theme::BG_CONTAINER_LOW,
                theme::BORDER_SUBTLE,
                0.0,
            ));
            output_col = output_col.push(diag_header);
            let mut diag_list = column![].spacing(4).padding(8);
            if result.diagnostics.is_empty() {
                diag_list =
                    diag_list.push(text("No diagnostics").size(11).color(theme::TEXT_MUTED));
            } else {
                for diagnostic in &result.diagnostics {
                    diag_list = diag_list.push(
                        container(
                            text(&diagnostic.message)
                                .size(11)
                                .color(theme::ERROR)
                                .font(theme::font_code()),
                        )
                        .padding([4, 8])
                        .style(theme::panel_flat(
                            theme::ERROR_CONTAINER,
                            theme::ERROR,
                            6.0,
                        )),
                    );
                }
            }
            output_col = output_col.push(scrollable(diag_list).height(Length::FillPortion(1)));
        } else {
            output_col = output_col.push(
                container(
                    text("Run the transpiler to see generated output and diagnostics.")
                        .size(12)
                        .color(theme::TEXT_MUTED),
                )
                .padding(12)
                .width(Length::Fill)
                .height(Length::Fill),
            );
        }

        let output_pane = container(output_col)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(theme::panel_flat(
                theme::BG_CONTAINER,
                theme::BORDER_SUBTLE,
                8.0,
            ));

        let panes = row![source_pane, output_pane]
            .spacing(10)
            .height(Length::Fill);

        container(
            column![toolbar, projects_bar, panes]
                .spacing(10)
                .height(Length::Fill),
        )
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
