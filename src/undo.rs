#[derive(Debug, Clone)]
pub enum UndoAction {
    InsertText {
        line: usize,
        column: usize,
        text: String,
    },
    DeleteText {
        line: usize,
        column: usize,
        text: String,
    },
    InsertLine {
        line: usize,
        text: String,
    },
    DeleteLine {
        line: usize,
        text: String,
    },
    SplitLine {
        line: usize,
        column: usize,
        text: String, // The text that was moved to the new line
    },
    JoinLines {
        line: usize,
        separator: String,        // What was used to join (space, nothing, etc.)
        second_line_text: String, // The text from the second line
    },
}

impl UndoAction {
    /// Apply this undo action to a document
    pub fn apply_to_document(&self, document: &mut crate::document::Document) {
        match self {
            UndoAction::InsertText { line, column, text } => {
                // Make sure the line exists
                if *line < document.line_count() {
                    document.insert_text_at(*line, *column, text);
                }
            }
            UndoAction::DeleteText { line, column, text } => {
                // Make sure the line exists and has enough content
                if *line < document.line_count() {
                    let current_line_len = document.get_line_length(*line);
                    if *column <= current_line_len {
                        let end_col = (*column + text.len()).min(current_line_len);
                        if end_col > *column {
                            document.delete_text_at(*line, *column, end_col - *column);
                        }
                    }
                }
            }
            UndoAction::InsertLine { line, text } => {
                document.insert_line_at(*line, text);
            }
            UndoAction::DeleteLine { line, .. } => {
                if *line < document.line_count() {
                    document.delete_line_at(*line);
                }
            }
            UndoAction::SplitLine { line, column, text } => {
                if *line < document.line_count() {
                    document.split_line_at(*line, *column, text);
                }
            }
            UndoAction::JoinLines { line, separator, .. } => {
                // Join the line at `line` with the line at `line + 1`
                if *line < document.line_count().saturating_sub(1) {
                    document.join_lines_at(*line, separator);
                }
            }
        }
    }

    pub fn reverse(&self) -> UndoAction {
        match self {
            UndoAction::InsertText { line, column, text } => UndoAction::DeleteText {
                line: *line,
                column: *column,
                text: text.clone(),
            },
            UndoAction::DeleteText { line, column, text } => UndoAction::InsertText {
                line: *line,
                column: *column,
                text: text.clone(),
            },
            UndoAction::InsertLine { line, text } => UndoAction::DeleteLine {
                line: *line,
                text: text.clone(),
            },
            UndoAction::DeleteLine { line, text } => UndoAction::InsertLine {
                line: *line,
                text: text.clone(),
            },
            UndoAction::SplitLine {
                line,
                column: _,
                text,
            } => UndoAction::JoinLines {
                line: *line,
                separator: String::new(),
                second_line_text: text.clone(),
            },
            UndoAction::JoinLines {
                line,
                separator: _,
                second_line_text,
            } => UndoAction::SplitLine {
                line: *line,
                column: 0, // Will be recalculated based on the line content
                text: second_line_text.clone(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct UndoGroup {
    pub actions: Vec<UndoAction>,
    pub cursor_before: (usize, usize),
    pub cursor_after: (usize, usize),
}

impl UndoGroup {
    pub fn new(cursor_pos: (usize, usize)) -> Self {
        Self {
            actions: Vec::new(),
            cursor_before: cursor_pos,
            cursor_after: cursor_pos,
        }
    }

    pub fn add_action(&mut self, action: UndoAction) {
        self.actions.push(action);
    }

    pub fn set_cursor_after(&mut self, cursor_pos: (usize, usize)) {
        self.cursor_after = cursor_pos;
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Apply this undo group to a document (for redo operations)
    pub fn apply_to_document(&self, document: &mut crate::document::Document) {
        // Apply actions in forward order for redo
        for action in &self.actions {
            action.apply_to_document(document);
        }
        // Set cursor to after position
        document.cursor_line = self.cursor_after.0.min(document.line_count().saturating_sub(1));
        document.cursor_column = self.cursor_after.1.min(document.get_line_length(document.cursor_line));
    }

    /// Apply the reverse of this undo group to a document (for undo operations)  
    pub fn apply_reverse_to_document(&self, document: &mut crate::document::Document) {
        // Apply reverse actions in reverse order for undo
        for action in self.actions.iter().rev() {
            let reverse_action = action.reverse();
            reverse_action.apply_to_document(document);
        }
        // Set cursor to before position
        document.cursor_line = self.cursor_before.0.min(document.line_count().saturating_sub(1));
        document.cursor_column = self.cursor_before.1.min(document.get_line_length(document.cursor_line));
    }
}

#[derive(Clone)]
pub struct UndoManager {
    undo_stack: Vec<UndoGroup>,
    redo_stack: Vec<UndoGroup>,
    current_group: Option<UndoGroup>,
    max_undo_levels: usize,
}

impl UndoManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_group: None,
            max_undo_levels: 1000,
        }
    }

    pub fn start_group(&mut self, cursor_pos: (usize, usize)) {
        if let Some(group) = self.current_group.take() {
            if !group.is_empty() {
                self.push_undo_group(group);
            }
        }
        self.current_group = Some(UndoGroup::new(cursor_pos));
    }

    pub fn add_action(&mut self, action: UndoAction) {
        if let Some(ref mut group) = self.current_group {
            group.add_action(action);
        } else {
            // If no group is active, create one with a default cursor position
            let mut group = UndoGroup::new((0, 0));
            group.add_action(action);
            self.current_group = Some(group);
        }
    }

    pub fn end_group(&mut self, cursor_pos: (usize, usize)) {
        if let Some(mut group) = self.current_group.take() {
            if !group.is_empty() {
                group.set_cursor_after(cursor_pos);
                self.push_undo_group(group);
            }
        }
    }

    fn push_undo_group(&mut self, group: UndoGroup) {
        self.undo_stack.push(group);

        // Limit the undo stack size
        if self.undo_stack.len() > self.max_undo_levels {
            self.undo_stack.remove(0);
        }

        // Clear redo stack when new actions are performed
        self.redo_stack.clear();
    }

    #[allow(dead_code)]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    #[allow(dead_code)]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> Option<UndoGroup> {
        // First, finish any current group
        if let Some(group) = self.current_group.take() {
            if !group.is_empty() {
                self.push_undo_group(group);
            }
        }

        if let Some(group) = self.undo_stack.pop() {
            self.redo_stack.push(group.clone());
            Some(group)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<UndoGroup> {
        if let Some(group) = self.redo_stack.pop() {
            self.undo_stack.push(group.clone());
            Some(group)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_group = None;
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}
