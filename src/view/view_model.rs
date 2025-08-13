/// View Model - Abstracts document data for the view layer
/// This ensures the view has no direct dependencies on Document internals

#[derive(Debug, Clone)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct BracketHighlight {
    pub matching: Option<(usize, usize)>,
    pub unmatched_at_cursor: Option<(usize, usize)>,
    pub all_unmatched: Vec<(usize, usize)>,
}

/// The ViewModel trait provides everything the view needs to render
/// without depending on Document internals
pub trait ViewModel {
    /// Get current cursor position
    fn get_cursor_position(&self) -> CursorPosition;
    
    /// Get total line count
    fn get_line_count(&self) -> usize;
    
    /// Get a specific line by number
    fn get_line(&self, line_number: usize) -> Option<String>;
}

/// Concrete implementation that adapts Document to ViewModel
pub struct DocumentViewModel<'a> {
    document: &'a crate::document_model::Document,
}

impl<'a> DocumentViewModel<'a> {
    pub fn new(document: &'a crate::document_model::Document) -> Self {
        Self { document }
    }
}

impl<'a> ViewModel for DocumentViewModel<'a> {
    fn get_cursor_position(&self) -> CursorPosition {
        CursorPosition {
            line: self.document.cursor_line(),
            column: self.document.cursor_column(),
        }
    }
    
    fn get_line_count(&self) -> usize {
        self.document.line_count()
    }
    
    fn get_line(&self, line_number: usize) -> Option<String> {
        self.document.get_line(line_number)
    }
}