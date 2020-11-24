use crate::event::{Event, EventOrLine, EventReader, ObservationEvent, StudyEvent};
use crate::runner::StudyRunnerOpt;
use crate::trial::CompactObservation;
use anyhow::Context;
use std::collections::BTreeMap;

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum ShowOpt {
    Best(BestOpt),
    // ParetFront, Trial, Observation
}

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct BestOpt {
    // TODO: --unknown-lines=ignore|passthrough
    #[structopt(long, short = "p")]
    pub passthrough_unknown_lines: bool,
}

impl ShowOpt {
    pub fn show(&self) -> anyhow::Result<()> {
        match self {
            Self::Best(opt) => self.show_best(opt)?,
        }
        Ok(())
    }

    fn show_best(&self, opt: &BestOpt) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let mut reader = EventReader::new(stdin.lock());
        let mut current_study = None;
        let mut best = BTreeMap::new();

        fn output(
            study: &StudyRunnerOpt,
            best_per_metric: &BTreeMap<String, CompactObservation>,
        ) -> anyhow::Result<()> {
            let json = serde_json::json!({
                "study": {
                    "name": study.study_name,
                    "instance": study.study_instance
                },
                "best": best_per_metric
            });
            serde_json::to_writer_pretty(std::io::stdout().lock(), &json)?;
            println!();
            Ok(())
        }

        let mut skip = true;
        while let Some(event) = reader.read_event_or_line()? {
            match event {
                EventOrLine::Event(Event::Study(StudyEvent::Defined { opt })) => {
                    if let Some(study) = current_study.take() {
                        output(&study, &best)?;
                    }
                    current_study = Some(opt);
                    best = BTreeMap::new();
                    skip = false;
                }
                EventOrLine::Event(Event::Study(StudyEvent::Started)) => {
                    skip = true;
                }
                EventOrLine::Event(Event::Obs(ObservationEvent::Finished { obs, .. })) => {
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
                EventOrLine::Line(line, err) => {
                    if opt.passthrough_unknown_lines {
                        println!("{}", line);
                    } else {
                        return Err(err)
                            .with_context(|| format!("Expected a JSON object, got {:?}", line));
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
