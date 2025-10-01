# Contributing

Thanks for your interest in contributing to spacetime_tiled!

## Setup

1. Fork and clone the repository
2. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. Install the WASM target: `rustup target add wasm32-unknown-unknown`
4. Install LLVM/clang (for the zstd-sys dependency)
   - macOS: `brew install llvm`
   - Ubuntu/Debian: `sudo apt-get install llvm clang`
   - Windows: Download from https://releases.llvm.org/

## Building

```bash
# Check syntax (fast)
cargo check

# Build the library
cargo build --lib

# Run tests
cargo test --lib
```

Note: Regular `cargo build` will fail with linker errors because the library has `crate-type = ["cdylib", "rlib"]`. This is expected - use `cargo check` for development.

## Testing with SpacetimeDB

```bash
cd examples/simple_game/server
spacetime build
spacetime publish simple-game
spacetime call simple-game load_demo_map
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write doc comments for public APIs
- Use descriptive variable names
- Keep functions focused and small

## Writing Docs

- Use natural language, not marketing-speak
- Be honest about limitations
- Include code examples that actually work
- Mention gotchas and common mistakes
- Test your examples

## Pull Requests

1. Create a feature branch: `git checkout -b feature-name`
2. Make your changes
3. Add tests if applicable
4. Run `cargo fmt` and `cargo clippy`
5. Update documentation if needed
6. Commit with a clear message
7. Push and create a PR

## Reporting Bugs

Include:
- spacetime_tiled version
- SpacetimeDB version
- Rust version
- OS
- Minimal reproduction steps
- Error messages or unexpected behavior

## Suggesting Features

Explain:
- What problem it solves
- Your proposed solution
- Why it can't be done with current features
- Whether you're willing to implement it

## Areas That Need Help

- Support for base64/gzip tile encoding
- Polygon/polyline vertex data storage
- Tile animation support
- External tileset (.tsx) handling
- More comprehensive examples
- Performance improvements

## Questions?

Open an issue or discussion on GitHub.
