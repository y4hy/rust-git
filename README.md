# gir (rust-git)
A learning-oriented, minimal Git implementation in Rust.

This project explores core Git plumbing concepts—object storage, hashing, and repository walking—implemented in Rust with a small CLI. It’s intended for educational purposes and experimentation, not as a drop-in replacement for Git.

---

## Highlights

- CLI built with `clap`.
- Foundations for Git-like operations:
  - SHA‑1 hashing (`sha1`)
  - zlib compression/decompression (`flate2`)
  - Binary parsing and byte order handling (`byteorder`)
  - Hex encoding/decoding (`hex`)
  - Filesystem traversal (`walkdir`)
- Index-related logic lives in [`src/index.rs`](src/index.rs).
- Subcommand scaffolding under [`src/commands/`](src/commands/).

---

## Getting Started

### Prerequisites
- Rust toolchain with Cargo installed.  
  You can install via [rustup](https://rustup.rs/).

### Build
```bash
cargo build
```

### Run
Show general help:
```bash
cargo run -- --help
```

Run the binary directly (after a successful build):
```bash
./target/debug/gir --help
```

Run a subcommand (examples, exact commands may change as the project evolves):
```bash
cargo run -- <subcommand> [options]
# e.g.
cargo run -- init
cargo run -- status
cargo run -- cat-file -p <object>
```

Because the CLI is powered by `clap`, you can also get help for individual subcommands:
```bash
cargo run -- <subcommand> --help
```
