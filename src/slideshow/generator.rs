use std::path::{Path, PathBuf};
use crate::utils::{read_files_from_directory, filter_media_files};
use anyhow::Result;
use log::{debug, error};

/// Configuration options for slideshow generation
#[derive(Debug, Clone)]
pub struct SlideshowOptions {
    /// Duration in seconds for each image
    pub image_duration: f32,
    /// Output video width
    pub width: u32,
    /// Output video height  
    pub height: u32,
    /// Output video frame rate
    pub fps: u32,
    /// Video codec to use
    pub codec: String,
    /// Pixel format
    pub pixel_format: String,
}

impl Default for SlideshowOptions {
    fn default() -> Self {
        Self {
            image_duration: 3.0,
            width: 1920,
            height: 1080,
            fps: 30,
            codec: "libx264".to_string(),
            pixel_format: "yuv420p".to_string(),
        }
    }
}

impl SlideshowOptions {
    /// Create a new SlideshowOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the duration for each image in seconds
    pub fn with_image_duration(mut self, duration: f32) -> Self {
        self.image_duration = duration;
        self
    }

    /// Set the output resolution
    pub fn with_output_resolution(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the output frame rate
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Set the video codec
    pub fn with_codec(mut self, codec: &str) -> Self {
        self.codec = codec.to_string();
        self
    }

    /// Set the pixel format
    pub fn with_pixel_format(mut self, format: &str) -> Self {
        self.pixel_format = format.to_string();
        self
    }
}

/// Main slideshow generator struct
pub struct SlideshowGenerator {
    images: Vec<PathBuf>,
    videos: Vec<PathBuf>,
    options: SlideshowOptions,
}

impl Default for SlideshowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideshowGenerator {
    /// Create a new empty slideshow generator with default options
    pub fn new() -> Self {
        Self::with_options(SlideshowOptions::default())
    }

    /// Create a new empty slideshow generator with custom options
    pub fn with_options(options: SlideshowOptions) -> Self {
        SlideshowGenerator {
            images: Vec::new(),
            videos: Vec::new(),
            options,
        }
    }

    /// Create a slideshow generator from a directory
    pub fn from_directory<P: AsRef<Path>>(input_dir: P, options: SlideshowOptions) -> Result<Self> {
        let mut generator = Self::with_options(options);
        generator.load_directory(input_dir)?;
        Ok(generator)
    }

    /// Load media files from a directory (legacy method for backward compatibility)
    pub fn load_from_directory<P: AsRef<Path>>(input_dir: P) -> Result<Self> {
        Self::from_directory(input_dir, SlideshowOptions::default())
    }

    /// Load media files from a directory into the current generator
    pub fn load_directory<P: AsRef<Path>>(&mut self, input_dir: P) -> Result<()> {
        let files = read_files_from_directory(input_dir.as_ref().to_str().unwrap())?;
        let media_files = filter_media_files(files);
        
        for file in media_files {
            if let Some(extension) = file.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                match ext.as_str() {
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" => {
                        self.images.push(file);
                    },
                    "mp4" | "mov" | "avi" | "mkv" | "webm" => {
                        self.videos.push(file);
                    },
                    _ => {}
                }
            }
        }
        
        // Sort files by name for consistent ordering
        self.images.sort();
        self.videos.sort();
        
        Ok(())
    }

    /// Add an image file to the slideshow
    pub fn add_image<P: AsRef<Path>>(&mut self, image_path: P) {
        self.images.push(image_path.as_ref().to_path_buf());
    }

    /// Add a video file to the slideshow
    pub fn add_video<P: AsRef<Path>>(&mut self, video_path: P) {
        self.videos.push(video_path.as_ref().to_path_buf());
    }

    /// Get the current slideshow options
    pub fn options(&self) -> &SlideshowOptions {
        &self.options
    }

    /// Update slideshow options
    pub fn set_options(&mut self, options: SlideshowOptions) {
        self.options = options;
    }

    /// Get the number of images in the slideshow
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Get the number of videos in the slideshow
    pub fn video_count(&self) -> usize {
        self.videos.len()
    }

    /// Get the total number of media items
    pub fn total_count(&self) -> usize {
        self.images.len() + self.videos.len()
    }

    /// Generate the slideshow video (modern API)
    pub fn generate<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        self.generate_slideshow(output_path)
    }

    /// Generate the slideshow video (legacy method for backward compatibility)
    pub fn generate_slideshow<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        debug!("Generating slideshow with {} images and {} videos", self.images.len(), self.videos.len());
        
        if self.images.is_empty() && self.videos.is_empty() {
            anyhow::bail!("No media files found to create slideshow");
        }

        // Check if FFmpeg is available
        let ffmpeg_check = std::process::Command::new("ffmpeg")
            .arg("-version")
            .output();

        match ffmpeg_check {
            Ok(output) if output.status.success() => {
                debug!("FFmpeg found, proceeding with video generation...");
            },
            _ => {
                anyhow::bail!("FFmpeg not found. Please install FFmpeg and add it to your PATH.");
            }
        }

        // Create a simple concatenation of media files
        let mut filter_parts = Vec::new();
        let mut input_count = 0;

        // Add images (convert to video segments)
        for (i, _image_path) in self.images.iter().enumerate() {
            filter_parts.push(format!(
                "[{}:v]scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setpts=PTS-STARTPTS,fps={}[img{}]", 
                i, self.options.width, self.options.height, self.options.width, self.options.height, self.options.fps, i
            ));
            input_count += 1;
        }

        // Add videos (scale to same resolution)
        for (i, _video_path) in self.videos.iter().enumerate() {
            let input_idx = self.images.len() + i;
            filter_parts.push(format!(
                "[{}:v]scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setpts=PTS-STARTPTS[vid{}]", 
                input_idx, self.options.width, self.options.height, self.options.width, self.options.height, i
            ));
            input_count += 1;
        }

        // Create concatenation filter
        let mut concat_inputs = Vec::new();
        for i in 0..self.images.len() {
            concat_inputs.push(format!("[img{}]", i));
        }
        for i in 0..self.videos.len() {
            concat_inputs.push(format!("[vid{}]", i));
        }

        let concat_filter = format!("{}concat=n={}:v=1:a=0[outv]", concat_inputs.join(""), input_count);
        filter_parts.push(concat_filter);

        let filter_complex = filter_parts.join(";");

        // Build FFmpeg command
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-y"); // Overwrite output file

        // Add input files
        for image_path in &self.images {
            cmd.arg("-loop").arg("1")
               .arg("-t").arg(self.options.image_duration.to_string())
               .arg("-i").arg(image_path);
        }
        
        for video_path in &self.videos {
            cmd.arg("-i").arg(video_path);
        }

        // Add filter and output
        cmd.arg("-filter_complex").arg(&filter_complex)
           .arg("-map").arg("[outv]")
           .arg("-c:v").arg(&self.options.codec)
           .arg("-pix_fmt").arg(&self.options.pixel_format)
           .arg("-r").arg(self.options.fps.to_string())
           .arg(output_path.as_ref());

        debug!("Running FFmpeg with Command: {:?}", cmd);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("FFmpeg failed: {}", stderr);
            anyhow::bail!("FFmpeg failed: {}", stderr);
        }

        debug!("Slideshow generated successfully: {}", output_path.as_ref().display());
        Ok(())
    }
}
