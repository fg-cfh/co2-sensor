#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]

use co2_sensor::{drivers, subsys};

#[rtic::app(device = drivers::pac, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use device_nrf::NrfUsbDevice;
    use drivers::api::gpio::*;
    use drivers::api::mono::*;
    use drivers::api::rng::*;
    use fugit::ExtU32;
    use smoltcp::{
        iface::{Config, Interface, SocketHandle, SocketSet, SocketStorage},
        socket::dhcpv4,
        time::{Duration, Instant},
        wire::{DhcpOption, EthernetAddress, HardwareAddress, Ipv4Address, Ipv4Cidr},
    };
    use static_cell::StaticCell;
    use subsys::usb::{self, *};

    const HOST_NAME: &[u8] = b"co2-sensor-gateway";
    const DEVICE_MAC_ADDR: [u8; 6] = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];

    #[shared]
    struct Shared {
        mono: drivers::MonoDriver,
        gpio: drivers::GpioDriver,
    }

    #[local]
    struct Local {
        usb_dev: NrfUsbDevice,
        dhcp_handle: SocketHandle,
        interface: Interface,
        sockets: SocketSet<'static>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Clear terminal and position the cursor in the upper left corner.
        let CLEAR_SCREEN = defmt::intern!("\x1b[2J\x1b[1;1H");

        defmt::println!("{=istr}", CLEAR_SCREEN);

        defmt::info!("----------------");
        defmt::info!("-- CO2 Sensor --");
        defmt::info!("----------------");

        let mut drivers = drivers::init(cx.device);

        let ethernet = usb::class_cdc_ncm_eth::CdcNcmEthClass::new(drivers.usb);

        let mut interface_config =
            Config::new(HardwareAddress::Ethernet(EthernetAddress(DEVICE_MAC_ADDR)));
        interface_config.random_seed = drivers.rng.next_u64();

        let now = Instant::from_micros(
            i64::try_from(drivers.mono.now().duration_since_epoch().to_micros()).unwrap(),
        );
        let mut interface = Interface::new(interface_config, &mut ethernet.usb_class, now);
        interface.update_ip_addrs(|ip_addrs| {
            ip_addrs
                .push(Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0).into())
                .unwrap();
        });

        // Create sockets
        let mut dhcp_socket = dhcpv4::Socket::new();
        let mut retry_config = dhcpv4::RetryConfig::default();
        retry_config.discover_timeout = Duration::from_secs(5);
        dhcp_socket.set_retry_config(retry_config);
        // register hostname with dhcp
        dhcp_socket.set_outgoing_options(&[DhcpOption {
            kind: 12, // Host Name
            data: HOST_NAME,
        }]);

        static SOCKETS: StaticCell<[SocketStorage<'static>; 2]> = StaticCell::new();
        let sockets = SOCKETS.init(Default::default());
        let mut sockets = SocketSet::new(&mut sockets[..]);
        let dhcp_handle = sockets.add(dhcp_socket);

        let usb_dev = usb::device_nrf::NrfUsbDevice::new(drivers.usb, ethernet);

        // Schedule the blinking task
        blink::spawn().ok();

        (
            Shared {
                mono: drivers.mono,
                gpio: drivers.gpio,
            },
            Local {
                usb_dev,
                dhcp_handle,
                interface,
                sockets,
            },
        )
    }

    #[task(shared = [gpio, &mono], priority=1)]
    async fn blink(cx: blink::Context) {
        let mono = cx.shared.mono;
        let mut gpio = cx.shared.gpio;

        let mut next_tick = mono.now();
        let mut blink_on = false;
        loop {
            gpio.lock(|gpio| {
                if blink_on {
                    gpio.output_pin(GpioOutputPin::LED).set_high().unwrap();
                } else {
                    gpio.output_pin(GpioOutputPin::LED).set_low().unwrap();
                }
            });

            blink_on = !blink_on;

            next_tick += 1000.millis();
            mono.delay_until(next_tick).await;
        }
    }

    #[idle(local = [usb_dev, dhcp_handle, interface, sockets], shared = [&mono])]
    fn idle(cx: idle::Context) -> ! {
        let idle::LocalResources {
            usb_dev,
            dhcp_handle,
            interface,
            sockets,
            ..
        } = cx.local;
        let mono = cx.shared.mono;

        co2_sensor::app::ethernet::idle(usb_dev, dhcp_handle, interface, sockets, mono)
    }
}
