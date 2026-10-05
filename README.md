# Slideshow Generator

A Rust library/cli/gui and CLI tool for generating slideshow videos from images
and videos using FFmpeg.

## Features

- 🖼️ Support for multiple image formats (PNG, JPG, JPEG, GIF, BMP, TIFF)
- 🎥 Support for multiple video formats (MP4, MOV, AVI, MKV, WEBM)
- 📐 Automatic scaling with aspect ratio preservation
- ⚙️ Configurable image duration, resolution, frame rate, and codec
- 📚 Both library and CLI interfaces
- 🔄 Mixed content support (images + videos)
- ❄️ Optional Christmas snowfall overlay effect

## Transition Showcase

It is possible to add new transitions via library API.

See builtin transitions in action:

https://github.com/user-attachments/assets/7e2550d8-d3f5-433f-8ce1-51c736d8b443

## Prerequisites

- FFmpeg must be installed and available in your PATH
- Rust 1.70+ for building from source

## Installation

### As a CLI tool

```bash
cargo install slideshow-generator
```

### As a library dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
slideshow-generator = "0.2.0"
```

## Usage

### Command Line Interface

Show help:

```bash
slideshow-generator --help
```

```
A CLI tool to generate slideshow videos from images and videos

Usage: slideshow-generator.exe [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>
          Input directory containing images and videos
  -o, --output <OUTPUT>
          Output video file path [default: output.mp4]
  -d, --duration-per-slide <DURATION_PER_SLIDE>
          Duration in seconds for each slide [default: 3.0]
  -W, --width <WIDTH>
          Output video width
  -H, --height <HEIGHT>
          Output video height
      --resolution-coefficient <RESOLUTION_COEFFICIENT>
          Resolution coefficient for auto-detected dimensions (0.0-1.0)
  -t, --transition <TRANSITION>

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
               [default: none]
  -s, --snow
          Add the Christmas snowfall effect over the slideshow

          Every render draws its own randomly derived snowfall, so consecutive
          videos look different without any further configuration.
  -v, --verbose
          Enable verbose logging
  -h, --help
          Print help
```

### Christmas snowfall

Add `--snow` to enable the effect:

```bash
slideshow-generator -i photos -o christmas.mp4 --snow
```

That is the whole interface. Each render draws a fresh seed, and everything
visible derives from it: how heavy the snow is, how fast it falls, which way the
wind blows, the flake size, and whether any shooting stars streak across the
sky. Six videos rendered with the same command therefore come out looking
genuinely different, with no configuration at all.

The resolved seed is printed on startup, so a look that turns out well can be
reproduced through the library API:

```rust
use slideshow_generator::{SlideshowOptions, SnowfallOptions};

let snowfall = SnowfallOptions::randomized(1791189322608723675);
let options = SlideshowOptions::new().with_snowfall(Some(snowfall));
```

The snow is generated procedurally, no external assets are needed. Four layers
are composited for depth, each with its own speed, drift direction and sway, so
the flakes move independently rather than sliding across as one block. The
pattern repeats seamlessly over any duration.

Shooting stars are a garnish: a video may get none, or up to three, spread
across it and never overlapping.

### Library API

#### Quick Start

```rust
use slideshow_generator::quick_slideshow;

fn main() -> anyhow::Result<()> {
    // Generate a slideshow with default settings
    quick_slideshow("input_folder", "output.mp4")?;
    Ok(())
}
```

#### Custom Configuration

```rust
use slideshow_generator::{SlideshowGenerator, SlideshowOptions};

fn main() -> anyhow::Result<()> {
    // Create custom options
    let options = SlideshowOptions::new()
        .with_image_duration(5.0)
        .with_output_resolution(1280, 720)
        .with_fps(24)
        .with_codec("libx265");

    // Generate slideshow with custom options
    let generator = SlideshowGenerator::from_directory("input_folder", options)?;
    generator.generate("output.mp4")?;

    Ok(())
}
```

#### Christmas Snowfall

```rust
use slideshow_generator::{SlideshowGenerator, SlideshowOptions, SnowfallOptions};

fn main() -> anyhow::Result<()> {
    let snowfall = SnowfallOptions::new()
        .with_density(400.0)   // snowflakes per megapixel, per layer
        .with_speed(1.2)       // fall speed multiplier
        .with_wind(-0.6)       // negative blows the snow to the left
        .with_flake_size(10.0) // diameter in pixels at 1080p
        .with_opacity(0.9);

    let options = SlideshowOptions::new()
        .with_output_resolution(1920, 1080)
        .with_snowfall(Some(snowfall));

    let generator = SlideshowGenerator::from_directory("christmas_photos", options)?;
    generator.generate("christmas.mp4")?;

    Ok(())
}
```

Use `with_snowfall_enabled()` for a fresh random snowfall per render and
`with_snowfall(None)` to turn the effect back off. To pin a specific look, pass
`SnowfallOptions::randomized(seed)` as above. The snow is generated
procedurally, so no assets are required and the animation loops seamlessly for
any duration.

#### Manual File Management

```rust
use slideshow_generator::{SlideshowGenerator, SlideshowOptions};

fn main() -> anyhow::Result<()> {
    let mut generator = SlideshowGenerator::new();

    // Add files manually
    generator.add_image("image1.jpg");
    generator.add_image("image2.png");
    generator.add_video("video1.mp4");

    // Generate slideshow
    generator.generate("output.mp4")?;

    Ok(())
}
```

## API Reference

### `SlideshowOptions`

Configuration struct for slideshow generation with builder pattern methods.

### `SnowfallOptions`

Christmas snowfall configuration, built with `with_density`, `with_speed`,
`with_wind`, `with_flake_size`, `with_opacity` and `with_seed`. Attach it to a
slideshow with `SlideshowOptions::with_snowfall`.

### `SlideshowGenerator`

Main generator struct with methods for loading media files and generating
slideshows.

### Convenience Functions

- `quick_slideshow(input_dir, output_path)` - Generate with defaults
- `generate_slideshow(input_dir, output_path, options)` - Generate with custom
  options

## Examples

See the `examples/` directory for comprehensive usage examples.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
