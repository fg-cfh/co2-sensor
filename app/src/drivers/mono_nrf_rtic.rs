use super::api::{self, mono::*, osc::OscillatorDriver, Driver, StatelessDriver};
use super::osc_nrf::NrfLfOscillatorDriver;
use super::resources_nrf::NrfDriverResources;
use di::Singleton;
use nrf52840_hal::pac::RTC0;
use rtic_monotonics::{Monotonic, TimerQueueBasedMonotonic};

mod private {
    pub use rtic_monotonics::nrf::rtc::prelude::*;
    nrf_rtc0_monotonic!(Rtc0Mono);
}

pub struct NrfRticMonoDriver;

impl StatelessDriver<RTC0> for NrfRticMonoDriver {
    fn with_dependencies<F>(f: F) {
        NrfDriverResources::with_state(|resources| f(resources.rtc0.take().unwrap()));
    }

    fn init_once(rtc0: RTC0) {
        NrfLfOscillatorDriver::ensure_is_initialized();
        NrfLfOscillatorDriver::request();
        private::Rtc0Mono::start(rtc0);
    }
}

impl Driver for NrfRticMonoDriver {
    fn ensure_is_initialized() {
        Self::ensure_is_initialized();
    }
}

pub type Instant = <private::Rtc0Mono as TimerQueueBasedMonotonic>::Instant;
pub type Duration = <private::Rtc0Mono as TimerQueueBasedMonotonic>::Duration;

impl MonoDriver for NrfRticMonoDriver {
    type Instant = Instant;
    type Duration = Duration;

    fn now(&self) -> Self::Instant {
        private::Rtc0Mono::now()
    }

    async fn delay(&self, duration: Self::Duration) {
        private::Rtc0Mono::delay(duration).await
    }

    async fn delay_until(&self, instant: Self::Instant) {
        private::Rtc0Mono::delay_until(instant).await
    }

    async fn timeout_at<F: core::future::Future>(
        &self,
        instant: Self::Instant,
        future: F,
    ) -> Result<F::Output, api::ApiError> {
        private::Rtc0Mono::timeout_at::<F>(instant, future)
            .await
            .map_err(|_| api::TIMEOUT)
    }

    async fn timeout_after<F: core::future::Future>(
        &self,
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, api::ApiError> {
        private::Rtc0Mono::timeout_after::<F>(duration, future)
            .await
            .map_err(|_| api::TIMEOUT)
    }
}
