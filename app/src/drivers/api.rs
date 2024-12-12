pub mod gpio;
pub mod log;
pub mod mono;
pub mod osc;
pub mod power;
pub mod rng;
pub mod soc;
pub mod usb;

use core::error::Error;
use core::fmt;
use core::fmt::Debug;
use core::sync::atomic::{AtomicBool, Ordering};
use di::singleton::SingletonHolderImpl;
use di::Initialized;

#[derive(Debug, Clone, Copy)]
pub struct ApiError(&'static str);
impl Error for ApiError {}
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiError: {}.", self.0)
    }
}
pub(super) static TIMEOUT: ApiError = ApiError("timeout");

pub trait Driver: Initialized {}

pub type DriverStateHolder<State> = SingletonHolderImpl<State>;

pub struct InitState(AtomicBool);

impl Initialized for InitState {
    fn init(&self) {
        self.0.store(true, Ordering::Release)
    }

    fn is_initialized(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Default)]
pub struct StatelessDriver;

impl StatelessDriver {
    fn init_state() -> &'static InitState {
        static STATE: InitState = InitState(AtomicBool::new(false));
        &STATE
    }
}

impl Initialized for StatelessDriver {
    fn init(&self) {
        Self::init_state().ensure_is_initialized();
    }

    fn is_initialized(&self) -> bool {
        Self::init_state().is_initialized()
    }
}
