use std::path::PathBuf;

pub struct Image {
    pub path: PathBuf,
}

impl Image {
    pub fn new(path: PathBuf) -> Self {
        Image { path }
    }

    pub fn load(&self) -> anyhow::Result<()> {
        // Logic to load the image file
        Ok(())
    }

    pub fn process(&self) -> anyhow::Result<()> {
        // Logic to process the image file
        Ok(())
    }
}
