use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::sync::{Mutex, OnceLock};

pub struct RandomGenerator;
static RNG: OnceLock<Mutex<ChaCha12Rng>> = OnceLock::new();

impl RandomGenerator {
    pub fn random_num(start: u32, end: u32) -> u32 {
        let rng_mutex = RNG.get_or_init(|| Mutex::new(ChaCha12Rng::from_entropy()));
        let mut rng = rng_mutex.lock().expect("Failed to lock RNG mutex");
        rng.gen_range(start..=end)
    }

    pub fn rand_u32() -> u32 {
        Self::random_num(0, u32::MAX)
    }
}
