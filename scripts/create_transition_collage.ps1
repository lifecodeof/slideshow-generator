# PowerShell Script to Generate Transition Collage for Slideshow Generator
# This script automates the creation of a 3x4 grid collage showcasing all available slideshow transitions
# Requires: FFmpeg in PATH, slideshow-generator.exe built, test_items directory with images
#
# Usage:
# 1. Change directory to the project root
# 2. Place test images in test_items/ directory
# 3. Run this script: .\create_transition_collage.ps1
# 4. The collage will be created at temp_transition_examples/transition_collage.mp4
#
# The script will skip existing files to allow for incremental runs.


# Configuration
$transitionExamplesDir = "temp_transition_examples"
$testItemsDir = "test_items"
$cliPath = "target\debug\slideshow-generator.exe"  # Adjust if using release build
$transitions = @("none", "fade", "dissolve", "slide-left", "slide-right", "slide-up", "slide-down", "wipe-left", "wipe-right", "wipe-up", "wipe-down", "wipe-diagonal-tl")

# Create transition_examples directory if it doesn't exist
if (!(Test-Path $transitionExamplesDir)) {
  New-Item -ItemType Directory -Path $transitionExamplesDir | Out-Null
}

# Step 1: Generate original transition videos using the Rust CLI
Write-Host "Step 1: Generating original transition videos..."
foreach ($transition in $transitions) {
  $outputFile = Join-Path $transitionExamplesDir "$transition.mp4"

  $duration = if ($transition -eq "none") { 2 } else { 3 }

  if (!(Test-Path $outputFile)) {
    Write-Host "Generating $transition transition..."
    cargo run -qF cli -- --quiet --input $testItemsDir --output $outputFile --transition $transition --resolution-coefficient 0.25 --duration-per-slide $duration
  }
  else {
    Write-Host "$transition.mp4 already exists, skipping..."
  }
}

# Step 2: Add text labels to each video
Write-Host "Step 2: Adding text labels to videos..."
foreach ($transition in $transitions) {
  $inputFile = Join-Path $transitionExamplesDir "$transition.mp4"
  $outputFile = Join-Path $transitionExamplesDir "labeled_$transition.mp4"
  if (!(Test-Path $outputFile)) {
    Write-Host "Adding label to $transition..."
    # Use FFmpeg to overlay text on the video
    & ffmpeg -loglevel error -i $inputFile -vf "drawtext=text='$transition':fontcolor=white:fontsize=24:box=1:boxcolor=black@0.5:boxborderw=5:x=(w-text_w)/2:y=h-text_h-10:enable='between(t,0,999999)'" -c:a copy $outputFile
  }
  else {
    Write-Host "labeled_$transition.mp4 already exists, skipping..."
  }
}

# Step 5: Create the 3x4 grid collage
Write-Host "Step 5: Creating 3x4 grid collage..."
$collageFile = Join-Path $transitionExamplesDir "transition_collage.mp4"
if (!(Test-Path $collageFile)) {
  Write-Host "Stacking videos into grid..."
  # Build the FFmpeg command with all 12 inputs
  $inputs = @()
  foreach ($transition in $transitions) {
    $inputs += "-i"
    $inputs += (Join-Path $transitionExamplesDir "labeled_$transition.mp4")
  }

  # Layout for 3x4 grid: positions for each video in the stack
  $layout = "0_0|w0_0|w0+w0_0|0_h0|w0_h0|w0+w0_h0|0_h0+h0|w0_h0+h0|w0+w0_h0+h0|0_h0+h0+h0|w0_h0+h0+h0|w0+w0_h0+h0+h0"

  & ffmpeg -loglevel error $inputs -filter_complex "xstack=inputs=12:layout=$layout" -c:v libx264 $collageFile
}
else {
  Write-Host "transition_collage.mp4 already exists, skipping..."
}
