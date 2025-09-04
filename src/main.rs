use clap::Parser;
use log::{error, info};
use slideshow_generator::{SlideshowGenerator, SlideshowOptions};
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

    /// Duration in seconds for each image
    #[arg(short = 'd', long, default_value = "3.0")]
    image_duration: f32,

    /// Output video width
    #[arg(short = 'W', long, default_value = "1920")]
    width: u32,

    /// Output video height
    #[arg(short = 'H', long, default_value = "1080")]
    height: u32,

    /// Output video frame rate
    #[arg(short, long, default_value = "30")]
    fps: u32,

    /// Video codec to use
    #[arg(short, long, default_value = "libx264")]
    codec: String,

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

    // Create slideshow options from CLI arguments
    let options = SlideshowOptions::new()
        .with_image_duration(cli.image_duration)
        .with_output_resolution(cli.width, cli.height)
        .with_fps(cli.fps)
        .with_codec(&cli.codec);

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
