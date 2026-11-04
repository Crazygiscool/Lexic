#![allow(dead_code)]
use crate::theme;
use iced::widget::{
    button, column, container, progress_bar as widget_progress_bar, row, space, text,
};
use iced::{Color, Element, Length};

pub fn card<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(18)
        .width(Length::Fill)
        .style(theme::surface_container(
            theme::BG_CONTAINER,
            theme::BORDER_SUBTLE,
            12.0,
        ))
        .into()
}

pub fn panel<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    padding: f32,
    radius: f32,
) -> Element<'a, Message> {
    container(content)
        .padding(padding)
        .width(Length::Fill)
        .style(theme::panel_flat(
            theme::BG_CONTAINER_LOW,
            theme::BORDER_SUBTLE,
            radius,
        ))
        .into()
}

pub fn stat_tile<'a, Message: 'a>(
    title: &'static str,
    value: String,
    accent: Color,
) -> Element<'a, Message> {
    container(
        column![
            text(title).size(11).color(theme::TEXT_MUTED),
            text(value).size(26).color(accent).font(theme::font_code()),
        ]
        .spacing(6),
    )
    .padding(14)
    .width(Length::Fill)
    .style(theme::surface_container(
        theme::BG_CONTAINER_LOW,
        theme::BORDER_SUBTLE,
        10.0,
    ))
    .into()
}

pub fn chip<'a, Message: 'a>(
    label: String,
    color: Color,
    background: Color,
) -> Element<'a, Message> {
    container(text(label).size(11).color(color).font(theme::font_code()))
        .padding([2, 8])
        .style(theme::panel_flat(background, color, 999.0))
        .into()
}

pub fn kbd<'a, Message: 'a>(label: &'static str) -> Element<'a, Message> {
    container(
        text(label)
            .size(10)
            .color(theme::TEXT_MUTED)
            .font(theme::font_code()),
    )
    .padding([1, 5])
    .style(theme::panel_flat(
        theme::BG_CONTAINER_HIGHEST,
        theme::BORDER_SUBTLE,
        4.0,
    ))
    .into()
}

pub fn section_header<'a, Message: 'a>(title: &'static str) -> Element<'a, Message> {
    text(title).size(20).color(theme::TEXT_PRIMARY).into()
}

pub fn section_label<'a, Message: 'a>(title: &'static str) -> Element<'a, Message> {
    text(title)
        .size(11)
        .color(theme::TEXT_MUTED)
        .font(theme::font_code())
        .into()
}

pub fn empty_state<'a, Message: 'a>(title: &'static str, detail: String) -> Element<'a, Message> {
    container(
        column![
            text(title).size(16).color(theme::TEXT_PRIMARY),
            text(detail).size(13).color(theme::TEXT_MUTED),
        ]
        .spacing(6)
        .align_x(iced::Alignment::Center),
    )
    .padding(24)
    .width(Length::Fill)
    .align_x(iced::Alignment::Center)
    .style(theme::panel_flat(
        theme::BG_CONTAINER_LOW,
        theme::BORDER_SUBTLE,
        12.0,
    ))
    .into()
}

pub fn progress_bar<'a, Message: 'a>(value: f32, maximum: f32) -> Element<'a, Message> {
    widget_progress_bar(0.0..=maximum, value)
        .girth(6)
        .style(|_theme| iced::widget::progress_bar::Style {
            background: iced::Background::Color(theme::BG_CONTAINER_HIGHEST),
            bar: iced::Background::Color(theme::PRIMARY_CONTAINER),
            border: iced::Border::default(),
        })
        .into()
}

pub fn status_dot<'a, Message: 'a>(color: Color) -> Element<'a, Message> {
    container(space::horizontal())
        .width(8)
        .height(8)
        .style(theme::panel_flat(color, color, 999.0))
        .into()
}

pub fn primary_button<'a, Message: 'a + Clone + 'static>(
    label: impl Into<String>,
    msg: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text(label.into()).size(12).font(theme::font_code()))
        .padding([6, 12])
        .style(theme::primary_button());
    if let Some(message) = msg {
        btn = btn.on_press(message);
    }
    btn.into()
}

pub fn secondary_button<'a, Message: 'a + Clone + 'static>(
    label: impl Into<String>,
    msg: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text(label.into()).size(12).font(theme::font_code()))
        .padding([6, 12])
        .style(theme::secondary_button());
    if let Some(message) = msg {
        btn = btn.on_press(message);
    }
    btn.into()
}

pub fn ghost_button<'a, Message: 'a + Clone + 'static>(
    label: impl Into<String>,
    msg: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text(label.into()).size(12).font(theme::font_code()))
        .padding([6, 12])
        .style(theme::ghost_button());
    if let Some(message) = msg {
        btn = btn.on_press(message);
    }
    btn.into()
}

pub fn horizontal_spacer<'a, Message: 'a>() -> Element<'a, Message> {
    row![].width(Length::Fill).into()
}
