<!-- generated-by: gsd-doc-writer -->
# chiki8

[![CI](https://github.com/frogiraffe/chiki8/actions/workflows/ci.yml/badge.svg)](https://github.com/frogiraffe/chiki8/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

A configurable desktop CHIP-8 emulator built with Rust and SDL2.

chiki8 keeps the emulator core deliberately small: a 4 KiB virtual machine, 34 instruction handlers, a 64×32 display, 60 Hz timers, and deterministic unit tests around the machine state. SDL2 handles the native window, keyboard, rendering, and square-wave audio.

## Highlights

- Classic CHIP-8 CPU, memory, stack, timers, keypad, font sprites, and XOR drawing
- Configurable scale, colors, emulation speed, volume, and key mapping
- Safe ROM-size validation against the 3,584-byte program region
- TOML configuration with CLI overrides
- 49 unit tests plus rustfmt and strict Clippy checks in CI

## Prerequisites

- A stable Rust toolchain
- SDL2 development libraries

On Ubuntu/Debian:

```bash
sudo apt-get install libsdl2-dev
```

See [Getting Started](docs/GETTING-STARTED.md) for other platforms and troubleshooting.

## Quick start

1. Build the emulator:

   ```bash
   cargo build --release
   ```

2. Run a legally obtained CHIP-8 ROM:

   ```bash
   cargo run --release -- -f path/to/game.ch8
   ```

ROM files are intentionally not included in this repository.

## Usage

```text
cargo run --release -- -f <rom_file> [options]

-f, --file FILE        ROM file (required)
-s, --scale SCALE      Pixel scale (default: 15)
-p, --speed SPEED      CPU cycles per frame (default: 10)
-v, --volume VOLUME    Volume from 0 to 100 (default: 25)
-b, --background COLOR Background color as R,G,B
-c, --color COLOR      Foreground color as R,G,B
    --profile PROFILE   Emulation profile (default: classic)
    --config PATH       TOML configuration path
    --create-config     Write an example chiki8.toml
    --help              Show help
```

Examples:

```bash
# Green pixels at 20× scale
cargo run --release -- -f path/to/game.ch8 -s 20 -c 0,255,0

# Use a custom configuration file
cargo run --release -- -f path/to/game.ch8 --config configs/fast.toml

# Select the Classic profile explicitly
cargo run --release -- -f path/to/game.ch8 --profile classic

# Generate a documented starter configuration
cargo run -- --create-config
```

## Keyboard

```text
CHIP-8           Keyboard
1 2 3 C          1 2 3 4
4 5 6 D          Q W E R
7 8 9 E          A S D F
A 0 B F          Z X C V
```

Press `Esc` or close the window to exit. Key assignments can be changed in TOML; see [Configuration](docs/CONFIGURATION.md).

## Architecture

```text
ROM + TOML + CLI
       │
       ▼
  CHIP-8 core ──► 64×32 framebuffer ──► SDL2 renderer
       │
       ├───────► keypad events ◄────── SDL2 input
       └───────► sound timer ─────────► SDL2 audio
```

The CPU core is independent of SDL and directly unit-tested. The runtime loop composes configuration, input, CPU cycles, rendering, timers, and audio. See [Architecture](docs/ARCHITECTURE.md) for the complete flow.

## Compatibility scope

The `classic` profile names chiki8's established modern Classic contract; it is not a claim of original COSMAC VIP behavior. Its five deterministically unit-tested choices are:

- Shift instructions use `VX` and ignore `Y`.
- `FX55` and `FX65` preserve `I`.
- `BNNN` uses `V0`.
- OR, AND, and XOR preserve `VF`.
- `DXYN` wraps at both display edges.

The project does not yet publish external compatibility ROM evidence. SUPER-CHIP remains unsupported and is not implemented; its profile and behavior are outside the current compatibility scope.

## Development

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

More detail is available in [Development](docs/DEVELOPMENT.md) and [Testing](docs/TESTING.md).

## References

- [CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Write a CHIP-8 Emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- [CHIP-8 Book](https://github.com/aquova/chip8-book)

## License

Licensed under the [MIT License](LICENSE).
