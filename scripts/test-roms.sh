#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"
manifest=${CHIKI8_MANIFEST:-tests/fixtures/vendor/timendus-chip8-test-suite.tsv}
output=target/conformance.jsonl
requested_case=
mode=run
license_file=tests/fixtures/vendor/LICENSES/Timendus-chip8-test-suite-GPL-3.0.txt
license_sha256=3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}';
    else shasum -a 256 "$1" | awk '{print $1}'; fi
}

sha256_stream() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum | awk '{print $1}';
    else shasum -a 256 | awk '{print $1}'; fi
}

while (($#)); do
    case $1 in
        --case) requested_case=${2:?missing case}; shift 2 ;;
        --output) output=${2:?missing output}; shift 2 ;;
        --verify-inputs) mode=verify; shift ;;
        --self-test) mode=self-test; shift ;;
        *) printf 'unknown argument: %s\n' "$1" >&2; exit 2 ;;
    esac
done

verify_inputs() {
    [[ -f $manifest ]] || { printf 'manifest missing: %s\n' "$manifest" >&2; return 1; }
    local header suite_header suite_revision
    suite_header=$'# suite_url=https://github.com/Timendus/chip8-test-suite.git\tsuite_commit=cb24d5595384a80b49ddedae13bec4042b16d41d\tsuite_license=GPL-3.0-only\tsuite_license_path=LICENSE'
    [[ $(sed -n '1p' "$manifest") == "$suite_header" ]] || { printf 'manifest suite header mismatch\n' >&2; return 1; }
    header=$'#case_id\tsource_url\tcommit\trom_path\tsha256\tlicense\tlicense_file\tauthor_project\tprofile\tselector\tschedule\tcheckpoint\tpurpose\tclassification\tdimensions\texpected'
    [[ $(sed -n '2p' "$manifest") == "$header" ]] || { printf 'manifest header mismatch\n' >&2; return 1; }
    suite_revision=$(sed -n '1s/.*suite_commit=\([^[:space:]]*\).*/\1/p' "$manifest")
    [[ $suite_revision =~ ^[0-9a-f]{40}$ ]] || { printf 'manifest suite commit invalid\n' >&2; return 1; }
    awk -F '\t' -v suite_revision="$suite_revision" '
        NR <= 2 { next }
        NF != 16 { print "manifest row " NR ": expected 16 fields" > "/dev/stderr"; bad=1; next }
        $1 !~ /^[A-Za-z0-9_.+-]+$/ || seen[$1]++ { print "manifest row " NR ": invalid/duplicate case_id" > "/dev/stderr"; bad=1 }
        $2 != "https://github.com/Timendus/chip8-test-suite.git" { print "manifest row " NR ": invalid source_url" > "/dev/stderr"; bad=1 }
        $3 !~ /^[0-9a-f]{40}$/ || $3 != suite_revision { print "manifest row " NR ": invalid revision" > "/dev/stderr"; bad=1 }
        $4 !~ /^bin\/[A-Za-z0-9+_.-]+\.ch8$/ { print "manifest row " NR ": unsafe rom_path" > "/dev/stderr"; bad=1 }
        $5 !~ /^[0-9a-f]{64}$/ { print "manifest row " NR ": invalid sha256" > "/dev/stderr"; bad=1 }
        $6 != "GPL-3.0-only" || $7 != "tests/fixtures/vendor/LICENSES/Timendus-chip8-test-suite-GPL-3.0.txt" || $8 != "Timendus/chip8-test-suite" || $11 !~ /^cycles:[1-9][0-9]*$/ || $12 == "" || $13 == "" { print "manifest row " NR ": invalid provenance field" > "/dev/stderr"; bad=1 }
        $9 != "classic" && $9 != "superchip-1.1" { print "manifest row " NR ": invalid profile" > "/dev/stderr"; bad=1 }
        $10 != "none" && $10 !~ /^memory-0x1ff=[1-5]$/ { print "manifest row " NR ": invalid selector" > "/dev/stderr"; bad=1 }
        $12 !~ /^[A-Za-z0-9_.+ :\/-]+$/ || $13 !~ /^[A-Za-z0-9_.+ :\/-]+$/ { print "manifest row " NR ": JSON-unsafe checkpoint or purpose" > "/dev/stderr"; bad=1 }
        $14 != "test-only-transient" { print "manifest row " NR ": invalid classification" > "/dev/stderr"; bad=1 }
        $15 != "64x32" && $15 != "128x64" { print "manifest row " NR ": invalid dimensions" > "/dev/stderr"; bad=1 }
        $16 !~ /^[0-9a-f]{64}$/ { print "manifest row " NR ": invalid expected digest" > "/dev/stderr"; bad=1 }
        END { exit bad }
    ' "$manifest" || return 1
    [[ -f $license_file ]] || { printf 'upstream license copy missing\n' >&2; return 1; }
    [[ $(sha256_file "$license_file") == "$license_sha256" ]] || { printf 'upstream license copy SHA-256 mismatch\n' >&2; return 1; }
    if git ls-files 'tests/fixtures/vendor/*.ch8' 'tests/fixtures/vendor/**/*.ch8' | grep -q .; then
        printf 'tracked external .ch8 fixture violates transient policy\n' >&2; return 1
    fi
}

verify_checkout() {
    [[ $(git -C "$1" rev-parse HEAD 2>/dev/null) == "$2" ]] || { printf 'suite revision mismatch\n' >&2; return 1; }
}

verify_rom() {
    local actual
    actual=$(sha256_file "$1")
    [[ $actual == "$2" ]] || { printf 'ROM SHA-256 mismatch: %s\n' "$actual" >&2; return 1; }
}

