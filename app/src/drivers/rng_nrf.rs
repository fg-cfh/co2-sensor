use super::api::rng::*;
use super::api::{self, Driver};
use super::resources_nrf::DriverResources;
use core::cell::RefCell;
use critical_section::Mutex;
use di::{InitializedSingleton, RefProvider, Singleton};
use nrf52840_hal::Rng;
use rand_core::CryptoRng;

static DRIVER: Mutex<Option<NrfRngDriver>> = Mutex::new(None);

pub struct NrfRngDriver {
    rng: RefCell<Rng>,
}

impl InitializedSingleton for NrfRngDriver {
    fn init() {
        let prev = DRIVER.get_mut();
        if prev.is_none() {
            prev.replace(Some(NrfRngDriver {
                rng: RefCell::new(DriverResources::get().take()),
            }));
        }
    }
}

impl RefProvider<'static, Self> for NrfRngDriver {
    fn get() -> &'static Self {
        &DRIVER.get_mut().expect("not initialized")
    }
}
impl Singleton<Self> for NrfRngDriver {}
impl api::Driver<Self> for NrfRngDriver {}

impl CryptoRng for NrfRngDriver {}

impl RngDriver for NrfRngDriver {
    fn random(&self, buf: &mut [u8]) {
        self.rng.borrow_mut().random(buf);
    }
}
