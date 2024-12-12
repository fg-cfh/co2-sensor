use di::token::{Release, SharedToken};

use super::{ApiError, Driver};

pub struct Hertz(pub u32);
pub struct Ppm(pub u16);

pub trait OscillatorDriver<'a>: Driver {
    type OscToken: Default + Release + Send;

    fn freq() -> Hertz;
    fn freq_drift() -> Ppm;
    fn request() -> Result<SharedToken<'a, Self::OscToken>, ApiError>;
}
