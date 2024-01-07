use crate::prediction::{graph::WotGraph, node::WotFollow};

/**
 * Prunes all attributions that are not equal to `desired_attribution`.
 */
pub fn prune_undesired_attributions(mut graph: WotGraph, desired_attribution: &str) -> WotGraph {
    for node in graph.nodes.iter_mut() {
        let selected_follows: Vec<WotFollow> = node
            .follows
            .clone()
            .into_iter()
            .filter(|follow| match follow.attribution.clone() {
                None => true,
                Some(att) => att == desired_attribution,
            })
            .collect();
        node.follows = selected_follows;
    }
    graph
}
