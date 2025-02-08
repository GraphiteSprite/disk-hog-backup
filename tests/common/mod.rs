pub mod test_helpers;

// Key imports
use std::path::Path;         // For path manipulation
use std::{fs, io};                 // File system operations
use tempfile::TempDir;
use crate::common::test_helpers::create_tmp_folder;
// For temporary test directories

const DEEP_PATH: &str = "thats/deep";

// Creates temporary test directory that auto-cleans up
pub fn create_test_folder() -> TempDir {
    TempDir::new().unwrap()  // Creates and returns temp directory
}

// Compares contents of two files
pub fn compare_files(file1: &Path, file2: &Path) -> bool {
    // Read both files to strings
    let content1 = fs::read_to_string(file1).unwrap();
    let content2 = fs::read_to_string(file2).unwrap();
    // Compare contents
    content1 == content2
}

pub fn create_source() -> io::Result<String> {
    let source = create_tmp_folder("orig")?;

    let folder_path = Path::new(&source).join(DEEP_PATH);
    fs::create_dir_all(&folder_path)?;

    let test_file_name = folder_path.join("testfile.txt");
    let the_text = "backmeup susie";
    fs::write(test_file_name, the_text)?;

    Ok(source)
}
