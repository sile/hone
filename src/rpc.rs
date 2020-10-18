use crate::envvar;
use crate::hp::{HpDistribution, HpValue};
use bytecodec::bincode_codec::{BincodeDecoder, BincodeEncoder};
use fibers_rpc::client::ClientServiceBuilder;
use fibers_rpc::{Call, ProcedureId};
use futures::Future;
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
    let server_addr: SocketAddr = std::env::var(envvar::KEY_SERVER_ADDR)?.parse()?;
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

    type Res = Result<HpValue, AskError>;
    type ResEncoder = BincodeEncoder<Self::Res>;
    type ResDecoder = BincodeDecoder<Self::Res>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AskReq {
    pub trial_id: u64,
    pub param_name: String,
    pub distribution: Option<HpDistribution>,
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum AskError {}
