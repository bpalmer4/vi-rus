/// Configuration subsystem - Editor settings and preferences
/// 
/// This module handles loading and applying configuration from .virusrc files,
/// providing centralized settings management for the entire application.

pub mod rc;

// Re-export public interface
pub use rc::{RcConfig, RcLoader};