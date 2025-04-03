//! The app, creates window, renders the game etc.
use remun::State;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run(mut state: State) {
    //let options = eframe::NativeOptions {
    //    viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
    //    ..Default::default()
    //};
    //eframe::run_native(
    //    "Remnun debugger",
    //    options,
    //    Box::new(|cc| {
    //        // This gives us image support:
    //        egui_extras::install_image_loaders(&cc.egui_ctx);
    //        let disassembly = asmnes::disassemble(&(0..=u16::MAX).map(|a| state.read(a, true)).collect::<Vec<u8>>()).0
    //            .iter().map(|(a, i)| (*a as u16, i.clone())).collect();
    //        Ok(Box::new(MyApp {
    //            state,
    //            running: false,
    //            speed: 1,
    //            scroll: 0xC000,
    //            following_pc: true,
    //            disassembly,
    //        }))
    //    }),
    //)
}

