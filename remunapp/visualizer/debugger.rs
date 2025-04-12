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
        ui.toggle_value(&mut self.following_pc, "Following PC");
        if self.following_pc {
            self.cursor = state.pc;
        }
        let input = ctx.input(|i| i.clone());
        crate::visualizer::scroll_area(ctx, ui, &mut self.line_number, &mut self.cursor);
        if input.key_pressed(egui::Key::Enter) || self.following_pc {
            if let Some(ln) = self.disassembly.iter().position(|(addr, _)| *addr >= self.cursor) {
                self.line_number = ln;
            }
        }
        self.disassembly[self.line_number..(self.line_number+NR_ROWS)].iter().for_each(|(addr, i)| {
            if self.cursor >= *addr && self.cursor <= *addr + (i.1.arity() as u16) {
                ui.label(RichText::new(format!("{addr:04X}: {i}")).color(Color32::GREEN));
            } else if state.pc >= *addr && state.pc <= *addr + (i.1.arity() as u16) {
                ui.label(RichText::new(format!("{addr:04X}: {i}")).color(Color32::YELLOW));
            } else {
                ui.monospace(format!("{addr:04X}: {i}"));
            }
        });
    }
}
