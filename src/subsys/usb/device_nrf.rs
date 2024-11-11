use super::{
    class_cdc_ncm_eth::CdcNcmEthClass, SubsysUsbClass, SubsysUsbDevice, SubsysUsbDeviceFactory,
};
use crate::drivers::api::usb::UsbDriver;
use core::cell::RefCell;
use critical_section::{with as with_cs, Mutex};
use defmt;
use nrf52840_hal::usbd::*;
use usb_device::device::*;

const MANUFACTURER: &str = "CfH";
const PRODUCT: &str = "CO2 Gateway";
const SERIAL_NUMBER: &str = "00001";
const VENDOR_ID: u16 = 0x1209;
const PRODUCT_ID: u16 = 0x0004;

static USB_DEVICE: Mutex<RefCell<Option<NrfUsbDeviceState<'static>>>> =
    Mutex::new(RefCell::new(None));

pub type HalUsbBus<'a> = Usbd<UsbPeripheral<'a>>;

struct NrfUsbDeviceState<'a> {
    usb_dev: UsbDevice<'a, HalUsbBus<'a>>,
    class: &'a mut CdcNcmEthClass,
}

pub struct NrfUsbDevice;

impl SubsysUsbDeviceFactory<HalUsbBus<'static>, CdcNcmEthClass> for NrfUsbDevice {
    fn new(
        usb_driver: &'static dyn UsbDriver<HalUsbBus<'static>>,
        class: &'static mut CdcNcmEthClass,
    ) -> Self {
        with_cs(|cs| {
            USB_DEVICE.replace_with(cs, |prev| {
                if prev.is_some() {
                    // TODO: support multiple instances
                    defmt::panic!("single instance")
                }
                let usb_dev =
                    UsbDeviceBuilder::new(usb_driver.usb_alloc(), UsbVidPid(VENDOR_ID, PRODUCT_ID))
                        .device_class(usbd_ethernet::USB_CLASS_CDC)
                        .strings(&[StringDescriptors::default()
                            .manufacturer(MANUFACTURER)
                            .product(PRODUCT)
                            .serial_number(SERIAL_NUMBER)])
                        .unwrap()
                        .build();
                Some(NrfUsbDeviceState { usb_dev, class })
            });
        });
        NrfUsbDevice
    }
}
impl SubsysUsbDevice<HalUsbBus<'static>, CdcNcmEthClass> for NrfUsbDevice {
    fn poll(&self, f: &mut dyn FnMut(&mut CdcNcmEthClass)) {
        with_cs(|cs| {
            if let Some(dev_state) = USB_DEVICE.borrow_ref_mut(cs).as_mut() {
                if dev_state.usb_dev.poll(&mut [dev_state.class.usb_class()]) {
                    dev_state.class.handle_signal();
                    f(dev_state.class);
                };
            } else {
                unreachable!()
            }
        });
    }
}
