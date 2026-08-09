<!-- generated-by: gsd-doc-writer -->
# Development

## Local setup

```bash
git clone https://github.com/frogiraffe/chiki8.git
cd chiki8
cargo build
cargo test
```

SDL2 development libraries must be installed before the crate can link. No database, service, environment file, or code-generation step is required.

## Commands

| Command | Purpose |
|---------|---------|
| `cargo run -- -f path/to/game.ch8` | Run a development build with a ROM |
| `cargo build --release` | Produce an optimized executable |
| `cargo fmt --check` | Verify standard Rust formatting |
| `cargo test --locked` | Run the unit suite with locked dependencies |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Reject Clippy and compiler warnings |

## Code style

- Use rustfmt; do not hand-format around it.
- Keep the CPU model direct and deterministic.
- Put opcode behavior in the matching `op_*` handler in `src/cpu.rs`.
- Add one focused regression test for every non-trivial behavior change.
- Propagate errors at file and native-platform boundaries instead of adding global warning suppressions.

## Change workflow

1. Write a test that reproduces the incorrect machine-state transition.
2. Run the focused test and confirm the expected failure.
3. Make the smallest change in the shared handler or boundary.
4. Run formatting, the full test suite, and strict Clippy.
5. Keep documentation claims within the behavior those gates actually verify.

## Branch and pull requests

The default branch is `main`; no branch naming convention is documented. A pull request should explain the affected opcode or boundary, include a regression test when behavior changes, and pass every command in the table above.

## Project boundaries

The current modules already separate VM logic, configuration, audio, and the SDL runtime. New traits, factories, or framework layers should be added only when a second real implementation requires them.
