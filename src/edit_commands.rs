use crate::controller::{Controller, Mode};
use arboard::Clipboard;

impl Controller {
    pub fn handle_paste_command(&mut self) {
        let clipboard_text = match Clipboard::new().and_then(|mut c| c.get_text()) {
            Ok(text) => text,
            Err(e) => {
                self.status_message = format!("Error: Could not access clipboard: {e}");
                return;
            }
        };

        // Sanitize the text - remove null bytes and other problematic characters
        let sanitized_text = self.sanitize_paste_text(&clipboard_text);

        if sanitized_text.is_empty() {
            self.status_message = "Nothing to paste".to_string();
            return;
        }

        match self.mode {
            Mode::Normal => {
                self.mode = Mode::Insert;
                self.paste_text_and_update_status(&sanitized_text);
            }
            Mode::Insert => {
                self.paste_text_and_update_status(&sanitized_text);
            }
            Mode::Command => {
                // In command mode, exit command mode and paste into document
                self.mode = Mode::Normal;
                self.command_buffer.clear();
                self.mode = Mode::Insert;
                self.paste_text_and_update_status(&sanitized_text);
            }
            Mode::Search | Mode::SearchBackward => {
                // In search mode, exit search mode and paste into document
                self.mode = Mode::Normal;
                self.command_buffer.clear();
                self.mode = Mode::Insert;
                self.paste_text_and_update_status(&sanitized_text);
            }
            Mode::VisualChar | Mode::VisualLine | Mode::VisualBlock => {
                // In visual mode, replace selection with pasted text
                self.mode = Mode::Insert;
                self.paste_text_and_update_status(&sanitized_text);
            }
        }
    }

    pub fn handle_help_command(&mut self) {
        self.buffer_manager.add_help_buffer();
        self.status_message = "Help buffer opened".to_string();
        self.view.reset_scroll();
    }

    fn sanitize_paste_text(&self, text: &str) -> String {
        // Remove trailing newlines that might be added by clipboard
        let trimmed = text.trim_end_matches('\n').trim_end_matches('\r');

        trimmed
            .chars()
            .filter(|c| {
                // Allow printable characters, tabs, and newlines
                // Filter out null bytes, control characters (except tab/newline), etc.
                *c != '\0' && (*c == '\t' || *c == '\n' || *c >= ' ')
            })
            .collect()
    }

    fn paste_text_and_update_status(&mut self, text: &str) {
        match self.current_document_mut().paste_at_cursor(text) {
            Ok(bytes) => self.status_message = format!("{bytes} bytes pasted"),
            Err(e) => self.status_message = format!("Error: Could not paste text: {e}"),
        }
    }
}
