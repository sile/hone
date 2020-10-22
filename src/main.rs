use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Hp(hone::hp::HpOpt), // TODO: ask ?
    Run(hone::run::RunOpt),
    Tell(hone::tell::TellOpt),
}

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
        Opt::Tell(opt) => {
            opt.tell()?;
        }
    }
    Ok(())
}
