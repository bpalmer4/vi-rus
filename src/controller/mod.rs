/// Controller subsystem - Handles all user input and command execution
/// 
/// This module contains all the mode-specific controllers and command processors,
/// providing a clean separation between user interaction logic and data/view layers.

pub mod editor;
pub mod normal;
pub mod insert;
pub mod visual;
pub mod command;
pub mod shared_state;
pub mod help;
pub mod command_types;
pub mod key_handler;
pub mod visual_mode;
pub mod yank_paste;
pub mod search_commands;

// Re-export public interface
pub use editor::EditorController;
pub use shared_state::SharedEditorState;
pub use command_types::Mode;
pub use visual_mode::Selection;