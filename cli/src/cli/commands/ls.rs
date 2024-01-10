use std::path::PathBuf;
use clap::ArgMatches;
use pknames_core::{ visualization::visualization::visualize_graph, pruning::prune::prune_graph, config_directory::{dirs::main_directory::MainDirectory}, prediction::graph::WotGraph};



pub fn cli_ls(matches: &ArgMatches, folder_path: PathBuf, verbose: bool) {
    let dir = MainDirectory::new(folder_path);
    dir.create_if_it_does_not_exist().unwrap();

    let lists = dir.static_lists_dir.read_lists().expect("Readable directory");
    if lists.len() == 0 {
        eprintln!("No lists found in \"{}\".", dir.static_lists_dir.path.to_str().unwrap());
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

    let show_gui: bool = *matches.get_one("ui").unwrap();
    if show_gui {
        let lists = dir.static_lists_dir.read_valid_lists();
        let mut graph: WotGraph = lists.into();
        if domain.len() > 0 {
            println!("Prune graph for domain {}", domain);
            graph = prune_graph(graph, &dir.get_public_key_uri(), domain);
        }
        visualize_graph(graph, "pknamescli ls", Some(&dir.get_public_key_uri()), None);
    }
}