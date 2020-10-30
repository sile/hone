use crate::domain::{ObjectiveType, ObjectiveValue};
use crate::envvar;
use crate::rpc;

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct TellOpt {
    #[structopt(long = "name", short = "n", default_value = "objective value")]
    pub metric_name: String,

    pub metric_value: f64,

    #[structopt(long)]
    pub maximize: bool,

    #[structopt(long)]
    pub ignore: bool,
}

impl TellOpt {
    pub fn tell(&self) -> anyhow::Result<()> {
        let run_id = envvar::get_run_id()?;
        let req = rpc::TellReq {
            run_id,
            objective_name: self.metric_name.clone(),
            objective_type: ObjectiveType {
                minimize: !self.maximize,
                ignore: self.ignore,
            },
            objective_value: ObjectiveValue::new(self.metric_value),
        };
        rpc::call::<rpc::TellRpc>(req)??;
        Ok(())
    }
}
