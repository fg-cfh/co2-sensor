pub use hal::pac;
use nrf52840_hal as hal;

pub struct NrfOscDriverResources {
    pub clock: pac::CLOCK,
}

pub struct NrfPowerDriverResources {
    pub power: pac::POWER,
}

pub struct NrfGpioDriverResources {
    pub p0: pac::P0,
}

pub struct NrfRticMonoDriverResources {
    pub rtc0: pac::RTC0,
}

pub struct NrfUsbDriverResources {
    pub usbd: pac::USBD,
}

pub struct NrfRngDriverResources {
    pub rng: pac::RNG,
}

pub struct NrfDriverResources {
    pub power: NrfPowerDriverResources,
    pub osc: NrfOscDriverResources,
    pub rng: NrfRngDriverResources,
    pub gpio: NrfGpioDriverResources,
    pub monotonic: NrfRticMonoDriverResources,
    pub usb: NrfUsbDriverResources,
}

/// Transfer ownership of peripherals to specific drivers.
pub fn transfer(peripherals: pac::Peripherals) -> NrfDriverResources {
    let osc = NrfOscDriverResources {
        clock: peripherals.CLOCK,
    };

    let power = NrfPowerDriverResources {
        power: peripherals.POWER,
    };

    let gpio = NrfGpioDriverResources { p0: peripherals.P0 };

    let monotonic = NrfRticMonoDriverResources {
        rtc0: peripherals.RTC0,
    };

    let usb = NrfUsbDriverResources {
        usbd: peripherals.USBD,
    };

    let rng = NrfRngDriverResources {
        rng: peripherals.RNG,
    };

    NrfDriverResources {
        osc,
        rng,
        power,
        gpio,
        monotonic,
        usb,
    }
}
