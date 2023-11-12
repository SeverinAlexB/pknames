use super::node::{WotNode, WotFollow, WotNodeType};


#[derive(Debug, Clone)]
pub struct WotGraph {
    pub nodes: Vec<WotNode>
}


impl WotGraph {
    pub fn new(mut nodes: Vec<WotNode>) -> Result<Self, &'static str> {
        if !WotNode::is_unique(&nodes) {
            Err("Given node list with pubkeys, pubkeys are not unique.")
        } else {
            nodes.sort_unstable_by_key(|node| node.pubkey.clone());
            Ok(WotGraph { nodes })
        }
    }

    /**
     * Get WotFollow by pubkeys
     */
    pub fn get_follow(&self, source_pubkey: &str, target_pubkey: &str) -> Option<&WotFollow> {
        let source = self.nodes.iter().find(|n| n.pubkey == source_pubkey)?;
        source.get_follow(target_pubkey)
    }

    /**
     * Find node by pubkey
     */
    pub fn get_node(&self, pubkey: &str) -> Option<&WotNode> {
        WotNode::binary_search(pubkey, &self.nodes)
    }

    pub fn get_classes(&self) -> Vec<&WotNode> {
        let result: Vec<&WotNode> = self.nodes.iter().filter(|n| {
            if let WotNodeType::WotClass { name: _ } = n.typ {
                true
            } else {
                false
            }
        }).collect();
        result
    }

    pub fn get_follow_nodes(&self) -> Vec<&WotNode> {
        let result: Vec<&WotNode> = self.nodes.iter().filter(|n| {
            if let WotNodeType::WotFollowNode { follows: _ } = n.typ {
                true
            } else {
                false
            }
        }).collect();
        result
    }

    pub fn get_layers(&self) -> Vec<Vec<&WotNode>> {
        let mut remaining_nodes: Vec<&WotNode> = self.nodes.iter().collect();
        let mut layers: Vec<Vec<&WotNode>> = Vec::new();

        loop {
            if remaining_nodes.len() == 0 {
                break
            };

            // Find leaf nodes
            let mut current_layer: Vec<&WotNode> = Vec::new();
            for node in remaining_nodes.iter() {
                let is_leaf_node;
                let follows = node.get_follows();
                if follows.is_none() {
                    is_leaf_node = true
                } else {
                    let target_node = follows.unwrap().iter().find(|follow| {
                        let target_node = WotNode::binary_search_ref(&follow.target_pubkey, &remaining_nodes);
                        target_node.is_some()
                    });
                    is_leaf_node = target_node.is_none();
                };

                if is_leaf_node {
                    current_layer.push(node);
                };
            }
            current_layer.sort_unstable_by_key(|node| &node.pubkey);
            
            // Remove leaf nodes
            for node in current_layer.iter() {
                let index = remaining_nodes.iter().position(|x| *x == *node).unwrap();
                remaining_nodes.remove(index);
            }

            layers.push(current_layer);
        }
        layers.reverse();
        layers
    }

    pub fn depth(&self) -> usize {
        self.get_layers().len()
    }
}

#[cfg(test)]
mod tests {
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
            pubkey: "me".to_string(),
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow {
                        source_pubkey: "me".to_string(),
                        target_pubkey: "n1".to_string(),
                        weight: 1.0,
                    },
                    WotFollow {
                        source_pubkey: "me".to_string(),
                        target_pubkey: "n2".to_string(),
                        weight: 0.5,
                    },
                ],
            },
        });

        WotGraph::new(nodes).unwrap()
    }
    
    #[test]
    fn get_follow() {
        let graph = get_simple_graph();
        assert_eq!(graph.nodes.len(), 5);
        let follow = graph.get_follow("n2", "d1");
        assert_eq!(follow.unwrap().weight, 1.0);
    }

    #[test]
    fn pubkeys_unique() {
        let mut nodes: Vec<WotNode> = Vec::new();

        // Classes
        nodes.push(WotNode {
            pubkey: "d1".to_string(),
            typ: WotNodeType::WotClass {
                name: "example.com1".to_string(),
            },
        });
        nodes.push(WotNode {
            pubkey: "d1".to_string(),
            typ: WotNodeType::WotClass {
                name: "example.com2".to_string(),
            },
        });
        let result = std::panic::catch_unwind(|| WotGraph::new(nodes));
        assert!(result.is_err());
    }

    #[test]
    fn get_node() {
        let graph = get_simple_graph();
        assert_eq!(graph.get_node("n2").unwrap().pubkey, "n2");
        assert_eq!(graph.get_node("n2").unwrap().pubkey, "n2");
        assert_eq!(graph.get_node("n1").unwrap().pubkey, "n1");
        assert_eq!(graph.get_node("missing").is_none(), true);
    }

    #[test]
    fn get_classes() {
        let graph = get_simple_graph();
        let classes = graph.get_classes();
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].pubkey, "d1");
        assert_eq!(classes[1].pubkey, "d2");
    }

    #[test]
    fn get_follow_nodes() {
        let graph = get_simple_graph();
        let nodes = graph.get_follow_nodes();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].pubkey, "me");
        assert_eq!(nodes[1].pubkey, "n1");
        assert_eq!(nodes[2].pubkey, "n2");
    }

    #[test]
    fn sorted_nodes() {
        let graph = get_simple_graph();
        assert_eq!(graph.nodes[0].pubkey, "d1");
        assert_eq!(graph.nodes[1].pubkey, "d2");
        assert_eq!(graph.nodes[2].pubkey, "me");
        assert_eq!(graph.nodes[3].pubkey, "n1");
        assert_eq!(graph.nodes[4].pubkey, "n2");
    }

    #[test]
    fn layers() {
        let graph = get_simple_graph();
        let layers = graph.get_layers();
        assert_eq!(layers.len(), 3);
        let first = &layers[0];
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].pubkey, "me");

        let second = &layers[1];
        assert_eq!(second.len(), 2);
        assert_eq!(second[0].pubkey, "n1");
        assert_eq!(second[1].pubkey, "n2");

        let third = &layers[2];
        assert_eq!(third.len(), 2);
        assert_eq!(third[0].pubkey, "d1");
        assert_eq!(third[1].pubkey, "d2");
    }

    #[test]
    fn depth() {
        let graph = get_simple_graph();
        assert_eq!(graph.depth(), 3);
    }
}
