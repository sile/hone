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
    Get(ParamOpt), // or Sample
    // Config(..), get|set
    // Prune(..) or EarlyStop
    // Report {step: ..., values: [{"name": "foo", "minimize":true, value:10}]} | float
    // New Study|Thread
    // Switch Study|Thread
    Observe(hone::observe::ObserveOpt), // TODO: regex, etc
}

fn main() -> trackable::result::TopLevelResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Init(opt) => {
            let initializer = init::Initializer::new(opt);
            track!(initializer.init())?;
        }
        Opt::Get(_opt) => {
            // let config = track!(hone::config::Config::load_from_default_file())?;

            // // TODO: load history
            // // TODO: resample or reuse
            // let mut sampler = RandomSampler::new();
            // let param = opt.to_param();
            // let value = track!(sampler.sample(&param, &[]))?;
            // println!("{}", param.repr(value));
        }
        Opt::Observe(_opt) => {
            // let stdin = io::stdin();
            // let source = stdin.lock();
            // let config = track!(hone::config::Config::load_from_default_file())?;
            // let mut observer = hone::observe::Observer::new(source, config, opt);
            // track!(observer.observe())?;
        }
    }
    Ok(())
}
