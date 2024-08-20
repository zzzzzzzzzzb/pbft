use std::net::AddrParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("serde_json err: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("seq is not allowed")]
    SequenceNotAllowed(),
    #[error("view is not allowed")]
    ViewNotAllowed(),
    #[error("mutex lock err")]
    MutexError(),
    #[error("status is :{0}")]
    RPCError(#[from] tonic::Status),
    #[error("tonic transport err is :{0}")]
    TransportError(#[from] tonic::transport::Error),
    #[error("parse addr err is :{0}")]
    ParseAddrError(#[from] AddrParseError),
    #[error("no such message type")]
    NoSuchMessageType(),
}