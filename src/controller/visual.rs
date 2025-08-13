use crate::controller::shared_state::{ModeController, ModeTransition, SharedEditorState};
use crate::controller::command_types::{Mode, Command};
use crate::controller::key_handler::KeyHandler;
use crate::controller::visual_mode::{Selection, VisualMode, VisualModeHandler};
use crossterm::event::{KeyEvent};

pub struct VisualController {
    pub visual_selection: Option<Selection>,
}

impl VisualController {
    pub fn new() -> Self {
        Self {
            visual_selection: None,
        }
    }
    
    pub fn start_selection(&mut self, mode: Mode, line: usize, column: usize) {
        let visual_mode = match mode {
            Mode::VisualChar => VisualMode::Char,
            Mode::VisualLine => VisualMode::Line,
            Mode::VisualBlock => VisualMode::Block,
            _ => VisualMode::Char, // fallback
        };
        self.visual_selection = Some(Selection::new(line, column, visual_mode));
    }
}

impl ModeController for VisualController {
    fn handle_key(&mut self, key_event: KeyEvent, shared: &mut SharedEditorState) -> ModeTransition {
        // Parse the key event using the existing key handler
        let command = KeyHandler::parse_key_with_state(
            &Mode::VisualChar, // Visual modes use same key parsing
            &key_event,
            &mut None, // pending_key not used much in visual mode
            &mut None, // number_prefix could be used but simplified for now
            &mut None, // pending_register not used much in visual mode
        );
        
        if let Some(command) = command {
            match command {
                Command::ExitVisualMode => {
                    self.visual_selection = None;
                    return ModeTransition::ToMode(Mode::Normal);
                }
                
                // Mode transitions from visual mode
                Command::EnterInsertMode => {
                    // Delete selection and enter insert mode
                    if let Some(selection) = &self.visual_selection {
                        VisualModeHandler::delete_selection(selection, shared.session_controller.current_document_mut());
                    }
                    self.visual_selection = None;
                    let doc = shared.session_controller.current_document();
                    let cursor_pos = (doc.cursor_line(), doc.cursor_column());
                    shared.session_controller.current_document_mut()
                        .undo_manager_mut()
                        .start_group(cursor_pos);
                    return ModeTransition::ToMode(Mode::Insert);
                }
                
                // Visual mode operations
                Command::Yank(_, _) => {
                    if let Some(selection) = &self.visual_selection {
                        let selected_text = VisualModeHandler::get_selected_text(selection, shared.session_controller.current_document());
                        let register_type = match selection.mode {
                            VisualMode::Line => crate::document_model::RegisterType::Line,
                            VisualMode::Char => crate::document_model::RegisterType::Character,
                            VisualMode::Block => crate::document_model::RegisterType::Block,
                        };
                        shared.register_manager.store_in_register(None, selected_text, register_type);
                    }
                    self.visual_selection = None;
                    return ModeTransition::ToMode(Mode::Normal);
                }
                
                Command::DeleteChar => {
                    if let Some(selection) = &self.visual_selection {
                        VisualModeHandler::delete_selection(selection, shared.session_controller.current_document_mut());
                    }
                    self.visual_selection = None;
                    return ModeTransition::ToMode(Mode::Normal);
                }
                
                Command::IndentLine => {
                    if let Some(selection) = &self.visual_selection {
                        VisualModeHandler::indent_selection(selection, shared.session_controller.current_document_mut(), shared.view.get_tab_stop(), true); // Default to spaces
                    }
                    // Stay in visual mode after indenting
                }
                
                Command::DedentLine => {
                    if let Some(selection) = &self.visual_selection {
                        VisualModeHandler::dedent_selection(selection, shared.session_controller.current_document_mut(), shared.view.get_tab_stop());
                    }
                    // Stay in visual mode after dedenting
                }
                
                // Movement commands - update selection
                Command::MoveUp | Command::MoveDown | Command::MoveLeft | Command::MoveRight |
                Command::MoveWordForward | Command::MoveWordBackward | Command::MoveLineStart | Command::MoveLineEnd => {
                    // Execute the movement command on the document
                    self.execute_movement_command(command, shared);
                    
                    // Update selection end position
                    if let Some(selection) = &mut self.visual_selection {
                        let doc = shared.session_controller.current_document();
                        selection.update_end(doc.cursor_line(), doc.cursor_column());
                    }
                }
                
                _ => {
                    // Unhandled command in visual mode
                    shared.status_message = format!("Unhandled command in visual mode: {:?}", command);
                }
            }
        }
        
        ModeTransition::Stay
    }
}

impl VisualController {
    fn execute_movement_command(&self, command: Command, shared: &mut SharedEditorState) {
        match command {
            Command::MoveUp => { let _ = shared.session_controller.current_document_mut().move_cursor_up(); },
            Command::MoveDown => { let _ = shared.session_controller.current_document_mut().move_cursor_down(); },
            Command::MoveLeft => { let _ = shared.session_controller.current_document_mut().move_cursor_left(); },
            Command::MoveRight => { let _ = shared.session_controller.current_document_mut().move_cursor_right(); },
            Command::MoveWordForward => shared.session_controller.current_document_mut().move_word_forward(),
            Command::MoveWordBackward => shared.session_controller.current_document_mut().move_word_backward(),
            Command::MoveLineStart => shared.session_controller.current_document_mut().move_line_start(),
            Command::MoveLineEnd => shared.session_controller.current_document_mut().move_line_end(),
            // Add more movement commands as needed
            _ => {}
        }
    }
}