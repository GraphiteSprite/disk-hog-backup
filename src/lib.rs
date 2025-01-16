// src/lib.rs

// Module declarations
pub mod backup;
pub mod backup_sets;
pub mod dhcopy;
pub mod test_helpers;

// Re-export main components
pub use crate::backup::{backup, BackupOptions};
pub use crate::dhcopy::copy_file::copy_file;

// No test code here, as it's in the individual modules
