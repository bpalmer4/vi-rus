use super::document::Document;

impl Document {
    // Word movement methods - moved from document.rs to keep it focused on data
    fn move_word_forward_impl(&mut self, use_word_boundaries: bool) {
        loop {
            let line = self.get_line(self.cursor_line).unwrap_or_default();
            let line_count = self.line_count();
            
            // If at end of line, move to next line
            if self.cursor_column >= line.len() {
                if self.cursor_line < line_count - 1 {
                    self.cursor_line += 1;
                    self.cursor_column = 0;
                } else {
                    break;
                }
                continue;
            }

            let chars: Vec<char> = line.chars().collect();
            let start_col = self.cursor_column;
            
            if use_word_boundaries {
                let current_char = chars[self.cursor_column];
                
                // Skip whitespace first
                if current_char.is_whitespace() {
                    while self.cursor_column < chars.len() && chars[self.cursor_column].is_whitespace() {
                        self.cursor_column += 1;
                    }
                } else {
                    // Determine the type of the current character
                    let is_word_char = current_char.is_alphanumeric() || current_char == '_';

                    if is_word_char {
                        // Skip alphanumeric/underscore characters
                        while self.cursor_column < chars.len() {
                            let c = chars[self.cursor_column];
                            if !(c.is_alphanumeric() || c == '_') {
                                break;
                            }
                            self.cursor_column += 1;
                        }
                    } else {
                        // Skip punctuation characters (non-whitespace, non-alphanumeric)
                        while self.cursor_column < chars.len() {
                            let c = chars[self.cursor_column];
                            if c.is_whitespace() || c.is_alphanumeric() || c == '_' {
                                break;
                            }
                            self.cursor_column += 1;
                        }
                    }

                    // Skip whitespace after the word/punctuation
                    while self.cursor_column < chars.len() && chars[self.cursor_column].is_whitespace() {
                        self.cursor_column += 1;
                    }
                }
            } else {
                // Big word logic: skip current non-whitespace, then skip whitespace
                while self.cursor_column < chars.len() && !chars[self.cursor_column].is_whitespace() {
                    self.cursor_column += 1;
                }
                while self.cursor_column < chars.len() && chars[self.cursor_column].is_whitespace() {
                    self.cursor_column += 1;
                }
            }

            // If we didn't move or reached end of line, continue to next line
            if self.cursor_column == start_col || self.cursor_column >= chars.len() {
                if self.cursor_line < line_count - 1 {
                    self.cursor_line += 1;
                    self.cursor_column = 0;
                } else {
                    // At end of document, clamp to last character
                    self.cursor_column = if chars.is_empty() { 0 } else { chars.len() - 1 };
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn move_word_forward(&mut self) {
        self.move_word_forward_impl(true);
    }

    fn move_word_backward_impl(&mut self, use_word_boundaries: bool) {
        loop {
            // If at beginning of line, move to end of previous line
            if self.cursor_column == 0 {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    let line = self.get_line(self.cursor_line).unwrap_or_default();
                    let chars: Vec<char> = line.chars().collect();
                    self.cursor_column = chars.len();
                    // Find last non-whitespace character
                    while self.cursor_column > 0 && self.cursor_column <= chars.len() {
                        if chars.get(self.cursor_column - 1).map_or(false, |c| !c.is_whitespace()) {
                            break;
                        }
                        self.cursor_column -= 1;
                    }
                } else {
                    break;
                }
                continue;
            }

            let line = self.get_line(self.cursor_line).unwrap_or_default();
            let chars: Vec<char> = line.chars().collect();

            if chars.is_empty() {
                self.cursor_column = 0;
                return;
            }

            // Move back one position first, with bounds check
            if self.cursor_column > 0 {
                self.cursor_column -= 1;
            }

            if use_word_boundaries {
                // Skip whitespace
                while self.cursor_column > 0 && self.cursor_column < chars.len() && chars[self.cursor_column].is_whitespace() {
                    self.cursor_column -= 1;
                }

                // If we're now on a character, move to start of this word/punctuation group
                if self.cursor_column < chars.len() && !chars[self.cursor_column].is_whitespace() {
                    let current_char = chars[self.cursor_column];
                    let is_word_char = current_char.is_alphanumeric() || current_char == '_';

                    if is_word_char {
                        // Move to start of alphanumeric word
                        while self.cursor_column > 0 {
                            if let Some(prev_char) = chars.get(self.cursor_column - 1) {
                                if prev_char.is_alphanumeric() || *prev_char == '_' {
                                    self.cursor_column -= 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    } else {
                        // Move to start of punctuation group
                        while self.cursor_column > 0 {
                            if let Some(prev_char) = chars.get(self.cursor_column - 1) {
                                if !prev_char.is_whitespace() && !prev_char.is_alphanumeric() && *prev_char != '_' {
                                    self.cursor_column -= 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            } else {
                // Big word logic: skip whitespace, then move to start of big word
                while self.cursor_column > 0 && chars[self.cursor_column].is_whitespace() {
                    self.cursor_column -= 1;
                }
                
                // Move to start of big word
                while self.cursor_column > 0 && !chars[self.cursor_column - 1].is_whitespace() {
                    self.cursor_column -= 1;
                }
            }
            
            break;
        }
    }

    pub fn move_word_backward(&mut self) {
        self.move_word_backward_impl(true);
    }

    fn move_to_word_end(&mut self, use_word_boundaries: bool) {
        let line = self.get_line(self.cursor_line).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        
        if chars.is_empty() {
            // On blank line, move to next line and find first word end
            let line_count = self.line_count();
            if self.cursor_line + 1 < line_count {
                self.cursor_line += 1;
                self.cursor_column = 0;
                
                // Get the new line and find first word end
                let next_line = self.get_line(self.cursor_line).unwrap_or_default();
                let next_chars: Vec<char> = next_line.chars().collect();
                
                if !next_chars.is_empty() {
                    // Skip leading whitespace on new line
                    while self.cursor_column < next_chars.len() && next_chars[self.cursor_column].is_whitespace() {
                        self.cursor_column += 1;
                    }
                    
                    // Move to end of first word on new line
                    if self.cursor_column < next_chars.len() {
                        if use_word_boundaries {
                            let current_type = self.get_word_type(next_chars[self.cursor_column]);
                            while self.cursor_column + 1 < next_chars.len() {
                                let next_char = next_chars[self.cursor_column + 1];
                                if !next_char.is_whitespace() && self.get_word_type(next_char) == current_type {
                                    self.cursor_column += 1;
                                } else {
                                    break;
                                }
                            }
                        } else {
                            // Big word: move to end of non-whitespace sequence
                            while self.cursor_column < next_chars.len() - 1 && !next_chars[self.cursor_column + 1].is_whitespace() {
                                self.cursor_column += 1;
                            }
                        }
                    }
                } else {
                    // If next line is also blank, recursively call
                    self.move_to_word_end(use_word_boundaries);
                }
            }
            return;
        }

        // Clamp cursor to valid range
        if self.cursor_column >= chars.len() {
            self.cursor_column = chars.len().saturating_sub(1);
        }

        // Check if we're already at word end
        let at_word_end = self.cursor_column < chars.len() && 
            !chars[self.cursor_column].is_whitespace() &&
            (self.cursor_column + 1 >= chars.len() || 
             chars[self.cursor_column + 1].is_whitespace() ||
             (use_word_boundaries && self.is_different_word_type(chars[self.cursor_column], chars[self.cursor_column + 1])));

        // If already at word end, move forward to find next word
        if at_word_end {
            self.cursor_column += 1;
        }

        // Skip whitespace to find next word
        while self.cursor_column < chars.len() && chars[self.cursor_column].is_whitespace() {
            self.cursor_column += 1;
        }

        // If we reached end of line, try next line
        if self.cursor_column >= chars.len() {
            let line_count = self.line_count();
            if self.cursor_line + 1 < line_count {
                self.cursor_line += 1;
                self.cursor_column = 0;
                
                // Get the new line and find first word end
                let next_line = self.get_line(self.cursor_line).unwrap_or_default();
                let next_chars: Vec<char> = next_line.chars().collect();
                
                if !next_chars.is_empty() {
                    // Skip leading whitespace on new line
                    while self.cursor_column < next_chars.len() && next_chars[self.cursor_column].is_whitespace() {
                        self.cursor_column += 1;
                    }
                    
                    // Move to end of first word on new line
                    if self.cursor_column < next_chars.len() {
                        if use_word_boundaries {
                            let current_type = self.get_word_type(next_chars[self.cursor_column]);
                            while self.cursor_column + 1 < next_chars.len() {
                                let next_char = next_chars[self.cursor_column + 1];
                                if !next_char.is_whitespace() && self.get_word_type(next_char) == current_type {
                                    self.cursor_column += 1;
                                } else {
                                    break;
                                }
                            }
                        } else {
                            // Big word: move to end of non-whitespace sequence
                            while self.cursor_column < next_chars.len() - 1 && !next_chars[self.cursor_column + 1].is_whitespace() {
                                self.cursor_column += 1;
                            }
                        }
                    }
                }
            }
        } else {
            // Move to end of current word
            if use_word_boundaries {
                let current_type = self.get_word_type(chars[self.cursor_column]);
                while self.cursor_column + 1 < chars.len() {
                    let next_char = chars[self.cursor_column + 1];
                    if !next_char.is_whitespace() && self.get_word_type(next_char) == current_type {
                        self.cursor_column += 1;
                    } else {
                        break;
                    }
                }
            } else {
                // Big word: move to end of non-whitespace sequence
                while self.cursor_column < chars.len() - 1 && !chars[self.cursor_column + 1].is_whitespace() {
                    self.cursor_column += 1;
                }
            }
        }

        // Final bounds check
        let final_line = self.get_line(self.cursor_line).unwrap_or_default();
        let final_chars: Vec<char> = final_line.chars().collect();
        if self.cursor_column >= final_chars.len() && !final_chars.is_empty() {
            self.cursor_column = final_chars.len() - 1;
        }
    }

    pub fn move_word_end(&mut self) {
        self.move_to_word_end(true);
    }

    fn get_word_type(&self, c: char) -> u8 {
        if c.is_alphanumeric() || c == '_' {
            1 // alphanumeric
        } else {
            2 // punctuation
        }
    }

    fn is_different_word_type(&self, c1: char, c2: char) -> bool {
        self.get_word_type(c1) != self.get_word_type(c2)
    }

    // Big word movement (space-separated)
    pub fn move_big_word_forward(&mut self) {
        self.move_word_forward_impl(false);
    }

    pub fn move_big_word_backward(&mut self) {
        self.move_word_backward_impl(false);
    }

    pub fn move_big_word_end(&mut self) {
        self.move_to_word_end(false);
    }

    // Line movement
    pub fn move_line_start(&mut self) {
        self.cursor_column = 0;
    }

    pub fn move_line_end(&mut self) {
        let line_len = self.get_line_length(self.cursor_line);
        self.cursor_column = line_len;
        if self.cursor_column > 0 {
            self.cursor_column -= 1; // Don't go past last character
        }
    }

    pub fn move_first_non_whitespace(&mut self) {
        let line = self.get_line(self.cursor_line).unwrap_or_default();
        self.cursor_column = 0;
        for (i, c) in line.chars().enumerate() {
            if !c.is_whitespace() {
                self.cursor_column = i;
                break;
            }
        }
    }

    pub fn move_down_to_first_non_whitespace(&mut self) {
        let line_count = self.line_count();
        if self.cursor_line < line_count - 1 {
            self.cursor_line += 1;
            self.move_first_non_whitespace();
        }
    }

    pub fn move_up_to_first_non_whitespace(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.move_first_non_whitespace();
        }
    }

    // Document movement
    pub fn move_document_start(&mut self) {
        self.cursor_line = 0;
        self.cursor_column = 0;
    }

    pub fn move_document_end(&mut self) {
        let line_count = self.line_count();
        self.cursor_line = if line_count == 0 {
            0
        } else {
            line_count - 1
        };
        self.cursor_column = 0;
    }

    pub fn move_page_up(&mut self) {
        let page_size = 20; // Could be made configurable
        self.cursor_line = self.cursor_line.saturating_sub(page_size);
        self.clamp_cursor_column();
    }

    pub fn move_page_down(&mut self) {
        let page_size = 20; // Could be made configurable
        let line_count = self.line_count();
        self.cursor_line = std::cmp::min(self.cursor_line + page_size, line_count.saturating_sub(1));
        self.clamp_cursor_column();
    }

    pub fn move_half_page_up(&mut self) {
        let half_page = 10; // Could be made configurable
        self.cursor_line = self.cursor_line.saturating_sub(half_page);
        self.clamp_cursor_column();
    }

    pub fn move_half_page_down(&mut self) {
        let half_page = 10; // Could be made configurable
        let line_count = self.line_count();
        self.cursor_line = std::cmp::min(self.cursor_line + half_page, line_count.saturating_sub(1));
        self.clamp_cursor_column();
    }

    pub fn move_to_line(&mut self, line: usize) {
        let line_count = self.line_count();
        self.cursor_line = std::cmp::min(line.saturating_sub(1), line_count.saturating_sub(1));
        self.cursor_column = 0;
    }

    // Character search
    pub fn find_char(&mut self, target: char, forward: bool, before: bool) {
        let line = self.get_line(self.cursor_line).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();

        if forward {
            let start = if before {
                self.cursor_column
            } else {
                self.cursor_column + 1
            };
            for (i, ch) in chars.iter().enumerate().skip(start) {
                if *ch == target {
                    self.cursor_column = if before && i > 0 { i - 1 } else { i };
                    break;
                }
            }
        } else {
            let end = if before && self.cursor_column < chars.len() {
                self.cursor_column + 1
            } else {
                self.cursor_column
            };
            for i in (0..end).rev() {
                if chars[i] == target {
                    self.cursor_column = if before && i < chars.len() - 1 {
                        i + 1
                    } else {
                        i
                    };
                    break;
                }
            }
        }
    }
}
