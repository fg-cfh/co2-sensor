use super::api::rng::*;
use super::api::Driver;
use super::api::DriverStateHolder;
use super::resources_nrf::NrfDriverResources;
use di::singleton::Singleton;
use di::Initialized;
use di::WithDependency;
use nrf52840_hal::{pac::RNG, Rng};
use rand_core::CryptoRng;

pub struct NrfRngState {
    rng: Rng,
}

impl WithDependency<RNG> for NrfRngState {
    fn with_dependency<Result, F: FnOnce(RNG) -> Result>(f: F) -> Result {
        NrfDriverResources::with_ref_mut(|resources| f(resources.rng.take().unwrap()))
    }
}

impl Default for NrfRngState {
    fn default() -> NrfRngState {
        Self::with_dependency(|rng| Self { rng: Rng::new(rng) })
    }
}

struct NrfRngDriverState;

impl Singleton for NrfRngDriverState {
    type Content = NrfRngState;

    fn with_state_holder<Result, F>(f: F) -> Result
    where
        F: FnOnce(&DriverStateHolder<Self::Content>) -> Result,
    {
        static DRIVER_STATE: DriverStateHolder<NrfRngState> = DriverStateHolder::new();
        f(&DRIVER_STATE)
    }
}

struct NrfRngDriver;

impl NrfRngDriver {
    const fn new() -> NrfRngDriver {
        NrfRngDriver
    }
}

impl Initialized for NrfRngDriver {
    fn init(&self) {
        NrfRngDriverState.init();
    }

    fn is_initialized(&self) -> bool {
        NrfRngDriverState.is_initialized()
    }
}

impl CryptoRng for NrfRngDriver {}

impl Driver for NrfRngDriver {}

impl RngDriver for NrfRngDriver {
    fn next(&self, dest: &mut [u8]) {
        NrfRngDriverState::with_ref_mut(|state| state.rng.random(dest))
    }
}

impl_rng_driver_rng_core!(NrfRngDriver);
