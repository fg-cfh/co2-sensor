use super::api::{ApiError, StatelessDriver};

static DRIVER: StatelessDriver = StatelessDriver::new();

pub fn init() -> Result<(), ApiError> {
    DRIVER.init((), |_| {
        use defmt_rtt as _;
    })
}
