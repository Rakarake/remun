use std::error::Error;

mod app;

pub fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    //app::run(state);
    let ines = shared::Ines::from_file("rom.nes")?;
    println!("inesprg: {:?}, ineschr: {:?}, inesmap: {:?}, banks: {:?}", ines.inesprg, ines.ineschr, ines.mapper, ines.banks);
    Ok(())
}
