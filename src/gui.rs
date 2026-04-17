#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use eframe::egui;
use log::{Level, LevelFilter, Log, Metadata, Record};
use rfd::FileDialog;
use slideshow_generator::{
    BuiltinTransition, SlideDirection, SlideshowGenerator, SlideshowOptions, WipeDirection,
};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

struct GuiLogger {
    buffer: Arc<Mutex<String>>,
}

impl GuiLogger {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(String::new())),
        }
    }

    fn get_buffer(&self) -> Arc<Mutex<String>> {
        Arc::clone(&self.buffer)
    }
}

impl Log for GuiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut buffer = self.buffer.lock().unwrap();
            use std::fmt::Write;
            let _ = writeln!(buffer, "[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

#[derive(Parser)]
#[command(name = "slideshow-generator-gui")]
#[command(about = "GUI for generating slideshow videos")]
struct Cli {
    /// Input directory containing images/videos
    input_dir: Option<PathBuf>,

    /// transition type
    #[arg(
        short = 't',
        long,
        help = r#"
Transition type between slides

Available transitions:
  none, fade, dissolve,
  slide-left, slide-right, slide-up, slide-down,
  wipe-left, wipe-right, wipe-up, wipe-down,
  wipe-diagonal-tl, wipe-diagonal-tr
    "#
    )]
    transition: Option<String>,

    /// Resolution coefficient for auto-detected dimensions (0.0-1.0)
    #[arg(short = 'c', long)]
    resolution_coefficient: Option<f32>,

    /// Duration in seconds for each slide
    #[arg(short = 'd', long)]
    duration_per_slide: Option<f32>,

    /// Duration in seconds for transition effects
    #[arg(short = 'g', long)]
    transition_duration: Option<f32>,
}

#[derive(Clone, PartialEq)]
enum TransitionType {
    Triplet,
    Every,
    None,
    Fade,
    Dissolve,
    Slide(SlideDirection),
    Wipe(WipeDirection),
}

impl TransitionType {
    fn to_builtin(&self, duration: f32) -> BuiltinTransition {
        match self {
            TransitionType::Triplet => BuiltinTransition::None, // This won't be used for Triplet
            TransitionType::Every => BuiltinTransition::None,   // This won't be used for Every
            TransitionType::None => BuiltinTransition::None,
            TransitionType::Fade => BuiltinTransition::fade(duration),
            TransitionType::Dissolve => BuiltinTransition::dissolve(duration),
            TransitionType::Slide(dir) => BuiltinTransition::slide(*dir, duration),
            TransitionType::Wipe(dir) => BuiltinTransition::wipe(*dir, duration),
        }
    }

    fn name(&self) -> &str {
        match self {
            TransitionType::Triplet => "Triplet",
            TransitionType::Every => "Every",
            TransitionType::None => "None",
            TransitionType::Fade => "Fade",
            TransitionType::Dissolve => "Dissolve",
            TransitionType::Slide(SlideDirection::Left) => "Slide Left",
            TransitionType::Slide(SlideDirection::Right) => "Slide Right",
            TransitionType::Slide(SlideDirection::Up) => "Slide Up",
            TransitionType::Slide(SlideDirection::Down) => "Slide Down",
            TransitionType::Wipe(WipeDirection::Left) => "Wipe Left",
            TransitionType::Wipe(WipeDirection::Right) => "Wipe Right",
            TransitionType::Wipe(WipeDirection::Up) => "Wipe Up",
            TransitionType::Wipe(WipeDirection::Down) => "Wipe Down",
            TransitionType::Wipe(WipeDirection::DiagonalTL) => "Wipe Diagonal TL",
            TransitionType::Wipe(WipeDirection::DiagonalTR) => "Wipe Diagonal TR",
        }
    }
}

impl TransitionType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "triplet" => Some(TransitionType::Triplet),
            "every" => Some(TransitionType::Every),
            "none" => Some(TransitionType::None),
            "fade" => Some(TransitionType::Fade),
            "dissolve" => Some(TransitionType::Dissolve),
            "slide-left" => Some(TransitionType::Slide(SlideDirection::Left)),
            "slide-right" => Some(TransitionType::Slide(SlideDirection::Right)),
            "slide-up" => Some(TransitionType::Slide(SlideDirection::Up)),
            "slide-down" => Some(TransitionType::Slide(SlideDirection::Down)),
            "wipe-left" => Some(TransitionType::Wipe(WipeDirection::Left)),
            "wipe-right" => Some(TransitionType::Wipe(WipeDirection::Right)),
            "wipe-up" => Some(TransitionType::Wipe(WipeDirection::Up)),
            "wipe-down" => Some(TransitionType::Wipe(WipeDirection::Down)),
            "wipe-diagonal-tl" => Some(TransitionType::Wipe(WipeDirection::DiagonalTL)),
            "wipe-diagonal-tr" => Some(TransitionType::Wipe(WipeDirection::DiagonalTR)),
            _ => None,
        }
    }
}

