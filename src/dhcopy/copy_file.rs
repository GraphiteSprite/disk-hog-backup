use crate::test_helpers::test_helpers::{create_tmp_folder, file_contents_matches};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const THE_FILE: &str = "testfile.txt";
const THE_TEXT: &str = "backmeup susie";

#[test]
fn test_copy() -> io::Result<()> {
	let source_folder = create_tmp_folder("orig")?;
	let dest = create_tmp_folder("backups")?;

	let source_file_path = Path::new(&source_folder).join(THE_FILE);
	let mut source_file = fs::File::create(&source_file_path)?;
	source_file.write_all(THE_TEXT.as_bytes())?;

	let destination_file_path = Path::new(&dest).join(THE_FILE);

	copy_file(&source_file_path, &destination_file_path)?;

	let contents_matches = file_contents_matches(
		&source_file_path.to_string_lossy(),
		&destination_file_path.to_string_lossy(),
	)?;
	assert!(
		contents_matches,
		"file contents should be copied to backup folder"
	);

	Ok(())
}

// copy_file.rs - Add hard linking support
fn copy_file(source: &Path, dest: &Path) -> io::Result<u64> {
    // Try to create hard link first
    match fs::hard_link(source, dest) {
        Ok(_) => Ok(fs::metadata(source)?.len()),
        Err(_) => fs::copy(source, dest) // Fall back to regular copy
    }
}
