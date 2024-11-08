#![deny(unsafe_code)]
#![deny(warnings)]
#![allow(unsafe_op_in_unsafe_fn)]
#![no_main]
#![no_std]

use co2_sensor::hal;

use rtic_monotonics::nrf::rtc::prelude::*;
nrf_rtc0_monotonic!(Mono);

#[rtic::app(device = hal::pac, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use defmt;
    use embedded_hal::digital::OutputPin;
    use hal::gpio::{Level, Output, Pin, PushPull};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Pin<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Clear terminal and position the cursor in the upper left corner.
        let CLEAR_SCREEN: defmt::Str = defmt::intern!("\x1b[2J\x1b[1;1H");

        defmt::println!("{=istr}", CLEAR_SCREEN);
        defmt::info!("----------------");
        defmt::info!("-- CO2 Sensor --");
        defmt::info!("----------------");

        // Enable the DC/DC converter
        cx.device.POWER.dcdcen.write(|w| w.dcdcen().enabled());

        // Configure low frequency clock
        hal::clocks::Clocks::new(cx.device.CLOCK).start_lfclk();

        // Initialize Monotonic
        Mono::start(cx.device.RTC0);

        // Setup LED
        let port0 = hal::gpio::p0::Parts::new(cx.device.P0);
        let led = port0.p0_13.into_push_pull_output(Level::Low).degrade();

        // Schedule the blinking task
        blink::spawn().ok();

        (Shared {}, Local { led })
    }

    #[task(local = [led], priority=1)]
    async fn blink(cx: blink::Context) {
        let blink::LocalResources { led, .. } = cx.local;

        let mut next_tick = Mono::now();
        let mut blink_on = true;
        loop {
            // let now = Mono::now();
            // let now_ms: fugit::SecsDurationU64 = now.duration_since_epoch().convert();
            // defmt::println!("Timer {} ({})", now_ms, now.ticks());

            blink_on = !blink_on;
            if blink_on {
                led.set_high().unwrap();
            } else {
                led.set_low().unwrap();
            }

            next_tick += 1000.millis();
            Mono::delay_until(next_tick).await;
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rtic::export::wfi()
        }
    }
}
