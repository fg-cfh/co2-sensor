use nrf52840_hal::{
    clocks::{ExternalOscillator, LfOscConfiguration, LfOscStarted, LfOscStopped},
    pac::CLOCK,
    Clocks,
};

use super::resources::ClocksDriverResources;

pub struct Api<LSTATE> {
    pub clocks: Clocks<ExternalOscillator, ExternalOscillator, LSTATE>,
}

impl Api<LfOscStopped> {
    fn new(clock: CLOCK) -> Api<LfOscStopped> {
        Api {
            clocks: Clocks::new(clock)
                .enable_ext_hfosc()
                .set_lfclk_src_external(LfOscConfiguration::NoExternalNoBypass),
        }
    }

    pub fn start_lf_osc(self) -> Api<LfOscStarted> {
        Api {
            clocks: self.clocks.start_lfclk(),
        }
    }
}

impl Api<LfOscStarted> {
    pub fn stop_lf_osc(self) -> Api<LfOscStopped> {
        Api {
            clocks: self.clocks.stop_lfclk(),
        }
    }
}

pub fn init(resources: ClocksDriverResources) -> Api<LfOscStopped> {
    Api::new(resources.clock)
}
