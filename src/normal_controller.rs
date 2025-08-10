use crate::mode_controllers::{ModeController, ModeTransition, SharedEditorState};
use crate::command::{Mode, Command};
use crate::key_handler::KeyHandler;
use crossterm::event::{KeyEvent};

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
                    let doc = shared.buffer_manager.current_document();
                    let cursor_pos = (doc.cursor_line, doc.cursor_column);
                    shared.buffer_manager.current_document_mut()
                        .undo_manager
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeAfter => {
                    shared.buffer_manager.current_document_mut().move_cursor_right();
                    let doc = shared.buffer_manager.current_document();
                    let cursor_pos = (doc.cursor_line, doc.cursor_column);
                    shared.buffer_manager.current_document_mut()
                        .undo_manager
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeNewLine => {
                    shared.buffer_manager.current_document_mut().open_line_below();
                    let doc = shared.buffer_manager.current_document();
                    let cursor_pos = (doc.cursor_line, doc.cursor_column);
                    shared.buffer_manager.current_document_mut()
                        .undo_manager
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeNewLineAbove => {
                    shared.buffer_manager.current_document_mut().open_line_above();
                    let doc = shared.buffer_manager.current_document();
                    let cursor_pos = (doc.cursor_line, doc.cursor_column);
                    shared.buffer_manager.current_document_mut()
                        .undo_manager
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeLineEnd => {
                    shared.buffer_manager.current_document_mut().move_line_end();
                    let doc = shared.buffer_manager.current_document();
                    let cursor_pos = (doc.cursor_line, doc.cursor_column);
                    shared.buffer_manager.current_document_mut()
                        .undo_manager
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                Command::EnterInsertModeLineStart => {
                    shared.buffer_manager.current_document_mut().move_first_non_whitespace();
                    let doc = shared.buffer_manager.current_document();
                    let cursor_pos = (doc.cursor_line, doc.cursor_column);
                    shared.buffer_manager.current_document_mut()
                        .undo_manager
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
                shared.buffer_manager.current_document_mut().substitute_char();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::SubstituteLine => {
                shared.buffer_manager.current_document_mut().substitute_line();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }

            // Change commands (delete + enter insert mode)
            Command::ChangeLine => {
                let _deleted = shared.buffer_manager.current_document_mut().change_line();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeLines(count) => {
                let mut deleted_lines = Vec::new();
                for _ in 0..count {
                    let is_empty = shared.buffer_manager.current_document().line_count() == 0;
                    if !is_empty {
                        deleted_lines.push(shared.buffer_manager.current_document_mut().change_line());
                        // Adjust cursor if we're at the end
                        let line_count = shared.buffer_manager.current_document().line_count();
                        if shared.buffer_manager.current_document().cursor_line >= line_count {
                            shared.buffer_manager.current_document_mut().cursor_line = line_count.saturating_sub(1);
                        }
                    } else {
                        break;
                    }
                }
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToEndOfLine => {
                let _deleted = shared.buffer_manager.current_document_mut().change_to_end_of_line();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeWord => {
                let _deleted = shared.buffer_manager.current_document_mut().change_word_forward();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeBigWord => {
                let _deleted = shared.buffer_manager.current_document_mut().change_big_word_forward();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeWordBackward => {
                let _deleted = shared.buffer_manager.current_document_mut().change_word_backward();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeBigWordBackward => {
                let _deleted = shared.buffer_manager.current_document_mut().change_big_word_backward();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToEndOfWord => {
                let _deleted = shared.buffer_manager.current_document_mut().change_to_end_of_word();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToEndOfBigWord => {
                let _deleted = shared.buffer_manager.current_document_mut().change_to_end_of_big_word();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToStartOfLine => {
                let _deleted = shared.buffer_manager.current_document_mut().change_to_start_of_line();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToFirstNonWhitespace => {
                let _deleted = shared.buffer_manager.current_document_mut().change_to_first_non_whitespace();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToEndOfFile => {
                let _deleted = shared.buffer_manager.current_document_mut().change_to_end_of_file();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeToStartOfFile => {
                let _deleted = shared.buffer_manager.current_document_mut().change_to_start_of_file();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeUntilChar(target) => {
                let _deleted = shared.buffer_manager.current_document_mut().change_until_char(target);
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeUntilCharBackward(target) => {
                let _deleted = shared.buffer_manager.current_document_mut().change_until_char_backward(target);
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeFindChar(target) => {
                let _deleted = shared.buffer_manager.current_document_mut().change_find_char(target);
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                return ModeTransition::ToMode(Mode::Insert);
            }
            Command::ChangeFindCharBackward(target) => {
                let _deleted = shared.buffer_manager.current_document_mut().change_find_char_backward(target);
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
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
                shared.buffer_manager.yank_text(yank_type, register, &mut shared.register_manager, &mut shared.status_message);
            }
            Command::Paste(paste_type, register) => {
                shared.buffer_manager.paste_text(paste_type, register, &mut shared.register_manager, &mut shared.status_message);
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
        match command {
            // Basic movement
            Command::MoveUp => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_cursor_up();
                }
            }
            Command::MoveDown => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_cursor_down();
                }
            }
            Command::MoveLeft => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_cursor_left();
                }
            }
            Command::MoveRight => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_cursor_right();
                }
            }

            // Word movement
            Command::MoveWordForward => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_word_forward();
                }
            }
            Command::MoveWordBackward => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_word_backward();
                }
            }
            Command::MoveWordEnd => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_word_end();
                }
            }
            Command::MoveBigWordForward => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_big_word_forward();
                }
            }
            Command::MoveBigWordBackward => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_big_word_backward();
                }
            }
            Command::MoveBigWordEnd => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_big_word_end();
                }
            }

            // Line movement
            Command::MoveLineStart => {
                shared.buffer_manager.current_document_mut().move_line_start();
            }
            Command::MoveLineEnd => {
                shared.buffer_manager.current_document_mut().move_line_end();
            }
            Command::MoveFirstNonWhitespace => {
                shared.buffer_manager.current_document_mut().move_first_non_whitespace();
            }
            Command::MoveDownToFirstNonWhitespace => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_down_to_first_non_whitespace();
                }
            }
            Command::MoveUpToFirstNonWhitespace => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_up_to_first_non_whitespace();
                }
            }

            // Document movement
            Command::MoveDocumentStart => {
                shared.buffer_manager.current_document_mut().move_document_start();
            }
            Command::MoveDocumentEnd => {
                shared.buffer_manager.current_document_mut().move_document_end();
            }
            Command::MovePageUp => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_page_up();
                }
            }
            Command::MovePageDown => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_page_down();
                }
            }
            Command::MoveHalfPageUp => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_half_page_up();
                }
            }
            Command::MoveHalfPageDown => {
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().move_half_page_down();
                }
            }

            // Line jumping
            Command::MoveToLine(line) => {
                shared.buffer_manager.current_document_mut().move_to_line(line);
            }

            // Character search
            Command::FindChar(c) => {
                self.last_find_char = Some(c);
                self.last_find_forward = true;
                self.last_find_before = false;
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().find_char(c, true, false);
                }
            }
            Command::FindCharBackward(c) => {
                self.last_find_char = Some(c);
                self.last_find_forward = false;
                self.last_find_before = false;
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().find_char(c, false, false);
                }
            }
            Command::FindCharBefore(c) => {
                self.last_find_char = Some(c);
                self.last_find_forward = true;
                self.last_find_before = true;
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().find_char(c, true, true);
                }
            }
            Command::FindCharBeforeBackward(c) => {
                self.last_find_char = Some(c);
                self.last_find_forward = false;
                self.last_find_before = true;
                for _ in 0..count {
                    shared.buffer_manager.current_document_mut().find_char(c, false, true);
                }
            }
            Command::RepeatFind => {
                if let Some(c) = self.last_find_char {
                    for _ in 0..count {
                        shared.buffer_manager.current_document_mut().find_char(c, self.last_find_forward, self.last_find_before);
                    }
                }
            }
            Command::RepeatFindReverse => {
                if let Some(c) = self.last_find_char {
                    for _ in 0..count {
                        shared.buffer_manager.current_document_mut().find_char(c, !self.last_find_forward, self.last_find_before);
                    }
                }
            }

            // Bracket matching
            Command::MatchBracket => {
                if let Some((target_line, target_column)) = shared.buffer_manager.current_document().find_matching_bracket() {
                    shared.buffer_manager.current_document_mut().cursor_line = target_line;
                    shared.buffer_manager.current_document_mut().cursor_column = target_column;
                    shared.status_message = "Bracket matched".to_string();
                } else {
                    shared.status_message = "No matching bracket found".to_string();
                }
            }

            // Screen positioning
            Command::MoveToScreenTop | Command::MoveToScreenMiddle | Command::MoveToScreenBottom => {
                // TODO: Need to implement screen positioning
                shared.status_message = "Screen positioning not yet implemented".to_string();
            }

            _ => {} // Should not reach here
        }
    }

    fn execute_edit_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        match command {
            Command::DeleteChar => {
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                shared.buffer_manager.current_document_mut().delete_char();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .end_group(cursor_pos);
            }
            Command::DeleteCharForward => {
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                shared.buffer_manager.current_document_mut().delete_char_forward();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .end_group(cursor_pos);
            }
            Command::DeleteCharBackward => {
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .start_group(cursor_pos);
                shared.buffer_manager.current_document_mut().delete_char_backward();
                let doc = shared.buffer_manager.current_document();
                let cursor_pos = (doc.cursor_line, doc.cursor_column);
                shared.buffer_manager.current_document_mut()
                    .undo_manager
                    .end_group(cursor_pos);
            }
            Command::DeleteLine => {
                shared.buffer_manager.current_document_mut().delete_line();
            }
            Command::DeleteLines(count) => {
                for _ in 0..count {
                    let line_count = shared.buffer_manager.current_document().line_count();
                    if line_count > 1 {
                        shared.buffer_manager.current_document_mut().delete_line();
                        // Adjust cursor if we deleted the last line
                        let new_line_count = shared.buffer_manager.current_document().line_count();
                        if shared.buffer_manager.current_document().cursor_line >= new_line_count {
                            shared.buffer_manager.current_document_mut().cursor_line = new_line_count.saturating_sub(1);
                        }
                    } else {
                        break;
                    }
                }
            }
            Command::DeleteToEndOfLine => {
                shared.buffer_manager.current_document_mut().delete_to_end_of_line();
            }
            Command::DeleteWord => {
                shared.buffer_manager.current_document_mut().delete_word_forward();
            }
            Command::DeleteBigWord => {
                shared.buffer_manager.current_document_mut().delete_big_word_forward();
            }
            Command::DeleteWordBackward => {
                shared.buffer_manager.current_document_mut().delete_word_backward();
            }
            Command::DeleteBigWordBackward => {
                shared.buffer_manager.current_document_mut().delete_big_word_backward();
            }
            Command::DeleteToEndOfWord => {
                shared.buffer_manager.current_document_mut().delete_to_end_of_word();
            }
            Command::DeleteToEndOfBigWord => {
                shared.buffer_manager.current_document_mut().delete_to_end_of_big_word();
            }
            Command::DeleteToStartOfLine => {
                shared.buffer_manager.current_document_mut().delete_to_start_of_line();
            }
            Command::DeleteToFirstNonWhitespace => {
                shared.buffer_manager.current_document_mut().delete_to_first_non_whitespace();
            }
            Command::DeleteToEndOfFile => {
                shared.buffer_manager.current_document_mut().delete_to_end_of_file();
            }
            Command::DeleteToStartOfFile => {
                shared.buffer_manager.current_document_mut().delete_to_start_of_file();
            }
            Command::DeleteUntilChar(target) => {
                shared.buffer_manager.current_document_mut().delete_until_char(target);
            }
            Command::DeleteUntilCharBackward(target) => {
                shared.buffer_manager.current_document_mut().delete_until_char_backward(target);
            }
            Command::DeleteFindChar(target) => {
                shared.buffer_manager.current_document_mut().delete_find_char(target);
            }
            Command::DeleteFindCharBackward(target) => {
                shared.buffer_manager.current_document_mut().delete_find_char_backward(target);
            }

            _ => {} // Should not reach here
        }
    }

    fn execute_mark_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        match command {
            Command::SetMark(mark_char) => {
                let (line, column, filename) = {
                    let doc = shared.buffer_manager.current_document();
                    (doc.cursor_line, doc.cursor_column, doc.filename.clone())
                };
                if mark_char.is_ascii_lowercase() {
                    let _ = shared.buffer_manager.current_document_mut().set_local_mark(mark_char, line, column);
                } else if mark_char.is_ascii_uppercase() {
                    let _ = shared.mark_manager.set_global_mark(mark_char, line, column, filename);
                }
            }
            Command::JumpToMark(mark_char) => {
                if mark_char.is_ascii_lowercase() {
                    if let Some((line, column)) = shared.buffer_manager.current_document().get_local_mark(mark_char) {
                        shared.buffer_manager.current_document_mut().cursor_line = line;
                        shared.buffer_manager.current_document_mut().cursor_column = column;
                    }
                } else if let Some(mark) = shared.mark_manager.get_global_mark(mark_char).cloned() {
                    shared.buffer_manager.current_document_mut().cursor_line = mark.line;
                    shared.buffer_manager.current_document_mut().cursor_column = mark.column;
                }
            }
            Command::JumpToMarkLine(mark_char) => {
                if mark_char.is_ascii_lowercase() {
                    if let Some((line, _)) = shared.buffer_manager.current_document().get_local_mark(mark_char) {
                        shared.buffer_manager.current_document_mut().cursor_line = line;
                        shared.buffer_manager.current_document_mut().move_first_non_whitespace();
                    }
                } else if let Some(mark) = shared.mark_manager.get_global_mark(mark_char).cloned() {
                    shared.buffer_manager.current_document_mut().cursor_line = mark.line;
                    shared.buffer_manager.current_document_mut().move_first_non_whitespace();
                }
            }
            Command::JumpBackward => {
                if let Some(entry) = shared.mark_manager.jump_backward().cloned() {
                    shared.buffer_manager.current_document_mut().cursor_line = entry.line;
                    shared.buffer_manager.current_document_mut().cursor_column = entry.column;
                }
            }
            Command::JumpForward => {
                if let Some(entry) = shared.mark_manager.jump_forward().cloned() {
                    shared.buffer_manager.current_document_mut().cursor_line = entry.line;
                    shared.buffer_manager.current_document_mut().cursor_column = entry.column;
                }
            }
            _ => {}
        }
    }

    fn execute_search_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        match command {
            Command::SearchNext => shared.search_state.next(shared.buffer_manager.current_document_mut(), &mut shared.status_message),
            Command::SearchPrevious => shared.search_state.previous(shared.buffer_manager.current_document_mut(), &mut shared.status_message),
            Command::SearchWordUnderCursor => shared.search_state.search_word_forward(shared.buffer_manager.current_document_mut(), &mut shared.status_message),
            Command::SearchWordUnderCursorBackward => shared.search_state.search_word_backward(shared.buffer_manager.current_document_mut(), &mut shared.status_message),
            _ => {}
        }
    }

    fn execute_indentation_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        shared.buffer_manager.execute_indent_command(command, &mut shared.status_message);
    }

    fn execute_join_lines_command(&mut self, shared: &mut SharedEditorState) {
        let doc = shared.buffer_manager.current_document_mut();
        if doc.join_lines() {
            shared.status_message = "Lines joined".to_string();
        } else {
            shared.status_message = "Cannot join: at last line".to_string();
        }
    }

    fn execute_case_command(&mut self, command: Command, shared: &mut SharedEditorState) {
        let doc = shared.buffer_manager.current_document_mut();
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
                if let Some(undo_group) = shared.buffer_manager.current_document_mut().undo_manager.undo() {
                    // Apply the reverse of the undo group to undo the changes
                    undo_group.apply_reverse_to_document(shared.buffer_manager.current_document_mut());
                    
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
                if let Some(redo_group) = shared.buffer_manager.current_document_mut().undo_manager.redo() {
                    // Apply the redo group to redo the changes
                    redo_group.apply_to_document(shared.buffer_manager.current_document_mut());
                    
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