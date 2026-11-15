#![allow(clippy::collapsible_if)]

use crate::app::*;
use crate::theme;
use iced::widget::canvas as iced_canvas;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Length, Point, Rectangle};
use iced_canvas::{Frame, Path, Stroke, Text};
use iced_fonts::lucide::*;
use lexic_core::model::Id;
use std::collections::HashMap;

const NODE_RADIUS: f32 = 28.0;
const EDGE_COLOR: Color = Color::from_rgb8(66, 71, 84);
const NODE_COLOR: Color = Color::from_rgb8(173, 198, 255);
const NODE_COLOR_ACTIVE: Color = Color::from_rgb8(78, 222, 163);
const NODE_COLOR_NEW: Color = Color::from_rgb8(168, 85, 247);

struct GraphNode {
    id: Id,
    title: String,
    position: Point,
    color: Color,
}

struct GraphEdge {
    from: Point,
    to: Point,
}

#[allow(dead_code)]
struct KnowledgeGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    center_id: Option<Id>,
    width: f32,
    height: f32,
}

impl iced_canvas::Program<Message> for KnowledgeGraph {
    type State = ();
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Draw edges first (so they appear behind nodes)
        for edge in &self.edges {
            frame.stroke(
                &Path::line(edge.from, edge.to),
                Stroke::default().with_color(EDGE_COLOR).with_width(1.0),
            );
        }

