# Architecture

chiki8 is a CHIP-8 and SUPER-CHIP emulator written in Rust using SDL2 for windowing, input, graphics, and audio.

## Components

```text
ROM File + chiki8.toml + CLI Flags
                 │
                 ▼
          ┌───────────────┐
          │   CHIP-8 CPU  │ ◄────── Keypad input (SDL2)
          │   src/cpu.rs  │
          └───────┬───────┘
                  │
        ┌─────────┴─────────┐
        ▼                   ▼
   Framebuffer         Sound Timer
   (128×64)               (60 Hz)
        │                   │
        ▼                   ▼
  SDL2 Renderer        SDL2 Audio
```

## Static Memory Layout

The machine state is modeled directly in `src/cpu.rs`:

| Component | Size | Description |
|---|---|---|
| **RAM** | 4 KiB | Font sprites, ROM memory (starting at `0x200`), and working data |
| **Registers** | 16 × 8-bit | General purpose registers `V0` through `VF` |
| **Stack** | 16 × 16-bit | Subroutine call stack |
| **Program Counter (PC)** | 16-bit | Points to current instruction in RAM |
| **Index Register (I)** | 16-bit | Points to memory addresses for sprite/data access |
| **Timers** | 2 × 8-bit | 60 Hz Delay Timer (`DT`) and Sound Timer (`ST`) |
| **Display Buffer** | 128 × 64 | Monochrome framebuffer (supports 64×32 low-res and 128×64 high-res) |
| **RPL Flags** | 8 × 8-bit | In-memory flags for SUPER-CHIP save/load opcodes (`FX75`/`FX85`) |
| **Keypad** | 16 keys | Hexadecimal keypad state (`0x0` to `0xF`) |

## Independent CPU and Display Timing

The emulator decouples three separate clocks in `src/main.rs`:

1. **CPU Execution:** Runs at the configured cycle rate (default: 10 cycles per 60 Hz frame = ~600 Hz).
2. **Timers:** Delay and sound timers decrement at a fixed 60 Hz rate.
3. **Display Refresh:** Presentation runs at the configured refresh rate (30, 60, or 120 Hz).

This ensures that changing the display refresh rate does not alter game physics or timer speeds.
