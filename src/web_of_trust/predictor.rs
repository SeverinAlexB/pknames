use std::collections::HashMap;

use super::graph::WotGraph;

pub struct WotPredictionResult {
    map: HashMap<String, f32>
}

impl WotPredictionResult {
    pub fn new() -> Self {
        WotPredictionResult { map: HashMap::new() }
    }

    pub fn get_pubkeys(&self) -> Vec<&String> {
        let keys: Vec<&String> = self.map.keys().collect();
        keys
    }

    pub fn get_value(&self, pubkey: &str) -> Option<&f32> {
        self.map.get(pubkey)
    }
}


pub struct WotPredictor {
    pub graph: WotGraph
}

impl WotPredictor {

}

impl From<WotGraph> for WotPredictor {
    fn from(value: WotGraph) -> Self {
        WotPredictor { graph: value }
    }
}


#[cfg(test)]
mod tests {
    use crate::web_of_trust::predictor::WotPredictor;

    use super::super::node::{WotNode, WotNodeType, WotFollow};
    use super::WotGraph;

    /**
     * Constructs a simple graph
     */
    fn get_simple_graph() -> WotGraph {
        let mut nodes: Vec<WotNode> = Vec::new();

        // Classes
        nodes.push(WotNode {
            pubkey: "d1".to_string(),
            typ: WotNodeType::WotClass {
                name: "example.com1".to_string(),
            },
        });
        nodes.push(WotNode {
            pubkey: "d2".to_string(),
            typ: WotNodeType::WotClass {
                name: "example.com2".to_string(),
            },
        });

        nodes.push(WotNode {
            pubkey: "n2".to_string(),
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow::new("n2".to_string(), "d1".to_string(), 1.0).unwrap(),
                    WotFollow::new("n2".to_string(), "d2".to_string(), -1.0).unwrap()
                ],
            },
        });

        nodes.push(WotNode {
            pubkey: "n1".to_string(),
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow::new("n1".to_string(), "d1".to_string(), -0.5).unwrap(),
                    WotFollow::new("n1".to_string(), "d2".to_string(), 0.0).unwrap()
                ],
            },
        });

        nodes.push(WotNode {
            pubkey: "me".to_string(),
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow::new("me".to_string(), "n1".to_string(), 1.0).unwrap(),
                    WotFollow::new("me".to_string(), "n2".to_string(), 0.5).unwrap()
                ],
            },
        });

        WotGraph::new(nodes).unwrap()
    }

    #[test]
    fn from_into_graph() {
        let old_graph = get_simple_graph();
        let predictor: WotPredictor = old_graph.clone().into();
        let new_graph: WotGraph = predictor.into();

        assert_eq!(old_graph.depth(), new_graph.depth());
        assert_eq!(old_graph.nodes.len(), new_graph.nodes.len());
    }
}
