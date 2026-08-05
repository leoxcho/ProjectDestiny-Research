# Protocol Discovery Report

The analyzer preserves packet payloads and produces fingerprints, repeated/constant/variable byte offsets, candidate boundaries, and session timelines. These are discovery candidates only.

- Confirmed: capture metadata, timestamps, direction labels, raw payload preservation, session grouping.
- Probable: repeated byte offsets and stable prefixes are useful clustering signals.
- Unknown: field meanings, framing, opcodes, handshake, request/response semantics.

No opcode or packet meaning is assigned without external evidence.
