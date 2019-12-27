use structopt::StructOpt;

pub type ParamValue = f64;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub distribution: Distribution,
    pub range: Range,
}

impl Param {
    pub fn repr(&self, value: f64) -> String {
        self.range.repr(value)
    }
}

#[derive(Debug, Clone)]
pub enum Distribution {
    Uniform,
    LogUniform { base: f64 },
}

#[derive(Debug, Clone)]
pub enum Range {
    Continuous { low: f64, high: f64 },
    Discrete { low: i64, high: i64 },
    Categorical { choices: Vec<String> },
}

impl Range {
    pub fn low(&self) -> f64 {
        match self {
            Self::Continuous { low, .. } => *low,
            Self::Discrete { low, .. } => *low as f64,
            Self::Categorical { .. } => 0.0,
        }
    }

    pub fn high(&self) -> f64 {
        match self {
            Self::Continuous { high, .. } => *high,
            Self::Discrete { high, .. } => *high as f64,
            Self::Categorical { choices } => choices.len() as f64,
        }
    }

    fn repr(&self, value: f64) -> String {
        match self {
            Self::Continuous { .. } => value.to_string(),
            Self::Discrete { .. } => (value as i64).to_string(),
            Self::Categorical { choices } => choices[value as usize].clone(),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ParamOpt {
    pub name: String,

    #[structopt(long)]
    pub ln: bool,

    #[structopt(long)]
    pub log2: bool,

    #[structopt(long)]
    pub log10: bool,

    #[structopt(long)]
    pub resample: bool,

    #[structopt(subcommand)]
    pub value: ValueOpt,
}

impl ParamOpt {
    pub fn to_param(&self) -> Param {
        Param {
            name: self.name.clone(),
            distribution: Distribution::Uniform, // TODO
            range: self.value.to_range(),
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum ValueOpt {
    Float { low: f64, high: f64 }, // TODO: fix field, ln flag
    Int { low: i64, high: i64 },   // TODO: fix field
    // TODO: Seq(Vec<f64>)
    // TODO: Num(low,high,ln,round)
    Choice { choices: Vec<String> }, // TODO: fix field
}

impl ValueOpt {
    fn to_range(&self) -> Range {
        match self.clone() {
            Self::Float { low, high } => Range::Continuous { low, high },
            Self::Int { low, high } => Range::Discrete { low, high },
            Self::Choice { choices } => Range::Categorical { choices },
        }
    }
}
