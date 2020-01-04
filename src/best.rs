use crate::trial::Trial;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct BestOpt {
    // TODO: metrics
}

pub fn pareto_front(_opt: BestOpt, trials: &[Trial]) -> Vec<&Trial> {
    let mut pareto_front = Vec::new();
    for trial in trials {
        if trials.iter().all(|t| trial.dominates(t)) {
            pareto_front.push(trial);
        }
    }
    pareto_front
}
