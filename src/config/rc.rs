// use crate::controller::Controller; // Old Controller no longer used
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RcConfig {
    pub tab_stop: usize,
    pub expand_tab: bool,
    pub show_line_numbers: bool,
    pub show_whitespace: bool,
    pub line_ending: String,
}

impl Default for RcConfig {
    fn default() -> Self {
        Self {
            tab_stop: 4,
            expand_tab: false,
            show_line_numbers: false,
            show_whitespace: false,
            line_ending: "unix".to_string(),
        }
    }
}

pub struct RcLoader;

impl RcLoader {
    /// Get the path to the RC file
    /// Looks for .virusrc in:
    /// 1. Current directory
    /// 2. Home directory (~/.virusrc)
    pub fn get_rc_path() -> Option<PathBuf> {
        // First check current directory
        let current_rc = Path::new(".virusrc");
        if current_rc.exists() {
            return Some(current_rc.to_path_buf());
        }

        // Then check home directory
        if let Ok(home) = env::var("HOME") {
            let home_rc = Path::new(&home).join(".virusrc");
            if home_rc.exists() {
                return Some(home_rc);
            }
        }

        None
    }

    /// Load and parse the RC file
    pub fn load_config() -> RcConfig {
        let mut config = RcConfig::default();

        if let Some(rc_path) = Self::get_rc_path() {
            match fs::read_to_string(&rc_path) {
                Ok(content) => {
                    Self::parse_config_content(&content, &mut config);
                }
                Err(_) => {
                    // Silently fail if we can't read the file
                }
            }
        }

        config
    }

