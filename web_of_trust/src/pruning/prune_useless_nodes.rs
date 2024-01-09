use core::panic;
use std::{fmt, collections::HashSet};
use crate::prediction::{graph::WotGraph, node::WotNode};


/**
 * Prunes the graph from useless nodes that would not contribute to final result.
 * For example: Nodes that do not follow anybody.
 */

struct DfsResult<'a> {
    visited: Vec<String>,
    current: &'a WotNode, 
    current_path: Vec<&'a WotNode>, 
    found_paths: Vec<Vec<&'a WotNode>>,
    end: &'a WotNode
}

impl<'a> DfsResult<'a> {
    pub fn new(start: &'a WotNode, end: &'a WotNode) -> Self {
        DfsResult {
            visited: vec![],
            current: start,
            current_path: vec![start],
            found_paths: vec![],
            end
        }
    }
}

pub struct UselessNodePruner<'a> {
    graph: &'a WotGraph,
    me_pubkey: String
}

impl<'a> UselessNodePruner<'a> {
    fn new(graph: &'a WotGraph, me_pubkey: &str) -> Self {
        if let None = graph.get_node(me_pubkey) {
            panic!("me_pubkey not found in graph");
        };

        UselessNodePruner{
            graph,
            me_pubkey: me_pubkey.to_string()
        }
    }

    fn get_start_node(&self)-> &'a WotNode {
        self.graph.get_node(&self.me_pubkey).expect("me_pubkey must be in graph")
    }

    fn dfs(&mut self, mut result: DfsResult<'a>) -> DfsResult<'a> {
        // DFS traversal https://www.geeksforgeeks.org/depth-first-search-or-dfs-for-a-graph/
        result.visited.insert(0, result.current.pubkey.clone());
        if result.current.pubkey == result.end.pubkey {
            result.found_paths.push(result.current_path.to_vec());
        } else {
            for follow in result.current.follows.iter() {

                let target_node_option = self.graph.get_node(&follow.target_pubkey);
                if target_node_option.is_none() {
                    // We don't have any data about the target node. Skip
                    continue;
                }
                let target_node = target_node_option.unwrap();
                let is_new_cyle = result.current_path.contains(&target_node);
                if is_new_cyle {
                    continue
                    // panic!("Graph includes cycle. Prune cycles first before pruning useless nodes.")

                };

                let has_been_visited = result.visited.contains(&target_node.pubkey);
                if !has_been_visited {
                    result.current_path.push(target_node);
                    let old_current = result.current;
                    result.current = target_node;
                    result = self.dfs(result);
                    result.current = old_current;
                    result.current_path.pop();
                }
            };
        }
        let visited_index = result.visited.iter().position(|x| *x == result.current.pubkey).unwrap();
        result.visited.remove(visited_index);
        result
    }

    fn search_nodes_in_paths(&mut self) -> HashSet<&'a WotNode> {
        let start = self.get_start_node();
        let classes = self.graph.get_classes();
        let mut found_nodes_in_paths: HashSet<& WotNode> = HashSet::new();
        for class in classes.iter() {
            let result = DfsResult::new(start, class);
            let result = self.dfs(result);
            for path in result.found_paths {
                for node in path {
                    found_nodes_in_paths.insert(node);
                }
            };
        }
        found_nodes_in_paths
    }



    pub fn find<'b>(graph: &'b WotGraph, me_pubkey: &str) -> HashSet<&'b WotNode> {
        let mut pruner = UselessNodePruner::new(&graph, me_pubkey);
        let useful_nodes = pruner.search_nodes_in_paths();
        let all_nodes: HashSet<&WotNode> = graph.nodes.iter().collect();
        let useless_nodes: HashSet<&WotNode> = all_nodes.difference(&useful_nodes).map(|n| *n).collect();
        useless_nodes
    }

    pub fn prune(mut graph: WotGraph, me_pubkey: &str) -> WotGraph {
        let useless_node_refs = UselessNodePruner::find(&graph, me_pubkey);

        let useless_nodes: HashSet<WotNode> = useless_node_refs.into_iter().map(|node| node.clone()).collect();
        for node in useless_nodes.iter() {
            graph.remove_node(node);
        };

        let useless_node_ids: HashSet<String> = useless_nodes.into_iter().map(|node| node.pubkey).collect();
        for node in graph.nodes.iter_mut() {
            node.follows.retain(|follow| !useless_node_ids.contains(&follow.target_pubkey));
        };
        graph
    }

}


impl fmt::Display for DfsResult<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strs: Vec<String> = self.found_paths.iter().map(|path| {
            let path_str = path.iter().map(|node| node.pubkey.clone()).collect::<Vec<String>>().join(" -> ");
            path_str
        }).collect();
        let result = strs.join("\n");
        write!(f, "{}", result)
    }
}


#[cfg(test)]
mod tests {
    use crate::prediction::node::{WotNode, WotFollow};
    use super::{WotGraph, UselessNodePruner};


    fn get_graph() -> WotGraph {
        let mut nodes: Vec<WotNode> = Vec::new();

        // Classes
        nodes.push(WotNode {
            pubkey: "d1".to_string(),
            alias: String::from("example.com1"),
            follows: vec![]
        });
        nodes.push(WotNode {
            pubkey: "d2".to_string(),
            alias: String::from("example.com2"),
            follows: vec![]
        });

        nodes.push(WotNode {
            pubkey: "n1".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n1".to_string(), "d1".to_string(), -0.5, Some("example.com".to_string())),
            ]
        });

        nodes.push(WotNode {
            pubkey: "n2".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n2".to_string(), "d2".to_string(), 1.0, Some("example.com".to_string())),
                WotFollow::new("n2".to_string(), "n3".to_string(), 1.0, None),
            ]
        });

        nodes.push(WotNode {
            pubkey: "n3".to_string(),
            alias: "".to_string(),
            follows: vec![]
        });

        nodes.push(WotNode {
            pubkey: "n4".to_string(),
            alias: "".to_string(),
            follows: vec![]
        });

        nodes.push(WotNode {
            pubkey: "me".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("me".to_string(), "n1".to_string(), 1.0, None),
                WotFollow::new("me".to_string(), "n2".to_string(), 0.5, None)
            ]
        });

        WotGraph::new(nodes)
    }
    

    #[test]
    fn find() {
        let graph = get_graph();
        let useless_nodes = UselessNodePruner::find(&graph, "me");
        assert_eq!(useless_nodes.len(), 2);
        let n3 = graph.get_node("n3").unwrap();       
        assert!(useless_nodes.contains(n3));
        let n4 = graph.get_node("n4").unwrap();       
        assert!(useless_nodes.contains(n4));
    }

    #[test]
    fn prune() {
        let graph = get_graph();
        let graph = UselessNodePruner::prune(graph, "me");
        assert!(graph.get_node("n3").is_none());
        assert!(graph.get_node("n4").is_none());
        assert_eq!(graph.get_node("n2").unwrap().follows.len(), 1);
    }
}
