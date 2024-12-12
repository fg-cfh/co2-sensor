use super::api::Driver;
use super::api::{usb::*, DriverStateHolder};
use super::osc_nrf::{DontCare, NrfHighAccOscToken, NrfHighAccOscillatorDriver};
use super::resources_nrf::NrfDriverResources;
use super::OscillatorDriver;
use di::singleton::Singleton;
use di::{Initialized, WithDependency};
use nrf52840_hal::clocks::{Clocks, ExternalOscillator};
use nrf52840_hal::pac::USBD;
use nrf52840_hal::usbd::{UsbPeripheral, Usbd};
use static_cell::StaticCell;

pub struct NrfUsbState {
    bus: Usbd<UsbPeripheral<'static>>,
}

impl WithDependency<USBD> for NrfUsbState {
    fn with_dependency<Result, F: FnOnce(USBD) -> Result>(f: F) -> Result {
        NrfDriverResources::with_ref_mut(|resources| f(resources.usbd.take().unwrap()))
    }
}

static CLOCKS: StaticCell<Clocks<ExternalOscillator, DontCare, DontCare>> = StaticCell::new();

impl Default for NrfUsbState {
    fn default() -> Self {
        NrfHighAccOscillatorDriver.ensure_is_initialized();
        let token = NrfHighAccOscillatorDriver::request().unwrap();
        let NrfHighAccOscToken(clocks) = token.acquire();
        let clocks = CLOCKS.init(clocks);
        let bus = Self::with_dependency(|usbd| Usbd::new(UsbPeripheral::new(usbd, clocks)));
        Self { bus }
    }
}

struct NrfUsbDriverState;

impl Singleton for NrfUsbDriverState {
    type Content = NrfUsbState;

    fn with_state_holder<Result, F>(f: F) -> Result
    where
        F: FnOnce(&DriverStateHolder<Self::Content>) -> Result,
    {
        static DRIVER_STATE: DriverStateHolder<NrfUsbState> = DriverStateHolder::new();
        f(&DRIVER_STATE)
    }
}

struct NrfUsbDriver;

impl NrfUsbDriver {
    const fn new() -> NrfUsbDriver {
        NrfUsbDriver
    }
}

impl Initialized for NrfUsbDriver {
    fn init(&self) {
        NrfUsbDriverState.init();
    }

    fn is_initialized(&self) -> bool {
        NrfUsbDriverState.is_initialized()
    }
}

impl<'a> UsbDriver<Usbd<UsbPeripheral<'a>>> for NrfUsbDriver {
    fn bus(&self) -> &'static Usbd<UsbPeripheral<'a>> {
        NrfUsbDriverState::with_ref(|usb_state| *usb_state.bus)
    }
}

impl Driver for NrfUsbDriver {}
