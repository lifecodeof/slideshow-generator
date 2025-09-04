//! # Slideshow Generator
//! 
//! A Rust library for generating slideshow videos from images and videos.
//! 
//! ## Features
//! 
//! - Support for multiple image formats (PNG, JPG, JPEG, GIF, BMP, TIFF)
//! - Support for multiple video formats (MP4, MOV, AVI, MKV, WEBM)
//! - Automatic scaling to 1920x1080 with aspect ratio preservation
//! - Configurable image display duration
//! - Command-line interface for easy usage
//! 
//! ## Example
//! 
//! ```rust,no_run
//! use slideshow_generator::{SlideshowGenerator, SlideshowOptions};
//! use std::path::PathBuf;
//! 
//! # fn main() -> anyhow::Result<()> {
//! let options = SlideshowOptions::new()
//!     .with_duration_per_slide(3.0)
//!     .with_output_resolution(1920, 1080);
//! 
//! let generator = SlideshowGenerator::from_directory("input_folder", options)?;
//! generator.generate("output.mp4")?;
//! # Ok(())
//! # }
//! ```

pub mod media;
pub mod slideshow;
pub mod utils;
pub mod transitions;

// Re-export the main types for convenience
pub use slideshow::{SlideshowGenerator, SlideshowOptions};
pub use media::{Image, Video};
pub use transitions::{SlideshowTransition, BuiltinTransition, SlideDirection, WipeDirection};

use std::path::Path;
use anyhow::Result;

/// Convenience function to generate a slideshow from a directory
/// 
/// This is a high-level function that combines directory loading and slideshow generation.
/// 
/// # Arguments
/// 
/// * `input_dir` - Path to directory containing images and videos
/// * `output_path` - Path for the output video file
/// * `options` - Optional slideshow configuration
/// 
/// # Example
/// 
/// ```rust,no_run
/// use slideshow_generator::{generate_slideshow, SlideshowOptions};
/// 
/// # fn main() -> anyhow::Result<()> {
/// let options = SlideshowOptions::default();
/// generate_slideshow("input_images", "slideshow.mp4", Some(options))?;
/// # Ok(())
/// # }
/// ```
pub fn generate_slideshow<P1, P2>(
    input_dir: P1,
    output_path: P2,
    options: Option<SlideshowOptions>,
) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let options = options.unwrap_or_default();
    let generator = SlideshowGenerator::from_directory(input_dir, options)?;
    generator.generate(output_path)
}

/// Quick slideshow generation with default settings
/// 
/// # Arguments
/// 
/// * `input_dir` - Path to directory containing images and videos
/// * `output_path` - Path for the output video file
/// 
/// # Example
/// 
/// ```rust,no_run
/// use slideshow_generator::quick_slideshow;
/// 
/// # fn main() -> anyhow::Result<()> {
/// quick_slideshow("input_images", "slideshow.mp4")?;
/// # Ok(())
/// # }
/// ```
pub fn quick_slideshow<P1, P2>(input_dir: P1, output_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    generate_slideshow(input_dir, output_path, None)
}
