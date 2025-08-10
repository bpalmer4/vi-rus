use crate::mode_controllers::{ModeController, ModeTransition, SharedEditorState};
use crate::command::Mode;
use crossterm::event::{KeyEvent, KeyCode};

pub struct CommandController {
    pub command_buffer: String,
}

impl CommandController {
    pub fn new() -> Self {
        Self {
            command_buffer: String::new(),
        }
    }
    
    pub fn get_command_buffer(&self) -> &str {
        &self.command_buffer
    }
}

impl ModeController for CommandController {
    fn handle_key(&mut self, key_event: KeyEvent, shared: &mut SharedEditorState) -> ModeTransition {
        match key_event.code {
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                ModeTransition::Stay
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                ModeTransition::Stay
            }
            KeyCode::Enter => {
                // Execute the command
                let command_str = self.command_buffer.clone();
                let quit = self.execute_command(&command_str, shared);
                self.command_buffer.clear();
                
                if quit {
                    ModeTransition::Quit
                } else {
                    ModeTransition::ToMode(Mode::Normal)
                }
            }
            KeyCode::Esc => {
                // Cancel command mode
                self.command_buffer.clear();
                ModeTransition::ToMode(Mode::Normal)
            }
            _ => ModeTransition::Stay,
        }
    }
}

impl CommandController {
    fn execute_command(&mut self, command_str: &str, shared: &mut SharedEditorState) -> bool {
        // Parse and execute vim commands
        let trimmed = command_str.trim();
        
        if trimmed.is_empty() {
            return false;
        }
        
        // Handle buffer commands
        if let Some(result) = self.execute_buffer_command(trimmed, shared) {
            return result;
        }
        
        // Handle file commands
        if let Some(result) = self.execute_file_command(trimmed, shared) {
            return result;
        }
        
        // Handle setting commands
        if let Some(result) = self.execute_setting_command(trimmed, shared) {
            return result;
        }
        
        // Handle search & replace commands
        if let Some(result) = self.execute_search_replace_command(trimmed, shared) {
            return result;
        }
        
        // Handle mark management commands
        if let Some(result) = self.execute_mark_command(trimmed, shared) {
            return result;
        }
        
        // Handle utility commands
        if let Some(result) = self.execute_utility_command(trimmed, shared) {
            return result;
        }
        
        // Handle numeric line jumps and other patterns
        self.execute_misc_command(trimmed, shared)
    }

    fn execute_buffer_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> Option<bool> {
        match trimmed {
            "ls" | "buffers" => {
                shared.status_message = shared.buffer_manager.list_buffers();
                Some(false)
            }
            "bn" | "bnext" => {
                shared.status_message = shared.buffer_manager.next_buffer();
                Some(false)
            }
            "bp" | "bprev" | "bprevious" => {
                shared.status_message = shared.buffer_manager.prev_buffer();
                Some(false)
            }
            "bd" | "bdelete" => {
                match shared.buffer_manager.close_buffer() {
                    Ok(msg) => shared.status_message = msg,
                    Err(msg) => shared.status_message = msg,
                }
                Some(false)
            }
            "bd!" => {
                match shared.buffer_manager.force_close_buffer() {
                    Ok(msg) => shared.status_message = msg,
                    Err(msg) => shared.status_message = msg,
                }
                Some(false)
            }
            _ if trimmed.starts_with("b") && trimmed[1..].parse::<usize>().is_ok() => {
                let buffer_num = trimmed[1..].parse::<usize>().unwrap();
                match shared.buffer_manager.switch_to_buffer(buffer_num) {
                    Ok(msg) => shared.status_message = msg,
                    Err(msg) => shared.status_message = msg,
                }
                Some(false)
            }
            _ if trimmed.starts_with("bf ") => {
                // Switch to buffer by filename
                let filename = &trimmed[3..];
                let path = std::path::PathBuf::from(filename);
                match shared.buffer_manager.switch_to_file(&path) {
                    Ok(_) => {
                        shared.status_message = format!("Switched to buffer: {}", filename);
                    }
                    Err(e) => {
                        shared.status_message = format!("Error switching to file: {}", e);
                    }
                }
                Some(false)
            }
            _ => None
        }
    }

