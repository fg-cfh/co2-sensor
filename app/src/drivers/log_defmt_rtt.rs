use super::api::{log::LogDriver, Driver, StatelessDriver};

struct DefmtRttDriver;

impl StatelessDriver<()> for DefmtRttDriver {
    fn with_dependencies<F>(f: F) {
        f(());
    }

    fn init_once(_: ()) {
        use defmt_rtt as _;
    }
}

impl Driver for DefmtRttDriver {
    fn ensure_is_initialized() {
        Self::ensure_is_initialized();
    }
}

impl LogDriver for DefmtRttDriver {}
