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
    // Define
    Get(hone::get::GetOpt),
    Report(hone::report::ReportOpt),
    // Watch, Show, Studies, Trials
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
        }
        Opt::Get(opt) => {
            let getter = track!(hone::get::Getter::new(opt))?;
            let value = track!(getter.get())?;
            println!("{}", value);
        }
        Opt::Report(opt) => {
            let reporter = track!(hone::report::Reporter::new(opt))?;
            track!(reporter.report())?;
        }
    }
    Ok(())
}
