# Destiny Protocol Status

## Known

- Packet samples can be ingested from JSON Lines while preserving raw payload bytes.
- Sessions are grouped by supplied session ID and retain first/last timestamps and packet counts.
- Client/server direction, timestamps, optional opcode observations, payload sizes, confidence, and notes are stored.
- The analyzer provides timeline, hex-view, and conservative field-boundary output.
- The RPCS3 harness accepts TCP connections and records connection/byte observations.

## Unknown

- No Destiny opcode values are assigned by this framework.
- Packet framing, handshake semantics, authentication, message schemas, and field encodings remain unmapped.
- The harness does not claim RPCS3 compatibility or emit protocol responses.

## Confidence

- Confirmed: preservation of supplied capture metadata and raw payloads.
- Probable: session grouping when capture session IDs are reliable.
- Unknown: all inferred protocol meaning, field boundaries, and opcode semantics.

## Future protocol requirements

1. Capture evidence with provenance and direction metadata.
2. Record repeated samples before assigning field boundaries.
3. Promote opcode/field claims only with corroborated traces.
4. Add response fixtures and compatibility assertions after handshake behavior is observed.
