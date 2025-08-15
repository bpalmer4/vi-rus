use crate::controller::command_types::{Command, Mode};
use crate::controller::yank_paste::{YankType, PasteType};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct KeyHandler;

impl KeyHandler {
    pub fn parse_key_with_state(
        mode: &Mode,
        key_event: &crossterm::event::KeyEvent,
        pending_key: &mut Option<char>,
        number_prefix: &mut Option<usize>,
        pending_register: &mut Option<char>,
    ) -> Option<Command> {
        let key = key_event.code;
        let modifiers = key_event.modifiers;

        match mode {
            Mode::Normal => Self::parse_normal_mode_with_state(
                key,
                modifiers,
                pending_key,
                number_prefix,
                pending_register,
            ),
            Mode::Insert => Self::parse_insert_mode_key(key),
            Mode::Command => Self::parse_command_mode_key(key),
            Mode::Search | Mode::SearchBackward => None, // Search mode input is handled directly in controller
            Mode::VisualChar | Mode::VisualLine | Mode::VisualBlock => {
                Self::parse_visual_mode_key(key, modifiers)
            }
        }
    }

    fn parse_normal_mode_key(key: KeyCode, modifiers: KeyModifiers) -> Option<Command> {
        match key {
            // Control key movements first (more specific)
            KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::MovePageDown)
            }
            KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::MovePageUp)
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::MoveHalfPageDown)
            }
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::MoveHalfPageUp)
            }
            // Alternative binding for half page down (in case Ctrl+D is intercepted by terminal)
            KeyCode::Char('j') if modifiers.contains(KeyModifiers::ALT) => {
                Some(Command::MoveHalfPageDown)
            }
            KeyCode::Char('r') if modifiers.contains(KeyModifiers::CONTROL) => Some(Command::Redo),
            KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::Redraw)
            }
            KeyCode::Char('v') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::EnterVisualBlock)
            }
            KeyCode::Char('o') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::JumpBackward)
            }
            KeyCode::Char('i') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::JumpForward)
            }

            // Insert modes
            KeyCode::Char('i') => Some(Command::EnterInsertMode),
            KeyCode::Char('a') => Some(Command::EnterInsertModeAfter),
            KeyCode::Char('o') => Some(Command::EnterInsertModeNewLine),
            KeyCode::Char('O') => Some(Command::EnterInsertModeNewLineAbove),
            KeyCode::Char('A') => Some(Command::EnterInsertModeLineEnd),
            KeyCode::Char('I') => Some(Command::EnterInsertModeLineStart),

            // Basic movement
            KeyCode::Char('h') | KeyCode::Left => Some(Command::MoveLeft),
            KeyCode::Char('j') | KeyCode::Down => Some(Command::MoveDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Command::MoveUp),
            KeyCode::Char('l') | KeyCode::Right => Some(Command::MoveRight),

            // Word movement
            KeyCode::Char('w') => Some(Command::MoveWordForward),
            KeyCode::Char('b') => Some(Command::MoveWordBackward),
            KeyCode::Char('e') => Some(Command::MoveWordEnd),
            KeyCode::Char('W') => Some(Command::MoveBigWordForward),
            KeyCode::Char('B') => Some(Command::MoveBigWordBackward),
            KeyCode::Char('E') => Some(Command::MoveBigWordEnd),

            // Line movement
            KeyCode::Char('0') => Some(Command::MoveLineStart),
            KeyCode::Char('$') => Some(Command::MoveLineEnd),
            KeyCode::Char('^') => Some(Command::MoveFirstNonWhitespace),
            KeyCode::Char('+') => Some(Command::MoveDownToFirstNonWhitespace),
            KeyCode::Char('-') => Some(Command::MoveUpToFirstNonWhitespace),
            KeyCode::Enter => Some(Command::MoveDownToFirstNonWhitespace),

            // Document movement
            KeyCode::Char('g') => None, // gg and other g commands handled in stateful parser
            KeyCode::Char('G') => Some(Command::MoveDocumentEnd),

            // Screen positioning
            KeyCode::Char('H') => Some(Command::MoveToScreenTop),
            KeyCode::Char('M') => Some(Command::MoveToScreenMiddle),
            KeyCode::Char('L') => Some(Command::MoveToScreenBottom),
            KeyCode::PageDown => Some(Command::MovePageDown),
            KeyCode::PageUp => Some(Command::MovePageUp),

            // Character search (will need special handling)
            KeyCode::Char('f') => None, // Handled specially - needs next char
            KeyCode::Char('F') => None, // Handled specially - needs next char
            KeyCode::Char('t') => None, // Handled specially - needs next char
            KeyCode::Char('T') => None, // Handled specially - needs next char
            KeyCode::Char(';') => Some(Command::RepeatFind),
            KeyCode::Char(',') => Some(Command::RepeatFindReverse),

            // Delete commands
            KeyCode::Char('x') => Some(Command::DeleteCharForward),
            KeyCode::Char('X') => Some(Command::DeleteCharBackward),
            KeyCode::Char('D') => Some(Command::DeleteToEndOfLine),
            KeyCode::Char('s') => Some(Command::SubstituteChar),
            KeyCode::Char('S') => Some(Command::SubstituteLine),
            KeyCode::Char('J') => Some(Command::JoinLines),
            KeyCode::Char('~') => Some(Command::ToggleCase),

            // Change commands
            KeyCode::Char('c') => None, // Handled in stateful parser for cc, cw, etc.
            KeyCode::Char('C') => Some(Command::ChangeToEndOfLine),

            // Visual mode
            KeyCode::Char('v') => Some(Command::EnterVisualChar),
            KeyCode::Char('V') => Some(Command::EnterVisualLine),

            // Yank (copy) commands
            KeyCode::Char('y') => None, // Handled in stateful parser for yy, yw, etc.

            // Paste commands
            KeyCode::Char('p') => Some(Command::Paste(
                PasteType::After,
                None,
            )),
            KeyCode::Char('P') => Some(Command::Paste(
                PasteType::Before,
                None,
            )),

            // Indentation (handled in new stateful parser)
            KeyCode::Char('>') => None,
            KeyCode::Char('<') => None,

            // Command mode
            KeyCode::Char(':') => Some(Command::EnterCommandMode),

            // Search mode
            KeyCode::Char('/') => Some(Command::EnterSearchMode),
            KeyCode::Char('?') => Some(Command::EnterSearchBackwardMode),
            KeyCode::Char('n') => Some(Command::SearchNext),
            KeyCode::Char('N') => Some(Command::SearchPrevious),
            KeyCode::Char('*') => Some(Command::SearchWordUnderCursor),
            KeyCode::Char('#') => Some(Command::SearchWordUnderCursorBackward),
            KeyCode::Char('%') => Some(Command::MatchBracket),

            // Undo/Redo
            KeyCode::Char('u') => Some(Command::Undo),

            _ => None,
        }
    }

    fn parse_insert_mode_key(key: KeyCode) -> Option<Command> {
        match key {
            KeyCode::Esc => Some(Command::ExitInsertMode),
            KeyCode::Enter => Some(Command::InsertNewline),
            KeyCode::Tab => Some(Command::InsertTab),
            KeyCode::Backspace => Some(Command::DeleteChar),
            KeyCode::Char(c) => Some(Command::InsertChar(c)),
            // Add arrow key support for insert mode
            KeyCode::Left => Some(Command::MoveLeft),
            KeyCode::Right => Some(Command::MoveRight),
            KeyCode::Up => Some(Command::MoveUp),
            KeyCode::Down => Some(Command::MoveDown),
            _ => None,
        }
    }

    fn parse_normal_mode_with_state(
        key: KeyCode,
        modifiers: KeyModifiers,
        pending_key: &mut Option<char>,
        number_prefix: &mut Option<usize>,
        pending_register: &mut Option<char>,
    ) -> Option<Command> {
        match key {
            // Handle '0' specially - if no number prefix exists, it's MoveLineStart
            KeyCode::Char('0') if number_prefix.is_none() => {
                Some(Command::MoveLineStart)
            }
            // Handle numbers for prefixes
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    *number_prefix = Some(number_prefix.unwrap_or(0) * 10 + digit as usize);
                }
                None // Wait for the actual command
            }

            // Handle pending multi-key sequences
            KeyCode::Char(c) if pending_key.is_some() => {
                let pending = pending_key.take().expect("pending_key was just checked to be Some");
                let count = number_prefix.take().unwrap_or(1);

                // Handle register sequences first
                if pending == '"' && (c.is_ascii_alphabetic() || c.is_ascii_digit()) {
                    *pending_register = Some(c);
                    return None; // Wait for the actual command (y, d, p, etc.)
                }

                match (pending, c) {
                    ('>', '>') => Some(if count == 1 {
                        Command::IndentLine
                    } else {
                        Command::IndentLines(count)
                    }),
                    ('<', '<') => Some(if count == 1 {
                        Command::DedentLine
                    } else {
                        Command::DedentLines(count)
                    }),
                    ('d', 'd') => Some(if count == 1 {
                        Command::DeleteLine
                    } else {
                        Command::DeleteLines(count)
                    }),
                    ('d', 'w') => Some(Command::DeleteWord),
                    ('d', 'W') => Some(Command::DeleteBigWord),
                    ('d', 'b') => Some(Command::DeleteWordBackward),
                    ('d', 'B') => Some(Command::DeleteBigWordBackward),
                    ('d', 'e') => Some(Command::DeleteToEndOfWord),
                    ('d', 'E') => Some(Command::DeleteToEndOfBigWord),
                    ('d', '0') => Some(Command::DeleteToStartOfLine),
                    ('d', '$') => Some(Command::DeleteToEndOfLine),
                    ('d', '^') => Some(Command::DeleteToFirstNonWhitespace),
                    ('d', 'G') => Some(Command::DeleteToEndOfFile),
                    ('d', 'g') => Some(Command::DeleteToStartOfFile), // dgg -> delete to start
                    ('d', 't') => {
                        // For dt{char} - wait for target character
                        *pending_key = Some('~'); // Use '~' to indicate delete-until-char mode
                        None
                    }
                    ('d', 'T') => {
                        // For dT{char} - wait for target character
                        *pending_key = Some('@'); // Use '@' to indicate delete-until-char-backward mode
                        None
                    }
                    ('d', 'f') => {
                        // For df{char} - wait for target character
                        *pending_key = Some('#'); // Use '#' to indicate delete-find-char mode
                        None
                    }
                    ('d', 'F') => {
                        // For dF{char} - wait for target character
                        *pending_key = Some('%'); // Use '%' to indicate delete-find-char-backward mode
                        None
                    }
                    ('~', target_char) => Some(Command::DeleteUntilChar(target_char)),
                    ('@', target_char) => Some(Command::DeleteUntilCharBackward(target_char)),
                    ('#', target_char) => Some(Command::DeleteFindChar(target_char)),
                    ('%', target_char) => Some(Command::DeleteFindCharBackward(target_char)),

                    // Yank (copy) commands
                    ('y', 'y') => {
                        let register = pending_register.take();
                        Some(if count == 1 {
                            Command::Yank(YankType::Line, register)
                        } else {
                            Command::Yank(
                                YankType::Lines(count),
                                register,
                            )
                        })
                    }
                    ('y', 'w') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::Word,
                            register,
                        ))
                    }
                    ('y', 'W') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::BigWord,
                            register,
                        ))
                    }
                    ('y', 'b') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::WordBackward,
                            register,
                        ))
                    }
                    ('y', 'B') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::BigWordBackward,
                            register,
                        ))
                    }
                    ('y', 'e') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::ToEndOfWord,
                            register,
                        ))
                    }
                    ('y', 'E') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::ToEndOfBigWord,
                            register,
                        ))
                    }
                    ('y', '0') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::ToStartOfLine,
                            register,
                        ))
                    }
                    ('y', '$') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::ToEndOfLine,
                            register,
                        ))
                    }
                    ('y', '^') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::ToFirstNonWhitespace,
                            register,
                        ))
                    }
                    ('y', 'G') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::ToEndOfFile,
                            register,
                        ))
                    }
                    ('y', 'g') => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::ToStartOfFile,
                            register,
                        ))
                    } // ygg -> yank to start
                    ('y', 't') => {
                        // For yt{char} - wait for target character
                        *pending_key = Some('&'); // Use '&' to indicate yank-until-char mode
                        None
                    }
                    ('y', 'T') => {
                        // For yT{char} - wait for target character
                        *pending_key = Some('*'); // Use '*' to indicate yank-until-char-backward mode
                        None
                    }
                    ('y', 'f') => {
                        // For yf{char} - wait for target character
                        *pending_key = Some('('); // Use '(' to indicate yank-find-char mode
                        None
                    }
                    ('y', 'F') => {
                        // For yF{char} - wait for target character
                        *pending_key = Some(')'); // Use ')' to indicate yank-find-char-backward mode
                        None
                    }
                    ('&', target_char) => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::UntilChar(target_char),
                            register,
                        ))
                    }
                    ('*', target_char) => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::UntilCharBackward(target_char),
                            register,
                        ))
                    }
                    ('(', target_char) => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::FindChar(target_char),
                            register,
                        ))
                    }
                    (')', target_char) => {
                        let register = pending_register.take();
                        Some(Command::Yank(
                            YankType::FindCharBackward(target_char),
                            register,
                        ))
                    }

                    // Change (delete + insert mode) commands
                    ('c', 'c') => Some(if count == 1 {
                        Command::ChangeLine
                    } else {
                        Command::ChangeLines(count)
                    }),
                    ('c', 'w') => Some(Command::ChangeWord),
                    ('c', 'W') => Some(Command::ChangeBigWord),
                    ('c', 'b') => Some(Command::ChangeWordBackward),
                    ('c', 'B') => Some(Command::ChangeBigWordBackward),
                    ('c', 'e') => Some(Command::ChangeToEndOfWord),
                    ('c', 'E') => Some(Command::ChangeToEndOfBigWord),
                    ('c', '0') => Some(Command::ChangeToStartOfLine),
                    ('c', '$') => Some(Command::ChangeToEndOfLine),
                    ('c', '^') => Some(Command::ChangeToFirstNonWhitespace),
                    ('c', 'G') => Some(Command::ChangeToEndOfFile),
                    ('c', 'g') => Some(Command::ChangeToStartOfFile), // cgg -> change to start
                    ('c', 't') => {
                        // For ct{char} - wait for target character
                        *pending_key = Some('!'); // Use '!' to indicate change-until-char mode
                        None
                    }
                    ('c', 'T') => {
                        // For cT{char} - wait for target character
                        *pending_key = Some('?'); // Use '?' to indicate change-until-char-backward mode
                        None
                    }
                    ('c', 'f') => {
                        // For cf{char} - wait for target character
                        *pending_key = Some('['); // Use '[' to indicate change-find-char mode
                        None
                    }
                    ('c', 'F') => {
                        // For cF{char} - wait for target character
                        *pending_key = Some(']'); // Use ']' to indicate change-find-char-backward mode
                        None
                    }
                    ('!', target_char) => Some(Command::ChangeUntilChar(target_char)),
                    ('?', target_char) => Some(Command::ChangeUntilCharBackward(target_char)),
                    ('[', target_char) => Some(Command::ChangeFindChar(target_char)),
                    (']', target_char) => Some(Command::ChangeFindCharBackward(target_char)),
                    ('m', mark_char) if mark_char.is_ascii_alphabetic() => {
                        Some(Command::SetMark(mark_char))
                    }
                    ('\'', mark_char)
                        if mark_char.is_ascii_alphabetic()
                            || mark_char == '\''
                            || mark_char == '.'
                            || mark_char == '^' =>
                    {
                        Some(Command::JumpToMarkLine(mark_char))
                    }
                    ('`', mark_char)
                        if mark_char.is_ascii_alphabetic()
                            || mark_char == '`'
                            || mark_char == '.'
                            || mark_char == '^' =>
                    {
                        Some(Command::JumpToMark(mark_char))
                    }
                    // Handle 'g' commands: gg for goto line 1, gu for lowercase, gU for uppercase
                    ('g', 'g') => Some(Command::MoveDocumentStart),
                    ('g', 'u') => Some(Command::Lowercase),
                    ('g', 'U') => Some(Command::Uppercase),
                    
                    // Handle character search commands
                    ('f', target_char) => Some(Command::FindChar(target_char)),
                    ('F', target_char) => Some(Command::FindCharBackward(target_char)),
                    ('t', target_char) => Some(Command::FindCharBefore(target_char)),
                    ('T', target_char) => Some(Command::FindCharBeforeBackward(target_char)),
                    _ => {
                        // Invalid sequence, clear state
                        *pending_key = None;
                        *number_prefix = None;
                        None
                    }
                }
            }

            // Start multi-key sequences (only for unmodified keys)
            KeyCode::Char('>') if modifiers.is_empty() => {
                *pending_key = Some('>');
                None // Wait for second >
            }
            KeyCode::Char('<') if modifiers.is_empty() => {
                *pending_key = Some('<');
                None // Wait for second <
            }
            KeyCode::Char('d') if modifiers.is_empty() => {
                *pending_key = Some('d');
                None // Wait for second key (d, w, W, etc.)
            }
            KeyCode::Char('y') if modifiers.is_empty() => {
                *pending_key = Some('y');
                None // Wait for second key (y, w, W, etc.)
            }
            KeyCode::Char('c') if modifiers.is_empty() => {
                *pending_key = Some('c');
                None // Wait for second key (c, w, W, etc.)
            }
            KeyCode::Char('m') if modifiers.is_empty() => {
                *pending_key = Some('m');
                None // Wait for mark character
            }
            KeyCode::Char('\'') if modifiers.is_empty() => {
                *pending_key = Some('\'');
                None // Wait for mark character (line jump)
            }
            KeyCode::Char('`') if modifiers.is_empty() => {
                *pending_key = Some('`');
                None // Wait for mark character (exact position jump)
            }
            KeyCode::Char('"') if modifiers.is_empty() => {
                *pending_key = Some('"');
                None // Wait for register name (a-z, A-Z, 0-9)
            }
            KeyCode::Char('g') if modifiers.is_empty() => {
                *pending_key = Some('g');
                None // Wait for second key (g, u, U)
            }
            KeyCode::Char('f') if modifiers.is_empty() => {
                *pending_key = Some('f');
                None // Wait for target character
            }
            KeyCode::Char('F') if modifiers.is_empty() => {
                *pending_key = Some('F');
                None // Wait for target character
            }
            KeyCode::Char('t') if modifiers.is_empty() => {
                *pending_key = Some('t');
                None // Wait for target character
            }
            KeyCode::Char('T') if modifiers.is_empty() => {
                *pending_key = Some('T');
                None // Wait for target character
            }

            // Fall back to regular parsing for other keys
            _ => {
                // Check if there's a pending register for this command
                let register_char = pending_register.take();

                // Clear any pending state for non-multi-key commands
                *pending_key = None;
                // Don't consume number_prefix here - let the controller handle it

                // Handle register-aware commands
                match (key, register_char) {
                    (KeyCode::Char('p'), Some(reg)) => Some(Command::Paste(
                        PasteType::After,
                        Some(reg),
                    )),
                    (KeyCode::Char('P'), Some(reg)) => Some(Command::Paste(
                        PasteType::Before,
                        Some(reg),
                    )),
                    (KeyCode::Char('y'), Some(_reg)) => {
                        // Store the register for the yank command sequence
                        *pending_register = register_char;
                        *pending_key = Some('y');
                        None // Wait for second key (y, w, W, etc.)
                    }
                    _ => {
                        // Use existing parsing but apply count if relevant
                        Self::parse_normal_mode_key(key, modifiers)
                    }
                }
            }
        }
    }

    fn parse_command_mode_key(key: KeyCode) -> Option<Command> {
        match key {
            KeyCode::Enter => None,                          // Handled in run loop
            KeyCode::Esc => Some(Command::EnterCommandMode), // Exit command mode
            KeyCode::Char(_) | KeyCode::Backspace => None,   // Handled in run loop
            _ => None,
        }
    }

    fn parse_visual_mode_key(key: KeyCode, modifiers: KeyModifiers) -> Option<Command> {
        match key {
            // Exit visual mode
            KeyCode::Esc => Some(Command::ExitVisualMode),

            // Visual mode operations
            KeyCode::Char('d') => Some(Command::VisualDelete),
            KeyCode::Char('x') => Some(Command::VisualDelete),
            KeyCode::Char('y') => Some(Command::VisualYank),
            KeyCode::Char('>') => Some(Command::VisualIndent),
            KeyCode::Char('<') => Some(Command::VisualDedent),

            // Movement in visual mode (same as normal mode)
            KeyCode::Char('h') | KeyCode::Left => Some(Command::MoveLeft),
            KeyCode::Char('j') | KeyCode::Down => Some(Command::MoveDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Command::MoveUp),
            KeyCode::Char('l') | KeyCode::Right => Some(Command::MoveRight),

            // Word movement
            KeyCode::Char('w') => Some(Command::MoveWordForward),
            KeyCode::Char('b') => Some(Command::MoveWordBackward),
            KeyCode::Char('e') => Some(Command::MoveWordEnd),
            KeyCode::Char('W') => Some(Command::MoveBigWordForward),
            KeyCode::Char('B') => Some(Command::MoveBigWordBackward),
            KeyCode::Char('E') => Some(Command::MoveBigWordEnd),

            // Line movement
            KeyCode::Char('0') => Some(Command::MoveLineStart),
            KeyCode::Char('$') => Some(Command::MoveLineEnd),
            KeyCode::Char('^') => Some(Command::MoveFirstNonWhitespace),
            KeyCode::Char('+') => Some(Command::MoveDownToFirstNonWhitespace),
            KeyCode::Char('-') => Some(Command::MoveUpToFirstNonWhitespace),
            KeyCode::Enter => Some(Command::MoveDownToFirstNonWhitespace),

            // Document movement (only if not already handled above)
            KeyCode::Char('g') => Some(Command::MoveDocumentStart),
            KeyCode::Char('G') => Some(Command::MoveDocumentEnd),

            // Screen positioning
            KeyCode::Char('H') => Some(Command::MoveToScreenTop),
            KeyCode::Char('M') => Some(Command::MoveToScreenMiddle),
            KeyCode::Char('L') => Some(Command::MoveToScreenBottom),
            KeyCode::PageDown => Some(Command::MovePageDown),
            KeyCode::PageUp => Some(Command::MovePageUp),
            KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::MovePageDown)
            }
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::MoveHalfPageUp)
            }

            // Word search
            KeyCode::Char('*') => Some(Command::SearchWordUnderCursor),
            KeyCode::Char('#') => Some(Command::SearchWordUnderCursorBackward),

            // Bracket matching
            KeyCode::Char('%') => Some(Command::MatchBracket),

            // Switch visual modes (Ctrl+v must come first to avoid conflict)
            KeyCode::Char('v') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::EnterVisualBlock)
            }
            KeyCode::Char('v') => Some(Command::EnterVisualChar),
            KeyCode::Char('V') => Some(Command::EnterVisualLine),

            _ => None,
        }
    }
}
