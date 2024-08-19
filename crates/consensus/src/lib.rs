pub mod error;
pub mod server;
mod message;
mod pool;
mod client;
mod event;

#[cfg(test)]
mod tests {
    use std::env;
    use crate::client::send;
    use crate::message::{Message, Request};
    use crate::message::message::Payload;

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
            payload: Some(Payload::Request(Request{
                payload: Vec::new(),
            })),
        };
        send("http://127.0.0.1:8080", msg).await.unwrap();
    }
}