use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Ask(hone::commands::ask::AskOpt),
    Tell(hone::commands::tell::TellOpt),
    Run(hone::commands::run::RunOpt),
    // TODO: Tuner or Tune
    Optim(hone::commands::optim::OptimOpt),
    Get(hone::commands::get::GetOpt),
    // Summarize or digest or analyze or show
    // Plot
    // Watch
    // Import
    // Export
}

fn main() -> anyhow::Result<()> {
    hone::rpc::init();

    let opt = Opt::from_args();
    match opt {
        Opt::Ask(opt) => {
            let value = opt.ask()?;
            println!("{}", value);
        }
        Opt::Tell(opt) => {
            opt.tell()?;
        }
        Opt::Run(opt) => {
            opt.run()?;
        }
        Opt::Optim(opt) => {
            serde_json::to_writer(std::io::stdout().lock(), &opt.spec)?;
            println!();
        }
        Opt::Get(opt) => {
            let value = opt.get()?;
            println!("{}", value);
        }
    }
    Ok(())
}
