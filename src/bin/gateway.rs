#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]

use co2_sensor::drivers;

#[rtic::app(device = drivers::pac, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use drivers::gpio::*;
    use drivers::monotonic::*;
    use drivers::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        drivers: Drivers,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Clear terminal and position the cursor in the upper left corner.
        let CLEAR_SCREEN = defmt::intern!("\x1b[2J\x1b[1;1H");

        defmt::println!("{=istr}", CLEAR_SCREEN);

        defmt::info!("----------------");
        defmt::info!("-- CO2 Sensor --");
        defmt::info!("----------------");

        let drivers = drivers::init(cx.device);

        // Schedule the blinking task
        blink::spawn().ok();

        (Shared {}, Local { drivers })
    }

    #[task(local = [drivers], priority=1)]
    async fn blink(cx: blink::Context) {
        let Drivers {
            gpio, monotonic, ..
        } = cx.local.drivers;

        let mut next_tick = monotonic.now();
        let mut blink_on = false;
        loop {
            // let now = Mono::now();
            // let now_ms: fugit::SecsDurationU64 = now.duration_since_epoch().convert();
            // defmt::println!("Timer {} ({})", now_ms, now.ticks());

            if blink_on {
                gpio.led.set_high().unwrap();
            } else {
                gpio.led.set_low().unwrap();
            }

            blink_on = !blink_on;

            next_tick += 1000.millis();
            monotonic.delay_until(next_tick).await;
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rtic::export::wfi()
        }
    }
}
