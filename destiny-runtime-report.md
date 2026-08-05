# Destiny Runtime Compatibility Report

## Implemented

- SQLite definition database and read-only `destiny-runtime-core` access.
- Definition API and server-runtime service composition.
- Packet records with direction, timestamp, session ID, payload, unknown fields, logging, and replay interfaces.
- Simulated authentication sessions with expiration and validation.
- Inventory, activity, and world state models.
- `destiny-test-client` lifecycle, definition, inventory, activity, world, and unknown-packet exercises.
- Runtime telemetry for request logs, dispatch timing, packet traces, and session history.

## Simulated

- Test-client transport calls are direct in-process service calls.
- Authentication accepts an explicitly supplied test account.
- Activity and world transitions are model-level state changes only.
- Inventory and session persistence are in memory.

## Unknown

- Destiny opcode values and packet framing.
- Authentication handshake and credential exchange.
- Message schemas, field encodings, matchmaking, world replication, and RPCS3 behavior.
- Server response semantics and client compatibility.

## Future protocol requirements

1. Capture and preserve authenticated client/server packet traces.
2. Establish framing and direction from evidence, without assigning unknown opcodes.
3. Document each observed opcode and field with sample payloads and confidence.
4. Add replay fixtures and response-validation rules.
5. Replace simulated services only after behavior is independently verified.
