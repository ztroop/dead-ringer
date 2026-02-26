# Example Binaries

Pre-built binary file pairs for testing `dring`.

## File Pairs

| Pair | Size | Description |
|------|------|-------------|
| `simple_v1.bin` / `simple_v2.bin` | 64 B | Sequential bytes `0x00–0x3F` with 5 scattered modifications. Good smoke test. |
| `header_original.bin` / `header_modified.bin` | 126 B | Simulated ELF-like binary where the magic bytes and version field differ. |
| `firmware_v1.bin` / `firmware_v2.bin` | 512 B | Pseudorandom data with 80 byte differences throughout, simulating a firmware update. |
| `text_original.bin` / `text_modified.bin` | 161 B | ASCII text with a few character case-swaps and punctuation changes. |
| `identical_a.bin` / `identical_b.bin` | 256 B | Identical files — verifies the zero-diff case. |

## Usage

```sh
cargo run -- examples/simple_v1.bin examples/simple_v2.bin
cargo run -- examples/firmware_v1.bin examples/firmware_v2.bin
```

## Regenerating

The files are deterministic (seeded RNG). Regenerate them with:

```sh
python3 examples/generate.py
```
