use cli::cli::run_cli;

mod web_of_trust;
mod cli;

fn main() {
    run_cli();
}



// fn main() {
//     let native_options = eframe::NativeOptions::default();
//     run_native(
//         "egui_graphs_interactive_demo",
//         native_options,
//         Box::new(|cc| Box::new(InteractiveApp::new(cc))),
//     )
//     .unwrap();
// }