    /// Parse the content of an RC file
    fn parse_config_content(content: &str, config: &mut RcConfig) {
        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with('"') {
                continue;
            }

            Self::parse_config_line(line, config);
        }
    }

    /// Parse a single configuration line
    fn parse_config_line(line: &str, config: &mut RcConfig) {
        // Remove inline comments
        let line = if let Some(pos) = line.find('#') {
            &line[..pos]
        } else {
            line
        }
        .trim();

        // Handle "set" commands (vim-style)
        if let Some(stripped) = line.strip_prefix("set ") {
            let setting = stripped.trim();

            if setting == "nu" || setting == "number" {
                config.show_line_numbers = true;
            } else if setting == "nonu" || setting == "nonumber" {
                config.show_line_numbers = false;
            } else if setting == "expandtab" {
                config.expand_tab = true;
            } else if setting == "noexpandtab" {
                config.expand_tab = false;
            } else if setting == "list" {
                config.show_whitespace = true;
            } else if setting == "nolist" {
                config.show_whitespace = false;
            } else if setting.starts_with("tabstop=") {
                if let Some(value) = setting.strip_prefix("tabstop=") {
                    if let Ok(tab_stop) = value.parse::<usize>() {
                        if tab_stop > 0 && tab_stop <= 16 {
                            config.tab_stop = tab_stop;
                        }
                    }
                }
            } else if setting.starts_with("fileformat=") {
                if let Some(value) = setting.strip_prefix("fileformat=") {
                    match value {
                        "unix" | "dos" | "mac" => {
                            config.line_ending = value.to_string();
                        }
                        _ => {} // Invalid value, ignore
                    }
                }
            }
        }
        // Handle direct key-value pairs
        else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "tabstop" | "tab_stop" => {
                    if let Ok(tab_stop) = value.parse::<usize>() {
                        if tab_stop > 0 && tab_stop <= 16 {
                            config.tab_stop = tab_stop;
                        }
                    }
                }
                "expandtab" | "expand_tab" => {
                    config.expand_tab = value == "true" || value == "1" || value == "yes";
                }
                "linenumbers" | "line_numbers" | "number" => {
                    config.show_line_numbers = value == "true" || value == "1" || value == "yes";
                }
                "whitespace" | "show_whitespace" | "list" => {
                    config.show_whitespace = value == "true" || value == "1" || value == "yes";
                }
                "fileformat" | "line_ending" => {
                    match value {
                        "unix" | "dos" | "mac" => {
                            config.line_ending = value.to_string();
                        }
                        _ => {} // Invalid value, ignore
                    }
                }
                _ => {} // Unknown setting, ignore
            }
        }
    }

    /// Apply the configuration to the new modular architecture
    pub fn apply_config_to_shared_state(shared_state: &mut crate::controller::SharedEditorState, config: &RcConfig) {
        // Apply view settings
        shared_state.view.set_tab_stop(config.tab_stop);
        shared_state.view.set_line_numbers(config.show_line_numbers);
        shared_state.view.set_show_whitespace(config.show_whitespace);

        // Apply document settings
        shared_state.buffer_manager
            .current_document_mut()
            .set_expand_tab(config.expand_tab);

        // Apply line ending setting
        match config.line_ending.as_str() {
            "unix" => shared_state.buffer_manager
                .current_document_mut()
                .set_line_ending(crate::document_model::LineEnding::Unix),
            "dos" => shared_state.buffer_manager
                .current_document_mut()
                .set_line_ending(crate::document_model::LineEnding::Windows),
            "mac" => shared_state.buffer_manager
                .current_document_mut()
                .set_line_ending(crate::document_model::LineEnding::Mac),
            _ => {} // Default to Unix
        }
    }

    /// Generate a sample RC file content
    pub fn generate_sample_rc() -> String {
        r#"# vi-rus configuration file (.virusrc)
# This file configures the vi-rus text editor
# Lines starting with # or " are comments

# Display settings
set nu                  # Show line numbers (or set nonu to disable)
set list               # Show whitespace characters (or set nolist to disable)

# Tab settings
set tabstop=4          # Set tab width to 4 spaces
set expandtab          # Use spaces instead of tabs (or set noexpandtab)

# File format
set fileformat=unix    # Line endings: unix, dos, or mac

# Alternative key=value syntax:
# tab_stop=4
# expand_tab=true
# line_numbers=true
# show_whitespace=false
# line_ending=unix
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vim_style_config() {
        let mut config = RcConfig::default();
        let content = r#"
            set nu
            set expandtab
            set tabstop=8
            set list
            set fileformat=dos
        "#;

        RcLoader::parse_config_content(content, &mut config);

        assert!(config.show_line_numbers);
        assert!(config.expand_tab);
        assert_eq!(config.tab_stop, 8);
        assert!(config.show_whitespace);
        assert_eq!(config.line_ending, "dos");
    }

    #[test]
    fn test_parse_key_value_config() {
        let mut config = RcConfig::default();
        let content = r#"
            tabstop=2
            expandtab=true
            line_numbers=yes
            show_whitespace=false
            line_ending=mac
        "#;

        RcLoader::parse_config_content(content, &mut config);

        assert!(config.show_line_numbers);
        assert!(config.expand_tab);
        assert_eq!(config.tab_stop, 2);
        assert!(!config.show_whitespace);
        assert_eq!(config.line_ending, "mac");
    }

    #[test]
    fn test_parse_mixed_config_with_comments() {
        let mut config = RcConfig::default();
        let content = r#"
            # This is a comment
            set nu                 # Enable line numbers
            " This is also a comment
            
            tabstop=6              # Custom tab stop
            # set expandtab        # This is commented out
            set nolist             # Disable whitespace display
        "#;

        RcLoader::parse_config_content(content, &mut config);

        assert!(config.show_line_numbers);
        assert!(!config.expand_tab); // Should remain false (default)
        assert_eq!(config.tab_stop, 6);
        assert!(!config.show_whitespace);
    }

    #[test]
    fn test_invalid_values_ignored() {
        let mut config = RcConfig::default();
        let content = r#"
            set tabstop=0          # Invalid: too small
            set tabstop=20         # Invalid: too large  
            tabstop=invalid        # Invalid: not a number
            line_ending=invalid    # Invalid: unknown format
            unknown_setting=value  # Unknown setting
        "#;

        RcLoader::parse_config_content(content, &mut config);

        // Should remain at defaults since all values are invalid
        assert_eq!(config.tab_stop, 4);
        assert_eq!(config.line_ending, "unix");
    }
}
