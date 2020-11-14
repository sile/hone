use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Ask(hone::commands::ask::AskOpt),
    Tell(hone::commands::tell::TellOpt),
    Run(hone::commands::run::RunOpt),
    Tuner(hone::commands::tuner::TunerOpt),
    Get(hone::commands::get::GetOpt),
    Show(hone::commands::show::ShowOpt),
    // TODO: Plot, Watch, Import, Export
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
        Opt::Tuner(opt) => {
            serde_json::to_writer(std::io::stdout().lock(), &opt.spec)?;
            println!();
        }
        Opt::Get(opt) => {
            let value = opt.get()?;
            println!("{}", value);
        }
        Opt::Show(opt) => {
            opt.show()?;
        }
    }
    Ok(())
}
