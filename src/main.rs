mod app;

pub fn main() -> eframe::Result {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    //app::run(state);
    Ok(())
}
