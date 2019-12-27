#[macro_use]
extern crate trackable;

use hone::param::ParamOpt;
use hone::samplers::{RandomSampler, Sampler as _};
use hone::Error;
use std::io::{self, BufRead as _};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Param(ParamOpt),
    Observe, // TODO: regex, etc
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
        Opt::Observe => {
            let stdin = io::stdin();
            let mut stdin = stdin.lock();
            let mut line = String::new();
            let mut last_line = String::new();
            while 0 != track!(stdin.read_line(&mut line).map_err(Error::from))? {
                print!("{}", line);
                last_line = line.clone();
                line.clear();
            }
            let value: f64 = track!(last_line.trim().parse().map_err(Error::from))?;
            println!("[VALUE]: {}", value); // TODO
        }
    }
    Ok(())
}
