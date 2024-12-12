use super::api::{log::LogDriver, Driver, StatelessDriver};
use di::Initialized;

#[derive(Default)]
struct DefmtRttDriver(StatelessDriver);

impl Initialized for DefmtRttDriver {
    fn init(&self) {
        self.0.init();
        use defmt_rtt as _;
    }

    fn is_initialized(&self) -> bool {
        self.0.is_initialized()
    }
}

impl Driver for DefmtRttDriver {}

impl LogDriver for DefmtRttDriver {}