struct SlideshowApp {
    input_dir: Option<PathBuf>,
    output_path: Option<PathBuf>,
    duration_per_slide: f32,
    use_custom_dimensions: bool,
    width: u32,
    height: u32,
    resolution_coefficient: f32,
    transition: TransitionType,
    transition_duration: f32,
    log_buffer: Arc<Mutex<String>>,
    generating: bool,
    tx: Sender<Result<(), String>>,
    rx: Receiver<Result<(), String>>,
}

impl Default for SlideshowApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            input_dir: None,
            output_path: None,
            duration_per_slide: 3.0,
            use_custom_dimensions: false,
            width: 1920,
            height: 1080,
            resolution_coefficient: 1.0,
            transition: TransitionType::Triplet,
            transition_duration: 0.5,
            log_buffer: Arc::new(Mutex::new(String::new())), // Placeholder, will be set in main
            generating: false,
            tx,
            rx,
        }
    }
}

impl eframe::App for SlideshowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for messages from the generation thread
        if let Ok(result) = self.rx.try_recv() {
            self.generating = false;
            match result {
                Ok(_) => log::info!("Slideshow generated successfully!"),
                Err(e) => log::error!("Error: {}", e),
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Slideshow Generator");

            ui.separator();

            // Input directory selection
            ui.horizontal(|ui| {
                ui.label("Input Directory:");
                let folder_selected = self.input_dir.is_some();
                let mut select_folder_button = egui::Button::new(
                    egui::RichText::new("Select Folder").color(egui::Color32::WHITE),
                )
                .rounding(4.0);

                // Only apply custom styling when no folder is selected
                if !folder_selected {
                    select_folder_button = select_folder_button
                        .fill(egui::Color32::from_rgb(34, 139, 34)) // Green when no folder selected
                        .stroke(egui::Stroke::new(
                            1.5,
                            egui::Color32::from_rgb(0, 100, 0), // Dark green border
                        ))
                        .min_size(egui::Vec2::new(120.0, 30.0));
                }

                if ui.add(select_folder_button).clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.input_dir = Some(path.clone());
                        // Auto-set output path to input_dir/slideshow.mp4
                        let output_path = path.join("slideshow.mp4");
                        self.output_path = Some(output_path.clone());
                        log::info!("Selected input: {}", path.display());
                        log::info!("Output will be: {}", output_path.display());
                    }
                }
            });

            if let Some(ref path) = self.input_dir {
                ui.label(format!("Selected: {}", path.display()));
            }

            ui.separator();

            // Output file selection
            ui.horizontal(|ui| {
                ui.label("Output File:");
                if ui.button("Select Save Location").clicked() {
                    let mut dialog = FileDialog::new()
                        .add_filter("MP4 Video", &["mp4"])
                        .set_file_name("slideshow.mp4");

                    // If input directory is selected, start from there
                    if let Some(ref input_dir) = self.input_dir {
                        dialog = dialog.set_directory(input_dir);
                    }

                    if let Some(path) = dialog.save_file() {
                        self.output_path = Some(path.clone());
                        log::info!("Selected output: {}", path.display());
                    }
                }
            });

            if let Some(ref path) = self.output_path {
                ui.label(format!("Selected: {}", path.display()));
            }

            ui.separator();

            // Duration per slide
            ui.horizontal(|ui| {
                ui.label("Duration per slide (seconds):");
                ui.add(egui::DragValue::new(&mut self.duration_per_slide).clamp_range(0.5..=10.0));
            });

            // Custom dimensions checkbox
            ui.checkbox(&mut self.use_custom_dimensions, "Use custom resolution");

            // Resolution (only shown if custom dimensions is enabled)
            if self.use_custom_dimensions {
                ui.horizontal(|ui| {
                    ui.label("Resolution:");
                    ui.add(egui::DragValue::new(&mut self.width).clamp_range(640..=3840));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut self.height).clamp_range(480..=2160));
                });
                ui.label("(Custom resolution - coefficient will be ignored)");
            } else {
                ui.label("Resolution: Auto (from first image)");
            }

            // Resolution coefficient (only shown if NOT using custom dimensions)
            if !self.use_custom_dimensions {
                ui.horizontal(|ui| {
                    ui.label("Resolution coefficient:");
                    ui.add(
                        egui::DragValue::new(&mut self.resolution_coefficient)
                            .clamp_range(0.1..=2.0)
                            .speed(0.01),
                    );
                    ui.label("(multiplier for auto-detected resolution)");
                });
            }

            ui.separator();

            // Transition selection
            ui.label("Transition:");
            egui::ComboBox::from_label("")
                .selected_text(self.transition.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.transition, TransitionType::Triplet, "Triplet");
                    ui.selectable_value(&mut self.transition, TransitionType::Every, "Every");
                    ui.selectable_value(&mut self.transition, TransitionType::None, "None");
                    ui.selectable_value(&mut self.transition, TransitionType::Fade, "Fade");
                    ui.selectable_value(&mut self.transition, TransitionType::Dissolve, "Dissolve");
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Slide(SlideDirection::Left),
                        "Slide Left",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Slide(SlideDirection::Right),
                        "Slide Right",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Slide(SlideDirection::Up),
                        "Slide Up",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Slide(SlideDirection::Down),
                        "Slide Down",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Wipe(WipeDirection::Left),
                        "Wipe Left",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Wipe(WipeDirection::Right),
                        "Wipe Right",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Wipe(WipeDirection::Up),
                        "Wipe Up",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Wipe(WipeDirection::Down),
                        "Wipe Down",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Wipe(WipeDirection::DiagonalTL),
                        "Wipe Diagonal TL",
                    );
                    ui.selectable_value(
                        &mut self.transition,
                        TransitionType::Wipe(WipeDirection::DiagonalTR),
                        "Wipe Diagonal TR",
                    );
                });

            if !matches!(
                self.transition,
                TransitionType::None | TransitionType::Every
            ) {
                ui.horizontal(|ui| {
                    ui.label("Transition duration (seconds):");
                    ui.add(
                        egui::DragValue::new(&mut self.transition_duration).clamp_range(0.1..=5.0),
                    );
                });
            }

            ui.separator();

            // Generate button
            let can_generate =
                self.input_dir.is_some() && self.output_path.is_some() && !self.generating;
            let button_text = if self.generating {
                "Generating..."
            } else {
                "Generate Slideshow"
            };
            let button = egui::Button::new(
                egui::RichText::new(button_text)
                    .color(egui::Color32::WHITE)
                    .size(16.0),
            )
            .fill(if can_generate {
                egui::Color32::from_rgb(0, 123, 255)
            } else {
                egui::Color32::GRAY
            })
            .stroke(egui::Stroke::new(
                2.0,
                if can_generate {
                    egui::Color32::from_rgb(0, 100, 200)
                } else {
                    egui::Color32::DARK_GRAY
                },
            ))
            .rounding(6.0)
            .min_size(egui::Vec2::new(200.0, 40.0));

            if ui.add_enabled(can_generate, button).clicked() {
                self.generate_slideshow();
            }

            ui.separator();

            // Log output with scrollbar filling remaining space
            ui.label("Log Output:");
            let available_rect = ui.available_rect_before_wrap();
            let desired_height = available_rect.height();

            egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .auto_shrink([false, false])
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    // Make the text edit take full width and desired height
                    let log_text = self.log_buffer.lock().unwrap().clone();
                    let mut display_text = log_text;
                    let text_edit = egui::TextEdit::multiline(&mut display_text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(
                            (desired_height / ui.text_style_height(&egui::TextStyle::Body)).floor()
                                as usize,
                        );
                    ui.add(text_edit);
                });
        });
    }
}

