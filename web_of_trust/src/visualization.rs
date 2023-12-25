
use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, GraphView, SettingsInteraction, SettingsStyle, Graph,
};

use std::collections::HashMap;


use petgraph::stable_graph::{NodeIndex, StableGraph};

use crate::prediction::{node::{WotFollow, WotNode}, graph::WotGraph};



impl<F, C> Into<Graph<WotNode<F, C>, WotFollow>> for WotGraph<F, C> where F: Clone, C: Clone {
    fn into(self) -> Graph<WotNode<F, C>, WotFollow> {
        let mut g: StableGraph<WotNode<F, C>, WotFollow> = StableGraph::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        for node in self.nodes.iter() {
            let copy = node.clone();
            let index = g.add_node(copy);
            node_map.insert(node.pubkey.clone(), index);
        }

        for node in self.nodes.iter() {
            if let Some(follows) = node.get_follows() {
                for follow in follows.iter() {
                    let source_index = node_map.get(&follow.source_pubkey).unwrap().clone();
                    let target_index = node_map.get(&follow.target_pubkey).unwrap().clone();
                    g.add_edge(source_index, target_index, follow.clone());
                }
            };
        }

        let mut graph = Graph::from(&g);
        let node_indexes: Vec<petgraph::prelude::NodeIndex> =
            graph.nodes_iter().map(|(index, _)| index).collect();
        for index in node_indexes.iter() {
            let label = graph.node_mut(*index).unwrap().payload().pubkey.clone();
            graph.node_mut(*index).unwrap().set_label(label);
        }

        graph
    }
}


struct InteractiveApp<F, C> where F: Clone, C: Clone {
    graph: Graph<WotNode<F, C>, WotFollow>,
}

impl<F, C> InteractiveApp<F, C> where F: Clone, C: Clone {
    pub fn new(_: &CreationContext<'_>, graph: Graph<WotNode<F, C>, WotFollow>) -> Self {
        Self { graph }
    }
}


impl<F, C> App for InteractiveApp<F, C> where F: Clone, C: Clone {
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
pub fn visualize_graph<F, C>(graph: WotGraph<F, C>, title: &str) -> () where F: Clone + 'static, C: Clone + 'static {
    let egui_graph: Graph<WotNode<F, C>, WotFollow> = graph.into();

    let native_options = eframe::NativeOptions::default();
    run_native(
        title,
        native_options,
        Box::new(|cc| Box::new(InteractiveApp::new(cc, egui_graph))),
    )
    .unwrap();
}
