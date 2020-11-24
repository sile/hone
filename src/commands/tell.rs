use crate::envvar;
use crate::metric::{MetricName, MetricType, MetricValue};
use crate::rpc;

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum TellOpt {
    Minimize {
        #[structopt(long, short = "n", default_value = "objective value")]
        name: String,
        value: f64,
    },
    Maximize {
        #[structopt(long, short = "n", default_value = "objective value")]
        name: String,
        value: f64,
    },
    Record {
        #[structopt(long, short = "n")]
        name: String,
        value: f64,
    },
    Judge {
        #[structopt(long, short = "n", default_value = "judge")]
        name: String,
        feasibility: f64,
    },
}

impl TellOpt {
    pub fn tell(&self) -> anyhow::Result<()> {
        let observation_id = envvar::get_observation_id()?;
        let (name, ty, value) = match self {
            Self::Minimize { name, value } => (name, MetricType::Minimize, value),
            Self::Maximize { name, value } => (name, MetricType::Maximize, value),
            Self::Record { name, value } => (name, MetricType::Record, value),
            Self::Judge { name, feasibility } => (name, MetricType::Judge, feasibility),
        };
        let req = rpc::TellReq {
            observation_id,
            metric_name: MetricName::new(name.clone()),
            metric_type: ty,
            metric_value: MetricValue::new(*value)?,
        };
        rpc::call::<rpc::TellRpc>(req)?;
        Ok(())
    }
}
