#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct TellOpt {
    #[structopt(long = "name", short = "n")]
    pub metric_name: Option<String>,

    pub metric_value: f64,

    #[structopt(long)]
    pub maximize: bool,

    #[structopt(long)]
    pub ln: bool,
}

impl TellOpt {
    pub fn tell(&self) -> anyhow::Result<()> {
        todo!()
    }
}
