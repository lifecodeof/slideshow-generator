use std::path::PathBuf;
use image::GenericImageView;

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

    /// Get the dimensions of the image
    pub fn dimensions(&self) -> anyhow::Result<(u32, u32)> {
        let img = image::open(&self.path)?;
        Ok(img.dimensions())
    }
}
