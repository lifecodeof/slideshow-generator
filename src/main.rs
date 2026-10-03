use clap::Parser;
use log::{error, info};
use slideshow_generator::{
    BuiltinTransition, SlideshowGenerator, SlideshowOptions, SnowfallOptions,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "slideshow-generator")]
#[command(about = "A CLI tool to generate slideshow videos from images and videos")]
#[command(allow_negative_numbers = true)]
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
    #[arg(
        short = 'W',
        long,
        requires = "height",
        conflicts_with = "resolution_coefficient"
    )]
    width: Option<u32>,

    /// Output video height
    #[arg(
        short = 'H',
        long,
        requires = "width",
        conflicts_with = "resolution_coefficient"
    )]
    height: Option<u32>,

    /// Resolution coefficient for auto-detected dimensions (0.0-1.0)
    #[arg(long, conflicts_with_all = ["width", "height"])]
    resolution_coefficient: Option<f32>,

    #[arg(
        short = 't',
        long,
        default_value = "none",
        help = r#"
Transition type between slides

Examples:
  --transition fade
  --transition fade:2.5
  --transition slide-left:1.2
  --transition wipe-diagonal-tl

Available transitions:
  none, fade, dissolve,
  slide-left, slide-right, slide-up, slide-down,
  wipe-left, wipe-right, wipe-up, wipe-down,
  wipe-diagonal-tl, wipe-diagonal-tr
    "#
    )]
    transition: String,

    /// Add the Christmas snowfall effect over the slideshow
    ///
    /// Every snowfall option below turns the effect on as well, so passing
    /// `--snow-density 150` is enough without `--snow`.
    #[arg(short = 's', long)]
    snow: bool,

    /// Snowflakes per megapixel, per layer [default: 200]
    #[arg(long)]
    snow_density: Option<f32>,

    /// Snow fall speed multiplier [default: 1.0]
    #[arg(long)]
    snow_speed: Option<f32>,

    /// Horizontal wind, negative blows the snow to the left [default: 0.3]
    #[arg(long)]
    snow_wind: Option<f32>,

    /// Snowflake diameter in pixels for 1080p [default: 8.0]
    #[arg(long)]
    snow_size: Option<f32>,

    /// Snow opacity between 0.0 and 1.0 [default: 0.85]
    #[arg(long)]
    snow_opacity: Option<f32>,

    /// Seed of the snowflake pattern, for reproducible results
    #[arg(long)]
    snow_seed: Option<u64>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Quiet mode, suppress non-error output
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logger based on verbosity
    let log_level = if cli.quiet {
        "error"
    } else if cli.verbose {
        "debug"
    } else {
        "info"
    };

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
    let transition = cli
        .transition
        .parse::<BuiltinTransition>()
        .map_err(|e| anyhow::anyhow!("Invalid transition '{}': {}", cli.transition, e))?;

    // Create slideshow options from CLI arguments
    let mut options = SlideshowOptions::new()
        .with_duration_per_slide(cli.duration_per_slide)
        .with_output_path(&cli.output)
        .with_transition(transition)
        .with_snowfall(snowfall_options(&cli)?);

    // Handle resolution settings - mutually exclusive
    match (cli.width, cli.height, cli.resolution_coefficient) {
        (Some(width), Some(height), None) => {
            // Custom dimensions provided
            options = options.with_output_resolution(width, height);
        }
        (None, None, Some(coefficient)) => {
            // Coefficient provided - use auto-detection with coefficient
            options = options.with_resolution_coefficient(coefficient);
        }
        (None, None, None) => {
            // Neither provided - use defaults (1920x1080)
            options = options.with_output_resolution(1920, 1080);
        }
        _ => {
            // Invalid combination
            anyhow::bail!("Specify either custom dimensions (--width and --height) OR resolution coefficient (--resolution-coefficient), not both");
        }
    }

    // Create generator with custom options
    let generator = SlideshowGenerator::from_directory(&cli.input, options)?;

    info!("Found {} images and {} videos", generator.image_count(), generator.video_count());

    if generator.options().snowfall.is_some() {
        info!("Christmas snowfall enabled");
    }

    info!("Generating slideshow to: {}", cli.output.display());

    // Generate the slideshow using the modern API
    generator.generate(&cli.output)?;

    Ok(())
}

/// Build the snowfall configuration from the CLI arguments
///
/// The effect is enabled by `--snow` or by passing any of the tuning options,
/// so `--snow-density 150` works on its own. Returns `None` when disabled.
fn snowfall_options(cli: &Cli) -> anyhow::Result<Option<SnowfallOptions>> {
    let tuned = cli.snow_density.is_some()
        || cli.snow_speed.is_some()
        || cli.snow_wind.is_some()
        || cli.snow_size.is_some()
        || cli.snow_opacity.is_some()
        || cli.snow_seed.is_some();

    if !cli.snow && !tuned {
        return Ok(None);
    }

    let mut snowfall = SnowfallOptions::new();

    if let Some(density) = cli.snow_density {
        snowfall = snowfall.with_density(density);
    }
    if let Some(speed) = cli.snow_speed {
        snowfall = snowfall.with_speed(speed);
    }
    if let Some(wind) = cli.snow_wind {
        snowfall = snowfall.with_wind(wind);
    }
    if let Some(size) = cli.snow_size {
        snowfall = snowfall.with_flake_size(size);
    }
    if let Some(opacity) = cli.snow_opacity {
        snowfall = snowfall.with_opacity(opacity);
    }
    if let Some(seed) = cli.snow_seed {
        snowfall = snowfall.with_seed(seed);
    }

    snowfall.validate()?;

    Ok(Some(snowfall))
}
