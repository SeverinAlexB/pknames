use std::collections::HashSet;

use crate::prediction::{graph::WotGraph, node::WotFollow};

/**
 * If somebody claims a class it can't contribute to the result otherwise.
 * Prunes all regular follows of these nodes.
 */
pub fn prune_class_follows(mut graph: WotGraph) -> WotGraph {
    // Remove all other follows of nodes that attribute.
    let pubkeys_that_get_attributed: HashSet<String> = graph.get_follows().into_iter().filter(|follow| follow.attribution.is_some()).map(|follow| follow.target_pubkey.clone()).collect();

    for pubkey in pubkeys_that_get_attributed {
        let node = graph.get_node_mut(&pubkey).unwrap();
        node.follows.retain(|follow| follow.attribution.is_some());
    }
    graph
}


/**
 * If you attribute a class, you can't be attributed the same class.
 * This should make `prune_class_follows` ungameable.
 */
pub fn prune_attribution_chains(mut graph: WotGraph) -> WotGraph {
    let attribution_follows: Vec<&WotFollow> = graph.get_follows().into_iter().filter(|follow| {
        follow.attribution.is_some()
    }).collect();

    let attributor: HashSet<String> = attribution_follows.iter().map(|follow| follow.source_pubkey.clone()).collect();
    let invalid_follows: Vec<WotFollow> = attribution_follows.into_iter().filter(|follow| attributor.contains(&follow.target_pubkey)).map(|f| f.clone()).collect();
    for follow in invalid_follows {
        graph.remove_follow(&follow);
    };
    graph
}



#[cfg(test)]
mod tests {
    use crate::{prediction::{node::{WotNode, WotFollow}, graph::WotGraph}, pruning::prune_class_follows::prune_class_follows};

    use super::prune_attribution_chains;

    /**
     * Constructs a simple graph
     */
    fn get_graph() -> WotGraph {
        let mut nodes: Vec<WotNode> = Vec::new();

        // Classes
        nodes.push(WotNode {
            pubkey: "evilexample".to_string(),
            alias: String::from("example.com1"),
            follows: vec![]
        });
        nodes.push(WotNode {
            pubkey: "goodexample".to_string(),
            alias: String::from("example.com2"),
            follows: vec![]
        });

        nodes.push(WotNode {
            pubkey: "registrar".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("registrar", "goodexample", 1.0, Some("example.com")),
            ]
        });

        nodes.push(WotNode {
            pubkey: "eve".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("eve", "registrar", -1.0, Some("example.com")),
                WotFollow::new("eve", "evilexample", 1.0, Some("example.com")),
                WotFollow::new("eve", "goodexample", -1.0, Some("example.com")),
            ]
        });


        nodes.push(WotNode {
            pubkey: "me".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("me", "eve", 1.0, None),
                WotFollow::new("me", "registrar", 0.5, None)
            ]
        });

        WotGraph::new(nodes)
    }

    #[test]
    fn prune_registrars_other_follows() {
        let graph = get_graph();
        let graph = prune_class_follows(graph);
        let registrar = graph.get_node("registrar").unwrap();
        assert_eq!(registrar.follows.len(), 1);
        assert!(registrar.get_follow("goodexample").is_some());

        let me = graph.get_node("me").unwrap();
        assert_eq!(me.follows.len(), 2);

    }

    #[test]
    fn remove_eves_attribution_chain() {
        let graph = get_graph();
        let graph = prune_attribution_chains(graph);
        let eve = graph.get_node("eve").unwrap();
        assert_eq!(eve.follows.len(), 2);
        assert!(eve.get_follow("evilexample").is_some());
        assert!(eve.get_follow("registrar").is_none());
    }

}
