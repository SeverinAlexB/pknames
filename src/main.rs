// mod web_of_trust;
// mod cli;

// fn main() {
    // run_cli();
// }


use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsInteraction, SettingsStyle,
};
use petgraph::stable_graph::StableGraph;


#[derive(Debug, Clone)]
pub struct NodeProps {
    pub label: String
}

pub struct InteractiveApp {
    g: Graph<NodeProps, ()>,
}

impl InteractiveApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g }
    }
}


impl App for InteractiveApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let interaction_settings = &SettingsInteraction::new()
                .with_dragging_enabled(true)
                .with_node_clicking_enabled(true)
                .with_node_selection_enabled(true)
                .with_node_selection_multi_enabled(true)
                .with_edge_clicking_enabled(true)
                .with_edge_selection_enabled(true)
                .with_edge_selection_multi_enabled(true);
            let style_settings = &SettingsStyle::new().with_labels_always(true);
            ui.add(
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.g)
                    .with_styles(style_settings)
                    .with_interactions(interaction_settings),
            );
        });
    }
}

fn generate_graph() -> Graph<NodeProps, ()> {
    let mut g = StableGraph::new();

    let a = g.add_node(NodeProps{label: "A".to_string()});
    let b = g.add_node(NodeProps{label: "B".to_string()});
    let c = g.add_node(NodeProps{label: "C".to_string()});
    

    g.add_edge(a, b, ());
    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    let mut graph = Graph::from(&g);
    let indexes: Vec<petgraph::prelude::NodeIndex> = graph.nodes_iter().map(|(index, _)| index).collect();
    for index in indexes.iter() {
        let label = graph.node_mut(*index).unwrap().payload().label.clone();
        graph.node_mut(*index).unwrap().set_label(label);
    }
    graph
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_interactive_demo",
        native_options,
        Box::new(|cc| Box::new(InteractiveApp::new(cc))),
    )
    .unwrap();
}