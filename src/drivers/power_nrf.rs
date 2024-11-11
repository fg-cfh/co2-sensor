use super::api::{ApiError, StatelessDriver};
use super::resources_nrf::NrfPowerDriverResources;

static DRIVER: StatelessDriver = StatelessDriver::new();

pub fn init(resources: NrfPowerDriverResources) -> Result<(), ApiError> {
    DRIVER.init(resources, |resources| {
        // Enable the DC/DC converter
        resources.power.dcdcen.write(|w| w.dcdcen().enabled());
    })
}
