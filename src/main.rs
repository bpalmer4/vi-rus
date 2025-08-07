mod buffer_commands;
mod buffer_manager;
mod controller;
mod document;
mod edit_commands;
mod file_commands;
mod help;
mod key_handler;
mod marks;
mod movement;
mod rc;
mod registers;
mod search;
mod undo;
mod view;
mod visual_mode;
mod yank_paste_handler;

use controller::Controller;
use rc::RcLoader;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Load RC configuration
    let config = RcLoader::load_config();

    let mut controller = if args.len() > 1 {
        let filenames: Vec<PathBuf> = args[1..].iter().map(PathBuf::from).collect();
        Controller::new_with_files(filenames)?
    } else {
        Controller::new()
    };

    // Apply RC configuration to the controller
    RcLoader::apply_config(&mut controller, &config);

    controller.run()
}
