use crate::controller::shared_state::{ModeController, ModeTransition, SharedEditorState};
use crate::controller::command_types::Mode;
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

#[derive(Debug)]
enum RangeType {
    CurrentLine,
    Global,
    LineRange(usize, usize),
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
                shared.status_message = shared.session_controller.list_buffers();
                Some(false)
            }
            "bn" | "bnext" => {
                shared.status_message = shared.session_controller.next_buffer();
                Some(false)
            }
            "bp" | "bprev" | "bprevious" => {
                shared.status_message = shared.session_controller.prev_buffer();
                Some(false)
            }
            "bd" | "bdelete" => {
                match shared.session_controller.close_buffer(&mut shared.mark_manager) {
                    Ok(msg) => shared.status_message = msg,
                    Err(msg) => shared.status_message = msg,
                }
                Some(false)
            }
            "bd!" => {
                match shared.session_controller.force_close_buffer(&mut shared.mark_manager) {
                    Ok(msg) => shared.status_message = msg,
                    Err(msg) => shared.status_message = msg,
                }
                Some(false)
            }
            _ if trimmed.starts_with("b") => {
                match trimmed[1..].parse::<usize>() {
                    Ok(buffer_num) => {
                        match shared.session_controller.switch_to_buffer(buffer_num) {
                            Ok(msg) => shared.status_message = msg,
                            Err(msg) => shared.status_message = msg,
                        }
                        Some(false)
                    }
                    Err(_) => {
                        shared.status_message = "Invalid buffer number".to_string();
                        Some(false)
                    }
                }
            }
            _ if trimmed.starts_with("bf ") => {
                // Switch to buffer by filename
                let filename = &trimmed[3..];
                let path = std::path::PathBuf::from(filename);
                match shared.session_controller.switch_to_file(&path) {
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
                if shared.session_controller.current_document().is_modified() {
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
                match shared.session_controller.current_document_mut().save() {
                    Ok(_) => {
                        shared.status_message = format!("\"{}\" written", 
                            shared.session_controller.get_display_filename());
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
                match shared.session_controller.current_document_mut().save() {
                    Ok(_) => {
                        shared.status_message = format!("\"{}\" written", 
                            shared.session_controller.get_display_filename());
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
                match shared.session_controller.current_document_mut().save_as(filename.into()) {
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
                shared.session_controller.current_document_mut().set_expand_tab(true);
                shared.status_message = "Tab key will insert spaces".to_string();
                Some(false)
            }
            "set noet" | "set noexpandtab" => {
                shared.session_controller.current_document_mut().set_expand_tab(false);
                shared.status_message = "Tab key will insert tabs".to_string();
                Some(false)
            }
            "set ff=unix" => {
                shared.session_controller.current_document_mut().set_line_ending(crate::document_model::LineEnding::Unix);
                shared.status_message = "Line endings set to Unix (LF)".to_string();
                Some(false)
            }
            "set ff=dos" => {
                shared.session_controller.current_document_mut().set_line_ending(crate::document_model::LineEnding::Windows);
                shared.status_message = "Line endings set to DOS (CRLF)".to_string();
                Some(false)
            }
            "set ff=mac" => {
                shared.session_controller.current_document_mut().set_line_ending(crate::document_model::LineEnding::Mac);
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
                shared.session_controller.add_help_buffer();
                shared.status_message = "Help buffer opened".to_string();
                Some(false)
            }
            "mkvirus" => {
                let sample_rc = crate::config::RcLoader::generate_sample_rc();
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
                let count = shared.session_controller.current_document_mut().tabs_to_spaces(tab_width);
                shared.status_message = if count == 1 {
                    "1 tab converted to spaces".to_string()
                } else {
                    format!("{} tabs converted to spaces", count)
                };
                Some(false)
            }
            "retab" => {
                let tab_width = shared.view.get_tab_stop();
                let count = shared.session_controller.current_document_mut().spaces_to_tabs(tab_width);
                shared.status_message = if count == 1 {
                    "1 space sequence converted to tab".to_string()
                } else {
                    format!("{} space sequences converted to tabs", count)
                };
                Some(false)
            }
            "ascii" | "normalize" => {
                let count = shared.session_controller.current_document_mut().ascii_normalize();
                shared.status_message = if count == 0 {
                    "No Unicode characters found to normalize".to_string()
                } else if count == 1 {
                    "1 line normalized to ASCII".to_string()
                } else {
                    format!("{} lines normalized to ASCII", count)
                };
                Some(false)
            }
            "brackets" | "checkbrackets" => {
                let unmatched = shared.session_controller.current_document().find_all_unmatched_brackets();
                if unmatched.is_empty() {
                    shared.status_message = "All brackets are properly matched".to_string();
                } else {
                    let mut msg = format!("Found {} unmatched bracket(s):\n", unmatched.len());
                    for (line, col) in unmatched.iter().take(10) { // Limit to first 10
                        msg.push_str(&format!("  Line {}, Column {}\n", line + 1, col + 1));
                    }
                    if unmatched.len() > 10 {
                        msg.push_str(&format!("  ... and {} more", unmatched.len() - 10));
                    }
                    shared.status_message = msg;
                }
                Some(false)
            }
            "redraw" => {
                shared.view.force_redraw();
                shared.status_message = "Screen refreshed".to_string();
                Some(false)
            }
            "scroll" => {
                let offset = shared.view.get_scroll_offset();
                let visible = shared.view.get_visible_lines_count();
                shared.status_message = format!("Scroll offset: {}, Visible lines: {}", offset, visible);
                Some(false)
            }
            "resetscroll" => {
                shared.view.reset_scroll();
                shared.status_message = "Scroll position reset".to_string();
                Some(false)
            }
            "e" => {
                // Create new empty buffer
                shared.status_message = shared.session_controller.create_new_buffer();
                Some(false)
            }
            "badd" => {
                // Add new empty buffer (similar to :enew but numbered)
                shared.status_message = shared.session_controller.create_new_buffer();
                Some(false)
            }
            _ if trimmed.starts_with("badd ") => {
                // Add new buffers for specified files
                let filenames_str = &trimmed[5..];
                let filenames: Vec<&str> = filenames_str.split_whitespace().collect();
                if !filenames.is_empty() {
                    shared.status_message = shared.session_controller.open_files(filenames);
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
                    shared.status_message = shared.session_controller.open_file(filenames[0]);
                } else if filenames.len() > 1 {
                    shared.status_message = shared.session_controller.open_files(filenames);
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
            match shared.session_controller.current_document_mut().insert_file_at_cursor(filename.as_ref()) {
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
            match shared.session_controller.current_document_mut().insert_file_at_line(filename.as_ref(), 0) {
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
            let line_count = shared.session_controller.current_document().line_count();
            match shared.session_controller.current_document_mut().insert_file_at_line(filename.as_ref(), line_count) {
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
                match shared.session_controller.current_document_mut().insert_file_at_line(filename.as_ref(), line_num) {
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
                // Add current position to jump list before jumping
                let doc = shared.session_controller.current_document();
                let current_filename = doc.filename.clone();
                shared.mark_manager.add_to_jump_list(doc.cursor_line(), doc.cursor_column(), current_filename);
                
                let doc = shared.session_controller.current_document_mut();
                let target_line = (line_num - 1).min(doc.line_count().saturating_sub(1)); // Convert to 0-based and clamp
                doc.move_cursor_to(target_line, 0);
                shared.status_message = format!("Jumped to line {}", line_num);
            }
            false
        } else {
            shared.status_message = format!("Unknown command: {}", trimmed);
            false
        }
    }

    fn execute_search_replace_command(&mut self, trimmed: &str, shared: &mut SharedEditorState) -> Option<bool> {
        // Handle substitute commands: s/pattern/replacement/flags, %s/pattern/replacement/flags, or range-based like 5,10s/pattern/replacement/flags
        if trimmed.starts_with("s/") || trimmed.starts_with("%s/") || self.is_range_substitute_command(trimmed) {
            // Parse range if present
            let (range_type, command_part) = self.parse_substitute_range(trimmed);
            
            // Parse s/pattern/replacement/flags format
            let parts: Vec<&str> = command_part.split('/').collect();
            if parts.len() >= 2 {
                let pattern = parts[0];
                let replacement = parts.get(1).unwrap_or(&"");
                let flags = parts.get(2).unwrap_or(&"");
                
                // Parse flags
                let global_flag = flags.contains('g');
                let case_insensitive = flags.contains('i');
                
                // Execute the substitution based on range type
                let result = match range_type {
                    RangeType::Global => {
                        // %s - substitute in entire document
                        crate::controller::search_commands::SearchReplace::substitute_document_with_flags(
                            shared.session_controller.current_document_mut(),
                            pattern,
                            replacement,
                            global_flag,
                            case_insensitive,
                        )
                    }
                    RangeType::CurrentLine => {
                        // s - substitute in current line only
                        let current_line = shared.session_controller.current_document().cursor_line();
                        crate::controller::search_commands::SearchReplace::substitute_line_with_flags(
                            shared.session_controller.current_document_mut(),
                            current_line,
                            pattern,
                            replacement,
                            global_flag,
                            case_insensitive,
                        ).map(|count| (count, if count > 0 { 1 } else { 0 }))
                    }
                    RangeType::LineRange(start, end) => {
                        // 5,10s - substitute in specified line range
                        crate::controller::search_commands::SearchReplace::substitute_range_with_flags(
                            shared.session_controller.current_document_mut(),
                            start,
                            end,
                            pattern,
                            replacement,
                            global_flag,
                            case_insensitive,
                        )
                    }
                };
                
                match result {
                    Ok((substitutions, _lines_affected)) => {
                        if substitutions > 0 {
                            if substitutions == 1 {
                                shared.status_message = "1 substitution made".to_string();
                            } else {
                                shared.status_message = format!("{} substitutions made", substitutions);
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
                let local_marks = shared.session_controller.current_document().get_all_local_marks();
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
                // Clear local marks in current document
                shared.session_controller.current_document_mut().clear_local_marks();
                // Clear global marks in mark manager
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
                        match shared.session_controller.current_document_mut().insert_text_at_cursor(&output_str) {
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

    /// Check if command is a range-based substitute command (e.g., "5,10s/old/new/")
    fn is_range_substitute_command(&self, trimmed: &str) -> bool {
        // Look for pattern like "5,10s/" or "1,$s/" 
        if let Some(s_pos) = trimmed.find("s/") {
            let before_s = &trimmed[..s_pos];
            // Check if there's a range before the 's/'
            before_s.contains(',') && !before_s.is_empty()
        } else {
            false
        }
    }

    /// Parse substitute command range and return (range_type, command_part)
    fn parse_substitute_range<'a>(&self, trimmed: &'a str) -> (RangeType, &'a str) {
        if trimmed.starts_with("%s/") {
            // %s/pattern/replacement/ - global range
            (RangeType::Global, &trimmed[3..])
        } else if trimmed.starts_with("s/") {
            // s/pattern/replacement/ - current line
            (RangeType::CurrentLine, &trimmed[2..])
        } else if let Some(s_pos) = trimmed.find("s/") {
            // Range like "5,10s/pattern/replacement/"
            let range_part = &trimmed[..s_pos];
            let command_part = &trimmed[s_pos + 2..];
            
            if let Some((start_str, end_str)) = range_part.split_once(',') {
                let start_str = start_str.trim();
                let end_str = end_str.trim();
                
                // Handle special cases like "$" for end of file
                let start = if start_str == "1" || start_str.is_empty() {
                    0  // Convert to 0-based indexing
                } else {
                    start_str.parse::<usize>().unwrap_or(1).saturating_sub(1)
                };
                
                let end = if end_str == "$" {
                    usize::MAX  // Will be clamped to document length
                } else {
                    end_str.parse::<usize>().unwrap_or(1).saturating_sub(1)
                };
                
                (RangeType::LineRange(start, end), command_part)
            } else {
                // Single number like "5s/pattern/replacement/" - treat as single line range
                if let Ok(line_num) = range_part.trim().parse::<usize>() {
                    let line_index = line_num.saturating_sub(1);
                    (RangeType::LineRange(line_index, line_index), command_part)
                } else {
                    // Fallback to current line
                    (RangeType::CurrentLine, command_part)
                }
            }
        } else {
            // Fallback to current line
            (RangeType::CurrentLine, &trimmed[2..])
        }
    }
}