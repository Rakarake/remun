use asmnes::Instruction;
use remun::State;

const NR_ROWS: usize = 40;

pub struct Debugger {
    following_pc: bool,
    disassembly: Vec<(u16, Instruction)>,
    scroll: u16,
}

impl Debugger {
    pub fn new(state: &mut State) -> Self {
        let disassembly = asmnes::disassemble(&(0..=u16::MAX).map(|a| state.read(a, true)).collect::<Vec<u8>>()).0
            .iter().map(|(a, i)| (*a, i.clone())).collect();
        Self { following_pc: true, disassembly, scroll: 0 }
    }
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, state: &mut State) {
        ui.label("Address");
        crate::visualizer::integer_edit_field(ui, &mut self.scroll);
        if ui.small_button("-").clicked() { self.scroll -= 1; }
        if ui.small_button("+").clicked() { self.scroll += 1; }
        ui.toggle_value(&mut self.following_pc, "Following PC");
        if self.following_pc {
            self.scroll = state.pc;
        }
        let current_scroll = self.scroll;
        let input = ctx.input(|i| i.clone());
        self.scroll = (self.scroll as i16 - (input.smooth_scroll_delta.y / 5.) as i16) as u16;
        self.disassembly.iter().skip_while(|(addr, _)| {*addr < current_scroll }).take(NR_ROWS).for_each(|(addr, i)| {
            ui.monospace(format!("{addr:04X}: {i}"));
        });
    }
}
