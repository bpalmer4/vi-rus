use super::undo::UndoManager;
use super::text_buffer::{TextBuffer, Position, Range};
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
    // Cursor state - MODULE PRIVATE: controlled access only  
    pub(super) cursor_line: usize,
    pub(super) cursor_column: usize,
    
    // File metadata - PUBLIC: direct access allowed for now
    pub filename: Option<PathBuf>,
    pub modified: bool,
    
    // Format settings - PUBLIC: direct access allowed for now
    pub line_ending: LineEnding,
    pub expand_tab: bool,
    
    // Internal data structures - MODULE PRIVATE: controlled access only
    local_marks: HashMap<char, (usize, usize)>, // Local marks (a-z) for this buffer
    pub(super) undo_manager: UndoManager,
    pub(super) text_buffer: TextBuffer, // Piece table backend - single source of truth
}

impl Document {
    pub fn new() -> Self {
        let text_buffer = TextBuffer::new();
        
        Self {
            cursor_line: 0,
            cursor_column: 0,
            filename: None,
            modified: false,
            line_ending: LineEnding::system_default(),
            expand_tab: true, // Default to spaces
            local_marks: HashMap::new(),
            undo_manager: UndoManager::new(),
            text_buffer,
        }
    }
    
    /// Create a new document from string content
    pub fn from_string(content: String) -> Self {
        Self {
            cursor_line: 0,
            cursor_column: 0,
            filename: None,
            modified: false,
            line_ending: LineEnding::Unix,
            expand_tab: true,
            local_marks: HashMap::new(),
            undo_manager: UndoManager::new(),
            text_buffer: TextBuffer::from_string(content),
        }
    }

    pub fn from_file(filename: PathBuf) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(&filename)?;
        let line_ending = LineEnding::detect(&content);
        
        let mut text_buffer = TextBuffer::from_string(content);
        text_buffer.set_line_ending(line_ending);

