use std::collections::HashMap;

use super::node::WotNodeTrait;

#[derive(Debug)]
struct WotGraphIndex <'a> {
    map: HashMap<String, &'a Box<dyn WotNodeTrait>>
}

impl <'a> WotGraphIndex<'a> {
    pub fn new(nodes: & 'a Vec<Box<dyn WotNodeTrait>>) -> Self {
        let mut map = HashMap::new();
        for node in nodes.iter() {
            let pubkey = String::from(node.get_pubkey().clone());
            map.insert(pubkey, node);
        };
        Self { map }
    }
}

#[derive(Debug)]
pub struct WotGraph<'a> {
    pub nodes: Vec<Box<dyn WotNodeTrait>>,
    index: WotGraphIndex<'a>
}

impl<'a> WotGraph<'a> {
    pub fn new<'b>(nodes: Vec<Box<dyn WotNodeTrait>>) -> Self {
        let index: WotGraphIndex<'a> = WotGraphIndex::new(&nodes);
        WotGraph { nodes, index }
    }
}


#[cfg(test)]
mod tests {
    use super::WotGraph;
    use super::super::node::{WotNodeTrait, WotClass, WotFollow, WotNode};
    #[test]
    fn construct_graph() {
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
        assert_eq!(graph.map.len(), 4);
        assert_eq!(graph.nodes.len(), 4);
    }
}