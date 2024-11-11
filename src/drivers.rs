mod resources;

pub mod clocks;
pub mod gpio;
pub mod log;
pub mod monotonic;
pub mod power;
pub mod soc;

pub use resources::pac;

pub struct Drivers {
    pub clocks: clocks::Api,
    pub monotonic: monotonic::Api,
    pub gpio: gpio::Api,
}

pub fn init(peripherals: pac::Peripherals) -> Drivers {
    let resources = resources::from(peripherals);

    soc::init();
    log::init();
    power::init(resources.power);
    let clocks = clocks::init(resources.clocks);
    let (monotonic, clocks) = monotonic::init(resources.monotonic, clocks);
    let gpio = gpio::init(resources.gpio);

    Drivers {
        clocks,
        monotonic,
        gpio,
    }
}
