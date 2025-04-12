use asmnes::Instruction;
use egui::{Color32, RichText};
use remun::State;

const NR_ROWS: usize = 40;

pub struct Debugger {
    following_pc: bool,
    disassembly: Vec<(u16, Instruction)>,
    cursor: u16,
    line_number: usize,
}

impl Debugger {
    pub fn new(state: &mut State) -> Self {
        let disassembly = asmnes::disassemble(&(0..=u16::MAX).map(|a| state.read(a, true)).collect::<Vec<u8>>()).0
            .iter().map(|(a, i)| (*a, i.clone())).collect();
        Self { following_pc: true, disassembly, cursor: 0, line_number: 0 }
    }
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, state: &mut State) {
        ui.label("Address");
        ui.horizontal(|ui| {
            if ui.small_button("-").clicked() { self.cursor -= 1; }
            crate::visualizer::integer_edit_field(ui, &mut self.cursor);
            if ui.small_button("+").clicked() { self.cursor += 1; }
        });
        ui.toggle_value(&mut self.following_pc, "Following PC");
        if self.following_pc {
            self.cursor = state.pc;
        }
        let input = ctx.input(|i| i.clone());
        let new_line_number = self.line_number as isize - (input.smooth_scroll_delta.y / 5.) as isize;
        if input.key_down(egui::Key::Enter) {
            self.line_number = self.cursor;
        }
        if new_line_number >= 0 { self.line_number = new_line_number as usize }
        self.disassembly[self.line_number..(self.line_number+NR_ROWS)].iter().for_each(|(addr, i)| {
            if *addr == self.cursor {
                ui.monospace(RichText::new(format!("{addr:04X}: {i}")).monospace().color(Color32::GREEN));
            }
            if *addr == state.pc {
                ui.monospace(RichText::new(format!("{addr:04X}: {i}")).monospace().color(Color32::YELLOW));
            }
            ui.monospace(format!("{addr:04X}: {i}"));
        });
    }
}
