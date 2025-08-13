use crate::controller::shared_state::{ModeController, ModeTransition, SharedEditorState};
use crate::controller::command_types::{Mode, Command};
use crate::controller::key_handler::KeyHandler;
use crossterm::event::{KeyEvent};

// Helper macros to reduce boilerplate
macro_rules! repeat_command {
    ($doc:expr, $method:ident, $count:expr) => {
        for _ in 0..$count { 
            $doc.$method(); 
        }
    }
}

macro_rules! doc_mut {
    ($shared:expr) => { 
        $shared.session_controller.current_document_mut() 
    }
}


pub struct NormalController {
    // Normal mode specific state
    pub last_find_char: Option<char>,
    pub last_find_forward: bool,
    pub last_find_before: bool,
    pub pending_key: Option<char>,
    pub number_prefix: Option<usize>,
    pub pending_register: Option<char>,
}

impl NormalController {
    pub fn new() -> Self {
        Self {
            last_find_char: None,
            last_find_forward: true,
            last_find_before: false,
            pending_key: None,
            number_prefix: None,
            pending_register: None,
        }
    }
}

impl ModeController for NormalController {
    fn handle_key(&mut self, key_event: KeyEvent, shared: &mut SharedEditorState) -> ModeTransition {
        // Parse the key event using the existing key handler with state
        let command = KeyHandler::parse_key_with_state(
            &Mode::Normal,
            &key_event,
            &mut self.pending_key,
            &mut self.number_prefix,
            &mut self.pending_register,
        );
        
        if let Some(command) = command {
            // Take the number prefix (count) before executing the command
            let count = self.number_prefix.take().unwrap_or(1);
            
            // Handle commands that transition to other modes
            match command {
                // Mode transitions
                Command::EnterInsertMode => {
                    let doc = shared.session_controller.current_document();
                    let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                    shared.session_controller.current_document_mut()
                        .undo_manager_mut()
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeAfter => {
                    shared.session_controller.current_document_mut().move_cursor_right();
                    let doc = shared.session_controller.current_document();
                    let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                    shared.session_controller.current_document_mut()
                        .undo_manager_mut()
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeNewLine => {
                    shared.session_controller.current_document_mut().open_line_below();
                    let doc = shared.session_controller.current_document();
                    let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                    shared.session_controller.current_document_mut()
                        .undo_manager_mut()
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeNewLineAbove => {
                    shared.session_controller.current_document_mut().open_line_above();
                    let doc = shared.session_controller.current_document();
                    let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                    shared.session_controller.current_document_mut()
                        .undo_manager_mut()
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeLineEnd => {
                    shared.session_controller.current_document_mut().move_line_end();
                    let doc = shared.session_controller.current_document();
                    let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                    shared.session_controller.current_document_mut()
                        .undo_manager_mut()
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeLineStart => {
                    shared.session_controller.current_document_mut().move_first_non_whitespace();
                    let doc = shared.session_controller.current_document();
                    let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                    shared.session_controller.current_document_mut()
                        .undo_manager_mut()
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterSearchMode => {
                    return ModeTransition::ToMode(Mode::Search);
                }
                Command::EnterSearchBackwardMode => {
                    return ModeTransition::ToMode(Mode::SearchBackward);
                }
                Command::EnterCommandMode => {
                    return ModeTransition::ToMode(Mode::Command);
                }
                Command::EnterVisualChar => {
                    return ModeTransition::ToMode(Mode::VisualChar);
                }
                Command::EnterVisualLine => {
                    return ModeTransition::ToMode(Mode::VisualLine);
                }
                Command::EnterVisualBlock => {
                    return ModeTransition::ToMode(Mode::VisualBlock);
                }
                // Quit is handled by command mode (:q), not a direct key command
                
                // All other normal mode commands might change mode
                _ => {
                    return self.execute_normal_command(command, count, shared);
                }
            }
        }
        
        ModeTransition::Stay
    }
}

impl NormalController {
    fn execute_normal_command(&mut self, command: Command, count: usize, shared: &mut SharedEditorState) -> ModeTransition {
        match command {
            // Movement commands
            Command::MoveUp
            | Command::MoveDown
            | Command::MoveLeft
            | Command::MoveRight
            | Command::MoveWordForward
            | Command::MoveWordBackward
            | Command::MoveWordEnd
            | Command::MoveBigWordForward
            | Command::MoveBigWordBackward
            | Command::MoveBigWordEnd
            | Command::MoveLineStart
            | Command::MoveLineEnd
            | Command::MoveFirstNonWhitespace
            | Command::MoveDownToFirstNonWhitespace
            | Command::MoveUpToFirstNonWhitespace
            | Command::MoveDocumentStart
            | Command::MoveDocumentEnd
            | Command::MovePageUp
            | Command::MovePageDown
            | Command::MoveHalfPageUp
            | Command::MoveHalfPageDown
            | Command::MoveToLine(_)
            | Command::MoveToScreenTop
            | Command::MoveToScreenMiddle
            | Command::MoveToScreenBottom
            | Command::MatchBracket
            | Command::FindChar(_)
            | Command::FindCharBackward(_)
            | Command::FindCharBefore(_)
            | Command::FindCharBeforeBackward(_)
            | Command::RepeatFind
            | Command::RepeatFindReverse => {
                self.execute_movement_command(command, count, shared);
            }

            // Edit commands
            Command::DeleteChar
            | Command::DeleteCharForward
            | Command::DeleteCharBackward
            | Command::DeleteLine
            | Command::DeleteLines(_)
            | Command::DeleteToEndOfLine
            | Command::DeleteWord
            | Command::DeleteBigWord
            | Command::DeleteWordBackward
            | Command::DeleteBigWordBackward
            | Command::DeleteToEndOfWord
            | Command::DeleteToEndOfBigWord
            | Command::DeleteToStartOfLine
            | Command::DeleteToFirstNonWhitespace
            | Command::DeleteToEndOfFile
            | Command::DeleteToStartOfFile
            | Command::DeleteUntilChar(_)
            | Command::DeleteUntilCharBackward(_)
            | Command::DeleteFindChar(_)
            | Command::DeleteFindCharBackward(_) => {
                self.execute_edit_command(command, shared);
            }

            // Substitute commands that enter insert mode  
            Command::SubstituteChar => {
                shared.session_controller.current_document_mut().substitute_char();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::SubstituteLine => {
                shared.session_controller.current_document_mut().substitute_line();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }

            // Change commands (delete + enter insert mode)
            Command::ChangeLine => {
                let _deleted = shared.session_controller.current_document_mut().change_line();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeLines(count) => {
                let mut deleted_lines = Vec::new();
                for _ in 0..count {
                    let is_empty = shared.session_controller.current_document().line_count() == 0;
                    if !is_empty {
                        deleted_lines.push(shared.session_controller.current_document_mut().change_line());
                        // Adjust cursor if we're at the end
                        let line_count = shared.session_controller.current_document().line_count();
                        if shared.session_controller.current_document().cursor_line() >= line_count {
                            let current_column = shared.session_controller.current_document().cursor_column();
                            let _ = shared.session_controller.current_document_mut().set_cursor(line_count.saturating_sub(1), current_column);
                        }
                    } else {
                        break;
                    }
                }
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToEndOfLine => {
                let _deleted = shared.session_controller.current_document_mut().change_to_end_of_line();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeWord => {
                let _deleted = shared.session_controller.current_document_mut().change_word_forward();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeBigWord => {
                let _deleted = shared.session_controller.current_document_mut().change_big_word_forward();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeWordBackward => {
                let _deleted = shared.session_controller.current_document_mut().change_word_backward();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeBigWordBackward => {
                let _deleted = shared.session_controller.current_document_mut().change_big_word_backward();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToEndOfWord => {
                let _deleted = shared.session_controller.current_document_mut().change_to_end_of_word();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToEndOfBigWord => {
                let _deleted = shared.session_controller.current_document_mut().change_to_end_of_big_word();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToStartOfLine => {
                let _deleted = shared.session_controller.current_document_mut().change_to_start_of_line();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToFirstNonWhitespace => {
                let _deleted = shared.session_controller.current_document_mut().change_to_first_non_whitespace();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToEndOfFile => {
                let _deleted = shared.session_controller.current_document_mut().change_to_end_of_file();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToStartOfFile => {
                let _deleted = shared.session_controller.current_document_mut().change_to_start_of_file();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeUntilChar(target) => {
                let _deleted = shared.session_controller.current_document_mut().change_until_char(target);
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeUntilCharBackward(target) => {
                let _deleted = shared.session_controller.current_document_mut().change_until_char_backward(target);
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeFindChar(target) => {
                let _deleted = shared.session_controller.current_document_mut().change_find_char(target);
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeFindCharBackward(target) => {
                let _deleted = shared.session_controller.current_document_mut().change_find_char_backward(target);
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }

            // Mark commands
            Command::SetMark(_)
            | Command::JumpToMark(_)
            | Command::JumpToMarkLine(_)
            | Command::JumpBackward
            | Command::JumpForward => {
                self.execute_mark_command(command, shared);
            }

            // Search commands
            Command::SearchNext
            | Command::SearchPrevious
            | Command::SearchWordUnderCursor
            | Command::SearchWordUnderCursorBackward => {
                self.execute_search_command(command, shared);
            }

            // Yank and paste commands
            Command::Yank(yank_type, register) => {
                shared.session_controller.yank_text(yank_type, register, &mut shared.register_manager, &mut shared.status_message);
            }
            Command::Paste(paste_type, register) => {
                shared.session_controller.paste_text(paste_type, register, &mut shared.register_manager, &mut shared.status_message);
            }

            // Indentation commands
            Command::IndentLine
            | Command::IndentLines(_)
            | Command::DedentLine
            | Command::DedentLines(_) => {
                self.execute_indentation_command(command, shared);
            }

            // Line operations
            Command::JoinLines => {
                self.execute_join_lines_command(shared);
            }

            // Case operations
            Command::ToggleCase | Command::Lowercase | Command::Uppercase => {
                self.execute_case_command(command, shared);
            }

            // Undo/Redo commands
            Command::Undo | Command::Redo => {
                self.execute_undo_redo_command(command, shared);
            }

            // Command mode
            Command::Redraw => {
                // Just clear and redraw - no specific action needed
                shared.status_message.clear();
            }

            _ => {
                shared.status_message = format!("Unhandled normal mode command: {:?}", command);
            }
        }
        
        ModeTransition::Stay
    }

    fn execute_movement_command(&mut self, command: Command, count: usize, shared: &mut SharedEditorState) {
        let doc = doc_mut!(shared);
        match command {
            // Basic movement
            Command::MoveUp => repeat_command!(doc, move_cursor_up, count),
            Command::MoveDown => repeat_command!(doc, move_cursor_down, count),
            Command::MoveLeft => repeat_command!(doc, move_cursor_left, count),
            Command::MoveRight => repeat_command!(doc, move_cursor_right, count),

            // Word movement
            Command::MoveWordForward => repeat_command!(doc, move_word_forward, count),
            Command::MoveWordBackward => repeat_command!(doc, move_word_backward, count),
            Command::MoveWordEnd => repeat_command!(doc, move_word_end, count),
            Command::MoveBigWordForward => repeat_command!(doc, move_big_word_forward, count),
            Command::MoveBigWordBackward => repeat_command!(doc, move_big_word_backward, count),
            Command::MoveBigWordEnd => repeat_command!(doc, move_big_word_end, count),

            // Line movement
            Command::MoveLineStart => doc.move_line_start(),
            Command::MoveLineEnd => doc.move_line_end(),
            Command::MoveFirstNonWhitespace => doc.move_first_non_whitespace(),
            Command::MoveDownToFirstNonWhitespace => repeat_command!(doc, move_down_to_first_non_whitespace, count),
            Command::MoveUpToFirstNonWhitespace => repeat_command!(doc, move_up_to_first_non_whitespace, count),

            // Document movement (special handling for jump list)
            Command::MoveDocumentStart => {
                let current_doc = shared.session_controller.current_document();
                shared.mark_manager.add_to_jump_list(current_doc.cursor_line(), current_doc.cursor_column(), current_doc.filename.clone());
                shared.session_controller.current_document_mut().move_document_start();
            }
            Command::MoveDocumentEnd => {
                let current_doc = shared.session_controller.current_document();
                shared.mark_manager.add_to_jump_list(current_doc.cursor_line(), current_doc.cursor_column(), current_doc.filename.clone());
                shared.session_controller.current_document_mut().move_document_end();
            }
            Command::MovePageUp => repeat_command!(doc, move_page_up, count),
            Command::MovePageDown => repeat_command!(doc, move_page_down, count),
            Command::MoveHalfPageUp => repeat_command!(doc, move_half_page_up, count),
            Command::MoveHalfPageDown => repeat_command!(doc, move_half_page_down, count),

            // Line jumping
            Command::MoveToLine(line) => doc.move_to_line(line),

            // Character search
            Command::FindChar(c) => {
                self.last_find_char = Some(c);
                self.last_find_forward = true;
                self.last_find_before = false;
                for _ in 0..count {
                    doc.find_char(c, true, false);
                }
            }
            Command::FindCharBackward(c) => {
                self.last_find_char = Some(c);
                self.last_find_forward = false;
                self.last_find_before = false;
                for _ in 0..count {
                    doc.find_char(c, false, false);
                }
            }
            Command::FindCharBefore(c) => {
                self.last_find_char = Some(c);
                self.last_find_forward = true;
                self.last_find_before = true;
                for _ in 0..count {
                    shared.session_controller.current_document_mut().find_char(c, true, true);
                }
            }
            Command::FindCharBeforeBackward(c) => {
                self.last_find_char = Some(c);
                self.last_find_forward = false;
                self.last_find_before = true;
                for _ in 0..count {
                    shared.session_controller.current_document_mut().find_char(c, false, true);
                }
            }
            Command::RepeatFind => {
                if let Some(c) = self.last_find_char {
                    for _ in 0..count {
                        shared.session_controller.current_document_mut().find_char(c, self.last_find_forward, self.last_find_before);
                    }
                }
            }
            Command::RepeatFindReverse => {
                if let Some(c) = self.last_find_char {
                    for _ in 0..count {
                        shared.session_controller.current_document_mut().find_char(c, !self.last_find_forward, self.last_find_before);
                    }
                }
            }

            // Bracket matching
            Command::MatchBracket => {
                if let Some((target_line, target_column)) = shared.session_controller.current_document().find_matching_bracket() {
                    let _ = shared.session_controller.current_document_mut().set_cursor(target_line, target_column);
                    shared.status_message = "Bracket matched".to_string();
                } else {
                    shared.status_message = "No matching bracket found".to_string();
                }
            }

            // Screen positioning
            Command::MoveToScreenTop => {
                // H - Move to top of screen
                let top_line = shared.view.get_scroll_offset();
                let _ = shared.session_controller.current_document_mut().set_cursor(top_line, 0);
                shared.session_controller.current_document_mut().move_first_non_whitespace();
            }
            Command::MoveToScreenMiddle => {
                // M - Move to middle of screen
                let scroll_offset = shared.view.get_scroll_offset();
                let visible_lines = shared.view.get_visible_lines_count();
                let middle_line = scroll_offset + (visible_lines / 2);
                let doc = shared.session_controller.current_document_mut();
                let max_line = doc.line_count().saturating_sub(1);
                let target_line = middle_line.min(max_line);
                let _ = doc.set_cursor(target_line, 0);
                doc.move_first_non_whitespace();
            }
            Command::MoveToScreenBottom => {
                // L - Move to bottom of screen
                let scroll_offset = shared.view.get_scroll_offset();
                let visible_lines = shared.view.get_visible_lines_count();
                let bottom_line = scroll_offset + visible_lines.saturating_sub(1);
                let doc = shared.session_controller.current_document_mut();
                let max_line = doc.line_count().saturating_sub(1);
                let target_line = bottom_line.min(max_line);
                let _ = doc.set_cursor(target_line, 0);
                doc.move_first_non_whitespace();
            }

            _ => {} // Should not reach here
        }
    }

    fn execute_edit_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        match command {
            Command::DeleteChar => {
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                shared.session_controller.current_document_mut().delete_char();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .end_group(cursor_pos);
            }
            Command::DeleteCharForward => {
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                shared.session_controller.current_document_mut().delete_char_forward();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .end_group(cursor_pos);
            }
            Command::DeleteCharBackward => {
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .start_group(cursor_pos);
                shared.session_controller.current_document_mut().delete_char_backward();
                let doc = shared.session_controller.current_document();
                let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                shared.session_controller.current_document_mut()
                    .undo_manager_mut()
                    .end_group(cursor_pos);
            }
            Command::DeleteLine => {
                shared.session_controller.current_document_mut().delete_line();
            }
            Command::DeleteLines(count) => {
                for _ in 0..count {
                    let line_count = shared.session_controller.current_document().line_count();
                    if line_count > 1 {
                        shared.session_controller.current_document_mut().delete_line();
                        // Adjust cursor if we deleted the last line
                        let new_line_count = shared.session_controller.current_document().line_count();
                        if shared.session_controller.current_document().cursor_line() >= new_line_count {
                            let current_column = shared.session_controller.current_document().cursor_column();
                            let _ = shared.session_controller.current_document_mut().set_cursor(new_line_count.saturating_sub(1), current_column);
                        }
                    } else {
                        break;
                    }
                }
            }
            Command::DeleteToEndOfLine => doc_mut!(shared).delete_to_end_of_line(),
            Command::DeleteWord => doc_mut!(shared).delete_word_forward(),
            Command::DeleteBigWord => doc_mut!(shared).delete_big_word_forward(),
            Command::DeleteWordBackward => doc_mut!(shared).delete_word_backward(),
            Command::DeleteBigWordBackward => doc_mut!(shared).delete_big_word_backward(),
            Command::DeleteToEndOfWord => doc_mut!(shared).delete_to_end_of_word(),
            Command::DeleteToEndOfBigWord => doc_mut!(shared).delete_to_end_of_big_word(),
            Command::DeleteToStartOfLine => doc_mut!(shared).delete_to_start_of_line(),
            Command::DeleteToFirstNonWhitespace => doc_mut!(shared).delete_to_first_non_whitespace(),
            Command::DeleteToEndOfFile => doc_mut!(shared).delete_to_end_of_file(),
            Command::DeleteToStartOfFile => doc_mut!(shared).delete_to_start_of_file(),
            Command::DeleteUntilChar(target) => doc_mut!(shared).delete_until_char(target),
            Command::DeleteUntilCharBackward(target) => doc_mut!(shared).delete_until_char_backward(target),
            Command::DeleteFindChar(target) => doc_mut!(shared).delete_find_char(target),
            Command::DeleteFindCharBackward(target) => doc_mut!(shared).delete_find_char_backward(target),

            _ => {} // Should not reach here
        }
    }

    fn execute_mark_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        match command {
            Command::SetMark(mark_char) => {
                let (line, column, filename) = {
                    let doc = shared.session_controller.current_document();
                    (doc.cursor_line(), doc.cursor_column(), doc.filename.clone())
                };
                if mark_char.is_ascii_lowercase() {
                    let _ = shared.session_controller.current_document_mut().set_local_mark(mark_char, line, column);
                } else if mark_char.is_ascii_uppercase() {
                    let _ = shared.mark_manager.set_global_mark(mark_char, line, column, filename);
                }
            }
            Command::JumpToMark(mark_char) => {
                // Add current position to jump list before jumping
                let doc = shared.session_controller.current_document();
                let current_filename = doc.filename.clone();
                shared.mark_manager.add_to_jump_list(doc.cursor_line(), doc.cursor_column(), current_filename);
                
                if mark_char.is_ascii_lowercase() {
                    if let Some((line, column)) = shared.session_controller.current_document().get_local_mark(mark_char) {
                        let _ = shared.session_controller.current_document_mut().set_cursor(line, column);
                    }
                } else if let Some(mark) = shared.mark_manager.get_global_mark(mark_char).cloned() {
                    let _ = shared.session_controller.current_document_mut().set_cursor(mark.line, mark.column);
                }
            }
            Command::JumpToMarkLine(mark_char) => {
                // Add current position to jump list before jumping
                let doc = shared.session_controller.current_document();
                let current_filename = doc.filename.clone();
                shared.mark_manager.add_to_jump_list(doc.cursor_line(), doc.cursor_column(), current_filename);
                
                if mark_char.is_ascii_lowercase() {
                    if let Some((line, _)) = shared.session_controller.current_document().get_local_mark(mark_char) {
                        let current_column = shared.session_controller.current_document().cursor_column();
                        let _ = shared.session_controller.current_document_mut().set_cursor(line, current_column);
                        shared.session_controller.current_document_mut().move_first_non_whitespace();
                    }
                } else if let Some(mark) = shared.mark_manager.get_global_mark(mark_char).cloned() {
                    let current_column = shared.session_controller.current_document().cursor_column();
                    let _ = shared.session_controller.current_document_mut().set_cursor(mark.line, current_column);
                    shared.session_controller.current_document_mut().move_first_non_whitespace();
                }
            }
            Command::JumpBackward => {
                if let Some(entry) = shared.mark_manager.jump_backward().cloned() {
                    // Update the '' (last jump) mark before jumping
                    let doc = shared.session_controller.current_document();
                    shared.mark_manager.set_last_jump(doc.cursor_line(), doc.cursor_column());
                    
                    let _ = shared.session_controller.current_document_mut().set_cursor(entry.line, entry.column);
                }
            }
            Command::JumpForward => {
                if let Some(entry) = shared.mark_manager.jump_forward().cloned() {
                    // Update the '' (last jump) mark before jumping
                    let doc = shared.session_controller.current_document();
                    shared.mark_manager.set_last_jump(doc.cursor_line(), doc.cursor_column());
                    
                    let _ = shared.session_controller.current_document_mut().set_cursor(entry.line, entry.column);
                }
            }
            _ => {}
        }
    }

    fn execute_search_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        // Add current position to jump list for major search movements
        match command {
            Command::SearchWordUnderCursor | Command::SearchWordUnderCursorBackward => {
                let doc = shared.session_controller.current_document();
                let current_filename = doc.filename.clone();
                shared.mark_manager.add_to_jump_list(doc.cursor_line(), doc.cursor_column(), current_filename);
            }
            _ => {}
        }
        
        match command {
            Command::SearchNext => crate::controller::search_commands::SearchCommands::next(&mut shared.search_state, shared.session_controller.current_document_mut(), &mut shared.status_message),
            Command::SearchPrevious => crate::controller::search_commands::SearchCommands::previous(&mut shared.search_state, shared.session_controller.current_document_mut(), &mut shared.status_message),
            Command::SearchWordUnderCursor => crate::controller::search_commands::SearchCommands::search_word_forward(&mut shared.search_state, shared.session_controller.current_document_mut(), &mut shared.status_message),
            Command::SearchWordUnderCursorBackward => crate::controller::search_commands::SearchCommands::search_word_backward(&mut shared.search_state, shared.session_controller.current_document_mut(), &mut shared.status_message),
            _ => {}
        }
    }

    fn execute_indentation_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        shared.session_controller.execute_indent_command(command, &mut shared.status_message);
    }

    fn execute_join_lines_command(&mut self, shared: &mut SharedEditorState) {
        let doc = doc_mut!(shared);
        if doc.join_lines() {
            shared.status_message = "Lines joined".to_string();
        } else {
            shared.status_message = "Cannot join: at last line".to_string();
        }
    }

    fn execute_case_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        let doc = doc_mut!(shared);
        match command {
            Command::ToggleCase => {
                if doc.toggle_case_char() {
                    shared.status_message = "Case toggled".to_string();
                } else {
                    shared.status_message = "No character to toggle".to_string();
                }
            }
            Command::Lowercase => {
                doc.lowercase_line();
                shared.status_message = "Line converted to lowercase".to_string();
            }
            Command::Uppercase => {
                doc.uppercase_line();
                shared.status_message = "Line converted to uppercase".to_string();
            }
            _ => {}
        }
    }

    fn execute_undo_redo_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        match command {
            Command::Undo => {
                if let Some(undo_group) = shared.session_controller.current_document_mut().undo_manager_mut().undo() {
                    // Apply the reverse of the undo group to undo the changes
                    undo_group.apply_reverse_to_document(shared.session_controller.current_document_mut());
                    
                    // Show feedback with action count
                    let action_count = undo_group.actions.len();
                    if action_count == 1 {
                        shared.status_message = "1 change undone".to_string();
                    } else {
                        shared.status_message = format!("{} changes undone", action_count);
                    }
                } else {
                    shared.status_message = "Nothing to undo".to_string();
                }
            }
            Command::Redo => {
                if let Some(redo_group) = shared.session_controller.current_document_mut().undo_manager_mut().redo() {
                    // Apply the redo group to redo the changes
                    redo_group.apply_to_document(shared.session_controller.current_document_mut());
                    
                    // Show feedback with action count
                    let action_count = redo_group.actions.len();
                    if action_count == 1 {
                        shared.status_message = "1 change redone".to_string();
                    } else {
                        shared.status_message = format!("{} changes redone", action_count);
                    }
                } else {
                    shared.status_message = "Nothing to redo".to_string();
                }
            }
            _ => {} // Should not reach here
        }
    }
}