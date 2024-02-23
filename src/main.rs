use file::{diff_files, read_file};
use viewer::FileDiffViewer;

mod file;
mod viewer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file1> <file2>", args[0]);
        std::process::exit(1);
    }

    let file1_data = read_file(&args[1])?;
    let file2_data = read_file(&args[2])?;
    let diffs = diff_files(&file1_data, &file2_data);

    let mut viewer = FileDiffViewer::new(diffs);
    viewer.run()
}
