#[derive(Debug, Clone)]
pub struct Attr {
    pub key: String,
    pub value: String,
}

impl std::str::FromStr for Attr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.splitn(2, ':');
        let key = iter.next().expect("unreachable");
        let value = iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("No value part in an attribute string: {:?}", s))?;
        Ok(Self {
            key: key.to_owned(),
            value: value.to_owned(),
        })
    }
}
