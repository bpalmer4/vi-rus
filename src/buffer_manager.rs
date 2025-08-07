use crate::document::Document;

pub struct BufferManager {
    pub buffers: Vec<Document>,
    pub current_buffer: usize,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: vec![Document::new()],
            current_buffer: 0,
        }
    }

    pub fn new_with_files(filenames: Vec<std::path::PathBuf>) -> Result<Self, std::io::Error> {
        if filenames.is_empty() {
            return Ok(Self::new());
        }

        let mut buffers = Vec::new();
        for filename in filenames {
            match Document::from_file(filename.clone()) {
                Ok(doc) => buffers.push(doc),
                Err(_) => {
                    // Create new file if it doesn't exist
                    let mut new_doc = Document::new();
                    new_doc.filename = Some(filename);
                    buffers.push(new_doc);
                }
            }
        }

        Ok(Self {
            buffers,
            current_buffer: 0,
        })
    }

    pub fn current_document(&self) -> &Document {
        &self.buffers[self.current_buffer]
    }

    pub fn current_document_mut(&mut self) -> &mut Document {
        &mut self.buffers[self.current_buffer]
    }

    pub fn get_display_filename(&self) -> &str {
        self.current_document()
            .filename
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]")
    }

    pub fn open_file(&mut self, filename: &str) -> String {
        let path = std::path::PathBuf::from(filename);
        match Document::from_file(path.clone()) {
            Ok(doc) => {
                self.buffers.push(doc);
                self.current_buffer = self.buffers.len() - 1;
                format!("\"{filename}\" opened")
            }
            Err(_) => {
                // Create new file if it doesn't exist
                let mut new_doc = Document::new();
                new_doc.filename = Some(path);
                self.buffers.push(new_doc);
                self.current_buffer = self.buffers.len() - 1;
                format!("\"{filename}\" [New File]")
            }
        }
    }

    pub fn open_files(&mut self, filenames: Vec<&str>) -> String {
        if filenames.is_empty() {
            return "Error: No filenames specified".to_string();
        }

        let mut opened_files = Vec::new();
        let mut new_files = Vec::new();

        for filename in filenames {
            let path = std::path::PathBuf::from(filename);
            match Document::from_file(path.clone()) {
                Ok(doc) => {
                    self.buffers.push(doc);
                    opened_files.push(filename);
                }
                Err(_) => {
                    // Create new file if it doesn't exist
                    let mut new_doc = Document::new();
                    new_doc.filename = Some(path);
                    self.buffers.push(new_doc);
                    new_files.push(filename);
                }
            }
        }

        // Switch to the first newly opened buffer
        if !opened_files.is_empty() || !new_files.is_empty() {
            self.current_buffer = self.buffers.len() - (opened_files.len() + new_files.len());
        }

        // Build status message
        let mut message = String::new();
        if !opened_files.is_empty() {
            message.push_str(&format!("Opened: {}", opened_files.join(", ")));
        }
        if !new_files.is_empty() {
            if !message.is_empty() {
                message.push_str(" | ");
            }
            message.push_str(&format!("New files: {}", new_files.join(", ")));
        }
        message
    }

    pub fn list_buffers(&self) -> String {
        let mut buffer_list = String::new();
        for (i, buffer) in self.buffers.iter().enumerate() {
            let indicator = if i == self.current_buffer { "%" } else { " " };
            let filename = buffer
                .filename
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("[No Name]");
            let modified = if buffer.is_modified() { "+" } else { "" };
            buffer_list.push_str(&format!(
                "{} {}: \"{}\" {} | ",
                indicator,
                i + 1,
                filename,
                modified
            ));
        }
        // Remove trailing " | "
        if buffer_list.ends_with(" | ") {
            buffer_list.truncate(buffer_list.len() - 3);
        }
        buffer_list
    }

    pub fn next_buffer(&mut self) -> String {
        if self.buffers.len() > 1 {
            self.current_buffer = (self.current_buffer + 1) % self.buffers.len();
            let filename = self.get_display_filename();
            format!("Switched to buffer: \"{filename}\"")
        } else {
            "Only one buffer".to_string()
        }
    }

    pub fn prev_buffer(&mut self) -> String {
        if self.buffers.len() > 1 {
            self.current_buffer = if self.current_buffer == 0 {
                self.buffers.len() - 1
            } else {
                self.current_buffer - 1
            };
            let filename = self.get_display_filename();
            format!("Switched to buffer: \"{filename}\"")
        } else {
            "Only one buffer".to_string()
        }
    }

    pub fn close_buffer(&mut self) -> Result<String, String> {
        if self.buffers.len() == 1 {
            return Err("Cannot close last buffer".to_string());
        }

        let current_doc = &self.buffers[self.current_buffer];
        if current_doc.is_modified() {
            return Err(
                "Buffer has unsaved changes. Use :w to save or :bd! to force close".to_string(),
            );
        }

        self.buffers.remove(self.current_buffer);
        if self.current_buffer >= self.buffers.len() {
            self.current_buffer = self.buffers.len() - 1;
        }

        let filename = self.get_display_filename();
        Ok(format!("Buffer closed. Current: \"{filename}\""))
    }

    pub fn force_close_buffer(&mut self) -> Result<String, String> {
        if self.buffers.len() == 1 {
            return Err("Cannot close last buffer".to_string());
        }

        let filename = self.get_display_filename().to_string();
        self.buffers.remove(self.current_buffer);
        if self.current_buffer >= self.buffers.len() {
            self.current_buffer = self.buffers.len() - 1;
        }

        let new_filename = self.get_display_filename();
        Ok(format!(
            "Buffer \"{filename}\" forcibly closed. Current: \"{new_filename}\""
        ))
    }

    pub fn switch_to_buffer(&mut self, buffer_num: usize) -> Result<String, String> {
        if buffer_num > 0 && buffer_num <= self.buffers.len() {
            self.current_buffer = buffer_num - 1; // Convert to 0-based index
            let filename = self.get_display_filename();
            Ok(format!("Switched to buffer {buffer_num}: \"{filename}\""))
        } else {
            Err(format!("Buffer {buffer_num} does not exist"))
        }
    }

    pub fn add_help_buffer(&mut self) {
        let help_doc = crate::help::create_help_document();
        self.buffers.push(help_doc);
        self.current_buffer = self.buffers.len() - 1;
    }

    pub fn create_new_buffer(&mut self) -> String {
        let new_doc = Document::new();
        self.buffers.push(new_doc);
        self.current_buffer = self.buffers.len() - 1;
        "New buffer created".to_string()
    }

    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    pub fn current_buffer_index(&self) -> usize {
        self.current_buffer
    }

    /// Switch to an existing buffer with the given filename, or open it if not found
    pub fn switch_to_file(
        &mut self,
        target_filename: &std::path::PathBuf,
    ) -> Result<(), std::io::Error> {
        // First, check if the file is already open in a buffer
        for (i, buffer) in self.buffers.iter().enumerate() {
            if let Some(ref buffer_filename) = buffer.filename {
                if buffer_filename == target_filename {
                    self.current_buffer = i;
                    return Ok(());
                }
            }
        }

        // File not found in existing buffers, try to open it
        match Document::from_file(target_filename.clone()) {
            Ok(doc) => {
                self.buffers.push(doc);
                self.current_buffer = self.buffers.len() - 1;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
