use crate::envvar;
use crate::rpc;

#[derive(Debug, structopt::StructOpt)]
pub struct TellOpt {
    #[structopt(long)]
    pub name: Option<String>,
    #[structopt(long)]
    pub maximize: bool, // TODO: minimize | maximize | nooptimize

    pub value: f64,
}

impl TellOpt {
    pub fn tell(&self) -> anyhow::Result<()> {
        let trial_id = std::env::var(envvar::KEY_TRIAL_ID)?.parse()?;
        let req = rpc::TellReq {
            trial_id,
            value_name: self
                .name
                .clone()
                .unwrap_or_else(|| "objective value".to_string()),
            minimize: !self.maximize,
            value: self.value,
        };
        let _ = rpc::call::<rpc::TellRpc>(req)??;
        Ok(())
    }
}
