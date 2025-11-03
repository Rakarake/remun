use std::collections::HashSet;

use asmnes::Instruction;
use egui::{Color32, RichText};
use remun::State;

const NR_ROWS: usize = 40;

pub struct Debugger {
    following_pc: bool,
    /// address, instruction, breakpoint enabled
    disassembly: Vec<(u16, Instruction)>,
    pub breakpoints: HashSet<u64>,
    cursor: u64,
    pub line_number: usize,
}

impl Debugger {
    pub fn new(state: &mut State) -> Self {
        let disassembly = asmnes::disassemble(
            &(0..=u16::MAX)
                .map(|a| state.read(a, true))
                .collect::<Vec<u8>>(),
        )
        .0
        .iter()
        .map(|(a, i)| (*a, i.clone()))
        .collect();
        let breakpoints = HashSet::new();
        Self {
            following_pc: true,
            disassembly,
            cursor: 0xC000,
            line_number: 0,
            breakpoints,
        }
    }
    pub fn jump_to_pc(&mut self, state: &mut State) {
        if let Some(ln) = self
            .disassembly
            .iter()
            .position(|(addr, _)| *addr >= state.pc)
        {
            self.line_number = ln;
        }
    }
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, state: &mut State) {
        ui.toggle_value(&mut self.following_pc, "Following PC");
        if self.following_pc {
            if let Some(ln) = self
                .disassembly
                .iter()
                .position(|(addr, _)| *addr >= state.pc)
            {
                self.line_number = ln;
            }
        }
        let input = ctx.input(|i| i.clone());
        crate::visualizer::scroll_area(ctx, ui, &mut self.line_number, &mut self.cursor);
        if input.key_pressed(egui::Key::Enter) {
            if let Some(ln) = self
                .disassembly
                .iter()
                .position(|(addr, _)| *addr >= (self.cursor as u16))
            {
                self.line_number = ln;
            }
        }
        if input.key_pressed(egui::Key::B) {
            if self.breakpoints.contains(&self.cursor) {
                self.breakpoints.remove(&self.cursor);
            } else {
                self.breakpoints.insert(self.cursor);
            }
        }
        if input.key_pressed(egui::Key::ArrowDown) {
            self.cursor += 1;
        }
        if input.key_pressed(egui::Key::ArrowUp) {
            self.cursor -= 1;
        }
        if input.key_pressed(egui::Key::F) {
            self.following_pc = !self.following_pc;
        }
        self.disassembly[self.line_number..(self.line_number + NR_ROWS)]
            .iter()
            .for_each(|(addr, i)| {
                let breakpoint_symbol = if self.breakpoints.contains(&(*addr as u64)) {
                    "+"
                } else {
                    "|"
                };
                if (self.cursor as u16) >= *addr
                    && (self.cursor as u16) <= *addr + (i.1.arity() as u16)
                {
                    ui.label(
                        RichText::new(format!("{breakpoint_symbol}{addr:04X}: {i}"))
                            .color(Color32::GREEN),
                    );
                } else if state.pc >= *addr && state.pc <= *addr + (i.1.arity() as u16) {
                    ui.label(
                        RichText::new(format!("{breakpoint_symbol}{addr:04X}: {i}"))
                            .color(Color32::YELLOW),
                    );
                } else {
                    ui.monospace(format!("{breakpoint_symbol}{addr:04X}: {i}"));
                }
            });
    }
}
