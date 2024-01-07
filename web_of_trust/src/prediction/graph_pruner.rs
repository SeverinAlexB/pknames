use core::panic;
use std::{collections::{HashSet, HashMap}, fmt};

use super::{graph::WotGraph, node::{WotNode, WotFollow}};

/**
 * Turns the possibly cyclical Web of Trust graph into an acyclical graph and prunes unnecesarry nodes.
 * This is needed to do any calculation.
 * Intuitively, it removes all follows that point back towards the me node. 
 * It also removes all nodes that do not contribute to the classes.
 * 
 * Todo: Research if this type of pruning cycles can be abused to influence the web of trust.
 * https://github.com/zhenv5/breaking_cycles_in_noisy_hierarchies
 * 
 * Note to myself: Refactoring required.
 */
pub struct GraphPruner<'a> {
    graph: &'a WotGraph,
    found_paths: Vec<Vec<&'a WotNode>>,
    pruned_cycle_follows: Vec<& 'a WotFollow>,
    visited: Vec<String>,
    me_pubkey: String
}



impl<'a> GraphPruner<'a> {
    fn new(graph: &'a WotGraph, me_pubkey: String) -> Self {
        if let None = graph.get_node(&me_pubkey) {
            panic!("me_pubkey not found in graph");
        };

        GraphPruner{
            graph,
            found_paths: vec![],
            pruned_cycle_follows: vec![],
            visited: vec![],
            me_pubkey
        }
    }

    fn get_start_node(&self)-> &'a WotNode {
        self.graph.get_node(&self.me_pubkey).expect("me_pubkey must be in graph")
    }

    fn dfs(&mut self, current: &'a WotNode, current_path: & mut Vec<&'a WotNode>, end: &'a WotNode) {
        // DFS traversal https://www.geeksforgeeks.org/depth-first-search-or-dfs-for-a-graph/
        self.visited.insert(0, current.pubkey.clone());
        if current.pubkey == end.pubkey {
            self.found_paths.push(current_path.to_vec());
        } else {
            for follow in current.follows.iter() {
                let is_pruned = self.pruned_cycle_follows.contains(&follow);
                if is_pruned {
                    continue;
                };

                let target_node_option = self.graph.get_node(&follow.target_pubkey);
                if let None = target_node_option {
                    // We don't have any data about the target node. Skip
                    continue;
                }
                let target_node = self.graph.get_node(&follow.target_pubkey).unwrap();
                let is_new_cyle = current_path.contains(&target_node);
                if is_new_cyle {
                    self.pruned_cycle_follows.push(follow);
                };

                let has_been_visited = self.visited.contains(&target_node.pubkey);
                if !has_been_visited {
                    current_path.push(target_node);
                    self.dfs(target_node, current_path, end);
                    current_path.pop();
                }
            };
        }
        let visited_index = self.visited.iter().position(|x| *x == current.pubkey).unwrap();
        self.visited.remove(visited_index);
    }

    /**
     * Prunes all attributions that are not equal to `attribution`
     */
    fn prune_unnecessary_attributions(mut graph: WotGraph, attribution: &str) -> WotGraph {
        for node in graph.nodes.iter_mut() {
            let selected_follows: Vec<WotFollow> = node.follows.clone().into_iter().filter(|follow| {
                match follow.attribution.clone() {
                    None => true,
                    Some(att) => att == attribution
                }
            }).collect();
            node.follows = selected_follows;
        }
        graph
    }

    fn search_paths(&mut self) -> Vec<Vec<&'a WotNode>> {
        let start = self.get_start_node();
        let classes = self.graph.get_classes();
        for class in classes.iter() {
            let mut current_path = vec![start];
            self.dfs(start, &mut current_path, class);
        }
        self.found_paths.clone()
    }

    pub fn get_found_nodes(&self) -> HashSet<&WotNode> {
        let set: HashSet<&WotNode> = self.found_paths.iter().map(|path| {
            let new_path: Vec<&WotNode> = path.iter().map(|node| &**node).collect();
            new_path
        }).flatten().collect();
        set
        
    }

    pub fn get_missing_nodes(&self) -> HashSet<&WotNode> {
        let found_nodes = self.get_found_nodes();

        let all_nodes = self.graph.get_nodes();
        let diff = HashSet::from_iter(all_nodes.difference(&found_nodes).into_iter().map(|node| *node));
        diff
    }

    pub fn get_found_follows(&self) -> HashSet<&WotFollow> {
        let mut follows: HashSet<&WotFollow> = HashSet::new();
        for path in self.found_paths.iter() {
            for i in 0..path.len()-1 {
                let source = path[i];
                let target = path[i+1];
                let follow = self.graph.get_follow(&source.pubkey, &target.pubkey).unwrap();
                follows.insert(follow);
            };
        };
        follows
    }

    pub fn get_missing_follows(&self) -> HashSet<&WotFollow> {
        let found = self.get_found_follows();
        let all = self.graph.get_follows();
        let diff = HashSet::from_iter(all.difference(&found).into_iter().map(|follow| &**follow));
        diff
    }

    pub fn clone_and_prune(&mut self) -> WotGraph {
        let follows = self.get_found_follows();
        let mut map: HashMap<String, Vec<WotFollow>> = HashMap::new();

        for follow in follows.iter() {
            if !map.contains_key(&follow.source_pubkey) {
                map.insert(follow.source_pubkey.clone(), vec![]);
            }
            let cloned_follow = (*follow).clone();
            let list = map.get_mut(&follow.source_pubkey).unwrap();
            list.push(cloned_follow);
        }

        let nodes: Vec<WotNode> = self.get_found_nodes().iter().map(|original| {
            let mut node = (*original).clone();
            if let Some(list) = map.get(&node.pubkey) {
                let follows = list.clone();
                node.follows.clear();
                node.extend_follows(follows);
            }
            node
        }).collect();

        WotGraph::new(nodes)
    }

    pub fn prune(graph: WotGraph, attribution: &str, me_pubkey: &str) -> WotGraph {
        let graph = Self::prune_unnecessary_attributions(graph, attribution);
        let mut pruner = GraphPruner::new(&graph, me_pubkey.to_string());
        pruner.search_paths();
        let pruned = pruner.clone_and_prune();
        pruned
    }
    
}