    fn execute_file_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> Option<bool> {
        match trimmed {
            "q" | "quit" => {
                // Check if file is modified
                if shared.buffer_manager.current_document().is_modified() {
                    shared.status_message = "No write since last change (add ! to override)".to_string();
                    Some(false)
                } else {
                    Some(true) // Quit
                }
            }
            "q!" | "quit!" => {
                Some(true) // Force quit
            }
            "w" | "write" => {
                // Save current file
                match shared.buffer_manager.current_document_mut().save() {
                    Ok(_) => {
                        shared.status_message = format!("\"{}\" written", 
                            shared.buffer_manager.get_display_filename());
                        Some(false)
                    }
                    Err(e) => {
                        shared.status_message = format!("Error saving file: {}", e);
                        Some(false)
                    }
                }
            }
            "wq" | "x" => {
                // Save and quit
                match shared.buffer_manager.current_document_mut().save() {
                    Ok(_) => {
                        shared.status_message = format!("\"{}\" written", 
                            shared.buffer_manager.get_display_filename());
                        Some(true) // Quit after save
                    }
                    Err(e) => {
                        shared.status_message = format!("Error saving file: {}", e);
                        Some(false)
                    }
                }
            }
            _ if trimmed.starts_with("w ") => {
                // Save to specific file
                let filename = &trimmed[2..];
                match shared.buffer_manager.current_document_mut().save_as(filename.into()) {
                    Ok(_) => {
                        shared.status_message = format!("\"{}\" written", filename);
                    }
                    Err(e) => {
                        shared.status_message = format!("Error saving file: {}", e);
                    }
                }
                Some(false)
            }
            _ => None
        }
    }

    fn execute_setting_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> Option<bool> {
        match trimmed {
            "set nu" | "set number" => {
                shared.view.set_line_numbers(true);
                shared.status_message = "Line numbers enabled".to_string();
                Some(false)
            }
            "set nonu" | "set nonumber" => {
                shared.view.set_line_numbers(false);
                shared.status_message = "Line numbers disabled".to_string();
                Some(false)
            }
            "set list" => {
                shared.view.set_show_whitespace(true);
                shared.status_message = "Whitespace characters shown".to_string();
                Some(false)
            }
            "set nolist" => {
                shared.view.set_show_whitespace(false);
                shared.status_message = "Whitespace characters hidden".to_string();
                Some(false)
            }
            "set et" | "set expandtab" => {
                shared.buffer_manager.current_document_mut().set_expand_tab(true);
                shared.status_message = "Tab key will insert spaces".to_string();
                Some(false)
            }
            "set noet" | "set noexpandtab" => {
                shared.buffer_manager.current_document_mut().set_expand_tab(false);
                shared.status_message = "Tab key will insert tabs".to_string();
                Some(false)
            }
            "set ff=unix" => {
                shared.buffer_manager.current_document_mut().set_line_ending(crate::document::LineEnding::Unix);
                shared.status_message = "Line endings set to Unix (LF)".to_string();
                Some(false)
            }
            "set ff=dos" => {
                shared.buffer_manager.current_document_mut().set_line_ending(crate::document::LineEnding::Windows);
                shared.status_message = "Line endings set to DOS (CRLF)".to_string();
                Some(false)
            }
            "set ff=mac" => {
                shared.buffer_manager.current_document_mut().set_line_ending(crate::document::LineEnding::Mac);
                shared.status_message = "Line endings set to Mac (CR)".to_string();
                Some(false)
            }
            _ if trimmed.starts_with("set tabstop=") => {
                let value_part = &trimmed[12..];
                if let Ok(tab_stop) = value_part.parse::<usize>() {
                    if tab_stop > 0 && tab_stop <= 16 {
                        shared.view.set_tab_stop(tab_stop);
                        shared.status_message = format!("Tab width set to {}", tab_stop);
                    } else {
                        shared.status_message = "Tab width must be between 1 and 16".to_string();
                    }
                } else {
                    shared.status_message = "Invalid tab width value".to_string();
                }
                Some(false)
            }
            _ => None
        }
    }

