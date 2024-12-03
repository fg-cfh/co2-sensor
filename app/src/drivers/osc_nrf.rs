use super::api::{self, osc::*, Driver};
use super::resources_nrf::NrfDriverResources;
use core::mem::MaybeUninit;
use core::result::Result;
use di::{Singleton, SingletonHolder, SingletonHolderImpl};
use nrf52840_hal::clocks::{
    Clocks as NrfClocks, ExternalOscillator, Internal, LfOscConfiguration, LfOscStarted,
    LfOscStopped, HFCLK_FREQ, LFCLK_FREQ,
};
use nrf52840_hal::pac::CLOCK;

// TODO: Make this configurable.
const HFXO_ACCURACY: Ppm = Ppm(30);
const LFXO_ACCURACY: Ppm = Ppm(50);
const LFXO_CONFIGURATION: LfOscConfiguration = LfOscConfiguration::NoExternalNoBypass;
type LfOscType = ExternalOscillator;

pub struct DontCare;

enum NrfOscDriverState {
    OffOff(NrfClocks<Internal, LfOscType, LfOscStopped>),
    OffOn(NrfClocks<Internal, LfOscType, LfOscStarted>),
    OnOff(NrfClocks<ExternalOscillator, LfOscType, LfOscStopped>),
    OnOn(NrfClocks<ExternalOscillator, LfOscType, LfOscStarted>),
}

impl Singleton<NrfOscDriverState, CLOCK> for NrfOscDriverState {
    fn with_state_holder<F, R>(f: F) -> R {
        static DRIVER_STATE: SingletonHolderImpl<NrfOscDriverState> = Default::default();
        f(&mut DRIVER_STATE)
    }

    fn with_dependencies<F>(f: F) {
        NrfDriverResources::with_state(|resources| f(resources.clock.take().unwrap()));
    }

    fn from(clock: CLOCK) -> NrfOscDriverState {
        NrfOscDriverState::OffOff(NrfClocks::new(clock).set_lfclk_src_external(LFXO_CONFIGURATION))
    }
}

impl NrfOscDriverState {
    fn request_lf_osc() -> Result<NrfClocks<DontCare, LfOscType, LfOscStarted>, api::ApiError> {
        Self::with_state(|prev_driver_state| {
            let next_driver_state = match prev_driver_state {
                NrfOscDriverState::OffOff(clocks) => NrfOscDriverState::OffOn(clocks.start_lfclk()),
                NrfOscDriverState::OnOff(clocks) => NrfOscDriverState::OnOn(clocks.start_lfclk()),
                _ => prev_driver_state,
            };
            let result = Ok(Self::clone_lf_clocks(&next_driver_state));
            (next_driver_state, result)
        })
    }

    fn request_hf_osc() -> Result<NrfClocks<ExternalOscillator, DontCare, DontCare>, api::ApiError>
    {
        Self::with_state(|prev_driver_state| {
            let next_driver_state = match prev_driver_state {
                NrfOscDriverState::OffOff(clocks) => {
                    NrfOscDriverState::OnOff(clocks.enable_ext_hfosc())
                }
                NrfOscDriverState::OffOn(clocks) => {
                    NrfOscDriverState::OnOn(clocks.enable_ext_hfosc())
                }
                _ => prev_driver_state,
            };
            let result = Ok(Self::clone_hf_clocks(&next_driver_state));
            (next_driver_state, result)
        })
    }

    fn clone_lf_clocks(
        clocks_state: &NrfOscDriverState,
    ) -> NrfClocks<DontCare, LfOscType, LfOscStarted> {
        let mut dst: MaybeUninit<NrfClocks<DontCare, LfOscType, LfOscStarted>> =
            MaybeUninit::uninit();
        // SAFETY: We're replacing one ZST by another.
        let src: *const NrfClocks<DontCare, LfOscType, LfOscStarted> = match clocks_state {
            NrfOscDriverState::OffOn(ref clocks) => {
                clocks as *const NrfClocks<Internal, LfOscType, LfOscStarted>
                    as *const NrfClocks<DontCare, LfOscType, LfOscStarted>
            }
            NrfOscDriverState::OnOn(ref clocks) => {
                clocks as *const NrfClocks<ExternalOscillator, LfOscType, LfOscStarted>
                    as *const NrfClocks<DontCare, LfOscType, LfOscStarted>
            }
            _ => unreachable!(),
        };
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst.as_mut_ptr(), 1);
            dst.assume_init()
        }
    }

    fn clone_hf_clocks(
        clocks_state: &NrfOscDriverState,
    ) -> NrfClocks<ExternalOscillator, DontCare, DontCare> {
        let mut dst: MaybeUninit<NrfClocks<ExternalOscillator, DontCare, DontCare>> =
            MaybeUninit::uninit();
        // SAFETY: We're replacing one ZST by another.
        let src: *const NrfClocks<ExternalOscillator, DontCare, DontCare> = match clocks_state {
            NrfOscDriverState::OnOff(ref clocks) => {
                clocks as *const NrfClocks<ExternalOscillator, LfOscType, LfOscStopped>
                    as *const NrfClocks<ExternalOscillator, DontCare, DontCare>
            }
            NrfOscDriverState::OnOn(ref clocks) => {
                clocks as *const NrfClocks<ExternalOscillator, LfOscType, LfOscStarted>
                    as *const NrfClocks<ExternalOscillator, DontCare, DontCare>
            }
            _ => unreachable!(),
        };
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst.as_mut_ptr(), 1);
            dst.assume_init()
        }
    }
}

pub struct NrfLfOscillatorDriver;

struct NrfOscToken(NrfClocks<DontCare, LfOscType, LfOscStarted>);

impl Drop for NrfOscToken {
    fn drop(&mut self) {
        todo!()
    }
}

impl Driver for NrfLfOscillatorDriver {
    fn ensure_is_initialized() {
        NrfOscDriverState::ensure_is_initialized();
    }
}

impl OscillatorDriver for NrfLfOscillatorDriver {
    fn freq() -> Hertz {
        Hertz(LFCLK_FREQ)
    }

    fn freq_drift() -> Ppm {
        LFXO_ACCURACY
    }

    fn request<NrfOscToken>() -> Result<NrfOscToken, api::ApiError> {
        NrfOscToken(NrfOscDriverState::request_lf_osc())
    }
}

pub struct NrfHfOscillatorDriver;

impl Driver for NrfHfOscillatorDriver {
    fn ensure_is_initialized() {
        NrfOscDriverState::ensure_state();
    }
}

impl OscillatorDriver for NrfHfOscillatorDriver {
    fn freq() -> Hertz {
        Hertz(HFCLK_FREQ)
    }

    fn freq_drift() -> Ppm {
        HFXO_ACCURACY
    }

    fn request<NrfOscToken>() -> Result<NrfOscToken, api::ApiError> {
        NrfOscToken(NrfOscDriverState::request_hf_osc())
    }
}
