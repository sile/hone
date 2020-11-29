use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct UncheckedInclusiveRange {
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "UncheckedInclusiveRange")]
pub struct InclusiveRange {
    min: FiniteF64,
    max: FiniteF64,
}

impl InclusiveRange {
    pub fn new(min: f64, max: f64) -> anyhow::Result<Self> {
        let min = FiniteF64::new(min)?;
        let max = FiniteF64::new(max)?;
        anyhow::ensure!(
            min.get() <= max.get(),
            "`min`({})  must be smaller than or equal to `max`({})",
            min.get(),
            max.get()
        );
        anyhow::ensure!(
            (max.get() - min.get()).is_finite(),
            "the width of the range {}..{} is not a finite number",
            min.get(),
            max.get()
        );
        Ok(Self { min, max })
    }

    pub const fn min(self) -> FiniteF64 {
        self.min
    }

    pub const fn max(self) -> FiniteF64 {
        self.max
    }

    pub fn width(self) -> FiniteF64 {
        FiniteF64::new(self.max.get() - self.min.get()).expect("unreachable")
    }
}

impl TryFrom<UncheckedInclusiveRange> for InclusiveRange {
    type Error = anyhow::Error;

    fn try_from(from: UncheckedInclusiveRange) -> Result<Self, Self::Error> {
        Self::new(from.min, from.max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "f64")]
pub struct NonNegF64(OrderedFloat<f64>);

impl NonNegF64 {
    pub fn new(x: f64) -> anyhow::Result<Self> {
        anyhow::ensure!(x.is_finite(), "{} isn't a finite number", x);
        anyhow::ensure!(x.is_sign_positive(), "{} isn't a positive number", x);
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
        anyhow::ensure!(x.is_finite(), "{} isn't a finite number", x);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Scope {
    Observation,
    Trial,
    Study,
}

impl Scope {
    pub const CHOICES: &'static [&'static str] = &["observation", "trial", "study"];
}

impl std::str::FromStr for Scope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "observation" => Ok(Self::Observation),
            "trial" => Ok(Self::Trial),
            "study" => Ok(Self::Study),
            _ => anyhow::bail!("unknown scope {:?}", s),
        }
    }
}

/// Elapsed seconds.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ElapsedSeconds(f64);

impl ElapsedSeconds {
    /// Makes a new `ElapsedSeconds` instance.
    pub fn new(seconds: f64) -> Self {
        Self(seconds)
    }

    /// Makes a `ElapsedSeconds` instance that represents the zero elapsed seconds.
    pub const fn zero() -> Self {
        Self(0.0)
    }

    /// Returns the elapsed seconds value.
    pub const fn get(self) -> f64 {
        self.0
    }

    /// Converts the elapsed seconds to `Duration`.
    pub fn to_duration(self) -> Duration {
        Duration::from_secs_f64(self.0)
    }
}

impl From<Duration> for ElapsedSeconds {
    fn from(f: Duration) -> Self {
        Self(f.as_secs_f64())
    }
}
