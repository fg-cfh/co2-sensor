use super::api::usb::*;
use super::api::{self, Driver};
use super::osc_nrf::{
    DontCare, NrfHfOscillatorDriver, NrfLfOscillatorDriver, NrfOscillatorsDriver,
};
use super::resources_nrf::NrfUsbDriverResources;
use super::{OscillatorsDriver, RequestOsc};
use nrf52840_hal::clocks::{Clocks, ExternalOscillator};
use nrf52840_hal::usbd::{UsbPeripheral, Usbd};
use static_cell::StaticCell;
use usb_device::class_prelude::UsbBusAllocator;

static USB_ALLOC: StaticCell<UsbBusAllocator<Usbd<UsbPeripheral<'static>>>> = StaticCell::new();
static USB_DRIVER: StaticCell<NrfUsbDriver> = StaticCell::new();

pub struct NrfUsbDriver {
    usb_alloc: &'static UsbBusAllocator<Usbd<UsbPeripheral<'static>>>,
}

impl UsbDriver<Usbd<UsbPeripheral<'static>>> for NrfUsbDriver {
    fn usb_alloc(&self) -> &UsbBusAllocator<Usbd<UsbPeripheral<'static>>> {
        self.usb_alloc
    }
}

static CLOCKS: StaticCell<Clocks<ExternalOscillator, DontCare, DontCare>> = StaticCell::new();

impl
    api::Driver<
        NrfUsbDriverResources,
        &'static NrfOscillatorsDriver<NrfLfOscillatorDriver, NrfHfOscillatorDriver>,
    > for NrfUsbDriver
{
    fn get(
        resources: NrfUsbDriverResources,
        osc_driver: &'static NrfOscillatorsDriver<NrfLfOscillatorDriver, NrfHfOscillatorDriver>,
    ) -> Self {
        let clocks = CLOCKS.init(osc_driver.oscillator().high_acc_hf_osc.request().unwrap());
        let usb_peripheral = UsbPeripheral::new(resources.usbd, clocks);
        let usb_alloc = USB_ALLOC.init(UsbBusAllocator::new(Usbd::new(usb_peripheral)));
        Self { usb_alloc }
    }
}

pub fn init(
    resources: NrfUsbDriverResources,
    osc_driver: &'static NrfOscillatorsDriver<NrfLfOscillatorDriver, NrfHfOscillatorDriver>,
) -> Result<&mut NrfUsbDriver, api::ApiError> {
    USB_DRIVER
        .try_init(NrfUsbDriver::get(resources, osc_driver))
        .ok_or(api::DRIVER_ALREADY_INITIALIZED)
}
