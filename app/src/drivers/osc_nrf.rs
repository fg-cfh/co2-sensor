use super::api::{self, osc::*, Driver, DriverStateHolder};
use super::resources_nrf::NrfDriverResources;
use core::mem::MaybeUninit;
use core::result::Result;
use di::singleton::Singleton;
use di::token::{Release, SharedToken};
use di::{Initialized, WithDependency};
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

enum NrfOscState {
    OffOff(NrfClocks<Internal, LfOscType, LfOscStopped>),
    OffOn(NrfClocks<Internal, LfOscType, LfOscStarted>),
    OnOff(NrfClocks<ExternalOscillator, LfOscType, LfOscStopped>),
    OnOn(NrfClocks<ExternalOscillator, LfOscType, LfOscStarted>),
}

impl WithDependency<CLOCK> for NrfOscState {
    fn with_dependency<Result, F: FnOnce(CLOCK) -> Result>(f: F) -> Result {
        NrfDriverResources::with_ref_mut(|resources| f(resources.clock.take().unwrap()))
    }
}

impl Default for NrfOscState {
    fn default() -> Self {
        Self::with_dependency(|clock| {
            Self::OffOff(NrfClocks::new(clock).set_lfclk_src_external(LFXO_CONFIGURATION))
        })
    }
}

// SAFETY: The LF and HF clock registers can be accessed independently from each other.
type NrfSleepOscState = NrfClocks<DontCare, LfOscType, LfOscStarted>;
type NrfHighAccOscState = NrfClocks<ExternalOscillator, DontCare, DontCare>;

pub struct NrfSleepOscToken(pub NrfSleepOscState);

impl Default for NrfSleepOscToken {
    fn default() -> Self {
        let mut dst: MaybeUninit<NrfClocks<DontCare, LfOscType, LfOscStarted>> =
            MaybeUninit::uninit();
        let src = NrfOscDriverState::with_ref(|osc_state| match osc_state {
            NrfOscState::OffOn(clocks) => {
                clocks as *const NrfClocks<Internal, LfOscType, LfOscStarted>
                    as *const NrfClocks<DontCare, LfOscType, LfOscStarted>
            }
            NrfOscState::OnOn(clocks) => {
                clocks as *const NrfClocks<ExternalOscillator, LfOscType, LfOscStarted>
                    as *const NrfClocks<DontCare, LfOscType, LfOscStarted>
            }
            _ => unreachable!(),
        });
        let nrf_clocks = unsafe {
            // SAFETY: We're replacing one ZST by another.
            core::ptr::copy_nonoverlapping(src, dst.as_mut_ptr(), 1);
            dst.assume_init()
        };
        // SAFETY: We hand out a unique token per oscillator.
        NrfSleepOscToken(nrf_clocks)
    }
}

impl Release for NrfSleepOscToken {
    fn release(&self) {
        NrfOscDriverState::stop_sleep_osc();
    }
}

pub struct NrfHighAccOscToken(pub NrfHighAccOscState);

impl Default for NrfHighAccOscToken {
    fn default() -> NrfHighAccOscToken {
        let mut dst: MaybeUninit<NrfClocks<ExternalOscillator, DontCare, DontCare>> =
            MaybeUninit::uninit();
        let src = NrfOscDriverState::with_ref(|osc_state| match osc_state {
            NrfOscState::OnOff(clocks) => {
                clocks as *const NrfClocks<ExternalOscillator, LfOscType, LfOscStopped>
                    as *const NrfClocks<ExternalOscillator, DontCare, DontCare>
            }
            NrfOscState::OnOn(clocks) => {
                clocks as *const NrfClocks<ExternalOscillator, LfOscType, LfOscStarted>
                    as *const NrfClocks<ExternalOscillator, DontCare, DontCare>
            }
            _ => unreachable!(),
        });
        let nrf_clocks = unsafe {
            // SAFETY: We're replacing one ZST by another.
            core::ptr::copy_nonoverlapping(src, dst.as_mut_ptr(), 1);
            dst.assume_init()
        };
        // SAFETY: We hand out a unique token per oscillator.
        NrfHighAccOscToken(nrf_clocks)
    }
}

impl Release for NrfHighAccOscToken {
    fn release(&self) {
        NrfOscDriverState::stop_high_acc_osc();
    }
}

struct NrfOscDriverState;

impl Singleton for NrfOscDriverState {
    type Content = NrfOscState;

