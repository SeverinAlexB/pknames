use std::{path::PathBuf, process::ExitCode};

use clap::ArgMatches;
use pknames_wot::{prediction::predictor::WotPredictor,  visualization::visualization::visualize_graph, pruning::prune::prune_graph};

use crate::cli::{config_directory::main_directory::MainDirectory, wot_transformer::follow_lists_into_wot_graph};

pub fn cli_lookup(matches: &ArgMatches, directory: PathBuf, verbose: bool) {
    let domain: &String = matches.get_one("domain").unwrap();
    println!("Lookup {}", domain);

    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();

    let lists = dir.static_lists_dir.read_valid_lists();
    if lists.len() == 0 {
        println!("No lists found in \"{}\".", dir.static_lists_dir.path.to_str().unwrap());
        std::process::exit(1);
    };

    let graph = follow_lists_into_wot_graph(lists);

    if !graph.contains_attribution(domain) {
        println!("Graph does not contain the domain.");
        std::process::exit(1);
    };

    let public_key = format!("{}", dir.get_zbase32_public_key());
    let graph = prune_graph(graph, public_key.as_str(), domain);

    let predictor: WotPredictor = graph.clone().into();
    let result = predictor.predict();

    for class in graph.get_classes() {
        let val = result.get_value(&class.pubkey);
        if val.is_some() {
            println!("- {} {:.2}%", class.pubkey, val.unwrap()*100.0);
        };
    };

    let show_gui: bool = *matches.get_one("ui").unwrap();
    if show_gui {
        visualize_graph(graph, "Lookup domain", Some(&dir.get_zbase32_public_key()), Some(result));
    };




}