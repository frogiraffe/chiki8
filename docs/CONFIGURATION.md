<!-- generated-by: gsd-doc-writer -->
# Configuration

## Sources and precedence

chiki8 reads `chiki8.toml` from the current directory by default. Pass `--config PATH` to use another file. CLI values for scale, speed, volume, background, and foreground override the loaded TOML values.

If the file does not exist, built-in defaults are used. If an existing file cannot be read or parsed, the error is printed and the emulator continues with built-in defaults.

## Minimal configuration

```toml
[display]
scale = 15
background = [0, 0, 0]
foreground = [255, 255, 255]

[audio]
volume = 25

[emulation]
speed = 10
```

All sections are optional. Missing sections and values use the defaults below.

## Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `display.scale` | integer | `15` | Window pixel scale |
| `display.background` | three bytes | `[0, 0, 0]` | Off-pixel RGB color |
| `display.foreground` | three bytes | `[255, 255, 255]` | On-pixel RGB color |
| `audio.volume` | integer | `25` | Percentage used by the square-wave callback; runtime audio is capped at 100 |
| `emulation.speed` | integer | `10` | CPU cycles executed per 60 Hz frame |
| `keymap.keys` | key/value table | classic layout | SDL keyboard name mapped to a CHIP-8 key value |

## Key mapping

```toml
[keymap.keys]
"1" = 0x1
"2" = 0x2
"3" = 0x3
"4" = 0xC
"Q" = 0x4
"W" = 0x5
"E" = 0x6
"R" = 0xD
"A" = 0x7
"S" = 0x8
"D" = 0x9
"F" = 0xE
"Z" = 0xA
"X" = 0x0
"C" = 0xB
"V" = 0xF
```

Recognized names are digits and uppercase Latin letters supported by `keycode_to_string` in `src/main.rs`. CHIP-8 values outside `0x0`–`0xF` are ignored safely.

## Generate the example

```bash
cargo run -- --create-config
```

This writes `chiki8.toml` in the current directory, replacing an existing file at that path.

## Environment variables

No environment variables are required or read by chiki8.
