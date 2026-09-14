//! `wd-daemon` bin: thin. Parse args, call lib, log via tracing.
use wd_daemon::{socket_path, supervised_children};

fn main() {
    wd_core::init_logging();
    tracing::info!(socket = %socket_path(), "wd-daemon: start");
    for child in supervised_children() {
        tracing::info!(name = %child.name, line = %child.spawn_line(), "wd-daemon: child");
    }
    tracing::info!("wd-daemon: stub serve, no listen yet (plan 02 scope)");
}
