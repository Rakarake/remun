#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use asmnes::AsmnesError;
use asmnes::AsmnesOutput;
use shared::Opcode::*;
use shared::AddressingMode::*;
use asmnes::Operand::*;
use asmnes::Directive;
use remun::State;
use eframe::egui;

// TODO make asmnes program struct, takes in ines or (prg, debug, char)? (no, depends on mappers)
// new_form_regions(regions, debug)

pub fn run(state: State) -> eframe::Result {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp {
                state,
                running: false,
                speed: 1,
            }))
        }),
    )
}

struct MyApp {
    state: State,
    running: bool,
    /// Instructions per second.
    speed: u32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.toggle_value(&mut self.running, "Running");
            ui.label(format!("running: {}", self.running));
            //ui.heading("My egui Application");
            //ui.horizontal(|ui| {
            //    let name_label = ui.label("Your name: ");
            //    ui.text_edit_singleline(&mut self.name)
            //        .labelled_by(name_label.id);
            //});
            //ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            //if ui.button("Increment").clicked() {
            //    self.age += 1;
            //}
            //ui.label(format!("Hello '{}', age {}", self.name, self.age));

            //ui.image(egui::include_image!(
            //    "../logo.png"
            //));
        });
    }
}

