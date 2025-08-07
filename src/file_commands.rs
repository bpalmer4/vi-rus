use crate::controller::Controller;
use crate::document::LineEnding;
use crate::rc::RcLoader;
use crate::search::SearchReplace;
use std::fs;
use std::process::Command as ProcessCommand;

impl Controller {
    pub fn handle_save_command(&mut self) -> bool {
        match self.current_document_mut().save() {
            Ok(bytes) => {
                let filename = self.get_display_filename();
                self.status_message = format!("\"{filename}\" {bytes}B written");
            }
            Err(_) => {
                self.status_message = "Error: Could not save file".to_string();
            }
        }
        false
    }

    pub fn handle_save_and_quit_command(&mut self) -> bool {
        match self.current_document_mut().save() {
            Ok(_) => true,
            Err(_) => {
                self.status_message = "Error: Could not save file".to_string();
                false
            }
        }
    }

    pub fn handle_file_info_command(&mut self) -> bool {
        let doc = self.current_document();
        let filename = self.get_display_filename();
        let current_line = doc.cursor_line + 1;
        let total_lines = doc.lines.len();
        let percentage = if total_lines > 0 {
            (current_line * 100) / total_lines
        } else {
            0
        };
        let modified_flag = if doc.is_modified() { " [Modified]" } else { "" };
        let new_file_flag = if doc.filename.is_none() {
            " [New File]"
        } else {
            ""
        };
        let buffer_info = format!(
            "Buffer {}/{}",
            self.buffer_manager.current_buffer_index() + 1,
            self.buffer_manager.buffer_count()
        );

        self.status_message = format!(
            "{} \"{}\" line {} of {} --{}%--{}{} [{}]",
            buffer_info,
            filename,
            current_line,
            total_lines,
            percentage,
            modified_flag,
            new_file_flag,
            doc.line_ending.name()
        );
        false
    }

    pub fn handle_save_as_command(&mut self, cmd: &str) -> bool {
        let filename = cmd[2..].trim();
        if !filename.is_empty() {
            let path = std::path::PathBuf::from(filename);
            match self.current_document_mut().save_as(path) {
                Ok(bytes) => {
                    self.status_message = format!("\"{filename}\" {bytes}B written");
                }
                Err(_) => {
                    self.status_message = "Error: Could not save file".to_string();
                }
            }
        }
        false
    }

    pub fn handle_set_command(&mut self, cmd: &str) -> bool {
        let trimmed = cmd[4..].trim();
        match trimmed {
            "ff=unix" | "ff=linux" => {
                self.current_document_mut()
                    .set_line_ending(LineEnding::Unix);
                self.status_message = "Line endings set to unix".to_string();
            }
            "ff=dos" | "ff=win" => {
                self.current_document_mut()
                    .set_line_ending(LineEnding::Windows);
                self.status_message = "Line endings set to dos".to_string();
            }
            "ff=mac" => {
                self.current_document_mut().set_line_ending(LineEnding::Mac);
                self.status_message = "Line endings set to mac".to_string();
            }
            "number" | "nu" => {
                self.view.set_line_numbers(true);
                self.status_message = "Line numbers enabled".to_string();
            }
            "nonumber" | "nonu" => {
                self.view.set_line_numbers(false);
                self.status_message = "Line numbers disabled".to_string();
            }
            "expandtab" | "et" => {
                self.current_document_mut().set_expand_tab(true);
                self.status_message = "Expand tab enabled (Tab inserts spaces)".to_string();
            }
            "noexpandtab" | "noet" => {
                self.current_document_mut().set_expand_tab(false);
                self.status_message = "Expand tab disabled (Tab inserts tabs)".to_string();
            }
            "list" => {
                self.view.set_show_whitespace(true);
                self.status_message = "Whitespace display enabled".to_string();
            }
            "nolist" => {
                self.view.set_show_whitespace(false);
                self.status_message = "Whitespace display disabled".to_string();
            }
            _ => {
                // Handle tabstop=N settings
                if let Some(equals_pos) = trimmed.find('=') {
                    let (setting, value) = trimmed.split_at(equals_pos);
                    let value = &value[1..]; // Remove '='

                    match setting {
                        "tabstop" | "ts" => {
                            if let Ok(tab_stop) = value.parse::<usize>() {
                                if tab_stop > 0 && tab_stop <= 16 {
                                    self.view.set_tab_stop(tab_stop);
                                    self.status_message = format!("Tab stop set to {tab_stop}");
                                } else {
                                    self.status_message =
                                        "Tab stop must be between 1 and 16".to_string();
                                }
                            } else {
                                self.status_message = "Invalid tab stop value".to_string();
                            }
                        }
                        _ => {
                            self.status_message = "Unknown setting".to_string();
                        }
                    }
                } else {
                    self.status_message = "Unknown setting".to_string();
                }
            }
        }
        false
    }

