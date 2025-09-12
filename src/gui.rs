use eframe::egui;
use rfd::FileDialog;
use slideshow_generator::{SlideshowGenerator, SlideshowOptions, BuiltinTransition, SlideDirection, WipeDirection};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Clone, PartialEq)]
enum TransitionType {
    None,
    Fade,
    Dissolve,
    Slide(SlideDirection),
    Wipe(WipeDirection),
}

impl TransitionType {
    fn to_builtin(&self, duration: f32) -> BuiltinTransition {
        match self {
            TransitionType::None => BuiltinTransition::None,
            TransitionType::Fade => BuiltinTransition::fade(duration),
            TransitionType::Dissolve => BuiltinTransition::dissolve(duration),
            TransitionType::Slide(dir) => BuiltinTransition::slide(*dir, duration),
            TransitionType::Wipe(dir) => BuiltinTransition::wipe(*dir, duration),
        }
    }

    fn name(&self) -> &str {
        match self {
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
            TransitionType::Wipe(WipeDirection::DiagonalTL) => "Wipe Diagonal TL-BR",
            TransitionType::Wipe(WipeDirection::DiagonalTR) => "Wipe Diagonal TR-BL",
        }
    }
}

struct SlideshowApp {
    input_dir: Option<PathBuf>,
    output_path: Option<PathBuf>,
    duration_per_slide: f32,
    width: u32,
    height: u32,
    transition: TransitionType,
    transition_duration: f32,
    status: String,
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
            width: 1920,
            height: 1080,
            transition: TransitionType::None,
            transition_duration: 1.0,
            status: "Ready".to_string(),
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
                Ok(_) => self.status = "Slideshow generated successfully!".to_string(),
                Err(e) => self.status = format!("Error: {}", e),
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Slideshow Generator");

            ui.separator();

            // Input directory selection
            ui.horizontal(|ui| {
                ui.label("Input Directory:");
                if ui.button("Select Folder").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.input_dir = Some(path.clone());
                        self.status = format!("Selected input: {}", path.display());
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
                    if let Some(path) = FileDialog::new()
                        .add_filter("MP4 Video", &["mp4"])
                        .set_file_name("slideshow.mp4")
                        .save_file() {
                        self.output_path = Some(path.clone());
                        self.status = format!("Selected output: {}", path.display());
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

            // Resolution
            ui.horizontal(|ui| {
                ui.label("Resolution:");
                ui.add(egui::DragValue::new(&mut self.width).clamp_range(640..=3840));
                ui.label("x");
                ui.add(egui::DragValue::new(&mut self.height).clamp_range(480..=2160));
            });

            ui.separator();

            // Transition selection
            ui.label("Transition:");
            egui::ComboBox::from_label("")
                .selected_text(self.transition.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.transition, TransitionType::None, "None");
                    ui.selectable_value(&mut self.transition, TransitionType::Fade, "Fade");
                    ui.selectable_value(&mut self.transition, TransitionType::Dissolve, "Dissolve");
                    ui.selectable_value(&mut self.transition, TransitionType::Slide(SlideDirection::Left), "Slide Left");
                    ui.selectable_value(&mut self.transition, TransitionType::Slide(SlideDirection::Right), "Slide Right");
                    ui.selectable_value(&mut self.transition, TransitionType::Slide(SlideDirection::Up), "Slide Up");
                    ui.selectable_value(&mut self.transition, TransitionType::Slide(SlideDirection::Down), "Slide Down");
                    ui.selectable_value(&mut self.transition, TransitionType::Wipe(WipeDirection::Left), "Wipe Left");
                    ui.selectable_value(&mut self.transition, TransitionType::Wipe(WipeDirection::Right), "Wipe Right");
                    ui.selectable_value(&mut self.transition, TransitionType::Wipe(WipeDirection::Up), "Wipe Up");
                    ui.selectable_value(&mut self.transition, TransitionType::Wipe(WipeDirection::Down), "Wipe Down");
                    ui.selectable_value(&mut self.transition, TransitionType::Wipe(WipeDirection::DiagonalTL), "Wipe Diagonal TL-BR");
                    ui.selectable_value(&mut self.transition, TransitionType::Wipe(WipeDirection::DiagonalTR), "Wipe Diagonal TR-BL");
                });

            if !matches!(self.transition, TransitionType::None) {
                ui.horizontal(|ui| {
                    ui.label("Transition duration (seconds):");
                    ui.add(egui::DragValue::new(&mut self.transition_duration).clamp_range(0.1..=5.0));
                });
            }

            ui.separator();

            // Generate button
            let can_generate = self.input_dir.is_some() && self.output_path.is_some() && !self.generating;
            if ui.add_enabled(can_generate, egui::Button::new("Generate Slideshow")).clicked() {
                self.generate_slideshow();
            }

            ui.separator();

            // Status
            ui.label(&self.status);
        });
    }
}

impl SlideshowApp {
    fn generate_slideshow(&mut self) {
        let input_dir = self.input_dir.clone().unwrap();
        let output_path = self.output_path.clone().unwrap();
        let duration_per_slide = self.duration_per_slide;
        let width = self.width;
        let height = self.height;
        let transition = self.transition.to_builtin(self.transition_duration);
        let tx = self.tx.clone();

        self.generating = true;
        self.status = "Generating slideshow...".to_string();

        thread::spawn(move || {
            let result = (|| {
                let options = SlideshowOptions::new()
                    .with_duration_per_slide(duration_per_slide)
                    .with_output_resolution(width, height)
                    .with_transition(transition);

                let generator = SlideshowGenerator::from_directory(input_dir, options)?;
                generator.generate(output_path)?;
                Ok(())
            })();

            let _ = tx.send(result.map_err(|e: anyhow::Error| e.to_string()));
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_title("Slideshow Generator"),
        ..Default::default()
    };

    eframe::run_native(
        "Slideshow Generator",
        options,
        Box::new(|_cc| Box::new(SlideshowApp::default())),
    )
}
