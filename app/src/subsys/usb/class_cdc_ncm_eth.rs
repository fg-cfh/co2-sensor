use super::{HalUsbBus, SubsysUsbClass, SubsysUsbClassFactory};
use crate::drivers::api::usb::UsbDriver;
use defmt;
use static_cell::{ConstStaticCell, StaticCell};
use usb_device::class_prelude::*;
use usbd_ethernet::{DeviceState, Ethernet};

const HOST_MAC_ADDR: [u8; 6] = [0x1e, 0x30, 0x6c, 0xa2, 0xc1, 0x66];

static ETHERNET_IN_BUFFER: ConstStaticCell<[u8; 2048]> = ConstStaticCell::new([0; 2048]);
static ETHERNET_OUT_BUFFER: ConstStaticCell<[u8; 2048]> = ConstStaticCell::new([0; 2048]);

static CDC_NCM_ETH_CLASS: StaticCell<CdcNcmEthClass> = StaticCell::new();

pub struct CdcNcmEthClass {
    pub usb_class: Ethernet<'static, HalUsbBus<'static>>,
}

impl SubsysUsbClassFactory<HalUsbBus<'static>> for CdcNcmEthClass {
    fn new<'a>(usb_driver: &'static dyn UsbDriver<HalUsbBus<'static>>) -> &'a mut Self {
        let usb_class = Ethernet::new(
            usb_driver.usb_alloc(),
            HOST_MAC_ADDR,
            64,
            ETHERNET_IN_BUFFER.take(),
            ETHERNET_OUT_BUFFER.take(),
        );
        CDC_NCM_ETH_CLASS.init(CdcNcmEthClass { usb_class })
    }
}

impl SubsysUsbClass<HalUsbBus<'static>> for CdcNcmEthClass {
    fn handle_signal(&mut self) {
        if self.usb_class.state() == DeviceState::Disconnected {
            if self.usb_class.connection_speed().is_none() {
                // 1000 Kps upload and download
                match self.usb_class.set_connection_speed(1_000_000, 1_000_000) {
                    Ok(_) | Err(UsbError::WouldBlock) => {}
                    Err(e) => defmt::error!("Failed to set connection speed: {}", e),
                }
            } else if self.usb_class.state() == DeviceState::Disconnected {
                match self.usb_class.connect() {
                    Ok(_) | Err(UsbError::WouldBlock) => {}
                    Err(e) => defmt::error!("Failed to connect: {}", e),
                }
            }
        }
    }

    fn usb_class(&mut self) -> &mut dyn UsbClass<HalUsbBus<'static>> {
        &mut self.usb_class
    }
}
