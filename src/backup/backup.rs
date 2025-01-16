use crate::backup_sets::backup_set::create_empty_set;
use crate::dhcopy::copy_folder::copy_folder;
use crate::test_helpers::test_helpers::create_tmp_folder;
use chrono::Utc;
use std::fs;
use std::io;
use std::path::Path;

const DEEP_PATH: &str = "thats/deep";
const BACKUP_FOLDER_NAME: &str = "backups";

#[derive(Debug)]
pub struct BackupOptions {
    pub max_space: Option<u64>,
    pub validate_checksums: bool,
}

impl Default for BackupOptions {
    fn default() -> Self {
        BackupOptions {
            max_space: None,
            validate_checksums: false,
        }
    }
}

pub fn backup(source: &str, dest: &str, options: Option<BackupOptions>) -> io::Result<String> {
    let options = options.unwrap_or_default();
    
    fs::create_dir_all(dest)?;
    let set_name = create_empty_set(dest, || Utc::now())?;
    
    if let Some(max_space) = options.max_space {
        manage_backup_space(dest, max_space)?;
    }
    
    let dest_folder = Path::new(dest).join(&set_name);
    println!("backing up {} into {:?}", source, dest_folder);
    copy_folder(source, dest_folder.to_str().unwrap())?;
    Ok(set_name)
}

fn manage_backup_space(backup_root: &str, max_space: u64) -> io::Result<()> {
    // TODO: Implement space management
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup() -> io::Result<()> {
        let source = create_source()?;
        let dest = create_tmp_folder(BACKUP_FOLDER_NAME)?;

        // smoke test
        let set_name = backup(&source, &dest, None)?;

        // Just a quick check that deeply nested file is copied.
        // All other edge cases are tested in unit tests.
        let test_file_path = Path::new(&dest)
            .join(&set_name)
            .join(DEEP_PATH)
            .join("testfile.txt");
        assert!(
            test_file_path.exists(),
            "test file should be copied to backup folder"
        );

        // cleanup
        let _ = fs::remove_dir_all(&source);
        Ok(())
    }

    #[test]
    fn test_backup_with_options() -> io::Result<()> {
        let source = create_source()?;
        let dest = create_tmp_folder(BACKUP_FOLDER_NAME)?;

        let options = BackupOptions {
            max_space: Some(1024 * 1024 * 1024), // 1GB
            validate_checksums: true,
        };

        let set_name = backup(&source, &dest, Some(options))?;
        assert!(Path::new(&dest).join(&set_name).exists());

        Ok(())
    }

    // ... rest of existing tests ...
}

#[test]
fn test_backup_non_existent_path() {
	// todo
}

#[test]
fn test_creates_destination_folder() -> io::Result<()> {
	let source = create_source()?;
	let dest = create_tmp_folder(BACKUP_FOLDER_NAME)?;

	let non_existent_destination = Path::new(&dest).join("to-be-created");

	backup(&source, non_existent_destination.to_str().unwrap())?;

	let dir = fs::read_dir(&non_existent_destination)?;
	assert!(dir.count() > 0, "destination folder should be copied");

	// cleanup
	let _ = fs::remove_dir_all(&source);
	Ok(())
}

fn create_source() -> io::Result<String> {
	let source = create_tmp_folder("orig")?;

	let folder_path = Path::new(&source).join(DEEP_PATH);
	fs::create_dir_all(&folder_path)?;

	let test_file_name = folder_path.join("testfile.txt");
	let the_text = "backmeup susie";
	fs::write(test_file_name, the_text)?;

	Ok(source)
}
 