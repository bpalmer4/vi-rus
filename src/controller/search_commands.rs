use crate::document_model::{Document, SearchState, SearchDirection, SearchError};

/// Search command handlers - controller logic for search operations
pub struct SearchCommands;

impl SearchCommands {
    /// Execute next search (n command)
    pub fn next(search_state: &mut SearchState, document: &mut Document, status_message: &mut String) {
        let line = document.cursor_line;
        let col = document.cursor_column;

        if let Some(search_match) = search_state.repeat_last_search(line, col) {
            document.cursor_line = search_match.line;
            document.cursor_column = search_match.start_col;
            
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
        let line = document.cursor_line;
        let col = document.cursor_column;

        if let Some(search_match) = search_state.repeat_last_search_reverse(line, col) {
            document.cursor_line = search_match.line;
            document.cursor_column = search_match.start_col;
            
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

/// Search and replace functionality
pub struct SearchReplace;

impl SearchReplace {
    pub fn substitute_line(
        document: &mut Document,
        line_num: usize,
        pattern: &str,
        replacement: &str,
        global: bool,
    ) -> Result<usize, String> {
        let regex = Regex::new(pattern)
            .map_err(|e| format!("Invalid pattern: {}", e))?;

        if let Some(line_text) = document.get_line(line_num) {
            let new_text = if global {
                regex.replace_all(&line_text, replacement).to_string()
            } else {
                regex.replace(&line_text, replacement).to_string()
            };

            if new_text != line_text {
                document.set_line(line_num, &new_text);
                document.modified = true;
                
                let count = if global {
                    regex.find_iter(&line_text).count()
                } else {
                    1
                };
                
                Ok(count)
            } else {
                Ok(0)
            }
        } else {
            Err("Line out of range".to_string())
        }
    }

    pub fn substitute_document(
        document: &mut Document,
        pattern: &str,
        replacement: &str,
        global: bool,
    ) -> Result<(usize, usize), String> {
        let mut total_substitutions = 0;
        let mut lines_affected = 0;

        let line_count = document.line_count();
        for line_num in 0..line_count {
            match Self::substitute_line(document, line_num, pattern, replacement, global) {
                Ok(count) if count > 0 => {
                    total_substitutions += count;
                    lines_affected += 1;
                }
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        Ok((total_substitutions, lines_affected))
    }

    pub fn substitute_all_document(
        document: &mut Document,
        pattern: &str,
        replacement: &str,
    ) -> Result<(usize, usize), String> {
        Self::substitute_document(document, pattern, replacement, true)
    }

    pub fn substitute_range(
        document: &mut Document,
        start_line: usize,
        end_line: usize,
        pattern: &str,
        replacement: &str,
        global: bool,
    ) -> Result<(usize, usize), String> {
        let mut total_substitutions = 0;
        let mut lines_affected = 0;

        let line_count = document.line_count();
        let end = std::cmp::min(end_line, line_count - 1);

        for line_num in start_line..=end {
            match Self::substitute_line(document, line_num, pattern, replacement, global) {
                Ok(count) if count > 0 => {
                    total_substitutions += count;
                    lines_affected += 1;
                }
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        Ok((total_substitutions, lines_affected))
    }
}

// Need to import Regex for SearchReplace
use regex::Regex;