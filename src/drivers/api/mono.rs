use super::ApiError;

#[allow(async_fn_in_trait)]
pub trait MonoDriver {
    type Instant;
    type Duration;

    fn now(&self) -> Self::Instant;

    async fn delay(&self, duration: Self::Duration);

    async fn delay_until(&self, instant: Self::Instant);

    async fn timeout_at<F: core::future::Future>(
        &self,
        instant: Self::Instant,
        future: F,
    ) -> Result<F::Output, ApiError>;

    async fn timeout_after<F: core::future::Future>(
        &self,
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, ApiError>;
}
