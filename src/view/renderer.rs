use crate::controller::Mode;
use crate::document_model::SearchState;
use crate::controller::Selection;
use super::view_model::{ViewModel, BracketHighlight};
use crossterm::{
    cursor, execute,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType, size},
};
use std::io::{self, Write, stdout};
use unicode_width::UnicodeWidthChar;

#[derive(Clone)]
pub struct RenderParams<'a> {
    pub mode: &'a Mode,
    pub command_buffer: &'a str,
    pub status_message: &'a str,
    pub buffer_info: Option<&'a str>,
    pub visual_selection: Option<&'a Selection>,
    pub search_state: Option<&'a SearchState>,
    pub bracket_highlights: Option<&'a BracketHighlight>,
}

pub struct View {
    last_lines: Vec<String>,
    last_buffer_info: Option<String>,
    last_status: String,
    last_mode: Mode,
    last_command_buffer: String,
    last_cursor_pos: (usize, usize),
    last_terminal_size: (u16, u16),
    scroll_offset: usize,
    horizontal_scroll: usize,
    needs_full_redraw: bool,
    render_count: usize,
    show_line_numbers: bool,
    tab_stop: usize,
    show_whitespace: bool,
}

impl View {
    pub fn new() -> Self {
        Self {
            last_lines: Vec::new(),
            last_buffer_info: None,
            last_status: String::new(),
            last_mode: Mode::Normal,
            last_command_buffer: String::new(),
            last_cursor_pos: (0, 0),
            last_terminal_size: (0, 0),
            scroll_offset: 0,
            horizontal_scroll: 0,
            needs_full_redraw: true,
            render_count: 0,
            show_line_numbers: false,
            tab_stop: 4, // default to 4 spaces
            show_whitespace: false,
        }
    }

    fn clear_screen(&self) -> io::Result<()> {
        execute!(stdout(), Clear(ClearType::All))
    }

    fn move_cursor(&self, line: usize, column: usize) -> io::Result<()> {
        execute!(stdout(), cursor::MoveTo(column as u16, line as u16))
    }

