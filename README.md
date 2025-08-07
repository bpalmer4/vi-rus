# vi-rus

A vim-like (vim-lite) text editor built in Rust with cross-platform terminal support.

## Features

- **Vim-style Modal Editing**: Normal, Insert, Visual, Command, and Search modes
- **Text Navigation**: Word movement, line jumping, document navigation
- **Advanced Editing**: Cut, copy, paste with registers, undo/redo system
- **File Operations**: Open, save, multiple buffer management
- **Search & Replace**: Regex-based search with case sensitivity options
- **Marks & Jumps**: Local and global marks, jump list navigation
- **Configuration**: RC file support with vim-compatible settings
- **Visual Features**: Bracket highlighting, search result highlighting, visual mode selection

## Quick Start

```bash
# Build the editor
cargo build --release

# Run with a file
cargo run -- filename.txt

# Or run without arguments for an empty buffer
cargo run
```

## Key Bindings

### Normal Mode
- `h/j/k/l` - Move cursor left/down/up/right
- `w/b/e` - Word forward/backward/end movement
- `i/a/o` - Enter insert mode (before/after/new line)
- `v` - Enter visual mode
- `y/d/c` - Yank/delete/change operations
- `p/P` - Paste after/before cursor
- `J` - Join current line with next line
- `~` - Toggle case of character under cursor
- `gu` - Convert current line to lowercase
- `gU` - Convert current line to uppercase
- `H` - Move cursor to top of screen
- `M` - Move cursor to middle of screen
- `L` - Move cursor to bottom of screen
- `u` - Undo, `Ctrl+r` - Redo
- `m[a-z]` - Set local mark, `'[a-z]` - Jump to mark
- `/` - Search forward, `?` - Search backward, `n/N` - Next/previous
- `*` - Search for word under cursor forward
- `#` - Search for word under cursor backward
- `%` - Jump to matching bracket/parenthesis/brace (with visual highlighting)
- `:` - Enter command mode

### Command Mode
- `:w` - Save file
- `:q` - Quit (`:q!` force quit)
- `:wq` - Save and quit
- `:e filename` - Open file
- `:bn/:bp` - Next/previous buffer
- `:ascii` - Normalize Unicode characters to ASCII equivalents
- `:help` or `:h` or `:?` - Show help information

## Configuration

Create a `.virusrc` file in your home directory:

```
set number          " Show line numbers
set expandtab       " Use spaces for tabs
set tabstop=4       " Tab width
set autoindent      " Auto-indent new lines
```

## Dependencies

- [crossterm](https://crates.io/crates/crossterm) - Cross-platform terminal manipulation
- [clap](https://crates.io/crates/clap) - Command line argument parsing
- [regex](https://crates.io/crates/regex) - Regular expression support
- [arboard](https://crates.io/crates/arboard) - Clipboard integration

## Architecture

- Modular design with separate concerns for editing, rendering, and file operations
- Efficient text manipulation with performance-optimized yank operations
- Comprehensive undo/redo system with operation grouping
- Memory-safe Rust implementation with zero-copy optimizations where possible

## License

MIT License - see LICENSE file for details.