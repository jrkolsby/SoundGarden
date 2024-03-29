use std::path::Path;
use std::fs::{self, File};
use std::io;

pub const PALIT_PROJECTS: &str = "./";
pub const PALIT_MODULES: &str = "./modules/";

pub fn get_files(path: &str, file_type: &str, mut collection: Vec<String>) -> io::Result<Vec<String>> {
    let dir = Path::new(path);
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                if let Some(path_type) = path.extension() {
                    if path_type == file_type {
                        collection.push(entry.file_name().into_string().unwrap());
                    }
                }
            }
        }
    }
    Ok(collection)
}