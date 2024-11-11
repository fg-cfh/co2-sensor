use super::api::rng::*;
use super::api::{self, Driver};
use super::resources_nrf::NrfRngDriverResources;
use nrf52840_hal::Rng;
use rand_core::CryptoRng;

pub struct NrfRngDriver {
    _rng: Rng,
}

impl api::Driver<NrfRngDriverResources> for NrfRngDriver {
    fn new(resources: NrfRngDriverResources, _: ()) -> NrfRngDriver {
        Self {
            _rng: Rng::new(resources.rng),
        }
    }
}

impl_rng_core!(NrfRngDriver);

impl CryptoRng for NrfRngDriver {}

impl RngDriver for NrfRngDriver {
    fn random(&mut self, buf: &mut [u8]) {
        self._rng.random(buf);
    }
}

pub fn init(resources: NrfRngDriverResources) -> Result<NrfRngDriver, ()> {
    Ok(NrfRngDriver::new(resources, ()))
}
