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
                let size = calculate_dir_size(&entry.path())?; // New helper function
                total_size += size;
                backup_sets.push((entry.path(), size));
                log::info!("Found backup set: {:?}, size: {}, modified: {:?}", 
                    entry.path(), 
                    size,
                    entry.metadata()?.modified()?
                );
            }
        }
    }

    log::info!("Before cleanup: total_size={}, max_space={}, sets={}", 
        total_size, max_space, backup_sets.len());

    // Sort backups by modification time (oldest first)
    backup_sets.sort_by(|a, b| {
        let a_time = fs::metadata(&a.0).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::now());
        let b_time = fs::metadata(&b.0).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::now());
        log::info!("Comparing: {:?} ({:?}) vs {:?} ({:?})", 
            a.0, a_time, b.0, b_time);
        a_time.cmp(&b_time)
    });

    log::info!("Sorted backup sets (oldest first): {:?}", 
        backup_sets.iter().map(|(path, size)| format!("{:?} ({})", path, size)).collect::<Vec<_>>());

    // Remove oldest backups until total size is under the limit
    while total_size > max_space && !backup_sets.is_empty() {
        if let Some((path, size)) = backup_sets.first() {
            log::info!("Attempting to remove backup: {:?}, size: {}", path, size);
            match fs::remove_dir_all(path) {
                Ok(_) => {
                    log::info!("Successfully removed {:?}", path);
                    total_size -= size;
                }
                Err(e) => {
                    log::error!("Failed to remove {:?}: {}", path, e);
                }
            }
            backup_sets.remove(0);
        }
    }

    log::info!("After cleanup: total_size={}, max_space={}, remaining_sets={}", 
        total_size, max_space, backup_sets.len());

    Ok(())
}

// Helper function to calculate directory size
fn calculate_dir_size(path: &Path) -> io::Result<u64> {
    let mut total = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += calculate_dir_size(&entry.path())?;
        }
    }
    Ok(total)
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
        let max_space = 2 * 1024 * 1024; // 2MB limit
    
        // Create some test backup sets with large files
        let mut total_size = 0;
        let mut backup_sets = Vec::new();
        for i in 0..3 {
            let set_path = Path::new(&backup_root).join(format!("backup_{}", i));
            fs::create_dir(&set_path)?;
            let large_file_path = set_path.join("large_file.txt");
            let large_content = vec![0u8; 1024 * 1024]; // 1MB file
            fs::write(&large_file_path, &large_content)?;
    
            backup_sets.push(set_path.clone());
            total_size += 1024 * 1024; // Update total size
    
            log::info!("Created backup set {}: {:?}", i, set_path);
            log::info!("Created file at: {:?}", large_file_path);

                    // Add a small delay between creating backups to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let backup_times: Vec<_> = backup_sets.iter().map(|path| {
            fs::metadata(&path).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::now())
        }).collect();
        println!("Backup modification times: {:?}", backup_times);
    
        println!("Initial state:");
        println!("  - Total size: {} bytes", total_size);
        println!("  - Space limit: {} bytes", max_space);
        println!("  - Number of backup sets: {}", backup_sets.len());
        println!("  - Backup sets: {:?}", backup_sets);
    
        log::info!(
            "Total size of backups before cleanup: {}, Max space: {}",
            total_size, max_space
        );
    
        println!("About to manage backup space...");
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
    
        println!("Final state:");
        println!("  - Remaining size: {} bytes", remaining_size);
        println!("  - Number of remaining backups: {}", remaining_backups.len());
        println!("  - Remaining backup paths: {:?}", remaining_backups);
    
        log::info!(
            "Total size of backups after cleanup: {}, Remaining backups: {}",
            remaining_size,
            remaining_backups.len()
        );
    
        // Verify our assumptions before the final assertions
        assert!(total_size > max_space, "Initial space should be over limit");
        assert!(!remaining_backups.is_empty(), "Should have some backups remaining");
        
        assert!(
            remaining_backups.len() < 3,
            "Should have removed some backups"
        );
        assert!(
            remaining_size <= max_space,
            "Remaining size {} exceeds the max space limit {}",
            remaining_size,
            max_space
        );
    
        Ok(())
    } 

}