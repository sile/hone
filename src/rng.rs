//! **R**andom **N**number **G**enerator.
use rand::rngs::StdRng;
use rand::{Error, RngCore, SeedableRng};
use std::sync::{Arc, Mutex};

/// The random number generator for `kurobako`.
#[derive(Debug, Clone)]
pub struct ArcRng(Arc<Mutex<StdRng>>);
impl ArcRng {
    /// Makes a new `ArcRng` with the given random seed.
    pub fn new(seed: RngSeed) -> Self {
        let mut seed256 = [0; 32];
        (&mut seed256[0..8]).copy_from_slice(&seed.0.to_be_bytes());

        let inner = StdRng::from_seed(seed256);
        Self(Arc::new(Mutex::new(inner)))
    }
}

impl RngCore for ArcRng {
    fn next_u32(&mut self) -> u32 {
        self.0.lock().unwrap_or_else(|e| panic!("{}", e)).next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.lock().unwrap_or_else(|e| panic!("{}", e)).next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0
            .lock()
            .unwrap_or_else(|e| panic!("{}", e))
            .fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.0
            .lock()
            .unwrap_or_else(|e| panic!("{}", e))
            .try_fill_bytes(dest)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct RngSeed(u64);

impl Default for RngSeed {
    fn default() -> Self {
        Self(rand::random())
    }
}

impl std::str::FromStr for RngSeed {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl std::fmt::Display for RngSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
