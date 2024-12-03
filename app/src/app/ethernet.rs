use crate::drivers;
use crate::subsys;
use drivers::api::mono::MonoDriver;
use smoltcp::{
    iface::{Interface, SocketHandle, SocketSet},
    socket::dhcpv4,
    time::Instant,
    wire::{IpCidr, Ipv4Address, Ipv4Cidr},
};
use subsys::usb::{class_cdc_ncm_eth::CdcNcmEthClass, HalUsbBus, SubsysUsbDevice};

pub fn idle(
    usb_dev: &impl SubsysUsbDevice<HalUsbBus<'static>, CdcNcmEthClass>,
    dhcp_handle: &SocketHandle,
    interface: &mut Interface,
    sockets: &mut SocketSet<'static>,
    mono: &impl MonoDriver<Instant = drivers::Instant, Duration = drivers::Duration>,
) -> ! {
    loop {
        // cortex_m::asm::wfe();

        usb_dev.poll(&mut |subsys_class| {
            let ethernet = &mut subsys_class.usb_class;
            if ethernet.state() == usbd_ethernet::DeviceState::Connected {
                // panic safety - will take 292_277 years to overflow at one tick per microsecond
                let timestamp = Instant::from_micros(
                    i64::try_from(mono.now().duration_since_epoch().to_micros()).unwrap(),
                );

                if interface.poll(timestamp, ethernet, sockets) {
                    dhcp_poll(interface, sockets.get_mut::<dhcpv4::Socket>(*dhcp_handle));
                }
            }
        });
    }
}
fn dhcp_poll(iface: &mut Interface, socket: &mut dhcpv4::Socket) {
    let event = socket.poll();
    match event {
        None => {}
        Some(dhcpv4::Event::Configured(config)) => {
            defmt::info!("dhcp: DHCP configured");

            defmt::info!("     IP address:      {}", config.address);
            set_ipv4_addr(iface, config.address);

            if let Some(router) = config.router {
                defmt::info!("     Default gateway: {}", router);
                iface.routes_mut().add_default_ipv4_route(router).unwrap();
            } else {
                defmt::info!("     Default gateway: None");
                iface.routes_mut().remove_default_ipv4_route();
            }

            for (i, s) in config.dns_servers.iter().enumerate() {
                defmt::info!("     DNS server {}:    {}", i, s);
            }
        }
        Some(dhcpv4::Event::Deconfigured) => {
            defmt::info!("dhcp: DHCP deconfigured");
            set_ipv4_addr(iface, Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0));
            iface.routes_mut().remove_default_ipv4_route();
        }
    }
}

fn set_ipv4_addr(iface: &mut Interface, cidr: Ipv4Cidr) {
    iface.update_ip_addrs(|addrs| {
        let dest = addrs.iter_mut().next().unwrap();
        *dest = IpCidr::Ipv4(cidr);
    });
}
