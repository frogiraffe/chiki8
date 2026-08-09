<!-- generated-by: gsd-doc-writer -->
# Testing

## Framework and setup

The project uses Rust's built-in test harness. The 49 unit tests live beside the CPU implementation in `src/cpu.rs`, so no additional test framework or fixture package is required.

Install SDL2 development libraries before compiling any test target, then resolve the locked Cargo dependencies normally.

## Running tests

Run the complete suite:

```bash
cargo test --locked
```

Run one test by name:

```bash
cargo test cpu::tests::new_cpu_contains_font_glyphs -- --exact
```

Run all quality gates used by CI:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

## What is covered

- Control flow, register loads, arithmetic, shifts, and carry/borrow flags
- Stack calls/returns, including all 16 levels
- Timers and no-underflow behavior
- Key state, key-skip instructions, and invalid keypad indices
- Display clearing and initial state
- Font initialization/reset and font-sprite addressing
- BCD conversion and register-memory transfer
- Oversized ROM rejection without partial memory mutation

## Writing tests

- Add tests to the `tests` module in `src/cpu.rs` while the crate remains a single binary.
- Name the incorrect behavior the test prevents.
- Use literal opcodes and hand-derived expected state.
- Prefer `execute_opcode` and the existing test-only accessors over exposing CPU internals publicly.
- Confirm a new regression test fails for the intended reason before implementing the fix.

## Current gaps

- No external ROM compatibility suite is checked into or run by CI.
- Random masking, sprite collision, and wait-for-key behavior lack direct tests.
- Audio, configuration parsing, CLI error paths, and native window behavior are not automated.
- No coverage percentage threshold is configured.

These gaps limit compatibility claims; they do not imply that untested behavior is broken.

## CI integration

`.github/workflows/ci.yml` runs formatting, all unit tests, and strict Clippy on pushes and pull requests using Ubuntu and SDL2 development libraries.
