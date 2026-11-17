#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Decode a keystore secret without printing its contents."""
import base64
import binascii
import json
import os
import re
import sys
from pathlib import Path


def decode_keystore(value: str) -> bytes:
    text = value.strip().lstrip("\ufeff").strip()
    if text.startswith('"') and text.endswith('"'):
        try:
            text = json.loads(text)
        except json.JSONDecodeError:
            text = text[1:-1]
    elif text.startswith("'") and text.endswith("'"):
        text = text[1:-1]
    elif text.startswith("“") and text.endswith("”"):
        text = text[1:-1]
    if text.startswith("```") and text.endswith("```") and "\n" in text:
        text = "\n".join(text.splitlines()[1:-1])
    if text.lower().startswith("data:") and ";base64," in text.lower():
        text = text.split(",", 1)[1]
    text = re.sub(r"[\s\u200b\u200c\u200d\ufeff]", "", text)
    for _ in range(2):
        try:
            decoded = base64.b64decode(text + "=" * (-len(text) % 4), altchars=b"-_", validate=True)
        except (binascii.Error, ValueError, TypeError):
            break
        if decoded[:4] in (b"\xfe\xed\xfe\xed", b"\xce\xce\xce\xce"):
            return decoded
        if len(decoded) >= 16 and decoded[:1] == b"\x30":
            return decoded  # PKCS#12: keytool validates the actual container and password.
        try:
            text = re.sub(r"[\s\u200b\u200c\u200d\ufeff]", "", decoded.decode("ascii"))
        except UnicodeDecodeError:
            break
    raise ValueError(
        "ANDROID_KEYSTORE is not a valid Base64-encoded JKS/JCEKS/PKCS#12 file. "
        "Re-encode the keystore file and update this GitHub secret."
    )


def main() -> None:
    try:
        decoded = decode_keystore(os.environ.get("KEYSTORE_B64", ""))
    except ValueError as error:
        raise SystemExit(str(error)) from None
    target = Path(sys.argv[1])
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_NOFOLLOW", 0)
    with os.fdopen(os.open(target, flags, 0o600), "wb") as stream:
        stream.write(decoded)
    target.chmod(0o600)
    print("Decoded the Android keystore; keytool will verify its password and private key.")


if __name__ == "__main__":
    main()
