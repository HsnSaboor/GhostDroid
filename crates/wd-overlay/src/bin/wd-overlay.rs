//! wd-overlay bin: thin egui glow editor. Logic lives in wd-input/wd-shell.
fn main() {
    wd_core::init_logging();
    tracing::info!("wd-overlay: start (eframe glow)");
    println!("wd-overlay {}", wd_core::STACK_YEAR);
}
