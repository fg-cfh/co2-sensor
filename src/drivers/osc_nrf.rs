use super::api::{self, Driver};
use super::api::{osc::*, ApiError};
use super::resources_nrf::NrfOscDriverResources;
use core::{marker::PhantomData, result::Result};
use nrf52840_hal::clocks::{
    Clocks as NrfClocks, ExternalOscillator, LfOscConfiguration, LfOscStarted, HFCLK_FREQ,
    LFCLK_FREQ,
};
use static_cell::StaticCell;

// TODO: Make this configurable.
const HFXO_ACCURACY: Ppm = Ppm(30);
const LFXO_ACCURACY: Ppm = Ppm(50);
const LFXO_CONFIGURATION: LfOscConfiguration = LfOscConfiguration::NoExternalNoBypass;
type LfOscType = ExternalOscillator;

pub struct DontCare;

static OSC_DRIVER: StaticCell<NrfOscillatorsDriver<NrfLfOscillator, NrfHfOscillator>> =
    StaticCell::new();

mod osc_state {
    use super::{api, DontCare, LfOscType, LFXO_CONFIGURATION};
    use core::cell::Cell;
    use core::mem::MaybeUninit;
    use critical_section::{with as with_cs, CriticalSection, Mutex};
    /* use heapless::{
        arc_pool,
        pool::arc::{Arc, ArcBlock},
    }; */
    use nrf52840_hal::{
        clocks::{ExternalOscillator, Internal, LfOscStarted, LfOscStopped},
        pac::CLOCK,
        Clocks as NrfClocks,
    };

    enum NrfClocksState {
        OffOff(NrfClocks<Internal, LfOscType, LfOscStopped>),
        OffOn(NrfClocks<Internal, LfOscType, LfOscStarted>),
        OnOff(NrfClocks<ExternalOscillator, LfOscType, LfOscStopped>),
        OnOn(NrfClocks<ExternalOscillator, LfOscType, LfOscStarted>),
    }

    /* arc_pool!(ARCP: Mutex<RefCell<NrfClocksState>>);
    static ARCB: ConstStaticCell<[ArcBlock<Mutex<RefCell<NrfClocksState>>>;2]> =
        ConstStaticCell::new([ArcBlock::new(), ArcBlock::new()]); */

    static CLOCKS_STATE: Mutex<Cell<Option<NrfClocksState>>> = Mutex::new(Cell::new(None));

    pub fn init(clock: CLOCK) {
        with_cs(|cs| {
            let maybe_clocks_cell = CLOCKS_STATE.borrow(cs);
            let maybe_clocks = maybe_clocks_cell.take();
            if maybe_clocks.is_none() {
                let clocks_state = NrfClocksState::OffOff(
                    NrfClocks::new(clock).set_lfclk_src_external(LFXO_CONFIGURATION),
                );
                maybe_clocks_cell.replace(Some(clocks_state));
            } else {
                panic!("already initialized");
            }
        });
    }

    pub fn request_lf_osc() -> Result<NrfClocks<DontCare, LfOscType, LfOscStarted>, api::ApiError> {
        with_cs(|cs| {
            let maybe_clocks_cell = CLOCKS_STATE.borrow(cs);
            let prev_clocks_state = maybe_clocks_cell.take().unwrap();
            let next_clocks_state = match prev_clocks_state {
                NrfClocksState::OffOff(clocks) => NrfClocksState::OffOn(clocks.start_lfclk()),
                NrfClocksState::OnOff(clocks) => NrfClocksState::OnOn(clocks.start_lfclk()),
                _ => prev_clocks_state,
            };
            let res = Ok(clone_lf_clocks(&next_clocks_state, cs));
            maybe_clocks_cell.replace(Some(next_clocks_state));
            res
        })
    }

    pub fn request_hf_osc(
    ) -> Result<NrfClocks<ExternalOscillator, DontCare, DontCare>, api::ApiError> {
        with_cs(|cs| {
            let maybe_clocks_cell = CLOCKS_STATE.borrow(cs);
            let prev_clocks_state = maybe_clocks_cell.take().unwrap();
            let next_clocks_state = match prev_clocks_state {
                NrfClocksState::OffOff(clocks) => NrfClocksState::OnOff(clocks.enable_ext_hfosc()),
                NrfClocksState::OffOn(clocks) => NrfClocksState::OnOn(clocks.enable_ext_hfosc()),
                _ => prev_clocks_state,
            };
            let res = Ok(clone_hf_clocks(&next_clocks_state, cs));
            maybe_clocks_cell.replace(Some(next_clocks_state));
            res
        })
    }

