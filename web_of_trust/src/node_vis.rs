use egui::{
    epaint::{CircleShape, TextShape},
    Color32, FontFamily, FontId, Pos2, Shape, Stroke, Vec2,
};
use egui_graphs::{DisplayNode, DrawContext, NodeProps};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{prediction::node::WotNode, visualization::PredictedVisWotNode};

/// This is the default node shape which is used to display nodes in the graph.
///
/// You can use this implementation as an example for implementing your own custom node shapes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct FancyNodeShape {
    pub pos: Pos2,

    pub selected: bool,
    pub dragged: bool,

    pub label_text: String,

    /// Shape defined property
    pub radius: f32,
    pub is_class: bool,
    pub power: Option<f32>,
}

impl From<NodeProps<PredictedVisWotNode>> for FancyNodeShape {
    fn from(node_props: NodeProps<PredictedVisWotNode>) -> Self {
        FancyNodeShape {
            pos: node_props.location,
            selected: node_props.selected,
            dragged: node_props.dragged,
            label_text: node_props.label.to_string(),
            radius: 5.0,
            is_class: node_props.payload.node.follows.len() == 0,
            power: node_props.payload.power,
        }
    }
}

impl<E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<PredictedVisWotNode, E, Ty, Ix>
    for FancyNodeShape
{
    fn is_inside(&self, pos: Pos2) -> bool {
        is_inside_circle(self.pos, self.radius, pos)
    }

    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        closest_point_on_circle(self.pos, self.radius, dir)
    }

    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let mut res = Vec::with_capacity(2);
        let is_interacted = self.selected || self.dragged;

        let style = match is_interacted {
            true => ctx.ctx.style().visuals.widgets.active,
            false => ctx.ctx.style().visuals.widgets.inactive,
        };

        let mut color = Color32::GREEN;
        if self.power.is_some() {

            let power = self.power.unwrap();
            if self.is_class {
                // between 0 and 1
                let val = f32::powf(10.0, power)/10.0;
                color = color.linear_multiply(val)
            } else {
                // between 0 and infinite. Mostly between 0 and 2 though.
                let val = f32::powf(3.0, power)/5.0 - 0.2;
                let val = f32::min(1.0, val);
                color = color.linear_multiply(val)
            };
        };

        let stroke = if self.is_class {
            Stroke::new(5.0, Color32::BLACK)
        } else {
            Stroke::new(1.0, Color32::GRAY)
        };
        let circle_center = ctx.meta.canvas_to_screen_pos(self.pos);
        let circle_radius = ctx.meta.canvas_to_screen_size(self.radius);
        let circle_shape = CircleShape {
            center: circle_center,
            radius: circle_radius,
            fill: color,
            stroke,
        };
        res.push(circle_shape.into());

        let label_visible = true;
        if !label_visible {
            return res;
        }

        let font_color = Color32::DARK_RED;
        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.label_text.clone(),
                FontId::new(circle_radius, FontFamily::Monospace),
                font_color,
            )
        });

        // display label centered over the circle
        let label_pos = Pos2::new(
            circle_center.x - galley.size().x / 2.,
            circle_center.y - circle_radius * 2.,
        );

        let label_shape = TextShape::new(label_pos, galley);
        res.push(label_shape.into());

        res
    }

    fn update(&mut self, state: &NodeProps<PredictedVisWotNode>) {
        self.pos = state.location;
        self.pos = state.location;
        self.selected = state.selected;
        self.dragged = state.dragged;
        self.label_text = state.label.to_string();
    }
}

fn closest_point_on_circle(center: Pos2, radius: f32, dir: Vec2) -> Pos2 {
    center + dir.normalized() * radius
}

fn is_inside_circle(center: Pos2, radius: f32, pos: Pos2) -> bool {
    let dir = pos - center;
    dir.length() <= radius
}
