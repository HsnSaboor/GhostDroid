//! `wd-gpui` binary: thin entry, delegates to [`wd_gpui::app::run`].

fn main() {
    let _ = wd_core::init_tracing("info");
    tracing::info!("wd-gpui start");
    wd_gpui::app::run();
}
