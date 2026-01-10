# chiki8 🎮

A CHIP-8 emulator written in Rust.

## Features

- Full CHIP-8 instruction set implementation
- Configurable display scaling and colors
- Adjustable emulation speed
- Sound support with volume control
- TOML configuration file support
- Customizable keyboard mapping

## Installation

```bash
cargo build --release
```

## Usage

```bash
cargo run -- -f <rom_file> [options]
```

### Options

| Option | Description |
|--------|-------------|
| `-f, --file` | ROM file (required) |
| `-s, --scale` | Pixel scale (default: 15) |
| `-p, --speed` | Speed multiplier (default: 10) |
| `-v, --volume` | Volume level 0-100 (default: 25) |
| `-b, --background` | Background color R,G,B (default: 0,0,0) |
| `-c, --color` | Foreground color R,G,B (default: 255,255,255) |
| `--config` | Config file path |
| `--create-config` | Create example config file |

### Examples

```bash
# Run a ROM with default settings
cargo run -- -f roms/Pong\ \(1\ player\).ch8

# Run with custom scale and speed
cargo run -- -f roms/Space\ Invaders\ \[David\ Winter\].ch8 -s 20 -p 15

# Run with custom colors (green on black)
cargo run -- -f roms/game.ch8 -c 0,255,0 -b 0,0,0
```

## Keyboard Mapping

```
CHIP-8:          Keyboard:
┌───┬───┬───┬───┐    ┌───┬───┬───┬───┐
│ 1 │ 2 │ 3 │ C │    │ 1 │ 2 │ 3 │ 4 │
├───┼───┼───┼───┤    ├───┼───┼───┼───┤
│ 4 │ 5 │ 6 │ D │    │ Q │ W │ E │ R │
├───┼───┼───┼───┤    ├───┼───┼───┼───┤
│ 7 │ 8 │ 9 │ E │    │ A │ S │ D │ F │
├───┼───┼───┼───┤    ├───┼───┼───┼───┤
│ A │ 0 │ B │ F │    │ Z │ X │ C │ V │
└───┴───┴───┴───┘    └───┴───┴───┴───┘
```

Press `ESC` to exit.

## Configuration

Create a config file with:

```bash
cargo run -- --create-config
```

This generates `chiki8.toml` with customizable settings for display, audio, emulation speed, and key mappings.

## References

- [CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) by Jean-François Verrnay
- [Write a Chip-8 Emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/) by Tobias Vermeulen
- [Chip-8 Book](https://github.com/aquova/chip8-book) by aquova

## License

MIT
