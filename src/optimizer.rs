use crate::rng::ArcRng;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct SearchSpace {
    pub params: Vec<Distribution>,
}

pub type ParamIndex = usize;

#[derive(Debug, Clone, Copy)]
pub enum Distribution {
    Continuous { size: f64 },
    Discrete { size: usize },
    Categorical { size: usize },
    Fidelity,
}

impl Distribution {
    pub fn size(&self) -> f64 {
        match self {
            Self::Continuous { size } => *size,
            Self::Discrete { size } => *size as f64,
            Self::Categorical { size } => *size as f64,
            Self::Fidelity => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrialId(pub usize);

#[derive(Debug, Clone)]
pub struct EvaluatedTrial {
    pub trial_id: TrialId,
    pub params: Vec<f64>,
    pub values: Option<Vec<f64>>, // `None` means it's a failed trial
}

pub trait Optimizer {
    fn initialize(&mut self, search_space: &SearchSpace) -> anyhow::Result<()>;

    fn ask(
        &mut self,
        trial_id: TrialId,
        param: ParamIndex,
        distribution: Distribution,
    ) -> anyhow::Result<f64>;

    fn tell(&mut self, trial: &EvaluatedTrial) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct RandomOptimizer {
    rng: ArcRng,
}

impl RandomOptimizer {
    pub fn new(rng: ArcRng) -> Self {
        Self { rng }
    }
}

impl Optimizer for RandomOptimizer {
    fn initialize(&mut self, _search_space: &SearchSpace) -> anyhow::Result<()> {
        Ok(())
    }

    fn ask(
        &mut self,
        _trial_id: TrialId,
        _param: ParamIndex,
        distribution: Distribution,
    ) -> anyhow::Result<f64> {
        if let Distribution::Fidelity = distribution {
            Ok(1.0)
        } else {
            Ok(self.rng.gen_range(0.0, distribution.size()))
        }
    }

    fn tell(&mut self, _trial: &EvaluatedTrial) -> anyhow::Result<()> {
        Ok(())
    }
}

// #[derive(Debug)]
// pub struct TpeOptimizer {
//     random: RandomOptimizer,
//     rng: ArcRng,
// }

// impl TpeOptimizer {
//     pub fn new(rng: ArcRng) -> Self {
//         Self {
//             rng,
//             inners: Vec::new(),
//         }
//     }
// }

// impl Optimizer for TpeOptimizer {
//     fn initialize(&mut self, search_space: &SearchSpace) -> anyhow::Result<()> {
//         self.inners = search_space
//             .params
//             .iter()
//             .map(|d| {
//                 let estimator = if let Distribution::Categorical { .. } = d {
//                     tpe::histogram_estimator()
//                 } else {
//                     tpe::parzen_estimator()
//                 };
//                 Ok(tpe::TpeOptimizer::new(
//                     estimator,
//                     tpe::range::Range::new(0.0, d.size())?,
//                 ))
//             })
//             .collect::<Result<_, tpe::range::RangeError>>()?;
//         Ok(())
//     }

//     fn ask_params(
//         &mut self,
//         _trial_id: TrialId,
//         search_space: &PartialSearchSpace,
//     ) -> anyhow::Result<PartialParams> {
//         let mut params = BTreeMap::new();
//         for (i, d) in search_space.params.iter() {}
//         let params = search_space
//             .params
//             .iter()
//             .map(|(i, _)| (*i, self.inners[*i].ask(&mut self.rng).expect("unreachable")))
//             .collect();
//         Ok(PartialParams(params))
//     }

//     fn tell(&mut self, trial: &EvaluatedTrial) -> anyhow::Result<()> {
//         let value = if let Some(values) = &trial.values {
//             ensure!(values.len() == 1, "TODO");
//             values[0]
//         } else {
//             std::f64::INFINITY // Penalty value.
//         };

//         for (p, o) in trial.params.iter().zip(self.inners.iter_mut()) {
//             o.tell(*p, value)?;
//         }

//         Ok(())
//     }
// }