impl SlideshowApp {
    fn setup_from_command_line(&mut self, folder_path: PathBuf) {
        // Set the input directory
        self.input_dir = Some(folder_path.clone());

        // Auto-set output path to input_dir/slideshow.mp4
        let output_path = folder_path.join("slideshow.mp4");
        self.output_path = Some(output_path.clone());

        // Log setup information
        let transition_name = self.transition.name();
        log::info!("Auto-started with folder: {}", folder_path.display());
        log::info!("Output will be: {}", output_path.display());
        log::info!("Using transition: {}", transition_name);

        // Start generation automatically
        self.generate_slideshow();
    }

    fn generate_slideshow(&mut self) {
        let input_dir = self.input_dir.clone().unwrap();
        let output_path = self.output_path.clone().unwrap();
        let duration_per_slide = self.duration_per_slide;
        let dimensions = if self.use_custom_dimensions {
            Some((self.width, self.height))
        } else {
            None
        };
        let resolution_coefficient = self.resolution_coefficient;
        let transition = self.transition.clone();
        let transition_duration = self.transition_duration;
        let tx = self.tx.clone();

        self.generating = true;
        log::info!("Generating slideshow...");

        thread::spawn(move || {
            let result = (|| {
                match transition {
                    TransitionType::Triplet => {
                        // Generate triplet transitions in parallel
                        Self::generate_triplet_transitions(
                            input_dir,
                            output_path,
                            duration_per_slide,
                            dimensions,
                            transition_duration,
                            resolution_coefficient,
                        )
                    }
                    TransitionType::Every => {
                        // Generate all transitions in parallel
                        Self::generate_all_transitions(
                            input_dir,
                            output_path,
                            duration_per_slide,
                            dimensions,
                            transition_duration,
                            resolution_coefficient,
                        )
                    }
                    _ => {
                        // Generate single slideshow
                        let builtin_transition = transition.to_builtin(transition_duration);
                        let mut options = SlideshowOptions::new()
                            .with_duration_per_slide(duration_per_slide)
                            .with_transition(builtin_transition)
                            .with_resolution_coefficient(resolution_coefficient);

                        if let Some((width, height)) = dimensions {
                            options = options.with_output_resolution(width, height);
                        }

                        let generator = SlideshowGenerator::from_directory(input_dir, options)?;
                        generator.generate(output_path)?;
                        Ok(())
                    }
                }
            })();

            let _ = tx.send(result.map_err(|e: anyhow::Error| e.to_string()));
        });
    }

