use std::path::PathBuf;

use clap::ArgMatches;
use fancyd_wot::{prediction::{predictor::WotPredictor}, pruning::graph_pruner::GraphPruner, visualization::visualization::visualize_graph};

use crate::cli::{config_directory::main_directory::MainDirectory, wot_transformer::follow_lists_into_wot_graph};

pub fn cli_lookup(matches: &ArgMatches, directory: PathBuf, verbose: bool) {
    let domain: &String = matches.get_one("domain").unwrap();
    println!("Lookup {}", domain);

    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();

    let lists = dir.static_lists_dir.read_valid_lists();
    if lists.len() == 0 {
        println!("No lists found in \"{}\".", dir.static_lists_dir.path.to_str().unwrap());
        return;
    };

    let graph = follow_lists_into_wot_graph(lists);

    let public_key = format!("{}", dir.get_zbase32_public_key());
    let graph = GraphPruner::prune(graph, domain, public_key.as_str());
    println!("Graph pruned {}", graph);

    let predictor: WotPredictor = graph.clone().into();
    let result = predictor.predict();

    println!("Result {}", result);
    let show_gui: bool = *matches.get_one("ui").unwrap();
    if show_gui {
        visualize_graph(graph, "Lookup domain", Some(result));
    };




}