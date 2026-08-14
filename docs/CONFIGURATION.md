# Configuration

chiki8 reads `chiki8.toml` from the current directory by default. You can pass a custom config path using `--config PATH`.

CLI arguments take precedence over values defined in `chiki8.toml`.

## Example Configuration

```toml
[display]
scale = 15
filter = "nearest"
integer_scaling = false
background = [0, 0, 0]
foreground = [255, 255, 255]

[audio]
volume = 25

[emulation]
speed = 10
refresh = 60
profile = "classic"
```

## Settings Reference

| Setting | Type | Default | CLI Flag | Description |
|---|---|---|---|---|
| `display.scale` | integer | `15` | `-s`, `--scale` | Pixel scaling factor |
| `display.filter` | string | `"nearest"` | `--filter` | Texture filter: `"nearest"` or `"linear"` |
| `display.integer_scaling` | boolean | `false` | `--integer-scaling` | Enable integer scaling |
| `display.background` | RGB array | `[0, 0, 0]` | `-b`, `--background` | Background color (R,G,B) |
| `display.foreground` | RGB array | `[255, 255, 255]` | `-c`, `--color` | Foreground color (R,G,B) |
| `audio.volume` | integer | `25` | `-v`, `--volume` | Volume from `0` to `100` |
| `emulation.speed` | integer | `10` | `-p`, `--speed` | CPU cycles executed per frame |
| `emulation.refresh` | integer | `60` | `--refresh` | Display refresh rate: `30`, `60`, or `120` |
| `emulation.profile` | string | `"classic"` | `--profile` | Emulation profile: `"classic"` or `"superchip-1.1"` |
| `keymap.keys` | table | standard layout | — | Key mappings from keyboard keys to CHIP-8 keys |

## Key Mapping

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

## Generating a Default Configuration

To generate a starter `chiki8.toml` file in the current directory:

```bash
cargo run -- --create-config
```
