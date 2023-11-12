

use super::node::{WotNode, WotFollow};



#[derive(Debug, Clone)]
pub struct WotGraph {
    pub nodes: Vec<WotNode>,
}

impl WotGraph {
    pub fn new(nodes: Vec<WotNode>) -> Self {
        WotGraph { nodes }
    }

    /**
     * Get WotFollow by pubkeys
     */
    pub fn get_follow(&self, source_pubkey: &str, target_pubkey: &str) -> Option<&WotFollow> {
        let source = self.nodes.iter().find(|n| n.pubkey == source_pubkey)?;
        source.get_follow(target_pubkey)
    }

    pub fn get_node(&self, pubkey: &str) -> Option<&WotNode> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::node::{WotNode, WotNodeType, WotFollow};
    use super::WotGraph;

    /*
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
            pubkey: "n1".to_string(),
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow {
                        source_pubkey: "n1".to_string(),
                        target_pubkey: "d1".to_string(),
                        weight: -0.5,
                    },
                    WotFollow {
                        source_pubkey: "n1".to_string(),
                        target_pubkey: "d2".to_string(),
                        weight: 0.0,
                    },
                ],
            },
        });

        nodes.push(WotNode {
            pubkey: "n2".to_string(),
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow {
                        source_pubkey: "n2".to_string(),
                        target_pubkey: "d1".to_string(),
                        weight: 1.0,
                    },
                    WotFollow {
                        source_pubkey: "n2".to_string(),
                        target_pubkey: "d2".to_string(),
                        weight: -1.0,
                    },
                ],
            },
        });

        WotGraph::new(nodes)
    }
    #[test]
    fn get_follow() {
        let graph = get_simple_graph();
        assert_eq!(graph.nodes.len(), 4);
        let follow = graph.get_follow("n2", "d1");
        assert_eq!(follow.unwrap().weight, 1.0);
    }
}