    fn apply_highlighting(
        &self,
        text: &str,
        line_idx: usize,
        cursor_line: usize,
        cursor_col: usize,
        horizontal_scroll: usize,
        search_state: Option<&SearchState>,
        bracket_highlights: Option<&BracketHighlight>,
    ) -> String {
        let mut result = String::new();
        let chars: Vec<char> = text.chars().collect();

        for (i, ch) in chars.iter().enumerate() {
            let actual_col = horizontal_scroll + i;
            let mut highlighted = false;

            // Search highlighting
            if let Some(search) = search_state {
                if !search.matches.is_empty() {
                    for search_match in &search.matches {
                        if search_match.line == line_idx
                            && actual_col >= search_match.start_col
                            && actual_col < search_match.end_col
                        {
                            if actual_col == search_match.start_col {
                                // Start highlight
                                result.push_str(&format!(
                                    "{}{}",
                                    SetBackgroundColor(Color::Yellow),
                                    SetForegroundColor(Color::Black)
                                ));
                            }
                            result.push(*ch);
                            if actual_col == search_match.end_col - 1 {
                                // End highlight
                                result.push_str(&format!("{ResetColor}"));
                            }
                            highlighted = true;
                            break;
                        }
                    }
                }
            }

            // Bracket highlighting
            if !highlighted {
                let is_cursor_bracket = line_idx == cursor_line
                    && actual_col == cursor_col
                    && matches!(*ch, '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>');

                let is_matching_bracket = if let Some(highlights) = bracket_highlights {
                    if let Some((match_line, match_col)) = highlights.matching {
                        line_idx == match_line
                            && actual_col == match_col
                            && matches!(*ch, '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>')
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Check if this position is an unmatched bracket (cursor-specific)
                let is_cursor_unmatched_bracket = if let Some(highlights) = bracket_highlights {
                    if let Some((unmatch_line, unmatch_col)) = highlights.unmatched_at_cursor {
                        line_idx == unmatch_line
                            && actual_col == unmatch_col
                            && matches!(*ch, '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>')
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Check if this position is in the list of all unmatched brackets
                let is_all_unmatched_bracket = if let Some(highlights) = bracket_highlights {
                    highlights.all_unmatched.iter().any(|(unmatch_line, unmatch_col)| {
                        line_idx == *unmatch_line
                            && actual_col == *unmatch_col
                            && matches!(*ch, '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>')
                    })
                } else {
                    false
                };

                if is_cursor_unmatched_bracket || is_all_unmatched_bracket {
                    // Highlight unmatched brackets with red background
                    result.push_str(&format!(
                        "{}{}{}{}",
                        SetBackgroundColor(Color::Red),
                        SetForegroundColor(Color::White),
                        ch,
                        ResetColor
                    ));
                    highlighted = true;
                } else if is_cursor_bracket || is_matching_bracket {
                    // Highlight matched brackets with cyan background
                    result.push_str(&format!(
                        "{}{}{}{}",
                        SetBackgroundColor(Color::Cyan),
                        SetForegroundColor(Color::Black),
                        ch,
                        ResetColor
                    ));
                    highlighted = true;
                }
            }

            if !highlighted {
                result.push(*ch);
            }
        }

        result
    }

    pub fn render<'a>(&mut self, view_model: &dyn ViewModel, params: &RenderParams<'a>) -> io::Result<()> {
        let (width, height) = size()?;
        let start_line = if params.buffer_info.is_some() {
            1usize
        } else {
            0usize
        };

        self.render_count += 1;

        // Force full redraw every 50 renders to prevent state drift
        if self.render_count % 50 == 0 {
            self.needs_full_redraw = true;
        }

        // Check if terminal size changed
        let current_size = (width, height);
        if self.last_terminal_size != current_size {
            self.needs_full_redraw = true;
            self.last_terminal_size = current_size;
            // Clear cached state since terminal dimensions changed
            self.last_lines.clear();
            self.last_buffer_info = None;
            self.last_status.clear();
            self.last_cursor_pos = (0, 0);
            // Scrolling will be adjusted below to keep cursor visible
        }

        // Force full redraw on mode changes to ensure clean state
        if self.last_mode != *params.mode {
            self.needs_full_redraw = true;
        }

        // Check if we need a full redraw
        if self.needs_full_redraw {
            self.clear_screen()?;
            self.needs_full_redraw = false;
            // Clear all cached state on full redraw
            self.last_lines.clear();
            self.last_buffer_info = None;
            self.last_status.clear();
            self.last_cursor_pos = (0, 0);
        }

        // Update buffer info if changed
        if self.last_buffer_info.as_deref() != params.buffer_info {
            self.move_cursor(0, 0)?;
            execute!(stdout(), Clear(ClearType::CurrentLine))?;
            if let Some(ref info) = params.buffer_info {
                let clipped_info = if info.len() > width as usize {
                    &info[..width as usize]
                } else {
                    info
                };
                print!("{clipped_info}");
            }
            self.last_buffer_info = params.buffer_info.map(|s| s.to_string());
        }

        // Calculate visible area dimensions
        let max_lines = if height > (1 + start_line as u16) {
            (height - 1 - start_line as u16) as usize
        } else {
            0
        };

        // Calculate line number width and text offset
        let line_num_width = if self.show_line_numbers {
            // Calculate width needed for line numbers (based on total lines)
            let total_lines = view_model.get_line_count();
            if total_lines == 0 {
                4
            } else {
                (total_lines.to_string().len() + 1).max(4)
            }
        } else {
            0
        };

        // Adjust available width for text
        let text_width = if width as usize > line_num_width {
            width as usize - line_num_width
        } else {
            1 // Minimum width
        };

        // Adjust scrolling to keep cursor visible
        self.adjust_scroll_to_cursor(view_model, max_lines, text_width);

        // Get visible lines with scrolling applied
        let visible_lines: Vec<String> = (0..max_lines)
            .map(|i| {
                let actual_line_num = self.scroll_offset + i + 1;
                let doc_line_idx = self.scroll_offset + i;
                let line_num_str = if self.show_line_numbers {
                    format!("{:>width$} ", actual_line_num, width = line_num_width - 1)
                } else {
                    String::new()
                };

                // Get the line from document
                let line = if doc_line_idx < view_model.get_line_count() {
                    view_model.get_line(doc_line_idx).unwrap_or_default()
                } else {
                    String::new()
                };

                // Apply horizontal scrolling to the text part
                let line_start = std::cmp::min(self.horizontal_scroll, line.len());
                let line_end = std::cmp::min(line_start + text_width, line.len());
                let mut text_part = if line_start < line.len() {
                    line[line_start..line_end].to_string()
                } else {
                    String::new()
                };

                // Show whitespace if enabled (before highlighting)
                if self.show_whitespace {
                    text_part = text_part
                        .replace('\t', &format!(">{}", "─".repeat(self.tab_stop - 1)))
                        .replace(' ', "·");
                }

                // Apply search and bracket highlighting
                text_part = self.apply_highlighting(
                    &text_part,
                    doc_line_idx,
                    view_model.get_cursor_position().line,
                    view_model.get_cursor_position().column,
                    self.horizontal_scroll,
                    params.search_state,
                    params.bracket_highlights,
                );

                // Add visual selection indicator only when in visual mode
                let line_marker = if let Some(selection) = params.visual_selection {
                    if selection.is_line_in_selection(doc_line_idx) {
                        ">" // Simple indicator for selected lines
                    } else {
                        " " // Space to maintain alignment when in visual mode
                    }
                } else {
                    "" // No marker when not in visual mode
                };

                format!("{line_marker}{line_num_str}{text_part}")
            })
            .collect();

        if self.last_lines != visible_lines {
            // Only redraw changed lines
            for (i, line) in visible_lines.iter().enumerate() {
                if i >= self.last_lines.len() || self.last_lines[i] != *line {
                    self.move_cursor(i + start_line, 0)?;
                    execute!(stdout(), Clear(ClearType::CurrentLine))?;
                    print!("{line}");
                }
            }

            // Clear any extra lines if the new content is shorter
            if visible_lines.len() < self.last_lines.len() {
                for i in visible_lines.len()..self.last_lines.len() {
                    if i + start_line < (height - 1) as usize {
                        self.move_cursor(i + start_line, 0)?;
                        execute!(stdout(), Clear(ClearType::CurrentLine))?;
                    }
                }
            }

            self.last_lines = visible_lines;
        }

        // Update status line if changed
        let current_status = match *params.mode {
            Mode::Normal => {
                if !params.status_message.is_empty() {
                    params.status_message.to_string()
                } else {
                    "-- NORMAL --".to_string()
                }
            }
            Mode::Insert => "-- INSERT --".to_string(),
            Mode::Command => format!(":{}", params.command_buffer),
            Mode::Search => format!("/{}", params.command_buffer),
            Mode::SearchBackward => format!("?{}", params.command_buffer),
            Mode::VisualChar => "-- VISUAL --".to_string(),
            Mode::VisualLine => "-- VISUAL LINE --".to_string(),
            Mode::VisualBlock => "-- VISUAL BLOCK --".to_string(),
        };

        if self.last_status != current_status
            || self.last_mode != *params.mode
            || self.last_command_buffer != params.command_buffer
        {
            self.move_cursor((height - 1) as usize, 0)?;
            execute!(stdout(), Clear(ClearType::CurrentLine))?;
            let clipped_status = if current_status.len() > width as usize {
                &current_status[..width as usize]
            } else {
                &current_status
            };
            print!("{clipped_status}");
            self.last_status = current_status;
            self.last_mode = *params.mode;
            self.last_command_buffer = params.command_buffer.to_string();
        }

        // Update cursor position if changed (adjusted for scrolling and line numbers)
        let new_cursor_pos = match &self.last_mode {
            Mode::Normal
            | Mode::Insert
            | Mode::VisualChar
            | Mode::VisualLine
            | Mode::VisualBlock => {
                let cursor_pos = view_model.get_cursor_position();
                let screen_line = cursor_pos.line.saturating_sub(self.scroll_offset) + start_line;
                
                // Convert logical cursor position to display column position
                let line_content = view_model.get_line(cursor_pos.line).unwrap_or_default();
                let display_column = self.calculate_display_column(&line_content, cursor_pos.column);
                let screen_column = display_column.saturating_sub(self.horizontal_scroll) + line_num_width;
                
                (screen_line, screen_column)
            }
            Mode::Command => ((height - 1) as usize, self.last_command_buffer.len() + 1),
            Mode::Search => ((height - 1) as usize, self.last_command_buffer.len() + 1),
            Mode::SearchBackward => ((height - 1) as usize, self.last_command_buffer.len() + 1),
        };

        if self.last_cursor_pos != new_cursor_pos {
            self.move_cursor(new_cursor_pos.0, new_cursor_pos.1)?;
            self.last_cursor_pos = new_cursor_pos;
        }

        stdout().flush()?;
        Ok(())
    }

    pub fn force_redraw(&mut self) {
        self.needs_full_redraw = true;
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
        self.horizontal_scroll = 0;
        self.needs_full_redraw = true;
    }

    pub fn set_line_numbers(&mut self, show: bool) {
        if self.show_line_numbers != show {
            self.show_line_numbers = show;
            self.needs_full_redraw = true;
        }
    }

    pub fn set_tab_stop(&mut self, tab_stop: usize) {
        if self.tab_stop != tab_stop {
            self.tab_stop = tab_stop;
            self.needs_full_redraw = true;
        }
    }

    pub fn set_show_whitespace(&mut self, show: bool) {
        if self.show_whitespace != show {
            self.show_whitespace = show;
            self.needs_full_redraw = true;
        }
    }

    pub fn get_tab_stop(&self) -> usize {
        self.tab_stop
    }

    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn get_visible_lines_count(&self) -> usize {
        // Calculate visible lines based on terminal height
        let (_, height) = crossterm::terminal::size().unwrap_or((80, 24));
        let start_line = if self.show_line_numbers { 1 } else { 0 };
        if height > (1 + start_line as u16) {
            (height - 1 - start_line as u16) as usize
        } else {
            0
        }
    }

    fn adjust_scroll_to_cursor(&mut self, view_model: &dyn ViewModel, visible_lines: usize, width: usize) {
        let cursor_pos = view_model.get_cursor_position();
        let cursor_line = cursor_pos.line;
        let cursor_column = cursor_pos.column;

        // Adjust vertical scrolling
        if cursor_line < self.scroll_offset {
            // Cursor is above visible area - scroll up
            self.scroll_offset = cursor_line;
            self.needs_full_redraw = true;
        } else if cursor_line >= self.scroll_offset + visible_lines {
            // Cursor is below visible area - scroll down
            self.scroll_offset = cursor_line - visible_lines + 1;
            self.needs_full_redraw = true;
        }

        // Adjust horizontal scrolling
        if cursor_column < self.horizontal_scroll {
            // Cursor is left of visible area - scroll left
            self.horizontal_scroll = cursor_column;
            self.needs_full_redraw = true;
        } else if cursor_column >= self.horizontal_scroll + width {
            // Cursor is right of visible area - scroll right
            self.horizontal_scroll = cursor_column - width + 1;
            self.needs_full_redraw = true;
        }
    }

    /// Convert logical character position to display column position
    /// Accounts for tab expansion and Unicode character widths
    fn calculate_display_column(&self, text: &str, logical_pos: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        let mut display_col = 0;
        
        for i in 0..logical_pos.min(chars.len()) {
            match chars[i] {
                '\t' => {
                    // Move to next tab stop
                    display_col = ((display_col / self.tab_stop) + 1) * self.tab_stop;
                }
                c => {
                    // Use unicode-width crate for proper Unicode handling
                    display_col += c.width().unwrap_or(1);
                }
            }
        }
        display_col
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_display_column() {
        let view = View::new();
        
        // Test ASCII characters
        assert_eq!(view.calculate_display_column("abc", 0), 0);
        assert_eq!(view.calculate_display_column("abc", 1), 1);
        assert_eq!(view.calculate_display_column("abc", 3), 3);
        
        // Test tab characters (default tab_stop = 4)
        assert_eq!(view.calculate_display_column("a\tb", 0), 0); // 'a' at position 0
        assert_eq!(view.calculate_display_column("a\tb", 1), 1); // tab at position 1
        assert_eq!(view.calculate_display_column("a\tb", 2), 4); // 'b' at position 4 (after tab)
        
        // Test tab alignment
        assert_eq!(view.calculate_display_column("\t", 1), 4);   // tab from 0 goes to 4
        assert_eq!(view.calculate_display_column("a\t", 2), 4);  // tab from 1 goes to 4
        assert_eq!(view.calculate_display_column("ab\t", 3), 4); // tab from 2 goes to 4
        assert_eq!(view.calculate_display_column("abc\t", 4), 4); // tab from 3 goes to 4
        assert_eq!(view.calculate_display_column("abcd\t", 5), 8); // tab from 4 goes to 8
        
        // Test Unicode characters
        assert_eq!(view.calculate_display_column("a中b", 0), 0); // 'a'
        assert_eq!(view.calculate_display_column("a中b", 1), 1); // '中' starts at 1
        assert_eq!(view.calculate_display_column("a中b", 2), 3); // 'b' at 3 (中 is 2 wide)
        
        // Test emojis (wide characters)
        assert_eq!(view.calculate_display_column("a😀b", 0), 0); // 'a'
        assert_eq!(view.calculate_display_column("a😀b", 1), 1); // '😀' starts at 1
        assert_eq!(view.calculate_display_column("a😀b", 2), 3); // 'b' at 3 (😀 is 2 wide)
    }
}
