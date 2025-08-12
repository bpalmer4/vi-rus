/// Document model subsystem - Core data structures and text operations
/// 
/// This module contains all document-related data structures and operations,
/// including text storage, editing operations, search state, marks, and undo/redo.

pub mod document;
pub mod text_buffer;
pub mod piece_table;
pub mod search_state;
pub mod marks;
pub mod movement;
pub mod registers;
pub mod undo;

// Re-export main types for convenience
pub use document::{Document, LineEnding};
pub use text_buffer::{TextBuffer, Position};
pub use search_state::{SearchState, SearchDirection, SearchError};
pub use marks::MarkManager;
pub use registers::{RegisterManager, RegisterType};
pub use undo::UndoManager;