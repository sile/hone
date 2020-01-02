#[macro_use]
extern crate trackable;

use hone::config;
use hone::init;
use hone::run;
use structopt::StructOpt;
// use hone::param::ParamOpt;
// use hone::samplers::{RandomSampler, Sampler as _};
// use std::io;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Init(init::InitOpt),
    Run(run::RunOpt),
    //  Get(Sample,Suggest), Report, Watch, Show, Studies, Trials

    // Get(ParamOpt), // or Sample
    // Config(..), get|set
    // Prune(..) or EarlyStop
    // Report {step: ..., values: [{"name": "foo", "minimize":true, value:10}]} | float
    // New Study|Thread
    // Switch Study|Thread
    // Observe(hone::observe::ObserveOpt), // TODO: regex, etc
}

fn main() -> trackable::result::TopLevelResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Init(opt) => {
            let initializer = init::Initializer::new(opt);
            track!(initializer.init())?;
        }
        Opt::Run(opt) => {
            let config_path = track!(config::Config::lookup_path())?;
            let config = track!(config::Config::load_from_file(config_path))?;
            let runner = track!(run::Runner::new(opt, config))?;
            track!(runner.run())?;
        } // Opt::Get(_opt) => {
          //     // let config = track!(hone::config::Config::load_from_default_file())?;

          //     // // TODO: load history
          //     // // TODO: resample or reuse
          //     // let mut sampler = RandomSampler::new();
          //     // let param = opt.to_param();
          //     // let value = track!(sampler.sample(&param, &[]))?;
          //     // println!("{}", param.repr(value));
          // }
          // Opt::Observe(_opt) => {
          //     // let stdin = io::stdin();
          //     // let source = stdin.lock();
          //     // let config = track!(hone::config::Config::load_from_default_file())?;
          //     // let mut observer = hone::observe::Observer::new(source, config, opt);
          //     // track!(observer.observe())?;
          // }
    }
    Ok(())
}
