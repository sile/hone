use crate::tuners::TunerSpec;

#[derive(Debug, structopt::StructOpt)]
pub struct TunerOpt {
    #[structopt(flatten)]
    pub spec: TunerSpec,
}
