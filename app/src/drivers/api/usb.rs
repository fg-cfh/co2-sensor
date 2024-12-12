use usb_device::bus::UsbBus;

pub trait UsbDriver<B: UsbBus>: Driver {
    fn bus(&self) -> &'static B;
}
