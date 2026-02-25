# Educational Game

An educational game built with [Bevy](https://bevyengine.org) and [Rust](https://www.rust-lang.org). Desktop-only application.

## Getting Started

Run the desktop application:

```bash
cargo run
```

### Development

```bash
cargo check                                                    # Type-check without building
cargo clippy --all-targets --all-features -- -D warnings       # Lint
cargo fmt                                                      # Auto-format
```

### Release Build

```bash
make release
```

This builds the release binary and patches dylib paths for macOS distribution.
