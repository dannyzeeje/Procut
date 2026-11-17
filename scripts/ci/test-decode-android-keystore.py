#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Credential-format regression checks using synthetic, non-secret bytes."""
import base64
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("decoder", Path(__file__).with_name("decode-android-keystore.py"))
decoder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(decoder)


class KeystoreEncodingTests(unittest.TestCase):
    def test_formatting_preserves_every_binary_byte(self):
        original = b"\xfe\xed\xfe\xed" + bytes(range(256))
        encoded = base64.b64encode(original).decode()
        wrapped = "\n ".join(encoded[i:i + 40] for i in range(0, len(encoded), 40))
        cases = [encoded, wrapped, "\ufeff" + wrapped, json.dumps(wrapped),
                 "'" + encoded + "'", "data:application/octet-stream;base64," + encoded,
                 "```base64\n" + wrapped + "\n```", encoded.rstrip("="),
                 base64.urlsafe_b64encode(original).decode(),
                 base64.b64encode(encoded.encode()).decode(), "“" + encoded + "”",
                 base64.b64encode(wrapped.encode()).decode()]
        for value in cases:
            with self.subTest(format=value[:8]):
                self.assertEqual(decoder.decode_keystore(value), original)

    def test_invalid_input_does_not_appear_in_errors(self):
        for value in ["", "/some/private/key.jks", "not a key!", base64.b64encode(b"wrong binary").decode()]:
            with self.assertRaises(ValueError) as error:
                decoder.decode_keystore(value)
            self.assertIn("ANDROID_KEYSTORE", str(error.exception))
            if value:
                self.assertNotIn(value, str(error.exception))


if __name__ == "__main__":
    unittest.main()