    fn with_state_holder<Result, F>(f: F) -> Result
    where
        F: FnOnce(&'static DriverStateHolder<Self::Content>) -> Result,
    {
        static DRIVER_STATE: DriverStateHolder<NrfOscState> = DriverStateHolder::new();
        f(&DRIVER_STATE)
    }
}

impl NrfOscDriverState {
    fn request_sleep_osc() -> Result<SharedToken<'static, NrfSleepOscToken>, api::ApiError> {
        Self::with(|prev_driver_state| {
            let next_driver_state = match prev_driver_state {
                NrfOscState::OffOff(clocks) => NrfOscState::OffOn(clocks.start_lfclk()),
                NrfOscState::OnOff(clocks) => NrfOscState::OnOn(clocks.start_lfclk()),
                _ => prev_driver_state,
            };
            (next_driver_state, ())
        });
        Ok(SharedToken::new(Default::default()))
    }

    fn stop_sleep_osc() {
        Self::with(|prev_driver_state| {
            let next_driver_state = match prev_driver_state {
                NrfOscState::OffOn(clocks) => NrfOscState::OffOff(clocks.stop_lfclk()),
                NrfOscState::OnOn(clocks) => NrfOscState::OnOff(clocks.stop_lfclk()),
                _ => unreachable!(),
            };
            (next_driver_state, ())
        });
    }

    fn request_high_acc_osc() -> Result<SharedToken<'static, NrfHighAccOscToken>, api::ApiError> {
        Self::with(|prev_driver_state| {
            let next_driver_state = match prev_driver_state {
                NrfOscState::OffOff(clocks) => NrfOscState::OnOff(clocks.enable_ext_hfosc()),
                NrfOscState::OffOn(clocks) => NrfOscState::OnOn(clocks.enable_ext_hfosc()),
                _ => prev_driver_state,
            };
            (next_driver_state, ())
        });
        Ok(SharedToken::new(Default::default()))
    }

    fn stop_high_acc_osc() {
        Self::with(|prev_driver_state| {
            let next_driver_state = match prev_driver_state {
                NrfOscState::OnOff(clocks) => NrfOscState::OffOff(clocks.disable_ext_hfosc()),
                NrfOscState::OnOn(clocks) => NrfOscState::OffOn(clocks.disable_ext_hfosc()),
                _ => unreachable!(),
            };
            (next_driver_state, ())
        });
    }
}

pub struct NrfSleepOscillatorDriver;

impl NrfSleepOscillatorDriver {
    pub const fn new() -> Self {
        NrfSleepOscillatorDriver
    }
}

impl Initialized for NrfSleepOscillatorDriver {
    fn init(&self) {
        NrfOscDriverState.init();
    }

    fn is_initialized(&self) -> bool {
        NrfOscDriverState.is_initialized()
    }
}

impl<'a> OscillatorDriver<'a> for NrfSleepOscillatorDriver {
    type OscToken = NrfSleepOscToken;

    fn freq() -> Hertz {
        Hertz(LFCLK_FREQ)
    }

    fn freq_drift() -> Ppm {
        LFXO_ACCURACY
    }

    fn request() -> Result<SharedToken<'a, Self::OscToken>, api::ApiError> {
        NrfOscDriverState::request_sleep_osc()
    }
}

impl Driver for NrfSleepOscillatorDriver {}

struct NrfHfOscToken(NrfClocks<ExternalOscillator, DontCare, DontCare>);

impl Drop for NrfHfOscToken {
    fn drop(&mut self) {
        todo!()
    }
}

pub struct NrfHighAccOscillatorDriver;

impl<'a> NrfHighAccOscillatorDriver {
    const fn new() -> Self {
        NrfHighAccOscillatorDriver
    }
}

impl Initialized for NrfHighAccOscillatorDriver {
    fn init(&self) {
        NrfOscDriverState.init();
    }

    fn is_initialized(&self) -> bool {
        NrfOscDriverState.is_initialized()
    }
}

impl<'a> OscillatorDriver<'a> for NrfHighAccOscillatorDriver {
    type OscToken = NrfHighAccOscToken;

    fn freq() -> Hertz {
        Hertz(HFCLK_FREQ)
    }

    fn freq_drift() -> Ppm {
        HFXO_ACCURACY
    }

    fn request() -> Result<SharedToken<'a, NrfHighAccOscToken>, api::ApiError> {
        NrfOscDriverState::request_high_acc_osc()
    }
}

impl Driver for NrfHighAccOscillatorDriver {}
