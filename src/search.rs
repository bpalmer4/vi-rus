use crate::document::Document;
use regex::Regex;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line: usize,
    pub start_col: usize,
    #[allow(dead_code)] // Will be used for search highlighting
    pub end_col: usize,
    #[allow(dead_code)] // Will be used for search highlighting
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub pattern: String,
    pub direction: SearchDirection,
    pub matches: Vec<SearchMatch>,
    pub current_match: Option<usize>,
    pub regex: Option<Regex>,
    pub case_sensitive: bool,
    pub wrap_search: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            direction: SearchDirection::Forward,
            matches: Vec::new(),
            current_match: None,
            regex: None,
            case_sensitive: false, // Default to case insensitive like vim
            wrap_search: true,     // Default to wrap search like vim
        }
    }

    pub fn set_pattern(
        &mut self,
        pattern: String,
        direction: SearchDirection,
    ) -> Result<(), SearchError> {
        let is_empty = pattern.is_empty();
        self.direction = direction;

        if is_empty {
            self.pattern = pattern;
            self.regex = None;
            self.matches.clear();
            self.current_match = None;
            return Ok(());
        }

        // Create regex with appropriate flags - use reference for efficiency
        let regex = if self.case_sensitive {
            Regex::new(&pattern)
        } else {
            Regex::new(&format!("(?i){pattern}"))
        };

        // Set pattern after we've used it
        self.pattern = pattern;

        match regex {
            Ok(regex) => {
                self.regex = Some(regex);
                Ok(())
            }
            Err(e) => Err(SearchError::InvalidPattern(format!("Invalid regex: {e}"))),
        }
    }

    pub fn search_document(&mut self, document: &Document) -> Result<(), SearchError> {
        self.matches.clear();
        self.current_match = None;

        let regex = match &self.regex {
            Some(r) => r,
            None => return Ok(()), // No pattern to search
        };

        // Find all matches in document
        let lines = if document.use_piece_table {
            if let Some(ref text_buffer) = document.text_buffer {
                let mut text_buffer = text_buffer.clone();
                text_buffer.get_lines()
            } else {
                Vec::new()
            }
        } else {
            document.lines.clone()
        };

        for (line_idx, line) in lines.iter().enumerate() {
            for mat in regex.find_iter(line) {
                self.matches.push(SearchMatch {
                    line: line_idx,
                    start_col: mat.start(),
                    end_col: mat.end(),
                    text: mat.as_str().to_string(),
                });
            }
        }

        Ok(())
    }

    pub fn find_next_match(&mut self, from_line: usize, from_col: usize) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        // Find first match after current position
        let start_pos = match self.direction {
            SearchDirection::Forward => {
                // Find first match after (from_line, from_col)
                self.matches.iter().position(|m| {
                    m.line > from_line || (m.line == from_line && m.start_col > from_col)
                })
            }
            SearchDirection::Backward => {
                // Find last match before (from_line, from_col)
                self.matches.iter().rposition(|m| {
                    m.line < from_line || (m.line == from_line && m.start_col < from_col)
                })
            }
        };

        if let Some(pos) = start_pos {
            self.current_match = Some(pos);
            return self.matches.get(pos);
        }

        // No match found in the direction, try wrapping if enabled
        if self.wrap_search && !self.matches.is_empty() {
            let wrapped_pos = match self.direction {
                SearchDirection::Forward => 0, // Wrap to first match
                SearchDirection::Backward => self.matches.len() - 1, // Wrap to last match
            };
            self.current_match = Some(wrapped_pos);
            return self.matches.get(wrapped_pos);
        }

        None
    }

    pub fn find_prev_match(&mut self, from_line: usize, from_col: usize) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        // Find last match before current position (opposite of find_next_match logic)
        let start_pos = match self.direction {
            SearchDirection::Forward => {
                // Find last match before (from_line, from_col)
                self.matches.iter().rposition(|m| {
                    m.line < from_line || (m.line == from_line && m.start_col < from_col)
                })
            }
            SearchDirection::Backward => {
                // Find first match after (from_line, from_col)
                self.matches.iter().position(|m| {
                    m.line > from_line || (m.line == from_line && m.start_col > from_col)
                })
            }
        };

        if let Some(pos) = start_pos {
            self.current_match = Some(pos);
            return self.matches.get(pos);
        }

        // No match found in the direction, try wrapping if enabled
        if self.wrap_search && !self.matches.is_empty() {
            let wrapped_pos = match self.direction {
                SearchDirection::Forward => self.matches.len() - 1, // Wrap to last match
                SearchDirection::Backward => 0,                     // Wrap to first match
            };
            self.current_match = Some(wrapped_pos);
            return self.matches.get(wrapped_pos);
        }

        None
    }

    pub fn repeat_last_search(
        &mut self,
        from_line: usize,
        from_col: usize,
    ) -> Option<&SearchMatch> {
        // Continue in the same direction as the original search
        self.find_next_match(from_line, from_col)
    }

    pub fn repeat_last_search_reverse(
        &mut self,
        from_line: usize,
        from_col: usize,
    ) -> Option<&SearchMatch> {
        // Continue in the opposite direction of the original search
        self.find_prev_match(from_line, from_col)
    }

    pub fn current_match_index(&self) -> Option<usize> {
        self.current_match.map(|idx| idx + 1) // 1-based for display
    }
}

