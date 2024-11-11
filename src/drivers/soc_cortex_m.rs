use super::api::{ApiError, StatelessDriver};

static DRIVER: StatelessDriver = StatelessDriver::new();

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

pub fn init() -> Result<(), ApiError> {
    DRIVER.init((), |_| {
        use panic_probe as _;
    })
}
