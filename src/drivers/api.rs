pub mod gpio;
pub mod mono;
pub mod osc;
pub mod rng;
pub mod usb;

use core::error::Error;
use core::fmt;
use core::fmt::Debug;
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy)]
pub struct ApiError(&'static str);
impl Error for ApiError {}
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiError: {}.", self.0)
    }
}
pub(super) static TIMEOUT: ApiError = ApiError("timeout");
pub(super) static DRIVER_ALREADY_INITIALIZED: ApiError = ApiError("driver already initialized");

pub(super) struct StatelessDriver {
    pub initialized: AtomicBool,
}
impl StatelessDriver {
    pub const fn new() -> Self {
        StatelessDriver {
            initialized: AtomicBool::new(false),
        }
    }

    pub fn init<R, F: FnOnce(R)>(&self, resources: R, f: F) -> Result<(), ApiError> {
        self.initialized
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_or_else(|_| Err(DRIVER_ALREADY_INITIALIZED), |_| Ok(()))
            .inspect(|_| {
                f(resources);
            })
    }
}

/// Drivers are typestate machines and must not implement Copy or Clone.
pub trait Driver<P, D = ()> {
    fn new(peripherals: P, dependencies: D) -> Self;
}
