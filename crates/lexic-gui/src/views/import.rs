use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide::*;
use lexic_core::model::ImportStatus;

impl App {
    pub fn import_view(&self) -> Element<'_, Message> {
        let pending = &self.pending_imports;

        let header = row![
            column![
                text("Import Browser History")
                    .size(20)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                text("Bring web pages into your knowledge base")
                    .size(12)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_serif()),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            button(
                row![
                    refresh_ccw().size(13).color(theme::ON_PRIMARY_CONTAINER),
                    text("Refresh")
                        .size(11)
                        .color(theme::ON_PRIMARY_CONTAINER)
                        .font(theme::font_code()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([8, 14])
            .style(theme::secondary_button())
            .on_press(Message::ImportBrowserHistory),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        // Configuration card
        let config_card = container(
            column![
                text("Import Settings")
                    .size(14)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                row![
                    text("Browser")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text_input("Browser", &self.import_browser)
                        .on_input(Message::ImportBrowserChanged)
                        .padding([6, 10])
                        .width(Length::FillPortion(1))
                        .style(theme::themed_input()),
                ]
                .spacing(8),
                row![
                    text("LLM Provider")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text_input("Ollama", &self.import_llm_provider)
                        .on_input(Message::ImportLLMProviderChanged)
                        .padding([6, 10])
                        .width(Length::FillPortion(1))
                        .style(theme::themed_input()),
                ]
                .spacing(8),
                row![
                    text("Endpoint")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text_input("http://localhost:11434", &self.import_llm_endpoint)
                        .on_input(Message::ImportLLMEndpointChanged)
                        .padding([6, 10])
                        .width(Length::FillPortion(2))
                        .style(theme::themed_input()),
                ]
                .spacing(8),
                row![
                    text("Model")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text_input("llama3.2", &self.import_llm_model)
                        .on_input(Message::ImportLLMModelChanged)
                        .padding([6, 10])
                        .width(Length::FillPortion(1))
                        .style(theme::themed_input()),
                ]
                .spacing(8),
                row![
                    text("API Key")
                        .size(11)
                        .color(theme::TEXT_MUTED)
                        .font(theme::font_code()),
                    text_input("API key", &self.import_llm_api_key)
                        .on_input(Message::ImportLLMApiKeyChanged)
                        .padding([6, 10])
                        .width(Length::FillPortion(2))
                        .style(theme::themed_input()),
                ]
                .spacing(8),
                row![
                    button(
                        row![
                            download().size(13).color(theme::ON_PRIMARY_CONTAINER),
                            text("Import History")
                                .size(11)
                                .color(theme::ON_PRIMARY_CONTAINER)
                                .font(theme::font_code()),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 14])
                    .style(theme::primary_button())
                    .on_press(Message::ImportBrowserHistory),
                ]
                .spacing(10),
            ]
            .spacing(10),
        )
        .padding(14)
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            10.0,
        ));

        // Status
        let status: Element<'_, Message> = if let Some(status) = &self.import_status {
            text(status)
                .size(11)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code())
                .into()
        } else {
            text("").size(1).into()
        };

        // Imports list
        let imports_list: Element<'_, Message> = if pending.is_empty() {
            components::empty_state(
                "No pending imports",
                String::from("Import browser history to see pending entries here."),
            )
        } else {
            let mut list = column![].spacing(6);
            for imp in pending {
                let status_color = match imp.status {
                    ImportStatus::Pending => theme::WARNING,
                    ImportStatus::Approved => theme::TERTIARY,
                    ImportStatus::Rejected => theme::ERROR,
                };
                list = list.push(
                    container(
                        column![
                            row![
                                text(&imp.url)
                                    .size(13)
                                    .color(theme::TEXT_PRIMARY)
                                    .font(theme::font_code()),
                                Space::new().width(Length::Fill),
                                text(&imp.suggested_subject)
                                    .size(11)
                                    .color(theme::TEXT_SECONDARY)
                                    .font(theme::font_code()),
                                components::chip(
                                    match imp.status {
                                        ImportStatus::Pending => "pending".to_string(),
                                        ImportStatus::Approved => "approved".to_string(),
                                        ImportStatus::Rejected => "rejected".to_string(),
                                    },
                                    status_color,
                                    theme::BG_CONTAINER_HIGH,
                                ),
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                            if let Some(title) = &imp.title {
                                text(title)
                                    .size(11)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code())
                            } else {
                                text("").size(1)
                            },
                            row![
                                text("Tags")
                                    .size(10)
                                    .color(theme::TEXT_MUTED)
                                    .font(theme::font_code()),
                                text_input("tags", &imp.suggested_tags.join(", "))
                                    .on_input(move |v| Message::ImportTagsChanged(imp.id, v))
                                    .padding([6, 10])
                                    .width(Length::FillPortion(1))
                                    .style(theme::themed_input()),
                                button(
                                    row![
                                        check().size(13).color(theme::ON_PRIMARY_CONTAINER),
                                        text("Approve")
                                            .size(11)
                                            .color(theme::ON_PRIMARY_CONTAINER)
                                            .font(theme::font_code()),
                                    ]
                                    .spacing(6)
                                    .align_y(Alignment::Center),
                                )
                                .padding([6, 12])
                                .style(theme::primary_button())
                                .on_press(Message::ApproveImport(imp.id)),
                                button(
                                    row![
                                        x().size(13).color(theme::TEXT_MUTED),
                                        text("Reject")
                                            .size(11)
                                            .color(theme::TEXT_MUTED)
                                            .font(theme::font_code()),
                                    ]
                                    .spacing(6)
                                    .align_y(Alignment::Center),
                                )
                                .padding([6, 12])
                                .style(theme::ghost_button())
                                .on_press(Message::RejectImport(imp.id)),
                                button(
                                    row![
                                        trash().size(13).color(theme::TEXT_MUTED),
                                        text("Delete")
                                            .size(11)
                                            .color(theme::TEXT_MUTED)
                                            .font(theme::font_code()),
                                    ]
                                    .spacing(6)
                                    .align_y(Alignment::Center),
                                )
                                .padding([6, 12])
                                .style(theme::ghost_button())
                                .on_press(Message::DeleteImport(imp.id)),
                            ]
                            .spacing(6),
                        ]
                        .spacing(8),
                    )
                    .padding(12)
                    .style(theme::panel_flat(
                        theme::BG_CONTAINER,
                        theme::BORDER_SUBTLE,
                        8.0,
                    )),
                );
            }
            column![
                text("Pending Imports")
                    .size(14)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                list
            ]
            .spacing(8)
            .into()
        };

        let content = column![header, config_card, status, imports_list].spacing(12);

        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
