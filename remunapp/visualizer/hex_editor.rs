use egui::{Color32, RichText};
use remun::State;

const NR_ROWS: usize = 40;

pub struct HexEditor {
    /// the address we are looking at
    line_number: usize,
    cursor: u16,
}

impl HexEditor {
    pub fn new() -> Self {
        Self {
            line_number: 0xC000 / 16,
            cursor: 0xC000,
        }
    }
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, state: &mut State) {
        crate::visualizer::scroll_area(ctx, ui, &mut self.line_number, &mut self.cursor);
        let mut addr = self.line_number as u16 * 16;
        for _ in 0..NR_ROWS {
            ui.horizontal(|ui| {
                ui.monospace(format!("{addr:04X}: "));
                loop {
                    let val = state.read(addr, true);
                    if addr == self.cursor {
                        ui.monospace(RichText::new(format!("{val:02X}")).color(Color32::GREEN));
                    } else {
                        ui.monospace(format!("{val:02X}"));
                    }
                    addr += 1;
                    if addr % 16 == 0 {
                        break;
                    }
                }
            });
        }
    }
}
