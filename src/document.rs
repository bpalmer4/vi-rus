use crate::undo::UndoManager;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEnding {
    Unix,    // \n (LF)
    Windows, // \r\n (CRLF)
    Mac,     // \r (CR)
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Unix => "\n",
            LineEnding::Windows => "\r\n",
            LineEnding::Mac => "\r",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LineEnding::Unix => "unix",
            LineEnding::Windows => "dos",
            LineEnding::Mac => "mac",
        }
    }

    pub fn system_default() -> Self {
        if cfg!(windows) {
            LineEnding::Windows
        } else {
            LineEnding::Unix
        }
    }

    pub fn detect(content: &str) -> Self {
        if content.contains("\r\n") {
            LineEnding::Windows
        } else if content.contains('\r') {
            LineEnding::Mac
        } else {
            LineEnding::Unix
        }
    }
}

#[derive(Clone)]
pub struct Document {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub filename: Option<PathBuf>,
    pub modified: bool,
    pub line_ending: LineEnding,
    pub expand_tab: bool,
    pub local_marks: HashMap<char, (usize, usize)>, // Local marks (a-z) for this buffer
    pub undo_manager: UndoManager,
}

impl Document {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_column: 0,
            filename: None,
            modified: false,
            line_ending: LineEnding::system_default(),
            expand_tab: true, // Default to spaces
            local_marks: HashMap::new(),
            undo_manager: UndoManager::new(),
        }
    }

    pub fn from_file(filename: PathBuf) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(&filename)?;
        let line_ending = LineEnding::detect(&content);
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };

        Ok(Self {
            lines,
            cursor_line: 0,
            cursor_column: 0,
            filename: Some(filename),
            modified: false,
            line_ending,
            expand_tab: true, // Default to spaces
            local_marks: HashMap::new(),
            undo_manager: UndoManager::new(),
        })
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn save(&mut self) -> Result<usize, std::io::Error> {
        if let Some(ref filename) = self.filename {
            self.save_as(filename.clone())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No filename specified",
            ))
        }
    }

    pub fn save_as(&mut self, filename: PathBuf) -> Result<usize, std::io::Error> {
        let content = self.lines.join(self.line_ending.as_str());
        let byte_count = content.len();
        fs::write(&filename, &content)?;
        self.filename = Some(filename);
        self.modified = false;
        Ok(byte_count)
    }

    pub fn set_line_ending(&mut self, line_ending: LineEnding) {
        if self.line_ending != line_ending {
            self.line_ending = line_ending;
            self.modified = true;
        }
    }

    pub fn insert_file_at_cursor(
        &mut self,
        filename: &std::path::Path,
    ) -> Result<usize, std::io::Error> {
        let content = fs::read_to_string(filename)?;
        self.insert_text_at_cursor(&content)
    }

    pub fn insert_file_at_line(
        &mut self,
        filename: &std::path::Path,
        line_num: usize,
    ) -> Result<usize, std::io::Error> {
        let content = fs::read_to_string(filename)?;
        self.insert_text_at_line(&content, line_num)
    }

    pub fn insert_text_at_cursor(&mut self, text: &str) -> Result<usize, std::io::Error> {
        self.insert_text_at_line(text, self.cursor_line + 1)
    }

    pub fn paste_at_cursor(&mut self, text: &str) -> Result<usize, std::io::Error> {
        if text.is_empty() {
            return Ok(0);
        }

        let byte_count = text.len();

        // Check if text contains newlines
        if !text.contains('\n') {
            // Simple single-line paste
            self.lines[self.cursor_line].insert_str(self.cursor_column, text);
            self.cursor_column += text.len();
        } else {
            // Multi-line paste - split on newlines
            let paste_lines: Vec<&str> = text.split('\n').collect();

            let current_line = &self.lines[self.cursor_line];
            let before_cursor = current_line[..self.cursor_column].to_string();
            let after_cursor = current_line[self.cursor_column..].to_string();

            // Replace current line with: before_cursor + first_paste_line
            self.lines[self.cursor_line] = before_cursor + paste_lines[0];

            // Insert all middle lines (if any)
            for (i, line) in paste_lines[1..paste_lines.len() - 1].iter().enumerate() {
                self.lines
                    .insert(self.cursor_line + 1 + i, line.to_string());
            }

            // Handle the last line
            if paste_lines.len() > 1 {
                let final_line = paste_lines[paste_lines.len() - 1].to_string() + &after_cursor;
                self.lines
                    .insert(self.cursor_line + paste_lines.len() - 1, final_line);

                // Move cursor to end of pasted content
                self.cursor_line += paste_lines.len() - 1;
                self.cursor_column = paste_lines[paste_lines.len() - 1].len();
            }
        }

        self.modified = true;
        Ok(byte_count)
    }

    pub fn insert_text_at_line(
        &mut self,
        text: &str,
        line_num: usize,
    ) -> Result<usize, std::io::Error> {
        if text.is_empty() {
            return Ok(0);
        }

        let new_lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        let byte_count = text.len();

        // Insert after the specified line (0-based internally, but line_num is 1-based from user)
        let insert_pos = if line_num == 0 {
            0 // Special case: insert at beginning
        } else {
            line_num.min(self.lines.len()) // Insert after line_num, clamped to end
        };

        // Insert the new lines
        for (i, line) in new_lines.into_iter().enumerate() {
            self.lines.insert(insert_pos + i, line);
        }

        self.modified = true;
        Ok(byte_count)
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_cursor_column();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.clamp_cursor_column();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_column > 0 {
            self.cursor_column -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_column < line_len {
            self.cursor_column += 1;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        // Record undo action
        self.undo_manager
            .add_action(crate::undo::UndoAction::InsertText {
                line: self.cursor_line,
                column: self.cursor_column,
                text: c.to_string(),
            });

        self.lines[self.cursor_line].insert(self.cursor_column, c);
        self.cursor_column += 1;
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        let current_line = &self.lines[self.cursor_line];
        let new_line = current_line[self.cursor_column..].to_string();

        // Record undo action for splitting the line
        self.undo_manager
            .add_action(crate::undo::UndoAction::SplitLine {
                line: self.cursor_line,
                column: self.cursor_column,
                text: new_line.clone(),
            });

        self.lines[self.cursor_line] = current_line[..self.cursor_column].to_string();

        self.cursor_line += 1;
        self.cursor_column = 0;
        self.lines.insert(self.cursor_line, new_line);
        self.modified = true;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_column > 0 {
            // Delete character before cursor
            let deleted_char = self.lines[self.cursor_line]
                .chars()
                .nth(self.cursor_column - 1)
                .unwrap();
            self.undo_manager
                .add_action(crate::undo::UndoAction::DeleteText {
                    line: self.cursor_line,
                    column: self.cursor_column - 1,
                    text: deleted_char.to_string(),
                });

            self.lines[self.cursor_line].remove(self.cursor_column - 1);
            self.cursor_column -= 1;
            self.modified = true;
        } else if self.cursor_line > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_line);
            let previous_line_len = self.lines[self.cursor_line - 1].len();

            // Record undo action for joining lines
            self.undo_manager
                .add_action(crate::undo::UndoAction::JoinLines {
                    line: self.cursor_line - 1,
                    separator: String::new(),
                    second_line_text: current_line.clone(),
                });

            self.cursor_line -= 1;
            self.cursor_column = previous_line_len;
            self.lines[self.cursor_line].push_str(&current_line);
            self.modified = true;
        }
    }

    pub fn delete_char_forward(&mut self) {
        let line = &mut self.lines[self.cursor_line];
        if self.cursor_column < line.len() {
            // Delete character at cursor
            let deleted_char = line.chars().nth(self.cursor_column).unwrap();
            self.undo_manager
                .add_action(crate::undo::UndoAction::DeleteText {
                    line: self.cursor_line,
                    column: self.cursor_column,
                    text: deleted_char.to_string(),
                });
            line.remove(self.cursor_column);
            self.modified = true;
        } else if self.cursor_line < self.lines.len() - 1 {
            // Join with next line
            let next_line = self.lines.remove(self.cursor_line + 1);

            // Record undo action for joining lines
            self.undo_manager
                .add_action(crate::undo::UndoAction::JoinLines {
                    line: self.cursor_line,
                    separator: String::new(),
                    second_line_text: next_line.clone(),
                });

            self.lines[self.cursor_line].push_str(&next_line);
            self.modified = true;
        }
    }

    pub fn delete_line(&mut self) {
        if self.lines.len() > 1 {
            self.lines.remove(self.cursor_line);
            if self.cursor_line >= self.lines.len() {
                self.cursor_line = self.lines.len() - 1;
            }
            self.cursor_column = 0;
            self.modified = true;
        } else {
            // If only one line, clear it
            self.lines[0].clear();
            self.cursor_column = 0;
            self.modified = true;
        }
    }

    pub fn delete_to_end_of_line(&mut self) {
        let line = &mut self.lines[self.cursor_line];
        if self.cursor_column < line.len() {
            line.truncate(self.cursor_column);
            self.modified = true;
            // Move cursor to end of line if it's now beyond the line
            if self.cursor_column > 0 && self.cursor_column >= line.len() {
                self.cursor_column = line.len().saturating_sub(1);
            }
        }
    }

    pub fn delete_word_forward(&mut self) {
        let original_line = self.cursor_line;
        let original_column = self.cursor_column;

        // Move to end of word to delete
        self.move_word_forward();

        // Delete from original position to current position
        if self.cursor_line == original_line {
            // Same line deletion
            let line = &mut self.lines[self.cursor_line];
            if original_column < line.len() && self.cursor_column <= line.len() {
                line.drain(original_column..self.cursor_column);
                self.cursor_column = original_column;
                self.modified = true;
            }
        } else {
            // Multi-line deletion - delete from original position to end of original line
            // then delete complete lines, then delete from start of final line to cursor
            let mut lines_to_remove = Vec::new();

            // Truncate original line
            self.lines[original_line].truncate(original_column);

            // Mark intermediate lines for removal
            for i in (original_line + 1)..self.cursor_line {
                lines_to_remove.push(i);
            }

            // Handle final line
            if self.cursor_line < self.lines.len() {
                let final_line_content =
                    self.lines[self.cursor_line][self.cursor_column..].to_string();
                self.lines[original_line].push_str(&final_line_content);
            }

            // Remove lines in reverse order to maintain indices
            lines_to_remove.push(self.cursor_line);
            for &line_idx in lines_to_remove.iter().rev() {
                if line_idx < self.lines.len() && line_idx > original_line {
                    self.lines.remove(line_idx);
                }
            }

            // Reset cursor position
            self.cursor_line = original_line;
            self.cursor_column = original_column;
            self.modified = true;
        }
    }

    pub fn delete_big_word_forward(&mut self) {
        let original_line = self.cursor_line;
        let original_column = self.cursor_column;

        // Move to end of big word to delete
        self.move_big_word_forward();

        // Delete from original position to current position
        if self.cursor_line == original_line {
            // Same line deletion
            let line = &mut self.lines[self.cursor_line];
            if original_column < line.len() && self.cursor_column <= line.len() {
                line.drain(original_column..self.cursor_column);
                self.cursor_column = original_column;
                self.modified = true;
            }
        } else {
            // Multi-line deletion
            let mut lines_to_remove = Vec::new();

            // Truncate original line
            self.lines[original_line].truncate(original_column);

            // Mark intermediate lines for removal
            for i in (original_line + 1)..self.cursor_line {
                lines_to_remove.push(i);
            }

            // Handle final line
            if self.cursor_line < self.lines.len() {
                let final_line_content =
                    self.lines[self.cursor_line][self.cursor_column..].to_string();
                self.lines[original_line].push_str(&final_line_content);
            }

            // Remove lines in reverse order to maintain indices
            lines_to_remove.push(self.cursor_line);
            for &line_idx in lines_to_remove.iter().rev() {
                if line_idx < self.lines.len() && line_idx > original_line {
                    self.lines.remove(line_idx);
                }
            }

            // Reset cursor position
            self.cursor_line = original_line;
            self.cursor_column = original_column;
            self.modified = true;
        }
    }

    pub fn delete_char_backward(&mut self) {
        if self.cursor_column > 0 {
            // Delete character before cursor
            let line = &mut self.lines[self.cursor_line];
            let deleted_char = line.chars().nth(self.cursor_column - 1).unwrap();
            self.undo_manager
                .add_action(crate::undo::UndoAction::DeleteText {
                    line: self.cursor_line,
                    column: self.cursor_column - 1,
                    text: deleted_char.to_string(),
                });
            line.remove(self.cursor_column - 1);
            self.cursor_column -= 1;
            self.modified = true;
        } else if self.cursor_line > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_line);
            let previous_line_len = self.lines[self.cursor_line - 1].len();

            // Record undo action for joining lines
            self.undo_manager
                .add_action(crate::undo::UndoAction::JoinLines {
                    line: self.cursor_line - 1,
                    separator: String::new(),
                    second_line_text: current_line.clone(),
                });

            self.cursor_line -= 1;
            self.cursor_column = previous_line_len;
            self.lines[self.cursor_line].push_str(&current_line);
            self.modified = true;
        }
    }

    pub fn delete_word_backward(&mut self) {
        let original_line = self.cursor_line;
        let original_column = self.cursor_column;

        // Move to start of word backward
        self.move_word_backward();

        // Delete from current position to original position
        if self.cursor_line == original_line {
            // Same line deletion
            let line = &mut self.lines[self.cursor_line];
            if self.cursor_column < original_column && original_column <= line.len() {
                line.drain(self.cursor_column..original_column);
                self.modified = true;
            }
        } else {
            // Multi-line deletion - delete from cursor to end of current line,
            // then delete complete lines, then delete from start of final line to original position
            let start_line = self.cursor_line;
            let start_column = self.cursor_column;

            // Delete from start position to end of start line
            if start_column < self.lines[start_line].len() {
                self.lines[start_line].drain(start_column..);
            }

            // Delete complete intermediate lines
            let lines_to_remove: Vec<usize> = ((start_line + 1)..original_line).collect();
            for &line_idx in lines_to_remove.iter().rev() {
                if line_idx < self.lines.len() {
                    self.lines.remove(line_idx);
                }
            }

            // Delete from start of final line to original column
            let final_line_idx = start_line + 1;
            if final_line_idx < self.lines.len() && original_column > 0 {
                let line_len = self.lines[final_line_idx].len();
                self.lines[final_line_idx].drain(0..original_column.min(line_len));
            }

            // Join the two lines
            if final_line_idx < self.lines.len() {
                let line_to_join = self.lines.remove(final_line_idx);
                self.lines[start_line].push_str(&line_to_join);
            }

            self.cursor_line = start_line;
            self.cursor_column = start_column;
            self.modified = true;
        }
    }

    pub fn delete_big_word_backward(&mut self) {
        let original_line = self.cursor_line;
        let original_column = self.cursor_column;

        // Move to start of big word backward
        self.move_big_word_backward();

        // Delete from current position to original position (same logic as delete_word_backward)
        if self.cursor_line == original_line {
            let line = &mut self.lines[self.cursor_line];
            if self.cursor_column < original_column && original_column <= line.len() {
                line.drain(self.cursor_column..original_column);
                self.modified = true;
            }
        } else {
            // Multi-line deletion (same as word backward)
            let start_line = self.cursor_line;
            let start_column = self.cursor_column;

            if start_column < self.lines[start_line].len() {
                self.lines[start_line].drain(start_column..);
            }

            let lines_to_remove: Vec<usize> = ((start_line + 1)..original_line).collect();
            for &line_idx in lines_to_remove.iter().rev() {
                if line_idx < self.lines.len() {
                    self.lines.remove(line_idx);
                }
            }

            let final_line_idx = start_line + 1;
            if final_line_idx < self.lines.len() && original_column > 0 {
                let line_len = self.lines[final_line_idx].len();
                self.lines[final_line_idx].drain(0..original_column.min(line_len));
            }

            if final_line_idx < self.lines.len() {
                let line_to_join = self.lines.remove(final_line_idx);
                self.lines[start_line].push_str(&line_to_join);
            }

            self.cursor_line = start_line;
            self.cursor_column = start_column;
            self.modified = true;
        }
    }

    pub fn delete_to_end_of_word(&mut self) {
        let original_line = self.cursor_line;
        let original_column = self.cursor_column;

        // Move to end of word
        self.move_word_end();

        // Delete from original position to current position
        if self.cursor_line == original_line {
            let line = &mut self.lines[self.cursor_line];
            if original_column < self.cursor_column && self.cursor_column <= line.len() {
                line.drain(original_column..self.cursor_column);
                self.cursor_column = original_column;
                self.modified = true;
            }
        }
        // Note: word end movement typically doesn't cross lines, so we don't handle multi-line case
    }

    pub fn delete_to_end_of_big_word(&mut self) {
        let original_line = self.cursor_line;
        let original_column = self.cursor_column;

        // Move to end of big word
        self.move_big_word_end();

        // Delete from original position to current position
        if self.cursor_line == original_line {
            let line = &mut self.lines[self.cursor_line];
            if original_column < self.cursor_column && self.cursor_column <= line.len() {
                line.drain(original_column..self.cursor_column);
                self.cursor_column = original_column;
                self.modified = true;
            }
        }
    }

    pub fn delete_to_start_of_line(&mut self) {
        let line = &mut self.lines[self.cursor_line];
        if self.cursor_column > 0 {
            line.drain(0..self.cursor_column);
            self.cursor_column = 0;
            self.modified = true;
        }
    }

    pub fn delete_to_first_non_whitespace(&mut self) {
        let line = &self.lines[self.cursor_line].clone();
        let first_non_ws = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);

        if self.cursor_column > first_non_ws {
            self.lines[self.cursor_line].drain(first_non_ws..self.cursor_column);
            self.cursor_column = first_non_ws;
            self.modified = true;
        }
    }

    pub fn delete_to_end_of_file(&mut self) {
        if self.cursor_line < self.lines.len() - 1
            || self.cursor_column < self.lines[self.cursor_line].len()
        {
            // Delete from cursor to end of current line
            let line = &mut self.lines[self.cursor_line];
            line.drain(self.cursor_column..);

            // Delete all subsequent lines
            if self.cursor_line < self.lines.len() - 1 {
                self.lines.drain((self.cursor_line + 1)..);
            }

            self.modified = true;
        }
    }

    pub fn delete_to_start_of_file(&mut self) {
        if self.cursor_line > 0 || self.cursor_column > 0 {
            // Delete from start of current line to cursor
            let line = &mut self.lines[self.cursor_line];
            line.drain(0..self.cursor_column);

            // Delete all previous lines
            if self.cursor_line > 0 {
                self.lines.drain(0..self.cursor_line);
            }

            self.cursor_line = 0;
            self.cursor_column = 0;
            self.modified = true;
        }
    }

    pub fn substitute_char(&mut self) {
        // Delete current character and enter insert mode
        self.delete_char_forward();
    }

    pub fn substitute_line(&mut self) {
        // Clear current line and move cursor to beginning
        self.lines[self.cursor_line].clear();
        self.cursor_column = 0;
        self.modified = true;
    }

    pub fn delete_until_char(&mut self, target: char) {
        let line = &self.lines[self.cursor_line];
        if let Some(pos) = line[self.cursor_column + 1..].find(target) {
            let end_pos = self.cursor_column + 1 + pos;
            self.lines[self.cursor_line].drain(self.cursor_column..end_pos);
            self.modified = true;
        }
    }

    pub fn delete_until_char_backward(&mut self, target: char) {
        let line = &self.lines[self.cursor_line];
        if let Some(pos) = line[..self.cursor_column].rfind(target) {
            self.lines[self.cursor_line].drain((pos + 1)..self.cursor_column);
            self.cursor_column = pos + 1;
            self.modified = true;
        }
    }

    pub fn delete_find_char(&mut self, target: char) {
        let line = &self.lines[self.cursor_line];
        if let Some(pos) = line[self.cursor_column + 1..].find(target) {
            let end_pos = self.cursor_column + 1 + pos + 1; // Include the target char
            self.lines[self.cursor_line].drain(self.cursor_column..end_pos);
            self.modified = true;
        }
    }

    pub fn delete_find_char_backward(&mut self, target: char) {
        let line = &self.lines[self.cursor_line];
        if let Some(pos) = line[..self.cursor_column].rfind(target) {
            self.lines[self.cursor_line].drain(pos..self.cursor_column);
            self.cursor_column = pos;
            self.modified = true;
        }
    }

    pub fn open_line_below(&mut self) {
        self.cursor_line += 1;
        self.cursor_column = 0;
        self.lines.insert(self.cursor_line, String::new());
        self.modified = true;
    }

    pub fn open_line_above(&mut self) {
        self.lines.insert(self.cursor_line, String::new());
        self.cursor_column = 0;
        self.modified = true;
    }

    pub fn clamp_cursor_column(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_column > line_len {
            self.cursor_column = line_len;
        }
    }

    pub fn set_expand_tab(&mut self, expand: bool) {
        self.expand_tab = expand;
    }

    pub fn tabs_to_spaces(&mut self, tab_width: usize) -> usize {
        let mut changed_lines = 0;
        let spaces = " ".repeat(tab_width);

        for line in &mut self.lines {
            if line.contains('\t') {
                *line = line.replace('\t', &spaces);
                changed_lines += 1;
            }
        }

        if changed_lines > 0 {
            self.modified = true;
        }

        changed_lines
    }

    pub fn spaces_to_tabs(&mut self, tab_width: usize) -> usize {
        let mut changed_lines = 0;
        let spaces = " ".repeat(tab_width);

        for line in &mut self.lines {
            if line.contains(&spaces) {
                *line = line.replace(&spaces, "\t");
                changed_lines += 1;
            }
        }

        if changed_lines > 0 {
            self.modified = true;
        }

        changed_lines
    }

    /// Convert Unicode characters to their ASCII equivalents
    pub fn ascii_normalize(&mut self) -> usize {
        let mut changed_lines = 0;

        for line in &mut self.lines {
            let normalized = Self::normalize_to_ascii(line);
            if normalized != *line {
                *line = normalized;
                changed_lines += 1;
            }
        }

        if changed_lines > 0 {
            self.modified = true;
        }

        changed_lines
    }

    /// Normalize various Unicode characters to their ASCII equivalents
    fn normalize_to_ascii(text: &str) -> String {
        let mut result = String::with_capacity(text.len());

        for ch in text.chars() {
            match ch {
                // Various space characters → ASCII space
                '\u{00A0}'  // Non-breaking space
                | '\u{1680}' // Ogham space mark
                | '\u{2000}' // En quad
                | '\u{2001}' // Em quad
                | '\u{2002}' // En space
                | '\u{2003}' // Em space
                | '\u{2004}' // Three-per-em space
                | '\u{2005}' // Four-per-em space
                | '\u{2006}' // Six-per-em space
                | '\u{2007}' // Figure space
                | '\u{2008}' // Punctuation space
                | '\u{2009}' // Thin space
                | '\u{200A}' // Hair space
                | '\u{202F}' // Narrow no-break space
                | '\u{205F}' // Medium mathematical space
                | '\u{3000}' // Ideographic space
                => result.push(' '),

                // Various dash/hyphen characters → ASCII hyphen-minus
                '\u{2010}' // Hyphen
                | '\u{2011}' // Non-breaking hyphen
                | '\u{2012}' // Figure dash
                | '\u{2013}' // En dash
                | '\u{2014}' // Em dash
                | '\u{2015}' // Horizontal bar
                | '\u{2212}' // Minus sign
                | '\u{FE58}' // Small em dash
                | '\u{FE63}' // Small hyphen-minus
                | '\u{FF0D}' // Fullwidth hyphen-minus
                => result.push('-'),

                // Various quotation marks → ASCII quotes
                '\u{2018}' // Left single quotation mark
                | '\u{2019}' // Right single quotation mark
                | '\u{201A}' // Single low-9 quotation mark
                | '\u{201B}' // Single high-reversed-9 quotation mark
                | '\u{2032}' // Prime
                | '\u{2035}' // Reversed prime
                => result.push('\''),

                '\u{201C}' // Left double quotation mark
                | '\u{201D}' // Right double quotation mark
                | '\u{201E}' // Double low-9 quotation mark
                | '\u{201F}' // Double high-reversed-9 quotation mark
                | '\u{2033}' // Double prime
                | '\u{2036}' // Reversed double prime
                | '\u{301D}' // Reversed double prime quotation mark
                | '\u{301E}' // Double prime quotation mark
                => result.push('"'),

                // Various ellipsis characters → three ASCII periods
                '\u{2026}' // Horizontal ellipsis
                => result.push_str("..."),

                // Keep ASCII characters as-is
                _ if ch.is_ascii() => result.push(ch),

                // For non-ASCII characters, try fallback conversion
                _ => {
                    if let Some(ascii_equivalent) = Self::unicode_to_ascii_fallback(ch) {
                        result.push_str(ascii_equivalent);
                    } else {
                        // If no ASCII equivalent found, keep the original character
                        result.push(ch);
                    }
                }
            }
        }

        result
    }

    /// Fallback method for Unicode to ASCII conversion
    fn unicode_to_ascii_fallback(ch: char) -> Option<&'static str> {
        match ch {
            // Accented letters → base letters (uppercase)
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' => Some("A"),
            'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => Some("E"),
            'Ì' | 'Í' | 'Î' | 'Ï' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => Some("I"),
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' => Some("O"),
            'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => Some("U"),
            'Ñ' => Some("N"),
            'Ç' => Some("C"),
            'Ý' | 'Ÿ' => Some("Y"),

            // Accented letters → base letters (lowercase)
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => Some("a"),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => Some("e"),
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'ĭ' | 'į' | 'ı' => Some("i"),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => Some("o"),
            'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => Some("u"),
            'ñ' => Some("n"),
            'ç' => Some("c"),
            'ý' | 'ÿ' => Some("y"),

            _ => None,
        }
    }

    pub fn insert_tab_or_spaces(&mut self, tab_width: usize) {
        if self.expand_tab {
            // Insert spaces
            let spaces = " ".repeat(tab_width);
            self.undo_manager
                .add_action(crate::undo::UndoAction::InsertText {
                    line: self.cursor_line,
                    column: self.cursor_column,
                    text: spaces.clone(),
                });
            self.lines[self.cursor_line].insert_str(self.cursor_column, &spaces);
            self.cursor_column += tab_width;
        } else {
            // Insert actual tab
            self.undo_manager
                .add_action(crate::undo::UndoAction::InsertText {
                    line: self.cursor_line,
                    column: self.cursor_column,
                    text: "\t".to_string(),
                });
            self.lines[self.cursor_line].insert(self.cursor_column, '\t');
            self.cursor_column += 1;
        }
        self.modified = true;
    }

    pub fn indent_line(&mut self, tab_width: usize, use_spaces: bool) {
        let indent = if use_spaces {
            " ".repeat(tab_width)
        } else {
            "\t".to_string()
        };

        self.lines[self.cursor_line].insert_str(0, &indent);
        self.cursor_column += if use_spaces { tab_width } else { 1 };
        self.modified = true;
    }

    pub fn indent_lines(
        &mut self,
        start_line: usize,
        count: usize,
        tab_width: usize,
        use_spaces: bool,
    ) {
        let end_line = std::cmp::min(start_line + count, self.lines.len());
        let indent = if use_spaces {
            " ".repeat(tab_width)
        } else {
            "\t".to_string()
        };

        for line_idx in start_line..end_line {
            self.lines[line_idx].insert_str(0, &indent);
        }

        self.modified = true;
    }

    pub fn dedent_line(&mut self, tab_width: usize) {
        let line = &mut self.lines[self.cursor_line];

        // Try to remove a tab first
        if line.starts_with('\t') {
            line.remove(0);
            if self.cursor_column > 0 {
                self.cursor_column -= 1;
            }
            self.modified = true;
        } else {
            // Try to remove spaces up to tab_width
            let mut removed = 0;
            while removed < tab_width && line.starts_with(' ') {
                line.remove(0);
                removed += 1;
            }
            if removed > 0 {
                self.cursor_column = self.cursor_column.saturating_sub(removed);
                self.modified = true;
            }
        }
    }

    pub fn dedent_lines(&mut self, start_line: usize, count: usize, tab_width: usize) {
        let end_line = std::cmp::min(start_line + count, self.lines.len());

        for line_idx in start_line..end_line {
            let line = &mut self.lines[line_idx];

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

        if start_line < end_line {
            self.modified = true;
        }
    }

    /// Set a local mark (a-z) for this buffer
    pub fn set_local_mark(
        &mut self,
        mark_char: char,
        line: usize,
        column: usize,
    ) -> Result<(), String> {
        if mark_char.is_ascii_lowercase() {
            self.local_marks.insert(mark_char, (line, column));
            Ok(())
        } else {
            Err(format!("Invalid local mark character: {mark_char}"))
        }
    }

    /// Get a local mark (a-z) for this buffer
    pub fn get_local_mark(&self, mark_char: char) -> Option<(usize, usize)> {
        if mark_char.is_ascii_lowercase() {
            self.local_marks.get(&mark_char).copied()
        } else {
            None
        }
    }

    /// Clear all local marks for this buffer
    pub fn clear_local_marks(&mut self) {
        self.local_marks.clear();
    }

    /// Get all local marks for this buffer (for :marks command)
    pub fn get_all_local_marks(&self) -> &HashMap<char, (usize, usize)> {
        &self.local_marks
    }

    // Yank (copy) operations - return text to be copied to registers

    pub fn yank_line(&self) -> String {
        if !self.lines.is_empty() && self.cursor_line < self.lines.len() {
            self.lines[self.cursor_line].clone()
        } else {
            String::new()
        }
    }

    pub fn yank_to_end_of_line(&self) -> String {
        if self.cursor_line < self.lines.len() {
            let line = &self.lines[self.cursor_line];
            if self.cursor_column <= line.len() {
                line[self.cursor_column..].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    pub fn yank_word_forward(&self) -> String {
        let start_line = self.cursor_line;
        let start_col = self.cursor_column;

        let (end_line, end_col) = self.calculate_word_forward_position();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_big_word_forward(&self) -> String {
        let start_line = self.cursor_line;
        let start_col = self.cursor_column;

        let (end_line, end_col) = self.calculate_big_word_forward_position();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_word_backward(&self) -> String {
        let end_line = self.cursor_line;
        let end_col = self.cursor_column;

        let (start_line, start_col) = self.calculate_word_backward_position();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_big_word_backward(&self) -> String {
        let end_line = self.cursor_line;
        let end_col = self.cursor_column;

        let (start_line, start_col) = self.calculate_big_word_backward_position();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_to_end_of_word(&self) -> String {
        let start_line = self.cursor_line;
        let start_col = self.cursor_column;

        let (end_line, end_col_raw) = self.calculate_word_end_position();
        let end_col = end_col_raw + 1; // Include the character at cursor

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_to_end_of_big_word(&self) -> String {
        let start_line = self.cursor_line;
        let start_col = self.cursor_column;

        let (end_line, end_col_raw) = self.calculate_big_word_end_position();
        let end_col = end_col_raw + 1; // Include the character at cursor

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_to_start_of_line(&self) -> String {
        if self.cursor_line < self.lines.len() {
            let line = &self.lines[self.cursor_line];
            if self.cursor_column <= line.len() {
                line[..self.cursor_column].to_string()
            } else {
                line.to_string()
            }
        } else {
            String::new()
        }
    }

    pub fn yank_to_first_non_whitespace(&self) -> String {
        if self.cursor_line < self.lines.len() {
            let line = &self.lines[self.cursor_line];
            let first_non_ws = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
            if self.cursor_column >= first_non_ws {
                line[first_non_ws..self.cursor_column].to_string()
            } else {
                line[self.cursor_column..first_non_ws].to_string()
            }
        } else {
            String::new()
        }
    }

    pub fn yank_to_end_of_file(&self) -> String {
        let start_line = self.cursor_line;
        let start_col = self.cursor_column;
        let end_line = self.lines.len().saturating_sub(1);
        let end_col = if !self.lines.is_empty() {
            self.lines[end_line].len()
        } else {
            0
        };

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_to_start_of_file(&self) -> String {
        let start_line = 0;
        let start_col = 0;
        let end_line = self.cursor_line;
        let end_col = self.cursor_column;

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_until_char(&self, target: char) -> String {
        let start_line = self.cursor_line;
        let start_col = self.cursor_column;

        if let Some((end_line, end_col)) = self.find_char_position(target, true, true) {
            self.get_text_range(start_line, start_col, end_line, end_col)
        } else {
            String::new()
        }
    }

    pub fn yank_until_char_backward(&self, target: char) -> String {
        let end_line = self.cursor_line;
        let end_col = self.cursor_column;

        if let Some((start_line, start_col)) = self.find_char_position(target, false, true) {
            self.get_text_range(start_line, start_col + 1, end_line, end_col)
        } else {
            String::new()
        }
    }

    pub fn yank_find_char(&self, target: char) -> String {
        let start_line = self.cursor_line;
        let start_col = self.cursor_column;

        if let Some((end_line, end_col)) = self.find_char_position(target, true, false) {
            self.get_text_range(start_line, start_col, end_line, end_col + 1)
        } else {
            String::new()
        }
    }

    pub fn yank_find_char_backward(&self, target: char) -> String {
        let end_line = self.cursor_line;
        let end_col = self.cursor_column;

        if let Some((start_line, start_col)) = self.find_char_position(target, false, false) {
            self.get_text_range(start_line, start_col, end_line, end_col)
        } else {
            String::new()
        }
    }

    // Change (delete + insert mode) operations - return deleted text and modify document

    #[allow(dead_code)]
    pub fn change_char(&mut self) -> String {
        let deleted = self.get_char_at_cursor();
        self.delete_char_forward();
        deleted
    }

    pub fn change_line(&mut self) -> String {
        let deleted = self.lines[self.cursor_line].clone();
        self.lines[self.cursor_line].clear();
        self.cursor_column = 0;
        self.modified = true;
        deleted
    }

    pub fn change_to_end_of_line(&mut self) -> String {
        let deleted = self.yank_to_end_of_line();
        self.delete_to_end_of_line();
        deleted
    }

    pub fn change_word_forward(&mut self) -> String {
        let deleted = self.yank_word_forward();
        self.delete_word_forward();
        deleted
    }

    pub fn change_big_word_forward(&mut self) -> String {
        let deleted = self.yank_big_word_forward();
        self.delete_big_word_forward();
        deleted
    }

    pub fn change_word_backward(&mut self) -> String {
        let deleted = self.yank_word_backward();
        self.delete_word_backward();
        deleted
    }

    pub fn change_big_word_backward(&mut self) -> String {
        let deleted = self.yank_big_word_backward();
        self.delete_big_word_backward();
        deleted
    }

    pub fn change_to_end_of_word(&mut self) -> String {
        let deleted = self.yank_to_end_of_word();
        self.delete_to_end_of_word();
        deleted
    }

    pub fn change_to_end_of_big_word(&mut self) -> String {
        let deleted = self.yank_to_end_of_big_word();
        self.delete_to_end_of_big_word();
        deleted
    }

    pub fn change_to_start_of_line(&mut self) -> String {
        let deleted = self.yank_to_start_of_line();
        self.delete_to_start_of_line();
        deleted
    }

    pub fn change_to_first_non_whitespace(&mut self) -> String {
        let deleted = self.yank_to_first_non_whitespace();
        self.delete_to_first_non_whitespace();
        deleted
    }

    pub fn change_to_end_of_file(&mut self) -> String {
        let deleted = self.yank_to_end_of_file();
        self.delete_to_end_of_file();
        deleted
    }

    pub fn change_to_start_of_file(&mut self) -> String {
        let deleted = self.yank_to_start_of_file();
        self.delete_to_start_of_file();
        deleted
    }

    pub fn change_until_char(&mut self, target: char) -> String {
        let deleted = self.yank_until_char(target);
        self.delete_until_char(target);
        deleted
    }

    pub fn change_until_char_backward(&mut self, target: char) -> String {
        let deleted = self.yank_until_char_backward(target);
        self.delete_until_char_backward(target);
        deleted
    }

    pub fn change_find_char(&mut self, target: char) -> String {
        let deleted = self.yank_find_char(target);
        self.delete_find_char(target);
        deleted
    }

    pub fn change_find_char_backward(&mut self, target: char) -> String {
        let deleted = self.yank_find_char_backward(target);
        self.delete_find_char_backward(target);
        deleted
    }

    // Helper method to get character at cursor
    #[allow(dead_code)]
    fn get_char_at_cursor(&self) -> String {
        if self.cursor_line < self.lines.len() {
            let line = &self.lines[self.cursor_line];
            if self.cursor_column < line.len() {
                let chars: Vec<char> = line.chars().collect();
                chars[self.cursor_column].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    /// Join the current line with the next line (vim J command)
    /// Returns true if lines were joined, false if at last line
    pub fn join_lines(&mut self) -> bool {
        // Check if we can join (not at the last line)
        if self.cursor_line >= self.lines.len() - 1 {
            return false;
        }

        let current_line = self.cursor_line;
        let next_line = current_line + 1;

        // Save original state for undo
        let _original_cursor = (self.cursor_line, self.cursor_column);

        // Get the lines to join
        let mut current_line_text = self.lines[current_line].clone();
        let next_line_text = self.lines[next_line].clone();

        // Remember cursor position before join for undo
        let join_position = current_line_text.len();

        // Add a space between lines unless the current line ends with whitespace
        // or the next line starts with whitespace (vim behavior)
        let needs_space = !current_line_text.ends_with(' ')
            && !current_line_text.ends_with('\t')
            && !next_line_text.starts_with(' ')
            && !next_line_text.starts_with('\t')
            && !current_line_text.is_empty()
            && !next_line_text.is_empty();

        if needs_space {
            current_line_text.push(' ');
        }

        // Trim leading whitespace from the next line
        let trimmed_next = next_line_text.trim_start();
        current_line_text.push_str(trimmed_next);

        // Record undo information
        let second_line_text = self.lines[next_line].clone();
        self.undo_manager
            .add_action(crate::undo::UndoAction::JoinLines {
                line: current_line,
                separator: if needs_space {
                    " ".to_string()
                } else {
                    String::new()
                },
                second_line_text,
            });

        // Update the document
        self.lines[current_line] = current_line_text;
        self.lines.remove(next_line);

        // Position cursor at the join point
        self.cursor_column = if needs_space {
            join_position + 1
        } else {
            join_position
        };

        self.modified = true;
        true
    }

    /// Toggle case of character at cursor position
    /// Returns true if a character was toggled, false if no character at cursor
    pub fn toggle_case_char(&mut self) -> bool {
        if self.cursor_line >= self.lines.len() {
            return false;
        }

        let line = &mut self.lines[self.cursor_line];
        if self.cursor_column >= line.len() {
            return false;
        }

        let chars: Vec<char> = line.chars().collect();
        if self.cursor_column >= chars.len() {
            return false;
        }

        let original_char = chars[self.cursor_column];
        let new_char = if original_char.is_uppercase() {
            original_char.to_lowercase().collect::<String>()
        } else if original_char.is_lowercase() {
            original_char.to_uppercase().collect::<String>()
        } else {
            return false; // No case to toggle
        };

        // Record undo action
        self.undo_manager
            .add_action(crate::undo::UndoAction::DeleteText {
                line: self.cursor_line,
                column: self.cursor_column,
                text: original_char.to_string(),
            });
        self.undo_manager
            .add_action(crate::undo::UndoAction::InsertText {
                line: self.cursor_line,
                column: self.cursor_column,
                text: new_char.clone(),
            });

        // Replace character
        let mut new_chars = chars;
        new_chars[self.cursor_column] = new_char.chars().next().unwrap();
        *line = new_chars.into_iter().collect();

        // Move cursor forward (vim behavior)
        if self.cursor_column < line.len() - 1 {
            self.cursor_column += 1;
        }

        self.modified = true;
        true
    }

    /// Convert current line to lowercase
    pub fn lowercase_line(&mut self) {
        if self.cursor_line >= self.lines.len() {
            return;
        }

        let line = &mut self.lines[self.cursor_line];
        let original_line = line.clone();
        let lowercase_line = line.to_lowercase();

        if original_line != lowercase_line {
            // Record undo action
            self.undo_manager
                .add_action(crate::undo::UndoAction::DeleteText {
                    line: self.cursor_line,
                    column: 0,
                    text: original_line,
                });
            self.undo_manager
                .add_action(crate::undo::UndoAction::InsertText {
                    line: self.cursor_line,
                    column: 0,
                    text: lowercase_line.clone(),
                });

            *line = lowercase_line;
            self.modified = true;
            // Ensure cursor column remains valid after line modification
            self.clamp_cursor_column();
        }
    }

    /// Convert current line to uppercase
    pub fn uppercase_line(&mut self) {
        if self.cursor_line >= self.lines.len() {
            return;
        }

        let line = &mut self.lines[self.cursor_line];
        let original_line = line.clone();
        let uppercase_line = line.to_uppercase();

        if original_line != uppercase_line {
            // Record undo action
            self.undo_manager
                .add_action(crate::undo::UndoAction::DeleteText {
                    line: self.cursor_line,
                    column: 0,
                    text: original_line,
                });
            self.undo_manager
                .add_action(crate::undo::UndoAction::InsertText {
                    line: self.cursor_line,
                    column: 0,
                    text: uppercase_line.clone(),
                });

            *line = uppercase_line;
            self.modified = true;
            // Ensure cursor column remains valid after line modification
            self.clamp_cursor_column();
        }
    }

    // Helper method to get text in a range
    fn get_text_range(
        &self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> String {
        if start_line >= self.lines.len() || end_line >= self.lines.len() {
            return String::new();
        }

        if start_line == end_line {
            let line = &self.lines[start_line];
            let start = start_col.min(line.len());
            let end = end_col.min(line.len());
            if start < end {
                line[start..end].to_string()
            } else {
                String::new()
            }
        } else {
            let mut result = String::new();

            // First line
            let first_line = &self.lines[start_line];
            let start = start_col.min(first_line.len());
            if start < first_line.len() {
                result.push_str(&first_line[start..]);
            }
            result.push('\n');

            // Middle lines
            for i in (start_line + 1)..end_line {
                result.push_str(&self.lines[i]);
                result.push('\n');
            }

            // Last line
            if end_line > start_line {
                let last_line = &self.lines[end_line];
                let end = end_col.min(last_line.len());
                result.push_str(&last_line[..end]);
            }

            result
        }
    }

    // Position calculation functions for yank operations - eliminates document cloning
    fn calculate_word_forward_position(&self) -> (usize, usize) {
        let mut cursor_line = self.cursor_line;
        let mut cursor_column = self.cursor_column;

        let line = &self.lines[cursor_line];

        // If at end of line, move to next line
        if cursor_column >= line.len() {
            if cursor_line < self.lines.len() - 1 {
                cursor_line += 1;
                cursor_column = 0;
                // Find first non-whitespace on new line
                let new_line = &self.lines[cursor_line];
                for (i, c) in new_line.chars().enumerate() {
                    if !c.is_whitespace() {
                        cursor_column = i;
                        break;
                    }
                }
            }
            return (cursor_line, cursor_column);
        }

        let chars: Vec<char> = line.chars().collect();
        let current_char = chars[cursor_column];

        // Skip whitespace first
        if current_char.is_whitespace() {
            while cursor_column < chars.len() && chars[cursor_column].is_whitespace() {
                cursor_column += 1;
            }
        } else {
            // Determine the type of the current character
            let is_word_char = current_char.is_alphanumeric() || current_char == '_';

            if is_word_char {
                // Skip alphanumeric/underscore characters
                while cursor_column < chars.len() {
                    let c = chars[cursor_column];
                    if !(c.is_alphanumeric() || c == '_') {
                        break;
                    }
                    cursor_column += 1;
                }
            } else {
                // Skip punctuation characters (non-whitespace, non-alphanumeric)
                while cursor_column < chars.len() {
                    let c = chars[cursor_column];
                    if c.is_whitespace() || c.is_alphanumeric() || c == '_' {
                        break;
                    }
                    cursor_column += 1;
                }
            }

            // Skip whitespace after the word/punctuation
            while cursor_column < chars.len() && chars[cursor_column].is_whitespace() {
                cursor_column += 1;
            }
        }

        // If we reached end of line, move to next line
        if cursor_column >= chars.len() {
            if cursor_line < self.lines.len() - 1 {
                cursor_line += 1;
                cursor_column = 0;
                // Find first non-whitespace on new line
                let new_line = &self.lines[cursor_line];
                for (i, c) in new_line.chars().enumerate() {
                    if !c.is_whitespace() {
                        cursor_column = i;
                        break;
                    }
                }
            } else {
                // At end of document, clamp to last character
                cursor_column = if chars.is_empty() { 0 } else { chars.len() - 1 };
            }
        }

        (cursor_line, cursor_column)
    }

    fn calculate_word_backward_position(&self) -> (usize, usize) {
        let mut cursor_line = self.cursor_line;
        let mut cursor_column = self.cursor_column;

        // If at beginning of line, move to end of previous line
        if cursor_column == 0 {
            if cursor_line > 0 {
                cursor_line -= 1;
                let line = &self.lines[cursor_line];
                cursor_column = line.len();
                // Find last non-whitespace character
                let chars: Vec<char> = line.chars().collect();
                while cursor_column > 0 && chars[cursor_column - 1].is_whitespace() {
                    cursor_column -= 1;
                }
            }
            return (cursor_line, cursor_column);
        }

        let line = &self.lines[cursor_line];
        let chars: Vec<char> = line.chars().collect();

        // Move back one position first
        cursor_column -= 1;

        // Skip whitespace
        while cursor_column > 0 && chars[cursor_column].is_whitespace() {
            cursor_column -= 1;
        }

        // If we're now on a character, move to start of this word/punctuation group
        if cursor_column < chars.len() && !chars[cursor_column].is_whitespace() {
            let current_char = chars[cursor_column];
            let is_word_char = current_char.is_alphanumeric() || current_char == '_';

            if is_word_char {
                // Move to start of alphanumeric word
                while cursor_column > 0 {
                    let prev_char = chars[cursor_column - 1];
                    if !(prev_char.is_alphanumeric() || prev_char == '_') {
                        break;
                    }
                    cursor_column -= 1;
                }
            } else {
                // Move to start of punctuation group
                while cursor_column > 0 {
                    let prev_char = chars[cursor_column - 1];
                    if prev_char.is_whitespace() || prev_char.is_alphanumeric() || prev_char == '_'
                    {
                        break;
                    }
                    cursor_column -= 1;
                }
            }
        }

        (cursor_line, cursor_column)
    }

    fn calculate_word_end_position(&self) -> (usize, usize) {
        let cursor_line = self.cursor_line;
        let mut cursor_column = self.cursor_column;

        let line = &self.lines[cursor_line];
        if cursor_column < line.len() {
            let chars: Vec<char> = line.chars().collect();

            // If on whitespace, move to start of next word first
            if chars[cursor_column].is_whitespace() {
                while cursor_column < chars.len() && chars[cursor_column].is_whitespace() {
                    cursor_column += 1;
                }
            }

            // Move to end of current word
            while cursor_column < chars.len() - 1
                && (chars[cursor_column + 1].is_alphanumeric() || chars[cursor_column + 1] == '_')
            {
                cursor_column += 1;
            }
        }

        (cursor_line, cursor_column)
    }

    fn calculate_big_word_forward_position(&self) -> (usize, usize) {
        let mut cursor_line = self.cursor_line;
        let mut cursor_column = self.cursor_column;

        loop {
            let line = &self.lines[cursor_line];
            if cursor_column >= line.len() {
                if cursor_line < self.lines.len() - 1 {
                    cursor_line += 1;
                    cursor_column = 0;
                } else {
                    break;
                }
            } else {
                let chars: Vec<char> = line.chars().collect();
                let start_col = cursor_column;

                // Skip current big word (non-whitespace)
                while cursor_column < chars.len() && !chars[cursor_column].is_whitespace() {
                    cursor_column += 1;
                }

                // Skip whitespace
                while cursor_column < chars.len() && chars[cursor_column].is_whitespace() {
                    cursor_column += 1;
                }

                if cursor_column == start_col {
                    if cursor_line < self.lines.len() - 1 {
                        cursor_line += 1;
                        cursor_column = 0;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        (cursor_line, cursor_column)
    }

    fn calculate_big_word_backward_position(&self) -> (usize, usize) {
        let mut cursor_line = self.cursor_line;
        let mut cursor_column = self.cursor_column;

        loop {
            if cursor_column == 0 {
                if cursor_line > 0 {
                    cursor_line -= 1;
                    cursor_column = self.lines[cursor_line].len();
                } else {
                    break;
                }
            } else {
                let line = &self.lines[cursor_line];
                let chars: Vec<char> = line.chars().collect();

                cursor_column -= 1;

                // Skip whitespace
                while cursor_column > 0 && chars[cursor_column].is_whitespace() {
                    cursor_column -= 1;
                }

                // Move to start of big word
                while cursor_column > 0 && !chars[cursor_column - 1].is_whitespace() {
                    cursor_column -= 1;
                }

                break;
            }
        }

        (cursor_line, cursor_column)
    }

    fn calculate_big_word_end_position(&self) -> (usize, usize) {
        let cursor_line = self.cursor_line;
        let mut cursor_column = self.cursor_column;

        let line = &self.lines[cursor_line];
        if cursor_column < line.len() {
            let chars: Vec<char> = line.chars().collect();

            // If on whitespace, move to start of next word first
            if chars[cursor_column].is_whitespace() {
                while cursor_column < chars.len() && chars[cursor_column].is_whitespace() {
                    cursor_column += 1;
                }
            }

            // Move to end of current big word
            while cursor_column < chars.len() - 1 && !chars[cursor_column + 1].is_whitespace() {
                cursor_column += 1;
            }
        }

        (cursor_line, cursor_column)
    }

    fn find_char_position(
        &self,
        target: char,
        forward: bool,
        before: bool,
    ) -> Option<(usize, usize)> {
        let line = &self.lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let mut cursor_column = self.cursor_column;

        if forward {
            let start = if before {
                cursor_column
            } else {
                cursor_column + 1
            };
            for (i, ch) in chars.iter().enumerate().skip(start) {
                if *ch == target {
                    cursor_column = if before && i > 0 { i - 1 } else { i };
                    return Some((self.cursor_line, cursor_column));
                }
            }
        } else {
            let end = if before && cursor_column < chars.len() {
                cursor_column + 1
            } else {
                cursor_column
            };
            for i in (0..end).rev() {
                if chars[i] == target {
                    cursor_column = if before && i < chars.len() - 1 {
                        i + 1
                    } else {
                        i
                    };
                    return Some((self.cursor_line, cursor_column));
                }
            }
        }

        None
    }

    pub fn get_word_under_cursor(&self) -> Option<String> {
        if self.cursor_line >= self.lines.len() {
            return None;
        }

        let line = &self.lines[self.cursor_line];
        if self.cursor_column >= line.len() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
            return None;
        }

        let cursor_pos = self.cursor_column.min(chars.len().saturating_sub(1));
        let ch = chars[cursor_pos];

        // Only search for words containing alphanumeric characters or underscores
        if !ch.is_alphanumeric() && ch != '_' {
            return None;
        }

        // Find start of word (go backward)
        let mut start = cursor_pos;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Find end of word (go forward)
        let mut end = cursor_pos;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    pub fn find_matching_bracket(&self) -> Option<(usize, usize)> {
        if self.cursor_line >= self.lines.len() {
            return None;
        }

        let line = &self.lines[self.cursor_line];
        if self.cursor_column >= line.len() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() || self.cursor_column >= chars.len() {
            return None;
        }

        let cursor_char = chars[self.cursor_column];

        // Define bracket pairs
        let bracket_pairs = [('(', ')'), ('[', ']'), ('{', '}'), ('<', '>')];

        // Check if cursor is on a bracket character
        let (opening, closing, is_opening) = if let Some((open, close)) =
            bracket_pairs.iter().find(|(open, _)| *open == cursor_char)
        {
            (*open, *close, true)
        } else if let Some((open, close)) = bracket_pairs
            .iter()
            .find(|(_, close)| *close == cursor_char)
        {
            (*open, *close, false)
        } else {
            return None;
        };

        if is_opening {
            // Search forward for closing bracket
            self.find_closing_bracket(opening, closing, self.cursor_line, self.cursor_column)
        } else {
            // Search backward for opening bracket
            self.find_opening_bracket(opening, closing, self.cursor_line, self.cursor_column)
        }
    }

    /// Check if the bracket at the cursor position is unmatched
    pub fn is_unmatched_bracket(&self) -> Option<(usize, usize)> {
        if self.cursor_line >= self.lines.len() {
            return None;
        }

        let line = &self.lines[self.cursor_line];
        if self.cursor_column >= line.len() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() || self.cursor_column >= chars.len() {
            return None;
        }

        let cursor_char = chars[self.cursor_column];

        // Define bracket pairs
        let bracket_pairs = [('(', ')'), ('[', ']'), ('{', '}'), ('<', '>')];

        // Check if cursor is on a bracket character
        let (opening, closing, is_opening) = if let Some((open, close)) =
            bracket_pairs.iter().find(|(open, _)| *open == cursor_char)
        {
            (*open, *close, true)
        } else if let Some((open, close)) = bracket_pairs
            .iter()
            .find(|(_, close)| *close == cursor_char)
        {
            (*open, *close, false)
        } else {
            return None;
        };

        // Try to find the matching bracket
        let has_match = if is_opening {
            self.find_closing_bracket(opening, closing, self.cursor_line, self.cursor_column)
                .is_some()
        } else {
            self.find_opening_bracket(opening, closing, self.cursor_line, self.cursor_column)
                .is_some()
        };

        // If no match found, this bracket is unmatched
        if !has_match {
            Some((self.cursor_line, self.cursor_column))
        } else {
            None
        }
    }

    /// Find all unmatched brackets in the document
    pub fn find_all_unmatched_brackets(&self) -> Vec<(usize, usize)> {
        let mut unmatched = Vec::new();
        let bracket_pairs = [('(', ')'), ('[', ']'), ('{', '}'), ('<', '>')];

        // For each bracket type, track opening brackets and match them with closing ones
        for (opening, closing) in bracket_pairs {
            let mut stack: Vec<(usize, usize)> = Vec::new(); // Stack of opening bracket positions

            // Scan through the entire document
            for (line_idx, line) in self.lines.iter().enumerate() {
                let chars: Vec<char> = line.chars().collect();
                for (col_idx, &ch) in chars.iter().enumerate() {
                    if ch == opening {
                        // Found opening bracket, push to stack
                        stack.push((line_idx, col_idx));
                    } else if ch == closing {
                        // Found closing bracket, try to match with most recent opening
                        if stack.is_empty() {
                            // Unmatched closing bracket
                            unmatched.push((line_idx, col_idx));
                        } else {
                            // Matched pair, remove from stack
                            stack.pop();
                        }
                    }
                }
            }

            // Any remaining opening brackets are unmatched
            unmatched.extend(stack);
        }

        // Sort by position for consistent ordering
        unmatched.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        unmatched
    }

    fn find_closing_bracket(
        &self,
        opening: char,
        closing: char,
        start_line: usize,
        start_col: usize,
    ) -> Option<(usize, usize)> {
        let mut depth = 1;
        let mut line_idx = start_line;
        let mut col_idx = start_col + 1;

        while line_idx < self.lines.len() {
            let line = &self.lines[line_idx];
            let chars: Vec<char> = line.chars().collect();

            while col_idx < chars.len() {
                match chars[col_idx] {
                    ch if ch == opening => depth += 1,
                    ch if ch == closing => {
                        depth -= 1;
                        if depth == 0 {
                            return Some((line_idx, col_idx));
                        }
                    }
                    _ => {}
                }
                col_idx += 1;
            }

            line_idx += 1;
            col_idx = 0;
        }

        None
    }

    fn find_opening_bracket(
        &self,
        opening: char,
        closing: char,
        start_line: usize,
        start_col: usize,
    ) -> Option<(usize, usize)> {
        let mut depth = 1;
        let mut line_idx = start_line;
        let mut col_idx = if start_col > 0 {
            start_col - 1
        } else {
            // If we're at the beginning of a line, we need to move to the previous line
            if start_line == 0 {
                return None; // Can't go back further
            }
            // Find the first non-empty line going backwards
            let mut search_line = start_line - 1;
            loop {
                let line = &self.lines[search_line];
                let chars: Vec<char> = line.chars().collect();
                if !chars.is_empty() {
                    line_idx = search_line;
                    break chars.len() - 1;
                }
                if search_line == 0 {
                    return None; // No non-empty lines found
                }
                search_line -= 1;
            }
        };

        loop {
            let line = &self.lines[line_idx];
            let chars: Vec<char> = line.chars().collect();

            // Search backwards through the current line
            loop {
                if col_idx < chars.len() {
                    match chars[col_idx] {
                        ch if ch == closing => depth += 1,
                        ch if ch == opening => {
                            depth -= 1;
                            if depth == 0 {
                                return Some((line_idx, col_idx));
                            }
                        }
                        _ => {}
                    }
                }

                if col_idx == 0 {
                    break;
                }
                col_idx -= 1;
            }

            // Move to previous line
            if line_idx == 0 {
                break;
            }
            line_idx -= 1;
            let prev_line = &self.lines[line_idx];
            let prev_chars: Vec<char> = prev_line.chars().collect();
            if prev_chars.is_empty() {
                // Empty line - continue to next iteration to check previous line
                col_idx = 0; // Set to 0 so the inner loop will immediately break
            } else {
                col_idx = prev_chars.len() - 1;
            };
        }

        None
    }
}
