use super::api::power::PowerDriver;
use super::api::{Driver, StatelessDriver};
use super::resources_nrf::NrfDriverResources;
use di::singleton::Singleton;
use di::{Initialized, WithDependency};
use nrf52840_hal::pac::POWER;

struct NrfPowerDriver(StatelessDriver);

impl WithDependency<POWER> for NrfPowerDriver {
    fn with_dependency<Result, F: FnOnce(POWER) -> Result>(f: F) -> Result
    where
        Self: Sized,
    {
        NrfDriverResources::with_ref_mut(|resources| f(resources.power.take().unwrap()))
    }
}

impl Initialized for NrfPowerDriver {
    fn init(&self) {
        self.0.ensure_is_initialized();
        Self::with_dependency(|power| {
            // Enable the DC/DC converter
            power.dcdcen.write(|w| w.dcdcen().enabled());
        });
    }

    fn is_initialized(&self) -> bool {
        self.0.is_initialized()
    }
}

impl Driver for NrfPowerDriver {}

impl PowerDriver for NrfPowerDriver {}
