mod web_of_trust;
mod cli;

use web_of_trust::prediction::{node::WotNodeType, graph::WotGraph, predictor::WotPredictor};

use crate::{web_of_trust::prediction::node::{WotFollow, WotNode}, cli::cli::run_cli};


fn main() {
    run_cli();

    let mut nodes: Vec<WotNode> = Vec::new();

    // Classes
    nodes.push(WotNode {
        pubkey: "d1".to_string(),
        alias: Some(String::from("example.com1")),
        typ: WotNodeType::WotClass,
    });
    nodes.push(WotNode {
        pubkey: "d2".to_string(),
        alias: Some(String::from("example.com2")),
        typ: WotNodeType::WotClass,
    });

    nodes.push(WotNode {
        pubkey: "n2".to_string(),
        alias: None,
        typ: WotNodeType::WotFollowNode {
            follows: vec![
                WotFollow::new("n2".to_string(), "d1".to_string(), 1.0).unwrap(),
                WotFollow::new("n2".to_string(), "d2".to_string(), -1.0).unwrap()
            ],
        },
    });

    nodes.push(WotNode {
        pubkey: "n1".to_string(),
        alias: None,
        typ: WotNodeType::WotFollowNode {
            follows: vec![
                WotFollow::new("n1".to_string(), "d1".to_string(), -0.5).unwrap(),
                WotFollow::new("n1".to_string(), "d2".to_string(), 0.0).unwrap()
            ],
        },
    });

    nodes.push(WotNode {
        pubkey: "me".to_string(),
        alias: None,
        typ: WotNodeType::WotFollowNode {
            follows: vec![
                WotFollow::new("me".to_string(), "n1".to_string(), 1.0).unwrap(),
                WotFollow::new("me".to_string(), "n2".to_string(), 0.5).unwrap()
            ],
        },
    });

    let graph = WotGraph::new(nodes).unwrap();

    let predictor: WotPredictor = graph.into();
    let result = predictor.predict();
    println!("Result {}", result);


}