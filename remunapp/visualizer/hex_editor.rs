use egui::{Color32, Key, RichText};
use remun::State;

const NR_ROWS: usize = 40;

pub struct HexEditor {
    /// the address we are looking at
    line_number: usize,
    cursor: u64,
    binary_type: BinaryType,
}

enum BinaryType {
    CpuBus,
    Banks,
}

impl HexEditor {
    pub fn new() -> Self {
        Self {
            line_number: 0xC000 / 16,
            cursor: 0xC000,
            binary_type: BinaryType::CpuBus,
        }
    }
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, state: &mut State) {
        crate::visualizer::scroll_area(ctx, ui, &mut self.line_number, &mut self.cursor);
        let input = ctx.input(|i| i.clone());
        if input.key_pressed(Key::Enter) {
            self.line_number = self.cursor as usize / 16;
        }
        if input.key_pressed(Key::U) {
            self.binary_type = BinaryType::CpuBus;
        }
        if input.key_pressed(Key::I) {
            self.binary_type = BinaryType::Banks;
        }
        let get_function: Box<dyn Fn(u64, &mut State) -> u8> = match self.binary_type {
            BinaryType::CpuBus => {
                ui.heading("CPU Bus View");
                Box::new(|addr: u64, state: &mut State| -> u8 {
                    state.read(addr as u16, true)
                })
            },
            BinaryType::Banks => {
                ui.heading("Bank View");
                Box::new(|addr: u64, state: &mut State| -> u8 {
                    *state.ines.banks.get(addr as usize).unwrap_or(&0)
                })
            }
        };
        let mut addr = self.line_number as u64 * 16;
        for _ in 0..NR_ROWS {
            ui.horizontal(|ui| {
                ui.monospace(format!("{addr:04X}: "));
                loop {
                    let val = get_function(addr, state);
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
