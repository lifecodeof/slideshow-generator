# Slideshow Generator

A Rust library and CLI tool for generating slideshow videos from images and videos using FFmpeg.

## Features

- 🖼️ Support for multiple image formats (PNG, JPG, JPEG, GIF, BMP, TIFF)
- 🎥 Support for multiple video formats (MP4, MOV, AVI, MKV, WEBM)
- 📐 Automatic scaling with aspect ratio preservation
- ⚙️ Configurable image duration, resolution, frame rate, and codec
- 📚 Both library and CLI interfaces
- 🔄 Mixed content support (images + videos)

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
slideshow-generator = "0.1.0"
```

## Usage

### Command Line Interface

Basic usage:
```bash
slideshow-generator --input images_folder --output slideshow.mp4
```

With custom options:
```bash
slideshow-generator 
  --input images_folder 
  --output slideshow.mp4 
  --image-duration 5.0 
  --width 1280 
  --height 720 
  --fps 24 
  --codec libx265
```

CLI Options:
- `-i, --input <INPUT>`: Input directory containing images and videos
- `-o, --output <OUTPUT>`: Output video file path (default: output.mp4)
- `-d, --image-duration <DURATION>`: Duration in seconds for each image (default: 3.0)
- `-W, --width <WIDTH>`: Output video width (default: 1920)
- `-H, --height <HEIGHT>`: Output video height (default: 1080)
- `-f, --fps <FPS>`: Output video frame rate (default: 30)
- `-c, --codec <CODEC>`: Video codec to use (default: libx264)

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

### `SlideshowGenerator`

Main generator struct with methods for loading media files and generating slideshows.

### Convenience Functions

- `quick_slideshow(input_dir, output_path)` - Generate with defaults
- `generate_slideshow(input_dir, output_path, options)` - Generate with custom options

## Examples

See the `examples/` directory for comprehensive usage examples.

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
