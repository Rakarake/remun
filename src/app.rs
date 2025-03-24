#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use asmnes::AsmnesError;
use asmnes::Directive;
use asmnes::Instruction;
use asmnes::Operand::*;
use eframe::egui;
use remun::State;
use shared::AddressingMode::*;
use shared::Opcode::*;

// TODO make asmnes program struct, takes in ines or (prg, debug, char)? (no, depends on mappers)
// new_form_regions(regions, debug)

pub fn run(state: State) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Remnun debugger",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let disassembly = asmnes::disassemble(&state.ines.banks).0.iter().fold(String::new(), |mut acc, i| {
                acc.push_str(&format!("{i}\n"));
                acc
            });
            let state = state;
            Ok(Box::new(MyApp {
                state,
                running: false,
                speed: 1,
                scroll: 0,
            }))
        }),
    )
}

struct MyApp {
    state: State,
    running: bool,
    /// Instructions per second.
    speed: u32,
    scroll: usize,
}

const NR_SHOWN_INSTRUCTIONS: usize = 0;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.toggle_value(&mut self.running, "Running");
            ui.label(format!("A: ${:02X}", self.state.a));
            ui.label(format!("X: ${:02X}", self.state.x));
            ui.label(format!("Y: ${:02X}", self.state.y));
            ui.label(format!("SR: ${:02X}", self.state.sr));
            ui.label(format!("SP: ${:02X}", self.state.sp));
            ui.label(format!("PC: ${:04X}", self.state.pc));
            let mut ptr = &self.state.ines.banks[..];
            while let Some((i, len)) = Instruction::from_bytes(ptr) {
                ui.monospace(i.to_string());
                ptr = &ptr[len..];
            }
                //write!(f, "${n:04X}")
            //ui.image(egui::include_image!(
            //    "../logo.png"
            //));
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

        });
    }
}
