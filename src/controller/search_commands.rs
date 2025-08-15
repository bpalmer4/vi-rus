use crate::document_model::{Document, SearchState, SearchDirection, SearchError};

/// Search command handlers - controller logic for search operations
pub struct SearchCommands;

impl SearchCommands {
    /// Execute next search (n command)
    pub fn next(search_state: &mut SearchState, document: &mut Document, status_message: &mut String) {
        let line = document.cursor_line();
        let col = document.cursor_column();

        if let Some(search_match) = search_state.repeat_last_search(line, col) {
            document.move_cursor_to(search_match.line, search_match.start_col);
            
            // Show match count if available
            if let (Some(current), matches) = (search_state.current_match_index(), search_state.matches.len()) {
                if matches > 0 {
                    *status_message = format!("/{}/  [{}/{}]", search_state.pattern, current + 1, matches);
                } else {
                    *status_message = format!("/{}/", search_state.pattern);
                }
            } else {
                *status_message = format!("/{}/", search_state.pattern);
            }
        } else {
            *status_message = if search_state.pattern.is_empty() {
                "No previous search".to_string()
            } else {
                format!("Pattern not found: {}", search_state.pattern)
            };
        }
    }

    /// Execute previous search (N command)
    pub fn previous(search_state: &mut SearchState, document: &mut Document, status_message: &mut String) {
        let line = document.cursor_line();
        let col = document.cursor_column();

        if let Some(search_match) = search_state.repeat_last_search_reverse(line, col) {
            document.move_cursor_to(search_match.line, search_match.start_col);
            
            // Show match count if available
            if let (Some(current), matches) = (search_state.current_match_index(), search_state.matches.len()) {
                if matches > 0 {
                    *status_message = format!("?{}?  [{}/{}]", search_state.pattern, current + 1, matches);
                } else {
                    *status_message = format!("?{}?", search_state.pattern);
                }
            } else {
                *status_message = format!("?{}?", search_state.pattern);
            }
        } else {
            *status_message = if search_state.pattern.is_empty() {
                "No previous search".to_string()
            } else {
                format!("Pattern not found: {}", search_state.pattern)
            };
        }
    }

    /// Search for word under cursor forward (* command)
    pub fn search_word_forward(search_state: &mut SearchState, document: &mut Document, status_message: &mut String) {
        if let Some(word) = document.get_word_under_cursor() {
            if let Err(e) = search_state.set_pattern(word.clone(), SearchDirection::Forward) {
                *status_message = format!("Search error: {}", e);
                return;
            }
            
            if let Err(e) = search_state.search_document(document) {
                *status_message = format!("Search error: {}", e);
                return;
            }
            
            Self::next(search_state, document, status_message);
        } else {
            *status_message = "No word under cursor".to_string();
        }
    }

    /// Search for word under cursor backward (# command)
    pub fn search_word_backward(search_state: &mut SearchState, document: &mut Document, status_message: &mut String) {
        if let Some(word) = document.get_word_under_cursor() {
            if let Err(e) = search_state.set_pattern(word.clone(), SearchDirection::Backward) {
                *status_message = format!("Search error: {}", e);
                return;
            }
            
            if let Err(e) = search_state.search_document(document) {
                *status_message = format!("Search error: {}", e);
                return;
            }
            
            Self::previous(search_state, document, status_message);
        } else {
            *status_message = "No word under cursor".to_string();
        }
    }

    /// Initialize search with pattern
    pub fn start_search(
        search_state: &mut SearchState,
        document: &Document,
        pattern: String,
        direction: SearchDirection,
    ) -> Result<(), SearchError> {
        search_state.set_pattern(pattern, direction)?;
        search_state.search_document(document)?;
        Ok(())
    }
}

// Note: Search and replace functionality has been moved to the unified range system
// in command.rs for better integration with vim-style range operations.