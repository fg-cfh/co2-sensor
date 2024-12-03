use di::{Singleton, SingletonHolder, SingletonHolderImpl};
pub use hal::pac::*;
use nrf52840_hal as hal;

pub struct NrfDriverResources {
    pub power: Option<POWER>,
    pub clock: Option<CLOCK>,
    pub rng: Option<RNG>,
    pub p0: Option<P0>,
    pub rtc0: Option<RTC0>,
    pub usbd: Option<USBD>,
}

impl Singleton<NrfDriverResources, Peripherals> for NrfDriverResources {
    fn with_state_holder<F>(f: F) {
        static STATE: SingletonHolderImpl<NrfDriverResources> = Default::default();
        STATE.with_state(|state| f(&mut state));
    }

    fn with_dependencies<F>(f: F) {
        // SAFETY: The driver framework ensures that this is the unique
        // canonical provider for peripherals. It is assumed that the OS will
        // consume peripherals safely.
        f(unsafe { Peripherals::steal() })
    }

    fn from(peripherals: Peripherals) -> NrfDriverResources {
        NrfDriverResources {
            power: Some(peripherals.POWER),
            clock: Some(peripherals.CLOCK),
            rng: Some(peripherals.RNG),
            p0: Some(peripherals.P0),
            rtc0: Some(peripherals.RTC0),
            usbd: Some(peripherals.USBD),
        }
    }
}
