use crate::tuners::TunerSpec;

#[derive(Debug, clap::Args)]
pub struct TunerOpt {
    #[clap(flatten)]
    pub spec: TunerSpec,
}
