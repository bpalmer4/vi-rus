use crate::mode_controllers::{ModeController, ModeTransition, SharedEditorState};
use crate::command::Mode;
use crate::insert_controller::InsertController;
use crate::normal_controller::NormalController;
use crate::visual_controller::VisualController;
use crate::command_controller::CommandController;
use crate::buffer_manager::BufferManager;
use crate::view::{View, RenderParams};
use crate::marks::MarkManager;
use crate::registers::RegisterManager;
use crate::search::SearchState;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::stdout;
use std::path::PathBuf;

pub struct EditorController {
    shared_state: SharedEditorState,
    current_mode: Mode,
    
    // Mode-specific controllers
    insert_controller: InsertController,
    normal_controller: NormalController,
    visual_controller: VisualController,
    command_controller: CommandController,
    
    // Search mode state (handled directly like in original)
    command_buffer: String,
}

impl EditorController {
    pub fn new() -> Self {
        Self {
            shared_state: SharedEditorState {
                buffer_manager: BufferManager::new(),
                view: View::new(),
                mark_manager: MarkManager::new(),
                register_manager: RegisterManager::new(),
                search_state: SearchState::new(),
                status_message: String::new(),
                show_all_unmatched: false,
                cached_unmatched_brackets: None,
            },
            current_mode: Mode::Normal,
            insert_controller: InsertController::new(),
            normal_controller: NormalController::new(),
            visual_controller: VisualController::new(),
            command_controller: CommandController::new(),
            command_buffer: String::new(),
        }
    }
    