    fn generate_all_transitions(
        input_dir: PathBuf,
        base_output_path: PathBuf,
        duration_per_slide: f32,
        dimensions: Option<(u32, u32)>,
        transition_duration: f32,
        resolution_coefficient: f32,
    ) -> Result<(), anyhow::Error> {
        // Define all transitions to generate
        let transitions = vec![
            (TransitionType::None, "none"),
            (TransitionType::Fade, "fade"),
            (TransitionType::Dissolve, "dissolve"),
            (TransitionType::Slide(SlideDirection::Left), "slide-left"),
            (TransitionType::Slide(SlideDirection::Right), "slide-right"),
            (TransitionType::Slide(SlideDirection::Up), "slide-up"),
            (TransitionType::Slide(SlideDirection::Down), "slide-down"),
            (TransitionType::Wipe(WipeDirection::Left), "wipe-left"),
            (TransitionType::Wipe(WipeDirection::Right), "wipe-right"),
            (TransitionType::Wipe(WipeDirection::Up), "wipe-up"),
            (TransitionType::Wipe(WipeDirection::Down), "wipe-down"),
            (
                TransitionType::Wipe(WipeDirection::DiagonalTL),
                "wipe-diagonal-tl",
            ),
            (
                TransitionType::Wipe(WipeDirection::DiagonalTR),
                "wipe-diagonal-tr",
            ),
        ];

        // Create output directory if it doesn't exist
        if let Some(parent) = base_output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Generate base filename without extension
        let base_name = base_output_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("slideshow");
        let extension = base_output_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("mp4");

        // Spawn threads for parallel generation
        let mut handles: Vec<thread::JoinHandle<Result<(), anyhow::Error>>> = vec![];

        for (transition_type, suffix) in transitions {
            let input_dir = input_dir.clone();

            let output_path = if suffix == "none" {
                // For "none", use the base filename without suffix
                base_output_path.clone()
            } else {
                // For others, add the suffix before extension
                let file_name = format!("{}.{}.{}", base_name, suffix, extension);
                base_output_path.with_file_name(file_name)
            };

            let handle = thread::spawn(move || {
                let builtin_transition = transition_type.to_builtin(transition_duration);
                let mut options = SlideshowOptions::new()
                    .with_duration_per_slide(duration_per_slide)
                    .with_transition(builtin_transition)
                    .with_resolution_coefficient(resolution_coefficient);

                if let Some((width, height)) = dimensions {
                    options = options.with_output_resolution(width, height);
                }

                let generator = SlideshowGenerator::from_directory(input_dir, options)?;
                generator.generate(output_path)?;
                Ok(())
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Thread panicked"))??;
        }

        Ok(())
    }

    fn generate_triplet_transitions(
        input_dir: PathBuf,
        base_output_path: PathBuf,
        duration_per_slide: f32,
        dimensions: Option<(u32, u32)>,
        transition_duration: f32,
        resolution_coefficient: f32,
    ) -> Result<(), anyhow::Error> {
        // Define the three triplet transitions
        let transitions = vec![
            (TransitionType::Fade, "1"),
            (TransitionType::Slide(SlideDirection::Right), "2"),
            (TransitionType::Wipe(WipeDirection::DiagonalTR), "3"),
        ];

        // Create output directory if it doesn't exist
        if let Some(parent) = base_output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Generate base filename without extension
        let base_name = base_output_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("slideshow");
        let extension = base_output_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("mp4");

        // Spawn threads for parallel generation
        let mut handles: Vec<thread::JoinHandle<Result<(), anyhow::Error>>> = vec![];

        for (transition_type, suffix) in transitions {
            let input_dir = input_dir.clone();

            let output_path =
                base_output_path.with_file_name(format!("{}.{}.{}", base_name, suffix, extension));

            let handle = thread::spawn(move || {
                let builtin_transition = transition_type.to_builtin(transition_duration);
                let mut options = SlideshowOptions::new()
                    .with_duration_per_slide(duration_per_slide)
                    .with_transition(builtin_transition)
                    .with_resolution_coefficient(resolution_coefficient);

                if let Some((width, height)) = dimensions {
                    options = options.with_output_resolution(width, height);
                }

                let generator = SlideshowGenerator::from_directory(input_dir, options)?;
                generator.generate(output_path)?;
                Ok(())
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Thread panicked"))??;
        }

        Ok(())
    }
}

fn show_help_dialog() -> eframe::Result<()> {
    // Get help text from clap
    let help_text = {
        use clap::CommandFactory;
        let mut cmd = Cli::command();
        cmd.render_long_help().to_string()
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 450.0])
            .with_title("Slideshow Generator GUI - Help"),
        ..Default::default()
    };

    eframe::run_native(
        "Slideshow Generator GUI - Help",
        options,
        Box::new(|_cc| Box::new(HelpApp { help_text })),
    )
}

struct HelpApp {
    help_text: String,
}

impl eframe::App for HelpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Slideshow Generator GUI - Help");

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(&self.help_text);
            });

            ui.separator();

            ui.label("Close this window to exit.");
        });
    }
}