        Ok(Self {
            cursor_line: 0,
            cursor_column: 0,
            filename: Some(filename),
            modified: false,
            line_ending,
            expand_tab: true, // Default to spaces
            local_marks: HashMap::new(),
            undo_manager: UndoManager::new(),
            text_buffer,
        })
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }


    
    



    
    // === CURSOR MANAGEMENT ===
    
    /// Get the current cursor line
    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }
    
    /// Get the current cursor column
    pub fn cursor_column(&self) -> usize {
        self.cursor_column
    }
    
    /// Set cursor position with bounds checking
    pub fn set_cursor(&mut self, line: usize, column: usize) -> Result<(), String> {
        let line_count = self.line_count();
        if line >= line_count && line_count > 0 {
            return Err(format!("Line {} out of bounds (max: {})", line, line_count - 1));
        }
        
        let line_length = self.get_line_length(line);
        let safe_column = column.min(line_length);
        
        self.cursor_line = line;
        self.cursor_column = safe_column;
        Ok(())
    }
    
    /// Move cursor to position (clamps to valid bounds)
    pub fn move_cursor_to(&mut self, line: usize, column: usize) {
        let line_count = self.line_count();
        let safe_line = if line_count == 0 { 0 } else { line.min(line_count - 1) };
        let line_length = self.get_line_length(safe_line);
        let safe_column = column.min(line_length);
        
        self.cursor_line = safe_line;
        self.cursor_column = safe_column;
    }
    
    // === ADVANCED CURSOR CONTROL ===
    
    /// Set only the cursor line (keeping current column, with clamping)
    pub fn set_cursor_line_only(&mut self, line: usize) -> Result<(), String> {
        let current_column = self.cursor_column();
        self.set_cursor(line, current_column)
    }
    
    /// Set only the cursor column (keeping current line, with clamping)
    pub fn set_cursor_column_only(&mut self, column: usize) -> Result<(), String> {
        let current_line = self.cursor_line();
        self.set_cursor(current_line, column)
    }
    
    /// Reset cursor column to 0 (start of line)
    pub fn reset_cursor_column(&mut self) {
        let current_line = self.cursor_line();
        let _ = self.set_cursor(current_line, 0);
    }
    
    /// Move cursor to the end of the current line
    pub fn move_cursor_to_current_line_end(&mut self) {
        let current_line = self.cursor_line();
        let line_length = self.get_line_length(current_line);
        let _ = self.set_cursor(current_line, line_length);
    }
    
    
    /// Safe cursor movement - returns true if movement was successful
    pub fn move_cursor_up(&mut self) -> bool {
        if self.cursor_line() > 0 {
            let _ = self.set_cursor_line_only(self.cursor_line() - 1);
            true
        } else {
            false
        }
    }
    
    /// Safe cursor movement - returns true if movement was successful  
    pub fn move_cursor_down(&mut self) -> bool {
        let line_count = self.line_count();
        if self.cursor_line() + 1 < line_count {
            let _ = self.set_cursor_line_only(self.cursor_line() + 1);
            true
        } else {
            false
        }
    }
    
    /// Safe cursor movement - returns true if movement was successful
    pub fn move_cursor_left(&mut self) -> bool {
        if self.cursor_column() > 0 {
            let _ = self.set_cursor_column_only(self.cursor_column() - 1);
            true
        } else {
            false
        }
    }
    
    /// Safe cursor movement - returns true if movement was successful
    pub fn move_cursor_right(&mut self) -> bool {
        let current_line_length = self.get_line_length(self.cursor_line());
        if self.cursor_column() < current_line_length {
            let _ = self.set_cursor_column_only(self.cursor_column() + 1);
            true
        } else {
            false
        }
    }
    
    /// Clamp cursor column to current line bounds (used after line content changes)
    pub fn clamp_cursor_column_to_current_line(&mut self) {
        let current_line = self.cursor_line();
        let current_column = self.cursor_column();
        let line_length = self.get_line_length(current_line);
        
        if current_column > line_length {
            let _ = self.set_cursor(current_line, line_length);
        }
    }

    
    

    // === UNDO MANAGEMENT ===
    
    /// Get mutable reference to undo manager
    pub fn undo_manager_mut(&mut self) -> &mut UndoManager {
        &mut self.undo_manager
    }
    
    /// Get mutable reference to text buffer  
    pub fn text_buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.text_buffer
    }
    

    // Get line count from piece table
    pub fn line_count(&self) -> usize {
        let mut text_buffer = self.text_buffer.clone();
        text_buffer.line_count()
    }

    // Get a specific line from piece table
    pub fn get_line(&self, line_num: usize) -> Option<String> {
        let mut text_buffer = self.text_buffer.clone();
        text_buffer.get_line(line_num)
    }

    // Get line length from piece table
    pub fn get_line_length(&self, line_num: usize) -> usize {
        let mut text_buffer = self.text_buffer.clone();
        text_buffer.line_length(line_num)
    }
    
    // Replace an entire line
    pub fn set_line(&mut self, line_num: usize, new_content: &str) {
        use super::text_buffer::{Position, Range};
        if line_num >= self.line_count() {
            return;
        }
        
        let line_length = self.get_line_length(line_num);
        let start_pos = Position::new(line_num, 0);
        let end_pos = Position::new(line_num, line_length);
        let range = Range::new(start_pos, end_pos);
        
        self.text_buffer.delete(range);
        self.text_buffer.insert(start_pos, new_content);
        self.modified = true;
    }
    
    
    // Check if document is empty
    pub fn is_empty(&self) -> bool {
        self.line_count() == 0 || (self.line_count() == 1 && self.get_line_length(0) == 0)
    }

    // Insert text at position using piece table
    pub fn insert_text_at(&mut self, line: usize, column: usize, text: &str) {
        use super::text_buffer::Position;
        let pos = Position::new(line, column);
        self.text_buffer.insert(pos, text);
        self.modified = true;
    }

    // Delete text at position using piece table
    pub fn delete_text_at(&mut self, line: usize, column: usize, length: usize) -> String {
        use super::text_buffer::{Position, Range};
        let start_pos = Position::new(line, column);
        let end_pos = Position::new(line, column + length);
        let range = Range::new(start_pos, end_pos);
        let deleted_text = self.text_buffer.get_text_range(range.clone());
        self.text_buffer.delete(range);
        self.modified = true;
        deleted_text
    }

    // Insert a new line using piece table
    pub fn insert_line_at(&mut self, line_num: usize, text: &str) {
        use super::text_buffer::Position;
        if line_num == 0 && self.line_count() == 0 {
            // Special case: inserting into empty document
            self.text_buffer.insert(Position::new(0, 0), text);
        } else if line_num >= self.line_count() {
            // Append at end
            let pos = Position::new(self.line_count() - 1, self.get_line_length(self.line_count() - 1));
            self.text_buffer.insert(pos, &format!("\n{}", text));
        } else {
            // Insert at beginning of specified line
            let pos = Position::new(line_num, 0);
            self.text_buffer.insert(pos, &format!("{}\n", text));
        }
        self.modified = true;
        
        // Update marks: new line inserted at line_num
        self.update_marks_line_inserted(line_num);
    }

    // Delete a line using piece table
    pub fn delete_line_at(&mut self, line_num: usize) -> String {
        use super::text_buffer::{Position, Range};
        if line_num >= self.line_count() {
            return String::new();
        }

        let line_content = self.get_line(line_num).unwrap_or_default();
        let line_length = self.get_line_length(line_num);
        
        // If this is the last line and there are other lines, include the newline from previous line
        let (start_pos, end_pos) = if line_num == self.line_count() - 1 && self.line_count() > 1 {
            // Delete the newline from the previous line and this line
            let prev_line_length = self.get_line_length(line_num - 1);
            (Position::new(line_num - 1, prev_line_length), Position::new(line_num, line_length))
        } else {
            // Delete this line and its newline
            (Position::new(line_num, 0), Position::new(line_num + 1, 0))
        };

        let range = Range::new(start_pos, end_pos);
        self.text_buffer.delete(range);
        self.modified = true;
        
        // Update marks: line deleted at line_num
        self.update_marks_line_deleted(line_num);
        
        line_content
    }

    // Split a line at given position using piece table
    pub fn split_line_at(&mut self, line_num: usize, column: usize, insert_text: &str) {
        use super::text_buffer::Position;
        if line_num >= self.line_count() {
            return;
        }

        let line_length = self.get_line_length(line_num);
        if column <= line_length {
            let pos = Position::new(line_num, column);
            self.text_buffer.insert(pos, &format!("\n{}", insert_text));
            self.modified = true;
            
            // Update marks: new line created at line_num + 1
            self.update_marks_line_inserted(line_num + 1);
        }
    }

    // Join two lines with separator using piece table
    pub fn join_lines_at(&mut self, line_num: usize, separator: &str) {
        use super::text_buffer::{Position, Range};
        if line_num >= self.line_count() - 1 {
            return;
        }

        // Replace the newline between the lines with the separator
        let first_line_length = self.get_line_length(line_num);
        let start_pos = Position::new(line_num, first_line_length);
        let end_pos = Position::new(line_num + 1, 0);
        let range = Range::new(start_pos, end_pos);
        
        self.text_buffer.delete(range.clone());
        self.text_buffer.insert(start_pos, separator);
        self.modified = true;
        
        // Update marks: line line_num + 1 was joined (removed)
        self.update_marks_line_deleted(line_num + 1);
    }




    #[cfg(test)]
    pub fn get_piece_table_content(&mut self) -> String {
        self.text_buffer.get_text()
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
        let content = self.text_buffer.get_text();
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
        self.insert_text_at_line(text, self.cursor_line() + 1)
    }


    pub fn insert_text_at_line(
        &mut self,
        text: &str,
        line_num: usize,
    ) -> Result<usize, std::io::Error> {
        if text.is_empty() {
            return Ok(0);
        }

        let byte_count = text.len();

        // Insert after the specified line (0-based internally, but line_num is 1-based from user)
        use super::text_buffer::Position;
        let insert_pos = if line_num == 0 {
            // Insert at beginning of document
            Position::new(0, 0)
        } else {
            let target_line = (line_num - 1).min(self.line_count().saturating_sub(1));
            let line_length = self.get_line_length(target_line);
            Position::new(target_line, line_length)
        };

        // Add newline prefix if we're inserting after content
        let text_to_insert = if line_num == 0 && self.line_count() > 0 {
            format!("{}\n", text)
        } else if line_num > 0 {
            format!("\n{}", text)
        } else {
            text.to_string()
        };

        self.text_buffer.insert(insert_pos, &text_to_insert);
        self.modified = true;
        Ok(byte_count)
    }


    pub fn insert_char(&mut self, c: char) {
        self.record_insert_undo(self.cursor_line(), self.cursor_column(), &c.to_string());
        
        let pos = Position::new(self.cursor_line(), self.cursor_column());
        self.text_buffer.insert(pos, &c.to_string());
        
        self.cursor_column += 1;
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        let current_line = self.get_line(self.cursor_line()).unwrap_or_default();
        let new_line = if self.cursor_column() < current_line.len() {
            current_line[self.cursor_column()..].to_string()
        } else {
            String::new()
        };

        // Record undo action for splitting the line
        self.undo_manager
            .add_action(super::undo::UndoAction::SplitLine {
                line: self.cursor_line(),
                column: self.cursor_column(),
                text: new_line.clone(),
            });

        // Use piece table for newline insertion
        use super::text_buffer::Position;
        let pos = Position::new(self.cursor_line(), self.cursor_column());
        self.text_buffer.insert_newline(pos);

        self.cursor_line += 1;
        self.reset_cursor_column();
        self.modified = true;
        
        // Update marks: new line created at cursor_line (after increment)
        self.update_marks_line_inserted(self.cursor_line());
    }

    pub fn delete_char(&mut self) {
        if self.cursor_column() > 0 {
            let pos = Position::new(self.cursor_line(), self.cursor_column() - 1);
            let deleted_char = self.text_buffer.char_at(pos).unwrap_or(' ');
            
            self.record_delete_undo(self.cursor_line(), self.cursor_column() - 1, &deleted_char.to_string());
            self.text_buffer.delete_char(pos);
            self.cursor_column -= 1;
            self.modified = true;
        } else if self.cursor_line() > 0 {
            let current_line = self.get_line(self.cursor_line()).unwrap_or_default();
            let previous_line_len = self.get_line_length(self.cursor_line() - 1);

            self.undo_manager.add_action(super::undo::UndoAction::JoinLines {
                line: self.cursor_line() - 1,
                separator: String::new(),
                second_line_text: current_line.clone(),
            });

            let pos = Position::new(self.cursor_line() - 1, previous_line_len);
            self.text_buffer.delete_char(pos);
            
            self.cursor_line -= 1;
            self.cursor_column = previous_line_len;
            self.modified = true;
        }
    }

    pub fn delete_char_forward(&mut self) {
        let pos = Position::new(self.cursor_line(), self.cursor_column());
        let line_length = self.get_line_length(self.cursor_line());
        
        if self.cursor_column() < line_length {
            if let Some(deleted_char) = self.text_buffer.char_at(pos) {
                self.record_delete_undo(self.cursor_line(), self.cursor_column(), &deleted_char.to_string());
                self.text_buffer.delete_char(pos);
                self.modified = true;
            }
        } else if self.cursor_line() < self.line_count() - 1 {
            let next_line = self.get_line(self.cursor_line() + 1).unwrap_or_default();

            self.undo_manager.add_action(super::undo::UndoAction::JoinLines {
                line: self.cursor_line(),
                separator: String::new(),
                second_line_text: next_line.clone(),
            });

            self.text_buffer.delete_char(pos);
            self.modified = true;
        }
    }

    pub fn delete_line(&mut self) {
        if self.line_count() > 1 {
            self.delete_line_at(self.cursor_line());
            if self.cursor_line() >= self.line_count() {
                let _ = self.set_cursor(self.line_count().saturating_sub(1), 0);
            } else {
                let _ = self.set_cursor(self.cursor_line(), 0);
            }
            self.modified = true;
        } else {
            // If only one line, clear it
            self.set_line(0, "");
            let _ = self.set_cursor(0, 0);
            self.modified = true;
        }
    }

    pub fn delete_to_end_of_line(&mut self) {
        let line_length = self.get_line_length(self.cursor_line());
        if self.cursor_column() < line_length {
            // Delete from cursor to end of line
            use super::text_buffer::{Position, Range};
            let start_pos = Position::new(self.cursor_line(), self.cursor_column());
            let end_pos = Position::new(self.cursor_line(), line_length);
            let range = Range::new(start_pos, end_pos);
            self.text_buffer.delete(range);
            self.modified = true;
            
            // Adjust cursor if needed
            let new_line_length = self.get_line_length(self.cursor_line());
            if self.cursor_column() > 0 && self.cursor_column() >= new_line_length {
                self.cursor_column = new_line_length.saturating_sub(1);
            }
        }
    }

    /// Generic delete operation that moves cursor and deletes text
    fn delete_with_movement<F>(&mut self, move_fn: F)
    where
        F: FnOnce(&mut Self),
    {
        let original_line = self.cursor_line();
        let original_column = self.cursor_column();

        move_fn(self);

        let start_pos = Position::new(original_line, original_column);
        let end_pos = Position::new(self.cursor_line(), self.cursor_column());
        let range = Range::new(start_pos, end_pos);
        
        self.text_buffer.delete(range);
        
        self.cursor_line = original_line;
        self.cursor_column = original_column;
        self.modified = true;
    }

    pub fn delete_word_forward(&mut self) {
        self.delete_with_movement(Self::move_word_forward);
    }

    pub fn delete_big_word_forward(&mut self) {
        self.delete_with_movement(Self::move_big_word_forward);
    }

    pub fn delete_char_backward(&mut self) {
        if self.cursor_column() > 0 {
            // Delete character before cursor using piece table
            use super::text_buffer::Position;
            let pos = Position::new(self.cursor_line(), self.cursor_column() - 1);
            let deleted_char = self.text_buffer.char_at(pos).unwrap_or(' ');
            
            self.undo_manager
                .add_action(super::undo::UndoAction::DeleteText {
                    line: self.cursor_line(),
                    column: self.cursor_column() - 1,
                    text: deleted_char.to_string(),
                });
            
            self.text_buffer.delete_char(pos);
            self.cursor_column -= 1;
            self.modified = true;
        } else if self.cursor_line() > 0 {
            // Join with previous line
            let current_line = self.delete_line_at(self.cursor_line());
            let previous_line_len = self.get_line_length(self.cursor_line() - 1);

            // Record undo action for joining lines
            self.undo_manager
                .add_action(super::undo::UndoAction::JoinLines {
                    line: self.cursor_line() - 1,
                    separator: String::new(),
                    second_line_text: current_line.clone(),
                });

            self.cursor_line -= 1;
            self.cursor_column = previous_line_len;
            // Append the current line content to the previous line using piece table
            use super::text_buffer::Position;
            let pos = Position::new(self.cursor_line(), previous_line_len);
            self.text_buffer.insert(pos, &current_line);
            self.modified = true;
        }
    }

    /// Generic delete operation that moves cursor backward and deletes text
    fn delete_with_backward_movement<F>(&mut self, move_fn: F)
    where
        F: FnOnce(&mut Self),
    {
        let original_line = self.cursor_line();
        let original_column = self.cursor_column();

        move_fn(self);

        let start_pos = Position::new(self.cursor_line(), self.cursor_column());
        let end_pos = Position::new(original_line, original_column);
        let range = Range::new(start_pos, end_pos);
        self.text_buffer.delete(range);
        self.modified = true;
    }

    pub fn delete_word_backward(&mut self) {
        self.delete_with_backward_movement(Self::move_word_backward);
    }

    pub fn delete_big_word_backward(&mut self) {
        self.delete_with_backward_movement(Self::move_big_word_backward);
    }

    pub fn delete_to_end_of_word(&mut self) {
        self.delete_with_movement(Self::move_word_end);
    }

    pub fn delete_to_end_of_big_word(&mut self) {
        self.delete_with_movement(Self::move_big_word_end);
    }

    /// Generic range delete operation
    fn delete_range(&mut self, start_line: usize, start_col: usize, end_line: usize, end_col: usize) {
        if start_line != end_line || start_col != end_col {
            let start_pos = Position::new(start_line, start_col);
            let end_pos = Position::new(end_line, end_col);
            let range = Range::new(start_pos, end_pos);
            self.text_buffer.delete(range);
            self.modified = true;
        }
    }

    pub fn delete_to_start_of_line(&mut self) {
        if self.cursor_column() > 0 {
            self.delete_range(self.cursor_line(), 0, self.cursor_line(), self.cursor_column());
            self.reset_cursor_column();
        }
    }

    pub fn delete_to_first_non_whitespace(&mut self) {
        let line = self.get_line(self.cursor_line()).unwrap_or_default();
        let first_non_ws = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);

        if self.cursor_column() > first_non_ws {
            self.delete_range(self.cursor_line(), first_non_ws, self.cursor_line(), self.cursor_column());
            self.cursor_column = first_non_ws;
        }
    }

    pub fn delete_to_end_of_file(&mut self) {
        let last_line = self.line_count().saturating_sub(1);
        let last_column = self.get_line_length(last_line);
        self.delete_range(self.cursor_line(), self.cursor_column(), last_line, last_column);
    }

    pub fn delete_to_start_of_file(&mut self) {
        if self.cursor_line() > 0 || self.cursor_column() > 0 {
            self.delete_range(0, 0, self.cursor_line(), self.cursor_column());
            self.cursor_line = 0;
            self.reset_cursor_column();
        }
    }

    pub fn substitute_char(&mut self) {
        // Delete current character and enter insert mode
        self.delete_char_forward();
    }

    pub fn substitute_line(&mut self) {
        // Clear current line and move cursor to beginning
        self.set_line(self.cursor_line(), "");
        self.reset_cursor_column();
        self.modified = true;
    }

    /// Generic character-based delete operation
    fn delete_until_char_generic(&mut self, target: char, forward: bool, include_char: bool) {
        let line = self.get_line(self.cursor_line()).unwrap_or_default();
        let cursor_col = self.cursor_column();
        
        let target_pos = if forward {
            line[cursor_col + 1..].find(target).map(|pos| cursor_col + 1 + pos)
        } else {
            line[..cursor_col].rfind(target)
        };

        if let Some(pos) = target_pos {
            let (start_col, end_col) = if forward {
                let end = if include_char { pos + 1 } else { pos };
                (cursor_col, end)
            } else {
                let start = if include_char { pos } else { pos + 1 };
                (start, cursor_col)
            };
            
            self.delete_range(self.cursor_line(), start_col, self.cursor_line(), end_col);
            
            if !forward {
                self.cursor_column = if include_char { pos } else { pos + 1 };
            }
        }
    }

    pub fn delete_until_char(&mut self, target: char) {
        self.delete_until_char_generic(target, true, false);
    }

    pub fn delete_until_char_backward(&mut self, target: char) {
        self.delete_until_char_generic(target, false, false);
    }

    pub fn delete_find_char(&mut self, target: char) {
        self.delete_until_char_generic(target, true, true);
    }

    pub fn delete_find_char_backward(&mut self, target: char) {
        self.delete_until_char_generic(target, false, true);
    }

    pub fn open_line_below(&mut self) {
        self.cursor_line += 1;
        self.reset_cursor_column();
        self.insert_line_at(self.cursor_line(), "");
        self.modified = true;
    }

    pub fn open_line_above(&mut self) {
        self.insert_line_at(self.cursor_line(), "");
        self.reset_cursor_column();
        self.modified = true;
    }


    pub fn set_expand_tab(&mut self, expand: bool) {
        self.expand_tab = expand;
    }

    pub fn tabs_to_spaces(&mut self, tab_width: usize) -> usize {
        let mut changed_lines = 0;
        let spaces = " ".repeat(tab_width);

        for line_idx in 0..self.line_count() {
            if let Some(line) = self.get_line(line_idx) {
                if line.contains('\t') {
                    let new_line = line.replace('\t', &spaces);
                    self.set_line(line_idx, &new_line);
                    changed_lines += 1;
                }
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

        for line_idx in 0..self.line_count() {
            if let Some(line) = self.get_line(line_idx) {
                if line.contains(&spaces) {
                    let new_line = line.replace(&spaces, "\t");
                    self.set_line(line_idx, &new_line);
                    changed_lines += 1;
                }
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

        for line_idx in 0..self.line_count() {
            if let Some(line) = self.get_line(line_idx) {
                let normalized = Self::normalize_to_ascii(&line);
                if normalized != line {
                    self.set_line(line_idx, &normalized);
                    changed_lines += 1;
                }
            }
        }

        if changed_lines > 0 {
            self.modified = true;
        }

        changed_lines
    }

    /// Normalize Unicode characters to ASCII equivalents (condensed version)
    fn normalize_to_ascii(text: &str) -> String {
        text.chars()
            .filter_map(|ch| match ch {
                // Common Unicode spaces → ASCII space
                '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => Some(' '),
                // Common dashes → ASCII hyphen
                '\u{2010}'..='\u{2015}' | '\u{2212}' | '\u{FE58}' | '\u{FE63}' | '\u{FF0D}' => Some('-'),
                // Common quotes → ASCII quotes
                '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '\u{2032}' | '\u{2035}' => Some('\''),
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '\u{2033}' | '\u{2036}' | '\u{301D}' | '\u{301E}' => Some('"'),
                // ASCII characters unchanged
                _ if ch.is_ascii() => Some(ch),
                // Basic accented letter fallback
                'À'..='Å' | 'Ā' | 'Ă' | 'Ą' => Some('A'),
                'à'..='å' | 'ā' | 'ă' | 'ą' => Some('a'),
                'È'..='Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => Some('E'),
                'è'..='ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => Some('e'),
                'Ì'..='Ï' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => Some('I'),
                'ì'..='ï' | 'ī' | 'ĭ' | 'į' | 'ı' => Some('i'),
                'Ò'..='Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' => Some('O'),
                'ò'..='ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => Some('o'),
                'Ù'..='Ü' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => Some('U'),
                'ù'..='ü' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => Some('u'),
                'Ñ' => Some('N'), 'ñ' => Some('n'), 'Ç' => Some('C'), 'ç' => Some('c'),
                // Drop other non-ASCII characters
                _ => None,
            })
            .collect()
    }

    pub fn insert_tab_or_spaces(&mut self, tab_width: usize) {
        if self.expand_tab {
            // Insert spaces
            let spaces = " ".repeat(tab_width);
            self.undo_manager
                .add_action(super::undo::UndoAction::InsertText {
                    line: self.cursor_line(),
                    column: self.cursor_column(),
                    text: spaces.clone(),
                });
            use super::text_buffer::Position;
            let pos = Position::new(self.cursor_line(), self.cursor_column());
            self.text_buffer.insert(pos, &spaces);
            self.cursor_column += tab_width;
        } else {
            // Insert actual tab
            self.undo_manager
                .add_action(super::undo::UndoAction::InsertText {
                    line: self.cursor_line(),
                    column: self.cursor_column(),
                    text: "\t".to_string(),
                });
            use super::text_buffer::Position;
            let pos = Position::new(self.cursor_line(), self.cursor_column());
            self.text_buffer.insert(pos, "\t");
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

        use super::text_buffer::Position;
        let pos = Position::new(self.cursor_line(), 0);
        self.text_buffer.insert(pos, &indent);
        
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
        let line_count = self.line_count();
        let end_line = std::cmp::min(start_line + count, line_count);
        let indent = if use_spaces {
            " ".repeat(tab_width)
        } else {
            "\t".to_string()
        };

        for line_idx in start_line..end_line {
            use super::text_buffer::Position;
            let pos = Position::new(line_idx, 0);
            self.text_buffer.insert(pos, &indent);
        }

        self.modified = true;
    }

    /// Helper to calculate dedent amount
    fn calculate_dedent_amount(line: &str, tab_width: usize) -> usize {
        if line.starts_with('\t') {
            1
        } else {
            line.chars().take(tab_width).take_while(|&ch| ch == ' ').count()
        }
    }

    pub fn dedent_line(&mut self, tab_width: usize) {
        if let Some(line) = self.get_line(self.cursor_line()) {
            let chars_to_remove = Self::calculate_dedent_amount(&line, tab_width);
            
            if chars_to_remove > 0 {
                let start_pos = Position::new(self.cursor_line(), 0);
                let end_pos = Position::new(self.cursor_line(), chars_to_remove);
                let range = Range::new(start_pos, end_pos);
                self.text_buffer.delete(range);
                self.cursor_column = self.cursor_column().saturating_sub(chars_to_remove);
                self.modified = true;
            }
        }
    }

    pub fn dedent_lines(&mut self, start_line: usize, count: usize, tab_width: usize) {
        let line_count = self.line_count();
        let end_line = std::cmp::min(start_line + count, line_count);
        let mut any_modified = false;

        for line_idx in start_line..end_line {
            if let Some(line) = self.get_line(line_idx) {
                let chars_to_remove = Self::calculate_dedent_amount(&line, tab_width);
                
                if chars_to_remove > 0 {
                    let start_pos = Position::new(line_idx, 0);
                    let end_pos = Position::new(line_idx, chars_to_remove);
                    let range = Range::new(start_pos, end_pos);
                    self.text_buffer.delete(range);
                    any_modified = true;
                }
            }
        }

        if any_modified {
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

    /// Update marks when a line is inserted (simple vim-like approach)
    fn update_marks_line_inserted(&mut self, inserted_line: usize) {
        for (_, (line, _column)) in self.local_marks.iter_mut() {
            if *line >= inserted_line {
                *line += 1;
            }
        }
    }

    /// Update marks when a line is deleted (simple vim-like approach)
    fn update_marks_line_deleted(&mut self, deleted_line: usize) {
        // Remove marks on the deleted line and move marks below up by 1
        self.local_marks.retain(|_, (line, _)| *line != deleted_line);
        for (_, (line, _column)) in self.local_marks.iter_mut() {
            if *line > deleted_line {
                *line -= 1;
            }
        }
    }

    /// Get all local marks for this buffer (for :marks command)
    pub fn get_all_local_marks(&self) -> &HashMap<char, (usize, usize)> {
        &self.local_marks
    }

    // Yank (copy) operations - return text to be copied to registers

    pub fn yank_line(&self) -> String {
        if !self.is_empty() && self.cursor_line() < self.line_count() {
            self.get_line(self.cursor_line()).unwrap_or_default()
        } else {
            String::new()
        }
    }

    pub fn yank_to_end_of_line(&self) -> String {
        if let Some(line) = self.get_line(self.cursor_line()) {
            if self.cursor_column() <= line.len() {
                line[self.cursor_column()..].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    pub fn yank_word_forward(&self) -> String {
        let start_line = self.cursor_line();
        let start_col = self.cursor_column();

        let (end_line, end_col) = self.calculate_word_forward_position();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_big_word_forward(&self) -> String {
        let start_line = self.cursor_line();
        let start_col = self.cursor_column();

        let (end_line, end_col) = self.calculate_big_word_forward_position();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_word_backward(&self) -> String {
        let end_line = self.cursor_line();
        let end_col = self.cursor_column();

        let (start_line, start_col) = self.calculate_word_backward_position();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_big_word_backward(&self) -> String {
        let end_line = self.cursor_line();
        let end_col = self.cursor_column();

        let (start_line, start_col) = self.calculate_big_word_backward_position();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_to_end_of_word(&self) -> String {
        let start_line = self.cursor_line();
        let start_col = self.cursor_column();

        let (end_line, end_col_raw) = self.calculate_word_end_position();
        let end_col = end_col_raw + 1; // Include the character at cursor

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_to_end_of_big_word(&self) -> String {
        let start_line = self.cursor_line();
        let start_col = self.cursor_column();

        let (end_line, end_col_raw) = self.calculate_big_word_end_position();
        let end_col = end_col_raw + 1; // Include the character at cursor

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_to_start_of_line(&self) -> String {
        if let Some(line) = self.get_line(self.cursor_line()) {
            if self.cursor_column() <= line.len() {
                line[..self.cursor_column()].to_string()
            } else {
                line.to_string()
            }
        } else {
            String::new()
        }
    }

    pub fn yank_to_first_non_whitespace(&self) -> String {
        if let Some(line) = self.get_line(self.cursor_line()) {
            let first_non_ws = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
            if self.cursor_column() >= first_non_ws {
                line[first_non_ws..self.cursor_column()].to_string()
            } else {
                line[self.cursor_column()..first_non_ws].to_string()
            }
        } else {
            String::new()
        }
    }

    pub fn yank_to_end_of_file(&self) -> String {
        let start_line = self.cursor_line();
        let start_col = self.cursor_column();
        let end_line = self.line_count().saturating_sub(1);
        let end_col = if !self.is_empty() {
            self.get_line_length(end_line)
        } else {
            0
        };

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_to_start_of_file(&self) -> String {
        let start_line = 0;
        let start_col = 0;
        let end_line = self.cursor_line();
        let end_col = self.cursor_column();

        self.get_text_range(start_line, start_col, end_line, end_col)
    }

    pub fn yank_until_char(&self, target: char) -> String {
        let start_line = self.cursor_line();
        let start_col = self.cursor_column();

        if let Some((end_line, end_col)) = self.find_char_position(target, true, true) {
            self.get_text_range(start_line, start_col, end_line, end_col)
        } else {
            String::new()
        }
    }

    pub fn yank_until_char_backward(&self, target: char) -> String {
        let end_line = self.cursor_line();
        let end_col = self.cursor_column();

        if let Some((start_line, start_col)) = self.find_char_position(target, false, true) {
            self.get_text_range(start_line, start_col + 1, end_line, end_col)
        } else {
            String::new()
        }
    }

    pub fn yank_find_char(&self, target: char) -> String {
        let start_line = self.cursor_line();
        let start_col = self.cursor_column();

        if let Some((end_line, end_col)) = self.find_char_position(target, true, false) {
            self.get_text_range(start_line, start_col, end_line, end_col + 1)
        } else {
            String::new()
        }
    }

    pub fn yank_find_char_backward(&self, target: char) -> String {
        let end_line = self.cursor_line();
        let end_col = self.cursor_column();

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
        let deleted = self.get_line(self.cursor_line()).unwrap_or_default();
        self.set_line(self.cursor_line(), "");
        self.reset_cursor_column();
        self.modified = true;
        deleted
    }

    /// Generic change operation: yank then delete
    fn change_with_operation<F, G>(&mut self, yank_fn: F, delete_fn: G) -> String
    where
        F: Fn(&Self) -> String,
        G: FnOnce(&mut Self),
    {
        let deleted = yank_fn(self);
        delete_fn(self);
        deleted
    }

    pub fn change_to_end_of_line(&mut self) -> String {
        self.change_with_operation(Self::yank_to_end_of_line, Self::delete_to_end_of_line)
    }

    pub fn change_word_forward(&mut self) -> String {
        self.change_with_operation(Self::yank_word_forward, Self::delete_word_forward)
    }

    pub fn change_big_word_forward(&mut self) -> String {
        self.change_with_operation(Self::yank_big_word_forward, Self::delete_big_word_forward)
    }

    pub fn change_word_backward(&mut self) -> String {
        self.change_with_operation(Self::yank_word_backward, Self::delete_word_backward)
    }

    pub fn change_big_word_backward(&mut self) -> String {
        self.change_with_operation(Self::yank_big_word_backward, Self::delete_big_word_backward)
    }

    pub fn change_to_end_of_word(&mut self) -> String {
        self.change_with_operation(Self::yank_to_end_of_word, Self::delete_to_end_of_word)
    }

    pub fn change_to_end_of_big_word(&mut self) -> String {
        self.change_with_operation(Self::yank_to_end_of_big_word, Self::delete_to_end_of_big_word)
    }

    pub fn change_to_start_of_line(&mut self) -> String {
        self.change_with_operation(Self::yank_to_start_of_line, Self::delete_to_start_of_line)
    }

    pub fn change_to_first_non_whitespace(&mut self) -> String {
        self.change_with_operation(Self::yank_to_first_non_whitespace, Self::delete_to_first_non_whitespace)
    }

    pub fn change_to_end_of_file(&mut self) -> String {
        self.change_with_operation(Self::yank_to_end_of_file, Self::delete_to_end_of_file)
    }

    pub fn change_to_start_of_file(&mut self) -> String {
        self.change_with_operation(Self::yank_to_start_of_file, Self::delete_to_start_of_file)
    }

    pub fn change_until_char(&mut self, target: char) -> String {
        self.change_with_operation(|doc| doc.yank_until_char(target), |doc| doc.delete_until_char(target))
    }

    pub fn change_until_char_backward(&mut self, target: char) -> String {
        self.change_with_operation(|doc| doc.yank_until_char_backward(target), |doc| doc.delete_until_char_backward(target))
    }

    pub fn change_find_char(&mut self, target: char) -> String {
        self.change_with_operation(|doc| doc.yank_find_char(target), |doc| doc.delete_find_char(target))
    }

    pub fn change_find_char_backward(&mut self, target: char) -> String {
        self.change_with_operation(|doc| doc.yank_find_char_backward(target), |doc| doc.delete_find_char_backward(target))
    }

    // Helper method to get character at cursor
    #[allow(dead_code)]
    fn get_char_at_cursor(&self) -> String {
        if self.cursor_line() < self.line_count() {
            let line = self.get_line(self.cursor_line()).unwrap_or_default();
            if self.cursor_column() < self.get_line_length(self.cursor_line()) {
                let chars: Vec<char> = line.chars().collect();
                chars[self.cursor_column()].to_string()
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
        let line_count = self.line_count();
        
        // Check if we can join (not at the last line)
        if self.cursor_line() >= line_count - 1 {
            return false;
        }

        let current_line = self.cursor_line();
        let next_line = current_line + 1;

        // Get the lines to join
        let current_line_text = self.get_line(current_line).unwrap_or_default();
        let next_line_text = self.get_line(next_line).unwrap_or_default();

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

        let mut joined_line = current_line_text;
        if needs_space {
            joined_line.push(' ');
        }

        // Trim leading whitespace from the next line
        let trimmed_next = next_line_text.trim_start();
        joined_line.push_str(trimmed_next);

        // Record undo information
        self.undo_manager
            .add_action(super::undo::UndoAction::JoinLines {
                line: current_line,
                separator: if needs_space {
                    " ".to_string()
                } else {
                    String::new()
                },
                second_line_text: next_line_text,
            });

        // Update the document using piece table operations
        self.set_line(current_line, &joined_line);
        self.delete_line_at(next_line);

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
        if self.cursor_line() >= self.line_count() {
            return false;
        }

        let line = self.get_line(self.cursor_line()).unwrap_or_default();
        
        if self.cursor_column() >= line.len() {
            return false;
        }

        let chars: Vec<char> = line.chars().collect();
        if self.cursor_column() >= chars.len() {
            return false;
        }

        let original_char = chars[self.cursor_column()];
        let new_char = if original_char.is_uppercase() {
            original_char.to_lowercase().collect::<String>()
        } else if original_char.is_lowercase() {
            original_char.to_uppercase().collect::<String>()
        } else {
            return false; // No case to toggle
        };

        // Record undo action
        self.undo_manager
            .add_action(super::undo::UndoAction::DeleteText {
                line: self.cursor_line(),
                column: self.cursor_column(),
                text: original_char.to_string(),
            });
        self.undo_manager
            .add_action(super::undo::UndoAction::InsertText {
                line: self.cursor_line(),
                column: self.cursor_column(),
                text: new_char.clone(),
            });

        // Replace character using piece table
        use super::text_buffer::{Position, Range};
        let char_start = Position::new(self.cursor_line(), self.cursor_column());
        let char_end = Position::new(self.cursor_line(), self.cursor_column() + 1);
        let range = Range::new(char_start, char_end);
        self.text_buffer.replace(range, &new_char);

        // Move cursor forward (vim behavior)
        let line_length = self.get_line_length(self.cursor_line());
        if self.cursor_column() < line_length.saturating_sub(1) {
            self.cursor_column += 1;
        }

        self.modified = true;
        true
    }

    /// Convert current line to lowercase
    pub fn lowercase_line(&mut self) {
        self.transform_line(|line| line.to_lowercase());
    }

    /// Convert current line to uppercase
    pub fn uppercase_line(&mut self) {
        self.transform_line(|line| line.to_uppercase());
    }

    /// Helper to transform current line with a function
    fn transform_line<F>(&mut self, transform: F)
    where
        F: Fn(&str) -> String,
    {
        if self.cursor_line() >= self.line_count() {
            return;
        }

        if let Some(original_line) = self.get_line(self.cursor_line()) {
            let transformed_line = transform(&original_line);

            if original_line != transformed_line {
                self.record_line_replace_undo(&original_line, &transformed_line);
                self.set_line(self.cursor_line(), &transformed_line);
                self.modified = true;
                self.clamp_cursor_column_to_current_line();
            }
        }
    }

    /// Helper to record undo actions for line replacement
    fn record_line_replace_undo(&mut self, original: &str, new: &str) {
        self.undo_manager.add_action(super::undo::UndoAction::DeleteText {
            line: self.cursor_line(),
            column: 0,
            text: original.to_string(),
        });
        self.undo_manager.add_action(super::undo::UndoAction::InsertText {
            line: self.cursor_line(),
            column: 0,
            text: new.to_string(),
        });
    }

    /// Helper to record undo actions for text insertion
    fn record_insert_undo(&mut self, line: usize, column: usize, text: &str) {
        self.undo_manager.add_action(super::undo::UndoAction::InsertText {
            line,
            column,
            text: text.to_string(),
        });
    }

    /// Helper to record undo actions for text deletion
    fn record_delete_undo(&mut self, line: usize, column: usize, text: &str) {
        self.undo_manager.add_action(super::undo::UndoAction::DeleteText {
            line,
            column,
            text: text.to_string(),
        });
    }

    // Helper method to get text in a range
    fn get_text_range(
        &self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> String {
        if start_line >= self.line_count() || end_line >= self.line_count() {
            return String::new();
        }

        use super::text_buffer::{Position, Range};
        let start_pos = Position::new(start_line, start_col);
        let end_pos = Position::new(end_line, end_col);
        let range = Range::new(start_pos, end_pos);
        
        let mut text_buffer = self.text_buffer.clone();
        text_buffer.get_text_range(range)
    }

    // Position calculation functions for yank operations - eliminates document cloning
    fn calculate_word_forward_position(&self) -> (usize, usize) {
        let mut cursor_line = self.cursor_line();
        let mut cursor_column = self.cursor_column();

        let line = self.get_line(cursor_line).unwrap_or_default();

        // If at end of line, move to next line
        if cursor_column >= self.get_line_length(self.cursor_line()) {
            if cursor_line < self.line_count() - 1 {
                cursor_line += 1;
                cursor_column = 0;
                // Find first non-whitespace on new line
                let new_line = self.get_line(cursor_line).unwrap_or_default();
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
            if cursor_line < self.line_count() - 1 {
                cursor_line += 1;
                cursor_column = 0;
                // Find first non-whitespace on new line
                let new_line = self.get_line(cursor_line).unwrap_or_default();
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
        let mut cursor_line = self.cursor_line();
        let mut cursor_column = self.cursor_column();

        // If at beginning of line, move to end of previous line
        if cursor_column == 0 {
            if cursor_line > 0 {
                cursor_line -= 1;
                let line = self.get_line(cursor_line).unwrap_or_default();
                cursor_column = self.get_line_length(self.cursor_line());
                // Find last non-whitespace character
                let chars: Vec<char> = line.chars().collect();
                while cursor_column > 0 && chars[cursor_column - 1].is_whitespace() {
                    cursor_column -= 1;
                }
            }
            return (cursor_line, cursor_column);
        }

        let line = self.get_line(cursor_line).unwrap_or_default();
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
        let cursor_line = self.cursor_line();
        let mut cursor_column = self.cursor_column();

        let line = self.get_line(cursor_line).unwrap_or_default();
        if cursor_column < self.get_line_length(self.cursor_line()) {
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
        let mut cursor_line = self.cursor_line();
        let mut cursor_column = self.cursor_column();

        loop {
            let line = self.get_line(cursor_line).unwrap_or_default();
            if cursor_column >= self.get_line_length(self.cursor_line()) {
                if cursor_line < self.line_count() - 1 {
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
                    if cursor_line < self.line_count() - 1 {
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
        let mut cursor_line = self.cursor_line();
        let mut cursor_column = self.cursor_column();

        loop {
            if cursor_column == 0 {
                if cursor_line > 0 {
                    cursor_line -= 1;
                    cursor_column = self.get_line_length(cursor_line);
                } else {
                    break;
                }
            } else {
                let line = self.get_line(cursor_line).unwrap_or_default();
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
        let cursor_line = self.cursor_line();
        let mut cursor_column = self.cursor_column();

        let line = self.get_line(cursor_line).unwrap_or_default();
        if cursor_column < self.get_line_length(self.cursor_line()) {
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
        let line = self.get_line(self.cursor_line()).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        let mut cursor_column = self.cursor_column();

        if forward {
            let start = if before {
                cursor_column
            } else {
                cursor_column + 1
            };
            for (i, ch) in chars.iter().enumerate().skip(start) {
                if *ch == target {
                    cursor_column = if before && i > 0 { i - 1 } else { i };
                    return Some((self.cursor_line(), cursor_column));
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
                    return Some((self.cursor_line(), cursor_column));
                }
            }
        }

        None
    }

    pub fn get_word_under_cursor(&self) -> Option<String> {
        if self.cursor_line() >= self.line_count() {
            return None;
        }

        let line = self.get_line(self.cursor_line()).unwrap_or_default();
        if self.cursor_column() >= self.get_line_length(self.cursor_line()) {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
            return None;
        }

        let cursor_pos = self.cursor_column().min(chars.len().saturating_sub(1));
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
        let line_count = self.line_count();
        
        if self.cursor_line() >= line_count {
            return None;
        }

        let line = self.get_line(self.cursor_line()).unwrap_or_default();
        
        if self.cursor_column() >= self.get_line_length(self.cursor_line()) {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() || self.cursor_column() >= chars.len() {
            return None;
        }

        let cursor_char = chars[self.cursor_column()];

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
            self.find_closing_bracket(opening, closing, self.cursor_line(), self.cursor_column())
        } else {
            // Search backward for opening bracket
            self.find_opening_bracket(opening, closing, self.cursor_line(), self.cursor_column())
        }
    }

    /// Check if the bracket at the cursor position is unmatched
    pub fn is_unmatched_bracket(&self) -> Option<(usize, usize)> {
        let line_count = self.line_count();
        
        if self.cursor_line() >= line_count {
            return None;
        }

        let line = self.get_line(self.cursor_line()).unwrap_or_default();
        if self.cursor_column() >= self.get_line_length(self.cursor_line()) {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() || self.cursor_column() >= chars.len() {
            return None;
        }

        let cursor_char = chars[self.cursor_column()];

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
            self.find_closing_bracket(opening, closing, self.cursor_line(), self.cursor_column())
                .is_some()
        } else {
            self.find_opening_bracket(opening, closing, self.cursor_line(), self.cursor_column())
                .is_some()
        };

        // If no match found, this bracket is unmatched
        if !has_match {
            Some((self.cursor_line(), self.cursor_column()))
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
            let mut text_buffer = self.text_buffer.clone();
            let lines = text_buffer.get_lines();
            
            for (line_idx, line) in lines.iter().enumerate() {
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

        let line_count = self.line_count();
        
        while line_idx < line_count {
            let line = self.get_line(line_idx).unwrap_or_default();
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
                let line = self.get_line(search_line).unwrap_or_default();
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
            let line = self.get_line(line_idx).unwrap_or_default();
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
            let prev_line = self.get_line(line_idx).unwrap_or_default();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_table_integration() {
        let mut doc = Document::new();
        
        // Verify piece table content is accessible
        let content = doc.get_piece_table_content();
        assert_eq!(content, "");
        
        println!("✅ Piece table integration working correctly");
    }

    #[test]
    fn test_piece_table_operations() {
        let mut doc = Document::new();
        // Piece table is now the only backend
        
        // Test insert_char uses piece table
        doc.move_cursor_to(0, 0);
        doc.insert_char('H');
        doc.insert_char('i');
        doc.insert_char('!');
        
        // Verify content using piece table methods
        assert_eq!(doc.get_line(0).unwrap_or_default(), "Hi!");
        assert_eq!(doc.cursor_column(), 3);
        
        let piece_table_content = doc.get_piece_table_content();
        assert_eq!(piece_table_content, "Hi!");
        
        // Test delete_char uses piece table  
        doc.delete_char(); // Delete '!'
        assert_eq!(doc.get_line(0).unwrap_or_default(), "Hi");
        assert_eq!(doc.cursor_column(), 2);
        
        let piece_table_content = doc.get_piece_table_content();
        assert_eq!(piece_table_content, "Hi");
        
        // Test insert_newline uses piece table
        doc.move_cursor_to(doc.cursor_line(), 1); // Position after 'H'
        doc.insert_newline();
        assert_eq!(doc.line_count(), 2);
        assert_eq!(doc.get_line(0).unwrap_or_default(), "H");
        assert_eq!(doc.get_line(1).unwrap_or_default(), "i");
        assert_eq!(doc.cursor_line(), 1);
        assert_eq!(doc.cursor_column(), 0);
        
        let piece_table_content = doc.get_piece_table_content();
        assert_eq!(piece_table_content, "H\ni");
        
        println!("✅ Piece table operations working correctly");
    }

    #[test]
    fn test_document_with_piece_table() {
        let mut doc = Document::new();
        
        // Verify initial state
        assert_eq!(doc.line_count(), 1);
        assert_eq!(doc.get_line(0).unwrap_or_default(), "");
        
        // Get piece table content
        let content = doc.get_piece_table_content();
        assert_eq!(content, "");
        
        println!("✅ Document creation with piece table successful");
    }

}
