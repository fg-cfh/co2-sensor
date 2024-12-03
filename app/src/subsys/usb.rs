use crate::drivers::api::usb::UsbDriver;
pub use device_nrf::HalUsbBus;
use usb_device::class_prelude::{UsbBus, UsbClass};

pub mod class_cdc_ncm_eth;
pub mod device_nrf;

pub const NUM_CLASSES: usize = 1;

pub trait SubsysUsbClassFactory<B: UsbBus> {
    fn new<'a>(usb_driver: &'static dyn UsbDriver<B>) -> &'a mut Self;
}

pub trait SubsysUsbClass<B: UsbBus>: Send {
    fn handle_signal(&mut self);
    fn usb_class(&mut self) -> &mut dyn UsbClass<B>;
}

pub trait SubsysUsbDeviceFactory<B: UsbBus, C: SubsysUsbClass<B>> {
    fn new(usb_driver: &'static dyn UsbDriver<B>, class: &'static mut C) -> Self;
}

pub trait SubsysUsbDevice<B: UsbBus, C: SubsysUsbClass<B>> {
    fn poll(&self, f: &mut dyn FnMut(&mut C));
}
