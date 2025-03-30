#![feature(let_chains)]
use std::{env, error::Error, path::Path};

mod app;

pub fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let ines = if let Some(file) = env::args().nth(1) {
        let p = Path::new(&file);
        if let Some(os_str) = p.extension() {
            match os_str.to_str() {
                Some("nes") => { shared::Ines::from_file(&file)? }
                Some("asm") => { asmnes::assemble(&file)? }
                _ => { panic!("wrong extension or werid format") }
            }
        } else {
            panic!("file missing extension")
        }

    } else {
        panic!("need file to run");
    };
    let state = remun::State::new(ines);
    app::run(state)?;
    Ok(())
}
