#[derive(Debug)]
pub enum Command {
    // Basic movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    // Word movement
    MoveWordForward,
    MoveWordBackward,
    MoveWordEnd,
    MoveBigWordForward,
    MoveBigWordBackward,
    MoveBigWordEnd,

    // Line movement
    MoveLineStart,
    MoveLineEnd,
    MoveFirstNonWhitespace,
    MoveDownToFirstNonWhitespace,
    MoveUpToFirstNonWhitespace,

    // Document movement
    MoveDocumentStart,
    MoveDocumentEnd,
    MovePageUp,
    MovePageDown,
    MoveHalfPageUp,
    MoveHalfPageDown,

    // Line jumping
    #[allow(dead_code)] // Will be wired up in key handler
    MoveToLine(usize),

    // Screen positioning
    MoveToScreenTop,    // H
    MoveToScreenMiddle, // M
    MoveToScreenBottom, // L

    // Bracket matching
    MatchBracket, // %

    // Character search
    #[allow(dead_code)] // Will be wired up in key handler
    FindChar(char),
    #[allow(dead_code)] // Will be wired up in key handler
    FindCharBackward(char),
    #[allow(dead_code)] // Will be wired up in key handler
    FindCharBefore(char),
    #[allow(dead_code)] // Will be wired up in key handler
    FindCharBeforeBackward(char),
    RepeatFind,
    RepeatFindReverse,

    // Mark commands
    SetMark(char),
    JumpToMark(char),
    JumpToMarkLine(char),
    JumpBackward,
    JumpForward,

    // Insert modes
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterInsertModeNewLine,
    EnterInsertModeNewLineAbove,
    EnterInsertModeLineEnd,
    EnterInsertModeLineStart,

    // Indentation commands
    IndentLine,
    IndentLines(usize), // count of lines
    DedentLine,
    DedentLines(usize), // count of lines

    // Search commands
    EnterSearchMode,
    EnterSearchBackwardMode,
    SearchNext,
    SearchPrevious,
    SearchWordUnderCursor,         // *
    SearchWordUnderCursorBackward, // #

    // Other commands
    EnterCommandMode,
    InsertChar(char),
    InsertNewline,
    InsertTab,
    DeleteChar,
    DeleteCharForward,
    DeleteCharBackward,
    DeleteLine,
    DeleteLines(usize), // count of lines
    DeleteToEndOfLine,
    DeleteWord,
    DeleteBigWord,
    DeleteWordBackward,
    DeleteBigWordBackward,
    DeleteToEndOfWord,
    DeleteToEndOfBigWord,
    DeleteToStartOfLine,
    DeleteToFirstNonWhitespace,
    DeleteToEndOfFile,
    DeleteToStartOfFile,
    SubstituteChar,
    SubstituteLine,
    DeleteUntilChar(char),
    DeleteUntilCharBackward(char),
    DeleteFindChar(char),
    DeleteFindCharBackward(char),

    // Change commands (delete + enter insert mode)
    ChangeLine,
    ChangeLines(usize),
    ChangeToEndOfLine,
    ChangeWord,
    ChangeBigWord,
    ChangeWordBackward,
    ChangeBigWordBackward,
    ChangeToEndOfWord,
    ChangeToEndOfBigWord,
    ChangeToStartOfLine,
    ChangeToFirstNonWhitespace,
    ChangeToEndOfFile,
    ChangeToStartOfFile,
    ChangeUntilChar(char),
    ChangeUntilCharBackward(char),
    ChangeFindChar(char),
    ChangeFindCharBackward(char),

    // Yank and paste commands (simplified)
    Yank(crate::yank_paste_handler::YankType, Option<char>),
    Paste(crate::yank_paste_handler::PasteType, Option<char>),

    // Visual mode commands
    EnterVisualChar,
    EnterVisualLine,
    EnterVisualBlock,
    ExitVisualMode,
    VisualDelete,
    VisualIndent,
    VisualDedent,
    VisualYank,

    ExitInsertMode,
    Redraw,

    // Line operations
    JoinLines,

    // Case operations
    ToggleCase,
    Lowercase,
    Uppercase,

    // Undo/Redo commands
    Undo,
    Redo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
    SearchBackward,
    VisualChar,
    VisualLine,
    VisualBlock,
}