use serde::Deserialize;

pub fn parse_json<T>(json: &str) -> anyhow::Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let v = serde_json::from_str(json)?;
    Ok(v)
}
