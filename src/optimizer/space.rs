#[derive(Debug)]
pub struct SearchSpace {
    param_types: Vec<ParamType>,
}

impl SearchSpace {
    pub fn new(param_types: Vec<ParamType>) -> Self {
        Self { param_types }
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

pub type ParamIndex = usize;
