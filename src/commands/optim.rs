use crate::optimizer::OptimizerSpec;

#[derive(Debug, structopt::StructOpt)]
pub struct OptimOpt {
    #[structopt(flatten)]
    pub spec: OptimizerSpec,
}