    fn execute_utility_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> Option<bool> {
        match trimmed {
            "help" | "h" | "?" => {
                shared.buffer_manager.add_help_buffer();
                shared.status_message = "Help buffer opened".to_string();
                Some(false)
            }
            "mkvirus" => {
                let sample_rc = crate::rc::RcLoader::generate_sample_rc();
                match std::fs::write(".virusrc", sample_rc) {
                    Ok(_) => {
                        shared.status_message = "Sample .virusrc created in current directory".to_string();
                    }
                    Err(e) => {
                        shared.status_message = format!("Error creating .virusrc: {}", e);
                    }
                }
                Some(false)
            }
            "detab" => {
                let tab_width = shared.view.get_tab_stop();
                let count = shared.buffer_manager.current_document_mut().tabs_to_spaces(tab_width);
                shared.status_message = if count == 1 {
                    "1 tab converted to spaces".to_string()
                } else {
                    format!("{} tabs converted to spaces", count)
                };
                Some(false)
            }
            "retab" => {
                let tab_width = shared.view.get_tab_stop();
                let count = shared.buffer_manager.current_document_mut().spaces_to_tabs(tab_width);
                shared.status_message = if count == 1 {
                    "1 space sequence converted to tab".to_string()
                } else {
                    format!("{} space sequences converted to tabs", count)
                };
                Some(false)
            }
            "e" => {
                // Create new empty buffer
                shared.status_message = shared.buffer_manager.create_new_buffer();
                Some(false)
            }
            "badd" => {
                // Add new empty buffer (similar to :enew but numbered)
                shared.status_message = shared.buffer_manager.create_new_buffer();
                Some(false)
            }
            _ if trimmed.starts_with("badd ") => {
                // Add new buffers for specified files
                let filenames_str = &trimmed[5..];
                let filenames: Vec<&str> = filenames_str.split_whitespace().collect();
                if !filenames.is_empty() {
                    shared.status_message = shared.buffer_manager.open_files(filenames);
                } else {
                    shared.status_message = "No filename specified".to_string();
                }
                Some(false)
            }
            _ if trimmed.starts_with("e ") => {
                // Open/create file(s)
                let filenames_str = &trimmed[2..];
                let filenames: Vec<&str> = filenames_str.split_whitespace().collect();
                if filenames.len() == 1 {
                    shared.status_message = shared.buffer_manager.open_file(filenames[0]);
                } else if filenames.len() > 1 {
                    shared.status_message = shared.buffer_manager.open_files(filenames);
                } else {
                    shared.status_message = "No filename specified".to_string();
                }
                Some(false)
            }
            _ => None
        }
    }

    fn execute_misc_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> bool {
        // Handle shell command execution first
        if let Some(result) = self.execute_shell_command(trimmed, shared) {
            return result;
        }

        // Handle file read operations (:r filename, :0r filename, :$r filename, etc.)
        if trimmed.starts_with("r ") {
            let filename = &trimmed[2..];
            match shared.buffer_manager.current_document_mut().insert_file_at_cursor(filename.as_ref()) {
                Ok(lines_added) => {
                    shared.status_message = format!("\"{}\" {} lines inserted", filename, lines_added);
                }
                Err(e) => {
                    shared.status_message = format!("Error reading file \"{}\": {}", filename, e);
                }
            }
            return false;
        } else if trimmed.starts_with("0r ") {
            // Insert at beginning of file
            let filename = &trimmed[3..];
            match shared.buffer_manager.current_document_mut().insert_file_at_line(filename.as_ref(), 0) {
                Ok(lines_added) => {
                    shared.status_message = format!("\"{}\" {} lines inserted at beginning", filename, lines_added);
                }
                Err(e) => {
                    shared.status_message = format!("Error reading file \"{}\": {}", filename, e);
                }
            }
            return false;
        } else if trimmed.starts_with("$r ") {
            // Insert at end of file
            let filename = &trimmed[3..];
            let line_count = shared.buffer_manager.current_document().line_count();
            match shared.buffer_manager.current_document_mut().insert_file_at_line(filename.as_ref(), line_count) {
                Ok(lines_added) => {
                    shared.status_message = format!("\"{}\" {} lines inserted at end", filename, lines_added);
                }
                Err(e) => {
                    shared.status_message = format!("Error reading file \"{}\": {}", filename, e);
                }
            }
            return false;
        } else if let Some(pos) = trimmed.find("r ") {
            // Handle :10r filename format
            let line_part = &trimmed[..pos];
            if let Ok(line_num) = line_part.parse::<usize>() {
                let filename = &trimmed[pos + 2..];
                match shared.buffer_manager.current_document_mut().insert_file_at_line(filename.as_ref(), line_num) {
                    Ok(lines_added) => {
                        shared.status_message = format!("\"{}\" {} lines inserted after line {}", filename, lines_added, line_num);
                    }
                    Err(e) => {
                        shared.status_message = format!("Error reading file \"{}\": {}", filename, e);
                    }
                }
                return false;
            }
        }
        
        // Handle numeric line jumps like ":42"
        if let Ok(line_num) = trimmed.parse::<usize>() {
            if line_num > 0 {
                let doc = shared.buffer_manager.current_document_mut();
                doc.cursor_line = (line_num - 1).min(doc.line_count().saturating_sub(1)); // Convert to 0-based and clamp
                doc.cursor_column = 0;
                shared.status_message = format!("Jumped to line {}", line_num);
            }
            false
        } else {
            shared.status_message = format!("Unknown command: {}", trimmed);
            false
        }
    }

