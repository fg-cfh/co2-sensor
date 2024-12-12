use di::{
    singleton::{Singleton, SingletonHolderImpl},
    WithDependency,
};
use hal::pac::*;
use nrf52840_hal as hal;

struct NrfResources {
    pub power: Option<POWER>,
    pub clock: Option<CLOCK>,
    pub rng: Option<RNG>,
    pub p0: Option<P0>,
    pub rtc0: Option<RTC0>,
    pub usbd: Option<USBD>,
}

impl WithDependency<Peripherals> for NrfResources {
    fn with_dependency<Result, F: FnOnce(Peripherals) -> Result>(f: F) -> Result {
        // SAFETY: The driver framework ensures that this is the unique
        // canonical provider for peripherals. It is assumed that the OS will
        // consume peripherals safely.
        f(unsafe { Peripherals::steal() })
    }
}

impl Default for NrfResources {
    fn default() -> Self {
        Self::with_dependency(|peripherals| NrfResources {
            power: Some(peripherals.POWER),
            clock: Some(peripherals.CLOCK),
            rng: Some(peripherals.RNG),
            p0: Some(peripherals.P0),
            rtc0: Some(peripherals.RTC0),
            usbd: Some(peripherals.USBD),
        })
    }
}

pub struct NrfDriverResources;

impl Singleton for NrfDriverResources {
    type Content = NrfResources;

    fn with_state_holder<R, F>(f: F) -> R
    where
        F: FnOnce(&'static SingletonHolderImpl<NrfResources>) -> R,
    {
        static RESOURCES_HOLDER: SingletonHolderImpl<NrfResources> = SingletonHolderImpl::new();
        f(&RESOURCES_HOLDER)
    }
}
