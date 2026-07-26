#!/usr/bin/env python3
"""Portable recovery utility for KeyTide v1 containers."""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path

from cryptography.exceptions import InvalidTag
from cryptography.hazmat.primitives.ciphers.aead import AESGCM


CONTAINER_MAGIC = b"COFFER\x00\x01"
KEY_MAGIC = b"COFKEY\x00\x01"
PREFIX_LEN = 30
KEY_FILE_LEN = 44
VERSION = 1
ALGORITHM_AES_256_GCM = 1
MAX_FILENAME_LEN = 1024


class RecoveryError(Exception):
    """A safe, user-facing recovery failure."""


def parse_key(data: bytes) -> bytes:
    if (
        len(data) != KEY_FILE_LEN
        or data[:8] != KEY_MAGIC
        or data[8] != VERSION
        or data[9] != ALGORITHM_AES_256_GCM
        or data[10:12] != b"\x00\x00"
    ):
        raise RecoveryError("invalid or unsupported KeyTide key")
    return data[12:44]


def decrypt_container(container: bytes, key: bytes) -> tuple[str, bytes]:
    if len(container) < PREFIX_LEN or container[:8] != CONTAINER_MAGIC:
        raise RecoveryError("invalid KeyTide container")
    if container[8] != VERSION or container[9] != ALGORITHM_AES_256_GCM:
        raise RecoveryError("unsupported KeyTide version or algorithm")

    prefix = container[:PREFIX_LEN]
    ciphertext_length = struct.unpack(">Q", prefix[22:30])[0]
    ciphertext = container[PREFIX_LEN:]
    if ciphertext_length < 16 or ciphertext_length != len(ciphertext):
        raise RecoveryError("invalid KeyTide container length")

    try:
        payload = AESGCM(key).decrypt(prefix[10:22], ciphertext, prefix)
    except InvalidTag as error:
        raise RecoveryError("authentication failed: wrong key or modified container") from error

    if len(payload) < 10:
        raise RecoveryError("invalid encrypted payload")
    filename_length = struct.unpack(">H", payload[:2])[0]
    name_end = 2 + filename_length
    size_end = name_end + 8
    if not 1 <= filename_length <= MAX_FILENAME_LEN or size_end > len(payload):
        raise RecoveryError("invalid encrypted payload")
    try:
        filename = payload[2:name_end].decode("utf-8")
    except UnicodeDecodeError as error:
        raise RecoveryError("invalid restored filename") from error
    if filename in {".", ".."} or any(character in filename for character in ("\x00", "/", "\\")):
        raise RecoveryError("unsafe restored filename")

    declared_size = struct.unpack(">Q", payload[name_end:size_end])[0]
    plaintext = payload[size_end:]
    if declared_size != len(plaintext):
        raise RecoveryError("invalid encrypted payload size")
    return filename, plaintext


def main() -> int:
    parser = argparse.ArgumentParser(description="Decrypt a KeyTide v1 container without the GUI")
    parser.add_argument("container", type=Path, help="path to the .coffer container")
    parser.add_argument("key", type=Path, help="path to the matching .cofferkey file")
    parser.add_argument(
        "--output",
        type=Path,
        help="output file path; defaults to the authenticated original filename in the current directory",
    )
    args = parser.parse_args()

    try:
        key = parse_key(args.key.read_bytes())
        filename, plaintext = decrypt_container(args.container.read_bytes(), key)
        output = args.output if args.output is not None else Path.cwd() / filename
        if output.exists():
            raise RecoveryError(f"refusing to replace existing output: {output}")
        output.write_bytes(plaintext)
        print(f"Restored {output}")
        return 0
    except (OSError, RecoveryError) as error:
        print(f"Recovery failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
