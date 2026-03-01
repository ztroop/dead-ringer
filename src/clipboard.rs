use std::env;
use std::io::{self, Write};

use base64::{engine::general_purpose::STANDARD, Engine};

/// Ordered range of selected diff-vector indices (inclusive on both ends).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: usize,
    pub cursor: usize,
}

impl Selection {
    pub fn new(anchor: usize) -> Self {
        Self {
            anchor,
            cursor: anchor,
        }
    }

    /// Inclusive start of the selection.
    pub fn start(&self) -> usize {
        self.anchor.min(self.cursor)
    }

    /// Inclusive end of the selection.
    pub fn end(&self) -> usize {
        self.anchor.max(self.cursor)
    }

    /// Number of selected positions.
    pub fn len(&self) -> usize {
        self.end() - self.start() + 1
    }

    /// Whether a given diff-vector index falls within the selection.
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start() && pos <= self.end()
    }
}

/// Format selected bytes as a space-separated hex string (e.g. `"ff 0a 3c"`).
pub fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format selected bytes as an ASCII string, replacing non-printable bytes with `.`.
pub fn format_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b.is_ascii_whitespace() {
                b as char
            } else {
                '.'
            }
        })
        .collect()
}

/// Copy `text` to the system clipboard via the OSC 52 escape sequence.
///
/// When running inside tmux, the sequence is wrapped in a DCS passthrough
/// so that tmux forwards it to the outer terminal. Requires tmux 3.3+ with
/// `set -g allow-passthrough on` in tmux.conf.
///
/// Writes to stderr since ratatui uses the alternate screen on stderr.
pub fn osc52_copy(text: &str) -> io::Result<()> {
    let encoded = STANDARD.encode(text.as_bytes());
    let osc = format!("\x1b]52;c;{}\x07", encoded);

    let mut out = io::stderr().lock();
    if in_tmux() {
        write!(out, "\x1bPtmux;\x1b{}\x1b\\", osc)?;
    } else {
        write!(out, "{}", osc)?;
    }
    out.flush()
}

fn in_tmux() -> bool {
    env::var_os("TMUX").is_some_and(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_single_byte() {
        let sel = Selection::new(5);
        assert_eq!(sel.start(), 5);
        assert_eq!(sel.end(), 5);
        assert_eq!(sel.len(), 1);
        assert!(sel.contains(5));
        assert!(!sel.contains(4));
    }

    #[test]
    fn selection_forward() {
        let mut sel = Selection::new(2);
        sel.cursor = 7;
        assert_eq!(sel.start(), 2);
        assert_eq!(sel.end(), 7);
        assert_eq!(sel.len(), 6);
        assert!(sel.contains(2));
        assert!(sel.contains(5));
        assert!(sel.contains(7));
        assert!(!sel.contains(8));
    }

    #[test]
    fn selection_backward() {
        let mut sel = Selection::new(10);
        sel.cursor = 3;
        assert_eq!(sel.start(), 3);
        assert_eq!(sel.end(), 10);
        assert_eq!(sel.len(), 8);
    }

    #[test]
    fn format_hex_basic() {
        assert_eq!(format_hex(&[0xFF, 0x0A, 0x00]), "ff 0a 00");
    }

    #[test]
    fn format_hex_empty() {
        assert_eq!(format_hex(&[]), "");
    }

    #[test]
    fn format_ascii_printable() {
        assert_eq!(format_ascii(b"Hello"), "Hello");
    }

    #[test]
    fn format_ascii_non_printable() {
        assert_eq!(format_ascii(&[0x01, 0x41, 0x00, 0x7F]), ".A..");
    }

    #[test]
    fn format_ascii_spaces_preserved() {
        assert_eq!(format_ascii(b"a b"), "a b");
    }
}
