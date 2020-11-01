use crate::envvar;
use crate::metric::{MetricName, MetricType, MetricValue};
use crate::param::{ParamName, ParamType, ParamValue};
use crate::trial::RunId;
use bytecodec::bincode_codec::{BincodeDecoder, BincodeEncoder};
use fibers_rpc::client::ClientServiceBuilder;
use fibers_rpc::server::ServerBuilder;
use fibers_rpc::{Call, ProcedureId};
use futures::{Async, Future, Stream};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub fn init() {
    fibers_global::set_thread_count(1);
}

pub fn call<RPC: Call>(req: RPC::Req) -> anyhow::Result<RPC::Res>
where
    RPC::ReqEncoder: Default,
    RPC::ResDecoder: Default,
{
    let server_addr = envvar::get_server_addr()?;
    let service = ClientServiceBuilder::new().finish(fibers_global::handle());
    let service_handle = service.handle();
    fibers_global::spawn(service.map_err(|e| panic!("{}", e)));
    let future = RPC::client(&service_handle).call(server_addr, req);
    let res = fibers_global::execute(future)?;
    Ok(res)
}

#[derive(Debug)]
pub struct AskRpc;

impl Call for AskRpc {
    const ID: ProcedureId = ProcedureId(0);
    const NAME: &'static str = "ask";

    type Req = AskReq;
    type ReqEncoder = BincodeEncoder<Self::Req>;
    type ReqDecoder = BincodeDecoder<Self::Req>;

    type Res = Result<ParamValue, AskError>;
    type ResEncoder = BincodeEncoder<Self::Res>;
    type ResDecoder = BincodeDecoder<Self::Res>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AskReq {
    pub run_id: RunId,
    pub param_name: ParamName,
    pub param_type: ParamType,
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum AskError {
    #[error("TODO")]
    RecvError,

    #[error("TODO")]
    InvalidRequest,
}

#[derive(Debug)]
pub struct TellRpc;

impl Call for TellRpc {
    const ID: ProcedureId = ProcedureId(1);
    const NAME: &'static str = "tell";

    type Req = TellReq;
    type ReqEncoder = BincodeEncoder<Self::Req>;
    type ReqDecoder = BincodeDecoder<Self::Req>;

    type Res = Result<(), TellError>;
    type ResEncoder = BincodeEncoder<Self::Res>;
    type ResDecoder = BincodeDecoder<Self::Res>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TellReq {
    pub run_id: RunId,
    pub metric_name: MetricName,
    pub metric_type: MetricType,
    pub metric_value: MetricValue,
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum TellError {
    #[error("TODO")]
    RecvError,

    #[error("TODO")]
    InvalidRequest,
}

#[derive(Debug)]
pub enum Message {
    Ask {
        req: AskReq,
        reply: fibers::sync::oneshot::Sender<Result<ParamValue, AskError>>,
    },
    Tell {
        req: TellReq,
        reply: fibers::sync::oneshot::Sender<Result<(), TellError>>,
    },
}

#[derive(Debug)]
pub struct Channel {
    rx: fibers::sync::mpsc::Receiver<Message>,
}

impl Channel {
    pub fn try_recv(&mut self) -> Option<Message> {
        match self.rx.poll() {
            Err(()) => unreachable!(),
            Ok(Async::Ready(None)) => unreachable!(),
            Ok(Async::NotReady) => None,
            Ok(Async::Ready(Some(m))) => Some(m),
        }
    }
}

#[derive(Debug)]
pub struct AskHandler {
    tx: fibers::sync::mpsc::Sender<Message>,
}

impl fibers_rpc::server::HandleCall<AskRpc> for AskHandler {
    fn handle_call(&self, req: <AskRpc as Call>::Req) -> fibers_rpc::server::Reply<AskRpc> {
        let (tx, rx) = fibers::sync::oneshot::channel();
        let _ = self.tx.send(Message::Ask { req, reply: tx });
        fibers_rpc::server::Reply::future(
            rx.then(|result| Ok(result.unwrap_or_else(|_| Err(AskError::RecvError)))),
        )
    }
}

#[derive(Debug)]
pub struct TellHandler {
    tx: fibers::sync::mpsc::Sender<Message>,
}

impl fibers_rpc::server::HandleCall<TellRpc> for TellHandler {
    fn handle_call(&self, req: <TellRpc as Call>::Req) -> fibers_rpc::server::Reply<TellRpc> {
        let (tx, rx) = fibers::sync::oneshot::channel();
        let _ = self.tx.send(Message::Tell { req, reply: tx });
        fibers_rpc::server::Reply::future(
            rx.then(|result| Ok(result.unwrap_or_else(|_| Err(TellError::RecvError)))),
        )
    }
}

// TODO:
pub fn spawn_rpc_server() -> anyhow::Result<(SocketAddr, Channel)> {
    let mut builder = ServerBuilder::new(SocketAddr::from(([127, 0, 0, 1], 0)));
    let (tx, rx) = fibers::sync::mpsc::channel();
    builder.add_call_handler(AskHandler { tx: tx.clone() });
    builder.add_call_handler(TellHandler { tx: tx.clone() });
    let server = builder.finish(fibers_global::handle());
    let (server, addr) = fibers_global::execute(server.local_addr())?;
    fibers_global::spawn(server.map_err(|e| panic!("{}", e)));

    Ok((addr, Channel { rx }))
}
