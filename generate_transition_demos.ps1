#!/usr/bin/env pwsh
# Slideshow Generator - Transition Demos Script
# This script generates demo videos for all transition types

Write-Host "🎬 Slideshow Generator - Transition Demos" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Ensure we're in the correct directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# Create demos directory if it doesn't exist
if (-not (Test-Path "demos")) {
  Write-Host "📁 Creating demos directory..." -ForegroundColor Yellow
  New-Item -ItemType Directory -Name "demos" | Out-Null
}

# Define all transition types and their configurations
$transitions = @(
  @{ Name = "None"; Type = "none"; Output = "demo_none.mp4"; Description = "No transitions (simple cuts)" },
  @{ Name = "Fade"; Type = "fade:0.2"; Output = "demo_fade.mp4"; Description = "Smooth fade transitions" },
  @{ Name = "Dissolve"; Type = "dissolve:0.2"; Output = "demo_dissolve.mp4"; Description = "Cross-dissolve blending effect" },
  @{ Name = "Slide Left"; Type = "slide-left:0.2"; Output = "demo_slide_left.mp4"; Description = "Slides move from right to left" },
  @{ Name = "Slide Right"; Type = "slide-right:0.2"; Output = "demo_slide_right.mp4"; Description = "Slides move from left to right" },
  @{ Name = "Slide Up"; Type = "slide-up:0.2"; Output = "demo_slide_up.mp4"; Description = "Slides move from bottom to top" },
  @{ Name = "Slide Down"; Type = "slide-down:0.2"; Output = "demo_slide_down.mp4"; Description = "Slides move from top to bottom" },
  @{ Name = "Wipe Left"; Type = "wipe-left:0.2"; Output = "demo_wipe_left.mp4"; Description = "Reveals next slide by wiping from left" },
  @{ Name = "Wipe Right"; Type = "wipe-right:0.2"; Output = "demo_wipe_right.mp4"; Description = "Reveals next slide by wiping from right" }
)

$totalTransitions = $transitions.Count
$currentTransition = 0
$startTime = Get-Date

Write-Host "🎯 Generating $totalTransitions transition demo videos..." -ForegroundColor Green
Write-Host ""

foreach ($transition in $transitions) {
  $currentTransition++
  $progress = [math]::Round(($currentTransition / $totalTransitions) * 100, 1)
    
  Write-Host "[$currentTransition/$totalTransitions] 🔄 Generating: $($transition.Name)" -ForegroundColor Cyan
  Write-Host "    Type: $($transition.Type)" -ForegroundColor Gray
  Write-Host "    Description: $($transition.Description)" -ForegroundColor Gray
  Write-Host "    Progress: $progress%" -ForegroundColor Yellow
    
  # Run the slideshow generator
  $command = @("run", "--features", "cli", "--", "-d", "1", "-i", "test_items", "-t", "$($transition.Type)", "-o", "demos/$($transition.Output)")
    
  try {
    # Execute the command and capture output
    $processInfo = Start-Process -FilePath "cargo" -ArgumentList $command -NoNewWindow -Wait -PassThru -RedirectStandardOutput output.log -RedirectStandardError error.log
    $result = Get-Content output.log
    $errorResult = Get-Content error.log
        
    if ($processInfo.ExitCode -eq 0) {
      # Check if file was created and get its size
      $outputPath = "demos/$($transition.Output)"
      if (Test-Path $outputPath) {
        $fileSize = (Get-Item $outputPath).Length
        $fileSizeMB = [math]::Round($fileSize / 1MB, 2)
        Write-Host "    ✅ Success! Generated $outputPath ($fileSizeMB MB)" -ForegroundColor Green
      }
      else {
        Write-Host "    ⚠️  Warning: Command succeeded but file not found" -ForegroundColor Yellow
      }
    }
    else {
      Write-Host "    ❌ Failed with exit code: $($processInfo.ExitCode)" -ForegroundColor Red
      Write-Host "    Error output: $errorResult" -ForegroundColor Red
    }
  }
  catch {
    Write-Host "    ❌ Exception occurred: $($_.Exception.Message)" -ForegroundColor Red
  }
    
  Write-Host ""
}

# Final summary
$endTime = Get-Date
$duration = $endTime - $startTime
$durationMinutes = [math]::Round($duration.TotalMinutes, 1)

Write-Host "🎉 Demo generation completed!" -ForegroundColor Green
Write-Host "Time taken: $durationMinutes minutes" -ForegroundColor Yellow
Write-Host ""

# List generated files
Write-Host "📹 Generated demo videos:" -ForegroundColor Cyan
if (Test-Path "demos") {
  $demoFiles = Get-ChildItem "demos/*.mp4" | Sort-Object Name
    
  if ($demoFiles.Count -gt 0) {
    foreach ($file in $demoFiles) {
      $sizeMB = [math]::Round($file.Length / 1MB, 2)
      Write-Host "    📄 $($file.Name) ($sizeMB MB)" -ForegroundColor White
    }
        
    Write-Host ""
    Write-Host "🎬 You can now watch these videos to see all transition types in action!" -ForegroundColor Green
    Write-Host "🔧 The extensible transition system supports custom durations and types." -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage examples:" -ForegroundColor Yellow
    Write-Host "    cargo run --features cli -- -i your_images -t fade:1.0 -o output.mp4" -ForegroundColor Gray
    Write-Host "    cargo run --features cli -- -i your_images -t slide-left:0.5 -o output.mp4" -ForegroundColor Gray
    Write-Host "    cargo run --features cli -- -i your_images -t dissolve:2.0 -o output.mp4" -ForegroundColor Gray
  }
  else {
    Write-Host "    ⚠️  No demo files found" -ForegroundColor Yellow
  }
}
else {
  Write-Host "    ❌ Demos directory not found" -ForegroundColor Red
}

Write-Host ""
Write-Host "✨ Script completed successfully!" -ForegroundColor Cyan
