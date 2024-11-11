use embedded_hal::digital::OutputPin;

pub enum GpioOutputPin {
    LED,
}

pub trait GpioDriver<E> {
    fn output_pin(&mut self, name: GpioOutputPin) -> &mut dyn OutputPin<Error = E>;
}
