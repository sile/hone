#[derive(Debug)]
pub struct SearchDomain {
    params: Vec<ParamDef>,
}

impl SearchDomain {
    pub fn new(params: Vec<ParamDef>) -> Self {
        Self { params }
    }

    pub fn params(&self) -> &[ParamDef] {
        &self.params
    }

    pub fn dimensions(&self) -> usize {
        self.params.len()
    }
}

#[derive(Debug)]
pub struct ObjectiveDomain {
    objectives: usize,
}

impl ObjectiveDomain {
    pub fn new(objectives: usize) -> Self {
        Self { objectives }
    }

    pub fn dimensions(&self) -> usize {
        self.objectives
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParamDef {
    Continuous { size: f64 },
    Discrete { size: usize },
    Categorical { size: usize },
    Fidelity,
}
