use crate::buffer_manager::BufferManager;
use crate::key_handler::KeyHandler;
use crate::marks::MarkManager;
use crate::registers::RegisterManager;
use crate::search::{SearchState, SearchDirection};
use crate::undo::{UndoAction, UndoGroup};
use crate::view::{View, RenderParams};
use crate::visual_mode::{Selection, VisualMode, VisualModeHandler};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use std::io::stdout;

pub enum Command {
    // Basic movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    // Word movement
    MoveWordForward,
    MoveWordBackward,
    MoveWordEnd,
    MoveBigWordForward,
    MoveBigWordBackward,
    MoveBigWordEnd,

    // Line movement
    MoveLineStart,
    MoveLineEnd,
    MoveFirstNonWhitespace,
    MoveDownToFirstNonWhitespace,
    MoveUpToFirstNonWhitespace,

    // Document movement
    MoveDocumentStart,
    MoveDocumentEnd,
    MovePageUp,
    MovePageDown,
    MoveHalfPageUp,
    MoveHalfPageDown,

    // Line jumping
    #[allow(dead_code)] // Will be wired up in key handler
    MoveToLine(usize),
    
    // Screen positioning
    MoveToScreenTop,    // H
    MoveToScreenMiddle, // M
    MoveToScreenBottom, // L
    
    // Bracket matching
    MatchBracket, // %

    // Character search
    #[allow(dead_code)] // Will be wired up in key handler
    FindChar(char),
    #[allow(dead_code)] // Will be wired up in key handler
    FindCharBackward(char),
    #[allow(dead_code)] // Will be wired up in key handler
    FindCharBefore(char),
    #[allow(dead_code)] // Will be wired up in key handler
    FindCharBeforeBackward(char),
    RepeatFind,
    RepeatFindReverse,

    // Mark commands
    SetMark(char),
    JumpToMark(char),
    JumpToMarkLine(char),
    JumpBackward,
    JumpForward,

    // Insert modes
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterInsertModeNewLine,
    EnterInsertModeNewLineAbove,
    EnterInsertModeLineEnd,
    EnterInsertModeLineStart,

    // Indentation commands
    IndentLine,
    IndentLines(usize), // count of lines
    DedentLine,
    DedentLines(usize), // count of lines

    // Search commands
    EnterSearchMode,
    EnterSearchBackwardMode,
    SearchForward(String),
    SearchBackward(String),
    SearchNext,
    SearchPrevious,
    ExitSearchMode,
    SearchWordUnderCursor,         // *
    SearchWordUnderCursorBackward, // #

    // Other commands
    EnterCommandMode,
    Execute(String),
    InsertChar(char),
    InsertNewline,
    InsertTab,
    DeleteChar,
    DeleteCharForward,
    DeleteCharBackward,
    DeleteLine,
    DeleteLines(usize), // count of lines
    DeleteToEndOfLine,
    DeleteWord,
    DeleteBigWord,
    DeleteWordBackward,
    DeleteBigWordBackward,
    DeleteToEndOfWord,
    DeleteToEndOfBigWord,
    DeleteToStartOfLine,
    DeleteToFirstNonWhitespace,
    DeleteToEndOfFile,
    DeleteToStartOfFile,
    SubstituteChar,
    SubstituteLine,
    DeleteUntilChar(char),
    DeleteUntilCharBackward(char),
    DeleteFindChar(char),
    DeleteFindCharBackward(char),
    
    // Change commands (delete + enter insert mode)
    ChangeLine,
    ChangeLines(usize),
    ChangeToEndOfLine,
    ChangeWord,
    ChangeBigWord,
    ChangeWordBackward,
    ChangeBigWordBackward,
    ChangeToEndOfWord,
    ChangeToEndOfBigWord,
    ChangeToStartOfLine,
    ChangeToFirstNonWhitespace,
    ChangeToEndOfFile,
    ChangeToStartOfFile,
    ChangeUntilChar(char),
    ChangeUntilCharBackward(char),
    ChangeFindChar(char),
    ChangeFindCharBackward(char),
    
    // Yank and paste commands (simplified)
    Yank(crate::yank_paste_handler::YankType, Option<char>),
    Paste(crate::yank_paste_handler::PasteType, Option<char>),
    
    // Visual mode commands
    EnterVisualChar,
    EnterVisualLine,
    EnterVisualBlock,
    ExitVisualMode,
    VisualDelete,
    VisualIndent,
    VisualDedent,
    VisualYank,
    
    ExitInsertMode,
    Redraw,
    
    // Line operations
    JoinLines,
    
    // Case operations
    ToggleCase,
    Lowercase,
    Uppercase,
    
    // Undo/Redo commands
    Undo,
    Redo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
    SearchBackward,
    VisualChar,
    VisualLine,
    VisualBlock,
}

