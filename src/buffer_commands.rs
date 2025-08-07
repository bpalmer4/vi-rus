use crate::controller::Controller;

impl Controller {
    pub fn handle_new_buffer_command(&mut self) {
        self.status_message = self.buffer_manager.create_new_buffer();
        self.view.reset_scroll();
    }

    pub fn handle_edit_command(&mut self, cmd: &str) {
        let args = cmd[2..].trim();
        if args.is_empty() {
            self.status_message = "Error: No filename specified".to_string();
            return;
        }

        // Split by whitespace to get multiple filenames
        let filenames: Vec<&str> = args.split_whitespace().collect();

        if filenames.len() == 1 {
            // Single file - use existing method for compatibility
            self.status_message = self.buffer_manager.open_file(filenames[0]);
        } else {
            // Multiple files - use new method
            self.status_message = self.buffer_manager.open_files(filenames);
        }

        self.view.reset_scroll();
    }

    pub fn handle_list_buffers_command(&mut self) {
        self.status_message = self.buffer_manager.list_buffers();
    }

    pub fn handle_next_buffer_command(&mut self) {
        self.status_message = self.buffer_manager.next_buffer();
        self.view.reset_scroll();
    }

    pub fn handle_prev_buffer_command(&mut self) {
        self.status_message = self.buffer_manager.prev_buffer();
        self.view.reset_scroll();
    }

    pub fn handle_close_buffer_command(&mut self) {
        // Get the filename of the buffer being closed before closing it
        let closing_filename = self.current_document().filename.clone();
        
        match self.buffer_manager.close_buffer() {
            Ok(msg) => {
                // Clean up marks associated with the closed buffer
                self.mark_manager.cleanup_for_closed_buffer(closing_filename.as_ref());
                self.status_message = msg;
            }
            Err(msg) => self.status_message = msg,
        }
    }

    pub fn handle_force_close_buffer_command(&mut self) {
        // Get the filename of the buffer being closed before closing it
        let closing_filename = self.current_document().filename.clone();
        
        match self.buffer_manager.force_close_buffer() {
            Ok(msg) => {
                // Clean up marks associated with the closed buffer
                self.mark_manager.cleanup_for_closed_buffer(closing_filename.as_ref());
                self.status_message = msg;
            }
            Err(msg) => self.status_message = msg,
        }
    }

    pub fn handle_buffer_switch_command(&mut self, cmd: &str) {
        let buffer_spec = &cmd[1..]; // Remove 'b' prefix

        if let Ok(buffer_num) = buffer_spec.parse::<usize>() {
            match self.buffer_manager.switch_to_buffer(buffer_num) {
                Ok(msg) => {
                    self.status_message = msg;
                    self.view.reset_scroll();
                }
                Err(msg) => self.status_message = msg,
            }
        } else {
            self.status_message = "Invalid buffer number".to_string();
        }
    }
}
