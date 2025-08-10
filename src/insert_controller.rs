use crate::mode_controllers::{ModeController, ModeTransition, SharedEditorState};
use crate::command::{Mode, Command};
use crate::key_handler::KeyHandler;
use crossterm::event::{KeyEvent};

pub struct InsertController {
    // Insert mode specific state can go here if needed
}

impl InsertController {
    pub fn new() -> Self {
        Self {}
    }
}

impl ModeController for InsertController {
    fn handle_key(&mut self, key_event: KeyEvent, shared: &mut SharedEditorState) -> ModeTransition {
        // Parse the key event using the existing key handler
        let command = KeyHandler::parse_key_with_state(
            &Mode::Insert,
            &key_event,
            &mut None, // pending_key not used in insert mode
            &mut None, // number_prefix not used in insert mode
            &mut None, // pending_register not used in insert mode
        );
        
        if let Some(command) = command {
            match command {
                Command::ExitInsertMode => {
                    // End undo group when leaving insert mode
                    let cursor_pos = {
                        let doc = shared.buffer_manager.current_document();
                        (doc.cursor_line, doc.cursor_column)
                    };
                    shared.buffer_manager.current_document_mut()
                        .undo_manager
                        .end_group(cursor_pos);

                    // Mark last insert position when leaving insert mode
                    let cursor_pos = {
                        let doc = shared.buffer_manager.current_document();
                        (doc.cursor_line, doc.cursor_column)
                    };
                    shared.mark_manager
                        .set_last_insert(cursor_pos.0, cursor_pos.1);
                    
                    return ModeTransition::ToMode(Mode::Normal);
                }
                Command::InsertChar(c) => {
                    shared.buffer_manager.current_document_mut().insert_char(c);
                    // Mark change position
                    let doc = shared.buffer_manager.current_document();
                    shared.mark_manager
                        .set_last_change(doc.cursor_line, doc.cursor_column);
                }
                Command::InsertNewline => {
                    shared.buffer_manager.current_document_mut().insert_newline();
                    // Mark change position
                    let doc = shared.buffer_manager.current_document();
                    shared.mark_manager
                        .set_last_change(doc.cursor_line, doc.cursor_column);
                }
                Command::InsertTab => {
                    let tab_width = shared.view.get_tab_stop();
                    shared.buffer_manager.current_document_mut().insert_tab_or_spaces(tab_width);
                }
                Command::DeleteChar => {
                    shared.buffer_manager.current_document_mut().delete_char();
                }
                // Movement commands in insert mode
                Command::MoveLeft => {
                    shared.buffer_manager.current_document_mut().move_cursor_left();
                }
                Command::MoveRight => {
                    shared.buffer_manager.current_document_mut().move_cursor_right();
                }
                Command::MoveUp => {
                    shared.buffer_manager.current_document_mut().move_cursor_up();
                }
                Command::MoveDown => {
                    shared.buffer_manager.current_document_mut().move_cursor_down();
                }
                _ => {
                    // Unhandled command in insert mode
                    shared.status_message = format!("Unhandled command in insert mode: {:?}", command);
                }
            }
        }
        
        ModeTransition::Stay
    }
}