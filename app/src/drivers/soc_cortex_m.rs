use super::api::{soc::SocDriver, Driver, StatelessDriver};
use di::Initialized;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

struct SocCortexMDriver(StatelessDriver);

impl Initialized for SocCortexMDriver {
    fn init(&self) {
        self.0.init();
        use panic_probe as _;
    }

    fn is_initialized(&self) -> bool {
        self.0.is_initialized()
    }
}

impl Driver for SocCortexMDriver {}

impl SocDriver for SocCortexMDriver {}
