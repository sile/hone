use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Ask(hone::commands::ask::AskOpt),
    Tell(hone::commands::tell::TellOpt),
    Run(hone::commands::run::RunOpt),
    // TODO: Optim
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
    }
    Ok(())
}
