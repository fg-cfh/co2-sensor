use super::clocks::Api as ClocksDriverApi;
use super::resources::MonotonicDriverResources;
use nrf52840_hal::{
    clocks::{LfOscStarted, LfOscStopped},
    pac::clock,
};
use rtic_monotonics::{Monotonic, TimerQueueBasedMonotonic};

mod _internal {
    pub use rtic_monotonics::nrf::rtc::prelude::*;
    nrf_rtc0_monotonic!(Mono);
}

pub type MonotonicInstant = <_internal::Mono as TimerQueueBasedMonotonic>::Instant;
pub type MonotonicDuration = <_internal::Mono as TimerQueueBasedMonotonic>::Duration;

pub use _internal::ExtU64;

pub struct Api;

impl Api {
    pub fn now(self: &Api) -> MonotonicInstant {
        _internal::Mono::now()
    }

    pub async fn delay_until(self: &Api, instant: MonotonicInstant) {
        _internal::Mono::delay_until(instant).await
    }
}

pub fn init(
    resources: MonotonicDriverResources,
    clocks_driver: ClocksDriverApi<LfOscStopped>,
) -> (Api, ClocksDriverApi<LfOscStarted>) {
    let clocks_driver = clocks_driver.start_lf_osc();
    _internal::Mono::start(resources.rtc0);
    (Api {}, clocks_driver)
}
