use super::document::Document;
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
        self.pattern = pattern.clone();
        self.direction = direction;

        if is_empty {
            self.regex = None;
            self.matches.clear();
            self.current_match = None;
            return Err(SearchError::NoPattern);
        }

        // Escape special regex characters for literal search
        let escaped_pattern = regex::escape(&pattern);

        // Create regex with case sensitivity setting
        let regex_str = if self.case_sensitive {
            escaped_pattern
        } else {
            format!("(?i){}", escaped_pattern)
        };

        match Regex::new(&regex_str) {
            Ok(regex) => {
                self.regex = Some(regex);
                Ok(())
            }
            Err(e) => Err(SearchError::InvalidPattern(e.to_string())),
        }
    }

    pub fn search_document(&mut self, document: &Document) -> Result<(), SearchError> {
        self.matches.clear();
        self.current_match = None;

        let Some(regex) = &self.regex else {
            return Ok(());
        };

        let line_count = document.line_count();
        for line_idx in 0..line_count {
            if let Some(line_text) = document.get_line(line_idx) {
                for mat in regex.find_iter(&line_text) {
                    self.matches.push(SearchMatch {
                        line: line_idx,
                        start_col: mat.start(),
                        end_col: mat.end(),
                        text: mat.as_str().to_string(),
                    });
                }
            }
        }

        if !self.matches.is_empty() {
            self.current_match = Some(0);
        }

        Ok(())
    }

    pub fn find_next_match(&mut self, from_line: usize, from_col: usize) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        // Find the first match after the current position
        let start_idx = self
            .matches
            .iter()
            .position(|m| m.line > from_line || (m.line == from_line && m.start_col > from_col));

        match start_idx {
            Some(idx) => {
                self.current_match = Some(idx);
                self.matches.get(idx)
            }
            None => {
                // No match found after cursor
                if self.wrap_search && !self.matches.is_empty() {
                    // Wrap to beginning
                    self.current_match = Some(0);
                    self.matches.first()
                } else {
                    None
                }
            }
        }
    }

    pub fn find_prev_match(&mut self, from_line: usize, from_col: usize) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        // Find the last match before the current position
        let start_idx = self
            .matches
            .iter()
            .rposition(|m| m.line < from_line || (m.line == from_line && m.start_col < from_col));

        match start_idx {
            Some(idx) => {
                self.current_match = Some(idx);
                self.matches.get(idx)
            }
            None => {
                // No match found before cursor
                if self.wrap_search && !self.matches.is_empty() {
                    // Wrap to end
                    let last_idx = self.matches.len() - 1;
                    self.current_match = Some(last_idx);
                    self.matches.last()
                } else {
                    None
                }
            }
        }
    }

    pub fn repeat_last_search(
        &mut self,
        from_line: usize,
        from_col: usize,
    ) -> Option<&SearchMatch> {
        self.find_next_match(from_line, from_col)
    }

    pub fn repeat_last_search_reverse(
        &mut self,
        from_line: usize,
        from_col: usize,
    ) -> Option<&SearchMatch> {
        self.find_prev_match(from_line, from_col)
    }

    pub fn current_match_index(&self) -> Option<usize> {
        self.current_match
    }
}

#[derive(Debug, Clone)]
pub enum SearchError {
    InvalidPattern(String),
    NoPattern,
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::InvalidPattern(msg) => write!(f, "Invalid search pattern: {}", msg),
            SearchError::NoPattern => write!(f, "No search pattern"),
        }
    }
}