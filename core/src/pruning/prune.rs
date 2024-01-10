use crate::prediction::graph::WotGraph;

use super::{prune_undesired_attributions::prune_undesired_attributions, prune_useless_nodes::UselessNodePruner, prune_cycles::CyclePruner, prune_class_follows::{prune_class_follows, prune_attribution_chains}};


/**
 * Turns the possibly cyclical Web of Trust graph into an acyclical graph and prunes unnecesarry nodes.
 * This is needed to do any calculation.
 */
pub fn prune_graph(graph: WotGraph, me_pubkey: &str, desired_attribution: &str) -> WotGraph {
    let graph = prune_undesired_attributions(graph, desired_attribution);
    let graph = UselessNodePruner::prune(graph, me_pubkey);
    let graph = CyclePruner::prune(graph, me_pubkey);
    let graph = prune_attribution_chains(graph);
    let graph = prune_class_follows(graph);
    let graph = UselessNodePruner::prune(graph, me_pubkey);
    graph
}


#[cfg(test)]
mod tests {
    use crate::{prediction::node::{WotNode, WotFollow}, pruning::prune::prune_graph};
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
                WotFollow::new("n2", "d1", 1.0, Some("example.com")),
                WotFollow::new("n2", "d2", -1.0, Some("example.com")),
                WotFollow::new("n2", "n3", -1.0, None),
            ]
        });

        nodes.push(WotNode {
            pubkey: "n1".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n1", "d1", -0.5, Some("example.com")),
                WotFollow::new("n1", "d2", 0.0, Some("example.com")),
                WotFollow::new("n1", "me", 0.0, None)
            ]
        });

        nodes.push(WotNode {
            pubkey: "n3".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("n3", "me", -0.5, None),
            ]
        });

        nodes.push(WotNode {
            pubkey: "me".to_string(),
            alias: "".to_string(),
            follows: vec![
                WotFollow::new("me", "n1", 1.0, None),
                WotFollow::new("me", "n2", 0.5, None)
            ]
        });

        WotGraph::new(nodes)
    }


    #[test]
    fn prune() {
        let graph = get_simple_graph();
        let pruned = prune_graph(graph, "me", "example.com");
        assert_eq!(pruned.nodes.len(), 5);
    }


}
