mod view;

// Subsystems
mod controller;
mod config;
mod document_model;

use controller::EditorController;
use config::RcLoader;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Load RC configuration
    let config = RcLoader::load_config();

    // Use the new modular EditorController for testing
    let mut controller = if args.len() > 1 {
        let filenames: Vec<PathBuf> = args[1..].iter().map(PathBuf::from).collect();
        EditorController::new_with_files(filenames)?
    } else {
        EditorController::new()
    };

    // Apply RC configuration to the controller
    controller.apply_config(&config);

    controller.run()
}
