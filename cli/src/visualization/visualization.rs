
use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{GraphView, SettingsInteraction, SettingsStyle, Graph,
};

use std::collections::HashMap;


use petgraph::{stable_graph::{NodeIndex, StableGraph, DefaultIx}, Directed};

use pknames_core::prediction::{node::{WotFollow, WotNode}, graph::WotGraph, predictor::WotPrediction};

use super::{node_vis::FancyNodeShape, edge_vis::FancyEdgeShape};

#[derive(Clone)]
pub struct PredictedVisWotNode {
    pub node: WotNode,
    pub power: Option<f32>
}


pub fn wotgraph_into_graph(wotgraph: WotGraph) -> Graph<PredictedVisWotNode, WotFollow, Directed, DefaultIx, FancyNodeShape, FancyEdgeShape> {
    let mut g: StableGraph<PredictedVisWotNode, WotFollow> = StableGraph::new();
    let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
    for node in wotgraph.nodes.iter() {
        let vis_node = PredictedVisWotNode {
            node: node.clone(),
            power: None
        };
        let index = g.add_node(vis_node);
        node_map.insert(node.pubkey.clone(), index);
    }

    for node in wotgraph.nodes.iter() {
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
        if payload.node.pubkey == "pk:s9y93dtpoibsfcnct35onkeyuiup9dfxwpftgerdqd7u84jcmkfy" {
            println!("");
        }
        let attributions = wotgraph.get_attributions(&payload.node.pubkey);
        let attribution_alias = Vec::from_iter(attributions.into_iter()).join(", ");
        let mut label = payload.node.pubkey[..8].to_string();
        if payload.node.alias.len() > 0 {
            label = format!("{} {}", label, payload.node.alias);
        };
        if attribution_alias.len() > 0 {
            label = format!("{} ({})", label, attribution_alias);
        }
        graph.node_mut(*index).unwrap().set_label(label);
        
    }

    graph
}

// impl Into<Graph<PredictedVisWotNode, WotFollow, Directed, DefaultIx, FancyNodeShape, FancyEdgeShape>> for WotGraph {
//     fn into(self) -> Graph<PredictedVisWotNode, WotFollow, Directed, DefaultIx, FancyNodeShape, FancyEdgeShape> {
//         let mut g: StableGraph<PredictedVisWotNode, WotFollow> = StableGraph::new();
//         let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
//         for node in self.nodes.iter() {
//             let vis_node = PredictedVisWotNode {
//                 node: node.clone(),
//                 power: None
//             };
//             let index = g.add_node(vis_node);
//             node_map.insert(node.pubkey.clone(), index);
//         }

//         for node in self.nodes.iter() {
//             for follow in node.follows.iter() {
//                 let source_index = node_map.get(&follow.source_pubkey).unwrap().clone();
//                 let target_index = node_map.get(&follow.target_pubkey).unwrap().clone();
//                 g.add_edge(source_index, target_index, follow.clone());
//             };
//         }

//         let mut graph = Graph::from(&g);
//         let node_indexes: Vec<petgraph::prelude::NodeIndex> =
//             graph.nodes_iter().map(|(index, _)| index).collect();

//         // Set label
//         for index in node_indexes.iter() {
//             let payload = graph.node(*index).unwrap().payload();
//             if payload.node.pubkey == "pk:s9y93dtpoibsfcnct35onkeyuiup9dfxwpftgerdqd7u84jcmkfy" {
//                 println!("");
//             }
//             let attributions = self.get_attributions(&payload.node.pubkey);
//             let attribution_alias = Vec::from_iter(attributions.into_iter()).join(", ");
//             let mut label = payload.node.pubkey[..8].to_string();
//             if payload.node.alias.len() > 0 {
//                 label = format!("{} {}", label, payload.node.alias);
//             };
//             if attribution_alias.len() > 0 {
//                 label = format!("{} ({})", label, attribution_alias);
//             }
//             graph.node_mut(*index).unwrap().set_label(label);
            
//         }

//         graph
//     }
// }


struct InteractiveApp {
    graph: Graph<PredictedVisWotNode, WotFollow, Directed, DefaultIx, FancyNodeShape, FancyEdgeShape>,
}

impl InteractiveApp {
    pub fn new(cc: &CreationContext<'_>, graph: Graph<PredictedVisWotNode, WotFollow, Directed, DefaultIx, FancyNodeShape, FancyEdgeShape>) -> Self {
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
                &mut GraphView::<_, _, _, _, FancyNodeShape, FancyEdgeShape>::new(&mut self.graph)
                    .with_styles(style_settings)
                    .with_interactions(interaction_settings)
                    ,
            );
        });
    }
}

/**
 * Show a GUI that visualized the graph in a simple way.
 */
pub fn visualize_graph(graph: WotGraph, title: &str, me_pubkey: Option<&str>, result: Option<WotPrediction>) -> () {
    let mut egui_graph: Graph<PredictedVisWotNode, WotFollow, Directed, DefaultIx, FancyNodeShape, FancyEdgeShape> = wotgraph_into_graph(graph);
    let node_indexes: Vec<petgraph::prelude::NodeIndex> =
    egui_graph.nodes_iter().map(|(index, _)| index).collect();


    // Set power if available
    for index in node_indexes.iter() {
        let node = egui_graph.node_mut(*index).unwrap();
        let payload = node.payload();

        let mut power_val = None;
        if let Some(ref res) = result {
            power_val = res.get_value(&payload.node.pubkey);
        }
        let is_me = me_pubkey.is_some() && me_pubkey.unwrap() == payload.node.pubkey;
        let internal_data = node.display_mut();
        internal_data.is_me = is_me;
        internal_data.power = power_val;

    };

    let native_options = eframe::NativeOptions::default();
    run_native(
        title,
        native_options,
        Box::new(|cc| {
            Box::new(InteractiveApp::new(cc, egui_graph))
        }),
    )
    .unwrap();
}
