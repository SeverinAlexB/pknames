use crate::{prediction::{predictor::{WotPredictor, WotPrediction}, graph::WotGraph}, pruning::prune::prune_graph, config_directory::dirs::main_directory::MainDirectory};



/**
 * Resolves a domain name to a pkarr uri.
 * Standalone function, no service needed.
 */
pub fn resolve_standalone(domain: &str, directory: &str) -> Result<WotPrediction, String> {
    let dir = MainDirectory::new_by_string(directory);
    dir.create_if_it_does_not_exist().unwrap();

    let lists = dir.static_lists_dir.read_valid_lists();
    if lists.len() == 0 {
        return Err("Graph does not contain the domain.".to_string())
    };

    let graph: WotGraph = lists.into();
    if !graph.contains_attribution(domain) {
        return Err("Graph does not contain the domain.".to_string())
    };
    let graph = prune_graph(graph, dir.get_public_key_uri().as_str(), domain);

    let predictor: WotPredictor = graph.clone().into();
    Ok(predictor.predict())
}