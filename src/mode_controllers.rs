use crate::buffer_manager::BufferManager;
use crate::marks::MarkManager;
use crate::registers::RegisterManager;
use crate::search::SearchState;
use crate::view::View;
use crossterm::event::KeyEvent;

/// Shared state that all mode controllers need access to
pub struct SharedEditorState {
    pub buffer_manager: BufferManager,
    pub view: View,
    pub mark_manager: MarkManager,
    pub register_manager: RegisterManager,
    pub search_state: SearchState,
    pub status_message: String,
    pub show_all_unmatched: bool,
    pub cached_unmatched_brackets: Option<Vec<(usize, usize)>>,
}

/// Result of handling a key event in a mode controller
#[derive(Debug, PartialEq)]
pub enum ModeTransition {
    Stay,
    ToMode(crate::command::Mode),
    Quit,
}

/// Trait that all mode controllers must implement
pub trait ModeController {
    fn handle_key(&mut self, key_event: KeyEvent, shared: &mut SharedEditorState) -> ModeTransition;
}