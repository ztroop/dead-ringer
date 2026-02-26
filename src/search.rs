use std::collections::HashSet;

/// Distinguishes between hex byte search and ASCII text search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchKind {
    Hex,
    Ascii,
}

/// Whether the application is accepting search input or in normal navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Normal,
    Input(SearchKind),
}

/// Holds all state related to an active or completed search.
#[derive(Debug, Clone)]
pub struct SearchState {
    pub mode: SearchMode,
    pub query: String,
    pub kind: SearchKind,
    pub matches: Vec<usize>,
    pub current_match: Option<usize>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            mode: SearchMode::Normal,
            query: String::new(),
            kind: SearchKind::Hex,
            matches: Vec::new(),
            current_match: None,
        }
    }
}

impl SearchState {
    /// Enter search input mode, clearing any previous results.
    pub fn start(&mut self, kind: SearchKind) {
        self.mode = SearchMode::Input(kind);
        self.kind = kind;
        self.query.clear();
        self.matches.clear();
        self.current_match = None;
    }

    /// Abort the search and reset state.
    pub fn cancel(&mut self) {
        self.mode = SearchMode::Normal;
        self.query.clear();
        self.matches.clear();
        self.current_match = None;
    }

    /// Toggle between hex and ASCII search while in input mode.
    pub fn toggle_kind(&mut self) {
        self.kind = match self.kind {
            SearchKind::Hex => SearchKind::Ascii,
            SearchKind::Ascii => SearchKind::Hex,
        };
        self.mode = SearchMode::Input(self.kind);
    }

    /// Execute the search against the diff byte data.
    pub fn submit(&mut self, diffs: &[(usize, u8)]) {
        self.mode = SearchMode::Normal;
        let pattern = self.parse_pattern();
        self.matches = find_matches(diffs, &pattern);
        self.current_match = if self.matches.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Advance to the next match, wrapping around.
    pub fn next_match(&mut self) {
        if let Some(ref mut idx) = self.current_match {
            *idx = (*idx + 1) % self.matches.len();
        }
    }

    /// Move to the previous match, wrapping around.
    pub fn prev_match(&mut self) {
        if let Some(ref mut idx) = self.current_match {
            *idx = idx.checked_sub(1).unwrap_or(self.matches.len() - 1);
        }
    }

    /// The diff-vector index of the current match, if any.
    pub fn current_match_pos(&self) -> Option<usize> {
        self.current_match.map(|idx| self.matches[idx])
    }

    /// Set of diff-vector indices covered by *all* matches (for highlighting).
    pub fn all_match_positions(&self) -> HashSet<usize> {
        let plen = self.pattern_len();
        self.matches
            .iter()
            .flat_map(|&start| start..start + plen)
            .collect()
    }

    /// Set of diff-vector indices covered by the *current* match.
    pub fn current_match_set(&self) -> HashSet<usize> {
        let plen = self.pattern_len();
        self.current_match_pos()
            .map(|start| (start..start + plen).collect())
            .unwrap_or_default()
    }

    fn pattern_len(&self) -> usize {
        match self.kind {
            SearchKind::Hex => parse_hex_string(&self.query).len(),
            SearchKind::Ascii => self.query.len(),
        }
    }

    fn parse_pattern(&self) -> Vec<u8> {
        match self.kind {
            SearchKind::Hex => parse_hex_string(&self.query),
            SearchKind::Ascii => self.query.as_bytes().to_vec(),
        }
    }
}

/// Parse a hex string (e.g. `"FF AA BB"` or `"ffaabb"`) into raw bytes.
///
/// Whitespace is stripped and the remaining characters are consumed in pairs.
/// Incomplete trailing nibbles are ignored.
pub fn parse_hex_string(s: &str) -> Vec<u8> {
    let hex: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    hex.as_bytes()
        .chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                let s = std::str::from_utf8(chunk).ok()?;
                u8::from_str_radix(s, 16).ok()
            } else {
                None
            }
        })
        .collect()
}

