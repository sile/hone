use crate::param::ParamValue;

#[derive(Debug)]
pub struct Obs {
    pub param: ParamValue,
    pub values: Vec<f64>,
}
