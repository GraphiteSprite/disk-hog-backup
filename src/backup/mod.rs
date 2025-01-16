// src/backup/mod.rs

// Import items from the backup_impl.rs module
pub mod backup_impl;

// Re-export the `backup` function and `BackupOptions` for easier use
pub use backup_impl::{backup_with_options as backup, BackupOptions};