pub struct Controller {
    pub buffer_manager: BufferManager,
    pub view: View,
    pub mode: Mode,
    pub command_buffer: String,
    pub status_message: String,
    pub last_find_char: Option<char>,
    pub last_find_forward: bool,
    pub last_find_before: bool,
    pub pending_key: Option<char>,
    pub number_prefix: Option<usize>,
    pub pending_register: Option<char>,
    pub visual_selection: Option<Selection>,
    pub search_state: SearchState,
    pub mark_manager: MarkManager,
    pub register_manager: RegisterManager,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            buffer_manager: BufferManager::new(),
            view: View::new(),
            mode: Mode::Normal,
            command_buffer: String::new(),
            status_message: String::new(),
            last_find_char: None,
            last_find_forward: true,
            last_find_before: false,
            pending_key: None,
            number_prefix: None,
            pending_register: None,
            visual_selection: None,
            search_state: SearchState::new(),
            mark_manager: MarkManager::new(),
            register_manager: RegisterManager::new(),
        }
    }


    pub fn new_with_files(filenames: Vec<std::path::PathBuf>) -> Result<Self, std::io::Error> {
        Ok(Self {
            buffer_manager: BufferManager::new_with_files(filenames)?,
            view: View::new(),
            mode: Mode::Normal,
            command_buffer: String::new(),
            status_message: String::new(),
            last_find_char: None,
            last_find_forward: true,
            last_find_before: false,
            pending_key: None,
            number_prefix: None,
            pending_register: None,
            visual_selection: None,
            search_state: SearchState::new(),
            mark_manager: MarkManager::new(),
            register_manager: RegisterManager::new(),
        })
    }

    pub fn current_document(&self) -> &crate::document::Document {
        self.buffer_manager.current_document()
    }

    pub fn current_document_mut(&mut self) -> &mut crate::document::Document {
        self.buffer_manager.current_document_mut()
    }

    fn update_visual_selection(&mut self) {
        if self.visual_selection.is_some() {
            let doc = self.current_document();
            let line = doc.cursor_line;
            let column = doc.cursor_column;
            if let Some(ref mut selection) = self.visual_selection {
                selection.update_end(line, column);
            }
        }
    }

    fn execute_command(&mut self, command: Command) -> bool {
        match command {
            // Movement commands
            Command::MoveUp | Command::MoveDown | Command::MoveLeft | Command::MoveRight |
            Command::MoveWordForward | Command::MoveWordBackward | Command::MoveWordEnd |
            Command::MoveBigWordForward | Command::MoveBigWordBackward | Command::MoveBigWordEnd |
            Command::MoveLineStart | Command::MoveLineEnd | Command::MoveFirstNonWhitespace |
            Command::MoveDownToFirstNonWhitespace | Command::MoveUpToFirstNonWhitespace |
            Command::MoveDocumentStart | Command::MoveDocumentEnd | Command::MovePageUp |
            Command::MovePageDown | Command::MoveHalfPageUp | Command::MoveHalfPageDown |
            Command::MoveToLine(_) | Command::MoveToScreenTop | Command::MoveToScreenMiddle | 
            Command::MoveToScreenBottom | Command::MatchBracket => {
                self.execute_movement_command(command);
                false
            }
            
            // Character search commands
            Command::FindChar(_) | Command::FindCharBackward(_) | Command::FindCharBefore(_) |
            Command::FindCharBeforeBackward(_) | Command::RepeatFind | Command::RepeatFindReverse => {
                self.execute_character_search_command(command);
                false
            }
            
            // Mark commands
            Command::SetMark(_) | Command::JumpToMark(_) | Command::JumpToMarkLine(_) |
            Command::JumpBackward | Command::JumpForward => {
                self.execute_mark_command(command);
                false
            }
            
            // Search commands
            Command::EnterSearchMode | Command::EnterSearchBackwardMode | Command::SearchForward(_) | Command::SearchBackward(_) |
            Command::SearchNext | Command::SearchPrevious | Command::ExitSearchMode | 
            Command::SearchWordUnderCursor | Command::SearchWordUnderCursorBackward => {
                self.execute_search_command(command);
                false
            }
            
            // Insert mode commands
            Command::EnterInsertMode | Command::EnterInsertModeAfter | Command::EnterInsertModeNewLine |
            Command::EnterInsertModeNewLineAbove | Command::EnterInsertModeLineEnd | Command::EnterInsertModeLineStart |
            Command::ExitInsertMode | Command::InsertChar(_) | Command::InsertNewline | Command::InsertTab => {
                self.execute_insert_command(command);
                false
            }
            
            // Edit commands
            Command::DeleteChar | Command::DeleteCharForward | Command::DeleteCharBackward | Command::DeleteLine | Command::DeleteLines(_) |
            Command::DeleteToEndOfLine | Command::DeleteWord | Command::DeleteBigWord | Command::DeleteWordBackward |
            Command::DeleteBigWordBackward | Command::DeleteToEndOfWord | Command::DeleteToEndOfBigWord |
            Command::DeleteToStartOfLine | Command::DeleteToFirstNonWhitespace | Command::DeleteToEndOfFile |
            Command::DeleteToStartOfFile | Command::SubstituteChar | Command::SubstituteLine |
            Command::DeleteUntilChar(_) | Command::DeleteUntilCharBackward(_) | Command::DeleteFindChar(_) |
            Command::DeleteFindCharBackward(_) => {
                self.execute_edit_command(command);
                false
            }
            
            // Change commands (delete + enter insert mode)
            Command::ChangeLine | Command::ChangeLines(_) | Command::ChangeToEndOfLine |
            Command::ChangeWord | Command::ChangeBigWord | Command::ChangeWordBackward | Command::ChangeBigWordBackward |
            Command::ChangeToEndOfWord | Command::ChangeToEndOfBigWord | Command::ChangeToStartOfLine |
            Command::ChangeToFirstNonWhitespace | Command::ChangeToEndOfFile | Command::ChangeToStartOfFile |
            Command::ChangeUntilChar(_) | Command::ChangeUntilCharBackward(_) | Command::ChangeFindChar(_) |
            Command::ChangeFindCharBackward(_) => {
                self.execute_change_command(command);
                false
            }
            
            // Yank and paste commands
            Command::Yank(yank_type, register) => {
                crate::yank_paste_handler::YankPasteHandler::execute_yank(self, yank_type, register);
                false
            }
            Command::Paste(paste_type, register) => {
                crate::yank_paste_handler::YankPasteHandler::execute_paste(self, paste_type, register);
                false
            }
            
            // Visual mode commands
            Command::EnterVisualChar | Command::EnterVisualLine | Command::EnterVisualBlock |
            Command::ExitVisualMode | Command::VisualDelete | Command::VisualIndent | Command::VisualDedent |
            Command::VisualYank => {
                self.execute_visual_command(command);
                false
            }
            
            // Indentation commands
            Command::IndentLine | Command::IndentLines(_) | Command::DedentLine | Command::DedentLines(_) => {
                self.execute_indentation_command(command);
                false
            }
            
            // Line operations
            Command::JoinLines => {
                self.execute_join_lines_command();
                false
            }
            
            // Case operations
            Command::ToggleCase | Command::Lowercase | Command::Uppercase => {
                self.execute_case_command(command);
                false
            }
            
            // Undo/Redo commands
            Command::Undo | Command::Redo => {
                self.execute_undo_redo_command(command);
                false
            }
            
            // Command mode and other commands
            Command::EnterCommandMode | Command::Execute(_) | Command::Redraw => {
                self.execute_other_command(command)
            }
        }
    }

    fn execute_movement_command(&mut self, command: Command) {
        match command {
            // Basic movement
            Command::MoveUp => {
                self.current_document_mut().move_cursor_up();
                self.update_visual_selection();
            }
            Command::MoveDown => {
                self.current_document_mut().move_cursor_down();
                self.update_visual_selection();
            }
            Command::MoveLeft => {
                self.current_document_mut().move_cursor_left();
                self.update_visual_selection();
            }
            Command::MoveRight => {
                self.current_document_mut().move_cursor_right();
                self.update_visual_selection();
            }

            // Word movement
            Command::MoveWordForward => {
                self.current_document_mut().move_word_forward();
                self.update_visual_selection();
            }
            Command::MoveWordBackward => {
                self.current_document_mut().move_word_backward();
                self.update_visual_selection();
            }
            Command::MoveWordEnd => {
                self.current_document_mut().move_word_end();
                self.update_visual_selection();
            }
            Command::MoveBigWordForward => {
                self.current_document_mut().move_big_word_forward();
                self.update_visual_selection();
            }
            Command::MoveBigWordBackward => {
                self.current_document_mut().move_big_word_backward();
                self.update_visual_selection();
            }
            Command::MoveBigWordEnd => {
                self.current_document_mut().move_big_word_end();
                self.update_visual_selection();
            }

            // Line movement
            Command::MoveLineStart => {
                self.current_document_mut().move_line_start();
                self.update_visual_selection();
            }
            Command::MoveLineEnd => {
                self.current_document_mut().move_line_end();
                self.update_visual_selection();
            }
            Command::MoveFirstNonWhitespace => {
                self.current_document_mut().move_first_non_whitespace()
            }
            Command::MoveDownToFirstNonWhitespace => self
                .current_document_mut()
                .move_down_to_first_non_whitespace(),
            Command::MoveUpToFirstNonWhitespace => self
                .current_document_mut()
                .move_up_to_first_non_whitespace(),

            // Document movement
            Command::MoveDocumentStart => self.current_document_mut().move_document_start(),
            Command::MoveDocumentEnd => self.current_document_mut().move_document_end(),
            Command::MovePageUp => self.current_document_mut().move_page_up(),
            Command::MovePageDown => self.current_document_mut().move_page_down(),
            Command::MoveHalfPageUp => self.current_document_mut().move_half_page_up(),
            Command::MoveHalfPageDown => self.current_document_mut().move_half_page_down(),
            
            // Screen positioning
            Command::MoveToScreenTop => self.move_to_screen_top(),
            Command::MoveToScreenMiddle => self.move_to_screen_middle(),
            Command::MoveToScreenBottom => self.move_to_screen_bottom(),

            // Line jumping
            Command::MoveToLine(line) => self.current_document_mut().move_to_line(line),
            
            // Bracket matching
            Command::MatchBracket => self.match_bracket(),
            
            _ => {} // Should not reach here
        }
    }

    fn execute_character_search_command(&mut self, command: Command) {
        match command {
            Command::FindChar(c) => {
                self.current_document_mut().find_char(c, true, false);
                self.last_find_char = Some(c);
                self.last_find_forward = true;
                self.last_find_before = false;
            }
            Command::FindCharBackward(c) => {
                self.current_document_mut().find_char(c, false, false);
                self.last_find_char = Some(c);
                self.last_find_forward = false;
                self.last_find_before = false;
            }
            Command::FindCharBefore(c) => {
                self.current_document_mut().find_char(c, true, true);
                self.last_find_char = Some(c);
                self.last_find_forward = true;
                self.last_find_before = true;
            }
            Command::FindCharBeforeBackward(c) => {
                self.current_document_mut().find_char(c, false, true);
                self.last_find_char = Some(c);
                self.last_find_forward = false;
                self.last_find_before = true;
            }
            Command::RepeatFind => {
                if let Some(c) = self.last_find_char {
                    let forward = self.last_find_forward;
                    let before = self.last_find_before;
                    self.current_document_mut().find_char(c, forward, before);
                }
            }
            Command::RepeatFindReverse => {
                if let Some(c) = self.last_find_char {
                    let forward = !self.last_find_forward;
                    let before = self.last_find_before;
                    self.current_document_mut().find_char(c, forward, before);
                }
            }

            _ => {} // Should not reach here
        }
    }

    fn execute_mark_command(&mut self, command: Command) {
        match command {
            Command::SetMark(mark_char) => {
                let doc = self.current_document();
                let line = doc.cursor_line;
                let column = doc.cursor_column;
                let filename = doc.filename.clone();
                
                if mark_char.is_ascii_lowercase() {
                    // Local mark - set on current document
                    if let Err(err) = self.current_document_mut().set_local_mark(mark_char, line, column) {
                        self.status_message = err;
                    } else {
                        self.status_message = format!("Mark '{}' set at line {}, column {}", mark_char, line + 1, column + 1);
                    }
                } else if mark_char.is_ascii_uppercase() {
                    // Global mark - set on mark manager
                    if let Err(err) = self.mark_manager.set_global_mark(mark_char, line, column, filename) {
                        self.status_message = err;
                    } else {
                        self.status_message = format!("Mark '{}' set at line {}, column {}", mark_char, line + 1, column + 1);
                    }
                } else {
                    self.status_message = format!("Invalid mark character: {mark_char}");
                }
            }
            Command::JumpToMark(mark_char) => {
                if mark_char.is_ascii_lowercase() {
                    // Local mark - check current document
                    if let Some((line, column)) = self.current_document().get_local_mark(mark_char) {
                        // Get current position before borrowing mutably
                        let (current_line, current_col, current_filename) = {
                            let doc = self.current_document();
                            (doc.cursor_line, doc.cursor_column, doc.filename.clone())
                        };
                        
                        // Add current position to jump list before jumping
                        self.mark_manager.add_to_jump_list(current_line, current_col, current_filename);
                        
                        // Update last jump position
                        self.mark_manager.set_last_jump(current_line, current_col);
                        
                        // Jump to mark (exact position)
                        let doc_mut = self.current_document_mut();
                        doc_mut.cursor_line = line;
                        doc_mut.cursor_column = column;
                        
                        self.status_message = format!("Jumped to mark '{}' at line {}, column {}", 
                            mark_char, line + 1, column + 1);
                    } else {
                        self.status_message = format!("Mark '{mark_char}' not set");
                    }
                } else {
                    // Global mark or special mark - check mark manager
                    if let Some(mark) = self.mark_manager.get_global_mark(mark_char).cloned() {
                        // Get current position before borrowing mutably
                        let (current_line, current_col, current_filename) = {
                            let doc = self.current_document();
                            (doc.cursor_line, doc.cursor_column, doc.filename.clone())
                        };
                        
                        // Add current position to jump list before jumping
                        self.mark_manager.add_to_jump_list(current_line, current_col, current_filename);
                        
                        // Update last jump position
                        self.mark_manager.set_last_jump(current_line, current_col);
                        
                        // For global marks (A-Z), switch to the correct buffer if needed
                        if mark_char.is_ascii_uppercase() && mark.filename.is_some() {
                            let target_filename = mark.filename.as_ref().unwrap();
                            if let Err(e) = self.buffer_manager.switch_to_file(target_filename) {
                                self.status_message = format!("Cannot open file for mark '{mark_char}': {e}");
                                return;
                            }
                            self.view.reset_scroll(); // Reset scroll when switching buffers
                        }
                        
                        // Jump to mark (exact position)
                        let doc_mut = self.current_document_mut();
                        doc_mut.cursor_line = mark.line;
                        doc_mut.cursor_column = mark.column;
                        
                        let filename_info = if let Some(ref filename) = mark.filename {
                            format!(" in {}", filename.display())
                        } else {
                            String::new()
                        };
                        self.status_message = format!("Jumped to mark '{}' at line {}, column {}{}", 
                            mark_char, mark.line + 1, mark.column + 1, filename_info);
                    } else {
                        self.status_message = format!("Mark '{mark_char}' not set");
                    }
                }
            }
            Command::JumpToMarkLine(mark_char) => {
                if mark_char.is_ascii_lowercase() {
                    // Local mark - check current document
                    if let Some((line, _column)) = self.current_document().get_local_mark(mark_char) {
                        // Get current position before borrowing mutably
                        let (current_line, current_col, current_filename) = {
                            let doc = self.current_document();
                            (doc.cursor_line, doc.cursor_column, doc.filename.clone())
                        };
                        
                        // Add current position to jump list before jumping
                        self.mark_manager.add_to_jump_list(current_line, current_col, current_filename);
                        
                        // Update last jump position
                        self.mark_manager.set_last_jump(current_line, current_col);
                        
                        // Jump to mark line, first non-whitespace character
                        let doc_mut = self.current_document_mut();
                        doc_mut.cursor_line = line;
                        doc_mut.move_first_non_whitespace();
                        
                        self.status_message = format!("Jumped to mark '{}' line {}", 
                            mark_char, line + 1);
                    } else {
                        self.status_message = format!("Mark '{mark_char}' not set");
                    }
                } else {
                    // Global mark or special mark - check mark manager
                    if let Some(mark) = self.mark_manager.get_global_mark(mark_char).cloned() {
                        // Get current position before borrowing mutably
                        let (current_line, current_col, current_filename) = {
                            let doc = self.current_document();
                            (doc.cursor_line, doc.cursor_column, doc.filename.clone())
                        };
                        
                        // Add current position to jump list before jumping
                        self.mark_manager.add_to_jump_list(current_line, current_col, current_filename);
                        
                        // Update last jump position
                        self.mark_manager.set_last_jump(current_line, current_col);
                        
                        // For global marks (A-Z), switch to the correct buffer if needed
                        if mark_char.is_ascii_uppercase() && mark.filename.is_some() {
                            let target_filename = mark.filename.as_ref().unwrap();
                            if let Err(e) = self.buffer_manager.switch_to_file(target_filename) {
                                self.status_message = format!("Cannot open file for mark '{mark_char}': {e}");
                                return;
                            }
                            self.view.reset_scroll(); // Reset scroll when switching buffers
                        }
                        
                        // Jump to mark line, first non-whitespace character
                        let doc_mut = self.current_document_mut();
                        doc_mut.cursor_line = mark.line;
                        doc_mut.move_first_non_whitespace();
                        
                        let filename_info = if let Some(ref filename) = mark.filename {
                            format!(" in {}", filename.display())
                        } else {
                            String::new()
                        };
                        self.status_message = format!("Jumped to mark '{}' line {}{}", 
                            mark_char, mark.line + 1, filename_info);
                    } else {
                        self.status_message = format!("Mark '{mark_char}' not set");
                    }
                }
            }
            Command::JumpBackward => {
                if let Some(entry) = self.mark_manager.jump_backward().cloned() {
                    // Switch buffer if needed
                    if let Some(ref filename) = entry.filename {
                        if let Err(e) = self.buffer_manager.switch_to_file(filename) {
                            self.status_message = format!("Cannot open file for jump: {e}");
                            return;
                        }
                        self.view.reset_scroll();
                    }
                    
                    let doc_mut = self.current_document_mut();
                    doc_mut.cursor_line = entry.line;
                    doc_mut.cursor_column = entry.column;
                    
                    let filename_info = if let Some(ref filename) = entry.filename {
                        format!(" in {}", filename.display())
                    } else {
                        String::new()
                    };
                    self.status_message = format!("Jumped backward to line {}, column {}{}", 
                        entry.line + 1, entry.column + 1, filename_info);
                } else {
                    self.status_message = "Already at oldest change".to_string();
                }
            }
            Command::JumpForward => {
                if let Some(entry) = self.mark_manager.jump_forward().cloned() {
                    // Switch buffer if needed
                    if let Some(ref filename) = entry.filename {
                        if let Err(e) = self.buffer_manager.switch_to_file(filename) {
                            self.status_message = format!("Cannot open file for jump: {e}");
                            return;
                        }
                        self.view.reset_scroll();
                    }
                    
                    let doc_mut = self.current_document_mut();
                    doc_mut.cursor_line = entry.line;
                    doc_mut.cursor_column = entry.column;
                    
                    let filename_info = if let Some(ref filename) = entry.filename {
                        format!(" in {}", filename.display())
                    } else {
                        String::new()
                    };
                    self.status_message = format!("Jumped forward to line {}, column {}{}", 
                        entry.line + 1, entry.column + 1, filename_info);
                } else {
                    self.status_message = "Already at newest change".to_string();
                }
            }
            
            _ => {} // Should not reach here
        }
    }

    fn execute_search_command(&mut self, command: Command) {
        match command {
            Command::EnterSearchMode => {
                self.mode = Mode::Search;
                self.command_buffer.clear();
            }
            Command::EnterSearchBackwardMode => {
                self.mode = Mode::SearchBackward;
                self.command_buffer.clear();
            }
            Command::SearchForward(pattern) => {
                // Set pattern first
                if let Err(e) = self.search_state.set_pattern(pattern.clone(), SearchDirection::Forward) {
                    self.status_message = e.to_string();
                } else {
                    // Use split borrowing to access buffer_manager and search_state separately
                    let doc = self.buffer_manager.current_document();
                    let cursor_line = doc.cursor_line;
                    let cursor_column = doc.cursor_column;
                    let search_result = self.search_state.search_document(doc);
                    
                    if let Err(e) = search_result {
                        self.status_message = e.to_string();
                    } else {
                        // Find next match
                        if let Some(search_match) = self.search_state.find_next_match(cursor_line, cursor_column) {
                            let line = search_match.line;
                            let column = search_match.start_col;
                            let current_index = self.search_state.current_match_index().unwrap_or(0);
                            let pattern = self.search_state.pattern.clone();
                            
                            // Update cursor position
                            self.buffer_manager.current_document_mut().cursor_line = line;
                            self.buffer_manager.current_document_mut().cursor_column = column;
                            self.status_message = format!("/{pattern} [{current_index}]");
                        } else {
                            self.status_message = format!("Pattern not found: {pattern}");
                        }
                    }
                }
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            Command::SearchBackward(pattern) => {
                if let Err(e) = self.search_state.set_pattern(pattern.clone(), SearchDirection::Backward) {
                    self.status_message = e.to_string();
                } else {
                    // Use split borrowing to access buffer_manager and search_state separately
                    let doc = self.buffer_manager.current_document();
                    let cursor_line = doc.cursor_line;
                    let cursor_column = doc.cursor_column;
                    let search_result = self.search_state.search_document(doc);
                    
                    if let Err(e) = search_result {
                        self.status_message = e.to_string();
                    } else if let Some(search_match) = self.search_state.find_next_match(cursor_line, cursor_column) {
                        let line = search_match.line;
                        let column = search_match.start_col;
                        let current_index = self.search_state.current_match_index().unwrap_or(0);
                        let pattern = self.search_state.pattern.clone();
                        
                        self.buffer_manager.current_document_mut().cursor_line = line;
                        self.buffer_manager.current_document_mut().cursor_column = column;
                        self.status_message = format!("?{pattern} [{current_index}]");
                    } else {
                        self.status_message = format!("Pattern not found: {pattern}");
                    }
                }
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            Command::SearchNext => {
                if self.search_state.pattern.is_empty() {
                    self.status_message = "No previous search pattern".to_string();
                } else {
                    let doc = self.buffer_manager.current_document();
                    let cursor_line = doc.cursor_line;
                    let cursor_column = doc.cursor_column;
                    if let Some(search_match) = self.search_state.repeat_last_search(cursor_line, cursor_column) {
                        let line = search_match.line;
                        let column = search_match.start_col;
                        let current_index = self.search_state.current_match_index().unwrap_or(0);
                        let pattern = self.search_state.pattern.clone();
                        
                        self.buffer_manager.current_document_mut().cursor_line = line;
                        self.buffer_manager.current_document_mut().cursor_column = column;
                        self.status_message = format!("{pattern} [{current_index}]");
                    } else {
                        let pattern = self.search_state.pattern.clone();
                        self.status_message = format!("Pattern not found: {pattern}");
                    }
                }
            }
            Command::SearchPrevious => {
                if self.search_state.pattern.is_empty() {
                    self.status_message = "No previous search pattern".to_string();
                } else {
                    let doc = self.buffer_manager.current_document();
                    let cursor_line = doc.cursor_line;
                    let cursor_column = doc.cursor_column;
                    if let Some(search_match) = self.search_state.repeat_last_search_reverse(cursor_line, cursor_column) {
                        let line = search_match.line;
                        let column = search_match.start_col;
                        let current_index = self.search_state.current_match_index().unwrap_or(0);
                        let pattern = self.search_state.pattern.clone();
                        
                        self.buffer_manager.current_document_mut().cursor_line = line;
                        self.buffer_manager.current_document_mut().cursor_column = column;
                        self.status_message = format!("{pattern} [{current_index}]");
                    } else {
                        let pattern = self.search_state.pattern.clone();
                        self.status_message = format!("Pattern not found: {pattern}");
                    }
                }
            }
            Command::SearchWordUnderCursor => {
                self.search_word_under_cursor(SearchDirection::Forward);
            }
            Command::SearchWordUnderCursorBackward => {
                self.search_word_under_cursor(SearchDirection::Backward);
            }
            Command::ExitSearchMode => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }

            _ => {} // Should not reach here
        }
    }

    fn execute_insert_command(&mut self, command: Command) {
        match command {
            Command::EnterInsertMode => {
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.mode = Mode::Insert;
            }
            Command::EnterInsertModeAfter => {
                self.current_document_mut().move_cursor_right();
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.mode = Mode::Insert;
            }
            Command::EnterInsertModeNewLine => {
                self.current_document_mut().open_line_below();
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.mode = Mode::Insert;
            }
            Command::EnterInsertModeNewLineAbove => {
                self.current_document_mut().open_line_above();
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.mode = Mode::Insert;
            }
            Command::EnterInsertModeLineEnd => {
                self.current_document_mut().move_line_end();
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.mode = Mode::Insert;
            }
            Command::EnterInsertModeLineStart => {
                self.current_document_mut().move_first_non_whitespace();
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.mode = Mode::Insert;
            }
            Command::ExitInsertMode => {
                // End undo group when leaving insert mode
                let cursor_pos = {
                    let doc = self.current_document();
                    (doc.cursor_line, doc.cursor_column)
                };
                self.current_document_mut().undo_manager.end_group(cursor_pos);
                
                // Mark last insert position when leaving insert mode
                let cursor_pos = {
                    let doc = self.current_document();
                    (doc.cursor_line, doc.cursor_column)
                };
                self.mark_manager.set_last_insert(cursor_pos.0, cursor_pos.1);
                self.mode = Mode::Normal;
            }
            Command::InsertChar(c) => {
                self.current_document_mut().insert_char(c);
                // Mark change position
                let doc = self.current_document();
                self.mark_manager.set_last_change(doc.cursor_line, doc.cursor_column);
            }
            Command::InsertNewline => {
                self.current_document_mut().insert_newline();
                // Mark change position
                let doc = self.current_document();
                self.mark_manager.set_last_change(doc.cursor_line, doc.cursor_column);
            }
            Command::InsertTab => {
                let tab_width = self.view.get_tab_stop();
                self.current_document_mut().insert_tab_or_spaces(tab_width);
            }

            _ => {} // Should not reach here
        }
    }

    fn execute_edit_command(&mut self, command: Command) {
        match command {
            Command::DeleteChar => {
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.current_document_mut().delete_char();
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.end_group(cursor_pos);
            }
            Command::DeleteCharForward => {
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.current_document_mut().delete_char_forward();
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.end_group(cursor_pos);
            }
            Command::DeleteCharBackward => {
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.start_group(cursor_pos);
                self.current_document_mut().delete_char_backward();
                let doc = self.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                self.current_document_mut().undo_manager.end_group(cursor_pos);
            }
            Command::DeleteLine => {
                self.current_document_mut().delete_line();
            }
            Command::DeleteLines(count) => {
                for _ in 0..count {
                    if self.current_document().lines.len() > 1 {
                        self.current_document_mut().delete_line();
                        // Adjust cursor if we deleted the last line
                        if self.current_document().cursor_line >= self.current_document().lines.len() {
                            self.current_document_mut().cursor_line = self.current_document().lines.len() - 1;
                        }
                    } else {
                        break;
                    }
                }
            }
            Command::DeleteToEndOfLine => {
                self.current_document_mut().delete_to_end_of_line();
            }
            Command::DeleteWord => {
                self.current_document_mut().delete_word_forward();
            }
            Command::DeleteBigWord => {
                self.current_document_mut().delete_big_word_forward();
            }
            Command::DeleteWordBackward => {
                self.current_document_mut().delete_word_backward();
            }
            Command::DeleteBigWordBackward => {
                self.current_document_mut().delete_big_word_backward();
            }
            Command::DeleteToEndOfWord => {
                self.current_document_mut().delete_to_end_of_word();
            }
            Command::DeleteToEndOfBigWord => {
                self.current_document_mut().delete_to_end_of_big_word();
            }
            Command::DeleteToStartOfLine => {
                self.current_document_mut().delete_to_start_of_line();
            }
            Command::DeleteToFirstNonWhitespace => {
                self.current_document_mut().delete_to_first_non_whitespace();
            }
            Command::DeleteToEndOfFile => {
                self.current_document_mut().delete_to_end_of_file();
            }
            Command::DeleteToStartOfFile => {
                self.current_document_mut().delete_to_start_of_file();
            }
            Command::SubstituteChar => {
                self.current_document_mut().substitute_char();
                self.mode = Mode::Insert;
            }
            Command::SubstituteLine => {
                self.current_document_mut().substitute_line();
                self.mode = Mode::Insert;
            }
            Command::DeleteUntilChar(target) => {
                self.current_document_mut().delete_until_char(target);
            }
            Command::DeleteUntilCharBackward(target) => {
                self.current_document_mut().delete_until_char_backward(target);
            }
            Command::DeleteFindChar(target) => {
                self.current_document_mut().delete_find_char(target);
            }
            Command::DeleteFindCharBackward(target) => {
                self.current_document_mut().delete_find_char_backward(target);
            }

            _ => {} // Should not reach here
        }
    }

    fn execute_change_command(&mut self, command: Command) {
        // Store the deleted text and enter insert mode
        let _deleted_text = match command {
            Command::ChangeLine => {
                let deleted = self.current_document_mut().change_line();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeLines(count) => {
                let mut deleted_lines = Vec::new();
                for _ in 0..count {
                    if !self.current_document().lines.is_empty() {
                        deleted_lines.push(self.current_document_mut().change_line());
                        // Adjust cursor if we're at the end
                        if self.current_document().cursor_line >= self.current_document().lines.len() {
                            self.current_document_mut().cursor_line = self.current_document().lines.len().saturating_sub(1);
                        }
                    } else {
                        break;
                    }
                }
                self.mode = Mode::Insert;
                deleted_lines.join("\n")
            }
            Command::ChangeToEndOfLine => {
                let deleted = self.current_document_mut().change_to_end_of_line();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeWord => {
                let deleted = self.current_document_mut().change_word_forward();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeBigWord => {
                let deleted = self.current_document_mut().change_big_word_forward();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeWordBackward => {
                let deleted = self.current_document_mut().change_word_backward();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeBigWordBackward => {
                let deleted = self.current_document_mut().change_big_word_backward();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeToEndOfWord => {
                let deleted = self.current_document_mut().change_to_end_of_word();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeToEndOfBigWord => {
                let deleted = self.current_document_mut().change_to_end_of_big_word();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeToStartOfLine => {
                let deleted = self.current_document_mut().change_to_start_of_line();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeToFirstNonWhitespace => {
                let deleted = self.current_document_mut().change_to_first_non_whitespace();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeToEndOfFile => {
                let deleted = self.current_document_mut().change_to_end_of_file();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeToStartOfFile => {
                let deleted = self.current_document_mut().change_to_start_of_file();
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeUntilChar(target) => {
                let deleted = self.current_document_mut().change_until_char(target);
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeUntilCharBackward(target) => {
                let deleted = self.current_document_mut().change_until_char_backward(target);
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeFindChar(target) => {
                let deleted = self.current_document_mut().change_find_char(target);
                self.mode = Mode::Insert;
                deleted
            }
            Command::ChangeFindCharBackward(target) => {
                let deleted = self.current_document_mut().change_find_char_backward(target);
                self.mode = Mode::Insert;
                deleted
            }
            _ => String::new(), // Should not reach here
        };

        // Store deleted text in unnamed register (for potential later use with undo/redo)
        self.register_manager.store_in_register(
            None, 
            _deleted_text, 
            crate::registers::RegisterType::Character
        );
    }

    fn execute_visual_command(&mut self, command: Command) {
        match command {
            Command::EnterVisualChar => {
                let doc = self.current_document();
                let selection = Selection::new(doc.cursor_line, doc.cursor_column, VisualMode::Char);
                self.visual_selection = Some(selection);
                self.mode = Mode::VisualChar;
            }
            Command::EnterVisualLine => {
                let doc = self.current_document();
                let selection = Selection::new(doc.cursor_line, doc.cursor_column, VisualMode::Line);
                self.visual_selection = Some(selection);
                self.mode = Mode::VisualLine;
            }
            Command::EnterVisualBlock => {
                let doc = self.current_document();
                let selection = Selection::new(doc.cursor_line, doc.cursor_column, VisualMode::Block);
                self.visual_selection = Some(selection);
                self.mode = Mode::VisualBlock;
            }
            Command::ExitVisualMode => {
                self.visual_selection = None;
                self.mode = Mode::Normal;
            }
            Command::VisualDelete => {
                if let Some(selection) = self.visual_selection.take() {
                    VisualModeHandler::delete_selection(&selection, self.current_document_mut());
                    self.mode = Mode::Normal;
                }
            }
            Command::VisualIndent => {
                if let Some(selection) = self.visual_selection.take() {
                    let tab_width = self.view.get_tab_stop();
                    let use_spaces = self.current_document().expand_tab;
                    VisualModeHandler::indent_selection(&selection, self.current_document_mut(), tab_width, use_spaces);
                    self.mode = Mode::Normal;
                }
            }
            Command::VisualDedent => {
                if let Some(selection) = self.visual_selection.take() {
                    let tab_width = self.view.get_tab_stop();
                    VisualModeHandler::dedent_selection(&selection, self.current_document_mut(), tab_width);
                    self.mode = Mode::Normal;
                }
            }
            Command::VisualYank => {
                crate::yank_paste_handler::YankPasteHandler::execute_visual_yank(self, None);
            }

            _ => {} // Should not reach here
        }
    }


    fn execute_indentation_command(&mut self, command: Command) {
        match command {
            Command::IndentLine => {
                let tab_width = self.view.get_tab_stop();
                let use_spaces = self.current_document().expand_tab;
                self.current_document_mut().indent_line(tab_width, use_spaces);
            }
            Command::IndentLines(count) => {
                let tab_width = self.view.get_tab_stop();
                let use_spaces = self.current_document().expand_tab;
                let start_line = self.current_document().cursor_line;
                self.current_document_mut().indent_lines(start_line, count, tab_width, use_spaces);
            }
            Command::DedentLine => {
                let tab_width = self.view.get_tab_stop();
                self.current_document_mut().dedent_line(tab_width);
            }
            Command::DedentLines(count) => {
                let tab_width = self.view.get_tab_stop();
                let start_line = self.current_document().cursor_line;
                self.current_document_mut().dedent_lines(start_line, count, tab_width);
            }

            _ => {} // Should not reach here
        }
    }

    fn execute_undo_redo_command(&mut self, command: Command) {
        match command {
            Command::Undo => {
                if let Some(undo_group) = self.current_document_mut().undo_manager.undo() {
                    self.apply_undo_group(&undo_group, true);
                    self.status_message = "Undo completed".to_string();
                } else {
                    self.status_message = "Nothing to undo".to_string();
                }
            }
            Command::Redo => {
                if let Some(redo_group) = self.current_document_mut().undo_manager.redo() {
                    self.apply_undo_group(&redo_group, false);
                    self.status_message = "Redo completed".to_string();
                } else {
                    self.status_message = "Nothing to redo".to_string();
                }
            }
            _ => {} // Should not reach here
        }
    }

    fn execute_join_lines_command(&mut self) {
        let doc = self.current_document_mut();
        if doc.join_lines() {
            self.status_message = "Lines joined".to_string();
        } else {
            self.status_message = "Cannot join: at last line".to_string();
        }
    }

    fn move_to_screen_top(&mut self) {
        let scroll_offset = self.view.get_scroll_offset();
        self.current_document_mut().move_to_line(scroll_offset);
        self.update_visual_selection();
    }

    fn move_to_screen_middle(&mut self) {
        let scroll_offset = self.view.get_scroll_offset();
        let visible_lines = self.view.get_visible_lines_count();
        let middle_line = scroll_offset + visible_lines / 2;
        self.current_document_mut().move_to_line(middle_line);
        self.update_visual_selection();
    }

    fn move_to_screen_bottom(&mut self) {
        let scroll_offset = self.view.get_scroll_offset();
        let visible_lines = self.view.get_visible_lines_count();
        let bottom_line = scroll_offset + visible_lines.saturating_sub(1);
        self.current_document_mut().move_to_line(bottom_line);
        self.update_visual_selection();
    }

    fn match_bracket(&mut self) {
        if let Some((target_line, target_column)) = self.current_document().find_matching_bracket() {
            self.current_document_mut().cursor_line = target_line;
            self.current_document_mut().cursor_column = target_column;
            self.update_visual_selection();
            self.status_message = "Bracket matched".to_string();
        } else {
            self.status_message = "No matching bracket found".to_string();
        }
    }

    fn search_word_under_cursor(&mut self, direction: SearchDirection) {
        // Get word under cursor and store cursor position
        let doc = self.buffer_manager.current_document();
        let word = if let Some(word) = doc.get_word_under_cursor() {
            word
        } else {
            self.status_message = "No word under cursor".to_string();
            return;
        };
        let cursor_line = doc.cursor_line;
        let cursor_column = doc.cursor_column;
        
        // Escape special regex characters in the word to search for literal text
        let escaped_word = regex::escape(&word);
        let is_forward = matches!(direction, SearchDirection::Forward);
        
        // Set up the search pattern
        if let Err(e) = self.search_state.set_pattern(escaped_word.clone(), direction) {
            self.status_message = e.to_string();
            return;
        }
        
        // Perform the search
        let doc = self.buffer_manager.current_document();
        if let Err(e) = self.search_state.search_document(doc) {
            self.status_message = e.to_string();
            return;
        }
        
        // Find next/previous match based on direction
        let search_match = if is_forward {
            self.search_state.find_next_match(cursor_line, cursor_column)
        } else {
            self.search_state.find_prev_match(cursor_line, cursor_column)
        };
        
        if let Some(search_match) = search_match {
            let line = search_match.line;
            let column = search_match.start_col;
            let current_index = self.search_state.current_match_index().unwrap_or(0);
            
            // Update cursor position
            self.current_document_mut().cursor_line = line;
            self.current_document_mut().cursor_column = column;
            
            let direction_char = if is_forward { '*' } else { '#' };
            self.status_message = format!("{direction_char}{word} [{current_index}]");
        } else {
            self.status_message = format!("Pattern not found: {word}");
        }
    }

    fn execute_case_command(&mut self, command: Command) {
        let doc = self.current_document_mut();
        match command {
            Command::ToggleCase => {
                if doc.toggle_case_char() {
                    self.status_message = "Case toggled".to_string();
                } else {
                    self.status_message = "No character to toggle".to_string();
                }
            }
            Command::Lowercase => {
                doc.lowercase_line();
                self.status_message = "Line converted to lowercase".to_string();
            }
            Command::Uppercase => {
                doc.uppercase_line();
                self.status_message = "Line converted to uppercase".to_string();
            }
            _ => {}
        }
    }

    fn apply_undo_group(&mut self, group: &UndoGroup, is_undo: bool) {
        if is_undo {
            // For undo, apply actions in reverse order and reverse each action
            for action in group.actions.iter().rev() {
                self.apply_undo_action(&action.reverse());
            }
            // Set cursor to the position before the group
            let doc = self.current_document_mut();
            doc.cursor_line = group.cursor_before.0;
            doc.cursor_column = group.cursor_before.1;
            doc.clamp_cursor_column();
        } else {
            // For redo, apply actions in original order
            for action in &group.actions {
                self.apply_undo_action(action);
            }
            // Set cursor to the position after the group
            let doc = self.current_document_mut();
            doc.cursor_line = group.cursor_after.0;
            doc.cursor_column = group.cursor_after.1;
            doc.clamp_cursor_column();
        }
    }

    fn apply_undo_action(&mut self, action: &UndoAction) {
        let doc = self.current_document_mut();
        
        match action {
            UndoAction::InsertText { line, column, text } => {
                if *line < doc.lines.len() {
                    let line_text = &mut doc.lines[*line];
                    if *column <= line_text.len() {
                        line_text.insert_str(*column, text);
                    }
                }
            }
            UndoAction::DeleteText { line, column, text } => {
                if *line < doc.lines.len() {
                    let line_text = &mut doc.lines[*line];
                    let end_pos = (*column + text.len()).min(line_text.len());
                    if *column < line_text.len() {
                        line_text.drain(*column..end_pos);
                    }
                }
            }
            UndoAction::InsertLine { line, text } => {
                if *line <= doc.lines.len() {
                    doc.lines.insert(*line, text.clone());
                }
            }
            UndoAction::DeleteLine { line, text: _ } => {
                if *line < doc.lines.len() {
                    doc.lines.remove(*line);
                }
            }
            UndoAction::SplitLine { line, column, text } => {
                if *line < doc.lines.len() {
                    let line_text = &mut doc.lines[*line];
                    if *column <= line_text.len() {
                        let remaining = line_text.split_off(*column);
                        doc.lines.insert(*line + 1, text.clone() + &remaining);
                    }
                }
            }
            UndoAction::JoinLines { line, separator, second_line_text } => {
                if *line < doc.lines.len() && *line + 1 < doc.lines.len() {
                    doc.lines.remove(*line + 1);
                    doc.lines[*line].push_str(separator);
                    doc.lines[*line].push_str(second_line_text);
                }
            }
        }
        
        doc.modified = true;
    }

    fn execute_other_command(&mut self, command: Command) -> bool {
        match command {
            Command::EnterCommandMode => {
                if self.mode == Mode::Command {
                    // Exit command mode
                    self.mode = Mode::Normal;
                    self.command_buffer.clear();
                } else {
                    // Enter command mode
                    self.mode = Mode::Command;
                    self.command_buffer.clear();
                }
                false
            }
            Command::Execute(cmd) => {
                let should_quit = self.execute_vim_command(&cmd);
                self.mode = Mode::Normal;
                self.command_buffer.clear();
                should_quit
            }
            Command::Redraw => {
                self.view.force_redraw();
                false
            }

            _ => false
        }
    }

    fn execute_vim_command(&mut self, cmd: &str) -> bool {
        self.status_message.clear();

        match cmd {
            "q" => !self.current_document().is_modified(),
            "q!" => true,
            "w" => self.handle_save_command(),
            "wq" => self.handle_save_and_quit_command(),
            "f" | "file" => self.handle_file_info_command(),
            _ if cmd.starts_with("w ") => self.handle_save_as_command(cmd),
            _ if cmd.starts_with("set ") => self.handle_set_command(cmd),
            _ if cmd.starts_with("r ") => self.handle_read_command(cmd),
            _ if cmd.starts_with("0r ") => self.handle_read_at_line_command(cmd, 0),
            _ if cmd.starts_with("$r ") => self.handle_read_at_end_command(cmd),
            "paste" => {
                self.handle_paste_command();
                false
            }
            "help" | "h" | "?" => {
                self.handle_help_command();
                false
            }
            "mkvirus" => {
                self.handle_generate_rc_command();
                false
            }
            "marks" => {
                self.handle_marks_command();
                false
            }
            "jumps" | "ju" => {
                self.handle_jumps_command();
                false
            }
            "clear" => {
                self.view.force_redraw();
                self.status_message = "Screen cleared".to_string();
                false
            }
            _ if cmd.starts_with("clear ") => {
                self.handle_clear_command(cmd);
                false
            }
            "redraw" => {
                self.view.force_redraw();
                self.status_message = "Screen redrawn".to_string();
                false
            }
            "detab" => {
                let tab_width = self.view.get_tab_stop();
                let changed = self.current_document_mut().tabs_to_spaces(tab_width);
                self.status_message = format!("Converted {changed} lines: tabs → spaces");
                false
            }
            "retab" => {
                let tab_width = self.view.get_tab_stop();
                let changed = self.current_document_mut().spaces_to_tabs(tab_width);
                self.status_message = format!("Converted {changed} lines: spaces → tabs");
                false
            }
            "ascii" => {
                self.handle_ascii_command();
                false
            }
            "ls" => {
                self.handle_list_buffers_command();
                false
            }
            "bn" => {
                self.handle_next_buffer_command();
                false
            }
            "bp" => {
                self.handle_prev_buffer_command();
                false
            }
            "bd" => {
                self.handle_close_buffer_command();
                false
            }
            "bd!" => {
                self.handle_force_close_buffer_command();
                false
            }
            "e" => {
                self.handle_new_buffer_command();
                false
            }
            _ if cmd.starts_with("e ") => {
                self.handle_edit_command(cmd);
                false
            }
            _ if cmd.starts_with("b") && cmd.len() > 1 => {
                self.handle_buffer_switch_command(cmd);
                false
            }
            _ if cmd.starts_with("s/") => {
                self.handle_substitute_command(cmd);
                false
            }
            _ if cmd == "%s" || cmd.starts_with("%s/") => {
                self.handle_substitute_all_command(cmd);
                false
            }
            _ => {
                // Check if it's a pure number (goto line)
                if let Ok(line_num) = cmd.parse::<usize>() {
                    self.current_document_mut().move_to_line(line_num);
                    self.status_message = format!("Line {line_num}");
                    false
                } else {
                    self.handle_numbered_read_command(cmd)
                }
            }
        }
    }

    /// Handle the :ascii command - normalize Unicode characters to ASCII equivalents
    fn handle_ascii_command(&mut self) {
        let changed = self.current_document_mut().ascii_normalize();
        if changed > 0 {
            self.status_message = format!("Normalized {changed} lines to ASCII");
        } else {
            self.status_message = "No Unicode characters found to normalize".to_string();
        }
    }

    pub fn get_display_filename(&self) -> &str {
        self.buffer_manager.get_display_filename()
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Enter alternate screen buffer (like vim does)
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        // Create a guard to ensure cleanup happens even on panic
        struct TerminalGuard;
        impl Drop for TerminalGuard {
            fn drop(&mut self) {
                let _ = disable_raw_mode();
                let _ = execute!(stdout(), LeaveAlternateScreen);
            }
        }
        let _guard = TerminalGuard;

        // Run the main loop
        self.run_loop()
    }

    fn run_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let buffer_info = format!(
                "Buffer {}/{}: \"{}\"",
                self.buffer_manager.current_buffer_index() + 1,
                self.buffer_manager.buffer_count(),
                self.get_display_filename()
            );
            
            // Borrow fields separately to avoid borrowing conflicts
            let doc = self.buffer_manager.current_document();
            
            // Calculate matching bracket position for highlighting
            let matching_bracket = doc.find_matching_bracket();
            
            let params = RenderParams {
                mode: &self.mode,
                command_buffer: &self.command_buffer,
                status_message: &self.status_message,
                buffer_info: Some(&buffer_info),
                visual_selection: self.visual_selection.as_ref(),
                search_state: Some(&self.search_state),
                matching_bracket,
            };
self.view.render(doc, &params)?;

            match event::read()? {
                Event::Key(key_event) => {
                    // Handle command mode character input directly
                    if self.mode == Mode::Command {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                self.command_buffer.push(c);
                                continue;
                            }
                            KeyCode::Backspace => {
                                self.command_buffer.pop();
                                continue;
                            }
                            KeyCode::Enter => {
                                // Execute the command
                                let cmd = self.command_buffer.clone(); // This clone is necessary for ownership
                                if self.execute_command(Command::Execute(cmd)) {
                                    break; // Quit
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }

                    // Handle search mode character input directly
                    if self.mode == Mode::Search {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                self.command_buffer.push(c);
                                continue;
                            }
                            KeyCode::Backspace => {
                                self.command_buffer.pop();
                                continue;
                            }
                            KeyCode::Enter => {
                                // Execute the search
                                let pattern = self.command_buffer.clone(); // This clone is necessary for ownership
                                if self.execute_command(Command::SearchForward(pattern)) {
                                    break; // Quit
                                }
                                continue;
                            }
                            KeyCode::Esc => {
                                if self.execute_command(Command::ExitSearchMode) {
                                    break; // Quit
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }

                    // Handle backward search mode character input directly
                    if self.mode == Mode::SearchBackward {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                self.command_buffer.push(c);
                                continue;
                            }
                            KeyCode::Backspace => {
                                self.command_buffer.pop();
                                continue;
                            }
                            KeyCode::Enter => {
                                // Execute the backward search
                                let pattern = self.command_buffer.clone(); // This clone is necessary for ownership
                                if self.execute_command(Command::SearchBackward(pattern)) {
                                    break; // Quit
                                }
                                continue;
                            }
                            KeyCode::Esc => {
                                if self.execute_command(Command::ExitSearchMode) {
                                    break; // Quit
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }

                    if let Some(command) = KeyHandler::parse_key_with_state(
                        &self.mode,
                        &key_event,
                        &mut self.pending_key,
                        &mut self.number_prefix,
                        &mut self.pending_register,
                    ) {
                        if self.execute_command(command) {
                            break; // Quit
                        }
                    }
                }
                Event::Resize(_, _) => {
                    // Handle terminal resize - force full redraw
                    self.view.force_redraw();
                }
                _ => {
                    // Ignore other events (mouse, etc.)
                }
            }
        }

        Ok(())
    }
}
