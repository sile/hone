use crate::obs::Obs;
use crate::param::{Distribution, Param, ParamValue};
use crate::Result;
use rand::{self, Rng};

pub trait Sampler {
    fn sample(&mut self, param: &Param, history: &[Obs]) -> Result<ParamValue>;
}

#[derive(Debug)]
pub struct RandomSampler {}

impl RandomSampler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Sampler for RandomSampler {
    fn sample(&mut self, param: &Param, _history: &[Obs]) -> Result<ParamValue> {
        if let Distribution::LogUniform { .. } = param.distribution {
            todo!();
        }

        let low = param.range.low();
        let high = param.range.high();
        Ok(rand::thread_rng().gen_range(low, high))
    }
}
