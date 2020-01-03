use crate::metric::Metric;
use crate::param::{Param, ParamSpec, ParamValue};
use crate::pubsub::{PubSub, PubSubChannel, TrialAction};
use crate::trial::Trial;
use crate::{Error, ErrorKind, Result};
use rand::seq::SliceRandom as _;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::env;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::num::NonZeroU64;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

const ENV_VAR_SERVER_ADDR: &str = "HONE_SERVER_ADDR";

#[derive(Debug)]
pub struct StudyClient {
    server: SocketAddr,
    socket: UdpSocket,
}

impl StudyClient {
    pub fn new() -> Result<Self> {
        let addr = track!(
            env::var(ENV_VAR_SERVER_ADDR).map_err(Error::from),
            "name={}",
            ENV_VAR_SERVER_ADDR
        )?;
        let addr: SocketAddr = track!(addr.parse().map_err(Error::from))?;
        let socket = track!(UdpSocket::bind("127.0.0.1:0").map_err(Error::from))?;
        Ok(Self {
            server: addr,
            socket,
        })
    }

    pub fn suggest(&self, param: Param) -> Result<ParamValue> {
        let message = Message::SuggestCall { param };
        let message = track!(serde_json::to_vec(&message).map_err(Error::from))?;
        track!(self
            .socket
            .send_to(&message[..], self.server)
            .map_err(Error::from))?;

        let mut buf = [0u8; 4096];
        let (size, addr) = track!(self.socket.recv_from(&mut buf).map_err(Error::from))?;
        track_assert_eq!(self.server, addr, ErrorKind::InvalidInput);

        let message: Message = track!(serde_json::from_slice(&buf[..size]).map_err(Error::from))?;
        if let Message::SuggestReply { value } = message {
            return Ok(value);
        } else {
            track_panic!(ErrorKind::InvalidInput, "Unexpected message: {:?}", message)
        }
    }

    pub fn report(&self, step: Option<NonZeroU64>, metric: Metric) -> Result<()> {
        let message = Message::ReportCast { step, metric };
        let message = track!(serde_json::to_vec(&message).map_err(Error::from))?;
        track!(self
            .socket
            .send_to(&message[..], self.server)
            .map_err(Error::from))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct StudyServer {
    study_name: String,
    socket: UdpSocket,
    pubsub: PubSubChannel,
    trials: HashMap<Uuid, Trial>,
    current_trial: Option<Trial>,
    rx: Receiver<Command>,
    tx: Option<Sender<Command>>,
}

impl StudyServer {
    pub fn new(study_name: String, mut pubsub: PubSub) -> Result<Self> {
        let socket = track!(UdpSocket::bind("127.0.0.1:0").map_err(Error::from))?;
        track!(socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(Error::from))?;

        let pubsub_channel = track!(pubsub.channel(&study_name))?;
        let (tx, rx) = mpsc::channel();
        let this = Self {
            study_name,
            socket,
            pubsub: pubsub_channel,
            trials: HashMap::new(),
            current_trial: None,
            rx,
            tx: Some(tx),
        };
        track!(this.set_addr_env_var())?;
        Ok(this)
    }

    fn set_addr_env_var(&self) -> Result<()> {
        env::set_var(ENV_VAR_SERVER_ADDR, track!(self.addr())?.to_string());
        Ok(())
    }

    pub fn addr(&self) -> Result<SocketAddr> {
        track!(self.socket.local_addr().map_err(Error::from))
    }

    pub fn spawn(mut self) -> StudyServerHandle {
        let tx = self.tx.take().unwrap_or_else(|| unreachable!());
        thread::spawn(move || {
            if let Err(e) = track!(self.run()) {
                eprintln!("Study Server Error: {}", e);
            }
        });
        StudyServerHandle { tx }
    }

    fn run(mut self) -> Result<()> {
        track!(self.sync())?;
        while track!(self.run_once())? {}
        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        for (id, action) in track!(self.pubsub.poll())? {
            track!(self.handle_action(id, action))?;
        }
        Ok(())
    }

    fn handle_action(&mut self, id: Uuid, action: TrialAction) -> Result<()> {
        match action {
            TrialAction::Start { id, timestamp } => {
                let trial = Trial {
                    id,
                    timestamp,
                    ..track!(Trial::new())?
                };
                self.trials.insert(trial.id, trial);
            }
            TrialAction::Define { param } => {
                let trial = track_assert_some!(self.trials.get_mut(&id), ErrorKind::InvalidInput);
                trial
                    .param_specs
                    .insert(param.name.clone(), param.spec.clone());
            }
            TrialAction::Sample { name, value } => {
                let trial = track_assert_some!(self.trials.get_mut(&id), ErrorKind::InvalidInput);
                trial.param_values.insert(name, value);
            }
            TrialAction::Report { step, metric } => {
                let trial = track_assert_some!(self.trials.get_mut(&id), ErrorKind::InvalidInput);
                trial.report(Some(step), &metric);
            }
            TrialAction::End => {}
        }
        Ok(())
    }

    fn run_once(&mut self) -> Result<bool> {
        let mut buf = [0u8; 4096];

        match self.socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let message: Message =
                    track!(serde_json::from_slice(&buf[..size]).map_err(Error::from))?;
                if let Some(reply) = track!(self.handle_message(message))? {
                    let reply = track!(serde_json::to_vec(&reply).map_err(Error::from))?;
                    track!(self.socket.send_to(&reply[..], addr).map_err(Error::from))?;
                }
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    return Err(track!(Error::from(e)));
                }
            }
        }

