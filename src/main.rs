mod command;
mod document;
mod help;
mod key_handler;
mod marks;
mod movement;
mod piece_table;
mod rc;
mod registers;
mod search;
mod text_buffer;
mod undo;
mod view;
mod visual_mode;
mod yank_paste_handler;

// New modular architecture
mod mode_controllers;
mod insert_controller;
mod normal_controller;
mod visual_controller;
mod command_controller;
mod editor_controller;

use editor_controller::EditorController;
use rc::RcLoader;
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
