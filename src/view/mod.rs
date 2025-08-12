/// View subsystem - Independent rendering and display management
/// 
/// This module provides a clean abstraction layer for all visual rendering,
/// completely independent of document internals through the ViewModel trait.

pub mod view_model;
pub mod renderer;
pub mod buffer_manager;

// Re-export public interface
pub use view_model::{DocumentViewModel, BracketHighlight};
pub use renderer::{View, RenderParams};
pub use buffer_manager::BufferManager;