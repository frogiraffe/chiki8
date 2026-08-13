#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

suite_commit=cb24d5595384a80b49ddedae13bec4042b16d41d
suite_url=https://github.com/Timendus/chip8-test-suite.git
case_id=chip8-logo
output=target/conformance-tracer.jsonl

while (($#)); do
    case $1 in
        --case) case_id=${2:?missing case}; shift 2 ;;
        --output) output=${2:?missing output}; shift 2 ;;
        *) printf 'unknown argument: %s\n' "$1" >&2; exit 2 ;;
    esac
done

[[ $case_id == chip8-logo ]] || { printf 'unknown case: %s\n' "$case_id" >&2; exit 2; }
mkdir -p "$(dirname -- "$output")"
: >"$output"
temp_dir=$(mktemp -d)
trap 'rm -rf "$temp_dir"' EXIT
suite=$temp_dir/suite
rom=$suite/bin/1-chip8-logo.ch8
capture=$temp_dir/frame.bmp
expected_rom_hash=15f7fb887ea4cb8e40615bb20b0cfd993ef55ca84c535c8c8f3fafa8bf129724
# Upstream's documented 39-cycle CHIP-8 logo, decoded to 64x32 row-major bits.
expected_frame_hash=2a73bb554cfb8b2a5eb96c8a706faca5799c0fb9b031a786f966895848e308e8

record() {
    local status=$1 observed=${2:-} message=${3:-}
    printf '{"case":"%s","checkpoint":"upstream-39-cycles","revision":"%s","profile":"classic","expected":"%s","observed":"%s","status":"%s","message":"%s"}\n' \
        "$case_id" "$suite_commit" "$expected_frame_hash" "$observed" "$status" "$message" >>"$output"
}

fail_case() {
    record fail "${2:-}" "$1"
    printf 'FAIL %s: %s\n' "$case_id" "$1" >&2
    exit 1
}

git init -q "$suite"
git -C "$suite" fetch -q --depth 1 "$suite_url" "$suite_commit"
git -C "$suite" checkout -q --detach FETCH_HEAD
[[ $(git -C "$suite" rev-parse HEAD) == "$suite_commit" ]] || fail_case 'suite revision mismatch'
actual_rom_hash=$(sha256sum "$rom" | cut -d ' ' -f 1)
[[ $actual_rom_hash == "$expected_rom_hash" ]] || fail_case 'ROM SHA-256 mismatch' "$actual_rom_hash"

cargo build --locked --quiet
env SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy target/debug/chiki8 \
    --file "$rom" --profile classic --cycles 39 --color 255,255,255 \
    --background 0,0,0 --capture-frame "$capture" >"$temp_dir/emulator.log" 2>&1 || {
        cat "$temp_dir/emulator.log" >&2
        fail_case 'emulator exited unsuccessfully'
    }

# Normalize the 15x-scaled RGB24 BMP to one ASCII bit per logical 64x32 pixel.
pixel_offset=$(od -An -tu4 -j10 -N4 "$capture" | tr -d ' ')
observed=$(od -An -v -tu1 -j"$pixel_offset" "$capture" | awk '
    { for (i = 1; i <= NF; i++) bytes[count++] = $i }
    END {
        for (y = 0; y < 32; y++) {
            source_y = (31 - y) * 15;
            for (x = 0; x < 64; x++) {
                pos = (source_y * 960 + x * 15) * 3;
                printf "%d", bytes[pos] == 255;
            }
        }
    }
' | sha256sum | cut -d ' ' -f 1)

[[ $observed == "$expected_frame_hash" ]] || fail_case 'framebuffer SHA-256 mismatch' "$observed"
record pass "$observed"
