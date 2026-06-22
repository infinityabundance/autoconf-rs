//! Signal handling for autoconf-rs — CROSS.040 resolution.
//!
//! Registers SIGPIPE and SIGINT handlers to enable graceful shutdown
//! instead of default Rust panic-on-signal behavior.
//! Uses raw libc for signal registration (minimal unsafe surface).

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag indicating SIGPIPE was received.
static SIGPIPE_RECEIVED: AtomicBool = AtomicBool::new(false);

/// Global flag indicating SIGINT was received.
static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);

/// Register POSIX signal handlers. Called once at startup.
/// Safe because handlers only set atomic flags.
pub fn register_signal_handlers() {
    unsafe {
        libc::signal(
            libc::SIGPIPE,
            sigpipe_handler as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGINT,
            sigint_handler as *const () as libc::sighandler_t,
        );
    }
}

/// Check if SIGPIPE was received since last clear.
pub fn sigpipe_received() -> bool {
    SIGPIPE_RECEIVED.swap(false, Ordering::Relaxed)
}

/// Check if SIGINT was received.
pub fn sigint_received() -> bool {
    SIGINT_RECEIVED.load(Ordering::Relaxed)
}

/// Reset all signal flags.
pub fn clear_signals() {
    SIGPIPE_RECEIVED.store(false, Ordering::Relaxed);
    SIGINT_RECEIVED.store(false, Ordering::Relaxed);
}

extern "C" fn sigpipe_handler(_sig: i32) {
    SIGPIPE_RECEIVED.store(true, Ordering::Relaxed);
}

extern "C" fn sigint_handler(_sig: i32) {
    SIGINT_RECEIVED.store(true, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signals_initially_clear() {
        clear_signals();
        assert!(!sigpipe_received());
        assert!(!sigint_received());
    }

    #[test]
    fn test_register_handlers_no_panic() {
        register_signal_handlers();
    }

    #[test]
    fn test_clear_signals() {
        SIGPIPE_RECEIVED.store(true, Ordering::Relaxed);
        clear_signals();
        assert!(!sigpipe_received());
    }
}
