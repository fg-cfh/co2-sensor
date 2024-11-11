use rand_core::RngCore;

macro_rules! impl_rng_core {
    (  $t:ty ) => {
        use rand_core::RngCore;

        impl RngCore for $t {
            fn next_u32(&mut self) -> u32 {
                self.random_u32()
            }

            fn next_u64(&mut self) -> u64 {
                self.random_u64()
            }

            fn fill_bytes(&mut self, dest: &mut [u8]) {
                self.random(dest)
            }

            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
                self.fill_bytes(dest);
                Ok(())
            }
        }
    };
}

pub(in super::super) use impl_rng_core;

pub trait RngDriver: RngCore {
    fn random(&mut self, buf: &mut [u8]);

    fn random_u8(&mut self) -> u8 {
        let mut buf = [0; 1];
        self.random(&mut buf);
        buf[0]
    }

    fn random_u16(&mut self) -> u16 {
        let mut buf = [0; 2];
        self.random(&mut buf);
        buf[0] as u16 | (buf[1] as u16) << 8
    }

    fn random_u32(&mut self) -> u32 {
        let mut buf = [0; 4];
        self.random(&mut buf);
        buf[0] as u32 | (buf[1] as u32) << 8 | (buf[2] as u32) << 16 | (buf[3] as u32) << 24
    }

    fn random_u64(&mut self) -> u64 {
        let mut buf = [0; 8];
        self.random(&mut buf);
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