/// Find all starting indices where `pattern` appears in the byte values of `diffs`.
pub fn find_matches(diffs: &[(usize, u8)], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() || diffs.len() < pattern.len() {
        return vec![];
    }
    let bytes: Vec<u8> = diffs.iter().map(|(_, b)| *b).collect();
    (0..=bytes.len() - pattern.len())
        .filter(|&i| bytes[i..i + pattern.len()] == *pattern)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_lowercase() {
        assert_eq!(parse_hex_string("ff aa bb"), vec![0xFF, 0xAA, 0xBB]);
    }

    #[test]
    fn parse_hex_uppercase_no_spaces() {
        assert_eq!(parse_hex_string("FFAABB"), vec![0xFF, 0xAA, 0xBB]);
    }

    #[test]
    fn parse_hex_mixed_spacing() {
        assert_eq!(parse_hex_string("0a 1B  2c"), vec![0x0A, 0x1B, 0x2C]);
    }

    #[test]
    fn parse_hex_trailing_nibble_ignored() {
        assert_eq!(parse_hex_string("FF A"), vec![0xFF]);
    }

    #[test]
    fn parse_hex_empty() {
        assert_eq!(parse_hex_string(""), Vec::<u8>::new());
    }

    #[test]
    fn parse_hex_invalid_chars() {
        assert_eq!(parse_hex_string("GG"), Vec::<u8>::new());
    }

    #[test]
    fn find_matches_single_byte() {
        let diffs = vec![(0, 0xAA), (1, 0xBB), (2, 0xAA), (3, 0xCC)];
        assert_eq!(find_matches(&diffs, &[0xAA]), vec![0, 2]);
    }

    #[test]
    fn find_matches_multi_byte() {
        let diffs = vec![(0, 0xAA), (1, 0xBB), (2, 0xAA), (3, 0xBB), (4, 0xCC)];
        assert_eq!(find_matches(&diffs, &[0xAA, 0xBB]), vec![0, 2]);
    }

    #[test]
    fn find_matches_no_match() {
        let diffs = vec![(0, 0x01), (1, 0x02)];
        assert_eq!(find_matches(&diffs, &[0xFF]), Vec::<usize>::new());
    }

    #[test]
    fn find_matches_empty_pattern() {
        let diffs = vec![(0, 0x01)];
        assert_eq!(find_matches(&diffs, &[]), Vec::<usize>::new());
    }

    #[test]
    fn find_matches_pattern_longer_than_data() {
        let diffs = vec![(0, 0x01)];
        assert_eq!(find_matches(&diffs, &[0x01, 0x02]), Vec::<usize>::new());
    }

    #[test]
    fn find_matches_overlapping() {
        let diffs = vec![(0, 0xAA), (1, 0xAA), (2, 0xAA)];
        assert_eq!(find_matches(&diffs, &[0xAA, 0xAA]), vec![0, 1]);
    }

    #[test]
    fn search_state_submit_and_navigate() {
        let diffs = vec![(0, 0x01), (1, 0xFF), (2, 0x02), (3, 0xFF), (4, 0x03)];
        let mut state = SearchState::default();
        state.start(SearchKind::Hex);
        state.query = "FF".to_string();
        state.submit(&diffs);

        assert_eq!(state.matches, vec![1, 3]);
        assert_eq!(state.current_match, Some(0));
        assert_eq!(state.current_match_pos(), Some(1));

        state.next_match();
        assert_eq!(state.current_match, Some(1));
        assert_eq!(state.current_match_pos(), Some(3));

        state.next_match();
        assert_eq!(state.current_match, Some(0));

        state.prev_match();
        assert_eq!(state.current_match, Some(1));
    }

    #[test]
    fn search_state_ascii_mode() {
        let diffs: Vec<(usize, u8)> = b"hello world"
            .iter()
            .enumerate()
            .map(|(i, &b)| (i, b))
            .collect();
        let mut state = SearchState::default();
        state.start(SearchKind::Ascii);
        state.query = "lo".to_string();
        state.submit(&diffs);

        assert_eq!(state.matches, vec![3]);
        assert_eq!(state.current_match_pos(), Some(3));
    }

    #[test]
    fn search_state_no_matches() {
        let diffs = vec![(0, 0x00)];
        let mut state = SearchState::default();
        state.start(SearchKind::Hex);
        state.query = "FF".to_string();
        state.submit(&diffs);

        assert!(state.matches.is_empty());
        assert_eq!(state.current_match, None);
        assert_eq!(state.current_match_pos(), None);
    }

    #[test]
    fn search_state_cancel_clears() {
        let diffs = vec![(0, 0xFF)];
        let mut state = SearchState::default();
        state.start(SearchKind::Hex);
        state.query = "FF".to_string();
        state.submit(&diffs);
        assert_eq!(state.matches.len(), 1);

        state.cancel();
        assert!(state.matches.is_empty());
        assert!(state.query.is_empty());
        assert_eq!(state.mode, SearchMode::Normal);
    }

    #[test]
    fn search_state_toggle_kind() {
        let mut state = SearchState::default();
        state.start(SearchKind::Hex);
        assert_eq!(state.kind, SearchKind::Hex);

        state.toggle_kind();
        assert_eq!(state.kind, SearchKind::Ascii);
        assert_eq!(state.mode, SearchMode::Input(SearchKind::Ascii));

        state.toggle_kind();
        assert_eq!(state.kind, SearchKind::Hex);
    }

    #[test]
    fn all_match_positions_covers_full_pattern() {
        let diffs = vec![(0, 0xAA), (1, 0xBB), (2, 0xCC), (3, 0xAA), (4, 0xBB)];
        let mut state = SearchState::default();
        state.start(SearchKind::Hex);
        state.query = "AA BB".to_string();
        state.submit(&diffs);

        let positions = state.all_match_positions();
        assert!(positions.contains(&0));
        assert!(positions.contains(&1));
        assert!(positions.contains(&3));
        assert!(positions.contains(&4));
        assert!(!positions.contains(&2));
    }

    #[test]
    fn current_match_set_only_covers_current() {
        let diffs = vec![(0, 0xAA), (1, 0xBB), (2, 0xCC), (3, 0xAA), (4, 0xBB)];
        let mut state = SearchState::default();
        state.start(SearchKind::Hex);
        state.query = "AA BB".to_string();
        state.submit(&diffs);

        let current = state.current_match_set();
        assert!(current.contains(&0));
        assert!(current.contains(&1));
        assert!(!current.contains(&3));

        state.next_match();
        let current = state.current_match_set();
        assert!(!current.contains(&0));
        assert!(current.contains(&3));
        assert!(current.contains(&4));
    }
}
