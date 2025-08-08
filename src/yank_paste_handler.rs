use crate::controller::Controller;
use crate::document::Document;

// Helper function to get line count efficiently
fn get_line_count(document: &Document) -> usize {
    document.line_count()
}
use crate::registers::RegisterType;
use crate::visual_mode::VisualModeHandler;

pub struct YankPasteHandler;

#[derive(Debug, Clone)]
pub enum YankType {
    Line,
    Lines(usize),
    Word,
    BigWord,
    WordBackward,
    BigWordBackward,
    ToEndOfWord,
    ToEndOfBigWord,
    ToStartOfLine,
    ToEndOfLine,
    ToFirstNonWhitespace,
    ToEndOfFile,
    ToStartOfFile,
    UntilChar(char),
    UntilCharBackward(char),
    FindChar(char),
    FindCharBackward(char),
}

#[derive(Debug, Clone)]
pub enum PasteType {
    After,
    Before,
}

impl YankPasteHandler {
    pub fn execute_yank(controller: &mut Controller, yank_type: YankType, register: Option<char>) {
        let (text, register_type) =
            Self::get_yank_content(&yank_type, controller.current_document());

        // Store in register
        controller
            .register_manager
            .store_in_register(register, text.clone(), register_type);

        // Show feedback message
        Self::show_yank_feedback(&mut controller.status_message, &text, register);
    }

    pub fn execute_paste(
        controller: &mut Controller,
        paste_type: PasteType,
        register: Option<char>,
    ) {
        if let Some(register_data) = controller.register_manager.get_register_content(register) {
            let content = register_data.content.clone();
            let register_type = register_data.register_type.clone();

            Self::paste_content(
                controller.current_document_mut(),
                &content,
                &register_type,
                &paste_type,
            );
        }
    }

    pub fn execute_visual_yank(controller: &mut Controller, register: Option<char>) {
        if let Some(selection) = controller.visual_selection.as_ref() {
            let text =
                VisualModeHandler::get_selected_text(selection, controller.current_document());
            let register_type = match selection.mode {
                crate::visual_mode::VisualMode::Line => RegisterType::Line,
                crate::visual_mode::VisualMode::Char => RegisterType::Character,
                crate::visual_mode::VisualMode::Block => RegisterType::Block,
            };

            controller
                .register_manager
                .store_in_register(register, text.clone(), register_type);

            // Show feedback message
            Self::show_visual_yank_feedback(&mut controller.status_message, &text, register);

            // Exit visual mode after yank
            controller.visual_selection = None;
            controller.mode = crate::controller::Mode::Normal;
        }
    }

    fn get_yank_content(yank_type: &YankType, document: &Document) -> (String, RegisterType) {
        match yank_type {
            YankType::Line => (document.yank_line(), RegisterType::Line),
            YankType::Lines(count) => {
                let mut lines = Vec::new();
                for i in 0..*count {
                    let line_idx = document.cursor_line + i;
                    if line_idx < get_line_count(document) {
                        lines.push(document.get_line(line_idx).unwrap_or_default());
                    } else {
                        break;
                    }
                }
                (lines.join("\n"), RegisterType::Line)
            }
            YankType::Word => (document.yank_word_forward(), RegisterType::Character),
            YankType::BigWord => (document.yank_big_word_forward(), RegisterType::Character),
            YankType::WordBackward => (document.yank_word_backward(), RegisterType::Character),
            YankType::BigWordBackward => {
                (document.yank_big_word_backward(), RegisterType::Character)
            }
            YankType::ToEndOfWord => (document.yank_to_end_of_word(), RegisterType::Character),
            YankType::ToEndOfBigWord => {
                (document.yank_to_end_of_big_word(), RegisterType::Character)
            }
            YankType::ToStartOfLine => (document.yank_to_start_of_line(), RegisterType::Character),
            YankType::ToEndOfLine => (document.yank_to_end_of_line(), RegisterType::Character),
            YankType::ToFirstNonWhitespace => (
                document.yank_to_first_non_whitespace(),
                RegisterType::Character,
            ),
            YankType::ToEndOfFile => (document.yank_to_end_of_file(), RegisterType::Block),
            YankType::ToStartOfFile => (document.yank_to_start_of_file(), RegisterType::Block),
            YankType::UntilChar(target) => {
                (document.yank_until_char(*target), RegisterType::Character)
            }
            YankType::UntilCharBackward(target) => (
                document.yank_until_char_backward(*target),
                RegisterType::Character,
            ),
            YankType::FindChar(target) => {
                (document.yank_find_char(*target), RegisterType::Character)
            }
            YankType::FindCharBackward(target) => (
                document.yank_find_char_backward(*target),
                RegisterType::Character,
            ),
        }
    }

    fn paste_content(
        document: &mut Document,
        content: &str,
        register_type: &RegisterType,
        paste_type: &PasteType,
    ) {
        match register_type {
            RegisterType::Line => {
                Self::paste_line_wise(document, content, paste_type);
            }
            RegisterType::Character | RegisterType::Block => {
                Self::paste_character_wise(document, content, paste_type);
            }
        }
        document.modified = true;
    }

    fn paste_line_wise(document: &mut Document, content: &str, paste_type: &PasteType) {
        let lines: Vec<&str> = content.lines().collect();
        let insert_line = match paste_type {
            PasteType::After => document.cursor_line + 1,
            PasteType::Before => document.cursor_line,
        };

        for (i, line) in lines.iter().enumerate() {
            document.insert_line_at(insert_line + i, line);
        }

        // Move cursor to first line of pasted content
        document.cursor_line = insert_line;
        document.cursor_column = 0;
    }

    fn paste_character_wise(document: &mut Document, content: &str, paste_type: &PasteType) {
        if document.cursor_line < get_line_count(document) {
            let line_length = document.get_line_length(document.cursor_line);
            let mut insert_col = document.cursor_column;

            match paste_type {
                PasteType::After => {
                    if insert_col < line_length {
                        insert_col += 1; // Move cursor after current character
                    }
                }
                PasteType::Before => {
                    // Insert at current position
                }
            }

            if insert_col <= line_length {
                use crate::text_buffer::Position;
                let pos = Position::new(document.cursor_line, insert_col);
                document.text_buffer.insert(pos, content);
                document.cursor_column = insert_col + content.len() - 1;
                document.modified = true;
            }
        }
    }

    fn show_yank_feedback(status_message: &mut String, text: &str, register: Option<char>) {
        let word_count = text.split_whitespace().count();
        let line_count = text.lines().count();

        let base_message = if line_count > 1 {
            format!("{line_count} lines yanked")
        } else if word_count > 1 {
            format!("{word_count} words yanked")
        } else {
            "Text yanked".to_string()
        };

        *status_message = match register {
            Some(reg) => format!("{base_message} to register {reg}"),
            None => base_message,
        };
    }

    fn show_visual_yank_feedback(status_message: &mut String, text: &str, register: Option<char>) {
        let word_count = text.split_whitespace().count();
        let line_count = text.lines().count();

        let base_message = if line_count > 1 {
            format!("{line_count} lines yanked")
        } else if word_count > 1 {
            format!("{word_count} words yanked")
        } else {
            "Selection yanked".to_string()
        };

        *status_message = match register {
            Some(reg) => format!("{base_message} to register {reg}"),
            None => base_message,
        };
    }
}