#[derive(Debug, Clone)]
pub enum SearchError {
    InvalidPattern(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::InvalidPattern(msg) => write!(f, "Search error: {msg}"),
        }
    }
}

pub struct SearchReplace;

impl SearchReplace {
    pub fn substitute_line(
        line: &str,
        pattern: &str,
        replacement: &str,
        global: bool,
        case_sensitive: bool,
    ) -> Result<(String, usize), SearchError> {
        let regex_pattern = if case_sensitive {
            pattern.to_string()
        } else {
            format!("(?i){pattern}")
        };

        let regex = Regex::new(&regex_pattern)
            .map_err(|e| SearchError::InvalidPattern(format!("Invalid regex: {e}")))?;

        let result = if global {
            regex.replace_all(line, replacement).to_string()
        } else {
            regex.replace(line, replacement).to_string()
        };

        // Count replacements
        let original_matches = regex.find_iter(line).count();
        let replacements = if global {
            original_matches
        } else if original_matches > 0 {
            1
        } else {
            0
        };

        Ok((result, replacements))
    }

    pub fn substitute_document(
        document: &mut Document,
        start_line: usize,
        end_line: usize,
        pattern: &str,
        replacement: &str,
        global: bool,
        case_sensitive: bool,
    ) -> Result<usize, SearchError> {
        let mut total_replacements = 0;
        let line_count = if document.use_piece_table {
            if let Some(ref text_buffer) = document.text_buffer {
                let mut text_buffer = text_buffer.clone();
                text_buffer.line_count()
            } else {
                0
            }
        } else {
            document.line_count()
        };
        let actual_end_line = end_line.min(line_count.saturating_sub(1));

        for line_idx in start_line..=actual_end_line {
            if document.use_piece_table {
                if let Some(ref mut text_buffer) = document.text_buffer {
                    if let Some(line) = text_buffer.get_line(line_idx) {
                        let (new_line, replacements) =
                            Self::substitute_line(&line, pattern, replacement, global, case_sensitive)?;

                        if replacements > 0 {
                            // Replace entire line in piece table
                            let start_pos = crate::text_buffer::Position::new(line_idx, 0);
                            let end_pos = crate::text_buffer::Position::new(line_idx, text_buffer.line_length(line_idx));
                            let range = crate::text_buffer::Range::new(start_pos, end_pos);
                            text_buffer.replace(range, &new_line);
                            total_replacements += replacements;
                        }
                    }
                }
            } else {
                if let Some(line) = document.lines.get_mut(line_idx) {
                    let (new_line, replacements) =
                        Self::substitute_line(line, pattern, replacement, global, case_sensitive)?;

                    if replacements > 0 {
                        *line = new_line;
                        total_replacements += replacements;
                    }
                }
            }
        }

        if total_replacements > 0 {
            // Sync piece table changes back to Vec<String>
            if document.use_piece_table {
                if let Some(ref mut text_buffer) = document.text_buffer {
                    document.lines = text_buffer.to_lines();
                }
            }
            document.modified = true;
        }

        Ok(total_replacements)
    }

    pub fn substitute_all_document(
        document: &mut Document,
        pattern: &str,
        replacement: &str,
        case_sensitive: bool,
    ) -> Result<usize, SearchError> {
        let line_count = if document.use_piece_table {
            if let Some(ref text_buffer) = document.text_buffer {
                let mut text_buffer = text_buffer.clone();
                text_buffer.line_count()
            } else {
                0
            }
        } else {
            document.line_count()
        };
        
        if line_count == 0 {
            return Ok(0);
        }

        Self::substitute_document(
            document,
            0,
            line_count - 1,
            pattern,
            replacement,
            true, // Always global for :%s
            case_sensitive,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_state_basic() {
        let mut search = SearchState::new();
        assert!(
            search
                .set_pattern("test".to_string(), SearchDirection::Forward)
                .is_ok()
        );
        assert_eq!(search.pattern, "test");
        assert_eq!(search.direction, SearchDirection::Forward);
    }

    #[test]
    fn test_substitute_line() {
        let result = SearchReplace::substitute_line("hello world", "world", "vim", false, true);
        assert!(result.is_ok());
        let (new_line, count) = result.unwrap();
        assert_eq!(new_line, "hello vim");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_substitute_line_global() {
        let result = SearchReplace::substitute_line("foo foo foo", "foo", "bar", true, true);
        assert!(result.is_ok());
        let (new_line, count) = result.unwrap();
        assert_eq!(new_line, "bar bar bar");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_substitute_case_insensitive() {
        let result = SearchReplace::substitute_line("Hello WORLD", "world", "vim", false, false);
        assert!(result.is_ok());
        let (new_line, count) = result.unwrap();
        assert_eq!(new_line, "Hello vim");
        assert_eq!(count, 1);
    }
}
