// src/lib.rs
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::os::unix::fs::{MetadataExt, linkcat};
use sha2::{Sha256, Digest};

#[derive(Debug)]
pub struct BackupConfig {
    source_dir: PathBuf,
    backup_root: PathBuf,
    max_space: u64,  // Maximum space to use in bytes
}

#[derive(Debug)]
pub struct FileInfo {
    path: PathBuf,
    size: u64,
    modified: DateTime<Utc>,
    checksum: String,
}

#[derive(Debug)]
pub struct BackupInfo {
    date: DateTime<Utc>,
    files: Vec<FileInfo>,
    total_size: u64,
}

#[derive(Debug)]
pub struct BackupManager {
    config: BackupConfig,
    backups: Vec<BackupInfo>,
}

impl BackupManager {
    pub fn new(config: BackupConfig) -> io::Result<Self> {
        let backups = Self::load_existing_backups(&config.backup_root)?;
        Ok(BackupManager { config, backups })
    }

    fn load_existing_backups(backup_root: &Path) -> io::Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        if !backup_root.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(backup_root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(backup_info) = Self::load_backup_info(&path)? {
                    backups.push(backup_info);
                }
            }
        }

        backups.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(backups)
    }

    fn load_backup_info(backup_dir: &Path) -> io::Result<Option<BackupInfo>> {
        let metadata_path = backup_dir.join(".backup-info");
        if !metadata_path.exists() {
            return Ok(None);
        }

        // Load and parse backup metadata
        // This is a placeholder - implement proper serialization
        Ok(None)
    }

    pub fn create_backup(&mut self) -> io::Result<BackupInfo> {
        let now = Utc::now();
        let backup_dir = self.config.backup_root.join(now.format("%Y-%m-%d_%H-%M-%S").to_string());
        fs::create_dir_all(&backup_dir)?;

        let mut files = Vec::new();
        let mut total_size = 0;

        self.backup_directory(&self.config.source_dir, &backup_dir, &mut files, &mut total_size)?;

        let backup_info = BackupInfo {
            date: now,
            files,
            total_size,
        };

        self.backups.push(backup_info.clone());
        self.manage_space()?;

        Ok(backup_info)
    }

    fn backup_directory(&self, source: &Path, dest: &Path, files: &mut Vec<FileInfo>, total_size: &mut u64) -> io::Result<()> {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.config.source_dir).unwrap();
            let dest_path = dest.join(relative_path);

            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                self.backup_directory(&path, dest, files, total_size)?;
            } else {
                self.backup_file(&path, &dest_path, files, total_size)?;
            }
        }
        Ok(())
    }

    fn backup_file(&self, source: &Path, dest: &Path, files: &mut Vec<FileInfo>, total_size: &mut u64) -> io::Result<()> {
        let metadata = fs::metadata(source)?;
        let checksum = self.calculate_checksum(source)?;

        // Check if we can hard link from a previous backup
        if let Some(existing_file) = self.find_existing_file(&checksum) {
            fs::hard_link(existing_file, dest)?;
        } else {
            fs::copy(source, dest)?;
        }

        // Preserve timestamps
        let atime = filetime::FileTime::from_unix_time(metadata.atime(), metadata.atime_nsec() as u32);
        let mtime = filetime::FileTime::from_unix_time(metadata.mtime(), metadata.mtime_nsec() as u32);
        filetime::set_file_times(dest, atime, mtime)?;

        files.push(FileInfo {
            path: dest.to_path_buf(),
            size: metadata.len(),
            modified: DateTime::from_timestamp(metadata.mtime(), 0).unwrap_or_else(|| Utc::now()),
            checksum,
        });

        *total_size += metadata.len();
        Ok(())
    }

    fn calculate_checksum(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn find_existing_file(&self, checksum: &str) -> Option<PathBuf> {
        for backup in &self.backups {
            for file in &backup.files {
                if file.checksum == checksum {
                    return Some(file.path.clone());
                }
            }
        }
        None
    }

    fn manage_space(&mut self) -> io::Result<()> {
        // Calculate total space used
        let mut total_used = 0;
        for backup in &self.backups {
            total_used += backup.total_size;
        }

        // Remove oldest backups if we're over the limit
        while total_used > self.config.max_space && self.backups.len() > 1 {
            if let Some(oldest_backup) = self.backups.pop() {
                let backup_dir = oldest_backup.files.first().unwrap().path.parent().unwrap();
                fs::remove_dir_all(backup_dir)?;
                total_used -= oldest_backup.total_size;
            }
        }

        Ok(())
    }

    pub fn validate_backups(&self) -> io::Result<Vec<(PathBuf, String)>> {
        let mut issues = Vec::new();

        for backup in &self.backups {
            for file in &backup.files {
                if !file.path.exists() {
                    issues.push((file.path.clone(), "File missing".to_string()));
                    continue;
                }

                let current_checksum = self.calculate_checksum(&file.path)?;
                if current_checksum != file.checksum {
                    issues.push((file.path.clone(), "Checksum mismatch".to_string()));
                }
            }
        }

        Ok(issues)
    }
}

// Add necessary test modules
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_backup_creation() -> io::Result<()> {
        let source_dir = tempdir()?;
        let backup_root = tempdir()?;
        
        // Create some test files
        fs::write(source_dir.path().join("test1.txt"), b"test content 1")?;
        fs::write(source_dir.path().join("test2.txt"), b"test content 2")?;

        let config = BackupConfig {
            source_dir: source_dir.path().to_path_buf(),
            backup_root: backup_root.path().to_path_buf(),
            max_space: 1024 * 1024 * 1024, // 1GB
        };

        let mut manager = BackupManager::new(config)?;
        let backup_info = manager.create_backup()?;

        assert_eq!(backup_info.files.len(), 2);
        Ok(())
    }
}