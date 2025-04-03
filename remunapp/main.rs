#![feature(let_chains)]
use remun::State;
use std::{env, error::Error, path::Path};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod visualizer;

pub fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let ines = remun::load_from_file(env::args().nth(1).ok_or("please give file as argument")?)?;
    let state = remun::State::new(ines);
    Ok(())
}

fn create_window(event_loop: &EventLoop<()>) -> (Window, PhysicalSize<u32>) {
    let size = PhysicalSize::new(400, 400);
    (
        WindowBuilder::new()
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap(),
        size,
    )
}
