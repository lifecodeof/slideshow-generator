use crate::transitions::BuiltinTransition;
use crate::utils::{filter_media_files, read_files_from_directory};
use anyhow::{bail, Result};
use log::{debug, error};
use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Configuration options for slideshow generation
#[derive(Debug, Clone)]
pub struct SlideshowOptions {
    pub duration_per_slide: f32,
    pub output_dimensions: Option<(u32, u32)>,
    pub output_path: PathBuf,
    pub transition: BuiltinTransition,
}

impl Default for SlideshowOptions {
    fn default() -> Self {
        Self {
            duration_per_slide: 3.0,
            output_dimensions: None,
            output_path: PathBuf::from("slideshow.mp4"),
            transition: BuiltinTransition::None,
        }
    }
}

impl SlideshowOptions {
    /// Create a new SlideshowOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the duration for each slide in seconds
    pub fn with_duration_per_slide(mut self, duration: f32) -> Self {
        self.duration_per_slide = duration;
        self
    }

    /// Set the output resolution
    pub fn with_output_resolution(mut self, width: u32, height: u32) -> Self {
        self.output_dimensions = Some((width, height));
        self
    }

    /// Set the output dimensions (alias for with_output_resolution)
    pub fn with_output_dimensions(mut self, dimensions: Option<(u32, u32)>) -> Self {
        self.output_dimensions = dimensions;
        self
    }

    /// Get the output width (for backward compatibility)
    pub fn output_width(&self) -> u32 {
        self.output_dimensions.unwrap_or((1920, 1080)).0
    }

    /// Get the output height (for backward compatibility)
    pub fn output_height(&self) -> u32 {
        self.output_dimensions.unwrap_or((1920, 1080)).1
    }

