import os
import socket
import struct
import threading
try:
    from dnslib import DNSRecord, RR, A
except ImportError:  # Allows the stdlib-only STUN tests to run in minimal environments.
    DNSRecord = RR = A = None
from socketserver import ThreadingUDPServer, BaseRequestHandler

REDIRECT = {
    "dev-stun.demonware.net.": "127.0.0.1",
    "stun.us.demonware.net.": "127.0.0.1",
}
DNS_TYPE_NAMES = {1: "A", 28: "AAAA", 33: "SRV", 16: "TXT"}
VERBOSE_DNS = os.environ.get("DESTINY_DNS_VERBOSE", "0") == "1"

class DNSHandler(BaseRequestHandler):
    def handle(self):
        data, sock = self.request

        try:
            if DNSRecord is None:
                raise RuntimeError("dnslib is required to run the DNS listener")
            request = DNSRecord.parse(data)
            reply = request.reply()

            for q in request.questions:
                name = str(q.qname).lower()
                query_type = DNS_TYPE_NAMES.get(int(q.qtype), f"TYPE{q.qtype}")

                if VERBOSE_DNS:
                    print(
                        f"DNS QUERY name={name} type={query_type} "
                        f"peer={self.client_address[0]}:{self.client_address[1]}",
                        flush=True,
                    )

                if name in REDIRECT and query_type == "A":
                    target = REDIRECT[name]

                    if VERBOSE_DNS:
                        print(
                            f"DNS ACCEPT name={name} type={query_type} "
                            f"answer={target}",
                            flush=True,
                        )

                    reply.add_answer(
                        RR(
                            q.qname,
                            rdata=A(target),
                            ttl=60
                        )
                    )
                elif VERBOSE_DNS:
                    print(
                        f"DNS REJECT name={name} type={query_type} "
                        "reason=unsupported-local-bootstrap-query",
                        flush=True,
                    )

            packed_reply = reply.pack()
            answers = [f"{answer.rname} {answer.rdata}" for answer in reply.rr]
            print(
                f"DNS RESPONSE peer={self.client_address[0]}:{self.client_address[1]} "
                f"answers={answers} bytes={len(packed_reply)} hex={packed_reply.hex()}",
                flush=True,
            )
            sock.sendto(packed_reply, self.client_address)

        except Exception as e:
            print("DNS ERROR:", e, flush=True)


STUN_MAGIC_COOKIE = 0x2112A442


def stun_binding_response(request, source):
    """Return an RFC 5389 IPv4 binding response without interpreting payloads."""
    if len(request) < 20:
        return None
    message_type, message_length, cookie = struct.unpack("!HHI", request[:8])
    if message_type != 0x0001 or cookie != STUN_MAGIC_COOKIE:
        return None
    transaction_id = request[8:20]
    host, port = source[0], source[1]
    try:
        address = struct.unpack("!I", socket.inet_aton(host))[0]
    except OSError:
        return None
    xor_port = port ^ (STUN_MAGIC_COOKIE >> 16)
    xor_address = address ^ STUN_MAGIC_COOKIE
    attribute = struct.pack("!HHBBHI", 0x0020, 8, 0, 1, xor_port, xor_address)
    header = struct.pack("!HHI12s", 0x0101, len(attribute), STUN_MAGIC_COOKIE, transaction_id)
    return header + attribute


class STUNHandler(BaseRequestHandler):
    def handle(self):
        request, sock = self.request
        source = self.client_address
        print(
            f"STUN UDP PACKET peer={source[0]}:{source[1]} bytes={len(request)} hex={request.hex()}",
            flush=True,
        )
        print(f"STUN REQUEST peer={source[0]}:{source[1]} bytes={len(request)}", flush=True)
        response = stun_binding_response(request, source)
        if response is None:
            print("STUN REQUEST ignored=not_binding_request", flush=True)
            return
        advertised = os.environ.get("DESTINY_SERVICE_ADDR", "127.0.0.1:39000")
        sock.sendto(response, source)
        print(f"STUN RESPONSE peer={source[0]}:{source[1]} returned_service_address={advertised} mapped_address={source[0]}:{source[1]}", flush=True)


if __name__ == "__main__":
    dns_server = ThreadingUDPServer(("127.0.0.1", 53), DNSHandler)
    stun_server = ThreadingUDPServer(("127.0.0.1", 3478), STUNHandler)
    threading.Thread(target=dns_server.serve_forever, daemon=True).start()
    threading.Thread(target=stun_server.serve_forever, daemon=True).start()

    print("Local Destiny DNS running on 127.0.0.1:53", flush=True)
    print("Local Destiny STUN running on 127.0.0.1:3478", flush=True)

    try:
        threading.Event().wait()
    except KeyboardInterrupt:
        print("\nDNS server stopped")
        dns_server.shutdown()
        stun_server.shutdown()
