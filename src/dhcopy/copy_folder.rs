use std::fs;
use std::io;
use std::path::Path;
use crate::dhcopy::copy_file::copy_file;

/// Recursively copies the contents of a source folder to a destination folder.
pub fn copy_folder(source: &str, dest: &str) -> io::Result<()> {
    println!("backing up folder {} into {}", source, dest);
    let contents = fs::read_dir(source)?;

    for entry in contents {
        let entry = entry?;
        let path = entry.path();
        let dest_path = Path::new(dest).join(entry.file_name());

        if path.is_dir() {
            // Recursively create directories and copy their contents
            fs::create_dir_all(&dest_path)?;
            copy_folder(path.to_str().unwrap(), dest_path.to_str().unwrap())?;
        } else {
            // Copy individual files
            copy_file(&path, &dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_helpers::create_tmp_folder;

    const EMPTY_FOLDER: &str = "NothingInHere";
    const BACKUP_FOLDER_NAME: &str = "backups";
    const THE_FILE: &str = "testfile.txt";
    const THE_TEXT: &str = "backmeup susie";

    /// Helper function to create a temporary source folder.
    fn create_source() -> io::Result<String> {
        let source = create_tmp_folder("orig")?;
        Ok(source)
    }

    /// Helper function to create a test file with specified contents.
    fn make_test_file(folder_path: &str, filename: &str, contents: &str) -> io::Result<()> {
        let deep_test_file_name = Path::new(folder_path).join(filename);
        fs::write(deep_test_file_name, contents)?;
        Ok(())
    }

    /// Helper function to verify that an empty folder was copied correctly.
    fn check_empty_folder_copied(dest: &str) -> io::Result<()> {
        let dir_path = Path::new(dest).join(EMPTY_FOLDER);
        let dir = fs::read_dir(&dir_path)?;
        assert_eq!(
            dir.count(),
            0,
            "empty folder in source should be empty in backup"
        );
        Ok(())
    }

    #[test]
    fn test_copies_file() -> io::Result<()> {
        let source = create_source()?;
        make_test_file(&source, THE_FILE, THE_TEXT)?;
        let dest = create_tmp_folder(BACKUP_FOLDER_NAME)?;

        copy_folder(&source, &dest)?;

        let test_file_path = Path::new(&dest).join(THE_FILE);
        assert!(
            test_file_path.exists(),
            "test file should be copied to backup folder"
        );

        // Cleanup
        let _ = fs::remove_dir_all(&source);
        Ok(())
    }

    #[test]
    fn test_copy_empty_folder() -> io::Result<()> {
        let source = create_source()?;
        let _ = fs::remove_dir_all(&source);

        let empty_folder_path = Path::new(&source).join(EMPTY_FOLDER);
        fs::create_dir_all(&empty_folder_path)?;

        let dest = create_tmp_folder(BACKUP_FOLDER_NAME)?;

        copy_folder(&source, &dest)?;

        check_empty_folder_copied(&dest)?;

        Ok(())
    }
}