    pub fn handle_read_command(&mut self, cmd: &str) -> bool {
        let rest = cmd[2..].trim();
        if rest.starts_with('!') {
            self.execute_and_insert_at_cursor(rest.strip_prefix('!').unwrap())
        } else {
            self.read_file_at_cursor(rest)
        }
    }

    pub fn handle_read_at_line_command(&mut self, cmd: &str, line_num: usize) -> bool {
        let rest = cmd[3..].trim();
        if rest.starts_with('!') {
            self.execute_and_insert_at_line(rest.strip_prefix('!').unwrap(), line_num)
        } else {
            self.read_file_at_line(rest, line_num)
        }
    }

    pub fn handle_read_at_end_command(&mut self, cmd: &str) -> bool {
        let rest = cmd[3..].trim();
        let line_num = self.current_document_mut().lines.len();
        if rest.starts_with('!') {
            self.execute_and_insert_at_line(rest.strip_prefix('!').unwrap(), line_num)
        } else {
            self.read_file_at_line(rest, line_num)
        }
    }

    pub fn handle_numbered_read_command(&mut self, cmd: &str) -> bool {
        if let Some(r_pos) = cmd.find('r') {
            if r_pos > 0 && cmd[..r_pos].chars().all(|c| c.is_ascii_digit()) {
                if let Ok(line_num) = cmd[..r_pos].parse::<usize>() {
                    let rest = cmd[r_pos + 1..].trim();
                    if rest.starts_with('!') {
                        return self.execute_and_insert_at_line(rest.strip_prefix('!').unwrap(), line_num);
                    } else {
                        return self.read_file_at_line(rest, line_num);
                    }
                }
            }
        }
        false
    }

    fn execute_and_insert_at_cursor(&mut self, command: &str) -> bool {
        match self.execute_shell_command(command) {
            Ok(output) => match self.current_document_mut().insert_text_at_cursor(&output) {
                Ok(bytes) => self.status_message = format!("{bytes} bytes inserted"),
                Err(_) => self.status_message = "Error: Could not insert output".to_string(),
            },
            Err(_) => self.status_message = "Error: Command execution failed".to_string(),
        }
        false
    }

    fn execute_and_insert_at_line(&mut self, command: &str, line_num: usize) -> bool {
        match self.execute_shell_command(command) {
            Ok(output) => {
                match self
                    .current_document_mut()
                    .insert_text_at_line(&output, line_num)
                {
                    Ok(bytes) => {
                        let location = match line_num {
                            0 => "at top".to_string(),
                            n if n == self.current_document_mut().lines.len() => {
                                "at end".to_string()
                            }
                            n => format!("after line {n}"),
                        };
                        self.status_message = format!("{bytes} bytes inserted {location}");
                    }
                    Err(_) => self.status_message = "Error: Could not insert output".to_string(),
                }
            }
            Err(_) => self.status_message = "Error: Command execution failed".to_string(),
        }
        false
    }

