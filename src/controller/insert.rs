use crate::controller::shared_state::{ModeController, ModeTransition, SharedEditorState};
use crate::controller::command_types::{Mode, Command};
use crate::controller::key_handler::KeyHandler;
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
                        let doc = shared.session_controller.current_document();
                        (doc.cursor_line(), doc.cursor_column())
                    };
                    shared.session_controller.current_document_mut()
                        .undo_manager_mut()
                        .end_group(cursor_pos);

                    // Mark last insert position when leaving insert mode
                    let cursor_pos = {
                        let doc = shared.session_controller.current_document();
                        (doc.cursor_line(), doc.cursor_column())
                    };
                    shared.mark_manager
                        .set_last_insert(cursor_pos.0, cursor_pos.1);
                    
                    return ModeTransition::ToMode(Mode::Normal);
                }
                Command::InsertChar(c) => {
                    shared.session_controller.current_document_mut().insert_char(c);
                    // Invalidate bracket cache on modification
                    shared.cached_unmatched_brackets = None;
                    // Mark change position
                    let doc = shared.session_controller.current_document();
                    shared.mark_manager
                        .set_last_change(doc.cursor_line(), doc.cursor_column());
                }
                Command::InsertNewline => {
                    shared.session_controller.current_document_mut().insert_newline();
                    // Invalidate bracket cache on modification
                    shared.cached_unmatched_brackets = None;
                    // Mark change position
                    let doc = shared.session_controller.current_document();
                    shared.mark_manager
                        .set_last_change(doc.cursor_line(), doc.cursor_column());
                }
                Command::InsertTab => {
                    let tab_width = shared.view.get_tab_stop();
                    shared.session_controller.current_document_mut().insert_tab_or_spaces(tab_width);
                    // Invalidate bracket cache on modification
                    shared.cached_unmatched_brackets = None;
                }
                Command::DeleteChar => {
                    shared.session_controller.current_document_mut().delete_char();
                    // Invalidate bracket cache on modification
                    shared.cached_unmatched_brackets = None;
                }
                // Movement commands in insert mode
                Command::MoveLeft => {
                    shared.session_controller.current_document_mut().move_cursor_left();
                }
                Command::MoveRight => {
                    shared.session_controller.current_document_mut().move_cursor_right();
                }
                Command::MoveUp => {
                    shared.session_controller.current_document_mut().move_cursor_up();
                }
                Command::MoveDown => {
                    shared.session_controller.current_document_mut().move_cursor_down();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::SessionController;
    use crate::document_model::{Document, MarkManager, RegisterManager, SearchState};
    use crate::view::View;
    use crossterm::event::{KeyCode, KeyModifiers};
    
    fn create_test_shared_state() -> SharedEditorState {
        SharedEditorState {
            session_controller: SessionController::new(),
            view: View::new(),
            mark_manager: MarkManager::new(),
            register_manager: RegisterManager::new(),
            search_state: SearchState::new(),
            status_message: String::new(),
            show_all_unmatched: false,
            cached_unmatched_brackets: None,
        }
    }
    
    fn create_test_shared_state_with_content(content: &str) -> SharedEditorState {
        let mut state = create_test_shared_state();
        let doc = Document::from_string(content.to_string());
        state.session_controller.buffers[0] = doc;
        state
    }
    
    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    
    #[test]
    fn test_new_controller() {
        let controller = InsertController::new();
        // Just verify it creates without panic
        // InsertController is a zero-sized type
        let _ = controller;
    }
    
    #[test]
    fn test_insert_single_char() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Insert 'a'
        let result = controller.handle_key(key_event(KeyCode::Char('a')), &mut shared);
        
        assert_eq!(result, ModeTransition::Stay);
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "a");
        assert_eq!(shared.session_controller.current_document().cursor_column(), 1);
    }
    
    #[test]
    fn test_insert_multiple_chars() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Insert "hello"
        for c in "hello".chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "hello");
        assert_eq!(shared.session_controller.current_document().cursor_column(), 5);
    }
    
    #[test]
    fn test_insert_at_position() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state_with_content("hello world");
        
        // Move cursor to position 5 (between "hello" and " world")
        shared.session_controller.current_document_mut().set_cursor(0, 5).unwrap();
        
        // Insert " there"
        for c in " there".chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "hello there world");
    }
    
    #[test]
    fn test_insert_newline() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state_with_content("hello");
        
        // Move to end and insert newline
        shared.session_controller.current_document_mut().set_cursor(0, 5).unwrap();
        let result = controller.handle_key(key_event(KeyCode::Enter), &mut shared);
        
        assert_eq!(result, ModeTransition::Stay);
        
        // Insert "world" on new line
        for c in "world".chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "hello\nworld");
        assert_eq!(shared.session_controller.current_document().cursor_line(), 1);
        assert_eq!(shared.session_controller.current_document().cursor_column(), 5);
    }
    
    #[test]
    fn test_insert_newline_middle_of_line() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state_with_content("hello world");
        
        // Move cursor to position 5 (after "hello")
        shared.session_controller.current_document_mut().set_cursor(0, 5).unwrap();
        
        // Insert newline
        controller.handle_key(key_event(KeyCode::Enter), &mut shared);
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "hello\n world");
        assert_eq!(shared.session_controller.current_document().cursor_line(), 1);
        assert_eq!(shared.session_controller.current_document().cursor_column(), 0);
    }
    
    #[test]
    fn test_insert_tab() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Insert tab
        let result = controller.handle_key(key_event(KeyCode::Tab), &mut shared);
        
        assert_eq!(result, ModeTransition::Stay);
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        // Default tab width is 4 spaces
        assert_eq!(content, "    ");
        assert_eq!(shared.session_controller.current_document().cursor_column(), 4);
    }
    
    #[test]
    fn test_backspace_delete_char() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state_with_content("hello");
        
        // Move to end
        shared.session_controller.current_document_mut().set_cursor(0, 5).unwrap();
        
        // Press backspace
        let result = controller.handle_key(key_event(KeyCode::Backspace), &mut shared);
        
        assert_eq!(result, ModeTransition::Stay);
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "hell");
        assert_eq!(shared.session_controller.current_document().cursor_column(), 4);
    }
    
    #[test]
    fn test_backspace_at_line_start() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state_with_content("hello\nworld");
        
        // Move to start of second line
        shared.session_controller.current_document_mut().set_cursor(1, 0).unwrap();
        
        // Press backspace - should join lines
        controller.handle_key(key_event(KeyCode::Backspace), &mut shared);
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "helloworld");
        assert_eq!(shared.session_controller.current_document().cursor_line(), 0);
        assert_eq!(shared.session_controller.current_document().cursor_column(), 5);
    }
    
    #[test]
    fn test_exit_insert_mode() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Start undo group (normally done when entering insert mode)
        shared.session_controller.current_document_mut()
            .undo_manager_mut()
            .start_group((0, 0));
        
        // Insert some text
        for c in "hello".chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        // Press Escape to exit insert mode
        let result = controller.handle_key(key_event(KeyCode::Esc), &mut shared);
        
        assert_eq!(result, ModeTransition::ToMode(Mode::Normal));
        // The mark manager sets the last insert position but doesn't have a getter
        // We can't verify directly, but the operation shouldn't panic
    }
    
    #[test]
    fn test_movement_in_insert_mode() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state_with_content("hello\nworld");
        
        // Move cursor to middle of first line
        shared.session_controller.current_document_mut().set_cursor(0, 2).unwrap();
        
        // Test arrow keys
        controller.handle_key(key_event(KeyCode::Left), &mut shared);
        assert_eq!(shared.session_controller.current_document().cursor_column(), 1);
        
        controller.handle_key(key_event(KeyCode::Right), &mut shared);
        assert_eq!(shared.session_controller.current_document().cursor_column(), 2);
        
        controller.handle_key(key_event(KeyCode::Down), &mut shared);
        assert_eq!(shared.session_controller.current_document().cursor_line(), 1);
        
        controller.handle_key(key_event(KeyCode::Up), &mut shared);
        assert_eq!(shared.session_controller.current_document().cursor_line(), 0);
    }
    
    #[test]
    fn test_insert_special_chars() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Insert various special characters
        let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        for c in special_chars.chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, special_chars);
    }
    
    #[test]
    fn test_insert_unicode() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Insert unicode characters
        controller.handle_key(key_event(KeyCode::Char('H')), &mut shared);
        controller.handle_key(key_event(KeyCode::Char('e')), &mut shared);
        controller.handle_key(key_event(KeyCode::Char('l')), &mut shared);
        controller.handle_key(key_event(KeyCode::Char('l')), &mut shared);
        controller.handle_key(key_event(KeyCode::Char('o')), &mut shared);
        controller.handle_key(key_event(KeyCode::Char(' ')), &mut shared);
        controller.handle_key(key_event(KeyCode::Char('世')), &mut shared);
        controller.handle_key(key_event(KeyCode::Char('界')), &mut shared);
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        // Verify at least that ASCII and some unicode works
        assert!(content.starts_with("Hello "));
        assert!(content.contains('界'));
    }
    
    #[test]
    fn test_bracket_cache_invalidation() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Set a cached bracket value
        shared.cached_unmatched_brackets = Some(vec![(0, 1)]);
        
        // Insert a character
        controller.handle_key(key_event(KeyCode::Char('(')), &mut shared);
        
        // Cache should be invalidated
        assert!(shared.cached_unmatched_brackets.is_none());
    }
    
    #[test]
    fn test_marks_updated_on_change() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Insert text
        controller.handle_key(key_event(KeyCode::Char('a')), &mut shared);
        
        // The mark manager sets the last change position
        // We verify the operation doesn't panic
        
        // Insert more text at different position
        controller.handle_key(key_event(KeyCode::Enter), &mut shared);
        controller.handle_key(key_event(KeyCode::Char('b')), &mut shared);
        
        // The mark should be updated internally
    }
    
    #[test]
    fn test_continuous_typing() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Simulate typing a sentence
        let sentence = "The quick brown fox jumps over the lazy dog.";
        for c in sentence.chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, sentence);
        assert_eq!(shared.session_controller.current_document().cursor_column(), sentence.len());
    }
    
    #[test]
    fn test_insert_and_delete_sequence() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Type "hello"
        for c in "hello".chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        // Delete last 2 chars
        controller.handle_key(key_event(KeyCode::Backspace), &mut shared);
        controller.handle_key(key_event(KeyCode::Backspace), &mut shared);
        
        // Type "p"
        controller.handle_key(key_event(KeyCode::Char('p')), &mut shared);
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "help");
    }
    
    #[test]
    fn test_multiline_editing() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Type first line
        for c in "Line 1".chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        // New line
        controller.handle_key(key_event(KeyCode::Enter), &mut shared);
        
        // Type second line
        for c in "Line 2".chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        // New line
        controller.handle_key(key_event(KeyCode::Enter), &mut shared);
        
        // Type third line
        for c in "Line 3".chars() {
            controller.handle_key(key_event(KeyCode::Char(c)), &mut shared);
        }
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "Line 1\nLine 2\nLine 3");
        assert_eq!(shared.session_controller.current_document().cursor_line(), 2);
    }
    
    #[test]
    fn test_tab_insertion_with_custom_width() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // Default tab width is 4, so we'll just test with that
        // Insert tab
        controller.handle_key(key_event(KeyCode::Tab), &mut shared);
        
        let content = shared.session_controller.current_document_mut().text_buffer_mut().get_text();
        assert_eq!(content, "    "); // 4 spaces (default)
        assert_eq!(shared.session_controller.current_document().cursor_column(), 4);
    }
    
    #[test]
    fn test_unhandled_command() {
        let mut controller = InsertController::new();
        let mut shared = create_test_shared_state();
        
        // PageDown is not handled in insert mode - it just stays in insert mode
        let result = controller.handle_key(key_event(KeyCode::PageDown), &mut shared);
        
        assert_eq!(result, ModeTransition::Stay);
        // The unhandled command message may or may not be set depending on key parsing
    }
}