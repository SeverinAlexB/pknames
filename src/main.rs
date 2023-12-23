use cli::cli::run_cli;
use web_of_trust::{prediction::{graph::WotGraph, node::{WotNodeType, WotFollow, WotNode}}, visualization::visualize_graph};

mod web_of_trust;
mod cli;

// fn main() {
//     run_cli();
// }

fn get_simple_graph() -> WotGraph {
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
                WotFollow::new("n2".to_string(), "d2".to_string(), -1.0).unwrap(),
            ],
        },
    });

    nodes.push(WotNode {
        pubkey: "n1".to_string(),
        alias: None,
        typ: WotNodeType::WotFollowNode {
            follows: vec![
                WotFollow::new("n1".to_string(), "d1".to_string(), -0.5).unwrap(),
                WotFollow::new("n1".to_string(), "d2".to_string(), 0.0).unwrap(),
            ],
        },
    });

    nodes.push(WotNode {
        pubkey: "me".to_string(),
        alias: None,
        typ: WotNodeType::WotFollowNode {
            follows: vec![
                WotFollow::new("me".to_string(), "n1".to_string(), 1.0).unwrap(),
                WotFollow::new("me".to_string(), "n2".to_string(), 0.5).unwrap(),
            ],
        },
    });

    WotGraph::new(nodes).unwrap()
}



fn main() {
    let wot_graph = get_simple_graph();
    visualize_graph(wot_graph);
}