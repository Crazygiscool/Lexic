use crate::app::*;
use iced::Element;
use iced::widget::{button, column, container, row, text};

impl App {
    pub fn settings_view(&self) -> Element<'_, Message> {
        let backup_row = row![
            button("Export backup").on_press(Message::BeginExport),
            button("Import backup").on_press(Message::BeginImport),
        ]
        .spacing(8);

        let mut content = column![
            text("Settings").size(32),
            text(format!(
                "Include review history in exports: {}",
                self.include_review_history
            ))
            .size(16),
            button("Toggle review history").on_press(Message::ToggleIncludeReviewHistory),
            backup_row,
        ]
        .spacing(16);

        if let Some(status) = &self.settings_status {
            content = content.push(text(status).size(14));
        }

        container(content).padding(32).into()
    }
}
