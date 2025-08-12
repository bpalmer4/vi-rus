use super::document::LineEnding;
use super::piece_table::PieceTable;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

pub struct TextBuffer {
    piece_table: PieceTable,
    line_ending: LineEnding,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            piece_table: PieceTable::new(),
            line_ending: LineEnding::system_default(),
        }
    }

    pub fn from_string(content: String) -> Self {
        let line_ending = LineEnding::detect(&content);
        let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
        
        Self {
            piece_table: PieceTable::from_string(normalized),
            line_ending,
        }
    }


    pub fn insert(&mut self, pos: Position, text: &str) {
        let offset = self.piece_table.position_to_offset(pos.line, pos.column);
        self.piece_table.insert(offset, text);
    }

    pub fn delete(&mut self, range: Range) {
        let start_offset = self.piece_table.position_to_offset(range.start.line, range.start.column);
        let end_offset = self.piece_table.position_to_offset(range.end.line, range.end.column);
        let length = end_offset - start_offset;
        self.piece_table.delete(start_offset, length);
    }

    pub fn delete_char(&mut self, pos: Position) {
        let offset = self.piece_table.position_to_offset(pos.line, pos.column);
        self.piece_table.delete(offset, 1);
    }

    pub fn insert_newline(&mut self, pos: Position) {
        self.insert(pos, "\n");
    }

    pub fn get_text(&self) -> String {
        let text = self.piece_table.get_text();
        match self.line_ending {
            LineEnding::Unix => text,
            LineEnding::Windows => text.replace('\n', "\r\n"),
            LineEnding::Mac => text.replace('\n', "\r"),
        }
    }

    pub fn get_text_range(&mut self, range: Range) -> String {
        let start_offset = self.piece_table.position_to_offset(range.start.line, range.start.column);
        let end_offset = self.piece_table.position_to_offset(range.end.line, range.end.column);
        self.piece_table.substring(start_offset, end_offset)
    }

    pub fn get_line(&mut self, line_number: usize) -> Option<String> {
        self.piece_table.get_line_fast(line_number)
    }

    pub fn get_lines(&mut self) -> Vec<String> {
        (0..self.line_count())
            .filter_map(|i| self.get_line(i))
            .collect()
    }


    pub fn line_count(&mut self) -> usize {
        self.piece_table.line_count()
    }

    pub fn line_length(&mut self, line_number: usize) -> usize {
        self.get_line(line_number).map_or(0, |line| line.len())
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.piece_table.is_empty()
    }


    pub fn char_at(&mut self, pos: Position) -> Option<char> {
        let offset = self.piece_table.position_to_offset(pos.line, pos.column);
        self.piece_table.char_at(offset)
    }


    pub fn set_line_ending(&mut self, line_ending: LineEnding) {
        self.line_ending = line_ending;
    }

    #[cfg(test)]
    pub fn from_lines(lines: Vec<String>) -> Self {
        let content = lines.join("\n");
        Self::from_string(content)
    }

    #[cfg(test)]
    pub fn offset_to_position(&mut self, offset: usize) -> Position {
        let (line, column) = self.piece_table.offset_to_position(offset);
        Position::new(line, column)
    }

    pub fn position_to_offset(&mut self, pos: Position) -> usize {
        self.piece_table.position_to_offset(pos.line, pos.column)
    }



    pub fn replace(&mut self, range: Range, replacement: &str) {
        let start_offset = self.position_to_offset(range.start);
        let end_offset = self.position_to_offset(range.end);
        let length = end_offset - start_offset;
        
        self.piece_table.delete(start_offset, length);
        self.piece_table.insert(start_offset, replacement);
    }

}

impl Clone for TextBuffer {
    fn clone(&self) -> Self {
        Self {
            piece_table: self.piece_table.clone(),
            line_ending: self.line_ending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let mut buffer = TextBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.line_count(), 1);
    }

    #[test]
    fn test_insert_text() {
        let mut buffer = TextBuffer::from_string("Hello World".to_string());
        buffer.insert(Position::new(0, 6), "Beautiful ");
        assert_eq!(buffer.get_text(), "Hello Beautiful World");
    }

    #[test]
    fn test_delete_range() {
        let mut buffer = TextBuffer::from_string("Hello Beautiful World".to_string());
        buffer.delete(Range::new(Position::new(0, 6), Position::new(0, 16)));
        assert_eq!(buffer.get_text(), "Hello World");
    }

    #[test]
    fn test_multiline() {
        let mut buffer = TextBuffer::from_string("Line 1\nLine 2\nLine 3".to_string());
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.get_line(1), Some("Line 2".to_string()));
        
        buffer.insert_newline(Position::new(1, 4));
        assert_eq!(buffer.line_count(), 4);
    }

    #[test]
    fn test_position_conversions() {
        let mut buffer = TextBuffer::from_string("Hello\nWorld\nTest".to_string());
        
        let pos = Position::new(1, 3);
        let offset = buffer.position_to_offset(pos);
        assert_eq!(offset, 9);
        
        let converted_pos = buffer.offset_to_position(offset);
        assert_eq!(converted_pos.line, 1);
        assert_eq!(converted_pos.column, 3);
    }

    #[test]
    fn test_from_lines_compatibility() {
        let lines = vec!["Hello".to_string(), "World".to_string()];
        let mut buffer = TextBuffer::from_lines(lines.clone());
        
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.get_line(0), Some("Hello".to_string()));
        assert_eq!(buffer.get_line(1), Some("World".to_string()));
        
        let back_to_lines = buffer.get_lines();
        assert_eq!(back_to_lines, lines);
    }
}