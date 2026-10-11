use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

/// The flag itself is what `signal_hook::flag::register` writes, so it has to be
/// an `Arc<AtomicBool>` rather than a plain static.
static TRIGGERED: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));

/// True once SIGTERM or SIGINT has been received.
pub fn triggered() -> bool {
    TRIGGERED.load(Ordering::Relaxed)
}

/// Register SIGTERM and SIGINT.
///
/// `signal_hook::flag::register` ADDS a handler rather than replacing one, so
/// this composes with Actix's own signal handling instead of racing it. Both
/// signals write the same flag, which every background task polls.
pub fn install() {
    use signal_hook::consts::signal::{SIGINT, SIGTERM};

    for (sig, name) in [(SIGTERM, "SIGTERM"), (SIGINT, "SIGINT")] {
        if let Err(err) = signal_hook::flag::register(sig, TRIGGERED.clone()) {
            // Falling back to the default action terminates the process at once
            // rather than hanging, so this is safe, only not the fast exit.
            log::error!("shutdown: {name} handler not installed: {err}");
        }
    }
}
