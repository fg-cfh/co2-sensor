pub mod gpio;
pub mod log;
pub mod mono;
pub mod osc;
pub mod power;
pub mod rng;
pub mod usb;

use core::fmt;
use core::fmt::Debug;
use core::{error::Error, sync::atomic::AtomicBool};
use di::{Singleton, SingletonHolder};

#[derive(Debug, Clone, Copy)]
pub struct ApiError(&'static str);
impl Error for ApiError {}
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiError: {}.", self.0)
    }
}
pub(super) static TIMEOUT: ApiError = ApiError("timeout");

pub trait Driver {
    fn ensure_is_initialized();
}

struct StatelessDriverInitState(AtomicBool);

impl StatelessDriverInitState {
    pub fn new() -> StatelessDriverInitState {
        StatelessDriverInitState(AtomicBool::new(false))
    }
}

impl SingletonHolder<bool> for StatelessDriverInitState {
    fn init_once<I, F: FnOnce(I) -> bool>(&mut self, dependencies: I, f: F) {
        self.0.init_once(dependencies, f);
    }

    fn with_state<R, F: FnOnce(bool) -> (bool, R)>(&mut self, f: F) -> R {
        self.0.with(f)
    }

    fn is_initialized(&mut self) -> bool {
        self.0.is_initialized()
    }
}

pub trait StatelessDriver<D> {
    fn init_state() -> &'static mut StatelessDriverInitState
    where
        Self: Sized,
    {
        static INIT_STATE: StatelessDriverInitState = StatelessDriverInitState::new();
        &mut INIT_STATE
    }

    fn with_dependencies<F>(f: F)
    where
        F: FnOnce(D),
        Self: Sized;

    fn init_once(dependencies: D)
    where
        Self: Sized;

    fn ensure_is_initialized() {
        Self::init_state().with(|is_initialized| {
            if !is_initialized {
                Self::with_dependencies(|dependencies| {
                    Self::init_once(dependencies);
                });
            }
        })
    }
}
