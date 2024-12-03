use super::api::{gpio::*, Driver};
use super::resources_nrf::NrfDriverResources;
use core::convert::Infallible;
use di::{Singleton, SingletonHolder, SingletonHolderImpl};
use embedded_hal::digital::OutputPin;
use nrf52840_hal::{
    gpio::{self, Level, Output, Pin, PushPull},
    pac::P0,
};

pub struct NrfGpioDriverState {
    led: Pin<Output<PushPull>>,
}

impl Singleton<NrfGpioDriverState, P0> for NrfGpioDriverState {
    fn with_state_holder<F, R>(f: F) -> R {
        static DRIVER_STATE: SingletonHolderImpl<NrfGpioDriverState> = Default::default();
        f(&mut DRIVER_STATE)
    }

    fn with_dependencies<F>(f: F) {
        NrfDriverResources::with_state(|resources| f(resources.p0.take().unwrap()));
    }

    fn from(p0: P0) -> NrfGpioDriverState {
        let port0 = gpio::p0::Parts::new(p0);
        Self {
            led: port0.p0_13.into_push_pull_output(Level::Low).degrade(),
        }
    }
}

struct NrfGpioDriver;

impl Driver for NrfGpioDriver {
    fn ensure_is_initialized() {
        NrfGpioDriverState::ensure_state();
    }
}

impl GpioDriver for NrfGpioDriver {
    type GpioError = Infallible;

    fn with_output_pin<F>(name: GpioOutputPin, f: F)
    where
        F: FnOnce(&mut dyn OutputPin<Error = Self::GpioError>),
    {
        NrfGpioDriverState::with_state(|state| match name {
            GpioOutputPin::LED => &mut state.led,
        })
    }
}
