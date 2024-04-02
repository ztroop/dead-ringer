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
/// differing byte and the byte itself.
pub fn diff_files(file1: &[u8], file2: &[u8]) -> Vec<(usize, u8)> {
    file1
        .iter()
        .zip(file2.iter())
        .enumerate()
        .filter_map(|(i, (&b1, &b2))| if b1 != b2 { Some((i, b1)) } else { None })
        .collect()
}