    fn read_file_at_cursor(&mut self, filename: &str) -> bool {
        let path = std::path::Path::new(filename);
        match self.current_document_mut().insert_file_at_cursor(path) {
            Ok(bytes) => self.status_message = format!("\"{filename}\" {bytes}B inserted"),
            Err(_) => self.status_message = format!("Error: Could not read file \"{filename}\""),
        }
        false
    }

    fn read_file_at_line(&mut self, filename: &str, line_num: usize) -> bool {
        let path = std::path::Path::new(filename);
        match self
            .current_document_mut()
            .insert_file_at_line(path, line_num)
        {
            Ok(bytes) => {
                let location = match line_num {
                    0 => "at top".to_string(),
                    n if n == self.current_document_mut().lines.len() => "at end".to_string(),
                    n => format!("after line {n}"),
                };
                self.status_message = format!("\"{filename}\" {bytes}B inserted {location}");
            }
            Err(_) => self.status_message = format!("Error: Could not read file \"{filename}\""),
        }
        false
    }

    fn execute_shell_command(&self, command: &str) -> Result<String, std::io::Error> {
        let output = if cfg!(target_os = "windows") {
            ProcessCommand::new("cmd").args(["/C", command]).output()?
        } else {
            ProcessCommand::new("sh").args(["-c", command]).output()?
        };

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(std::io::Error::other(
                format!("Command failed: {error_msg}"),
            ))
        }
    }

    pub fn handle_substitute_command(&mut self, cmd: &str) {
        // Parse :s/pattern/replacement/flags command
        if let Some(parsed) = self.parse_substitute_command(cmd) {
            let current_line = self.current_document().cursor_line;
            match SearchReplace::substitute_document(
                self.current_document_mut(),
                current_line,
                current_line,
                &parsed.pattern,
                &parsed.replacement,
                parsed.global,
                parsed.case_sensitive,
            ) {
                Ok(count) => {
                    if count > 0 {
                        self.status_message = format!("{} substitution{} on line {}", 
                            count, 
                            if count == 1 { "" } else { "s" },
                            current_line + 1
                        );
                    } else {
                        self.status_message = format!("Pattern not found: {}", parsed.pattern);
                    }
                }
                Err(e) => {
                    self.status_message = e.to_string();
                }
            }
        } else {
            self.status_message = "Invalid substitute syntax. Use :s/pattern/replacement/flags".to_string();
        }
    }

    pub fn handle_substitute_all_command(&mut self, cmd: &str) {
        let cmd_to_parse = if cmd == "%s" {
            // Handle bare %s by using last search pattern
            if self.search_state.pattern.is_empty() {
                self.status_message = "No previous search pattern".to_string();
                return;
            }
            format!("%s/{}/&/g", self.search_state.pattern) // Use & as replacement (keeps original)
        } else {
            cmd.to_string()
        };

        // Parse :%s/pattern/replacement/flags command
        if let Some(parsed) = self.parse_substitute_command(&cmd_to_parse.replacen("%s", "s", 1)) {
            match SearchReplace::substitute_all_document(
                self.current_document_mut(),
                &parsed.pattern,
                &parsed.replacement,
                parsed.case_sensitive,
            ) {
                Ok(count) => {
                    if count > 0 {
                        self.status_message = format!("{} substitution{} across {} line{}", 
                            count, 
                            if count == 1 { "" } else { "s" },
                            self.current_document().lines.len(),
                            if self.current_document().lines.len() == 1 { "" } else { "s" }
                        );
                    } else {
                        self.status_message = format!("Pattern not found: {}", parsed.pattern);
                    }
                }
                Err(e) => {
                    self.status_message = e.to_string();
                }
            }
        } else {
            self.status_message = "Invalid substitute syntax. Use :%s/pattern/replacement/flags".to_string();
        }
    }

    fn parse_substitute_command(&self, cmd: &str) -> Option<SubstituteArgs> {
        // Handle s/pattern/replacement/flags format
        if !cmd.starts_with("s/") {
            return None;
        }

        let cmd = &cmd[2..]; // Remove "s/"
        let parts: Vec<&str> = cmd.split('/').collect();

        if parts.len() < 2 {
            return None;
        }

        let pattern = parts[0].to_string();
        let replacement = parts.get(1).unwrap_or(&"").to_string();
        let flags = parts.get(2).unwrap_or(&"").to_string();

        // Parse flags
        let global = flags.contains('g');
        let case_sensitive = !flags.contains('i'); // Default to case sensitive unless 'i' flag

        Some(SubstituteArgs {
            pattern,
            replacement,
            global,
            case_sensitive,
        })
    }

    pub fn handle_generate_rc_command(&mut self) {
        let sample_rc = RcLoader::generate_sample_rc();
        match fs::write(".virusrc", sample_rc) {
            Ok(()) => {
                self.status_message = "Generated .virusrc in current directory".to_string();
            }
            Err(_) => {
                self.status_message = "Error: Could not create .virusrc file".to_string();
            }
        }
    }

    pub fn handle_marks_command(&mut self) {
        let local_marks = self.current_document().get_all_local_marks();
        let marks = self.mark_manager.list_marks(local_marks);
        if marks.is_empty() {
            self.status_message = "No marks set".to_string();
        } else {
            let mut marks_info = String::from("Marks: ");
            for (i, (mark_char, line, column, filename)) in marks.iter().enumerate() {
                if i > 0 {
                    marks_info.push_str(", ");
                }
                if let Some(filename) = filename {
                    marks_info.push_str(&format!("'{}' {}:{} {}", mark_char, line + 1, column + 1, filename.display()));
                } else {
                    marks_info.push_str(&format!("'{}' {}:{}", mark_char, line + 1, column + 1));
                }
            }
            self.status_message = marks_info;
        }
    }

    pub fn handle_clear_command(&mut self, cmd: &str) {
        let args = cmd.strip_prefix("clear ").unwrap_or("").trim();
        
        match args {
            "marks" => {
                // Clear both local marks (current document) and global marks (mark manager)
                self.current_document_mut().clear_local_marks();
                self.mark_manager.clear_global_marks();
                self.status_message = "All marks cleared".to_string();
            }
            "jumps" => {
                self.mark_manager.clear_jump_list();
                self.status_message = "Jump list cleared".to_string();
            }
            "all" => {
                // Clear local marks, global marks, and jump list
                self.current_document_mut().clear_local_marks();
                self.mark_manager.clear_all_marks();
                self.mark_manager.clear_jump_list();
                self.status_message = "All marks and jumps cleared".to_string();
            }
            "" => {
                // Just ":clear" without args - force redraw (like :redraw)
                self.view.force_redraw();
                self.status_message = "Screen cleared".to_string();
            }
            _ => {
                self.status_message = format!("Unknown clear target: {args}. Use 'marks', 'jumps', or 'all'");
            }
        }
    }

    pub fn handle_jumps_command(&mut self) {
        let (jump_list, current_position) = self.mark_manager.get_jump_list();
        
        if jump_list.is_empty() {
            self.status_message = "No jumps".to_string();
        } else {
            let mut jumps_info = String::from("Jumps: ");
            for (i, entry) in jump_list.iter().enumerate() {
                if i > 0 {
                    jumps_info.push_str(", ");
                }
                
                // Mark current position with >
                let marker = if i == current_position.saturating_sub(1) && current_position > 0 {
                    ">"
                } else {
                    " "
                };
                
                let filename = entry.filename
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("~");
                
                jumps_info.push_str(&format!("{}{}:{} {}", marker, entry.line + 1, entry.column + 1, filename));
            }
            self.status_message = jumps_info;
        }
    }
}

struct SubstituteArgs {
    pattern: String,
    replacement: String,
    global: bool,
    case_sensitive: bool,
}
