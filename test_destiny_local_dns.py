import struct
import unittest

from destiny_local_dns import REDIRECT, STUN_MAGIC_COOKIE, stun_binding_response


class BootstrapTests(unittest.TestCase):
    def test_stun_hostname_redirect_is_local_a_record(self):
        self.assertEqual(REDIRECT["dev-stun.demonware.net."], "127.0.0.1")

    def test_stun_binding_response_preserves_transaction_and_maps_peer(self):
        transaction_id = b"0123456789ab"
        request = struct.pack("!HHI12s", 0x0001, 0, STUN_MAGIC_COOKIE, transaction_id)
        response = stun_binding_response(request, ("127.0.0.1", 40000))

        self.assertIsNotNone(response)
        message_type, length, cookie = struct.unpack("!HHI", response[:8])
        self.assertEqual((message_type, length, cookie), (0x0101, 12, STUN_MAGIC_COOKIE))
        self.assertEqual(response[8:20], transaction_id)

    def test_unknown_udp_payload_is_not_interpreted_as_stun(self):
        self.assertIsNone(stun_binding_response(b"unknown", ("127.0.0.1", 40000)))


if __name__ == "__main__":
    unittest.main()
