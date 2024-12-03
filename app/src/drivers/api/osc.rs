use super::{ApiError, Driver};

pub struct Hertz(pub u32);
pub struct Ppm(pub u16);

pub trait OscillatorDriver: Driver {
    fn freq() -> Hertz;
    fn freq_drift() -> Ppm;
    fn request<OscToken>() -> Result<OscToken, ApiError>
    where
        OscToken: Drop;
}
