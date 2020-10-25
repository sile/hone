use crate::hp;
use crate::rng::ArcRng;
use crate::trial::{ObservationId, TrialId};
use rand::Rng;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ValueDomain {
    pub minimize: bool,
}

#[derive(Debug, Clone)]
pub struct ObjectiveSpace {
    pub values: BTreeMap<String, ValueDomain>,
}

impl ObjectiveSpace {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn index(&self, name: &str) -> Option<usize> {
        self.values.iter().position(|x| x.0 == name)
    }

    pub fn expand_if_need(&mut self, name: &str, value: &ValueDomain) -> anyhow::Result<bool> {
        if let Some(x) = self.values.get_mut(name) {
            anyhow::ensure!(x.minimize == value.minimize, "TODO");
            Ok(false)
        } else {
            self.values.insert(name.to_owned(), value.clone());
            Ok(true)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchSpace {
    pub params: BTreeMap<String, hp::HpDistribution>,
}

impl SearchSpace {
    pub fn new() -> Self {
        Self {
            params: BTreeMap::new(),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }

    pub fn index(&self, name: &str) -> Option<usize> {
        self.params.iter().position(|x| x.0 == name)
    }

    pub fn expand_if_need(&mut self, name: &str, d: &hp::HpDistribution) -> anyhow::Result<bool> {
        if let Some(x) = self.params.get_mut(name) {
            x.expand_if_need(d)
        } else {
            self.params.insert(name.to_owned(), d.clone());
            Ok(true)
        }
    }

    pub fn unwarp(&self, name: &str, value: f64) -> anyhow::Result<hp::HpValue> {
        let d = self
            .params
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("TOOD"))?;
        d.unwarp(value)
    }
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

impl From<hp::HpDistribution> for Distribution {
    fn from(f: hp::HpDistribution) -> Self {
        match f {
            hp::HpDistribution::Flag => Self::Categorical { size: 2 },
            hp::HpDistribution::Choice {
                choices,
                ordinal: false,
            } => Self::Categorical {
                size: choices.len(),
            },
            hp::HpDistribution::Choice {
                choices,
                ordinal: true,
            } => Self::Discrete {
                size: choices.len(),
            },
            hp::HpDistribution::Normal { .. } => Self::Continuous { size: 1.0 },
            hp::HpDistribution::Range {
                start,
                end,
                ln: false,
                step: None,
                fidelity: false,
            } => Self::Continuous { size: end - start },
            hp::HpDistribution::Range {
                start,
                end,
                ln: true,
                step: None,
                fidelity: false,
            } => Self::Continuous {
                size: end.ln() - start.ln(),
            },
            hp::HpDistribution::Range {
                start,
                end,
                ln: false,
                step: Some(n),
                fidelity: false,
            } => Self::Discrete {
                size: ((end - start) / n) as usize,
            },
            hp::HpDistribution::Range {
                start,
                end,
                ln: true,
                step: Some(n),
                fidelity: false,
            } => Self::Discrete {
                size: ((end.ln() - start.ln()) / n) as usize,
            },
            hp::HpDistribution::Range { fidelity: true, .. } => Self::Continuous { size: 1.0 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub id: ObservationId,
    pub params: Vec<f64>,
    pub values: Option<Vec<f64>>, // `None` means it's a failed trial
}

pub trait Optimizer {
    fn initialize(
        &mut self,
        search_space: &SearchSpace,
        objective_space: &ObjectiveSpace,
    ) -> anyhow::Result<()>;

    fn next_trial(&mut self) -> Option<TrialId> {
        None
    }

    fn ask(
        &mut self,
        obs_id: ObservationId,
        param: ParamIndex,
        distribution: Distribution,
    ) -> anyhow::Result<f64>;

    fn tell(&mut self, obs: &Observation) -> anyhow::Result<()>;
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
    fn initialize(
        &mut self,
        _search_space: &SearchSpace,
        _objective_space: &ObjectiveSpace,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn ask(
        &mut self,
        _obs_id: ObservationId,
        _param: ParamIndex,
        distribution: Distribution,
    ) -> anyhow::Result<f64> {
        if let Distribution::Fidelity = distribution {
            Ok(1.0)
        } else {
            Ok(self.rng.gen_range(0.0, distribution.size()))
        }
    }

    fn tell(&mut self, _obs: &Observation) -> anyhow::Result<()> {
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
