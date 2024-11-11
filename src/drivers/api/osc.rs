use super::ApiError;

pub struct Hertz(pub u32);
pub struct Ppm(pub u16);

// Oscillator States
pub struct OscStopped;
pub struct OscStarted;

pub trait RequestOsc<InternalState> {
    fn request(&self) -> Result<InternalState, ApiError>;
}

pub trait Oscillator {
    fn freq(&self) -> Hertz;
    fn freq_drift(&self) -> Ppm;
}

pub struct Oscillators<Osc1, Osc2> {
    pub lf_osc: Osc1,
    pub high_acc_hf_osc: Osc2,
}

pub trait OscillatorsDriver<Osc1, Osc2> {
    fn oscillators(&self) -> Oscillators<Osc1, Osc2>;
}
