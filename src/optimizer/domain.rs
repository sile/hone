use ordered_float::OrderedFloat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParamNo(usize);

impl ParamNo {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParamValue(OrderedFloat<f64>);

impl ParamValue {
    pub fn new(value: f64) -> anyhow::Result<Self> {
        anyhow::ensure!(value.is_finite() || value.is_nan(), "TODO");
        Ok(Self(OrderedFloat(value)))
    }

    pub const fn get(self) -> f64 {
        (self.0).0
    }

    pub fn is_asked(self) -> bool {
        !self.get().is_nan()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct ObjectiveValue(OrderedFloat<f64>);

impl ObjectiveValue {
    pub const fn new(value: f64) -> Self {
        Self(OrderedFloat(value))
    }

    pub const fn get(self) -> f64 {
        (self.0).0
    }

    pub fn is_told(self) -> bool {
        !self.get().is_nan()
    }
}

#[derive(Debug, Clone)]
pub struct SearchSpace {
    param_types: Vec<ParamType>,
}

impl SearchSpace {
    pub fn new() -> Self {
        Self {
            param_types: Vec::new(),
        }
    }

    pub fn get_param_type(&self, param_no: ParamNo) -> anyhow::Result<ParamType> {
        self.param_types
            .get(param_no.get())
            .copied()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "parameter number {} is out of range (must be less than {})",
                    param_no.get(),
                    self.param_types.len()
                )
            })
    }

    pub fn param_types(&self) -> &[ParamType] {
        &self.param_types
    }

    pub fn dimensions(&self) -> usize {
        self.param_types.len()
    }
}

#[derive(Debug)]
pub struct ObjectiveSpace {
    objectives: usize,
}

impl ObjectiveSpace {
    pub fn new(objectives: usize) -> Self {
        Self { objectives }
    }

    pub fn dimensions(&self) -> usize {
        self.objectives
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParamType {
    Continuous { size: f64 },
    Discrete { size: usize },
    Categorical { size: usize },
    Fidelity,
}
