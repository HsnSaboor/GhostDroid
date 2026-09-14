//! wd-shell bin: thin.
fn main() {
    wd_core::init_logging();
    tracing::info!("wd-shell: start (Slint 1.17 winit-skia)");
    println!("wd-shell {}", wd_core::STACK_YEAR);
}
