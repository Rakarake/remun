//! Visualizer of state, showing decompilation etc.
use std::collections::BTreeMap;
use std::sync::Arc;

use asmnes::AsmnesError;
use asmnes::Directive;
use asmnes::Instruction;
use asmnes::Operand::*;
use egui;
use egui::FontData;
use egui::FontDefinitions;
use egui::FontFamily;
use egui::FontId;
use egui::Slider;
use egui::TextStyle;
use remun::State;
use shared::AddressingMode::*;
use shared::Opcode::*;
use rfd::FileDialog;

pub struct Visualizer {
    running: bool,
    /// Instructions per second.
    speed: u32,
    scroll: u16,
    following_pc: bool,
    disassembly: Vec<(u16, Instruction)>,
}

const NR_SHOWN_INSTRUCTIONS: usize = 30;

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
        (Heading, FontId::new(30.0, Proportional)),
        (Body, FontId::new(18.0, Proportional)),
        (Monospace, FontId::new(22.0, Proportional)),
        (Button, FontId::new(14.0, Proportional)),
        (Small, FontId::new(10.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

impl Visualizer {
    pub fn new(ctx: &egui::Context) -> Self {
        font_setup(ctx);
        Self {
            running: false,
            speed: 1,
            scroll: 0,
            following_pc: true,
            disassembly: Vec::new(),
        }
    }
    pub fn update(&mut self, ctx: &egui::Context, state: &mut State) {
        egui::SidePanel::left("left_bar").show(ctx, |ui| {
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
            //ui.text_edit_singleline(&mut self.file_path);
            // TODO do both
            //if ui.button("Load File (.nes or .asm)").clicked() {
            //    self.state = State::new(asmnes::assemble_from_file(self.file_path.as_str()).unwrap());
            //}
            //ui.add(Slider::new(&mut self.scroll, 0..=u16::MAX).step_by(1.0));
            ui.label("Address");
            integer_edit_field(ui, &mut self.scroll);
            if ui.small_button("+").clicked() { self.scroll += 1; }
            if ui.small_button("-").clicked() { self.scroll -= 1; }
            if ui.small_button("step").clicked() {
                state.run_one_instruction();
            }
            ui.toggle_value(&mut self.running, "Running");
            ui.toggle_value(&mut self.following_pc, "Following PC");
            if self.following_pc {
                // TODO take other banks into consideration lol
                self.scroll = state.pc;
            }
            ui.monospace(format!("A: ${:02X}", state.a));
            ui.monospace(format!("X: ${:02X}", state.x));
            ui.monospace(format!("Y: ${:02X}", state.y));
            ui.monospace(format!("SR: ${:02X}", state.sr));
            ui.monospace(format!("SP: ${:02X}", state.sp));
            ui.monospace(format!("PC: ${:04X}", state.pc));
        });
        //egui::CentralPanel::default().show(ctx, |ui| {
        //    self.disassembly.iter().skip_while(|(addr, _)| *addr < self.scroll).take(NR_SHOWN_INSTRUCTIONS).for_each(|(addr, i)| {
        //        ui.monospace(format!("{addr:04X}: {i}"));
        //    });
        //    //ui.image(egui::include_image!(
        //    //    "../logo.png"
        //    //));
        //    //ui.heading("My egui Application");
        //    //ui.horizontal(|ui| {
        //    //    let name_label = ui.label("Your name: ");
        //    //    ui.text_edit_singleline(&mut self.name)
        //    //        .labelled_by(name_label.id);
        //    //});
        //    //ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
        //    //if ui.button("Increment").clicked() {
        //    //    self.age += 1;
        //    //}
        //    //ui.label(format!("Hello '{}', age {}", self.name, self.age));
    
        //});
    }
}

// slightly ugly way to input only valid numbers: https://github.com/emilk/egui/issues/1348#issuecomment-1652168882
fn integer_edit_field(ui: &mut egui::Ui, value: &mut u16) -> egui::Response {
    let mut tmp_value = format!("{:X}", value);
    let res = ui.text_edit_singleline(&mut tmp_value);
    if let Ok(result) = u16::from_str_radix(&tmp_value, 16) {
        *value = result;
    } else {
        *value = 0;
    }
    res
}
