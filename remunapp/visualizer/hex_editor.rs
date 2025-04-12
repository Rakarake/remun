use remun::State;

const NR_ROWS: usize = 40;

pub struct HexEditor {
    /// the address we are looking at
    scroll: u16,
}

impl HexEditor {
    pub fn new() -> Self {
        Self {
            scroll: 0xC000,
        }
    }
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, state: &mut State) {
        ui.label("Address");
        ui.horizontal(|ui| {
            if ui.small_button("-").clicked() { self.scroll -= 1; }
            crate::visualizer::integer_edit_field(ui, &mut self.scroll);
            if ui.small_button("+").clicked() { self.scroll += 1; }
        });
        let input = ctx.input(|i| i.clone());
        let scroll_addr = (self.scroll as i16 - (input.smooth_scroll_delta.y * 2.) as i16) as u16;
        self.scroll = scroll_addr - scroll_addr % 16;
        let mut addr = self.scroll;
        for _ in 0..NR_ROWS {
            ui.horizontal(|ui| {
                ui.monospace(format!("{addr:04X}: "));
                loop {
                    let val = state.read(addr, true);
                    ui.monospace(format!("{val:02X}"));
                    addr += 1;
                    if addr % 16 == 0 {
                        break;
                    }
                }
            });
            addr += 16;
        }
    }
}