        match self.rx.try_recv() {
            Ok(command) => {
                track!(self.handle_command(command))?;
                Ok(true)
            }
            Err(mpsc::TryRecvError::Empty) => {
                track!(self.sync())?;
                return Ok(true);
            }
            Err(mpsc::TryRecvError::Disconnected) => return Ok(false),
        }
    }

    fn handle_message(&mut self, message: Message) -> Result<Option<Message>> {
        match message {
            Message::SuggestCall { param } => {
                let value = track!(self.handle_suggest_call(param))?;
                Ok(Some(Message::SuggestReply { value }))
            }
            Message::SuggestReply { .. } => {
                track_panic!(ErrorKind::Bug, "Unexpected message: {:?}", message)
            }
            Message::ReportCast { step, metric } => {
                eprintln!("Reported: step={:?}, metric={:?}", step, metric);
                let trial = track_assert_some!(self.current_trial.as_mut(), ErrorKind::Bug);
                trial.report(step, &metric);

                let step = track_assert_some!(trial.last_step(&metric.name), ErrorKind::Bug);
                let action = TrialAction::Report { step, metric };
                track!(self.pubsub.publish(action))?;

                Ok(None)
            }
        }
    }

    fn handle_suggest_call(&mut self, param: Param) -> Result<ParamValue> {
        let trial = track_assert_some!(self.current_trial.as_mut(), ErrorKind::Bug);

        if let Some(value) = trial.param_values.get(&param.name) {
            return Ok(value.clone());
        }

        // TODO: check pre-defined parameter

        // TODO
        let ParamSpec::Choice { choices } = &param.spec;
        let mut rng = rand::thread_rng();
        let choice = track_assert_some!(choices.choose(&mut rng), ErrorKind::InvalidInput);
        let value = ParamValue(choice.clone());

        track!(self.pubsub.publish(TrialAction::Define {
            param: param.clone()
        }))?;
        track!(self.pubsub.publish(TrialAction::Sample {
            name: param.name.clone(),
            value: value.clone()
        }))?;
        trial.param_values.insert(param.name.clone(), value.clone());
        Ok(value)
    }

    fn handle_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::StartTrial { reply } => {
                let trial = track!(Trial::new())?;

                track!(self.pubsub.publish(TrialAction::Start {
                    id: trial.id,
                    timestamp: trial.timestamp
                }))?;
                let _ = reply.send(trial.id);

                self.current_trial = Some(trial);
                Ok(())
            }
            Command::EndTrial => {
                track!(self.pubsub.publish(TrialAction::End))?;
                self.current_trial = None;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StudyServerHandle {
    tx: Sender<Command>,
}

impl StudyServerHandle {
    pub fn start_trial(&self) -> Result<TrialHandle> {
        let (tx, rx) = mpsc::channel();
        let command = Command::StartTrial { reply: tx };
        let _ = self.tx.send(command);
        let id = track!(rx.recv().map_err(Error::from))?;
        env::set_var("HONE_TRIAL_ID", id.to_string());
        Ok(TrialHandle {
            id,
            tx: self.tx.clone(),
        })
    }
}

#[derive(Debug)]
enum Command {
    StartTrial { reply: mpsc::Sender<Uuid> },
    EndTrial,
}

#[derive(Debug)]
pub struct TrialHandle {
    id: Uuid,
    tx: Sender<Command>,
}

impl Drop for TrialHandle {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::EndTrial);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
enum Message {
    SuggestCall {
        param: Param,
    },
    SuggestReply {
        value: ParamValue,
    },
    ReportCast {
        step: Option<NonZeroU64>,
        metric: Metric,
    },
}
