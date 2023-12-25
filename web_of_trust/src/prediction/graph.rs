use super::{node::{WotNode, WotFollow, WotNodeType}, predictor::WotPredictor};
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone)]
pub struct WotGraph {
    pub nodes: Vec<WotNode>
}

impl fmt::Display for WotGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layers = self.get_layers();

        let layers_strings:Vec<String> = layers.iter().enumerate().map(|(i, layer)| {
            let node_strings: Vec<String> = layer.iter().map(|node| format!("- {}", node)).collect();
            let lay = node_strings.join("\n");
            format!("Layer {}\n{}", i, lay)
        }).collect();

        let graph = layers_strings.join("\n");
        write!(f, "{}",  graph)
    }
}


impl WotGraph {
    pub fn new(mut nodes: Vec<WotNode>) -> Result<Self, &'static str> {
        nodes.sort_unstable_by_key(|node| node.pubkey.clone());
        let graph = WotGraph { nodes };

        if !graph.is_unique() {
            return Err("Node pubkeys are not unique.");
        };
        if !graph.is_well_connected() {
            return Err("Graph is not well connected. WotFollow.target_pubkey does not have a coresponding node.")
        };

        Ok(graph)
    }

    /**
     * Checks if all follow target nodes exist.
     */
    fn is_well_connected(&self) -> bool {
        let mut is_target_missing = false;
        'outer: for node in self.nodes.iter() {
            let follows = node.get_follows();
            if follows.is_none() {
                continue;
            }

            for follow in follows.unwrap().iter() {
                let target = self.get_node(&follow.target_pubkey);
                if target.is_none() {
                    is_target_missing = true;
                    break 'outer
                }
            }
        };
        !is_target_missing
    }

     /**
     * Checks if pubkeys are unique
     */
    pub fn is_unique(&self) -> bool {
        let pubkeys = self.nodes.iter().map(|node| node.pubkey.clone());
        let set: HashSet<String> = HashSet::from_iter(pubkeys);
        set.len() == self.nodes.len()
    }

    /**
     * Get WotFollow by pubkeys
     */
    pub fn get_follow(&self, source_pubkey: &str, target_pubkey: &str) -> Option<&WotFollow> {
        let source = self.get_node(source_pubkey)?;
        source.get_follow(target_pubkey)
    }

    /**
     * Get all nodes
     */
    pub fn get_nodes(&self) -> HashSet<&WotNode> {
        HashSet::from_iter(self.nodes.iter())
    }

    /**
     * Get all unique follows
     */
    pub fn get_follows(&self) -> HashSet<&WotFollow> {
        let set: HashSet<&WotFollow, _> = self.nodes.iter().filter_map(|node| node.get_follows()).flatten().collect();
        set
    }

    /**
     * Get WotFollow by pubkeys
     */
    pub fn get_follow_mut(&mut self, source_pubkey: &str, target_pubkey: &str) -> Option<&mut WotFollow> {
        let source = self.nodes.iter_mut().find(|n| n.pubkey == source_pubkey)?;
        source.get_follow_mut(target_pubkey)
    }

    /**
     * Find node by pubkey
     */
    pub fn get_node(&self, pubkey: &str) -> Option<&WotNode> {
        WotNode::binary_search(pubkey, &self.nodes)
    }

    /**
     * Returns the me node. Panics if not found.
     */
    pub fn get_me_node(&self) -> &WotNode {
        let me = self.get_node("me");
        match me {
            None => panic!("Me node is missing in this graph."),
            Some(node) => node
        }
    }

    pub fn get_classes(&self) -> Vec<&WotNode> {
        let result: Vec<&WotNode> = self.nodes.iter().filter(|n| {
            if let WotNodeType::WotClass = n.typ {
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

    /**
     * Layers of WotNodes. Last: WotClass(es)
     */
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
            if current_layer.len() == 0 {
                panic!("Can't create layers of a graph with cycles. Prune cycles first.");
            };
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


    pub fn prune_cycles(&self) {
        // fn dfs<'a>(current: &'a WotNode, end: &WotNode, visited: & mut Vec<&'a WotNode>, current_path: &mut Vec<&'a WotNode>, all_paths: &mut Vec<Vec<&'a WotNode>>, graph: &'a WotGraph) {
        //     visited.insert(0, current);
        //     if current.pubkey == end.pubkey {
        //         all_paths.push(current_path.to_vec());
        //     } else {
        //         if let Some(follows) = current.get_follows() {
        //             for follow in follows {
        //                 let target_node = graph.get_node(&follow.target_pubkey).unwrap();
        //                 let has_already_been_visited = visited.contains(&target_node);
        //                 if !has_already_been_visited {
        //                     current_path.push(target_node);
        //                 }
        //             }
        //         };
        //     }
        //     visited.


        // }

        // let me = self.get_me_node();
        // let classes = self.get_classes();
        // for class in classes {
        //     let mut paths: Vec<Vec<&WotNode>> = vec![];
        //     let mut visited: Vec<String> = vec![];
        //     let mut stack: Vec<String> = vec![];
        //     stack.push(class.pubkey.clone());
        //     while stack.len() > 0 {
        //         let current = stack.pop().unwrap();
        //         let current_node = self.get_node(&current).unwrap();
        //         visited.push(current);
        //         if let Some(follows) = current_node.get_follows() {
        //             for follow in follows {
        //                 let has_already_been_visited = visited.contains(&follow.target_pubkey);
        //                 if !has_already_been_visited {
        //                     stack.push(follow.target_pubkey.clone());
        //                 }
        //             }
        //         };
        //     }




        // };

    }
}

impl From<WotPredictor> for WotGraph {
    fn from(value: WotPredictor) -> Self {
        value.graph
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
            alias: String::from("example.com1"),
            typ: WotNodeType::WotClass,
        });
        nodes.push(WotNode {
            pubkey: "d2".to_string(),
            alias: String::from("example.com2"),
            typ: WotNodeType::WotClass,
        });

        nodes.push(WotNode {
            pubkey: "n2".to_string(),
            alias: "".to_string(),
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow::new("n2".to_string(), "d1".to_string(), 1.0).unwrap(),
                    WotFollow::new("n2".to_string(), "d2".to_string(), -1.0).unwrap(),
                    WotFollow::new("n2".to_string(), "me".to_string(), -1.0).unwrap(),
                ],
            },
        });

        nodes.push(WotNode {
            pubkey: "n1".to_string(),
            alias: "".to_string(),
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow::new("n1".to_string(), "d1".to_string(), -0.5).unwrap(),
                    WotFollow::new("n1".to_string(), "d2".to_string(), 0.0).unwrap()
                ],
            },
        });

        nodes.push(WotNode {
            pubkey: "me".to_string(),
            alias: "".to_string(),
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
            alias: String::from("example.com1"),
            typ: WotNodeType::WotClass,
        });
        nodes.push(WotNode {
            pubkey: "d1".to_string(),
            alias: String::from("example.com2"),
            typ: WotNodeType::WotClass,
        });
        let result = WotGraph::new(nodes);
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

    #[test]
    fn display() {
        let graph = get_simple_graph();
        println!("{}", graph)
    }
}
