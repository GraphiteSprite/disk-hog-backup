// TODO: This version of test_helpers is a copy of src/test_helpers/test_helpers.rs. Choose whether to keep both or consolidate in one location

use chrono::{DateTime, Utc};
use rand::Rng;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

const DEEP_PATH: &str = "thats/deep";

pub fn create_tmp_folder(prefix: &str) -> io::Result<String> {
	let mut rng = rand::thread_rng();
	let random_suffix: u32 = rng.gen();
	let dir = env::temp_dir().join(format!("dhb-{}-{}", prefix, random_suffix));
	fs::create_dir_all(&dir)?;
	Ok(dir.to_string_lossy().into_owned())
}

pub fn file_contents_matches(file1_path: &str, file2_path: &str) -> io::Result<bool> {
	let file1_contents = read_contents(file1_path)?;
	let file2_contents = read_contents(file2_path)?;
	Ok(file1_contents == file2_contents)
}

fn read_contents<P: AsRef<Path>>(path: P) -> io::Result<String> {
	let mut file = File::open(path)?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;
	Ok(contents)
}

// Returns a function that always returns the same time
pub fn time_fixer() -> impl Fn() -> DateTime<Utc> {
	let fixed_time = Utc::now();
	move || fixed_time
}

pub fn create_source() -> io::Result<String> {
    let source = create_tmp_folder("orig")?;

    let folder_path = Path::new(&source).join(DEEP_PATH);
    fs::create_dir_all(&folder_path)?;

    let test_file_name = folder_path.join("testfile.txt");
    let the_text = "backmeup susie";
    fs::write(test_file_name, the_text)?;

    Ok(source)
}
