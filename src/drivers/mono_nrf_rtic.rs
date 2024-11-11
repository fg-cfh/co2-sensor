use super::super::drivers::osc_nrf::NrfLfOscillator;
use super::api::mono::*;
use super::api::osc::*;
use super::api::{self, Driver};
use super::osc_nrf::NrfHfOscillator;
use super::osc_nrf::NrfOscillatorsDriver;
use super::resources_nrf::NrfRticMonoDriverResources;
use rtic_monotonics::{Monotonic, TimerQueueBasedMonotonic};

mod private {
    pub use rtic_monotonics::nrf::rtc::prelude::*;
    nrf_rtc0_monotonic!(Rtc0Mono);
}

pub struct NrfRticMonoDriver;

impl
    api::Driver<NrfRticMonoDriverResources, &NrfOscillatorsDriver<NrfLfOscillator, NrfHfOscillator>>
    for NrfRticMonoDriver
{
    fn new(
        resources: NrfRticMonoDriverResources,
        osc_driver: &NrfOscillatorsDriver<NrfLfOscillator, NrfHfOscillator>,
    ) -> NrfRticMonoDriver {
        osc_driver.oscillators().lf_osc.request().unwrap();
        private::Rtc0Mono::start(resources.rtc0);
        NrfRticMonoDriver {}
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

pub fn init(
    resources: NrfRticMonoDriverResources,
    osc_driver: &NrfOscillatorsDriver<NrfLfOscillator, NrfHfOscillator>,
) -> Result<NrfRticMonoDriver, api::ApiError> {
    Ok(NrfRticMonoDriver::new(resources, osc_driver))
}
