# Conformance oracle derivation

The expected hashes in `timendus-chip8-test-suite.tsv` are approvals of the
logical monochrome framebuffer, not baselines captured from chiki8.

Source: Timendus/chip8-test-suite commit
`cb24d5595384a80b49ddedae13bec4042b16d41d`. The first four records are
reproducible directly from the pinned upstream reference PNGs:

```sh
for name in chip-8-logo ibm-logo corax+ flags; do
  magick "pictures/$name.png" -sample 64x32 -background black -alpha remove \
    -colorspace gray -depth 8 gray:- |
    od -An -v -tu1 |
    awk '{for(i=1;i<=NF;i++) printf "%d", ($i + 0 > 127 ? 1 : 0)}' |
    shasum -a 256
done
```

| Manifest case | Independent upstream checkpoint |
|---|---|
| chip8-logo | `pictures/chip-8-logo.png`, README “CHIP-8 logo” at 39 cycles |
| ibm-logo | `pictures/ibm-logo.png`, README “IBM logo” at 20 cycles |
| corax-opcodes | `pictures/corax+.png`, README “Corax+ opcode test” result |
| flags | `pictures/flags.png`, README “Flags test” result |
| classic-quirks | `docs/5-quirks.html`, selector 1, inspected result screen |
| superchip-quirks | `legacy-superchip.md`, HP48SX result, selector 4 |
| superchip-scroll-lores | `pictures/HP48SX-scrolling-lores.JPG`, selector 2 |
| superchip-scroll-hires | `pictures/HP48SX-scrolling-hires.JPG`, selector 3 |

For interactive cases, run the pinned upstream `docs/*.html` Octo build, apply
the manifest selector, advance to the named result screen, export the logical
frame, and hash its row-major `0`/`1` bytes with `shasum -a 256`. Oracle changes
require repeating that upstream-only procedure and reviewing the decoded frame;
running chiki8 is explicitly not an oracle-update procedure.
