// src/backup/backup_impl.rs
use chrono::Utc;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::Path;
use log::info;

use crate::backup_sets::backup_set::create_empty_set;
use crate::dhcopy::copy_folder::copy_folder;
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

// Function for calculating file checksum
fn calculate_checksum(path: &Path) -> io::Result<u64> {
    let content = fs::read(path)?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(hasher.finish())
}

// Updated backup function with options
pub fn backup_with_options(source: &str, dest: &str, options: Option<BackupOptions>) -> io::Result<String> {
    let options = options.unwrap_or_default(); // Use default options if none provided

    // Ensure destination directory exists
    fs::create_dir_all(dest)?;
    let set_name = create_empty_set(dest, || Utc::now())?;

    // Manage backup space if max_space is set
    if let Some(max_space) = options.max_space {
        manage_backup_space(dest, max_space)?;
    }

    let dest_folder = Path::new(dest).join(&set_name);
    info!("Backing up {} into {:?}", source, dest_folder);  // Corrected usage of log::info!

    // Copy source folder to destination
    copy_folder(source, dest_folder.to_str().unwrap())?;

    // Validate checksums if the option is enabled
    if options.validate_checksums {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            if entry.path().is_file() {
                let source_checksum = calculate_checksum(&entry.path())?;
                let dest_checksum = calculate_checksum(&dest_folder.join(entry.file_name()))?;
                assert_eq!(
                    source_checksum, dest_checksum,
                    "Checksum mismatch for file: {:?}",
                    entry.path()
                );
            }
        }
    }

    Ok(set_name)
}

// Function to manage backup space by removing old backups
fn manage_backup_space(backup_root: &str, max_space: u64) -> io::Result<()> {
    let path = Path::new(backup_root);
    let mut total_size = 0;
    let mut backup_sets = Vec::new();

    // Ensure the backup root exists
    if path.exists() {
        log::info!("Backup root exists: {:?}", path);
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let metadata = entry.metadata()?;
                total_size += metadata.len();
                backup_sets.push((entry.path(), metadata.len()));
                log::info!("Backup set: {:?}, size: {}", entry.path(), metadata.len());
            }
        }
    } else {
        log::warn!("Backup root does not exist: {:?}", path);
    }

    log::info!("Total size after collecting backups: {}", total_size);

    // If no backups were collected, skip cleanup
    if backup_sets.is_empty() {
        log::info!("No backup sets to clean up.");
        return Ok(());
    }

    // Sort backups by modification time (oldest first)
    backup_sets.sort_by(|a, b| {
        fs::metadata(&a.0)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now())
            .cmp(
                &fs::metadata(&b.0)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::now()),
            )
    });

    log::info!("Backup sets sorted. Starting cleanup...");

    // Remove oldest backups until total size is under the limit
    while total_size > max_space && !backup_sets.is_empty() {
        if let Some((path, size)) = backup_sets.first() {
            log::info!("Removing backup: {:?}, size: {}", path, size);
            fs::remove_dir_all(path)?;
            total_size -= size;
            backup_sets.remove(0);
        }
    }

    log::info!("Cleanup completed. Total size: {}", total_size);

    Ok(())
}


// Tests
#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::test_helpers::test_helpers::{create_tmp_folder, file_contents_matches};

    #[test]
    fn test_backup_with_options() -> io::Result<()> {
        let source = create_tmp_folder("test-source")?;
        let dest = create_tmp_folder("test-backup")?;
    
        // Create a test file
        let test_file = Path::new(&source).join("test.txt");
        fs::write(&test_file, "test content")?;
    
        let options = BackupOptions {
            max_space: Some(1024 * 1024), // 1MB
            validate_checksums: true,
        };
    
        let set_name = backup_with_options(&source, &dest, Some(options))?;
        let backup_path = Path::new(&dest).join(&set_name).join("test.txt");
    
        // Verify backup
        assert!(
            file_contents_matches(
                &backup_path.to_string_lossy(),
                &test_file.to_string_lossy()
            )?,
            "Backup file contents should match source file"
        );
    
        Ok(())
    }

    #[test]
    fn test_manage_backup_space() -> io::Result<()> {
        env_logger::init(); // Initialize logger
        log::info!("Starting test_manage_backup_space");
    
        let temp_dir = TempDir::new()?;
        let backup_root = temp_dir.path();
    
        // Create some test backup sets with large files
        let mut total_size = 0;
        for i in 0..3 {
            let set_path = Path::new(&backup_root).join(format!("backup_{}", i));
            fs::create_dir(&set_path)?;
            let large_file_path = set_path.join("large_file.txt");
            let large_content = vec![0u8; 1024 * 1024]; // 1MB file
            fs::write(&large_file_path, &large_content)?;
    
            // Log each backup set created
            log::info!("Created backup set: {:?}", set_path);
            log::info!("Created file at: {:?}", large_file_path);
            
            total_size += 1024 * 1024; // Update total size
        }
    
        // Set max space to a value that will trigger cleanup
        let max_space = 2 * 1024 * 1024; // 2MB limit
        log::info!(
            "Total size of backups before cleanup: {}, Max space: {}",
            total_size, max_space
        );
    
        // Run the backup space management function
        manage_backup_space(&backup_root.to_string_lossy(), max_space)?;
    
        // Verify that some backups were removed
        let remaining_backups: Vec<_> = fs::read_dir(&backup_root)?
            .filter_map(Result::ok)
            .collect();
        let remaining_size: u64 = remaining_backups
            .iter()
            .map(|entry| {
                entry
                    .path()
                    .join("large_file.txt")
                    .metadata()
                    .map(|metadata| metadata.len())
                    .unwrap_or(0)
            })
            .sum();
    
        log::info!(
            "Total size of backups after cleanup: {}, Remaining backups: {}",
            remaining_size,
            remaining_backups.len()
        );
    
        assert!(
            remaining_backups.len() < 3,
            "Should have removed some backups"
        );
        assert!(
            remaining_size <= max_space,
            "Remaining size exceeds the max space limit"
        );
    
        Ok(())
    }
    

}