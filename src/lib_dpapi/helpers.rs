//! Helper utilities for DPAPI operations.
//!
//! This module provides various utility functions for:
//! - Hex string conversion and manipulation
//! - GUID validation and formatting
//! - File parsing (masterkey files)
//! - String encoding/escaping (CSV, JSON)
//! - Binary data operations

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use regex::Regex;

// Compile GUID regex only once using lazy initialization
static GUID_REGEX: OnceLock<Regex> = OnceLock::new();

/// Returns a reference to the compiled GUID validation regex.
///
/// The regex is compiled once on first use and cached for subsequent calls.
fn guid_regex() -> &'static Regex {
    GUID_REGEX.get_or_init(|| {
        Regex::new(
            r"^(\{{0,1}([0-9a-fA-F]){8}-([0-9a-fA-F]){4}-([0-9a-fA-F]){4}-([0-9a-fA-F]){4}-([0-9a-fA-F]){12}\}{0,1})$"
        ).expect("Failed to compile GUID regex")
    })
}

/// Collection of helper utilities for DPAPI operations.
///
/// Provides static methods for common operations like hex conversion,
/// GUID validation, file parsing, and data manipulation.
pub struct Helpers;

impl Helpers {
    /// Convert hex string to byte array
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
        let hex = if !hex.len().is_multiple_of(2) {
            format!("0{}", hex)
        } else {
            hex.to_string()
        };

        hex::decode(&hex).map_err(|e| anyhow!("Failed to decode hex: {}", e))
    }

    /// Convert byte array to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }

    /// Capitalize first letter of string
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().chain(chars).collect(),
        }
    }

    /// Combine two byte arrays
    pub fn combine(first: &[u8], second: &[u8]) -> Vec<u8> {
        [first, second].concat()
    }

    /// Pad array to specified length
    pub fn pad_to_length(input: &[u8], len: usize) -> Vec<u8> {
        if !input.len().is_multiple_of(len) {
            let pad_len = len - (input.len() % len);
            let mut result = vec![0u8; pad_len];
            result.extend_from_slice(input);
            result
        } else {
            input.to_vec()
        }
    }

    /// Trim trailing zero bytes
    pub fn trim_bytes(input: &[u8]) -> Vec<u8> {
        let mut end = input.len();
        while end > 0 && input[end - 1] == 0x00 {
            end -= 1;
        }
        input[..end].to_vec()
    }

    /// Parse masterkey file ({GUID}:SHA1 format)
    pub fn parse_masterkey_file(file_path: &str) -> Result<HashMap<String, String>> {
        let mut masterkeys = HashMap::new();

        if !Path::new(file_path).exists() {
            return Err(anyhow!("Masterkey file '{}' doesn't exist!", file_path));
        }

        let contents = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = contents.lines().collect();

        for line in lines {
            let parts: Vec<&str> = line.split(' ').collect();
            for part in parts {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    if trimmed.starts_with('{') {
                        // SharpDPAPI {GUID}:SHA1 format
                        let mk_parts: Vec<&str> = trimmed.split(':').collect();
                        if mk_parts.len() == 2 {
                            masterkeys.insert(mk_parts[0].to_string(), mk_parts[1].to_string());
                        }
                    } else if trimmed.starts_with("GUID:") {
                        // Mimikatz dpapi::cache format
                        let mk_parts: Vec<&str> = trimmed.split(';').collect();
                        if mk_parts.len() == 2 {
                            let guid_parts: Vec<&str> = mk_parts[0].split(':').collect();
                            let sha1_parts: Vec<&str> = mk_parts[1].split(':').collect();
                            if guid_parts.len() == 2 && sha1_parts.len() == 2 {
                                masterkeys.insert(guid_parts[1].to_string(), sha1_parts[1].to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(masterkeys)
    }

    /// Check if string is a valid GUID
    pub fn is_guid(value: &str) -> bool {
        guid_regex().is_match(value)
    }

    /// Clean string for CSV output
    pub fn string_to_csv_cell(s: &str) -> String {
        let must_quote = s.contains(',') || s.contains('"') || s.contains('\r') || s.contains('\n');
        if must_quote {
            let escaped = s.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        } else {
            s.to_string()
        }
    }

    /// Clean string for JSON output
    pub fn clean_for_json(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                '\\' | '"' => {
                    result.push('\\');
                    result.push(c);
                }
                '/' => {
                    result.push('\\');
                    result.push(c);
                }
                '\u{0008}' => result.push_str("\\b"),
                '\t' => result.push_str("\\t"),
                '\n' => result.push_str("\\n"),
                '\u{000c}' => result.push_str("\\f"),
                '\r' => result.push_str("\\r"),
                _ => {
                    if c < ' ' {
                        result.push_str(&format!("\\u{:04x}", c as u32));
                    } else {
                        result.push(c);
                    }
                }
            }
        }
        result
    }

    /// Find pattern in byte array
    pub fn array_index_of(array: &[u8], pattern: &[u8], offset: usize) -> Option<usize> {
        if pattern.len() > array.len() {
            return None;
        }

        (offset..=array.len() - pattern.len())
            .find(|&i| &array[i..i + pattern.len()] == pattern)
    }

    /// Split string into chunks of specified length
    pub fn split(text: &str, part_length: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut start = 0;

        while start < text.len() {
            let end = std::cmp::min(start + part_length, text.len());
            result.push(text[start..end].to_string());
            start = end;
        }

        result
    }

    #[cfg(windows)]
    pub fn is_high_integrity() -> bool {
        // Windows-specific implementation
        // For now, return false in Rust port
        false
    }

    #[cfg(not(windows))]
    pub fn is_high_integrity() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_bytes() {
        let result = Helpers::hex_to_bytes("48656c6c6f").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_bytes_to_hex() {
        let result = Helpers::bytes_to_hex(b"Hello");
        assert_eq!(result, "48656c6c6f");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(Helpers::capitalize("hello"), "Hello");
        assert_eq!(Helpers::capitalize("h"), "H");
        assert_eq!(Helpers::capitalize(""), "");
    }

    #[test]
    fn test_combine() {
        let result = Helpers::combine(b"Hello", b"World");
        assert_eq!(result, b"HelloWorld");
    }
}
