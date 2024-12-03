use super::api::{power::PowerDriver, Driver, StatelessDriver};
use super::resources_nrf::NrfDriverResources;
use di::Singleton;
use nrf52840_hal::pac::POWER;

struct NrfPowerDriver;

impl StatelessDriver<POWER> for NrfPowerDriver {
    fn with_dependencies<F>(f: F) {
        NrfDriverResources::with_state(|resources| f(resources.power.take().unwrap()));
    }

    fn init_once(power: POWER) {
        // Enable the DC/DC converter
        power.dcdcen.write(|w| w.dcdcen().enabled());
    }
}

impl Driver for NrfPowerDriver {
    fn ensure_is_initialized() {
        Self::ensure_is_initialized();
    }
}

impl PowerDriver for NrfPowerDriver {}
