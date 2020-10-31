use ordered_float::OrderedFloat;
use rand::distributions::Distribution;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "Vec<T>")]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T> NonEmptyVec<T> {
    pub fn new(inner: Vec<T>) -> anyhow::Result<Self> {
        anyhow::ensure!(!inner.is_empty(), "empty vector isn't allowed");
        Ok(Self(inner))
    }

    pub fn get(&self) -> &[T] {
        &self.0
    }
}

impl<T> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = anyhow::Error;

    fn try_from(from: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(from)
    }
}

impl<T: Clone> Distribution<T> for NonEmptyVec<T> {
    fn sample<R: ?Sized + Rng>(&self, rng: &mut R) -> T {
        self.0.choose(rng).expect("unreachable").clone()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct UncheckedInclusiveRange {
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "UncheckedInclusiveRange")]
pub struct InclusiveRange {
    min: f64,
    max: f64,
}

impl InclusiveRange {
    pub fn new(min: f64, max: f64) -> anyhow::Result<Self> {
        anyhow::ensure!(min.is_finite(), "`min`({}) isn't an finite number", min);
        anyhow::ensure!(max.is_finite(), "`max`({}) isn't an finite number", max);
        anyhow::ensure!(
            min <= max,
            "`min`({})  must be smaller than or equal to `max`({})",
            min,
            max
        );
        Ok(Self { min, max })
    }

    pub const fn min(self) -> f64 {
        self.min
    }

    pub const fn max(self) -> f64 {
        self.max
    }

    pub fn width(self) -> f64 {
        self.max - self.min
    }
}

impl TryFrom<UncheckedInclusiveRange> for InclusiveRange {
    type Error = anyhow::Error;

    fn try_from(from: UncheckedInclusiveRange) -> Result<Self, Self::Error> {
        Self::new(from.min, from.max)
    }
}

impl Distribution<f64> for InclusiveRange {
    fn sample<R: ?Sized + Rng>(&self, rng: &mut R) -> f64 {
        rng.gen_range(self.min, self.max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "f64")]
pub struct NonNegF64(OrderedFloat<f64>);

impl NonNegF64 {
    pub fn new(x: f64) -> anyhow::Result<Self> {
        anyhow::ensure!(x.is_finite(), "`x`({}) isn't a finite number", x);
        anyhow::ensure!(x.is_sign_positive(), "`x`({}) isn't a positive number", x);
        Ok(Self(OrderedFloat(x)))
    }

    pub const fn get(self) -> f64 {
        (self.0).0
    }
}

impl TryFrom<f64> for NonNegF64 {
    type Error = anyhow::Error;

    fn try_from(from: f64) -> Result<Self, Self::Error> {
        Self::new(from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "f64")]
pub struct FiniteF64(OrderedFloat<f64>);

impl FiniteF64 {
    pub fn new(x: f64) -> anyhow::Result<Self> {
        anyhow::ensure!(x.is_finite(), "`x`({}) isn't a finite number", x);
        Ok(Self(OrderedFloat(x)))
    }

    pub const fn get(self) -> f64 {
        (self.0).0
    }
}

impl TryFrom<f64> for FiniteF64 {
    type Error = anyhow::Error;

    fn try_from(from: f64) -> Result<Self, Self::Error> {
        Self::new(from)
    }
}
