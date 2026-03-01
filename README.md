[![Build](https://github.com/ztroop/dead-ringer/actions/workflows/build.yml/badge.svg)](https://github.com/ztroop/dead-ringer/actions/workflows/build.yml)

# dead-ringer

A Rust-based command-line utility designed to compare two binary files, displaying differences by showcasing both hexadecimal and ASCII representations of the differing bytes.

![demo](./assets/demo.png)

## Features

- CLI Diff Viewer for Hex and ASCII.
- Color highlighting for different data types to enhance readability.
- Keyboard navigation enables interactive exploration of differences.
- Search for hex byte sequences (`/`) or ASCII strings (`?`), with `n`/`N` to cycle through matches.
- Displays byte offset for focused data, aiding in precise location identification.

## Installation

```sh
cargo install dead-ringer
```

## Usage

```
Usage: dring <file1> <file2>

Arguments:
  <file1>  Path to the first binary file
  <file2>  Path to the second binary file
```

**What is displayed:** For differing bytes at the same offset, the diff shows the byte from the first file. When the files differ in length, extra bytes from the longer file are shown.

## Keybindings

| Key | Action |
|-----|--------|
| `h/j/k/l` or arrow keys | Navigate |
| `/` | Search by hex bytes |
| `?` | Search by ASCII string |
| `Enter` | Submit search |
| `Tab` | Toggle between hex/ASCII search |
| `n` | Next match |
| `N` | Previous match |
| `v` | Enter visual selection mode |
| `y` | Copy selection as hex (OSC 52) |
| `Y` | Copy selection as ASCII (OSC 52) |
| `Esc` | Cancel search / selection |
| `q` | Quit |

### Clipboard (tmux)

Copy to clipboard uses the [OSC 52](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands) escape sequence, which works natively in most modern terminals (iTerm2, kitty, foot, WezTerm, etc.). If you run inside **tmux 3.3+**, add the following to `~/.tmux.conf` so the sequence is forwarded to the outer terminal:

```
set -g allow-passthrough on
```

Then reload with `tmux source-file ~/.tmux.conf`.

## Examples

The [`examples/`](./examples) directory contains pre-built binary file pairs for testing:

```sh
cargo run -- examples/simple_v1.bin examples/simple_v2.bin
cargo run -- examples/firmware_v1.bin examples/firmware_v2.bin
```

See [`examples/README.md`](./examples/README.md) for the full list of test pairs.

## Color Reference

|Type of Byte|Color|
|---|---|
|NULL|![#555753](https://placehold.co/10x10/555753/555753.png) Gray|
|OFFSET|![#555753](https://placehold.co/10x10/555753/555753.png) Gray|
|ASCII Printable|![#06989a](https://placehold.co/10x10/06989a/06989a.png) Cyan|
|ASCII Whitespace|![#4e9a06](https://placehold.co/10x10/4e9a06/4e9a06.png) Green|
|ASCII Other|![#4e9a06](https://placehold.co/10x10/4e9a06/4e9a06.png) Green|
|Non-ASCII|![#c4a000](https://placehold.co/10x10/c4a000/c4a000.png) Yellow|

## Alternatives

If you're looking for a full-featured Hex/ASCII viewer, check out [Hexyl](https://github.com/sharkdp/hexyl)!
