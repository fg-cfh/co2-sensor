pub use hal::pac;
use nrf52840_hal as hal;

pub struct ClocksDriverResources {
    pub clock: pac::CLOCK,
}

pub struct PowerDriverResources {
    pub power: pac::POWER,
}

pub struct GpioDriverResources {
    pub p0: pac::P0,
}

pub struct MonotonicDriverResources {
    pub rtc0: pac::RTC0,
}

pub struct Resources {
    pub power: PowerDriverResources,
    pub clocks: ClocksDriverResources,
    pub gpio: GpioDriverResources,
    pub monotonic: MonotonicDriverResources,
}

pub fn from(peripherals: pac::Peripherals) -> Resources {
    let clocks = ClocksDriverResources {
        clock: peripherals.CLOCK,
    };

    let power = PowerDriverResources {
        power: peripherals.POWER,
    };

    let gpio = GpioDriverResources { p0: peripherals.P0 };

    let monotonic = MonotonicDriverResources {
        rtc0: peripherals.RTC0,
    };

    Resources {
        clocks,
        power,
        gpio,
        monotonic,
    }
}
