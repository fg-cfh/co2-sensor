use rand_core::RngCore;

pub trait RngDriver {
    fn random(&self, buf: &mut [u8]);

    fn random_u8(&self) -> u8 {
        let buf = [0; 1];
        self.random(&buf);
        buf[0]
    }

    fn random_u16(&self) -> u16 {
        let buf = [0; 2];
        self.random(&buf);
        buf[0] as u16 | (buf[1] as u16) << 8
    }

    fn random_u32(&self) -> u32 {
        let buf = [0; 4];
        self.random(&buf);
        buf[0] as u32 | (buf[1] as u32) << 8 | (buf[2] as u32) << 16 | (buf[3] as u32) << 24
    }

    fn random_u64(&self) -> u64 {
        let buf = [0; 8];
        self.random(&buf);
        buf[0] as u64
            | (buf[1] as u64) << 8
            | (buf[2] as u64) << 16
            | (buf[3] as u64) << 24
            | (buf[4] as u64) << 32
            | (buf[5] as u64) << 40
            | (buf[6] as u64) << 48
            | (buf[7] as u64) << 56
    }

    fn as_rng_core(&self) -> dyn RngCore {
        RngDriverCore { rng_driver: self }
    }
}

struct RngDriverCore<'a> {
    rng_driver: &'a dyn RngDriver,
}

impl<'a> RngCore for RngDriverCore<'a> {
    fn next_u32(&mut self) -> u32 {
        self.rng_driver.random_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng_driver.random_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng_driver.random(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.rng_driver.fill_bytes(dest);
        Ok(())
    }
}
