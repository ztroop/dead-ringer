# Dead Ringer

A command-line tool written in Rust that compares two binary files and displays the differences. It uses a terminal-based user interface to present a side-by-side comparison, showing both hexadecimal and ASCII representations of the differing bytes.

## Installation

```sh
git clone git@github.com:ztroop/dead-ringer.git && cd ./dead-ringer
cargo install --path .
```

## Usage

```sh
Usage: dring <file1> <file2>

Arguments:
  <file1>  Path to the first binary file
  <file2>  Path to the second binary file

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Demonstration

![demo](./assets/demo.png)