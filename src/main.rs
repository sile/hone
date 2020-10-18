// use hone::config;
// use hone::init;
// use hone::run;
// use hone::trial::Trial;
// use hone::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Hp(hone::hp::HpOpt),
    Run(hone::run::RunOpt),
}
//     Init(init::InitOpt),
//     Run(run::RunOpt),
//     // Define Param|Metric
//     Get(hone::get::GetOpt),
//     Report(hone::report::ReportOpt),
//     Studies(hone::studies::StudiesOpt),
//     Trials(hone::trials::TrialsOpt),
//     Best(hone::best::BestOpt),
//     // Watch,  Stats (n_trials,param_num,elapsed_times), Plot, Save, Load
// }

fn main() -> anyhow::Result<()> {
    hone::rpc::init();

    let opt = Opt::from_args();
    match opt {
        Opt::Hp(opt) => {
            let value = opt.ask()?;
            println!("{}", value);
        }
        Opt::Run(opt) => {
            opt.run()?;
        }
    }
    // match opt {
    //     Opt::Init(opt) => {
    //         let initializer = init::Initializer::new(opt);
    //         track!(initializer.init())?;
    //     }
    //     Opt::Run(opt) => {
    //         let config_path = track!(config::Config::lookup_path())?;
    //         let config = track!(config::Config::load_from_file(config_path))?;
    //         let runner = track!(run::Runner::new(opt, config))?;
    //         track!(runner.run())?;
    //     }
    //     Opt::Get(opt) => {
    //         let getter = track!(hone::get::Getter::new(opt))?;
    //         let value = track!(getter.get())?;
    //         println!("{}", value);
    //     }
    //     Opt::Report(opt) => {
    //         let reporter = track!(hone::report::Reporter::new(opt))?;
    //         track!(reporter.report())?;
    //     }
    //     Opt::Studies(opt) => {
    //         let config_path = track!(config::Config::lookup_path())?;
    //         let config = track!(config::Config::load_from_file(config_path))?;
    //         let studies = track!(hone::studies::list_studies(opt, &config))?;
    //         let json = track!(serde_json::to_string_pretty(&studies).map_err(Error::from))?;
    //         println!("{}", json);
    //     }
    //     Opt::Trials(opt) => {
    //         let config_path = track!(config::Config::lookup_path())?;
    //         let config = track!(config::Config::load_from_file(config_path))?;
    //         let trials = track!(hone::trials::list_trials(opt, &config))?;
    //         let json = track!(serde_json::to_string_pretty(&trials).map_err(Error::from))?;
    //         println!("{}", json);
    //     }
    //     Opt::Best(opt) => {
    //         let stdin = std::io::stdin();
    //         let trials: Vec<Trial> =
    //             track!(serde_json::from_reader(stdin.lock()).map_err(Error::from))?;
    //         let pareto_front = hone::best::pareto_front(opt, &trials);
    //         let json = track!(serde_json::to_string_pretty(&pareto_front).map_err(Error::from))?;
    //         println!("{}", json);
    //     }
    // }
    Ok(())
}
