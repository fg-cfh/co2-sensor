use usb_device::bus::UsbBus;
use usb_device::class_prelude::UsbBusAllocator;

pub trait UsbDriver<B: UsbBus> {
    fn usb_alloc(&'static self) -> &'static UsbBusAllocator<B>;
}
