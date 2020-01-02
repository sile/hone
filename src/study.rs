use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::num::NonZeroU64;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct StudyServer {
    study_name: String,
    socket: UdpSocket,
    rx: Receiver<()>,
    tx: Option<Sender<()>>,
}

impl StudyServer {
    pub fn new(study_name: String) -> Result<Self> {
        let socket = track!(UdpSocket::bind("127.0.0.1:0").map_err(Error::from))?;
        track!(socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(Error::from))?;

        let (tx, rx) = mpsc::channel();
        Ok(Self {
            study_name,
            socket,
            rx,
            tx: Some(tx),
        })
    }

    pub fn addr(&self) -> Result<SocketAddr> {
        track!(self.socket.local_addr().map_err(Error::from))
    }

    pub fn spawn(mut self) -> StudyServerHandle {
        let tx = self.tx.take().unwrap_or_else(|| unreachable!());
        thread::spawn(move || self.run());
        StudyServerHandle { tx }
    }

    fn run(mut self) {
        loop {
            match self.run_once() {
                Ok(true) => {}
                Ok(false) => break,
                Err(e) => {
                    eprintln!("Study Server Error: {}", e);
                    break;
                }
            }
        }
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
            Ok(()) => return Ok(true),
            Err(mpsc::TryRecvError::Empty) => return Ok(true),
            Err(mpsc::TryRecvError::Disconnected) => return Ok(false),
        }
    }

    fn handle_message(&mut self, message: Message) -> Result<Option<Message>> {
        todo!("Message: {:?}", message)
    }
}

#[derive(Debug, Clone)]
pub struct StudyServerHandle {
    tx: Sender<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
enum Message {
    SuggestCall {
        param_name: String,
        // TODO: range
    },
    SuggestReply {
        param_value: f64,
    },
    ReportCast {
        step: NonZeroU64,
        metrics: Vec<Metric>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Metric {
    pub name: String,
    pub direction: Direction,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    Minimize,
    Maximize,
}