    /// Set the output path
    pub fn with_output_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.output_path = path.into();
        self
    }

    /// Set the transition between slides
    pub fn with_transition(mut self, transition: BuiltinTransition) -> Self {
        self.transition = transition;
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
            // Skip the output file if it's in the same directory
            if {
                // Try to compare canonical paths first
                if let (Ok(file_canonical), Ok(output_canonical)) =
                    (file.canonicalize(), self.options.output_path.canonicalize())
                {
                    file_canonical == output_canonical
                } else {
                    // Fall back to filename comparison if canonicalization fails
                    file.file_name() == self.options.output_path.file_name()
                        && file.file_name().is_some()
                }
            } {
                continue;
            }

            if let Some(extension) = file.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                match ext.as_str() {
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" => {
                        self.images.push(file);
                    }
                    "mp4" | "mov" | "avi" | "mkv" | "webm" => {
                        self.videos.push(file);
                    }
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

    /// Get the output dimensions, using the first image's dimensions if not specified
    pub fn get_output_dimensions(&self) -> Result<(u32, u32)> {
        if let Some(dimensions) = self.options.output_dimensions {
            Ok(dimensions)
        } else {
            // Get dimensions from the first image
            if let Some(first_image) = self.images.first() {
                let image = crate::media::Image::new(first_image.clone());
                image.dimensions()
            } else if let Some(_first_video) = self.videos.first() {
                bail!("Output dimensions must be specified when only videos are present");
            } else {
                bail!("No media files available to determine output dimensions");
            }
        }
    }
    pub fn set_options(&mut self, options: SlideshowOptions) {
        self.options = options;
    }

    /// Get the current options
    pub fn options(&self) -> &SlideshowOptions {
        &self.options
    }

    /// Generate transition filters for multiple inputs
    fn generate_transition_filters(&self, input_labels: &[String]) -> Result<String> {
        use crate::transitions::SlideshowTransition;

        match &self.options.transition {
            BuiltinTransition::None => {
                // Simple concatenation without transitions
                Ok(format!(
                    "{}concat=n={}:v=1:a=0[outv]",
                    input_labels.join(""),
                    input_labels.len()
                ))
            }
            transition => {
                // Apply transitions between consecutive slides
                if input_labels.len() < 2 {
                    // Single input - just pass through
                    let (width, height) = self.get_output_dimensions()?;
                    Ok(format!(
                        "{}scale={}:{}[outv]",
                        input_labels[0], width, height
                    ))
                } else if input_labels.len() == 2 {
                    // Two inputs - simple xfade (proven to work)
                    let transition_duration = transition.duration();
                    let offset = (self.options.duration_per_slide - transition_duration).max(0.0);
                    Ok(transition.to_ffmpeg_filter(
                        &input_labels[0],
                        &input_labels[1],
                        "[outv]",
                        offset,
                    ))
                } else {
                    // Multiple inputs: Use practical approach for common case (5 images + 1 video)
                    self.generate_practical_multi_transitions(input_labels, transition)
                }
            }
        }
    }

    /// Generate transitions for the common case: multiple images + optional video  
    /// Strategy: Build timeline segments with transitions to achieve exact mathematical timing
    fn generate_practical_multi_transitions(
        &self,
        input_labels: &[String],
        transition: &BuiltinTransition,
    ) -> Result<String> {
        use crate::transitions::SlideshowTransition;

        // Separate images from videos based on our processing order
        let num_images = self.images.len();
        let image_labels = &input_labels[..num_images];
        let video_labels = &input_labels[num_images..];

        if image_labels.len() < 2 {
            // No transitions possible
            return Ok(format!(
                "{}concat=n={}:v=1:a=0[outv]",
                input_labels.join(""),
                input_labels.len()
            ));
        }

        let transition_duration = transition.duration();

        // Use chained xfade approach - this creates smooth transitions without duplication
        // Each xfade overlaps the end of one clip with the beginning of the next

        let mut current_label = image_labels[0].clone();
        let mut filter_parts = Vec::new();

        // Apply transitions between consecutive images
        for i in 1..image_labels.len() {
            let next_label = image_labels[i].clone();
            let result_label = if i == image_labels.len() - 1 {
                "[images_result]".to_string()
            } else {
                format!("[temp{}]", i)
            };

            // For smooth transitions without duplication:
            // offset should be: (total_duration_so_far - transition_duration)
            // This makes the transition start near the end of the accumulated timeline
            let offset = if i == 1 {
                // First transition: start transition near end of first image
                self.options.duration_per_slide - transition_duration
            } else {
                // Subsequent transitions: account for previous transitions
                // Each previous transition reduces total duration by transition_duration
                let accumulated_duration = (i as f32) * self.options.duration_per_slide
                    - ((i - 1) as f32) * transition_duration;
                accumulated_duration - transition_duration
            };

            let transition_filter = transition.to_ffmpeg_filter(
                &current_label,
                &next_label,
                &result_label,
                offset.max(0.0),
            );
            filter_parts.push(transition_filter);
            current_label = result_label;
        }

        // Handle video concatenation
        if video_labels.is_empty() {
            // No videos - rename final result
            if image_labels.len() == 2 {
                // For 2 images, the result is already correctly labeled
                if let Some(last_filter) = filter_parts.last_mut() {
                    *last_filter = last_filter.replace("[images_result]", "[outv]");
                }
            } else {
                // For multiple images, rename the final result
                filter_parts.push("[images_result]null[outv]".to_string());
            }
        } else {
            // Concatenate with videos
            let mut final_inputs = vec!["[images_result]".to_string()];
            final_inputs.extend(video_labels.iter().cloned());

            filter_parts.push(format!(
                "{}concat=n={}:v=1:a=0[outv]",
                final_inputs.join(""),
                final_inputs.len()
            ));
        }

        Ok(filter_parts.join(";"))
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
        debug!(
            "Generating slideshow with {} images and {} videos",
            self.images.len(),
            self.videos.len()
        );

        if self.images.is_empty() && self.videos.is_empty() {
            anyhow::bail!("No media files found to create slideshow");
        }

        // Get output dimensions
        let (output_width, output_height) = self.get_output_dimensions()?;
        debug!(
            "Using output dimensions: {}x{}",
            output_width, output_height
        );

        // Check if FFmpeg is available
        let mut ffmpeg_check_cmd = std::process::Command::new("ffmpeg");
        ffmpeg_check_cmd.arg("-version");

        #[cfg(windows)]
        ffmpeg_check_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        match ffmpeg_check_cmd.output() {
            Ok(output) if output.status.success() => {
                debug!("FFmpeg found, proceeding with video generation...");
            }
            _ => {
                anyhow::bail!("FFmpeg not found. Please install FFmpeg and add it to your PATH.");
            }
        }

        // Create filter complex with transitions
        let mut filter_parts = Vec::new();

        // Add images (convert to video segments)
        for (i, _image_path) in self.images.iter().enumerate() {
            filter_parts.push(format!(
                "[{}:v]scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setpts=PTS-STARTPTS,fps=30[img{}]", 
                i, output_width, output_height, output_width, output_height, i
            ));
        }

        // Add videos (scale to same resolution and normalize frame rate)
        for (i, _video_path) in self.videos.iter().enumerate() {
            let input_idx = self.images.len() + i;
            filter_parts.push(format!(
                "[{}:v]scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,fps=30,setpts=PTS-STARTPTS[vid{}]", 
                input_idx, output_width, output_height, output_width, output_height, i
            ));
        }

        // Collect all input labels
        let mut input_labels = Vec::new();
        for i in 0..self.images.len() {
            input_labels.push(format!("[img{}]", i));
        }
        for i in 0..self.videos.len() {
            input_labels.push(format!("[vid{}]", i));
        }

        // Apply transitions between consecutive inputs
        let filter_result = if input_labels.len() <= 1 {
            // Single input or no inputs - just pass through
            let default_input = "[0:v]".to_string();
            let input = input_labels.first().unwrap_or(&default_input);
            format!("{}scale={}:{}[outv]", input, output_width, output_height)
        } else {
            // Multiple inputs - apply transitions or concatenation
            self.generate_transition_filters(&input_labels)?
        };

        filter_parts.push(filter_result);
        let filter_complex = filter_parts.join(";");
        debug!("Generated filter_complex: {}", filter_complex);

        // Build FFmpeg command
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-y"); // Overwrite output file

        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        // Add input files
        for image_path in &self.images {
            cmd.arg("-loop")
                .arg("1")
                .arg("-t")
                .arg(self.options.duration_per_slide.to_string())
                .arg("-i")
                .arg(image_path);
        }

        for video_path in &self.videos {
            cmd.arg("-i").arg(video_path);
        }

        // Add filter and output
        cmd.arg("-filter_complex")
            .arg(&filter_complex)
            .arg("-map")
            .arg("[outv]")
            .arg("-c:v")
            .arg("libx264")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-r")
            .arg("30")
            .arg(output_path.as_ref());

        debug!("Running FFmpeg with Command: {:?}", cmd);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("FFmpeg failed: {}", stderr);
            anyhow::bail!("FFmpeg failed: {}", stderr);
        }

        debug!(
            "Slideshow generated successfully: {}",
            output_path.as_ref().display()
        );
        Ok(())
    }
}
