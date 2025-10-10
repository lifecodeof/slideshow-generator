use clap::Parser;
use log::{error, info};
use slideshow_generator::{SlideshowGenerator, SlideshowOptions, BuiltinTransition};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "slideshow-generator")]
#[command(about = "A CLI tool to generate slideshow videos from images and videos")]
struct Cli {
    /// Input directory containing images and videos
    #[arg(short, long)]
    input: PathBuf,

    /// Output video file path
    #[arg(short, long, default_value = "output.mp4")]
    output: PathBuf,

    /// Duration in seconds for each slide
    #[arg(short = 'd', long, default_value = "3.0")]
    duration_per_slide: f32,

    /// Output video width
    #[arg(short = 'W', long, default_value = "1920")]
    width: u32,

    /// Output video height
    #[arg(short = 'H', long, default_value = "1080")]
    height: u32,

    /// Use auto-detected resolution from first image
    #[arg(long)]
    auto_resolution: bool,

    /// Resolution coefficient for auto-detected dimensions (0.0-1.0)
    #[arg(long, default_value = "1.0")]
    resolution_coefficient: f32,

    /// Transition type between slides
    #[arg(short = 't', long, default_value = "none")]
    transition: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logger based on verbosity
    let log_level = if cli.verbose { "debug" } else { "info" };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    if !cli.input.exists() {
        error!("Input directory does not exist: {}", cli.input.display());
        anyhow::bail!("Input directory does not exist: {}", cli.input.display());
    }

    if !cli.input.is_dir() {
        error!("Input path is not a directory: {}", cli.input.display());
        anyhow::bail!("Input path is not a directory: {}", cli.input.display());
    }

    info!("Loading media files from: {}", cli.input.display());

    // Parse transition from string
    let transition = cli.transition.parse::<BuiltinTransition>()
        .map_err(|e| anyhow::anyhow!("Invalid transition '{}': {}", cli.transition, e))?;

    // Create slideshow options from CLI arguments
    let mut options = SlideshowOptions::new()
        .with_duration_per_slide(cli.duration_per_slide)
        .with_resolution_coefficient(cli.resolution_coefficient)
        .with_output_path(&cli.output)
        .with_transition(transition);

    if !cli.auto_resolution {
        options = options.with_output_resolution(cli.width, cli.height);
    }

    // Create generator with custom options
    let generator = SlideshowGenerator::from_directory(&cli.input, options)?;

    info!(
        "Found {} images and {} videos",
        generator.image_count(),
        generator.video_count()
    );
    info!("Generating slideshow to: {}", cli.output.display());

    // Generate the slideshow using the modern API
    generator.generate(&cli.output)?;

    Ok(())
}
