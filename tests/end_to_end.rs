use assert_cmd::Command;
use tempfile::tempdir;
use std::fs;
use std::io::Write;
use std::path::Path;

#[test]
fn test_disk_hog_backup_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    // Set up source directory
    let src_dir = tempdir()?;
    let src_file_path = src_dir.path().join("test_file.txt");
    let mut src_file = fs::File::create(&src_file_path)?;
    writeln!(src_file, "This is a test file.")?;

    // Set up destination directory
    let dest_dir = tempdir()?;

    // Print debug information
    println!("Source directory: {:?}", src_dir.path());
    println!("Destination directory: {:?}", dest_dir.path());
    println!("Source file path: {:?}", src_file_path);

    let output = Command::cargo_bin("disk_hog_backup")?
        .arg("--source")
        .arg(src_dir.path())
        .arg("--destination")
        .arg(dest_dir.path())
        .output()?;

    println!("Command stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Command stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Extract backup set name from stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    let set_name = stdout
        .lines()
        .find(|line| line.starts_with("Backup successful: created set"))
        .and_then(|line| line.split_whitespace().last())
        .ok_or("Couldn't find backup set name in output")?;

    // Verify backup in the correct backup set directory
    let backup_file_path = dest_dir.path()
        .join(set_name)
        .join("test_file.txt");
    
    println!("Expected backup file path: {:?}", backup_file_path);
    
    assert!(backup_file_path.exists(), "Backup file does not exist at {:?}", backup_file_path);
    let original_content = fs::read_to_string(src_file_path)?;
    let backup_content = fs::read_to_string(backup_file_path)?;
    assert_eq!(original_content, backup_content, "File contents do not match.");

    Ok(())
}