fn main() -> eframe::Result<()> {
    // Initialize logging first
    let logger = GuiLogger::new();
    let log_buffer = logger.get_buffer();
    log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger");
    log::set_max_level(LevelFilter::Info);

    // Check for help flags before parsing CLI
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return show_help_dialog();
    }

    let cli = Cli::parse();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_title("Slideshow Generator"),
        ..Default::default()
    };

    // Create the app with the shared log buffer
    let mut app = SlideshowApp {
        log_buffer: log_buffer.clone(),
        ..Default::default()
    };

    log::info!("Slideshow Generator GUI started");

    // Set default transition from CLI if provided
    if let Some(transition_str) = cli.transition {
        if let Some(transition) = TransitionType::from_str(&transition_str) {
            app.transition = transition;
        } else {
            log::warn!(
                "Warning: Unknown transition '{}'. Using default.",
                transition_str
            );
        }
    }

    // Set resolution coefficient from CLI if provided
    if let Some(coef) = cli.resolution_coefficient {
        app.resolution_coefficient = coef;
    }

    // Set slide duration from CLI if provided
    if let Some(duration) = cli.duration_per_slide {
        app.duration_per_slide = duration;
    }

    // Set transition duration from CLI if provided
    if let Some(duration) = cli.transition_duration {
        app.transition_duration = duration;
    }

    // If input directory is provided, set it up automatically
    if let Some(folder_path) = cli.input_dir {
        if folder_path.is_dir() {
            app.setup_from_command_line(folder_path);
        } else {
            log::warn!(
                "Warning: '{}' is not a valid directory.",
                folder_path.display()
            );
        }
    }

    eframe::run_native(
        "Slideshow Generator",
        options,
        Box::new(|_cc| Box::new(app)),
    )
}
