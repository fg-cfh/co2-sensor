pub trait RngDriver {
    fn next(&self, dest: &mut [u8]);

    fn next_u8(&self) -> u8 {
        let mut buf = [0; 1];
        self.next(&mut buf);
        buf[0]
    }

    fn next_u16(&self) -> u16 {
        let mut buf = [0; 2];
        self.next(&mut buf);
        buf[0] as u16 | (buf[1] as u16) << 8
    }
}

#[macro_export]
macro_rules! impl_rng_driver_rng_core {
    ($rng_driver:ty) => {
        use rand_core::RngCore;

        impl RngCore for $rng_driver
        where
            $rng_driver: RngDriver,
        {
            fn fill_bytes(&mut self, dest: &mut [u8]) {
                self.next(dest);
            }

            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
                self.fill_bytes(dest);
                Ok(())
            }

            fn next_u32(&mut self) -> u32 {
                let mut buf = [0; 4];
                self.fill_bytes(&mut buf);
                buf[0] as u32 | (buf[1] as u32) << 8 | (buf[2] as u32) << 16 | (buf[3] as u32) << 24
            }

            fn next_u64(&mut self) -> u64 {
                let mut buf = [0; 8];
                self.fill_bytes(&mut buf);
                buf[0] as u64
                    | (buf[1] as u64) << 8
                    | (buf[2] as u64) << 16
                    | (buf[3] as u64) << 24
                    | (buf[4] as u64) << 32
                    | (buf[5] as u64) << 40
                    | (buf[6] as u64) << 48
                    | (buf[7] as u64) << 56
            }
        }
    };
}

pub use impl_rng_driver_rng_core;
