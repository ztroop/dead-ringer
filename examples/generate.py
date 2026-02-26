#!/usr/bin/env python3
"""Generate example binary file pairs for testing dead-ringer (dring)."""

import os
import struct
import random

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
random.seed(42)


def write_bin(name: str, data: bytes):
    path = os.path.join(SCRIPT_DIR, name)
    with open(path, "wb") as f:
        f.write(data)
    print(f"  wrote {path} ({len(data)} bytes)")


def generate_simple_pair():
    """Small pair (~64 bytes) with a handful of scattered byte differences."""
    base = bytes(range(64))
    modified = bytearray(base)
    modified[0] = 0xFF
    modified[15] = 0xAA
    modified[31] = 0xBB
    modified[47] = 0xCC
    modified[63] = 0xDD
    write_bin("simple_v1.bin", base)
    write_bin("simple_v2.bin", bytes(modified))


def generate_header_pair():
    """Simulates a binary with a modified header (magic bytes + version)."""
    magic_v1 = b"\x7fELF"
    magic_v2 = b"\x7fOLF"
    version_v1 = struct.pack("<H", 1)
    version_v2 = struct.pack("<H", 2)
    body = bytes(random.randint(0, 255) for _ in range(120))

    write_bin("header_original.bin", magic_v1 + version_v1 + body)
    write_bin("header_modified.bin", magic_v2 + version_v2 + body)


def generate_firmware_pair():
    """Larger pair (~512 bytes) with many differences, like a firmware update."""
    size = 512
    v1 = bytearray(random.randint(0, 255) for _ in range(size))
    v2 = bytearray(v1)
    indices = sorted(random.sample(range(size), 80))
    for i in indices:
        v2[i] = (v1[i] + random.randint(1, 255)) & 0xFF
    write_bin("firmware_v1.bin", bytes(v1))
    write_bin("firmware_v2.bin", bytes(v2))


def generate_text_pair():
    """Two text-like files where a few characters were swapped or corrupted."""
    original = (
        b"The quick brown fox jumps over the lazy dog.\n"
        b"Pack my box with five dozen liquor jugs.\n"
        b"How vexingly quick daft zebras jump!\n"
        b"The five boxing wizards jump quickly.\n"
    )
    modified = bytearray(original)
    modified[4] = ord("Q")  # quick -> Quick
    modified[10] = ord("B")  # brown -> Brown
    modified[35] = ord("L")  # lazy  -> Lazy
    modified[80] = ord("Z")  # zero-width change in second line
    modified[120] = ord("!")  # punctuation swap
    write_bin("text_original.bin", original)
    write_bin("text_modified.bin", bytes(modified))


def generate_identical_pair():
    """Two identical files — useful for verifying the 'no diffs' case."""
    data = bytes(range(256))
    write_bin("identical_a.bin", data)
    write_bin("identical_b.bin", data)


if __name__ == "__main__":
    print("Generating example binaries...")
    generate_simple_pair()
    generate_header_pair()
    generate_firmware_pair()
    generate_text_pair()
    generate_identical_pair()
    print("Done.")
