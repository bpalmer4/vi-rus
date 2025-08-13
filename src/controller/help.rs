use crate::document_model::Document;

pub fn create_help_document() -> Document {
    let help_lines = vec![
        "VI-RUS EDITOR HELP".to_string(),
        "==================".to_string(),
        "".to_string(),
        "MOVEMENT (Normal Mode):".to_string(),
        "  h, ← - Move left".to_string(),
        "  j, ↓ - Move down".to_string(),
        "  k, ↑ - Move up".to_string(),
        "  l, → - Move right".to_string(),
        "".to_string(),
        "WORD MOVEMENT:".to_string(),
        "  w - Next word start".to_string(),
        "  b - Previous word start".to_string(),
        "  e - Next word end".to_string(),
        "  W - Next WORD start (space-separated)".to_string(),
        "  B - Previous WORD start".to_string(),
        "  E - Next WORD end".to_string(),
        "".to_string(),
        "LINE MOVEMENT:".to_string(),
        "  0 - Start of line".to_string(),
        "  $ - End of line".to_string(),
        "  ^ - First non-whitespace character".to_string(),
        "  + - Down to first non-whitespace of next line".to_string(),
        "  - - Up to first non-whitespace of previous line".to_string(),
        "  Enter - Down to first non-whitespace of next line".to_string(),
        "".to_string(),
        "INDENTATION:".to_string(),
        "  >> - Indent current line".to_string(),
        "  << - Dedent current line".to_string(),
        "  3>> - Indent 3 lines starting from current".to_string(),
        "  5<< - Dedent 5 lines starting from current".to_string(),
        "".to_string(),
        "DOCUMENT MOVEMENT:".to_string(),
        "  gg - Start of document".to_string(),
        "  G - End of document".to_string(),
        "  :15 - Go to line 15".to_string(),
        "  Ctrl+f, Page Down - Page down".to_string(),
        "  Ctrl+b, Page Up - Page up".to_string(),
        "  Ctrl+d - Half page down".to_string(),
        "  Ctrl+u - Half page up".to_string(),
        "".to_string(),
        "CHARACTER SEARCH:".to_string(),
        "  f{char} - Find character forward".to_string(),
        "  F{char} - Find character backward".to_string(),
        "  t{char} - Move to before character forward".to_string(),
        "  T{char} - Move to before character backward".to_string(),
        "  ; - Repeat last find".to_string(),
        "  , - Repeat last find (reverse)".to_string(),
        "".to_string(),
        "MARKS & JUMPS:".to_string(),
        "  m{a-z} - Set local mark (a-z)".to_string(),
        "  m{A-Z} - Set global mark (A-Z, across files)".to_string(),
        "  '{a-z,A-Z} - Jump to mark line (switches files for A-Z)".to_string(),
        "  `{a-z,A-Z} - Jump to exact mark position (switches files)".to_string(),
        "  '' - Jump to last jump position".to_string(),
        "  '. - Jump to last change position".to_string(),
        "  '^ - Jump to last insert position".to_string(),
        "  Ctrl+o - Jump backward in jump list (switches files)".to_string(),
        "  Ctrl+i - Jump forward in jump list (switches files)".to_string(),
        "  :marks - List all marks".to_string(),
        "  :jumps, :ju - Show jump list history".to_string(),
        "  :clear marks - Clear all user marks (a-z, A-Z)".to_string(),
        "  :clear jumps - Clear jump list history".to_string(),
        "  :clear all - Clear marks and jumps".to_string(),
        "  :clear - Clear/redraw screen".to_string(),
        "".to_string(),
        "SEARCH & REPLACE:".to_string(),
        "  /{pattern} - Search forward for pattern (regex)".to_string(),
        "  ?{pattern} - Search backward for pattern (regex)".to_string(),
        "  n - Next search result (same direction)".to_string(),
        "  N - Previous search result (opposite direction)".to_string(),
        "  :s/old/new/ - Replace first match on current line".to_string(),
        "  :s/old/new/g - Replace all matches on current line".to_string(),
        "  :s/old/new/i - Case-insensitive replace on current line".to_string(),
        "  :%s/old/new/g - Replace all matches in entire document".to_string(),
        "  :%s/old/new/gi - Global case-insensitive replace".to_string(),
        "  Search results are highlighted in yellow".to_string(),
        "  Brackets under cursor are highlighted in cyan".to_string(),
        "  Unmatched brackets are highlighted in red".to_string(),
        "  Examples:".to_string(),
        "    /test - Find 'test' forward (highlighted in yellow)".to_string(),
        "    ?hello - Find 'hello' backward (highlighted in yellow)".to_string(),
        "    :s/foo/bar/ - Replace first 'foo' with 'bar'".to_string(),
        "    :%s/\\d+/NUMBER/g - Replace all numbers with 'NUMBER'".to_string(),
        "".to_string(),
        "DELETE OPERATIONS:".to_string(),
        "  x - Delete character forward (at cursor)".to_string(),
        "  X - Delete character backward".to_string(),
        "  s - Substitute character (delete and enter insert mode)".to_string(),
        "  S - Substitute line (clear line and enter insert mode)".to_string(),
        "  D - Delete to end of line".to_string(),
        "  dd - Delete entire line".to_string(),
        "  Backspace - Delete character backward (insert mode)".to_string(),
        "".to_string(),
        "MOTION-BASED DELETE:".to_string(),
        "  dw - Delete word forward".to_string(),
        "  dW - Delete WORD forward (space-separated)".to_string(),
        "  db - Delete word backward".to_string(),
        "  dB - Delete WORD backward".to_string(),
        "  de - Delete to end of word".to_string(),
        "  dE - Delete to end of WORD".to_string(),
        "  d0 - Delete to beginning of line".to_string(),
        "  d$ - Delete to end of line (same as D)".to_string(),
        "  d^ - Delete to first non-whitespace character".to_string(),
        "  dgg - Delete to beginning of file".to_string(),
        "  dG - Delete to end of file".to_string(),
        "".to_string(),
        "CHARACTER-BASED DELETE:".to_string(),
        "  dt{char} - Delete until (but not including) character".to_string(),
        "  dT{char} - Delete backward until character".to_string(),
        "  df{char} - Delete including character forward".to_string(),
        "  dF{char} - Delete including character backward".to_string(),
        "".to_string(),
        "NUMBERED DELETE COMMANDS:".to_string(),
        "  3dd - Delete 3 lines starting from current".to_string(),
        "  5dw - Delete 5 words forward".to_string(),
        "  2db - Delete 2 words backward".to_string(),
        "  4>> - Indent 4 lines (example of numbered commands)".to_string(),
        "".to_string(),
        "CHANGE OPERATIONS (delete + enter insert mode):".to_string(),
        "  cc - Change entire line".to_string(),
        "  C - Change to end of line".to_string(),
        "  cw - Change word forward".to_string(),
        "  cW - Change WORD forward (space-separated)".to_string(),
        "  cb - Change word backward".to_string(),
        "  cB - Change WORD backward".to_string(),
        "  ce - Change to end of word".to_string(),
        "  cE - Change to end of WORD".to_string(),
        "  c0 - Change to beginning of line".to_string(),
        "  c$ - Change to end of line (same as C)".to_string(),
        "  c^ - Change to first non-whitespace character".to_string(),
        "  cgg - Change to beginning of file".to_string(),
        "  cG - Change to end of file".to_string(),
        "".to_string(),
        "CHARACTER-BASED CHANGE:".to_string(),
        "  ct{char} - Change until (but not including) character".to_string(),
        "  cT{char} - Change backward until character".to_string(),
        "  cf{char} - Change including character forward".to_string(),
        "  cF{char} - Change including character backward".to_string(),
        "".to_string(),
        "NUMBERED CHANGE COMMANDS:".to_string(),
        "  3cc - Change 3 lines starting from current".to_string(),
        "  5cw - Change 5 words forward".to_string(),
        "  2cb - Change 2 words backward".to_string(),
        "".to_string(),
        "YANK (COPY) OPERATIONS:".to_string(),
        "  yy - Yank (copy) current line".to_string(),
        "  3yy - Yank 3 lines starting from current".to_string(),
        "  yw - Yank word forward".to_string(),
        "  yW - Yank WORD forward (space-separated)".to_string(),
        "  yb - Yank word backward".to_string(),
        "  yB - Yank WORD backward".to_string(),
        "  ye - Yank to end of word".to_string(),
        "  yE - Yank to end of WORD".to_string(),
        "  y0 - Yank to start of line".to_string(),
        "  y$ - Yank to end of line".to_string(),
        "  y^ - Yank to first non-whitespace character".to_string(),
        "  yG - Yank to end of file".to_string(),
        "  ygg - Yank to start of file".to_string(),
        "  yt{char} - Yank until (but not including) character".to_string(),
        "  yT{char} - Yank backward until character".to_string(),
        "  yf{char} - Yank including character forward".to_string(),
        "  yF{char} - Yank including character backward".to_string(),
        "".to_string(),
        "PASTE OPERATIONS:".to_string(),
        "  p - Paste after cursor/line".to_string(),
        "  P - Paste before cursor/line".to_string(),
        "".to_string(),
        "NAMED REGISTERS:".to_string(),
        "  \"ayy - Yank current line to register 'a'".to_string(),
        "  \"ayw - Yank word to register 'a'".to_string(),
        "  \"ap - Paste from register 'a' after cursor".to_string(),
        "  \"aP - Paste from register 'a' before cursor".to_string(),
        "  Registers a-z: replace content, A-Z: append to content".to_string(),
        "  Numbered registers 0-9: automatic delete history".to_string(),
        "  Examples:".to_string(),
        "    \"ayy - Copy line to register 'a'".to_string(),
        "    \"byW - Copy WORD to register 'b'".to_string(),
        "    \"ap - Paste register 'a' content".to_string(),
        "".to_string(),
        "VISUAL MODE:".to_string(),
        "  v - Enter visual character mode".to_string(),
        "  V - Enter visual line mode".to_string(),
        "  Ctrl+v - Enter visual block mode".to_string(),
        "  Esc - Exit visual mode".to_string(),
        "  d, x - Delete selected text".to_string(),
        "  y - Yank (copy) selected text".to_string(),
        "  > - Indent selected lines".to_string(),
        "  < - Dedent selected lines".to_string(),
        "  All movement keys work in visual mode".to_string(),
        "".to_string(),
        "INSERT MODES:".to_string(),
        "  i - Insert before cursor".to_string(),
        "  a - Insert after cursor".to_string(),
        "  o - Open new line below".to_string(),
        "  O - Open new line above".to_string(),
        "  A - Insert at end of line".to_string(),
        "  I - Insert at start of line".to_string(),
        "  Esc - Return to normal mode".to_string(),
        "".to_string(),
        "FILE OPERATIONS:".to_string(),
        "  :w - Save current file".to_string(),
        "  :w filename - Save as filename".to_string(),
        "  :wq - Save and quit".to_string(),
        "  :q - Quit (if no changes)".to_string(),
        "  :q! - Force quit without saving".to_string(),
        "  :f - Show file information".to_string(),
        "".to_string(),
        "BUFFER OPERATIONS:".to_string(),
        "  :e - Create new empty buffer".to_string(),
        "  :e filename - Edit/open new file".to_string(),
        "  :e file1 file2 - Open multiple files as buffers".to_string(),
        "  :badd - Add new empty buffer".to_string(),
        "  :badd file1 file2 - Add multiple files to buffer list".to_string(),
        "  :ls - List all open buffers (% = current, + = modified)".to_string(),
        "  :b1, :b2, :b3 - Switch to buffer 1, 2, 3".to_string(),
        "  :bf filename - Switch to buffer by filename".to_string(),
        "  :bn - Next buffer".to_string(),
        "  :bp - Previous buffer".to_string(),
        "  :bd - Close current buffer".to_string(),
        "  :bd! - Force close buffer (discard unsaved changes)".to_string(),
        "".to_string(),
        "READ OPERATIONS:".to_string(),
        "  :r filename - Insert file at cursor".to_string(),
        "  :r !command - Insert command output".to_string(),
        "  :0r filename - Insert at beginning".to_string(),
        "  :$r filename - Insert at end".to_string(),
        "  :10r filename - Insert after line 10".to_string(),
        "".to_string(),
        "TABS & SPACES:".to_string(),
        "  :set tabstop=4 - Set tab width to 4 spaces".to_string(),
        "  :set et - Tab key inserts spaces".to_string(),
        "  :set noet - Tab key inserts tabs".to_string(),
        "  :set list - Show whitespace characters".to_string(),
        "  :set nolist - Hide whitespace characters".to_string(),
        "  :detab - Convert all tabs to spaces".to_string(),
        "  :retab - Convert all spaces to tabs".to_string(),
        "  :ascii - Normalize Unicode characters to ASCII equivalents".to_string(),
        "  :normalize - Same as :ascii".to_string(),
        "  :brackets - Check for unmatched brackets".to_string(),
        "  :checkbrackets - Same as :brackets".to_string(),
        "  :redraw - Force screen refresh".to_string(),
        "  :scroll - Show scroll information".to_string(),
        "  :resetscroll - Reset scroll position".to_string(),
        "".to_string(),
        "EDIT OPERATIONS:".to_string(),
        "  :paste - Paste from clipboard".to_string(),
        "  :set ff=unix - Set Unix line endings".to_string(),
        "  :set ff=dos - Set Windows line endings".to_string(),
        "  :set ff=mac - Set Mac line endings".to_string(),
        "  :set nu - Show line numbers".to_string(),
        "  :set nonu - Hide line numbers".to_string(),
        "".to_string(),
        "RC CONFIGURATION:".to_string(),
        "  vi-rus loads settings from .virusrc file".to_string(),
        "  Search order: current directory, then ~/.virusrc".to_string(),
        "  :mkvirus - Generate sample .virusrc in current directory".to_string(),
        "".to_string(),
        "RC FILE FORMAT:".to_string(),
        "  # Comment lines start with # or \"".to_string(),
        "  set nu                # Show line numbers".to_string(),
        "  set nonu              # Hide line numbers".to_string(),
        "  set list              # Show whitespace".to_string(),
        "  set nolist            # Hide whitespace".to_string(),
        "  set expandtab         # Use spaces for tabs".to_string(),
        "  set noexpandtab       # Use tab characters".to_string(),
        "  set tabstop=4         # Set tab width".to_string(),
        "  set fileformat=unix   # Line endings (unix/dos/mac)".to_string(),
        "".to_string(),
        "RC ALTERNATIVE SYNTAX:".to_string(),
        "  tab_stop=4            # Tab width".to_string(),
        "  expand_tab=true       # Use spaces".to_string(),
        "  line_numbers=yes      # Show line numbers".to_string(),
        "  show_whitespace=false # Hide whitespace".to_string(),
        "  line_ending=unix      # Line endings".to_string(),
        "".to_string(),
        "UNDO & REDO:".to_string(),
        "  u - Undo last change".to_string(),
        "  Ctrl+r - Redo last undone change".to_string(),
        "".to_string(),
        "HELP & MISC:".to_string(),
        "  :help, :h - Show this help".to_string(),
        "  :redraw - Force screen redraw".to_string(),
        "  :unmatched - Toggle highlighting of all unmatched brackets".to_string(),
        "  Ctrl+l - Force screen redraw".to_string(),
        "".to_string(),
        "Press :bd to close this help buffer".to_string(),
        "".to_string(),
        "😊".to_string(),
    ];

    // Create a document with the help content
    
    let help_content = help_lines.join("\n");
    
    let help_doc = Document::from_string(help_content);
    help_doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_multiline_document() {
        
        let simple_content = "Line 1\nLine 2\nLine 3".to_string();
        let simple_doc = Document::from_string(simple_content);
        
        assert_eq!(simple_doc.line_count(), 3);
        assert_eq!(simple_doc.get_line(0).unwrap_or_default(), "Line 1");
        assert_eq!(simple_doc.get_line(1).unwrap_or_default(), "Line 2");
        assert_eq!(simple_doc.get_line(2).unwrap_or_default(), "Line 3");
    }
    
    #[test] 
    fn test_unicode_content() {
        
        let unicode_content = "  h, ← - Move left\n  j, ↓ - Move down\n  k, ↑ - Move up".to_string();
        let unicode_doc = Document::from_string(unicode_content);
        
        println!("Unicode test - {} lines:", unicode_doc.line_count());
        for i in 0..unicode_doc.line_count() {
            let line = unicode_doc.get_line(i).unwrap_or_default();
            println!("   Line {}: '{}'", i, line);
        }
        
        assert_eq!(unicode_doc.line_count(), 3);
        // Test that Unicode characters are handled correctly now
        assert_eq!(unicode_doc.get_line(0).unwrap_or_default(), "  h, ← - Move left");
        assert_eq!(unicode_doc.get_line(1).unwrap_or_default(), "  j, ↓ - Move down");
        assert_eq!(unicode_doc.get_line(2).unwrap_or_default(), "  k, ↑ - Move up");
    }
    
    #[test] 
    fn test_various_unicode_characters() {
        
        let content = "😀 emoji\n🔥 fire\n✅ checkmark\nкириллица\n中文字符\n→←↑↓ arrows".to_string();
        let doc = Document::from_string(content);
        
        println!("Various Unicode test - {} lines:", doc.line_count());
        for i in 0..doc.line_count() {
            let line = doc.get_line(i).unwrap_or_default();
            println!("   Line {}: '{}'", i, line);
        }
        
        assert_eq!(doc.line_count(), 6);
        assert_eq!(doc.get_line(0).unwrap_or_default(), "😀 emoji");
        assert_eq!(doc.get_line(1).unwrap_or_default(), "🔥 fire");
        assert_eq!(doc.get_line(2).unwrap_or_default(), "✅ checkmark");
        assert_eq!(doc.get_line(3).unwrap_or_default(), "кириллица");
        assert_eq!(doc.get_line(4).unwrap_or_default(), "中文字符");
        assert_eq!(doc.get_line(5).unwrap_or_default(), "→←↑↓ arrows");
        
        println!("✅ Various Unicode characters handled correctly");
    }
    
    #[test]
    fn test_help_document_creation() {
        let mut help_doc = create_help_document();
        
        // Basic verification that it works
        assert!(help_doc.line_count() > 200);
        let first_line = help_doc.get_line(0).unwrap_or_default();
        assert_eq!(first_line, "VI-RUS EDITOR HELP");
        
        // Test specific lines are correct
        assert_eq!(help_doc.get_line(1).unwrap_or_default(), "==================");
        assert_eq!(help_doc.get_line(2).unwrap_or_default(), "");
        assert_eq!(help_doc.get_line(3).unwrap_or_default(), "MOVEMENT (Normal Mode):");
        assert_eq!(help_doc.get_line(4).unwrap_or_default(), "  h, ← - Move left");
        assert_eq!(help_doc.get_line(5).unwrap_or_default(), "  j, ↓ - Move down");
        
        // Verify content is accessible overall
        let content = help_doc.get_piece_table_content();
        assert!(content.contains("VI-RUS EDITOR HELP"));
        assert!(content.contains("MOVEMENT (Normal Mode)"));
        assert!(content.contains("Press :bd to close this help buffer"));
        
        println!("✅ Help document creation working correctly");
        println!("   Help has {} lines", help_doc.line_count());
        println!("   Lines 0-5 are correctly formatted");
    }
}
