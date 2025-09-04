// Transition Demo - Shows all available transition types
use slideshow_generator::{SlideshowGenerator, SlideshowOptions, BuiltinTransition, SlideDirection, WipeDirection};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("🎬 Slideshow Generator - Transition Demo");
    println!("=========================================");

    // Demo 1: No transition (simple concatenation)
    println!("\n1. No Transition (Simple concatenation)");
    let options = SlideshowOptions::new()
        .with_duration_per_slide(2.0)
        .with_output_resolution(1280, 720)
        .with_transition(BuiltinTransition::None);

    let generator = SlideshowGenerator::from_directory("test_images", options)?;
    generator.generate("demo_none_lib.mp4")?;
    println!("   ✓ Generated: demo_none_lib.mp4");

    // Demo 2: Fade transitions
    println!("\n2. Fade Transition");
    let options = SlideshowOptions::new()
        .with_duration_per_slide(3.0)
        .with_transition(BuiltinTransition::fade(1.2));

    let generator = SlideshowGenerator::from_directory("test_images", options)?;
    generator.generate("demo_fade_lib.mp4")?;
    println!("   ✓ Generated: demo_fade_lib.mp4");

    // Demo 3: Dissolve transitions
    println!("\n3. Dissolve Transition");
    let options = SlideshowOptions::new()
        .with_duration_per_slide(2.5)
        .with_transition(BuiltinTransition::dissolve(1.0));

    let generator = SlideshowGenerator::from_directory("test_images", options)?;
    generator.generate("demo_dissolve_lib.mp4")?;
    println!("   ✓ Generated: demo_dissolve_lib.mp4");

    // Demo 4: Slide transitions
    println!("\n4. Slide Transitions");
    
    // Slide Left
    let options = SlideshowOptions::new()
        .with_duration_per_slide(2.0)
        .with_transition(BuiltinTransition::slide(SlideDirection::Left, 0.8));
    
    let generator = SlideshowGenerator::from_directory("test_images", options)?;
    generator.generate("demo_slide_left_lib.mp4")?;
    println!("   ✓ Generated: demo_slide_left_lib.mp4");

    // Slide Right  
    let options = SlideshowOptions::new()
        .with_duration_per_slide(2.0)
        .with_transition(BuiltinTransition::slide(SlideDirection::Right, 0.8));
    
    let generator = SlideshowGenerator::from_directory("test_images", options)?;
    generator.generate("demo_slide_right_lib.mp4")?;
    println!("   ✓ Generated: demo_slide_right_lib.mp4");

    // Demo 5: Wipe transitions
    println!("\n5. Wipe Transitions");
    
    // Wipe Down
    let options = SlideshowOptions::new()
        .with_duration_per_slide(2.5)
        .with_transition(BuiltinTransition::wipe(WipeDirection::Down, 1.0));
    
    let generator = SlideshowGenerator::from_directory("test_images", options)?;
    generator.generate("demo_wipe_down_lib.mp4")?;
    println!("   ✓ Generated: demo_wipe_down_lib.mp4");

    // Custom transition example
    println!("\n6. Custom Fast Transitions");
    let options = SlideshowOptions::new()
        .with_duration_per_slide(1.5)
        .with_output_resolution(1920, 1080)
        .with_transition(BuiltinTransition::slide(SlideDirection::Up, 0.5));

    let generator = SlideshowGenerator::from_directory("test_images", options)?;
    generator.generate("demo_fast_slide_lib.mp4")?;
    println!("   ✓ Generated: demo_fast_slide_lib.mp4");

    println!("\n🎉 All transition demos completed!");
    println!("Check the generated MP4 files to see the different transition effects.");
    
    Ok(())
}
