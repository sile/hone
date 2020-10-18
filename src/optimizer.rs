#[derive(Debug, Clone)]
pub struct SearchSpace {}

#[derive(Debug, Clone)]
pub struct PartialSearchSpace {
    // pub asked_params: Vec<usize>,
// pub fixed_params: Vec<(usize, f64)>,
}

#[derive(Debug, Clone)]
pub enum Distribution {
    Continuous { size: f64 },
    Discrete { size: usize },
    Categorical { size: usize },
}

#[derive(Debug, Clone)]
pub struct EvaluatedTrial {
    pub trial_id: usize,
    pub params: Vec<f64>,
    pub values: Option<Vec<f64>>, // `None` means it's a failed trial
    pub fidelity: f64,
}

pub trait Optimizer {
    fn ask(
        &mut self,
        trial_id: usize,
        param: usize,
        distribution: Distribution,
    ) -> anyhow::Result<f64>;

    fn tell(&mut self, trial: &EvaluatedTrial) -> anyhow::Result<()>;

    fn initialize(&mut self, search_space: &SearchSpace) -> anyhow::Result<()>;

    // fn ask(&mut self, trial_id: Uuid, param: &Param) -> Result<ParamValue>;
    // fn tell(&mut self, trial_id: Uuid, action: &TrialAction) -> Result<()>;
    // fn prune(&mut self, trial_id: Uuid) -> Result<bool>;
}

// use crate::param::{Param, ParamSpec, ParamValue};
// use crate::pubsub::TrialAction;
// use crate::{ErrorKind, Result};
// use rand::rngs::StdRng;
// use rand::seq::SliceRandom as _;
// use rand::SeedableRng as _;
// use std::fmt;
// use uuid::Uuid;

// pub trait Optimizer {
//     fn ask(&mut self, trial_id: Uuid, param: &Param) -> Result<ParamValue>;
//     fn tell(&mut self, trial_id: Uuid, action: &TrialAction) -> Result<()>;
//     fn prune(&mut self, trial_id: Uuid) -> Result<bool>;
// }

// pub struct BoxOptimizer(Box<dyn 'static + Optimizer + Send>);

// impl BoxOptimizer {
//     pub fn new<T>(optimizer: T) -> Self
//     where
//         T: 'static + Optimizer + Send,
//     {
//         Self(Box::new(optimizer))
//     }
// }

// impl fmt::Debug for BoxOptimizer {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "BoxOptimizer(..)")
//     }
// }

// impl Optimizer for BoxOptimizer {
//     fn ask(&mut self, trial_id: Uuid, param: &Param) -> Result<ParamValue> {
//         (*self.0).ask(trial_id, param)
//     }

//     fn tell(&mut self, trial_id: Uuid, action: &TrialAction) -> Result<()> {
//         (*self.0).tell(trial_id, action)
//     }

//     fn prune(&mut self, trial_id: Uuid) -> Result<bool> {
//         (*self.0).prune(trial_id)
//     }
// }

// #[derive(Debug)]
// pub struct RandomOptimizer {
//     rng: StdRng,
// }

// impl RandomOptimizer {
//     pub fn new() -> Self {
//         Self::with_seed(rand::random())
//     }

//     pub fn with_seed(seed: u64) -> Self {
//         let mut seed256 = [0; 32];
//         (&mut seed256[0..8]).copy_from_slice(&seed.to_be_bytes());
//         let rng = StdRng::from_seed(seed256);
//         Self { rng }
//     }
// }

// impl Optimizer for RandomOptimizer {
//     fn ask(&mut self, _trial_id: Uuid, param: &Param) -> Result<ParamValue> {
//         let ParamSpec::Choice { choices } = &param.spec;
//         let value = track_assert_some!(choices.choose(&mut self.rng), ErrorKind::InvalidInput);
//         Ok(ParamValue(value.clone()))
//     }

//     fn tell(&mut self, _trial_id: Uuid, _action: &TrialAction) -> Result<()> {
//         Ok(())
//     }

//     fn prune(&mut self, _trial_id: Uuid) -> Result<bool> {
//         Ok(false)
//     }
// }
