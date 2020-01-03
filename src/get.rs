use crate::param::{Param, ParamSpec, ParamValue};
use crate::study::StudyClient;
use crate::{ErrorKind, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct GetOpt {
    pub param_name: String,

    #[structopt(subcommand)]
    pub param_spec: Option<ParamSpec>,
}

#[derive(Debug)]
pub struct Getter {
    opt: GetOpt,
    client: StudyClient,
}

impl Getter {
    pub fn new(opt: GetOpt) -> Result<Self> {
        let client = track!(StudyClient::new())?;
        Ok(Self { opt, client })
    }

    pub fn get(&self) -> Result<ParamValue> {
        let spec = track_assert_some!(self.opt.param_spec.clone(), ErrorKind::Other, "TOOD");
        let param = Param {
            name: self.opt.param_name.clone(),
            spec,
        };
        track!(self.client.suggest(param))
    }
}
