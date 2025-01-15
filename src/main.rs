// src/main.rs
use clap::{Parser, Subcommand};
use disk_hog_backup::{BackupConfig, BackupManager};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new backup
    Backup {
        /// Source directory to backup
        #[arg(short, long)]
        source: PathBuf,

        /// Destination directory for backups
        #[arg(short, long)]
        dest: PathBuf,

        /// Maximum space to use (in GB)
        #[arg(short, long, default_value = "1000")]
        max_space: u64,
    },

    /// Validate existing backups
    Validate {
        /// Backup root directory
        #[arg(short, long)]
        backup_root: PathBuf,
    },

    /// List existing backups
    List {
        /// Backup root directory
        #[arg(short, long)]
        backup_root: PathBuf,
    },
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup { source, dest, max_space } => {
            let config = BackupConfig {
                source_dir: source,
                backup_root: dest,
                max_space: max_space * 1024 * 1024 * 1024, // Convert GB to bytes
            };

            let mut manager = BackupManager::new(config)?;
            let backup_info = manager.create_backup()?;
            println!("Backup created successfully!");
            println!("Total size: {} bytes", backup_info.total_size);
            println!("Files backed up: {}", backup_info.files.len());
        }

        Commands::Validate { backup_root } => {
            let config = BackupConfig {
                source_dir: PathBuf::new(), // Not needed for validation
                backup_root,
                max_space: 0, // Not needed for validation
            };

            let manager = BackupManager::new(config)?;
            let issues = manager.validate_backups()?;
            
            if issues.is_empty() {
                println!("All backups are valid!");
            } else {
                println!("Found {} issues:", issues.len());
                for (path, issue) in issues {
                    println!("{}: {}", path.display(), issue);
                }
            }
        }

        Commands::List { backup_root } => {
            let config = BackupConfig {
                source_dir: PathBuf::new(), // Not needed for listing
                backup_root,
                max_space: 0, // Not needed for listing
            };

            let manager = BackupManager::new(config)?;
            println!("Available backups:");
            for backup in manager.backups {
                println!("Date: {}", backup.date);
                println!("Size: {} bytes", backup.total_size);
                println!("Files: {}", backup.files.len());
                println!("---");
            }
        }
    }

    Ok(())
}