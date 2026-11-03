#![allow(dead_code)]
use crate::theme;
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub fn card<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(18)
        .width(Length::Fill)
        .style(theme::surface_container(
            theme::BG_ELEVATED,
            theme::BORDER_SUBTLE,
            16.0,
        ))
        .into()
}

pub fn stat_card<'a, Message: 'a>(title: &'static str, value: String) -> Element<'a, Message> {
    container(
        column![
            text(title).size(13).color(theme::TEXT_SECONDARY),
            text(value).size(28).color(theme::TEXT_PRIMARY),
        ]
        .spacing(4),
    )
    .padding(18)
    .width(Length::Fixed(190.0))
    .style(theme::surface_container(
        theme::BG_SURFACE,
        theme::BORDER_SUBTLE,
        16.0,
    ))
    .into()
}

pub fn section_header<'a, Message: 'a>(title: &'static str) -> Element<'a, Message> {
    text(title).size(24).color(theme::TEXT_PRIMARY).into()
}

pub fn horizontal_spacer<'a, Message: 'a>() -> Element<'a, Message> {
    row![].width(Length::Fill).into()
}
