use crate::document::Document;

impl Document {
    // Word movement methods - moved from document.rs to keep it focused on data
    pub fn move_word_forward(&mut self) {
        let line = &self.lines[self.cursor_line];

        // If at end of line, move to next line
        if self.cursor_column >= line.len() {
            if self.cursor_line < self.lines.len() - 1 {
                self.cursor_line += 1;
                self.cursor_column = 0;
                // Find first non-whitespace on new line
                let new_line = &self.lines[self.cursor_line];
                for (i, c) in new_line.chars().enumerate() {
                    if !c.is_whitespace() {
                        self.cursor_column = i;
                        break;
                    }
                }
            }
            return;
        }

        let chars: Vec<char> = line.chars().collect();
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

        // If we reached end of line, move to next line
        if self.cursor_column >= chars.len() {
            if self.cursor_line < self.lines.len() - 1 {
                self.cursor_line += 1;
                self.cursor_column = 0;
                // Find first non-whitespace on new line
                let new_line = &self.lines[self.cursor_line];
                for (i, c) in new_line.chars().enumerate() {
                    if !c.is_whitespace() {
                        self.cursor_column = i;
                        break;
                    }
                }
            } else {
                // At end of document, clamp to last character
                self.cursor_column = if chars.is_empty() { 0 } else { chars.len() - 1 };
            }
        }
    }

    pub fn move_word_backward(&mut self) {
        // If at beginning of line, move to end of previous line
        if self.cursor_column == 0 {
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                let line = &self.lines[self.cursor_line];
                self.cursor_column = line.len();
                // Find last non-whitespace character
                let chars: Vec<char> = line.chars().collect();
                while self.cursor_column > 0 && chars[self.cursor_column - 1].is_whitespace() {
                    self.cursor_column -= 1;
                }
            }
            return;
        }

        let line = &self.lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();

        // Move back one position first
        self.cursor_column -= 1;

        // Skip whitespace
        while self.cursor_column > 0 && chars[self.cursor_column].is_whitespace() {
            self.cursor_column -= 1;
        }

        // If we're now on a character, move to start of this word/punctuation group
        if self.cursor_column < chars.len() && !chars[self.cursor_column].is_whitespace() {
            let current_char = chars[self.cursor_column];
            let is_word_char = current_char.is_alphanumeric() || current_char == '_';

            if is_word_char {
                // Move to start of alphanumeric word
                while self.cursor_column > 0 {
                    let prev_char = chars[self.cursor_column - 1];
                    if !(prev_char.is_alphanumeric() || prev_char == '_') {
                        break;
                    }
                    self.cursor_column -= 1;
                }
            } else {
                // Move to start of punctuation group
                while self.cursor_column > 0 {
                    let prev_char = chars[self.cursor_column - 1];
                    if prev_char.is_whitespace() || prev_char.is_alphanumeric() || prev_char == '_'
                    {
                        break;
                    }
                    self.cursor_column -= 1;
                }
            }
        }
    }

    pub fn move_word_end(&mut self) {
        let line = &self.lines[self.cursor_line];
        if self.cursor_column < line.len() {
            let chars: Vec<char> = line.chars().collect();

            // If on whitespace, move to start of next word first
            if chars[self.cursor_column].is_whitespace() {
                while self.cursor_column < chars.len() && chars[self.cursor_column].is_whitespace()
                {
                    self.cursor_column += 1;
                }
            }

            // Move to end of current word
            while self.cursor_column < chars.len() - 1
                && (chars[self.cursor_column + 1].is_alphanumeric()
                    || chars[self.cursor_column + 1] == '_')
            {
                self.cursor_column += 1;
            }
        }
    }

    // Big word movement (space-separated)
    pub fn move_big_word_forward(&mut self) {
        loop {
            let line = &self.lines[self.cursor_line];
            if self.cursor_column >= line.len() {
                if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_column = 0;
                } else {
                    break;
                }
            } else {
                let chars: Vec<char> = line.chars().collect();
                let start_col = self.cursor_column;

                // Skip current big word (non-whitespace)
                while self.cursor_column < chars.len() && !chars[self.cursor_column].is_whitespace()
                {
                    self.cursor_column += 1;
                }

                // Skip whitespace
                while self.cursor_column < chars.len() && chars[self.cursor_column].is_whitespace()
                {
                    self.cursor_column += 1;
                }

                if self.cursor_column == start_col {
                    if self.cursor_line < self.lines.len() - 1 {
                        self.cursor_line += 1;
                        self.cursor_column = 0;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub fn move_big_word_backward(&mut self) {
        loop {
            if self.cursor_column == 0 {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_column = self.lines[self.cursor_line].len();
                } else {
                    break;
                }
            } else {
                let line = &self.lines[self.cursor_line];
                let chars: Vec<char> = line.chars().collect();

                self.cursor_column -= 1;

                // Skip whitespace
                while self.cursor_column > 0 && chars[self.cursor_column].is_whitespace() {
                    self.cursor_column -= 1;
                }

                // Move to start of big word
                while self.cursor_column > 0 && !chars[self.cursor_column - 1].is_whitespace() {
                    self.cursor_column -= 1;
                }

                break;
            }
        }
    }

    pub fn move_big_word_end(&mut self) {
        let line = &self.lines[self.cursor_line];
        if self.cursor_column < line.len() {
            let chars: Vec<char> = line.chars().collect();

            // If on whitespace, move to start of next word first
            if chars[self.cursor_column].is_whitespace() {
                while self.cursor_column < chars.len() && chars[self.cursor_column].is_whitespace()
                {
                    self.cursor_column += 1;
                }
            }

            // Move to end of current big word
            while self.cursor_column < chars.len() - 1
                && !chars[self.cursor_column + 1].is_whitespace()
            {
                self.cursor_column += 1;
            }
        }
    }

    // Line movement
    pub fn move_line_start(&mut self) {
        self.cursor_column = 0;
    }

    pub fn move_line_end(&mut self) {
        self.cursor_column = self.lines[self.cursor_line].len();
        if self.cursor_column > 0 {
            self.cursor_column -= 1; // Don't go past last character
        }
    }

    pub fn move_first_non_whitespace(&mut self) {
        let line = &self.lines[self.cursor_line];
        self.cursor_column = 0;
        for (i, c) in line.chars().enumerate() {
            if !c.is_whitespace() {
                self.cursor_column = i;
                break;
            }
        }
    }

    pub fn move_down_to_first_non_whitespace(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
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
        self.cursor_line = if self.lines.is_empty() {
            0
        } else {
            self.lines.len() - 1
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
        self.cursor_line = std::cmp::min(self.cursor_line + page_size, self.lines.len() - 1);
        self.clamp_cursor_column();
    }

    pub fn move_half_page_up(&mut self) {
        let half_page = 10; // Could be made configurable
        self.cursor_line = self.cursor_line.saturating_sub(half_page);
        self.clamp_cursor_column();
    }

    pub fn move_half_page_down(&mut self) {
        let half_page = 10; // Could be made configurable
        self.cursor_line = std::cmp::min(self.cursor_line + half_page, self.lines.len() - 1);
        self.clamp_cursor_column();
    }

    pub fn move_to_line(&mut self, line: usize) {
        self.cursor_line = std::cmp::min(line.saturating_sub(1), self.lines.len() - 1);
        self.cursor_column = 0;
    }

    // Character search
    pub fn find_char(&mut self, target: char, forward: bool, before: bool) {
        let line = &self.lines[self.cursor_line];
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
