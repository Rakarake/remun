use asmnes::Instruction;
use egui::{Color32, RichText};
use remun::State;
use std::io::Write;
use std::path::Path;

const NR_ROWS: usize = 40;

pub struct Debugger {
    following_pc: bool,
    /// address, instruction, breakpoint enabled
    disassembly: Vec<(u16, Instruction)>,
    cursor: u64,
    pub line_number: usize,
    new_label_text: String,
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
        Self {
            following_pc: true,
            disassembly,
            cursor: 0xC000,
            line_number: 0,
            new_label_text: String::new(),
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
        if ui.button("Load metadata").clicked() {}
        if ui.button("Save metadata").clicked() {
            save_metadata(state);
        }
        ui.text_edit_singleline(&mut self.new_label_text);
        if ui.button("Add label").clicked() && !self.new_label_text.is_empty() {
            state
                .ines
                .metadata
                .get_or_insert_default()
                .labels
                .insert(self.new_label_text.clone(), self.cursor as u16);
        }
        if ui.button("Remove label").clicked() {
            state
                .ines
                .metadata
                .get_or_insert_default()
                .labels
                .remove(&self.new_label_text);
        }
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
            let m = state.ines.metadata.get_or_insert_default();
            if m.breakpoints.contains(&(self.cursor as u16)) {
                m.breakpoints.remove(&(self.cursor as u16));
            } else {
                m.breakpoints.insert(self.cursor as u16);
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
                let breakpoint_symbol = if let Some(m) = state.ines.metadata.as_ref()
                    && m.breakpoints.contains(addr)
                {
                    "*"
                } else {
                    " "
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

fn save_metadata(state: &State) -> Option<()> {
    let metadata = state.ines.metadata.as_ref()?;
    let data_source = metadata.data_source.as_ref()?;
    let json_string = serde_json::to_string_pretty(metadata)
        .map_err(|e| log::error!("{e}"))
        .ok()?;
    let parent_dir = data_source.parent()?;
    let path = parent_dir.join(Path::new("metadata.json"));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .map_err(|e| log::error!("{e}"))
        .ok()?;
    write!(file, "{}", json_string)
        .map_err(|e| log::error!("{e}"))
        .ok()
}
