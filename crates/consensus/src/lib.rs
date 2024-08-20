mod client;
pub mod error;
mod event;
mod message;
mod pool;
pub mod server;

#[cfg(test)]
mod tests {
    use crate::{
        client::send,
        message::{message::Payload, Message, Request},
    };
    use std::env;

    #[test]
    fn build_proto() {
        env::set_var("OUT_DIR", "src/");
        tonic_build::compile_protos("protos/message.proto").unwrap();
    }

    #[tokio::test]
    async fn start_client() {
        let msg = Message {
            view: 1,
            seq: 3,
            id: 0,
            digest: "".to_string(),
            payload: Some(Payload::Request(Request {
                payload: Vec::new(),
            })),
        };
        send("http://127.0.0.1:8080", msg).await.unwrap();
    }
}
