//! Visualizer of state, showing decompilation etc.
use std::collections::BTreeMap;
use std::sync::Arc;

use asmnes::AsmnesError;
use asmnes::Directive;
use asmnes::Operand::*;
use egui;
use egui::Color32;
use egui::FontData;
use egui::FontDefinitions;
use egui::FontFamily;
use egui::FontId;
use egui::Key;
use egui::Slider;
use egui::TextStyle;
use remun::State;
use shared::AddressingMode::*;
use shared::Opcode::*;
use rfd::FileDialog;
use std::time::Instant;

mod debugger;
mod hex_editor;

use debugger::Debugger;
use hex_editor::HexEditor;

pub struct Visualizer {
    hidden: bool,
    running: bool,
    /// Instructions per second.
    speed: u64,
    time_last_frame: Instant,
    view: View,
    debugger: Debugger,
    hex_editor: HexEditor,
}

enum View {
    Disassembly,
    HexEditor,
}

fn font_setup(ctx: &egui::Context) {
    // Set fonts
    let name = "monogram";
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(name.to_owned(),
                           Arc::new(egui::FontData::from_static(include_bytes!("monogram.ttf"))));
    fonts.families.insert(egui::FontFamily::Name("Helvetica".into()), vec!["Helvetica".to_owned()]);
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap() //it works
        .insert(0, name.to_owned());
    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
        .insert(0, name.to_owned());//.push("Helvetica".to_owned());
    ctx.set_fonts(fonts);

    // Set style
    let mut style = (*ctx.style()).clone();
    use FontFamily::Proportional;
    use TextStyle::*;
    style.text_styles = [
        (Heading, FontId::new(26.0, Proportional)),
        (Body, FontId::new(22.0, Proportional)),
        (Monospace, FontId::new(22.0, Proportional)),
        (Button, FontId::new(22.0, Proportional)),
        (Small, FontId::new(20.0, Proportional)),
    ]
    .into();
    style.visuals.override_text_color = Some(Color32::WHITE);
    ctx.set_style(style);
}

impl Visualizer {
    pub fn new(ctx: &egui::Context, state: &mut State) -> Self {
        font_setup(ctx);
        egui_extras::install_image_loaders(ctx);
        Self {
            hidden: false,
            running: false,
            speed: 1,
            view: View::Disassembly,
            debugger: Debugger::new(state),
            hex_editor: HexEditor::new(),
            time_last_frame: Instant::now(),
        }
    }
    pub fn update(&mut self, ctx: &egui::Context, state: &mut State) {
        //egui::Window::new("hello").show(ctx, |ui| {
        use egui::containers::Frame;
        use egui::ecolor::Color32;
        let frame = Frame::default().fill(Color32::from_rgba_unmultiplied(0, 0, 0, 0xF0));
        egui::SidePanel::left("left_bar").frame(frame).show(ctx, |ui| {
            if let Some(data_source) = &state.ines.data_source {
                ui.label(format!("Loaded file: {}", data_source.display()));
            }
            if ui.button("Open ROM/assembly file").clicked() {
                let path = FileDialog::new()
                    .add_filter("NES Rom", &["nes"])
                    .add_filter("Assembly", &["asm"])
                    .pick_file();
                // just log the errors in the console!
                if let Some(path) = path  {
                    match remun::load_from_file(path) {
                        Ok(ines) => *state = State::new(ines),
                        Err(e) => log::error!("{e}"),
                    }
                } else {
                    log::warn!("failed to open file!");
                }
            }
            if ui.button("Soft Reset").clicked() {
                state.reset();
            }
            if ui.button("Hard Reset").clicked() {
                *state = State::new(state.ines.clone());
            }
            if ui.small_button("step").clicked() {
                state.run_one_instruction();
            }
            ui.heading("Run speed (instructions per second)");
            integer_edit_field(ui, &mut self.speed);
            if ui.toggle_value(&mut self.running, "Running").clicked() {
                self.time_last_frame = Instant::now();
            }
            if self.running {
                // run amount of instructions needed since last time frame
                let delta = self.time_last_frame.elapsed();
                let instructions_to_run = (delta.as_millis()) * self.speed as u128 / 1000;
                for _ in 0..instructions_to_run {
                    state.run_one_instruction();
                    self.time_last_frame = Instant::now();
                    if self.debugger.breakpoints.contains(&(state.pc as u64)) {
                        self.running = false;
                        break;
                    }
                }
            }
            ui.monospace(format!("PC: ${:04X}", state.pc));
            ui.monospace(format!("A: ${:02X}", state.a));
            ui.monospace(format!("X: ${:02X}", state.x));
            ui.monospace(format!("Y: ${:02X}", state.y));
            ui.monospace(format!("SP: ${:02X}", state.sp));
            ui.label("Status registers");
            macro_rules! show_flag {
                ($i:tt) => {
                    ui.monospace(format!("{}: {}", stringify!($i), (state.sr & shared::flags::$i) != 0));
                };
            }
            show_flag!(N);
            show_flag!(V);
            show_flag!(B);
            show_flag!(D);
            show_flag!(I);
            show_flag!(Z);
            show_flag!(C);

            ui.label("PPU state");
            if let Some(addr) = state.ppu_state.tmp_addr {
                ui.monospace(format!("temporary address: ${:04X}", addr));
            } else {
                ui.monospace("temporary address: no!");
            }
            ui.monospace(format!("temporary value: ${:02X}", state.ppu_state.tmp_val));

            //ui.image(egui::include_image!(
            //    "../logo.png"
            //));
        });
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let input = ctx.input(|i| i.clone());
            if input.key_pressed(Key::H) {
                self.view = View::HexEditor;
            }
            if input.key_pressed(Key::Y) {
                self.view = View::Disassembly;
            }
            match self.view {
                View::Disassembly => {
                    self.debugger.update(ctx, ui, state);
                },
                View::HexEditor => {
                    self.hex_editor.update(ctx, ui, state);
                },
            }
        });
    }
}

// slightly ugly way to input only valid numbers: https://github.com/emilk/egui/issues/1348#issuecomment-1652168882
fn integer_edit_field(ui: &mut egui::Ui, value: &mut u64) -> egui::Response {
    let mut tmp_value = format!("{:X}", value);
    let res = ui.text_edit_singleline(&mut tmp_value);
    if let Ok(result) = u64::from_str_radix(&tmp_value, 16) {
        *value = result;
    } else {
        *value = 0;
    }
    res
}

fn scroll_area(ctx: &egui::Context, ui: &mut egui::Ui, scroll: &mut usize, cursor: &mut u64) {
    ui.label("Cursor");
    ui.horizontal(|ui| {
        if ui.small_button("-").clicked() { *cursor -= 1; }
        crate::visualizer::integer_edit_field(ui, &mut *cursor);
        if ui.small_button("+").clicked() { *cursor += 1; }
    });
    let input = ctx.input(|i| i.clone());
    let new_line_number = *scroll as isize - (input.smooth_scroll_delta.y / 5.) as isize;
    if new_line_number >= 0 { *scroll = new_line_number as usize }
}


