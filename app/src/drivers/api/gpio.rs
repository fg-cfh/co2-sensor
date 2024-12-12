use super::Driver;
use embedded_hal::digital::{Error, OutputPin};

pub enum GpioOutputPin {
    LED,
}

pub trait GpioDriver: Driver {
    type GpioError: Error;

    fn with_output_pin<F>(name: GpioOutputPin, f: F)
    where
        F: FnOnce(&dyn OutputPin<Error = Self::GpioError>);

    fn with_output_pin_mut<F>(name: GpioOutputPin, f: F)
    where
        F: FnOnce(&mut dyn OutputPin<Error = Self::GpioError>);
}
