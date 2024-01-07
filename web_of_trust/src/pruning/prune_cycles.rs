use core::panic;
use std::{fmt, collections::HashSet};
use crate::prediction::{graph::WotGraph, node::{WotNode, WotFollow}};


/**
 * Prunes the graph from cycles. This is a very simple approach and has attack vectors.
 * Example: By following certain nodes you can artifically create cycles and because this pruning is not smart, it might prune the wrong follow.
 * TODO: Needs to be hardened in the future.
 * https://github.com/zhenv5/breaking_cycles_in_noisy_hierarchies
 */

struct DfsResult<'a> {
    pruned_cycle_follows: HashSet<& 'a WotFollow>,
    visited: Vec<String>,
    current: &'a WotNode, 
    current_path: Vec<&'a WotNode>, 
    found_paths: Vec<Vec<&'a WotNode>>,
    end: &'a WotNode
}

impl<'a> DfsResult<'a> {
    pub fn new(start: &'a WotNode, end: &'a WotNode, pruned_follows: HashSet<& 'a WotFollow>) -> Self {
        DfsResult {
            pruned_cycle_follows: pruned_follows,
            visited: vec![],
            current: start,
            current_path: vec![start],
            found_paths: vec![],
            end
        }
    }
}

pub struct CyclePruner<'a> {
    graph: &'a WotGraph,
    me_pubkey: String
}

impl<'a> CyclePruner<'a> {
    fn new(graph: &'a WotGraph, me_pubkey: &str) -> Self {
        if let None = graph.get_node(me_pubkey) {
            panic!("me_pubkey not found in graph");
        };

        CyclePruner{
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
                let is_pruned = result.pruned_cycle_follows.contains(&follow);
                if is_pruned {
                    continue;
                };

                let target_node_option = self.graph.get_node(&follow.target_pubkey);
                if target_node_option.is_none() {
                    // We don't have any data about the target node. Skip
                    continue;
                }
                let target_node = target_node_option.unwrap();
                let is_new_cyle = result.current_path.contains(&target_node);
                if is_new_cyle {
                    result.pruned_cycle_follows.insert(follow);
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

    fn search_cycles(&mut self) -> HashSet<&'a WotFollow> {
        let start = self.get_start_node();
        let classes = self.graph.get_classes();
        let mut pruned_follows: HashSet<& WotFollow> = HashSet::new();
        for class in classes.iter() {
            let result = DfsResult::new(start, class, pruned_follows.clone());
            let result = self.dfs(result);
            pruned_follows.extend(result.pruned_cycle_follows.iter());
        }
        pruned_follows
    }

    pub fn prune(mut graph: WotGraph, me_pubkey: &str) -> WotGraph {
        let mut pruner = CyclePruner::new(&graph, me_pubkey);
        let cycles = pruner.search_cycles();
        let cyles2: HashSet<WotFollow> = cycles.into_iter().map(|follow| follow.clone()).collect();
        for follow in cyles2.into_iter() {
            graph.remove_follow(&follow);
        };
        graph
    }

    pub fn find_cycles<'b>(graph: &'b WotGraph, me_pubkey: &str) -> HashSet<&'b WotFollow> {
        let mut pruner = CyclePruner::new(&graph, me_pubkey);
        let cycles = pruner.search_cycles();
        cycles
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
    use super::{WotGraph, CyclePruner};

    /**
     * Constructs a simple graph
     */
    fn get_simple_graph() -> WotGraph {
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
            pubkey: "n2".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n2".to_string(), "d1".to_string(), 1.0, Some("example.com".to_string())),
                WotFollow::new("n2".to_string(), "d2".to_string(), -1.0, Some("example.com".to_string())),
                WotFollow::new("n2".to_string(), "n3".to_string(), -1.0, None),
            ]
        });

        nodes.push(WotNode {
            pubkey: "n1".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n1".to_string(), "d1".to_string(), -0.5, Some("example.com".to_string())),
                WotFollow::new("n1".to_string(), "d2".to_string(), 0.0, Some("example.com".to_string())),
                WotFollow::new("n1".to_string(), "me".to_string(), 0.0, None)
            ]
        });

        nodes.push(WotNode {
            pubkey: "n3".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n3".to_string(), "me".to_string(), -0.5, None),
            ]
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
    

    fn get_complex_graph() -> WotGraph {
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
                WotFollow::new("n1".to_string(), "n4".to_string(), -0.5, None),
            ]
        });

        nodes.push(WotNode {
            pubkey: "n2".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n2".to_string(), "n3".to_string(), 1.0, None),
            ]
        });

        nodes.push(WotNode {
            pubkey: "n3".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n3".to_string(), "d2".to_string(), -0.5, Some("example.com".to_string())),
                WotFollow::new("n3".to_string(), "n1".to_string(), -0.5, None),
            ]
        });

        nodes.push(WotNode {
            pubkey: "n4".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n4".to_string(), "d1".to_string(), -0.5, Some("example.com".to_string())),
                WotFollow::new("n4".to_string(), "n2".to_string(), -0.5, None),
            ]
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
    fn find_cycles_simple() {
        let graph = get_simple_graph();
        let cycles = CyclePruner::find_cycles(&graph, "me");
        assert_eq!(cycles.len(), 2);
        let cycle1 = graph.get_follow("n1", "me").unwrap();
        let cycle2 = graph.get_follow("n3", "me").unwrap();
        assert!(cycles.contains(cycle1));
        assert!(cycles.contains(cycle2));
    }

    #[test]
    fn prune_simple() {
        let graph = get_simple_graph();
        let graph = CyclePruner::prune(graph, "me");
        let cycle1 = graph.get_follow("n1", "me");
        assert!(cycle1.is_none());
        let cycle2 = graph.get_follow("n3", "me");
        assert!(cycle2.is_none());
    }

    #[test]
    fn find_cycles_complex() {
        let graph = get_complex_graph();
        let cycles = CyclePruner::find_cycles(&graph, "me");
        assert_eq!(cycles.len(), 1);
        let cycle = graph.get_follow("n3", "n1").unwrap();
        assert!(cycles.contains(cycle));
    }

}
