//! Protocol research types only. No Bungie opcode or wire value is asserted.
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Direction {
    ClientToServer,
    ServerToClient,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Packet {
    pub opcode: Option<u16>,
    pub direction: Direction,
    pub timestamp_ms: u128,
    pub session_id: String,
    pub payload: Vec<u8>,
    pub unknown_fields: Vec<UnknownField>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnknownField {
    pub offset: usize,
    pub bytes: Vec<u8>,
    pub reason: String,
}
#[derive(Debug, Default)]
pub struct OpcodeRegistry {
    entries: Vec<OpcodeEntry>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcodeEntry {
    pub name: String,
    pub value: Option<u16>,
    pub status: String,
    pub notes: String,
}
impl OpcodeRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(&mut self, e: OpcodeEntry) {
        self.entries.push(e)
    }
    pub fn entries(&self) -> &[OpcodeEntry] {
        &self.entries
    }
}
pub fn encode(p: &Packet) -> Vec<u8> {
    p.payload.clone()
}
pub fn decode(payload: Vec<u8>) -> Packet {
    Packet {
        opcode: None,
        direction: Direction::Unknown,
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        session_id: String::new(),
        payload,
        unknown_fields: vec![],
    }
}
pub trait PacketLogger {
    fn record(&mut self, packet: Packet);
    fn packets(&self) -> &[Packet];
}
#[derive(Default)]
pub struct MemoryPacketLog {
    pub values: Vec<Packet>,
}
impl PacketLogger for MemoryPacketLog {
    fn record(&mut self, p: Packet) {
        self.values.push(p)
    }
    fn packets(&self) -> &[Packet] {
        &self.values
    }
}
pub trait ReplaySource {
    fn next(&mut self) -> Option<Packet>;
}
impl ReplaySource for std::vec::IntoIter<Packet> {
    fn next(&mut self) -> Option<Packet> {
        Iterator::next(self)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnknownMessage {
    pub timestamp_ms: u128,
    pub session_id: String,
    pub transport: String,
    pub payload_length: usize,
    pub hex: String,
    pub ascii: String,
    pub handler: String,
}
pub fn unknown_message(session_id: &str, transport: &str, payload: &[u8]) -> UnknownMessage {
    let ascii = payload
        .iter()
        .map(|b| {
            if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            }
        })
        .collect();
    UnknownMessage {
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        session_id: session_id.into(),
        transport: transport.into(),
        payload_length: payload.len(),
        hex: hex::encode(payload),
        ascii,
        handler: "unmapped-placeholder".into(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_packet_is_fully_described() {
        let m = unknown_message("s", "tcp", b"A\0");
        assert_eq!(m.hex, "4100");
        assert_eq!(m.ascii, "A.");
        assert_eq!(m.handler, "unmapped-placeholder");
    }
}
// TODO(protocol): replace optional opcode and unknown fields only after captured evidence.
// TODO: document experimentally observed framing before implementing it.
