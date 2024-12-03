pub mod api;

mod resources_nrf;

mod gpio_nrf;
mod log_defmt_rtt;
mod mono_nrf_rtic;
mod osc_nrf;
mod power_nrf;
mod rng_nrf;
mod soc_cortex_m;
mod usb_nrf;

use api::osc::*;
use gpio_nrf::NrfGpioDriverState;
use mono_nrf_rtic::NrfRticMonoDriver;
pub use mono_nrf_rtic::{Duration, Instant};
use osc_nrf::{NrfHfOscillatorDriver, NrfLfOscillatorDriver, NrfOscillatorsDriver};
pub use resources_nrf::pac;
use rng_nrf::NrfRngDriver;
use usb_nrf::NrfUsbDriver;

pub struct Drivers<'a> {
    pub osc: &'a NrfOscillatorsDriver<NrfLfOscillatorDriver, NrfHfOscillatorDriver>,
    pub rng: NrfRngDriver,
    pub mono: NrfRticMonoDriver,
    pub gpio: NrfGpioDriverState,
    pub usb: &'a mut NrfUsbDriver,
}

pub type MonoDriver = NrfRticMonoDriver;
pub type GpioDriver = NrfGpioDriverState;
pub type UsbDriver = NrfUsbDriver;

/// Instantiate drivers that take ownership of the peripherals.
pub fn init<'a>(peripherals: pac::Peripherals) -> Drivers<'a> {
    let resources = resources_nrf::init(peripherals);

    soc_cortex_m::init().unwrap();
    log_defmt_rtt::init().unwrap();
    power_nrf::init(resources.power).unwrap();
    let osc = osc_nrf::init(resources.osc).unwrap();
    let rng = rng_nrf::init(resources.rng).unwrap();
    let mono = mono_nrf_rtic::init(resources.monotonic, osc).unwrap();
    let gpio = gpio_nrf::init(resources.gpio).unwrap();
    let usb = usb_nrf::init(resources.usb, osc).unwrap();

    Drivers {
        osc,
        rng,
        mono,
        gpio,
        usb,
    }
}
