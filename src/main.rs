#[macro_use]
extern crate trackable;

use hone::init;
use hone::param::ParamOpt;
use hone::samplers::{RandomSampler, Sampler as _};
use std::io;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Init(init::InitOpt),
    Param(ParamOpt),                    // TODO: get or sample etc
    Observe(hone::observe::ObserveOpt), // TODO: regex, etc
}

fn main() -> trackable::result::TopLevelResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Init(opt) => {
            let initializer = init::Initializer::new(opt);
            track!(initializer.init())?;
        }
        Opt::Param(opt) => {
            // TODO: load history
            // TODO: resample or reuse
            let mut sampler = RandomSampler::new();
            let param = opt.to_param();
            let value = track!(sampler.sample(&param, &[]))?;
            println!("{}", param.repr(value));
        }
        Opt::Observe(opt) => {
            let stdin = io::stdin();
            let source = stdin.lock();
            let config = track!(hone::config::Config::load_from_default_file())?;
            let mut observer = hone::observe::Observer::new(source, config, opt);
            track!(observer.observe())?;
        }
    }
    Ok(())
}
