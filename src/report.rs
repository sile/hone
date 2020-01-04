use crate::metric::{Direction, Metric};
use crate::study::StudyClient;
use crate::Result;
use std::num::NonZeroU64;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ReportOpt {
    pub value: f64,

    #[structopt(long)]
    pub name: Option<String>,

    #[structopt(long)]
    pub step: Option<NonZeroU64>,

    // pub 異なるステップの値同士が比較可能: bool
    #[structopt(long, default_value = "MINIMIZE")]
    pub direction: Direction,
}

impl ReportOpt {
    pub fn to_metric(&self) -> Metric {
        Metric {
            name: self
                .name
                .clone()
                .unwrap_or_else(|| "objective value".to_owned()),
            value: self.value,
            direction: self.direction,
        }
    }
}

#[derive(Debug)]
pub struct Reporter {
    opt: ReportOpt,
    client: StudyClient,
}

impl Reporter {
    pub fn new(opt: ReportOpt) -> Result<Self> {
        let client = track!(StudyClient::new())?;
        Ok(Self { opt, client })
    }

    pub fn report(&self) -> Result<()> {
        let metric = self.opt.to_metric();
        track!(self.client.report(self.opt.step, metric))
    }
}
