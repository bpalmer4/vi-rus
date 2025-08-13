use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferType {
    Original,
    Add,
}

#[derive(Debug, Clone)]
pub struct Piece {
    buffer: BufferType,
    start: usize,
    length: usize,
}

impl Piece {
    pub fn new(buffer: BufferType, start: usize, length: usize) -> Self {
        Self {
            buffer,
            start,
            length,
        }
    }
}

pub struct LineIndex {
    line_starts: Vec<usize>,  // Offset positions where each line starts
    valid: bool,              // Whether the index is current
}

impl LineIndex {
    fn new() -> Self {
        Self {
            line_starts: vec![0],  // First line always starts at 0
            valid: false,
        }
    }
    
    fn invalidate(&mut self) {
        self.valid = false;
    }
    
    fn rebuild(&mut self, text: &str) {
        self.line_starts.clear();
        self.line_starts.push(0);
        
        let mut byte_pos = 0;
        for ch in text.chars() {
            if ch == '\n' {
                byte_pos += ch.len_utf8();
                self.line_starts.push(byte_pos);
            } else {
                byte_pos += ch.len_utf8();
            }
        }
        self.valid = true;
    }
    
    fn line_count(&self) -> usize {
        self.line_starts.len()
    }
    
    fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }
}

pub struct PieceTable {
    original: String,
    add: String,
    pieces: Vec<Piece>,
    total_length: usize,
    line_index: LineIndex,
}

impl PieceTable {
    /// Safe substring that respects UTF-8 character boundaries
    fn safe_substring(text: &str, start_byte: usize, end_byte: usize) -> String {
        if start_byte >= text.len() {
            return String::new();
        }
        
        let end_byte = end_byte.min(text.len());
        
        // Find safe start position (at a character boundary)
        let safe_start = if text.is_char_boundary(start_byte) {
            start_byte
        } else {
            // Find the next character boundary
            (start_byte..text.len()).find(|&i| text.is_char_boundary(i)).unwrap_or(text.len())
        };
        
        // Find safe end position (at a character boundary)  
        let safe_end = if text.is_char_boundary(end_byte) {
            end_byte
        } else {
            // Find the previous character boundary
            (0..=end_byte).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0)
        };
        
        if safe_start >= safe_end {
            return String::new();
        }
        
