use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    pub line: usize,
    pub column: usize,
    pub filename: Option<PathBuf>, // For global marks (A-Z)
}

impl Mark {
    pub fn new(line: usize, column: usize, filename: Option<PathBuf>) -> Self {
        Self {
            line,
            column,
            filename,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JumpListEntry {
    pub line: usize,
    pub column: usize,
    pub filename: Option<PathBuf>,
}

pub struct MarkManager {
    // Global marks (A-Z) - across all files
    global_marks: HashMap<char, Mark>,

    // Jump list for Ctrl+o / Ctrl+i navigation
    jump_list: Vec<JumpListEntry>,
    jump_position: usize,

    // Special marks
    last_jump: Option<Mark>,   // '' mark - last jump position
    last_change: Option<Mark>, // '. mark - last change position
    last_insert: Option<Mark>, // '^ mark - last insert position
}

impl MarkManager {
    pub fn new() -> Self {
        Self {
            global_marks: HashMap::new(),
            jump_list: Vec::new(),
            jump_position: 0,
            last_jump: None,
            last_change: None,
            last_insert: None,
        }
    }

    /// Set a global mark (A-Z) at the specified position
    pub fn set_global_mark(
        &mut self,
        mark_char: char,
        line: usize,
        column: usize,
        filename: Option<PathBuf>,
    ) -> Result<(), String> {
        match mark_char {
            'A'..='Z' => {
                // Global marks (A-Z) - store position and filename
                let mark = Mark::new(line, column, filename);
                self.global_marks.insert(mark_char, mark);
                Ok(())
            }
            _ => Err(format!("Invalid global mark character: {mark_char}")),
        }
    }

    /// Get a global mark (A-Z) or special mark
    pub fn get_global_mark(&self, mark_char: char) -> Option<&Mark> {
        match mark_char {
            'A'..='Z' => self.global_marks.get(&mark_char),
            '\'' => self.last_jump.as_ref(),  // '' - last jump
            '.' => self.last_change.as_ref(), // '. - last change
            '^' => self.last_insert.as_ref(), // '^ - last insert
            _ => None,
        }
    }

    /// Add a position to the jump list (for significant jumps)
    pub fn add_to_jump_list(&mut self, line: usize, column: usize, filename: Option<PathBuf>) {
        let entry = JumpListEntry {
            line,
            column,
            filename,
        };

        // Remove any duplicate entries at the current position
        if self.jump_position < self.jump_list.len() {
            self.jump_list.truncate(self.jump_position);
        }

        // Don't add if it's the same position as the last entry
        if let Some(last) = self.jump_list.last() {
            if last.line == line && last.column == column {
                return;
            }
        }

        self.jump_list.push(entry);

        // Limit jump list size to prevent memory issues
        const MAX_JUMP_LIST_SIZE: usize = 100;
        if self.jump_list.len() > MAX_JUMP_LIST_SIZE {
            self.jump_list.remove(0);
        } else {
            self.jump_position = self.jump_list.len();
        }
    }

    /// Jump backward in the jump list (Ctrl+o)
    pub fn jump_backward(&mut self) -> Option<&JumpListEntry> {
        if self.jump_position > 0 {
            self.jump_position -= 1;
            // Return the entry we're jumping to (older position)
            self.jump_list.get(self.jump_position)
        } else {
            None
        }
    }

    /// Jump forward in the jump list (Ctrl+i)
    pub fn jump_forward(&mut self) -> Option<&JumpListEntry> {
        if self.jump_position < self.jump_list.len() {
            let entry = self.jump_list.get(self.jump_position);
            self.jump_position += 1;
            entry
        } else {
            None
        }
    }

    /// Update the last jump position ('' mark)
    pub fn set_last_jump(&mut self, line: usize, column: usize) {
        self.last_jump = Some(Mark::new(line, column, None));
    }

    /// Update the last change position ('. mark)
    pub fn set_last_change(&mut self, line: usize, column: usize) {
        self.last_change = Some(Mark::new(line, column, None));
    }

    /// Update the last insert position ('^ mark)
    pub fn set_last_insert(&mut self, line: usize, column: usize) {
        self.last_insert = Some(Mark::new(line, column, None));
    }

    /// Clear all marks (global and special marks)
    pub fn clear_all_marks(&mut self) {
        self.global_marks.clear();
        self.last_jump = None;
        self.last_change = None;
        self.last_insert = None;
    }

    /// Clear only global marks (A-Z), keep special marks
    pub fn clear_global_marks(&mut self) {
        self.global_marks.clear();
    }

    /// Clear the jump list
    pub fn clear_jump_list(&mut self) {
        self.jump_list.clear();
        self.jump_position = 0;
    }

    /// Clear marks and jump list entries associated with a specific file
    /// Used when closing a buffer
    pub fn cleanup_for_closed_buffer(&mut self, closed_filename: Option<&std::path::PathBuf>) {
        // For global marks, only clear if they point to the closed file
        if let Some(filename) = closed_filename {
            self.global_marks
                .retain(|_mark, mark_data| mark_data.filename.as_ref() != Some(filename));

            // Remove jump list entries for the closed file
            self.jump_list
                .retain(|entry| entry.filename.as_ref() != Some(filename));

            // Adjust jump position if we removed entries
            if self.jump_position > self.jump_list.len() {
                self.jump_position = self.jump_list.len();
            }
        }

        // Clear special marks (they're buffer-specific)
        self.last_jump = None;
        self.last_change = None;
        self.last_insert = None;
    }

    /// List all marks (for :marks command)
    /// Takes local marks from the current document as parameter
    pub fn list_marks(
        &self,
        local_marks: &std::collections::HashMap<char, (usize, usize)>,
    ) -> Vec<(char, usize, usize, Option<&std::path::PathBuf>)> {
        let mut marks = Vec::new();

        // Add local marks from current document
        for (ch, (line, column)) in local_marks {
            marks.push((*ch, *line, *column, None));
        }

        // Add global marks
        for (ch, mark) in &self.global_marks {
            marks.push((*ch, mark.line, mark.column, mark.filename.as_ref()));
        }

        // Add special marks
        if let Some(mark) = &self.last_jump {
            marks.push(('\'', mark.line, mark.column, mark.filename.as_ref()));
        }
        if let Some(mark) = &self.last_change {
            marks.push(('.', mark.line, mark.column, mark.filename.as_ref()));
        }
        if let Some(mark) = &self.last_insert {
            marks.push(('^', mark.line, mark.column, mark.filename.as_ref()));
        }

        marks.sort_by_key(|(ch, _, _, _)| *ch);
        marks
    }

    /// Get jump list for display (for :jumps command)
    pub fn get_jump_list(&self) -> (&[JumpListEntry], usize) {
        (&self.jump_list, self.jump_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_marks() {
        use crate::document::Document;
        let mut doc = Document::new();

        // Set a local mark
        assert!(doc.set_local_mark('a', 10, 5).is_ok());

        // Retrieve the mark
        let (line, column) = doc.get_local_mark('a').unwrap();
        assert_eq!(line, 10);
        assert_eq!(column, 5);
    }

    #[test]
    fn test_global_marks() {
        let mut manager = MarkManager::new();
        let filename = Some(PathBuf::from("/tmp/test.txt"));

        // Set a global mark
        assert!(
            manager
                .set_global_mark('A', 20, 10, filename.clone())
                .is_ok()
        );

        // Retrieve the mark
        let mark = manager.get_global_mark('A').unwrap();
        assert_eq!(mark.line, 20);
        assert_eq!(mark.column, 10);
        assert_eq!(mark.filename, filename);
    }

    #[test]
    fn test_invalid_marks() {
        use crate::document::Document;
        let mut doc = Document::new();
        let mut manager = MarkManager::new();

        // Test invalid local mark characters
        assert!(doc.set_local_mark('1', 0, 0).is_err());
        assert!(doc.set_local_mark('!', 0, 0).is_err());
        assert!(doc.set_local_mark(' ', 0, 0).is_err());

        // Test invalid global mark characters
        assert!(manager.set_global_mark('1', 0, 0, None).is_err());
        assert!(manager.set_global_mark('!', 0, 0, None).is_err());
        assert!(manager.set_global_mark(' ', 0, 0, None).is_err());
    }

    #[test]
    fn test_jump_list() {
        let mut manager = MarkManager::new();

        // Simulate a user jumping around:
        // They were at line 10, then jumped to 20, then to 30
        manager.add_to_jump_list(10, 0, None);
        manager.add_to_jump_list(20, 5, None);
        manager.add_to_jump_list(30, 10, None);

        // At this point, position = 3, list = [10, 20, 30]
        // User is conceptually "at" line 30 (most recent jump)

        // Ctrl+o should take us backward through the jump history
        let entry = manager.jump_backward().unwrap(); // position becomes 2
        assert_eq!(entry.line, 30); // We jump to the most recent recorded position

        let entry = manager.jump_backward().unwrap(); // position becomes 1  
        assert_eq!(entry.line, 20); // Previous position

        let entry = manager.jump_backward().unwrap(); // position becomes 0
        assert_eq!(entry.line, 10); // Earlier position

        // No more backward jumps
        assert!(manager.jump_backward().is_none());

        // Test forward jumps (Ctrl+i)
        let entry = manager.jump_forward().unwrap(); // position becomes 1
        assert_eq!(entry.line, 10); // We're at position 0, so return entry 0 and advance to 1
    }

    #[test]
    fn test_special_marks() {
        let mut manager = MarkManager::new();

        // Test last jump mark
        manager.set_last_jump(15, 8);
        let mark = manager.get_global_mark('\'').unwrap();
        assert_eq!(mark.line, 15);
        assert_eq!(mark.column, 8);

        // Test last change mark
        manager.set_last_change(25, 12);
        let mark = manager.get_global_mark('.').unwrap();
        assert_eq!(mark.line, 25);
        assert_eq!(mark.column, 12);

        // Test last insert mark
        manager.set_last_insert(35, 16);
        let mark = manager.get_global_mark('^').unwrap();
        assert_eq!(mark.line, 35);
        assert_eq!(mark.column, 16);
    }

    #[test]
    fn test_clear_marks() {
        use crate::document::Document;
        let mut doc = Document::new();
        let mut manager = MarkManager::new();

        // Set some marks
        doc.set_local_mark('a', 10, 5).unwrap();
        manager
            .set_global_mark('A', 20, 10, Some(std::path::PathBuf::from("test.txt")))
            .unwrap();
        manager.set_last_jump(30, 15);

        // Verify marks are set
        assert!(doc.get_local_mark('a').is_some());
        assert!(manager.get_global_mark('A').is_some());
        assert!(manager.get_global_mark('\'').is_some());

        // Clear user marks only
        doc.clear_local_marks();
        manager.clear_global_marks();
        assert!(doc.get_local_mark('a').is_none());
        assert!(manager.get_global_mark('A').is_none());
        assert!(manager.get_global_mark('\'').is_some()); // Special marks should remain

        // Set marks again and clear all
        doc.set_local_mark('b', 40, 20).unwrap();
        manager.clear_all_marks();
        assert!(doc.get_local_mark('b').is_some()); // Local marks not affected by clear_all_marks
        assert!(manager.get_global_mark('\'').is_none()); // Special marks cleared too
    }

    #[test]
    fn test_clear_jump_list() {
        let mut manager = MarkManager::new();

        // Add some jumps
        manager.add_to_jump_list(10, 0, None);
        manager.add_to_jump_list(20, 5, None);
        manager.add_to_jump_list(30, 10, None);

        // Verify jump list has entries
        assert!(manager.jump_backward().is_some());

        // Clear jump list
        manager.clear_jump_list();

        // Verify jump list is empty
        assert!(manager.jump_backward().is_none());
        assert_eq!(manager.jump_position, 0);
    }

    #[test]
    fn test_get_jump_list() {
        let mut manager = MarkManager::new();

        // Add some jumps
        manager.add_to_jump_list(10, 0, None);
        manager.add_to_jump_list(20, 5, Some(std::path::PathBuf::from("test.txt")));
        manager.add_to_jump_list(30, 10, None);

        let (jump_list, position) = manager.get_jump_list();

        assert_eq!(jump_list.len(), 3);
        assert_eq!(position, 3); // Should be at end
        assert_eq!(jump_list[0].line, 10);
        assert_eq!(jump_list[1].line, 20);
        assert_eq!(jump_list[2].line, 30);
        assert_eq!(
            jump_list[1].filename,
            Some(std::path::PathBuf::from("test.txt"))
        );
    }

    #[test]
    fn test_cleanup_for_closed_buffer() {
        let mut manager = MarkManager::new();
        let file1 = Some(std::path::PathBuf::from("file1.txt"));
        let file2 = Some(std::path::PathBuf::from("file2.txt"));

        // Set up marks and jumps (only global marks now)
        manager.set_global_mark('A', 20, 10, file1.clone()).unwrap(); // Global mark for file1
        manager.set_global_mark('B', 30, 15, file2.clone()).unwrap(); // Global mark for file2
        manager.set_last_jump(5, 5);

        // Add jump entries
        manager.add_to_jump_list(40, 0, file1.clone());
        manager.add_to_jump_list(50, 5, file2.clone());
        manager.add_to_jump_list(60, 10, None);

        // Verify everything is set
        assert!(manager.get_global_mark('A').is_some());
        assert!(manager.get_global_mark('B').is_some());
        assert!(manager.get_global_mark('\'').is_some());
        assert_eq!(manager.jump_list.len(), 3);

        // Close file1
        manager.cleanup_for_closed_buffer(file1.as_ref());

        // Special marks should be cleared
        assert!(manager.get_global_mark('\'').is_none());

        // Global mark for file1 should be gone, but file2 should remain
        assert!(manager.get_global_mark('A').is_none());
        assert!(manager.get_global_mark('B').is_some());

        // Jump list should only contain entries not from file1
        assert_eq!(manager.jump_list.len(), 2); // file2 and None entries remain
        assert!(
            manager
                .jump_list
                .iter()
                .all(|entry| entry.filename != file1)
        );
    }
}
