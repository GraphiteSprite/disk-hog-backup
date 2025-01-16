mod backup;
mod backup_sets;
mod dhcopy;
mod test_helpers;

use clap::Parser;
use std::process;
use crate::backup::{backup, BackupOptions};

#[derive(Parser)]
#[command(name = "diskhog")]
#[command(about = "A tool for backing up directories", long_about = None)]
struct Args {
    /// Source folder to back up
    #[arg(short, long)]
    source: String,

    /// Destination folder for backups
    #[arg(short, long)]
    destination: String,

    /// Maximum space to use in GB (optional)
    #[arg(short, long)]
    max_space: Option<u64>,

    /// Validate checksums during backup
    #[arg(short, long)]
    validate: bool,
}

fn main() {
    env_logger::init(); // Initialize logger for the application
    let args = Args::parse();

    let options = BackupOptions {
        max_space: args.max_space.map(|gb| gb * 1024 * 1024 * 1024),
        validate_checksums: args.validate,
    };

    match backup(&args.source, &args.destination, Some(options)) {
        Ok(set_name) => println!("Backup successful: created set {}", set_name),
        Err(e) => {
            eprintln!("Backup failed: {}", e);
            process::exit(1);
        }
    }
       // Your application code here
       log::info!("Application started");
}