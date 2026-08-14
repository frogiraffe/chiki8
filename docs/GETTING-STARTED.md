# Getting Started

## Prerequisites

- Git
- A stable Rust toolchain installed through [rustup](https://rustup.rs/)
- SDL2 development libraries
- A legally obtained CHIP-8 ROM

Install SDL2 with the platform package manager:

```bash
# Ubuntu / Debian
sudo apt-get install libsdl2-dev

# Fedora
sudo dnf install SDL2-devel

# Arch Linux
sudo pacman -S sdl2

# macOS with Homebrew
brew install sdl2
```

Windows users need an SDL2 development package and runtime DLL discoverable by the Rust linker and the built executable.

## Installation

```bash
git clone https://github.com/frogiraffe/chiki8.git
cd chiki8
cargo build --release
```

## First run

```bash
cargo run --release -- -f path/to/game.ch8
```

Add `--profile classic` or `--profile superchip-1.1` to select a profile explicitly instead of relying on automatic detection.

Press `Esc` or close the window to stop the emulator.

## Create a configuration

```bash
cargo run -- --create-config
```

This writes `chiki8.toml` in the current directory. Edit it, then either keep it at that default path or pass another file with `--config`.

## Common setup issues

- `cannot find -lSDL2`: install the SDL2 development package, not only the runtime library.
- No window appears over SSH or in a container: SDL2 needs an available native video/display session.
- `Could not read ROM`: verify the path and file permissions. ROMs larger than 3,584 bytes are rejected.
- Keys do not match a custom layout: verify the names and values under `[keymap.keys]` in the active TOML file.

## Next steps

- [Configuration](CONFIGURATION.md) — settings, defaults, and CLI precedence
- [Architecture](ARCHITECTURE.md) — machine model, components, and data flow
