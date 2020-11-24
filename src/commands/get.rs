use crate::envvar;
use crate::rpc;
use crate::types::Scope;

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum GetOpt {
    Id {
        #[structopt(long, short="s", default_value= Scope::CHOICES[0], possible_values = Scope::CHOICES)]
        scope: Scope,
    },
    Tempdir {
        #[structopt(long)]
        parent: Option<std::path::PathBuf>,

        #[structopt(long, short="s", default_value= Scope::CHOICES[0], possible_values = Scope::CHOICES)]
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
