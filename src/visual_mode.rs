use crate::document::Document;

#[derive(Debug, Clone, PartialEq)]
pub enum VisualMode {
    Char,
    Line,
    Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub mode: VisualMode,
}

impl Selection {
    pub fn new(line: usize, column: usize, mode: VisualMode) -> Self {
        Self {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column,
            mode,
        }
    }

    pub fn update_end(&mut self, line: usize, column: usize) {
        self.end_line = line;
        self.end_column = column;
    }

    pub fn get_ordered_bounds(&self) -> (usize, usize, usize, usize) {
        let (start_line, start_col, end_line, end_col) = if self.start_line < self.end_line ||
            (self.start_line == self.end_line && self.start_column <= self.end_column) {
            (self.start_line, self.start_column, self.end_line, self.end_column)
        } else {
            (self.end_line, self.end_column, self.start_line, self.start_column)
        };

        match self.mode {
            VisualMode::Line => {
                // Line mode always selects entire lines
                (start_line, 0, end_line, usize::MAX)
            }
            VisualMode::Char => {
                // Character mode uses exact positions
                (start_line, start_col, end_line, end_col)
            }
            VisualMode::Block => {
                // Block mode maintains column boundaries
                let left_col = start_col.min(end_col);
                let right_col = start_col.max(end_col);
                (start_line, left_col, end_line, right_col)
            }
        }
    }

    pub fn is_line_in_selection(&self, line: usize) -> bool {
        let (start_line, _, end_line, _) = self.get_ordered_bounds();
        line >= start_line && line <= end_line
    }

    #[allow(dead_code)] // Will be used for visual selection highlighting
    pub fn get_selected_range_for_line(&self, line: usize, line_len: usize) -> Option<(usize, usize)> {
        if !self.is_line_in_selection(line) {
            return None;
        }

        let (start_line, start_col, end_line, end_col) = self.get_ordered_bounds();

        match self.mode {
            VisualMode::Line => {
                // Entire line is selected
                Some((0, line_len))
            }
            VisualMode::Char => {
                let left = if line == start_line { start_col } else { 0 };
                let right = if line == end_line { 
                    end_col.min(line_len) 
                } else { 
                    line_len 
                };
                Some((left, right))
            }
            VisualMode::Block => {
                // Block selection uses column boundaries
                let left = start_col.min(line_len);
                let right = end_col.min(line_len);
                if left <= right {
                    Some((left, right))
                } else {
                    None
                }
            }
        }
    }
}

pub struct VisualModeHandler;

impl VisualModeHandler {
    #[allow(dead_code)] // Will be used for copy/paste operations
    pub fn get_selected_text(selection: &Selection, document: &Document) -> String {
        let mut result = String::new();
        let (start_line, start_col, end_line, end_col) = selection.get_ordered_bounds();

        match selection.mode {
            VisualMode::Char => {
                if start_line == end_line {
                    // Single line selection
                    let line = &document.lines[start_line];
                    let end = end_col.min(line.len());
                    if start_col < line.len() && start_col < end {
                        result.push_str(&line[start_col..end]);
                    }
                } else {
                    // Multi-line selection
                    // First line
                    let first_line = &document.lines[start_line];
                    if start_col < first_line.len() {
                        result.push_str(&first_line[start_col..]);
                    }
                    result.push('\n');

                    // Middle lines
                    for line_idx in (start_line + 1)..end_line {
                        if line_idx < document.lines.len() {
                            result.push_str(&document.lines[line_idx]);
                            result.push('\n');
                        }
                    }

                    // Last line
                    if end_line < document.lines.len() {
                        let last_line = &document.lines[end_line];
                        let end = end_col.min(last_line.len());
                        result.push_str(&last_line[..end]);
                    }
                }
            }
            VisualMode::Line => {
                // Select entire lines
                for line_idx in start_line..=end_line {
                    if line_idx < document.lines.len() {
                        result.push_str(&document.lines[line_idx]);
                        if line_idx < end_line {
                            result.push('\n');
                        }
                    }
                }
            }
            VisualMode::Block => {
                // Block selection - rectangular region
                for line_idx in start_line..=end_line {
                    if line_idx < document.lines.len() {
                        let line = &document.lines[line_idx];
                        let left = start_col.min(line.len());
                        let right = end_col.min(line.len());
                        if left < right {
                            result.push_str(&line[left..right]);
                        }
                        if line_idx < end_line {
                            result.push('\n');
                        }
                    }
                }
            }
        }

        result
    }

