use slideshow_generator::{SlideshowGenerator, SlideshowOptions, quick_slideshow};
use anyhow::Result;
use std::path::PathBuf;
use log::{info, warn};

fn main() -> Result<()> {
    // Initialize simple logger for the example
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("=== Slideshow Generator Library Usage Examples ===");

    // Example 1: Quick slideshow with default settings
    info!("1. Quick slideshow generation:");
    if let Err(e) = quick_slideshow("test_images", "example_quick.mp4") {
        warn!("   Error: {} (This is expected if test_images doesn't exist)", e);
    } else {
        info!("   ✓ Quick slideshow generated successfully!");
    }

    // Example 2: Custom slideshow with options
    info!("2. Custom slideshow with options:");
    let options = SlideshowOptions::new()
        .with_image_duration(5.0)  // 5 seconds per image
        .with_output_resolution(1280, 720)  // 720p output
        .with_fps(24)  // 24 fps
        .with_codec("libx265");  // H.265 codec

    match SlideshowGenerator::from_directory("test_images", options) {
        Ok(generator) => {
            info!("   Found {} images and {} videos", 
                generator.image_count(), generator.video_count());
            
            if let Err(e) = generator.generate("example_custom.mp4") {
                warn!("   Error generating slideshow: {}", e);
            } else {
                info!("   ✓ Custom slideshow generated successfully!");
            }
        },
        Err(e) => {
            warn!("   Error loading directory: {} (This is expected if test_images doesn't exist)", e);
        }
    }

    // Example 3: Manual file addition
    info!("3. Manual file addition:");
    let mut generator = SlideshowGenerator::new();
    
    // Add files manually (these would need to exist)
    let sample_files = vec![
        "test_images/image1.png",
        "test_images/image2.png", 
        "test_images/slideshow.mp4"
    ];

    for file in &sample_files {
        let path = PathBuf::from(file);
        if path.exists() {
            if let Some(ext) = path.extension() {
                match ext.to_string_lossy().to_lowercase().as_str() {
                    "png" | "jpg" | "jpeg" => generator.add_image(&path),
                    "mp4" | "mov" | "avi" => generator.add_video(&path),
                    _ => {}
                }
            }
        }
    }

    info!("   Manually added {} images and {} videos", 
        generator.image_count(), generator.video_count());

    if generator.total_count() > 0 {
        if let Err(e) = generator.generate("example_manual.mp4") {
            warn!("   Error generating slideshow: {}", e);
        } else {
            info!("   ✓ Manual slideshow generated successfully!");
        }
    } else {
        info!("   No files found to add (expected if test files don't exist)");
    }

    // Example 4: Options modification
    info!("4. Runtime options modification:");
    let mut generator = SlideshowGenerator::new();
    
    // Start with default options
    info!("   Default options: {}s per image, {}x{} resolution", 
        generator.options().image_duration,
        generator.options().width, 
        generator.options().height);

    // Change options at runtime
    let new_options = SlideshowOptions::new()
        .with_image_duration(2.0)
        .with_output_resolution(854, 480);  // 480p
    
    generator.set_options(new_options);
    
    info!("   Updated options: {}s per image, {}x{} resolution", 
        generator.options().image_duration,
        generator.options().width, 
        generator.options().height);

    info!("=== Examples completed ===");
    info!("Note: Some operations may fail if test files don't exist, which is normal for this demo.");

    Ok(())
}
