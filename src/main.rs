use std::error::Error;

mod app;

pub fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let ines = shared::Ines::from_file("rom.nes")?;
    println!("inesprg: {:?}, ineschr: {:?}, inesmap: {:?}, banks: {:?}", ines.inesprg, ines.ineschr, ines.mapper, ines.banks);
    println!("disassembly:");
    let (s, _l) = asmnes::disassemble(&ines.banks);
    s.iter().for_each(|i| println!("{i}"));
    let state = remun::State::new(ines);
    app::run(state)?;
    Ok(())
}
