#![feature(let_chains)]
use std::{env, error::Error};

mod app;

pub fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let ines = shared::Ines::from_file("rom.nes")?;
    // Load file provided otherwise load default
    // TODO make a nicer interface
    let ines = if let Some(file) = env::args().skip(1).next() {
        asmnes::assemble_from_file(&file)?
    } else {
        shared::Ines::from_file("rom.nes")?
    };
    //println!("inesprg: {:?}, ineschr: {:?}, inesmap: {:?}, banks: {:?}", ines.inesprg, ines.ineschr, ines.mapper, ines.banks);
    //println!("disassembly:");
    //let (s, _l) = asmnes::disassemble(&ines.banks);
    //s.iter().for_each(|i| println!("{i}"));
    let state = remun::State::new(ines);
    app::run(state)?;
    Ok(())
}
