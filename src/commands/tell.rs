use crate::envvar;
use crate::metric::{MetricName, MetricType, MetricValue, Objective};
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
            metric_name: MetricName::new(self.metric_name.clone()),
            metric_type: MetricType {
                objective: if self.ignore {
                    None
                } else if self.maximize {
                    Some(Objective::Maximize)
                } else {
                    Some(Objective::Minimize)
                },
            },
            metric_value: MetricValue::new(self.metric_value)?,
        };
        rpc::call::<rpc::TellRpc>(req)?;
        Ok(())
    }
}
