#[macro_use]
extern crate trackable;

use hone::param::ParamOpt;
use hone::samplers::{RandomSampler, Sampler as _};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Param(ParamOpt),
}

fn main() -> trackable::result::TopLevelResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Param(opt) => {
            // TODO: load history
            // TODO: resample or reuse
            let mut sampler = RandomSampler::new();
            let param = opt.to_param();
            let value = track!(sampler.sample(&param, &[]))?;
            println!("{}", param.repr(value));
        }
    }
    Ok(())
}
