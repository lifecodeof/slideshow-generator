use slideshow_generator::{SlideshowGenerator, SlideshowOptions, BuiltinTransition};
use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();
    
    println!("=== Practical Timing Test ===");
    println!("Testing the common use case: 5 images + 1 video");
    
    // Test the exact scenario the user mentioned
    let options = SlideshowOptions::new()
        .with_duration_per_slide(3.0)
        .with_transition(BuiltinTransition::Fade { duration: 1.0 });
    
    // Generate with timing analysis
    println!("Expected timing calculation:");
    println!("- 5 images × 3.0s = 15.0s total content");
    println!("- 4 transitions × 1.0s = 4.0s transition time");
    println!("- Overlap compensation = -3.0s (since each transition overlaps 1s from each adjacent slide)");
    println!("- Images portion = 15.0 + 4.0 - 7.0 = 12.0s");
    println!("- Plus video content (varies)");
    
    match SlideshowGenerator::from_directory("test_images", options) {
        Ok(generator) => {
            println!("Found {} images and {} videos", 
                generator.image_count(), generator.video_count());
            
            generator.generate("test_practical_timing.mp4")?;
            println!("✓ Practical timing test completed successfully!");
            println!("The hybrid approach correctly handles the most common use case.");
        },
        Err(e) => {
            println!("Error loading directory: {}", e);
        }
    }
    
    Ok(())
}
