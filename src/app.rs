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

pub fn run(state: State) -> eframe::Result {
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
            let state = state;
            Ok(Box::new(MyApp {
                state,
                running: false,
                speed: 1,
                scroll: 0,
                following_pc: false,
                file_path: "".to_string(),
            }))
        }),
    )
}

struct MyApp {
    state: State,
    running: bool,
    /// Instructions per second.
    speed: u32,
    scroll: usize,
    following_pc: bool,
    /// Path to ROM/assembly file
    file_path: String,
}

const NR_SHOWN_INSTRUCTIONS: usize = 30;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("side_bar").show(ctx, |ui| {
            if ui.button("Open ROM/assembly file").clicked() {
                let files = FileDialog::new()
                    .add_filter("text", &["txt", "rs"])
                    .add_filter("rust", &["rs", "toml"])
                    .set_directory("/")
                    .pick_file();
                println!("{:?}", files);
            }
            ui.text_edit_singleline(&mut self.file_path);
            // TODO do both
            if ui.button("Load File (.nes or .asm)").clicked() {
                self.state.ines = asmnes::assemble_from_file(self.file_path.as_str()).unwrap();
            }
            ui.add(Slider::new(&mut self.scroll, 0..=self.state.ines.banks.len()-1).step_by(1.0));
            if ui.small_button("+").clicked() { self.scroll += 1; }
            if ui.small_button("-").clicked() { self.scroll -= 1; }
            ui.toggle_value(&mut self.running, "Running");
            ui.toggle_value(&mut self.following_pc, "Following PC");
            if self.following_pc {
                // TODO take other banks into consideration lol
                self.scroll = (self.state.pc - 0xC000) as usize;
            }
            ui.label(format!("A: ${:02X}", self.state.a));
            ui.label(format!("X: ${:02X}", self.state.x));
            ui.label(format!("Y: ${:02X}", self.state.y));
            ui.label(format!("SR: ${:02X}", self.state.sr));
            ui.label(format!("SP: ${:02X}", self.state.sp));
            ui.label(format!("PC: ${:04X}", self.state.pc));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ptr = &self.state.ines.banks[self.scroll..];
            let mut line_count: usize = 0;
            while let Some((i, len)) = Instruction::from_bytes(ptr)
                        && line_count < NR_SHOWN_INSTRUCTIONS {
                ui.monospace(i.to_string());
                ptr = &ptr[len..];
                line_count += 1;
            }
                //write!(f, "${n:04X}")
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
