use std::path::PathBuf;
use clap::ArgMatches;
use fancyd_wot::{visualization::visualization::visualize_graph, prediction::graph_pruner::GraphPruner};
use crate::cli::{config_directory::main_directory::{MainDirectory, self}, wot_transformer::{WotTransformer, follow_lists_into_wot_graph}};


pub fn cli_ls(matches: &ArgMatches, folder_path: PathBuf, verbose: bool) {
    let dir = MainDirectory::new(folder_path);
    dir.create_if_it_does_not_exist().unwrap();

    let lists = dir.static_lists_dir.read_lists().expect("Readable directory");
    if lists.len() == 0 {
        println!("No lists found in \"{}\".", dir.static_lists_dir.path.to_str().unwrap());
        return;
    };

    for list in lists.iter() {
        match list {
            Ok(list) => println!("{}", list),
            Err(e) => {
                println!("{}", e);
            }
        }
        println!("");
    }

    let default_value = "".to_string();
    let domain: &String = matches.get_one("domain").unwrap_or(&default_value);

    // println!("domain {}", val);

    let show_gui: bool = *matches.get_one("ui").unwrap();
    if show_gui {
        let lists = dir.static_lists_dir.read_valid_lists();
        let mut graph = follow_lists_into_wot_graph(lists);
        if domain.len() > 0 {
            println!("Prune graph for domain {}", domain);
            graph = GraphPruner::prune(graph, domain, &dir.get_zbase32_public_key());
        }
        visualize_graph(graph, "fancy-cli ls", None);
    }
}