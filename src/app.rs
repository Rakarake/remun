use asmnes::AsmnesError;
use asmnes::Directive;
use asmnes::Instruction;
use asmnes::Operand::*;
use eframe::egui;
use eframe::egui::Slider;
use remun::State;
use shared::AddressingMode::*;
use shared::Opcode::*;
use rfd::FileDialog;

// TODO make asmnes program struct, takes in ines or (prg, debug, char)? (no, depends on mappers)
// new_form_regions(regions, debug)

pub fn run(mut state: State) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Remnun debugger",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let disassembly = asmnes::disassemble(&(0..=u16::MAX).map(|a| state.read(a, true)).collect::<Vec<u8>>()).0
                .iter().map(|(a, i)| (*a as u16, i.clone())).collect();
            Ok(Box::new(MyApp {
                state,
                running: false,
                speed: 1,
                scroll: 0xC000,
                following_pc: true,
                disassembly,
            }))
        }),
    )
}

struct MyApp {
    state: State,
    running: bool,
    /// Instructions per second.
    speed: u32,
    scroll: u16,
    following_pc: bool,
    disassembly: Vec<(u16, Instruction)>,
}

const NR_SHOWN_INSTRUCTIONS: usize = 30;

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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_bar").show(ctx, |ui| {
            if ui.button("Open ROM/assembly file").clicked() {
                let files = FileDialog::new()
                    .add_filter("text", &["txt", "rs"])
                    .add_filter("rust", &["rs", "toml"])
                    .set_directory("/")
                    .pick_file();
                println!("{:?}", files);
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
                self.state.run_one_instruction();
            }
            ui.toggle_value(&mut self.running, "Running");
            ui.toggle_value(&mut self.following_pc, "Following PC");
            if self.following_pc {
                // TODO take other banks into consideration lol
                self.scroll = self.state.pc;
            }
            ui.label(format!("A: ${:02X}", self.state.a));
            ui.label(format!("X: ${:02X}", self.state.x));
            ui.label(format!("Y: ${:02X}", self.state.y));
            ui.label(format!("SR: ${:02X}", self.state.sr));
            ui.label(format!("SP: ${:02X}", self.state.sp));
            ui.label(format!("PC: ${:04X}", self.state.pc));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.disassembly.iter().skip_while(|(addr, _)| *addr < self.scroll).take(NR_SHOWN_INSTRUCTIONS).for_each(|(addr, i)| {
                ui.monospace(format!("{addr:04X}: {i}"));
            });
            //ui.image(egui::include_image!(
            //    "../logo.png"
            //));
            //ui.heading("My egui Application");
            //ui.horizontal(|ui| {
            //    let name_label = ui.label("Your name: ");
            //    ui.text_edit_singleline(&mut self.name)
            //        .labelled_by(name_label.id);
            //});
            //ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            //if ui.button("Increment").clicked() {
            //    self.age += 1;
            //}
            //ui.label(format!("Hello '{}', age {}", self.name, self.age));

        });
    }
}