    pub fn delete_selection(selection: &Selection, document: &mut Document) {
        let (start_line, start_col, end_line, end_col) = selection.get_ordered_bounds();

        match selection.mode {
            VisualMode::Char => {
                if start_line == end_line {
                    // Single line deletion
                    let line = &mut document.lines[start_line];
                    let end = end_col.min(line.len());
                    if start_col < line.len() && start_col < end {
                        line.drain(start_col..end);
                    }
                } else {
                    // Multi-line deletion
                    // Get the remaining parts of first and last lines
                    let first_line_start = if start_line < document.lines.len() {
                        document.lines[start_line][..start_col.min(document.lines[start_line].len())].to_string()
                    } else {
                        String::new()
                    };
                    
                    let last_line_end = if end_line < document.lines.len() {
                        let last_line = &document.lines[end_line];
                        let end_pos = end_col.min(last_line.len());
                        last_line[end_pos..].to_string()
                    } else {
                        String::new()
                    };

                    // Remove all lines in the selection
                    for _ in start_line..=end_line.min(document.lines.len() - 1) {
                        if start_line < document.lines.len() {
                            document.lines.remove(start_line);
                        }
                    }

                    // Insert the combined line
                    let combined_line = first_line_start + &last_line_end;
                    if start_line <= document.lines.len() {
                        document.lines.insert(start_line, combined_line);
                    } else {
                        document.lines.push(combined_line);
                    }
                }
            }
            VisualMode::Line => {
                // Delete entire lines
                for _ in start_line..=end_line.min(document.lines.len() - 1) {
                    if start_line < document.lines.len() {
                        document.lines.remove(start_line);
                    }
                }
                
                // Ensure we have at least one line
                if document.lines.is_empty() {
                    document.lines.push(String::new());
                }
            }
            VisualMode::Block => {
                // Block deletion - remove rectangular region from each line
                for line_idx in start_line..=end_line {
                    if line_idx < document.lines.len() {
                        let line = &mut document.lines[line_idx];
                        let left = start_col.min(line.len());
                        let right = end_col.min(line.len());
                        if left < right {
                            line.drain(left..right);
                        }
                    }
                }
            }
        }

        // Update cursor position
        document.cursor_line = start_line.min(document.lines.len() - 1);
        document.cursor_column = start_col.min(document.lines[document.cursor_line].len());
        document.modified = true;
    }

    pub fn indent_selection(selection: &Selection, document: &mut Document, tab_width: usize, use_spaces: bool) {
        let (start_line, _, end_line, _) = selection.get_ordered_bounds();
        let indent = if use_spaces {
            " ".repeat(tab_width)
        } else {
            "\t".to_string()
        };

        match selection.mode {
            VisualMode::Line | VisualMode::Char => {
                // Indent entire lines
                for line_idx in start_line..=end_line {
                    if line_idx < document.lines.len() {
                        document.lines[line_idx].insert_str(0, &indent);
                    }
                }
            }
            VisualMode::Block => {
                // Block indent - insert at the left column of the block
                let (_, start_col, _, _) = selection.get_ordered_bounds();
                for line_idx in start_line..=end_line {
                    if line_idx < document.lines.len() {
                        let line = &mut document.lines[line_idx];
                        if start_col <= line.len() {
                            line.insert_str(start_col, &indent);
                        }
                    }
                }
            }
        }

        document.modified = true;
    }

    pub fn dedent_selection(selection: &Selection, document: &mut Document, tab_width: usize) {
        let (start_line, _, end_line, _) = selection.get_ordered_bounds();

        match selection.mode {
            VisualMode::Line | VisualMode::Char => {
                // Dedent entire lines
                for line_idx in start_line..=end_line {
                    if line_idx < document.lines.len() {
                        let line = &mut document.lines[line_idx];
                        
                        // Try to remove a tab first
                        if line.starts_with('\t') {
                            line.remove(0);
                        } else {
                            // Try to remove spaces up to tab_width
                            let mut removed = 0;
                            while removed < tab_width && line.starts_with(' ') {
                                line.remove(0);
                                removed += 1;
                            }
                        }
                    }
                }
            }
            VisualMode::Block => {
                // Block dedent - remove from the left column of the block
                let (_, start_col, _, _) = selection.get_ordered_bounds();
                for line_idx in start_line..=end_line {
                    if line_idx < document.lines.len() {
                        let line = &mut document.lines[line_idx];
                        if start_col < line.len() {
                            if line.chars().nth(start_col) == Some('\t') {
                                line.remove(start_col);
                            } else {
                                // Try to remove spaces
                                let mut removed = 0;
                                while removed < tab_width && start_col < line.len() && 
                                      line.chars().nth(start_col) == Some(' ') {
                                    line.remove(start_col);
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        document.modified = true;
    }
}