    pub fn new_with_files(filenames: Vec<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        // Use BufferManager's efficient new_with_files method
        let buffer_manager = BufferManager::new_with_files(filenames)?;
        
        let controller = Self {
            shared_state: SharedEditorState {
                buffer_manager,
                view: View::new(),
                mark_manager: MarkManager::new(),
                register_manager: RegisterManager::new(),
                search_state: SearchState::new(),
                status_message: "Files loaded".to_string(),
                show_all_unmatched: false,
                cached_unmatched_brackets: None,
            },
            current_mode: Mode::Normal,
            insert_controller: InsertController::new(),
            normal_controller: NormalController::new(),
            visual_controller: VisualController::new(),
            command_controller: CommandController::new(),
            command_buffer: String::new(),
        };
        
        Ok(controller)
    }
    
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        
        let result = self.run_loop();
        
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        
        result
    }
    
    fn run_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let buffer_info = format!(
                "Buffer {}/{}: \"{}\"",
                self.shared_state.buffer_manager.current_buffer_index() + 1,
                self.shared_state.buffer_manager.buffer_count(),
                self.get_display_filename()
            );

            // Refresh unmatched brackets cache if highlighting is enabled and needed
            if self.shared_state.show_all_unmatched {
                self.refresh_unmatched_cache_if_needed();
            }

            // Borrow fields separately to avoid borrowing conflicts
            let doc = self.shared_state.buffer_manager.current_document();

            // Calculate matching bracket position for highlighting
            let matching_bracket = doc.find_matching_bracket();
            let unmatched_bracket = doc.is_unmatched_bracket();

            let command_buffer_str = self.get_command_buffer_for_mode();
            let params = RenderParams {
                mode: &self.current_mode,
                command_buffer: &command_buffer_str,
                status_message: &self.shared_state.status_message,
                buffer_info: Some(&buffer_info),
                visual_selection: self.visual_controller.visual_selection.as_ref(),
                search_state: Some(&self.shared_state.search_state),
                matching_bracket,
                unmatched_bracket,
                all_unmatched_brackets: if self.shared_state.show_all_unmatched {
                    self.shared_state.cached_unmatched_brackets.as_ref()
                } else {
                    None
                },
            };
            self.shared_state.view.render(doc, &params)?;

            match event::read()? {
                Event::Key(key_event) => {
                    // Handle special modes that need direct character input
                    if self.current_mode == Mode::Search || self.current_mode == Mode::SearchBackward {
                        if self.handle_search_mode_input(key_event)? {
                            break; // Quit
                        }
                        continue;
                    }
                    
                    // Handle command mode
                    if self.current_mode == Mode::Command {
                        match self.command_controller.handle_key(key_event, &mut self.shared_state) {
                            ModeTransition::Stay => continue,
                            ModeTransition::ToMode(mode) => {
                                self.current_mode = mode;
                                continue;
                            }
                            ModeTransition::Quit => break,
                        }
                    }
                    
                    // Delegate to appropriate mode controller
                    let transition = self.handle_key_in_current_mode(key_event);
                    
                    match transition {
                        ModeTransition::Stay => {}
                        ModeTransition::ToMode(new_mode) => {
                            self.transition_to_mode(new_mode);
                        }
                        ModeTransition::Quit => break,
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn handle_key_in_current_mode(&mut self, key_event: KeyEvent) -> ModeTransition {
        match self.current_mode {
            Mode::Normal => self.normal_controller.handle_key(key_event, &mut self.shared_state),
            Mode::Insert => self.insert_controller.handle_key(key_event, &mut self.shared_state),
            Mode::VisualChar | Mode::VisualLine | Mode::VisualBlock => {
                self.visual_controller.handle_key(key_event, &mut self.shared_state)
            }
            Mode::Command => {
                // Already handled above
                ModeTransition::Stay
            }
            Mode::Search | Mode::SearchBackward => {
                // Already handled above
                ModeTransition::Stay
            }
        }
    }
    
    fn transition_to_mode(&mut self, new_mode: Mode) {
        // Handle any cleanup from the old mode
        match self.current_mode {
            Mode::VisualChar | Mode::VisualLine | Mode::VisualBlock => {
                // Visual mode cleanup if needed
            }
            _ => {}
        }
        
        // Handle initialization for the new mode
        match new_mode {
            Mode::VisualChar => {
                let doc = self.shared_state.buffer_manager.current_document();
                self.visual_controller.start_selection(new_mode, doc.cursor_line, doc.cursor_column);
            }
            Mode::VisualLine => {
                let doc = self.shared_state.buffer_manager.current_document();
                self.visual_controller.start_selection(new_mode, doc.cursor_line, doc.cursor_column);
            }
            Mode::VisualBlock => {
                let doc = self.shared_state.buffer_manager.current_document();
                self.visual_controller.start_selection(new_mode, doc.cursor_line, doc.cursor_column);
            }
            Mode::Command => {
                self.command_controller.command_buffer.clear();
            }
            Mode::Search | Mode::SearchBackward => {
                self.command_buffer.clear();
            }
            _ => {}
        }
        
        self.current_mode = new_mode;
    }
    
    fn handle_search_mode_input(&mut self, key_event: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        match key_event.code {
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                Ok(false)
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                Ok(false)
            }
            KeyCode::Enter => {
                // Execute the search
                let pattern = self.command_buffer.clone();
                // Set the search pattern and direction
                let direction = if self.current_mode == Mode::Search {
                    crate::search::SearchDirection::Forward
                } else {
                    crate::search::SearchDirection::Backward
                };
                
                let _ = self.shared_state.search_state.set_pattern(pattern, direction);
                
                // Search the document
                let doc = self.shared_state.buffer_manager.current_document();
                if let Ok(_) = self.shared_state.search_state.search_document(doc) {
                    // Find first match and move cursor there
                    if let Some(search_match) = self.shared_state.search_state.find_next_match(0, 0) {
                        let doc = self.shared_state.buffer_manager.current_document_mut();
                        doc.cursor_line = search_match.line;
                        doc.cursor_column = search_match.start_col;
                    }
                }
                self.command_buffer.clear();
                self.current_mode = Mode::Normal;
                Ok(false)
            }
            KeyCode::Esc => {
                self.command_buffer.clear();
                self.current_mode = Mode::Normal;
                Ok(false)
            }
            _ => Ok(false),
        }
    }
    
    fn get_command_buffer_for_mode(&self) -> String {
        match self.current_mode {
            Mode::Command => self.command_controller.get_command_buffer().to_string(),
            Mode::Search | Mode::SearchBackward => self.command_buffer.clone(),
            _ => String::new(),
        }
    }
    
    fn get_display_filename(&self) -> String {
        self.shared_state.buffer_manager.get_display_filename().to_string()
    }
    
    fn refresh_unmatched_cache_if_needed(&mut self) {
        // TODO: Implement unmatched bracket caching
        // This was in the original controller
    }
    
    /// Apply RC configuration to this editor controller
    pub fn apply_config(&mut self, config: &crate::rc::RcConfig) {
        crate::rc::RcLoader::apply_config_to_shared_state(&mut self.shared_state, config);
    }
}