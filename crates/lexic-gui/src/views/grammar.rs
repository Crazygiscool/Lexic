#![allow(clippy::collapsible_if)]

use crate::app::*;
use crate::components;
use crate::theme;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide::*;

impl App {
    pub fn grammar_rules_view(&self) -> Element<'_, Message> {
        let rules = &self.grammar_rules;
        let categories = &self.grammar_categories;

        let header = row![
            column![
                text("Grammar Rules")
                    .size(20)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                text("Grammar rules and style guidelines")
                    .size(12)
                    .color(theme::TEXT_MUTED),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            button(
                row![
                    plus().size(13).color(theme::ON_PRIMARY_CONTAINER),
                    text("New Rule")
                        .size(11)
                        .color(theme::ON_PRIMARY_CONTAINER)
                        .font(theme::font_code()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([8, 14])
            .style(theme::primary_button())
            .on_press(Message::NewGrammarRule),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        // Category filter chips
        let mut category_chips = row![
            text("Category:")
                .size(11)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code()),
        ]
        .spacing(6)
        .align_y(Alignment::Center);
        category_chips = category_chips.push(
            button(text("All").size(11).font(theme::font_code()).color(
                if self.grammar_category_filter.is_none() {
                    theme::ON_PRIMARY_CONTAINER
                } else {
                    theme::TEXT_MUTED
                },
            ))
            .padding([4, 10])
            .style(theme::button_style(self.grammar_category_filter.is_none()))
            .on_press(Message::GrammarCategoryFilterChanged(None)),
        );
        for category in categories {
            let active = self.grammar_category_filter.as_deref() == Some(category.as_str());
            category_chips = category_chips.push(
                button(
                    text(category.as_str())
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
                .on_press(Message::GrammarCategoryFilterChanged(Some(
                    category.clone(),
                ))),
            );
        }

        // Rules list
        let rules_list: Element<'_, Message> = if rules.is_empty() {
            components::empty_state(
                "No grammar rules yet",
                String::from("Create your first grammar rule to build your language reference."),
            )
        } else {
            let mut list = column![].spacing(6);
            for rule in rules {
                let selected = self.grammar_selected_id == Some(rule.id);
                let category_color = theme::subject_color(&rule.category);
                list = list.push(
                    button(
                        row![
                            column![
                                text(&rule.title)
                                    .size(13)
                                    .color(if selected {
                                        theme::PRIMARY
                                    } else {
                                        theme::TEXT_PRIMARY
                                    })
                                    .font(theme::font_code()),
                                row![
                                    text(&rule.category)
                                        .size(10)
                                        .color(theme::TEXT_MUTED)
                                        .font(theme::font_code()),
                                    Space::new().width(6),
                                    components::chip(
                                        rule.category.clone(),
                                        category_color,
                                        theme::BG_CONTAINER_HIGH,
                                    ),
                                ]
                                .spacing(2),
                            ]
                            .spacing(2),
                            Space::new().width(Length::Fill),
                            text("✎").size(12).color(theme::TEXT_MUTED),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding([10, 14])
                    .style(theme::button_style(selected))
                    .on_press(Message::GrammarRuleSelected(rule.id)),
                );
            }
            column![category_chips, list].spacing(10).into()
        };

        let content = column![header, rules_list].spacing(16);

        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn grammar_rule_detail_view(&self) -> Element<'_, Message> {
        let is_new = self.active_grammar_rule.is_none();

        let title_input = text_input("Rule title", &self.grammar_editor_title)
            .on_input(Message::GrammarRuleTitleChanged)
            .padding([8, 12])
            .width(Length::Fill)
            .size(18)
            .style(theme::themed_input());

        let category_input = text_input(
            "Category (e.g. Syntax, Punctuation)",
            &self.grammar_editor_category,
        )
        .on_input(Message::GrammarRuleCategoryChanged)
        .padding([8, 12])
        .width(Length::FillPortion(1))
        .style(theme::themed_input());

        let explanation_input = text_input("Explanation", &self.grammar_editor_explanation)
            .on_input(Message::GrammarRuleExplanationChanged)
            .padding([8, 12])
            .width(Length::Fill)
            .style(theme::themed_input());

        let examples_input = text_input("Examples (one per line)", &self.grammar_editor_examples)
            .on_input(Message::GrammarRuleExamplesChanged)
            .padding([8, 12])
            .width(Length::FillPortion(1))
            .style(theme::themed_input());

        let counter_examples_input = text_input(
            "Counter-examples (one per line)",
            &self.grammar_editor_counter_examples,
        )
        .on_input(Message::GrammarRuleCounterExamplesChanged)
        .padding([8, 12])
        .width(Length::FillPortion(1))
        .style(theme::themed_input());

        let header = row![
            column![
                text(if is_new {
                    "New Grammar Rule"
                } else {
                    "Edit Grammar Rule"
                })
                .size(20)
                .color(theme::TEXT_PRIMARY)
                .font(theme::font_code()),
                text("Define grammar rules for your writing reference.")
                    .size(12)
                    .color(theme::TEXT_MUTED)
                    .font(theme::font_serif()),
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
                .on_press(Message::CancelGrammarRuleEdit),
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
                .on_press(Message::SaveGrammarRule),
            ]
            .spacing(8),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let status_text = if let Some(status) = &self.grammar_editor_status {
            text(status)
                .size(11)
                .color(theme::TEXT_MUTED)
                .font(theme::font_code())
        } else {
            text("").size(1)
        };

        let content = column![
            header,
            category_input,
            title_input,
            explanation_input,
            examples_input,
            counter_examples_input,
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
