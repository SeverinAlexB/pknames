
use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, GraphView, SettingsInteraction, SettingsStyle, Graph,
};

use std::collections::HashMap;


use petgraph::stable_graph::{NodeIndex, StableGraph};

use crate::prediction::{node::{WotFollow, WotNode}, graph::WotGraph};



impl Into<Graph<WotNode, WotFollow>> for WotGraph {
    fn into(self) -> Graph<WotNode, WotFollow> {
        let mut g: StableGraph<WotNode, WotFollow> = StableGraph::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        for node in self.nodes.iter() {
            let copy = node.clone();
            let index = g.add_node(copy);
            node_map.insert(node.pubkey.clone(), index);
        }

        for node in self.nodes.iter() {
            for follow in node.follows.iter() {
                let source_index = node_map.get(&follow.source_pubkey).unwrap().clone();
                let target_index = node_map.get(&follow.target_pubkey).unwrap().clone();
                g.add_edge(source_index, target_index, follow.clone());
            };
        }

        let mut graph = Graph::from(&g);
        let node_indexes: Vec<petgraph::prelude::NodeIndex> =
            graph.nodes_iter().map(|(index, _)| index).collect();

        // Set label
        for index in node_indexes.iter() {
            let payload = graph.node(*index).unwrap().payload();
            if payload.pubkey == "pk:s9y93dtpoibsfcnct35onkeyuiup9dfxwpftgerdqd7u84jcmkfy" {
                println!("");
            }
            let attributions = self.get_attributions(&payload.pubkey);
            let attribution_alias = Vec::from_iter(attributions.into_iter()).join(", ");
            let mut label = payload.pubkey[..8].to_string();
            if payload.alias.len() > 0 {
                label = format!("{} {}", label, payload.alias);
            };
            if attribution_alias.len() > 0 {
                label = format!("{} ({})", label, attribution_alias);
            }
            graph.node_mut(*index).unwrap().set_label(label);
        }

        graph
    }
}


struct InteractiveApp {
    graph: Graph<WotNode, WotFollow>,
}

impl InteractiveApp {
    pub fn new(_: &CreationContext<'_>, graph: Graph<WotNode, WotFollow>) -> Self {
        Self { graph }
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
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.graph)
                    .with_styles(style_settings)
                    .with_interactions(interaction_settings),
            );
        });
    }
}

/**
 * Show a GUI that visualized the graph in a simple way.
 */
pub fn visualize_graph(graph: WotGraph, title: &str) -> () {
    let egui_graph: Graph<WotNode, WotFollow> = graph.into();

    let native_options = eframe::NativeOptions::default();
    run_native(
        title,
        native_options,
        Box::new(|cc| Box::new(InteractiveApp::new(cc, egui_graph))),
    )
    .unwrap();
}
