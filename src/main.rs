mod web_of_trust;
use crate::web_of_trust::node::{WotClass, WotFollow, WotNodeTrait, WotNode};
use crate::web_of_trust::graph::{WotGraph};

fn main() {

    let mut nodes: Vec<Box<dyn WotNodeTrait>> = Vec::new();

    nodes.push(Box::new(WotClass{pubkey: "d1".to_string(), name: Some("example.com1".to_string())}));
    nodes.push(Box::new(WotClass{pubkey: "d2".to_string(), name: Some("example.com2".to_string())}));

    nodes.push(Box::new(WotNode{pubkey: "n1".to_string(), follows: vec![
        WotFollow{
            source_pubkey: "n1".to_string(),
            target_pubkey: "d1".to_string(),
            weight: -0.5
        },
        WotFollow{
            source_pubkey: "n1".to_string(),
            target_pubkey: "d2".to_string(),
            weight: 0.0
        }
    ]}));
    nodes.push(Box::new(WotNode{pubkey: "n2".to_string(), follows: vec![
        WotFollow{
            source_pubkey: "n2".to_string(),
            target_pubkey: "d1".to_string(),
            weight: 1.0
        },
        WotFollow{
            source_pubkey: "n2".to_string(),
            target_pubkey: "d2".to_string(),
            weight: -1.0
        }
    ]}));

    let graph = WotGraph::new(nodes);
    println!("Graph {:?}", graph);
    println!("")
    
    // let n = node::WotNode{pubkey: "".to_string()}
}