    fn clone_lf_clocks(
        clocks_state: &NrfClocksState,
        _cs: CriticalSection,
    ) -> NrfClocks<DontCare, LfOscType, LfOscStarted> {
        let mut dst: MaybeUninit<NrfClocks<DontCare, LfOscType, LfOscStarted>> =
            MaybeUninit::uninit();
        // SAFETY: We're replacing one ZST to another.
        let src: *const NrfClocks<DontCare, LfOscType, LfOscStarted> = match clocks_state {
            NrfClocksState::OffOn(ref clocks) => {
                clocks as *const NrfClocks<Internal, LfOscType, LfOscStarted>
                    as *const NrfClocks<DontCare, LfOscType, LfOscStarted>
            }
            NrfClocksState::OnOn(ref clocks) => {
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
        clocks_state: &NrfClocksState,
        _cs: CriticalSection,
    ) -> NrfClocks<ExternalOscillator, DontCare, DontCare> {
        let mut dst: MaybeUninit<NrfClocks<ExternalOscillator, DontCare, DontCare>> =
            MaybeUninit::uninit();
        // SAFETY: We're replacing one ZST to another.
        let src: *const NrfClocks<ExternalOscillator, DontCare, DontCare> = match clocks_state {
            NrfClocksState::OnOff(ref clocks) => {
                clocks as *const NrfClocks<ExternalOscillator, LfOscType, LfOscStopped>
                    as *const NrfClocks<ExternalOscillator, DontCare, DontCare>
            }
            NrfClocksState::OnOn(ref clocks) => {
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

pub struct NrfLfOscillator;

impl Oscillator for NrfLfOscillator {
    fn freq(&self) -> Hertz {
        Hertz(LFCLK_FREQ)
    }

    fn freq_drift(&self) -> Ppm {
        LFXO_ACCURACY
    }
}

impl RequestOsc<NrfClocks<DontCare, LfOscType, LfOscStarted>> for NrfLfOscillator {
    fn request(&self) -> Result<NrfClocks<DontCare, LfOscType, LfOscStarted>, api::ApiError> {
        osc_state::request_lf_osc()
    }
}

impl RequestOsc<NrfClocks<ExternalOscillator, DontCare, DontCare>> for NrfHfOscillator {
    fn request(&self) -> Result<NrfClocks<ExternalOscillator, DontCare, DontCare>, api::ApiError> {
        osc_state::request_hf_osc()
    }
}

pub struct NrfHfOscillator;

impl Oscillator for NrfHfOscillator {
    fn freq(&self) -> Hertz {
        Hertz(HFCLK_FREQ)
    }

    fn freq_drift(&self) -> Ppm {
        HFXO_ACCURACY
    }
}

pub struct NrfOscillatorsDriver<Osc1, Osc2> {
    _osc1: PhantomData<Osc1>,
    _osc2: PhantomData<Osc2>,
}

impl api::Driver<NrfOscDriverResources> for NrfOscillatorsDriver<NrfLfOscillator, NrfHfOscillator> {
    fn new(resources: NrfOscDriverResources, _: ()) -> Self {
        let NrfOscDriverResources { clock } = resources;
        osc_state::init(clock);
        Self {
            _osc1: PhantomData,
            _osc2: PhantomData,
        }
    }
}

impl OscillatorsDriver<NrfLfOscillator, NrfHfOscillator>
    for NrfOscillatorsDriver<NrfLfOscillator, NrfHfOscillator>
{
    fn oscillators(&self) -> Oscillators<NrfLfOscillator, NrfHfOscillator> {
        Oscillators {
            lf_osc: NrfLfOscillator,
            high_acc_hf_osc: NrfHfOscillator,
        }
    }
}

pub fn init(
    resources: NrfOscDriverResources,
) -> Result<&'static NrfOscillatorsDriver<NrfLfOscillator, NrfHfOscillator>, ApiError> {
    OSC_DRIVER
        .try_init(NrfOscillatorsDriver::new(resources, ()))
        .map(|r| &*r)
        .ok_or(api::DRIVER_ALREADY_INITIALIZED)
}
