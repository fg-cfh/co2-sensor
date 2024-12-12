use super::api::{gpio::*, Driver, DriverStateHolder};
use super::resources_nrf::NrfDriverResources;
use core::convert::Infallible;
use di::singleton::Singleton;
use di::{Initialized, WithDependency};
use embedded_hal::digital::OutputPin;
use nrf52840_hal::{
    gpio::{self, Level, Output, Pin, PushPull},
    pac::P0,
};

pub struct NrfGpioState {
    led: Pin<Output<PushPull>>,
}

impl WithDependency<P0> for NrfGpioState {
    fn with_dependency<Result, F: FnOnce(P0) -> Result>(f: F) -> Result {
        NrfDriverResources::with_ref_mut(|resources| f(resources.p0.take().unwrap()))
    }
}

impl Default for NrfGpioState {
    fn default() -> Self {
        Self::with_dependency(|p0| {
            let port0 = gpio::p0::Parts::new(p0);
            Self {
                led: port0.p0_13.into_push_pull_output(Level::Low).degrade(),
            }
        })
    }
}

struct NrfGpioDriverState;

impl Singleton for NrfGpioDriverState {
    type Content = NrfGpioState;

    fn with_state_holder<Result, F>(f: F) -> Result
    where
        F: FnOnce(&DriverStateHolder<Self::Content>) -> Result,
    {
        static DRIVER_STATE: DriverStateHolder<NrfGpioState> = DriverStateHolder::new();
        f(&DRIVER_STATE)
    }
}

struct NrfGpioDriver;

impl NrfGpioDriver {
    const fn new() -> NrfGpioDriver {
        NrfGpioDriver
    }
}

impl Initialized for NrfGpioDriver {
    fn init(&self) {
        NrfGpioDriverState.init();
    }

    fn is_initialized(&self) -> bool {
        NrfGpioDriverState.is_initialized()
    }
}

impl GpioDriver for NrfGpioDriver {
    type GpioError = Infallible;

    fn with_output_pin<F>(name: GpioOutputPin, f: F)
    where
        F: FnOnce(&dyn OutputPin<Error = Self::GpioError>),
    {
        NrfGpioDriverState::with_ref(|state| {
            let pin = match name {
                GpioOutputPin::LED => &state.led,
            };
            f(pin);
        });
    }

    fn with_output_pin_mut<F>(name: GpioOutputPin, f: F)
    where
        F: FnOnce(&mut dyn OutputPin<Error = Self::GpioError>),
    {
        NrfGpioDriverState::with_ref_mut(|state| {
            let pin = match name {
                GpioOutputPin::LED => &mut state.led,
            };
            f(pin);
        });
    }
}

impl Driver for NrfGpioDriver {}