self_test() {
    local fixture=$temp_dir/manifest.tsv checkout=$temp_dir/checkout
    local mutations=('3s/test-only-transient/release-asset/' '3s/upstream-39-cycles/upstream-"39-cycles/' '4s/^ibm-logo/chip8-logo/' '5s/[0-9a-f]\{64\}/bad-hash/' '6s/cb24d5595384a80b49ddedae13bec4042b16d41d/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/')
    for mutation in "${mutations[@]}"; do
        sed "$mutation" tests/fixtures/vendor/timendus-chip8-test-suite.tsv >"$fixture"
        if CHIKI8_MANIFEST=$fixture "$0" --verify-inputs >"$temp_dir/self-test.log" 2>&1; then
            printf 'self-test: malformed manifest unexpectedly passed: %s\n' "$mutation" >&2; return 1
        fi
    done
    git init -q "$checkout"
    printf x >"$checkout/rom.ch8"
    git -C "$checkout" add rom.ch8
    git -C "$checkout" -c user.name=chiki8 -c user.email=tests@invalid commit -qm fixture
    ! verify_checkout "$checkout" aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa >/dev/null 2>&1 || return 1
    ! verify_rom "$checkout/rom.ch8" aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa >/dev/null 2>&1 || return 1
}

if [[ $mode == verify ]]; then verify_inputs; exit; fi
if [[ $mode == self-test ]]; then
    temp_dir=$(mktemp -d); trap 'rm -rf "$temp_dir"' EXIT
    self_test; exit
fi

verify_inputs
mkdir -p "$(dirname -- "$output")"
: >"$output"
temp_dir=$(mktemp -d)
trap 'rm -rf "$temp_dir"' EXIT
suite=$temp_dir/suite
suite_url=https://github.com/Timendus/chip8-test-suite.git
suite_commit=$(sed -n '1s/.*suite_commit=\([^[:space:]]*\).*/\1/p' "$manifest")
git init -q "$suite"
git -C "$suite" fetch -q --depth 1 "$suite_url" "$suite_commit"
git -C "$suite" checkout -q --detach FETCH_HEAD
verify_checkout "$suite" "$suite_commit" || exit 1
cargo build --locked --quiet

normalize_frame() {
    local bmp=$1 dimensions=$2 width height sample_x sample_y pixel_offset
    width=${dimensions%x*}; height=${dimensions#*x}
    sample_x=$((128 / width)); sample_y=$((64 / height))
    pixel_offset=$(od -An -tu4 -j10 -N4 "$bmp" | tr -d ' ')
    od -An -v -tu1 -j"$pixel_offset" "$bmp" | awk -v width="$width" -v height="$height" -v sx="$sample_x" -v sy="$sample_y" '
        { for (i = 1; i <= NF; i++) bytes[count++] = $i }
        END {
            for (y = 0; y < height; y++) {
                source_y = 63 - y * sy;
                for (x = 0; x < width; x++) {
                    pos = (source_y * 128 + x * sx) * 3;
                    printf "%d", bytes[pos] == 255;
                }
            }
        }
    ' | sha256_stream
}

observe_capture() {
    [[ -r $1 ]] || { printf 'capture missing: %s\n' "$1" >&2; return 1; }
    normalize_frame "$1" "$2"
}

failures=0
selected=0
while IFS=$'\t' read -r case_id source commit rom_path rom_hash license license_file author profile selector schedule checkpoint purpose classification dimensions expected; do
    [[ $case_id == case_id || $case_id == \#* || -z $case_id ]] && continue
    [[ -z $requested_case || $requested_case == "$case_id" ]] || continue
    selected=$((selected + 1))
    rom=$suite/$rom_path
    capture=$temp_dir/$case_id.bmp
    observed=
    status=pass
    message=
    preparation=none
    if ! verify_rom "$rom" "$rom_hash" 2>"$temp_dir/error"; then
        status=fail; message='ROM SHA-256 mismatch'
    else
        cycles=${schedule#cycles:}
        args=(--file "$rom" --profile "$profile" --cycles "$cycles" --speed 1000 --scale 2 --color 255,255,255 --background 0,0,0 --capture-frame "$capture")
        if [[ $selector != none ]]; then
            selector_value=${selector#*=}; args+=(--suite-selector "$selector_value"); preparation=$selector
        fi
        if ! env SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy target/debug/chiki8 "${args[@]}" >"$temp_dir/emulator.log" 2>&1; then
            status=fail; message='emulator exited unsuccessfully'
        elif ! observed=$(observe_capture "$capture" "$dimensions"); then
            status=fail; message='capture missing or unreadable'
        else
            if [[ $expected != discover && $observed != "$expected" ]]; then status=fail; message='framebuffer SHA-256 mismatch'; fi
        fi
    fi
    printf '{"case":"%s","checkpoint":"%s","revision":"%s","rom_sha256":"%s","profile":"%s","schedule":"%s","selector":"%s","preparation":"%s","expected":"%s","observed":"%s","status":"%s","message":"%s"}\n' \
        "$case_id" "$checkpoint" "$commit" "$rom_hash" "$profile" "$schedule" "$selector" "$preparation" "$expected" "$observed" "$status" "$message" >>"$output"
    if [[ $status == fail ]]; then failures=$((failures + 1)); printf 'FAIL %s: %s\n' "$case_id" "$message" >&2; fi
done <"$manifest"

((selected > 0)) || { printf 'no manifest case selected\n' >&2; exit 2; }
((failures == 0))
