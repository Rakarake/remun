#![feature(let_chains)]
use std::{env, error::Error, path::Path};

use remun;

mod app;

pub fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let ines = remun::load_from_file(env::args().nth(1).ok_or("please give file as argument")?)?;
    let state = remun::State::new(ines);
    app::run(state);
    Ok(())
}
