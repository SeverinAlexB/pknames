use std::{collections::HashSet, fmt};

use super::{graph::WotGraph, node::{WotNode, WotFollow}};

pub struct AllPathSearch<'a> {
    graph: &'a WotGraph,
    current_paths: Vec<Vec<&'a WotNode>>,
    start: &'a WotNode,
    visited: Vec<String>
}

impl<'a> AllPathSearch<'a> {

    pub fn new(graph: &'a WotGraph) -> Self {
        AllPathSearch{
            graph,
            current_paths: vec![],
            start: graph.get_me_node(),
            visited: vec![]
        }
    }

    fn dfs(&mut self, current: &'a WotNode, current_path: & mut Vec<&'a WotNode>, end: &'a WotNode) {
        // DFS traversal https://www.geeksforgeeks.org/depth-first-search-or-dfs-for-a-graph/
        self.visited.insert(0, current.pubkey.clone());
        if current.pubkey == end.pubkey {
            self.current_paths.push(current_path.to_vec());
        } else {
            if let Some(follows) = current.get_follows() {
                for follow in follows {
                    let target_node = self.graph.get_node(&follow.target_pubkey).unwrap();
                    let has_been_visited = self.visited.contains(&target_node.pubkey);
                    if !has_been_visited {
                        current_path.push(target_node);
                        self.dfs(target_node, current_path, end);
                        current_path.pop();
                    }
                }
            };
        }
        let visited_index = self.visited.iter().position(|x| *x == current.pubkey).unwrap();
        self.visited.remove(visited_index);
    }

    pub fn run(&mut self) -> Vec<Vec<&'a WotNode>> {
        let classes = self.graph.get_classes();
        for class in classes.iter() {
            let mut current_path = vec![self.start];
            self.dfs(self.start, &mut current_path, class);
        }
        self.current_paths.clone()
    }

    pub fn get_found_nodes(&self) -> HashSet<&WotNode> {
        let set: HashSet<&WotNode> = self.current_paths.iter().map(|path| {
            let new_path: Vec<&WotNode> = path.iter().map(|node| &**node).collect();
            new_path
        }).flatten().collect();
        set
        
    }

    pub fn get_missing_nodes(&self) -> HashSet<&WotNode> {
        let found_nodes = self.get_found_nodes();

        let all_nodes = self.graph.get_nodes();
        let diff = HashSet::from_iter(all_nodes.difference(&found_nodes).into_iter().map(|node| &**node));
        diff
    }

    pub fn get_found_follows(&self) -> HashSet<&WotFollow> {
        let mut follows: HashSet<&WotFollow> = HashSet::new();
        for path in self.current_paths.iter() {
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
    
}


impl fmt::Display for AllPathSearch<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strs: Vec<String> = self.current_paths.iter().map(|path| {
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

    use super::super::node::{WotNode, WotNodeType, WotFollow};
    use super::{WotGraph, AllPathSearch};

    /**
     * Constructs a simple graph
     */
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
                    WotFollow::new("n2".to_string(), "n3".to_string(), -1.0).unwrap(),
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
                    WotFollow::new("n1".to_string(), "me".to_string(), 0.0).unwrap()
                ],
            },
        });

        nodes.push(WotNode {
            pubkey: "n3".to_string(),
            alias: None,
            typ: WotNodeType::WotFollowNode {
                follows: vec![
                    WotFollow::new("n3".to_string(), "me".to_string(), -0.5).unwrap(),
                ],
            },
        });

        nodes.push(WotNode {
            pubkey: "me".to_string(),
            alias: None,
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
    fn run() {
        let graph = get_simple_graph();
        let mut search = AllPathSearch::new(&graph);
        let result = search.run();
        println!("{}", search);
        assert_eq!(result.len(), 4);
        
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
        assert!(found.contains(&WotFollow::new("me".to_string(), "n1".to_string(), 0.0).unwrap()));
        assert!(found.contains(&WotFollow::new("me".to_string(), "n2".to_string(), 0.0).unwrap()));
        assert!(found.contains(&WotFollow::new("n1".to_string(), "d1".to_string(), 0.0).unwrap()));
        assert!(found.contains(&WotFollow::new("n1".to_string(), "d2".to_string(), 0.0).unwrap()));
        assert!(found.contains(&WotFollow::new("n2".to_string(), "d1".to_string(), 0.0).unwrap()));
        assert!(found.contains(&WotFollow::new("n2".to_string(), "d2".to_string(), 0.0).unwrap()));

        let missing = search.get_missing_follows();
        assert_eq!(missing.len(), 3);
        assert!(missing.contains(&WotFollow::new("n2".to_string(), "n3".to_string(), 0.0).unwrap()));
        assert!(missing.contains(&WotFollow::new("n3".to_string(), "me".to_string(), 0.0).unwrap()));
        assert!(missing.contains(&WotFollow::new("n1".to_string(), "me".to_string(), 0.0).unwrap()));
    }

}