    fn execute_search_replace_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> Option<bool> {
        // Handle substitute commands: s/pattern/replacement/flags or %s/pattern/replacement/flags
        if trimmed.starts_with("s/") || trimmed.starts_with("%s/") {
            let is_global_range = trimmed.starts_with("%s/");
            let command_part = if is_global_range { &trimmed[3..] } else { &trimmed[2..] };
            
            // Parse s/pattern/replacement/flags format
            let parts: Vec<&str> = command_part.split('/').collect();
            if parts.len() >= 2 {
                let pattern = parts[0];
                let replacement = parts.get(1).unwrap_or(&"");
                let flags = parts.get(2).unwrap_or(&"");
                
                // Parse flags
                let global_flag = flags.contains('g');
                let case_insensitive = flags.contains('i');
                
                // Execute the substitution
                let result = if is_global_range {
                    // %s - substitute in entire document
                    crate::search::SearchReplace::substitute_all_document(
                        shared.buffer_manager.current_document_mut(),
                        pattern,
                        replacement,
                        !case_insensitive,
                    )
                } else {
                    // s - substitute in current line only
                    let current_line = shared.buffer_manager.current_document().cursor_line;
                    crate::search::SearchReplace::substitute_document(
                        shared.buffer_manager.current_document_mut(),
                        current_line,
                        current_line,
                        pattern,
                        replacement,
                        global_flag,
                        !case_insensitive,
                    )
                };
                
                match result {
                    Ok(count) => {
                        if count > 0 {
                            if count == 1 {
                                shared.status_message = "1 substitution made".to_string();
                            } else {
                                shared.status_message = format!("{} substitutions made", count);
                            }
                        } else {
                            shared.status_message = "Pattern not found".to_string();
                        }
                    }
                    Err(e) => {
                        shared.status_message = format!("Substitution error: {:?}", e);
                    }
                }
                
                return Some(false);
            }
        }
        None
    }

    fn execute_mark_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> Option<bool> {
        match trimmed {
            "marks" => {
                // List all marks
                let local_marks = shared.buffer_manager.current_document().get_all_local_marks();
                let marks_vec = shared.mark_manager.list_marks(local_marks);
                
                // Format marks for display
                let mut marks_display = "Marks:\n".to_string();
                for (mark, line, col, filename) in marks_vec {
                    let file_display = filename.map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_else(|| "[current]".to_string());
                    marks_display.push_str(&format!("  {} line {}, col {} in {}\n", 
                        mark, line + 1, col + 1, file_display));
                }
                if marks_display == "Marks:\n" {
                    marks_display = "No marks set".to_string();
                }
                shared.status_message = marks_display;
                Some(false)
            }
            "jumps" | "ju" => {
                // Show jump list
                let (jump_list, current_pos) = shared.mark_manager.get_jump_list();
                let mut jump_info = format!("Jump list (current position: {}):\n", current_pos);
                for (i, entry) in jump_list.iter().enumerate() {
                    let marker = if i == current_pos { ">" } else { " " };
                    let filename = entry.filename.as_ref()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_else(|| "[current]".to_string());
                    jump_info.push_str(&format!("{}  {} line {}, col {} in {}\n", 
                        marker, i, entry.line + 1, entry.column + 1, filename));
                }
                shared.status_message = jump_info;
                Some(false)
            }
            "clear marks" => {
                shared.mark_manager.clear_all_marks();
                shared.status_message = "All marks cleared".to_string();
                Some(false)
            }
            "clear jumps" => {
                shared.mark_manager.clear_jump_list();
                shared.status_message = "Jump list cleared".to_string();
                Some(false)
            }
            "clear all" => {
                shared.mark_manager.clear_all_marks();
                shared.mark_manager.clear_jump_list();
                shared.status_message = "All marks and jump list cleared".to_string();
                Some(false)
            }
            _ => None
        }
    }

    fn execute_shell_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> Option<bool> {
        if trimmed.starts_with("r !") {
            let command_str = &trimmed[3..];
            match std::process::Command::new("sh")
                .arg("-c")
                .arg(command_str)
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if !output_str.is_empty() {
                        match shared.buffer_manager.current_document_mut().insert_text_at_cursor(&output_str) {
                            Ok(lines_added) => {
                                shared.status_message = format!("Command output: {} lines inserted", lines_added);
                            }
                            Err(e) => {
                                shared.status_message = format!("Error inserting command output: {}", e);
                            }
                        }
                    } else if !output.stderr.is_empty() {
                        let error_str = String::from_utf8_lossy(&output.stderr);
                        shared.status_message = format!("Command error: {}", error_str.trim());
                    } else {
                        shared.status_message = "Command executed (no output)".to_string();
                    }
                }
                Err(e) => {
                    shared.status_message = format!("Failed to execute command: {}", e);
                }
            }
            Some(false)
        } else {
            None
        }
    }
}