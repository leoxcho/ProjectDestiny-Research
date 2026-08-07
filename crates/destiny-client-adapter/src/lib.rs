//! External-client boundary. Translation is intentionally lossless and semantic-free.
use anyhow::Result;
use destiny_network::{Direction, Packet};
use destiny_server_runtime::{DispatchResult, ServerRuntime};
use std::sync::{Arc, Mutex};
pub trait ExternalClientConnection {
    fn receive(&mut self) -> Result<Vec<u8>>;
    fn send(&mut self, bytes: &[u8]) -> Result<()>;
}
pub trait PacketTranslator {
    fn inbound(&self, bytes: Vec<u8>, session_id: String) -> Packet;
    fn outbound(&self, packet: &Packet) -> Vec<u8>;
}
pub struct OpaqueTranslator;
impl PacketTranslator for OpaqueTranslator {
    fn inbound(&self, bytes: Vec<u8>, session_id: String) -> Packet {
        Packet {
            opcode: None,
            direction: Direction::ClientToServer,
            timestamp_ms: 0,
            session_id,
            payload: bytes,
            unknown_fields: vec![],
        }
    }
    fn outbound(&self, p: &Packet) -> Vec<u8> {
        p.payload.clone()
    }
}
pub struct ClientAdapter<T: PacketTranslator = OpaqueTranslator> {
    pub translator: T,
    pub runtime: Arc<Mutex<ServerRuntime>>,
}
impl<T: PacketTranslator> ClientAdapter<T> {
    pub fn route(&self, session: String, bytes: Vec<u8>) -> DispatchResult {
        self.runtime
            .lock()
            .unwrap()
            .dispatch(self.translator.inbound(bytes, session))
    }
}
pub struct ResponsePipeline;
impl ResponsePipeline {
    pub fn encode<T: PacketTranslator>(t: &T, p: &Packet) -> Vec<u8> {
        t.outbound(p)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opaque_route_remains_unknown() {
        let db = Arc::new(destiny_runtime_core::Runtime::open(":memory:").unwrap());
        let r = Arc::new(Mutex::new(ServerRuntime::new(db)));
        let a = ClientAdapter {
            translator: OpaqueTranslator,
            runtime: r,
        };
        assert!(matches!(
            a.route("s".into(), vec![1]),
            DispatchResult::UnknownOpcode
        ));
    }
}
