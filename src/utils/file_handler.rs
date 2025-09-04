use std::fs;
use std::path::PathBuf;

pub fn read_files_from_directory(dir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    let paths = fs::read_dir(dir)?;

    for path in paths {
        let path = path?.path();
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

pub fn filter_media_files(files: Vec<PathBuf>) -> Vec<PathBuf> {
    let image_extensions = ["jpg", "jpeg", "png", "gif"];
    let video_extensions = ["mp4", "mov", "avi", "mkv"];

    files.into_iter().filter(|file| {
        if let Some(extension) = file.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            image_extensions.contains(&ext_str.as_ref()) || video_extensions.contains(&ext_str.as_ref())
        } else {
            false
        }
    }).collect()
}
