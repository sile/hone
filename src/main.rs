use clap::Parser;

#[derive(Parser)]
enum Opt {
    Ask(hone::commands::ask::AskOpt),
    #[clap(subcommand)]
    Get(hone::commands::get::GetOpt),
    Run(hone::commands::run::RunOpt),
    #[clap(subcommand)]
    Show(hone::commands::show::ShowOpt),
    #[clap(subcommand)]
    Tell(hone::commands::tell::TellOpt),
    Tuner(hone::commands::tuner::TunerOpt),
}

fn main() -> anyhow::Result<()> {
    hone::rpc::init();

    let opt = Opt::parse();
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
