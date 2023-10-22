use crate::event::{Event, EventReader, ObservationEvent, StudyEvent};
use crate::study::StudySpec;
use crate::trial::CompactObservation;
use std::collections::BTreeMap;

#[derive(Debug, clap::Subcommand)]
pub enum ShowOpt {
    Best(BestOpt),
    // ParetFront, Trial, Observation
}

#[derive(Debug, clap::Args)]
pub struct BestOpt {}

impl ShowOpt {
    pub fn show(&self) -> anyhow::Result<()> {
        match self {
            Self::Best(opt) => self.show_best(opt)?,
        }
        Ok(())
    }

    fn show_best(&self, _opt: &BestOpt) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let mut reader = EventReader::new(stdin.lock());
        let mut current_study = None;
        let mut best = BTreeMap::new();

        fn output(
            study: &StudySpec,
            best_per_metric: &BTreeMap<String, CompactObservation>,
        ) -> anyhow::Result<()> {
            let json = serde_json::json!({
                "study": {
                    "name": study.name,
                    "id": study.id.to_string()
                },
                "best": best_per_metric
            });
            serde_json::to_writer_pretty(std::io::stdout().lock(), &json)?;
            println!();
            Ok(())
        }

        let mut skip = true;
        while let Some(event) = reader.read()? {
            match event {
                Event::Study(StudyEvent::Defined { spec }) => {
                    if let Some(study) = current_study.take() {
                        output(&study, &best)?;
                    }
                    current_study = Some(spec);
                    best = BTreeMap::new();
                    skip = false;
                }
                Event::Study(StudyEvent::Started) => {
                    skip = true;
                }
                Event::Observation(ObservationEvent::Finished { obs, .. }) => {
                    if skip {
                        continue;
                    }
                    for (name, metric) in &obs.metrics {
                        // TODO: consider fidelity

                        // TODO
                        // if metric.ty.objective.is_none() {
                        //     continue;
                        // }
                        let current = best
                            .entry(name.get().to_owned())
                            .or_insert_with(|| obs.to_compact());
                        if metric.is_better_than(current.metrics[name]) {
                            *current = obs.to_compact()
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(study) = current_study.take() {
            output(&study, &best)?;
        }
        Ok(())
    }
}
