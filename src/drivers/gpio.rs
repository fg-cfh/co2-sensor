use super::resources::GpioDriverResources;
pub use embedded_hal::digital::OutputPin;
use nrf52840_hal::{
    self as hal,
    gpio::{Level, Output, Pin, PushPull},
};

pub struct Api {
    pub led: Pin<Output<PushPull>>,
}

pub fn init(resources: GpioDriverResources) -> Api {
    // Setup LED
    let port0 = hal::gpio::p0::Parts::new(resources.p0);
    Api {
        led: port0.p0_13.into_push_pull_output(Level::Low).degrade(),
    }
}
