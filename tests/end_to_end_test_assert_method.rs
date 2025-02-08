use std::{fs, io};
use std::io::BufRead;
use std::path::Path;
use assert_cmd::Command as AssertCommand;
use predicates::path::exists;
use predicates::prelude::predicate;
use disk_hog_backup::backup::backup::backup;
use disk_hog_backup::test_helpers::test_helpers::create_tmp_folder;
use crate::common::test_helpers::create_source;
use regex::Regex;

mod common;

const BACKUP_FOLDER_NAME: &str = "backups";

// As seen in gitopolis, assert_cmd finds the crate's binary to test, then assert on the result of the program's run
fn diskhog_executable() -> AssertCommand {
    AssertCommand::cargo_bin("disk-hog-backup").expect("failed to find binary")
}

#[test]
// ./target/debug/disk-hog-backup --help
fn disk_hog_end_to_end_help() {
    diskhog_executable()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: disk-hog-backup --source <SOURCE> --destination <DESTINATION>"));
}



#[test]
// ./target/debug/disk-hog-backup --source --destination
fn disk_hog_end_to_end_source_destination_regex() {
    let source = create_source().expect("failed to create source");
    let dest = create_tmp_folder(BACKUP_FOLDER_NAME).expect("failed to create tmp folder");

    let output = diskhog_executable()
        .args(["--source", &source, "--destination", BACKUP_FOLDER_NAME])
        .output().expect("failed to execute process");
    assert!(output.status.success());


    let message = String::from_utf8_lossy(&output.stdout); // Could this be cleaned up? by getting the output string in a nicer way; is utf8lossy needed?
    let re = Regex::new(r#"backups\/([^"]*)""#).unwrap(); // Is there a nicer way to do this that doesn't use Regex?
    let mut results = vec![];
    for (_, [path_name]) in re.captures_iter(&*message).map(|c| c.extract()) {
        results.push(path_name);
    }

    let name = Path::new(BACKUP_FOLDER_NAME);
    let full_path = name.join(results.first().unwrap());

    assert!(full_path.exists()); // Check if the destiation files exit
    // TODO add check for thats/deep/testfile.txt
}





#[test]
fn disk_hog_end_to_end_source_destination_using_lines() {
    let source = create_source().expect("failed to create source");
    let dest = create_tmp_folder(BACKUP_FOLDER_NAME).expect("failed to create tmp folder");

    let output = diskhog_executable()
        .args(["--source", &source, "--destination", BACKUP_FOLDER_NAME])
        .output().expect("failed to execute process");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)
        .expect("Command output was not valid UTF-8");

    let backup_path = stdout
        .lines()
        .find(|line| dbg!(line).contains("backing up /tmp"))
        .and_then(|line| dbg!(line).split("backups/").nth(1))
        .map(|path| dbg!(path).trim_matches('"'))
        .expect("Could not find backup path in output");

    let name = Path::new(BACKUP_FOLDER_NAME);
    let full_path = name.join(backup_path);

    assert!(full_path.exists()); // Check if the destiation files exit
    // TODO add check for thats/deep/testfile.txt

}

// TODO Extract path extraction method into a function that can be used elsewhere
//
// let stdout = String::from_utf8(output.stdout)
// .expect("Command output was not valid UTF-8");
//
// let backup_path = stdout
// .lines()
// .find(|line| dbg!(line).contains("backing up /tmp"))
// .and_then(|line| dbg!(line).split("backups/").nth(1))
// .map(|path| dbg!(path).trim_matches('"'))
// .expect("Could not find backup path in output");


//
// #[test]
// fn disk_hog_end_to_end_source_destination_claude() {
//     // Setup phase
//     let source = create_source().expect("failed to create source");
//     let dest = create_tmp_folder(BACKUP_FOLDER_NAME).expect("failed to create tmp folder");
//
//     // Execute phase
//     let output = diskhog_executable()
//         .args(["--source", &source, "--destination", BACKUP_FOLDER_NAME])
//         .output()
//         .expect("failed to execute process");
//     assert!(output.status.success());
//
//     // Parse output phase
//     let stdout = String::from_utf8(output.stdout)
//         .expect("Command output was not valid UTF-8");
//
//     // Extract backup path (assumes format "Created backup at: backups/filename")
//     let backup_path = stdout
//         .lines()
//         .find(|line| line.contains("Created backup at:"))
//         .and_then(|line| line.split("backups/").nth(1))
//         .map(|path| path.trim_matches('"'))
//         .expect("Could not find backup path in output");
//
//     // Verify phase
//     let backup_full_path = Path::new(BACKUP_FOLDER_NAME).join(backup_path);
//     assert!(
//         backup_full_path.exists(),
//         "Backup file not found at: {:?}",
//         backup_full_path
//     );
//
//     // Verify expected test file exists
//     let test_file_path = backup_full_path.join("thats/deep/testfile.txt");
//     assert!(
//         test_file_path.exists(),
//         "Test file not found at: {:?}",
//         test_file_path
//     );
// }

//  Implementation options:
// This method could be it's own separate function and could be called back into the test:
//    ``` let re = Regex::new(r#"backups\/([^"]*)""#).unwrap(); // Is there a nicer way to do this that doesn't use Regex?
//     let mut results = vec![];
//     for (_, [path_name]) in re.captures_iter(&*message).map(|c| c.extract()) {
//         results.push(path_name);
//     }```
//
// let backup_path = stdout
// .lines()
// .find(|line| line.contains("Created backup at:"))
// .and_then(|line| line.split("backups/").nth(1))
// .map(|path| path.trim_matches('"'))
// .expect("Could not find backup path in output");
//


// backing up /tmp/dhb-orig-2417924247 into "backups/dhb-set-20250207-160118"
// backing up folder /tmp/dhb-orig-2417924247 into backups/dhb-set-20250207-160118
// backing up folder /tmp/dhb-orig-2417924247/thats into backups/dhb-set-20250207-160118/thats
// backing up folder /tmp/dhb-orig-2417924247/thats/deep into backups/dhb-set-20250207-160118/thats/deep
// Backup successful

// #[test]
// fn disk_hog_end_to_end_verify_contents() {
//     let source = create_source().expect("failed to create source");
//     let dest = create_tmp_folder(BACKUP_FOLDER_NAME).expect("failed to create tmp folder");
//
//     diskhog_executable()
//         .args(["--source", &source, "--destination", BACKUP_FOLDER_NAME])
//         .assert()
//         .success();
//
//     // Verify directory structure matches
//     let verify_tree = |dir1: &Path, dir2: &Path| -> io::Result<()> {
//         for entry in fs::read_dir(dir1)? {
//             let entry = entry?;
//             let path1 = entry.path();
//             let path2 = dir2.join(entry.file_name());
//
//             if path1.is_dir() {
//                 assert!(path2.is_dir(), "Directory structure mismatch");
//                 verify_tree(&path1, &path2)?;
//             } else {
//                 // Compare file contents
//                 let content1 = fs::read(&path1)?;
//                 let content2 = fs::read(&path2)?;
//                 assert_eq!(content1, content2, "File contents mismatch");
//             }
//         }
//         Ok(())
//     };
//
//     verify_tree(Path::new(&source), Path::new(BACKUP_FOLDER_NAME))
//         .expect("Failed to verify directory contents");
// }