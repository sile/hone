use crate::envvar;
use crate::rpc;
use crate::types::Scope;

#[derive(Debug, clap::Subcommand)]
pub enum GetOpt {
    Id {
        #[clap(long, short='s', default_value= Scope::CHOICES[0])]
        scope: Scope,
    },
    Tempdir {
        #[clap(long)]
        parent: Option<std::path::PathBuf>,

        #[clap(long, short='s', default_value= Scope::CHOICES[0])]
        scope: Scope,
    },
}

impl GetOpt {
    pub fn get(&self) -> anyhow::Result<String> {
        let value = match self {
            Self::Id { scope } => match scope {
                Scope::Observation => envvar::get_string(envvar::KEY_OBSERVATION_ID)?,
                Scope::Trial => envvar::get_string(envvar::KEY_TRIAL_ID)?,
                Scope::Study => envvar::get_string(envvar::KEY_STUDY_ID)?,
            },
            Self::Tempdir { scope, parent } => {
                let observation_id = envvar::get_observation_id()?;
                let req = rpc::MktempReq {
                    observation_id,
                    parent: parent.clone(),
                    scope: *scope,
                };
                let res = rpc::call::<rpc::MktempRpc>(req)?;
                res.to_str()
                    .ok_or_else(|| anyhow::anyhow!("invalid path: {:?}", res))?
                    .to_owned()
            }
        };
        Ok(value)
    }
}