        text[safe_start..safe_end].to_string()
    }

    pub fn new() -> Self {
        Self {
            original: String::new(),
            add: String::new(),
            pieces: Vec::new(),
            total_length: 0,
            line_index: LineIndex::new(),
        }
    }

    pub fn from_string(text: String) -> Self {
        let length = text.len();
        let mut table = Self {
            original: text,
            add: String::new(),
            pieces: if length > 0 { 
                vec![Piece::new(BufferType::Original, 0, length)] 
            } else { 
                Vec::new() 
            },
            total_length: length,
            line_index: LineIndex::new(),
        };
        table.rebuild_line_index();
        table
    }

    pub fn insert(&mut self, position: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        let add_start = self.add.len();
        self.add.push_str(text);
        
        let insert_piece = Piece::new(BufferType::Add, add_start, text.len());
        
        if position >= self.total_length {
            // Insert at end
            self.pieces.push(insert_piece);
        } else {
            // Find the piece and position to split
            let mut current_offset = 0;
            
            for i in 0..self.pieces.len() {
                let piece = &self.pieces[i];
                
                if current_offset + piece.length > position {
                    // This piece contains our insertion point
                    let split_point = position - current_offset;
                    
                    if split_point == 0 {
                        // Insert at the beginning of this piece
                        self.pieces.insert(i, insert_piece);
                    } else if split_point == piece.length {
                        // Insert at the end of this piece
                        self.pieces.insert(i + 1, insert_piece);
                    } else {
                        // Split the piece
                        let left_piece = Piece::new(piece.buffer, piece.start, split_point);
                        let right_piece = Piece::new(
                            piece.buffer,
                            piece.start + split_point,
                            piece.length - split_point,
                        );
                        
                        self.pieces[i] = left_piece;
                        self.pieces.insert(i + 1, insert_piece);
                        self.pieces.insert(i + 2, right_piece);
                    }
                    break;
                }
                
                current_offset += piece.length;
            }
        }
        
        self.total_length += text.len();
        self.line_index.invalidate();
    }

    pub fn delete(&mut self, start: usize, length: usize) {
        if length == 0 || start >= self.total_length {
            return;
        }

        let end = (start + length).min(self.total_length);
        let mut current_offset = 0;
        let mut pieces_to_remove = Vec::new();
        let mut pieces_to_add = Vec::new();

        for (i, piece) in self.pieces.iter().enumerate() {
            let piece_start = current_offset;
            let piece_end = current_offset + piece.length;

            if piece_end <= start || piece_start >= end {
                // Piece is outside deletion range
                current_offset = piece_end;
                continue;
            }

            if piece_start >= start && piece_end <= end {
                // Piece is completely within deletion range
                pieces_to_remove.push(i);
            } else if piece_start < start && piece_end > end {
                // Piece spans the entire deletion range - split into two
                let left_length = start - piece_start;
                let right_start = piece.start + (end - piece_start);
                let right_length = piece_end - end;

                let left_piece = Piece::new(piece.buffer, piece.start, left_length);
                let right_piece = Piece::new(piece.buffer, right_start, right_length);

                pieces_to_add.push((i, vec![left_piece, right_piece]));
                pieces_to_remove.push(i);
            } else if piece_start < start {
                // Piece starts before deletion, ends within
                let new_length = start - piece_start;
                let new_piece = Piece::new(piece.buffer, piece.start, new_length);
                pieces_to_add.push((i, vec![new_piece]));
                pieces_to_remove.push(i);
            } else {
                // Piece starts within deletion, ends after
                let new_start = piece.start + (end - piece_start);
                let new_length = piece_end - end;
                let new_piece = Piece::new(piece.buffer, new_start, new_length);
                pieces_to_add.push((i, vec![new_piece]));
                pieces_to_remove.push(i);
            }

            current_offset = piece_end;
        }

        // Apply changes in reverse order to maintain indices
        // First, collect the insert operations and sort them
        let mut sorted_adds: Vec<_> = pieces_to_add.into_iter().collect();
        sorted_adds.sort_by_key(|(i, _)| *i);
        
        // Remove pieces in reverse order 
        for &i in pieces_to_remove.iter().rev() {
            self.pieces.remove(i);
        }

        // Adjust insert indices based on how many pieces were removed before each position
        for (original_i, new_pieces) in sorted_adds.into_iter() {
            // Count how many pieces were removed before this position
            let removed_before = pieces_to_remove.iter().filter(|&&removed_i| removed_i < original_i).count();
            let adjusted_i = original_i - removed_before;
            
            // Insert the new pieces at the adjusted position
            for (j, piece) in new_pieces.into_iter().enumerate() {
                let insert_pos = (adjusted_i + j).min(self.pieces.len());
                self.pieces.insert(insert_pos, piece);
            }
        }

        self.total_length = self.total_length.saturating_sub(end - start);
        self.line_index.invalidate();
    }

    pub fn get_text(&self) -> String {
        let mut result = String::with_capacity(self.total_length);
        
        for piece in &self.pieces {
            let text = match piece.buffer {
                BufferType::Original => {
                    if piece.start <= self.original.len() {
                        Self::safe_substring(&self.original, piece.start, piece.start + piece.length)
                    } else {
                        eprintln!("Warning: Invalid piece start in original buffer - start: {}, buffer len: {}", 
                                piece.start, self.original.len());
                        String::new()
                    }
                },
                BufferType::Add => {
                    if piece.start <= self.add.len() {
                        Self::safe_substring(&self.add, piece.start, piece.start + piece.length)
                    } else {
                        eprintln!("Warning: Invalid piece start in add buffer - start: {}, buffer len: {}", 
                                piece.start, self.add.len());
                        String::new()
                    }
                },
            };
            result.push_str(&text);
        }
        
        result
    }

    pub fn substring(&self, start: usize, end: usize) -> String {
        if start >= end || start >= self.total_length {
            return String::new();
        }

        let end = end.min(self.total_length);
        let mut result = String::new();
        let mut current_offset = 0;

        for piece in &self.pieces {
            let piece_end = current_offset + piece.length;

            if current_offset >= end {
                break;
            }
            if piece_end <= start {
                current_offset = piece_end;
                continue;
            }

            let piece_start_in_range = start.saturating_sub(current_offset);
            let piece_end_in_range = (end - current_offset).min(piece.length);

            let text = match piece.buffer {
                BufferType::Original => {
                    if piece.start <= self.original.len() {
                        Self::safe_substring(&self.original, piece.start, piece.start + piece.length)
                    } else {
                        eprintln!("Warning: Invalid piece start in original buffer (get_range) - start: {}, buffer len: {}", 
                                piece.start, self.original.len());
                        String::new()
                    }
                },
                BufferType::Add => {
                    if piece.start <= self.add.len() {
                        Self::safe_substring(&self.add, piece.start, piece.start + piece.length)
                    } else {
                        eprintln!("Warning: Invalid piece start in add buffer (get_range) - start: {}, buffer len: {}", 
                                piece.start, self.add.len());
                        String::new()
                    }
                },
            };

            // Ensure we slice at UTF-8 character boundaries
            let safe_text = Self::safe_substring(&text, piece_start_in_range, piece_end_in_range);
            result.push_str(&safe_text);
            current_offset = piece_end;
        }

        result
    }

    pub fn char_at(&self, position: usize) -> Option<char> {
        if position >= self.total_length {
            return None;
        }

        let mut current_offset = 0;
        
        for piece in &self.pieces {
            if current_offset + piece.length > position {
                let char_pos = position - current_offset;
                let text = match piece.buffer {
                    BufferType::Original => {
                        if piece.start <= self.original.len() {
                            Self::safe_substring(&self.original, piece.start, piece.start + piece.length)
                        } else {
                            eprintln!("Warning: Invalid piece start in original buffer (char_at) - start: {}, buffer len: {}", 
                                    piece.start, self.original.len());
                            String::new()
                        }
                    },
                    BufferType::Add => {
                        if piece.start <= self.add.len() {
                            Self::safe_substring(&self.add, piece.start, piece.start + piece.length)
                        } else {
                            eprintln!("Warning: Invalid piece start in add buffer (char_at) - start: {}, buffer len: {}", 
                                    piece.start, self.add.len());
                            String::new()
                        }
                    },
                };
                return text.chars().nth(char_pos);
            }
            current_offset += piece.length;
        }
        
        None
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.total_length == 0
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.total_length
    }

    #[cfg(test)]
    pub fn offset_to_position(&mut self, offset: usize) -> (usize, usize) {
        if !self.line_index.valid {
            self.rebuild_line_index();
        }

        let offset = offset.min(self.total_length);
        
        // Binary search to find the line
        let mut left = 0;
        let mut right = self.line_index.line_starts.len();
        
        while left < right {
            let mid = (left + right) / 2;
            if self.line_index.line_starts[mid] <= offset {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        
        let line = left.saturating_sub(1);
        let line_start = self.line_index.line_starts[line];
        let column = offset - line_start;
        
        (line, column)
    }

    fn rebuild_line_index(&mut self) {
        let text = self.get_text();
        self.line_index.rebuild(&text);
    }

    pub fn line_count(&mut self) -> usize {
        if !self.line_index.valid {
            self.rebuild_line_index();
        }
        self.line_index.line_count()
    }

    pub fn get_line_fast(&mut self, line_number: usize) -> Option<String> {
        if !self.line_index.valid {
            self.rebuild_line_index();
        }

        let line_start = self.line_index.line_start(line_number)?;
        let line_end = self.line_index.line_start(line_number + 1)
            .unwrap_or(self.total_length);

        if line_end > line_start && line_end <= self.total_length {
            let mut line = self.substring(line_start, line_end);
            // Remove the newline character if present
            if line.ends_with('\n') {
                line.pop();
            }
            Some(line)
        } else {
            Some(String::new())
        }
    }


    pub fn position_to_offset(&mut self, line: usize, column: usize) -> usize {
        if !self.line_index.valid {
            self.rebuild_line_index();
        }

        if let Some(line_start) = self.line_index.line_start(line) {
            (line_start + column).min(self.total_length)
        } else {
            self.total_length
        }
    }

}

impl Clone for PieceTable {
    fn clone(&self) -> Self {
        let mut cloned = Self {
            original: self.original.clone(),
            add: self.add.clone(),
            pieces: self.pieces.clone(),
            total_length: self.total_length,
            line_index: LineIndex::new(),
        };
        if self.line_index.valid {
            cloned.rebuild_line_index();
        }
        cloned
    }
}

impl fmt::Display for PieceTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let table = PieceTable::new();
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
        assert_eq!(table.get_text(), "");
    }

    #[test]
    fn test_from_string() {
        let table = PieceTable::from_string("Hello, World!".to_string());
        assert_eq!(table.len(), 13);
        assert!(!table.is_empty());
        assert_eq!(table.get_text(), "Hello, World!");
    }

    #[test]
    fn test_insert_at_beginning() {
        let mut table = PieceTable::from_string("World!".to_string());
        table.insert(0, "Hello, ");
        assert_eq!(table.get_text(), "Hello, World!");
    }

    #[test]
    fn test_insert_at_end() {
        let mut table = PieceTable::from_string("Hello".to_string());
        table.insert(5, ", World!");
        assert_eq!(table.get_text(), "Hello, World!");
    }

    #[test]
    fn test_insert_in_middle() {
        let mut table = PieceTable::from_string("HelloWorld!".to_string());
        table.insert(5, ", ");
        assert_eq!(table.get_text(), "Hello, World!");
    }

    #[test]
    fn test_delete_index_bounds() {
        // This test specifically checks the bug where delete operations
        // could cause insert index out of bounds errors
        let mut table = PieceTable::new();
        table.insert(0, "Hello World");
        
        // Delete middle portion - this should split a piece and require proper index management
        table.delete(5, 6); // Remove " World"
        assert_eq!(table.get_text(), "Hello");
        
        // Test another deletion that could trigger the index bug
        table.insert(5, " Beautiful Day");
        assert_eq!(table.get_text(), "Hello Beautiful Day");
        
        // Delete overlapping multiple pieces
        table.delete(5, 10); // Remove " Beautiful"
        assert_eq!(table.get_text(), "Hello Day");
    }

    #[test]
    fn test_delete() {
        let mut table = PieceTable::from_string("Hello, Cruel World!".to_string());
        table.delete(7, 6); // Remove "Cruel "
        assert_eq!(table.get_text(), "Hello, World!");
    }

    #[test]
    fn test_multiple_operations() {
        let mut table = PieceTable::new();
        
        table.insert(0, "Hello");
        assert_eq!(table.get_text(), "Hello");
        
        table.insert(5, " World");
        assert_eq!(table.get_text(), "Hello World");
        
        table.insert(11, "!");
        assert_eq!(table.get_text(), "Hello World!");
        
        table.delete(5, 6); // Remove " World"
        assert_eq!(table.get_text(), "Hello!");
    }

    #[test]
    fn test_get_lines() {
        let mut table = PieceTable::from_string("Line 1\nLine 2\nLine 3".to_string());
        let line_count = table.line_count();
        let lines: Vec<String> = (0..line_count)
            .map(|i| table.get_line_fast(i).unwrap_or_default())
            .collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
    }
}