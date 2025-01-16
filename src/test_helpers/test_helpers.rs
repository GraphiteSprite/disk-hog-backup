#[cfg(test)]
use chrono::{DateTime, Utc};
#[cfg(test)]
use rand::Rng;
#[cfg(test)]
use std::env;
#[cfg(test)]
use std::fs::{self, File};
#[cfg(test)]
use std::io::{self, Read};
#[cfg(test)]
use std::path::Path;

#[cfg(test)]
pub fn create_tmp_folder(prefix: &str) -> io::Result<String> {
    let mut rng = rand::thread_rng();
    let random_suffix: u32 = rng.gen();
    let dir = env::temp_dir().join(format!("dhb-{}-{}", prefix, random_suffix));
    fs::create_dir_all(&dir)?;
    Ok(dir.to_string_lossy().into_owned())
}

#[cfg(test)]
pub fn file_contents_matches(file1_path: &str, file2_path: &str) -> io::Result<bool> {
    let file1_contents = read_contents(file1_path)?;
    let file2_contents = read_contents(file2_path)?;
    Ok(file1_contents == file2_contents)
}

#[cfg(test)]
fn read_contents<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
pub fn time_fixer() -> impl Fn() -> DateTime<Utc> {
    let fixed_time = Utc::now();
    move || fixed_time
}