        // Draw nodes
        for node in &self.nodes {
            let is_active = self.center_id.map(|id| id == node.id).unwrap_or(false);
            let node_color = if is_active {
                NODE_COLOR_ACTIVE
            } else if node.color == NODE_COLOR_NEW {
                NODE_COLOR_NEW
            } else {
                NODE_COLOR
            };

            frame.fill(&Path::circle(node.position, NODE_RADIUS), node_color);
            frame.stroke(
                &Path::circle(node.position, NODE_RADIUS),
                Stroke::default().with_color(Color::WHITE).with_width(1.0),
            );

            // Draw title text
            frame.fill_text(Text {
                content: node.title.clone(),
                position: Point::new(
                    node.position.x - NODE_RADIUS * 0.7,
                    node.position.y + NODE_RADIUS + 14.0,
                ),
                color: theme::TEXT_SECONDARY,
                size: iced::Pixels(11.0),
                font: theme::font_code(),
                align_x: iced::widget::text::Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<iced_canvas::Action<Message>> {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) =
            event
        {
            if let Some(position) = cursor.position_in(bounds) {
                for node in &self.nodes {
                    let dx = position.x - node.position.x;
                    let dy = position.y - node.position.y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance < NODE_RADIUS + 8.0 {
                        return Some(iced_canvas::Action::publish(
                            Message::OpenNoteGraphCentered(node.id),
                        ));
                    }
                }
            }
        }
        None
    }
}

impl App {
    pub fn notes_graph_view(&self) -> Element<'_, Message> {
        let center_id = match self.screen {
            Screen::NoteGraph(Some(id)) => Some(id),
            _ => None,
        };

        // Build graph data
        let graph_data = match self.store.as_ref() {
            Some(store) => store.all_notes_for_graph().unwrap_or_default(),
            None => Vec::new(),
        };

        let width = 800.0;
        let height = 600.0;

        // Simple layout: place center node in middle, others in a circle
        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut id_to_index: HashMap<Id, usize> = HashMap::new();

        if let Some(center) = graph_data
            .iter()
            .find(|(n, _)| center_id.map(|id| n.id == id).unwrap_or(false))
        {
            // Center node
            let center_node = GraphNode {
                id: center.0.id,
                title: center.0.title.clone(),
                position: Point::new(width / 2.0, height / 2.0),
                color: if center_id.is_some() {
                    NODE_COLOR_ACTIVE
                } else {
                    NODE_COLOR
                },
            };
            id_to_index.insert(center.0.id, 0);
            nodes.push(center_node);

            // Place connected nodes in a circle around center
            let connected: Vec<Id> = center.1.clone();
            let radius = 220.0;
            for (i, target_id) in connected.iter().enumerate() {
                if let Some(target) = graph_data.iter().find(|(n, _)| n.id == *target_id) {
                    let angle = (i as f32 / connected.len().max(1) as f32) * std::f32::consts::TAU;
                    let pos = Point::new(
                        width / 2.0 + radius * angle.cos(),
                        height / 2.0 + radius * angle.sin(),
                    );
                    let node = GraphNode {
                        id: target.0.id,
                        title: target.0.title.clone(),
                        position: pos,
                        color: NODE_COLOR,
                    };
                    id_to_index.insert(target.0.id, nodes.len());
                    nodes.push(node);
                }
            }
        } else if !graph_data.is_empty() {
            // No center, arrange in a grid
            let cols = (graph_data.len() as f32).sqrt().ceil() as usize;
            let cell_w = width / cols as f32;
            let cell_h = height / graph_data.len().div_ceil(cols) as f32;
            for (i, (note, _)) in graph_data.iter().enumerate() {
                let row = i / cols;
                let col = i % cols;
                let node = GraphNode {
                    id: note.id,
                    title: note.title.clone(),
                    position: Point::new(
                        cell_w * col as f32 + cell_w / 2.0,
                        cell_h * row as f32 + cell_h / 2.0,
                    ),
                    color: NODE_COLOR_NEW,
                };
                id_to_index.insert(note.id, nodes.len());
                nodes.push(node);
            }
        }

        // Build edges
        let mut edges: Vec<GraphEdge> = Vec::new();
        for (note, targets) in &graph_data {
            let source_idx = match id_to_index.get(&note.id) {
                Some(idx) => *idx,
                None => continue,
            };
            let source_pos = nodes[source_idx].position;
            for target_id in targets {
                if let Some(&target_idx) = id_to_index.get(target_id) {
                    edges.push(GraphEdge {
                        from: source_pos,
                        to: nodes[target_idx].position,
                    });
                }
            }
        }

        let graph = KnowledgeGraph {
            nodes,
            edges,
            center_id,
            width,
            height,
        };

        let header = row![
            column![
                text("Knowledge Graph")
                    .size(20)
                    .color(theme::TEXT_PRIMARY)
                    .font(theme::font_code()),
                text("Notes and their connections")
                    .size(12)
                    .color(theme::TEXT_MUTED),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            button(
                row![
                    network().size(13).color(theme::ON_PRIMARY_CONTAINER),
                    text("Full Graph")
                        .size(11)
                        .color(theme::ON_PRIMARY_CONTAINER)
                        .font(theme::font_code()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([8, 14])
            .style(theme::primary_button())
            .on_press(Message::OpenNoteGraph),
            if let Some(center_id) = center_id {
                button(
                    row![
                        external_link().size(13).color(theme::ON_PRIMARY_CONTAINER),
                        text("Open Note")
                            .size(11)
                            .color(theme::ON_PRIMARY_CONTAINER)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::ghost_button())
                .on_press(Message::Navigate(Screen::NoteDetail(center_id)))
            } else {
                button(
                    row![
                        external_link().size(13).color(theme::TEXT_MUTED),
                        text("Open Note")
                            .size(11)
                            .color(theme::TEXT_MUTED)
                            .font(theme::font_code()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 14])
                .style(theme::ghost_button())
            },
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let canvas_widget = iced::widget::canvas(graph)
            .width(Length::Fill)
            .height(Length::Fill);

        let instructions = text("Click a node to open that note · Use Full Graph to reset layout")
            .size(11)
            .color(theme::TEXT_MUTED)
            .font(theme::font_code());

        let content = column![header, canvas_widget, instructions].spacing(12);

        let scroll = scrollable(content).height(Length::Fill);
        container(scroll)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
