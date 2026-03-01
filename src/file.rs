use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

/// Read the contents of a file into a vector of bytes.
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Compare two files and return a vector of tuples containing the index of the
/// differing byte and the byte itself (from file1 for the common range and
/// file1's extra bytes; from file2 for file2's extra bytes when file2 is longer).
pub fn diff_files(file1: &[u8], file2: &[u8]) -> Vec<(usize, u8)> {
    let min_len = file1.len().min(file2.len());
    let mut diffs: Vec<(usize, u8)> = file1[..min_len]
        .iter()
        .zip(file2[..min_len].iter())
        .enumerate()
        .filter_map(|(i, (&b1, &b2))| if b1 != b2 { Some((i, b1)) } else { None })
        .collect();

    for (i, &b) in file1[min_len..].iter().enumerate() {
        diffs.push((min_len + i, b));
    }
    for (i, &b) in file2[min_len..].iter().enumerate() {
        diffs.push((min_len + i, b));
    }

    diffs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_files_same_length_differing() {
        let f1 = b"abc";
        let f2 = b"axc";
        assert_eq!(diff_files(f1, f2), vec![(1, b'b')]);
    }

    #[test]
    fn diff_files_same_length_identical() {
        let f1 = b"abc";
        let f2 = b"abc";
        assert!(diff_files(f1, f2).is_empty());
    }

    #[test]
    fn diff_files_file1_longer() {
        let f1 = b"abcd";
        let f2 = b"ab";
        assert_eq!(diff_files(f1, f2), vec![(2, b'c'), (3, b'd')]);
    }

    #[test]
    fn diff_files_file2_longer() {
        let f1 = b"ab";
        let f2 = b"abcd";
        assert_eq!(diff_files(f1, f2), vec![(2, b'c'), (3, b'd')]);
    }

    #[test]
    fn diff_files_both_differ_and_length_differs() {
        let f1 = b"ax";
        let f2 = b"ayz";
        assert_eq!(diff_files(f1, f2), vec![(1, b'x'), (2, b'z')]);
    }
}
