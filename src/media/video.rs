use std::path::PathBuf;

pub struct Video {
    pub path: PathBuf,
}

impl Video {
    pub fn new(path: PathBuf) -> Self {
        Video { path }
    }

    pub fn load(&self) -> anyhow::Result<()> {
        // Logic to load the video file
        Ok(())
    }

    pub fn process(&self) -> anyhow::Result<()> {
        // Logic to process the video file
        Ok(())
    }
}
