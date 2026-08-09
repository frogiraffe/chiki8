<!-- generated-by: gsd-doc-writer -->
# Architecture

## System overview

chiki8 is a native desktop emulator with a small state-machine core and an SDL2 shell. A ROM is loaded into the classic CHIP-8 program region, CPU instructions mutate fixed-size machine state, and a 60 Hz runtime loop turns that state into video, input, timers, and audio.

## Components

```text
                         ┌─────────────────┐
CLI + chiki8.toml ─────► │ Runtime loop    │ ◄──── SDL2 events
                         │ src/main.rs     │
                         └───────┬─────────┘
                                 │ cycles
                                 ▼
ROM file ───────────────► ┌─────────────────┐
                          │ CHIP-8 CPU      │
                          │ src/cpu.rs      │
                          └───┬─────────┬───┘
                              │         │
                    framebuffer         sound timer
                              │         │
                              ▼         ▼
                        SDL2 renderer  SDL2 audio
                        src/main.rs    src/audio.rs
```

## Runtime data flow

1. `src/main.rs` parses command-line options and loads `chiki8.toml` through `Config::load`.
2. `Cpu::load` reads the ROM and rejects data larger than the 3,584-byte program region.
3. SDL2 creates the audio device, native window, canvas, and event pump.
4. Each frame processes key events, executes the configured number of CPU cycles, and draws the 64×32 framebuffer.
5. Delay and sound timers decrement once per frame; the audio device follows the sound timer.
6. The loop sleeps for the remaining part of a roughly 16.667 ms frame.

## Core model

`src/cpu.rs` keeps the machine model explicit with fixed-size arrays:

| State | Size | Purpose |
|-------|------|---------|
| Memory | 4 KiB | Font sprites, ROM, and working memory |
| Registers | 16 × 8-bit | `V0` through `VF` |
| Stack | 16 × 16-bit | Subroutine return addresses |
| Display | 64 × 32 booleans | Monochrome framebuffer |
| Keypad | 16 booleans | Current CHIP-8 key state |

Opcode dispatch uses masks in `Cpu::decode_opcode`; each of the 34 handlers owns one instruction transition. This direct design keeps behavior visible and testable without an abstraction layer.

## Module boundaries

- `src/cpu.rs` — VM state, opcode decoding, timers, ROM bounds, and unit tests
- `src/config.rs` — TOML schema, defaults, key mapping, and example generation
- `src/audio.rs` — SDL2 square-wave callback and playback device
- `src/main.rs` — CLI, composition, native I/O, frame pacing, and rendering

## Compatibility decisions

- Program memory begins at `0x200`; font sprites occupy the beginning of memory.
- `8XY6` and `8XYE` shift `VX` directly.
- `DXYN` wraps pixels at both screen edges and reports collisions through `VF`.
- `FX55` and `FX65` do not increment `I`.
- SUPER-CHIP, XO-CHIP, and runtime-selectable quirk profiles are outside the current scope.
