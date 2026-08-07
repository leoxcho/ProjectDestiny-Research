#!/usr/bin/env python3
import socket
import struct

# STUN magic cookie and message types
MAGIC_COOKIE = 0x2112A442
BINDING_REQUEST = 0x0001
BINDING_RESPONSE = 0x0101
ATTR_ALTERNATE_SERVER = 0x8023  # Demonware often uses 0x8023, but 0x8020 is standard; we'll try 0x8023 first

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(('127.0.0.1', 3478))
print("Python STUN redirect server listening on 127.0.0.1:3478")

while True:
    data, addr = sock.recvfrom(2048)
    # Minimal STUN binding request check: first 2 bytes == 0x0001
    if len(data) < 20:
        continue
    msg_type = struct.unpack('!H', data[:2])[0]
    if msg_type != BINDING_REQUEST:
        print(f"Ignoring non-binding-request type {msg_type:04x}")
        continue

    # Extract the transaction ID (16 bytes starting at offset 4)
    tid = data[4:20]

    # Build binding response
    # Header: type (0x0101), length (we'll calculate later), magic cookie, transaction ID
    resp = struct.pack('!H', BINDING_RESPONSE)  # type
    resp += struct.pack('!H', 0)                # length placeholder (will patch)
    resp += struct.pack('!I', MAGIC_COOKIE)
    resp += tid

    # Add ALTERNATE-SERVER attribute (type 0x8023, length 8)
    # Format: 1 byte reserved (0x00), 1 byte family (0x01 IPv4), 2 bytes port, 4 bytes IP
    attr_type = ATTR_ALTERNATE_SERVER
    attr_value = b'\x00\x01' + struct.pack('!H', 39000) + socket.inet_aton('127.0.0.1')
    attr_len = len(attr_value)
    resp += struct.pack('!H', attr_type)
    resp += struct.pack('!H', attr_len)
    resp += attr_value

    # Patch the length field (bytes 2-3) with total length after header (i.e., from byte 4 onwards)
    final_len = len(resp) - 4
    resp = resp[:2] + struct.pack('!H', final_len) + resp[4:]

    sock.sendto(resp, addr)
    print(f"Sent binding response with alternate-server 127.0.0.1:39000 to {addr}")