impl fmt::Display for GraphPruner<'_> {
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
    use std::collections::HashSet;

    use super::super::node::{WotNode, WotFollow};
    use super::{WotGraph, GraphPruner};

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
                WotFollow::new("n4".to_string(), "n2".to_string(), -0.5, Some("apple.com".to_string())),
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
    fn search() {
        let graph = get_simple_graph();
        let mut search = GraphPruner::new(&graph, "me".to_string());
        let result = search.search_paths();
        println!("{}", search);
        assert_eq!(
            result.len(), 4);
        
        let found: HashSet<String> = search.get_found_nodes().iter().map(|n| n.pubkey.clone()).collect();
        assert_eq!(found.len(), 5);
        assert!(found.contains("me"));
        assert!(found.contains("n1"));
        assert!(found.contains("n2"));
        assert!(found.contains("d1"));
        assert!(found.contains("d2"));

        let missing: HashSet<String> = search.get_missing_nodes().iter().map(|n| n.pubkey.clone()).collect();
        assert_eq!(missing.len(), 1);
        assert!(missing.contains("n3"));

        let found = search.get_found_follows();
        assert_eq!(found.len(), 6);
        assert!(found.contains(&WotFollow::new("me".to_string(), "n1".to_string(), 0.0, None)));
        assert!(found.contains(&WotFollow::new("me".to_string(), "n2".to_string(), 0.0, None)));
        assert!(found.contains(&WotFollow::new("n1".to_string(), "d1".to_string(), 0.0, Some("example.com".to_string()))));
        assert!(found.contains(&WotFollow::new("n1".to_string(), "d2".to_string(), 0.0, Some("example.com".to_string()))));
        assert!(found.contains(&WotFollow::new("n2".to_string(), "d1".to_string(), 0.0, Some("example.com".to_string()))));
        assert!(found.contains(&WotFollow::new("n2".to_string(), "d2".to_string(), 0.0, Some("example.com".to_string()))));

        let missing = search.get_missing_follows();
        assert_eq!(missing.len(), 3);
        assert!(missing.contains(&WotFollow::new("n2".to_string(), "n3".to_string(), 0.0, None)));
        assert!(missing.contains(&WotFollow::new("n3".to_string(), "me".to_string(), 0.0, None)));
        assert!(missing.contains(&WotFollow::new("n1".to_string(), "me".to_string(), 0.0, None)));
    }

    #[test]
    fn prune() {
        let graph = get_simple_graph();
        let pruned = GraphPruner::prune(graph, "example.com", "me");
        assert_eq!(pruned.nodes.len(), 5);
    }

    #[test]
    fn search_critical_graph() {
        let graph = get_complex_graph();
        let graph = GraphPruner::prune_unnecessary_attributions(graph, "example.com");
        // let pruned1 = GraphPruner::prune(&graph);
        let mut search = GraphPruner::new(&graph, "me".to_string());
        search.search_paths();
        println!("{}", search);

        
        // let found: HashSet<String> = search.get_found_nodes().iter().map(|n| n.pubkey.clone()).collect();
        // assert_eq!(found.len(), 5);
        // assert!(found.contains("me"));
        // assert!(found.contains("n1"));
        // assert!(found.contains("n2"));
        // assert!(found.contains("d1"));
        // assert!(found.contains("d2"));

        // let missing: HashSet<String> = search.get_missing_nodes().iter().map(|n| n.pubkey.clone()).collect();
        // assert_eq!(missing.len(), 1);
        // assert!(missing.contains("n3"));

        // let found = search.get_found_follows();
        // assert_eq!(found.len(), 6);
        // assert!(found.contains(&WotFollow::new("me".to_string(), "n1".to_string(), 0.0).unwrap()));
        // assert!(found.contains(&WotFollow::new("me".to_string(), "n2".to_string(), 0.0).unwrap()));
        // assert!(found.contains(&WotFollow::new("n1".to_string(), "d1".to_string(), 0.0).unwrap()));
        // assert!(found.contains(&WotFollow::new("n1".to_string(), "d2".to_string(), 0.0).unwrap()));
        // assert!(found.contains(&WotFollow::new("n2".to_string(), "d1".to_string(), 0.0).unwrap()));
        // assert!(found.contains(&WotFollow::new("n2".to_string(), "d2".to_string(), 0.0).unwrap()));

        // let missing = search.get_missing_follows();
        // assert_eq!(missing.len(), 3);
        // assert!(missing.contains(&WotFollow::new("n2".to_string(), "n3".to_string(), 0.0).unwrap()));
        // assert!(missing.contains(&WotFollow::new("n3".to_string(), "me".to_string(), 0.0).unwrap()));
        // assert!(missing.contains(&WotFollow::new("n1".to_string(), "me".to_string(), 0.0).unwrap()));
    }

}
