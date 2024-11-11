use super::api::gpio::*;
use super::api::{self, Driver};
use super::resources_nrf::NrfGpioDriverResources;
use core::convert::Infallible;
use embedded_hal::digital::OutputPin;
use nrf52840_hal::{
    self as hal,
    gpio::{Level, Output, Pin, PushPull},
};

pub struct NrfGpioDriver {
    _led: Pin<Output<PushPull>>,
}

impl api::Driver<NrfGpioDriverResources> for NrfGpioDriver {
    fn new(resources: NrfGpioDriverResources, _: ()) -> NrfGpioDriver {
        let port0 = hal::gpio::p0::Parts::new(resources.p0);
        Self {
            _led: port0.p0_13.into_push_pull_output(Level::Low).degrade(),
        }
    }
}

impl GpioDriver<Infallible> for NrfGpioDriver {
    fn output_pin(&mut self, name: GpioOutputPin) -> &mut dyn OutputPin<Error = Infallible> {
        match name {
            GpioOutputPin::LED => &mut self._led,
        }
    }
}

pub fn init(resources: NrfGpioDriverResources) -> Result<NrfGpioDriver, ()> {
    Ok(NrfGpioDriver::new(resources, ()))
}
