#!/usr/bin/env bash
# tools/doom/build.sh — fetch + cross-compile doomgeneric for vibeos (the
# foreign-binary demo). Off the critical path; only needed for the doom payload
# opt-in image. Full bring-up story + why each flag is load-bearing:
# docs/engineering/doom-on-vibeos.md.
#
# Usage:
#   tools/doom/build.sh [arm64|amd64|i386|all]   # default: arm64
#
# Outputs (consumed via tools/payloads/doom/payload.yaml when VIBEOS_PAYLOAD=doom):
#   tools/doom/doom-<arch>.exe
#   tools/doom/doom1.wad          (shareware IWAD)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
SRCDIR="$HERE/doomgeneric/doomgeneric"
WAD="$HERE/doom1.wad"

ARCHES="${1:-arm64}"
[ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"

declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )

# --- fetch doomgeneric (once) ---------------------------------------------
if [ ! -d "$HERE/doomgeneric/.git" ]; then
    echo "[doom] cloning doomgeneric..."
    git clone --depth 1 https://github.com/ozkl/doomgeneric.git "$HERE/doomgeneric"
fi
# doomgeneric_win.c ships `#include <Windows.h>` — case-sensitive fs / mingw
# headers want lowercase. Idempotent.
sed -i 's/#include <Windows.h>/#include <windows.h>/' "$SRCDIR/doomgeneric_win.c"

# --- fetch shareware IWAD (once) ------------------------------------------
# The slitaz mirror this used to fetch from is GONE (verified 2026-09-20: a
# plain 404, not a transient failure), which is why doom could not be built
# here at all and the ladder entry said so.
if [ ! -f "$WAD" ]; then
    echo "[doom] fetching shareware doom1.wad..."
    curl -fsSL -o "$WAD" https://github.com/Akbar30Bill/DOOM_wads/raw/master/doom1.wad
fi
# md5 of the canonical shareware IWAD (v1.9), 4196020 bytes.
#
# This used to WARN and continue. It now FAILS, because the source above is a
# third-party mirror rather than a distro package tree: an unverified binary
# from a mirror is exactly the case where "carry on with whatever arrived" is
# wrong, and a bad IWAD would surface as some baffling in-game fault far from
# here. The hash is what makes the mirror safe to use at all -- it is the
# canonical shareware WAD or the build stops.
EXPECT=f0cefca49926d00903cf57551d901abe
GOT="$(md5sum "$WAD" | cut -d' ' -f1)"
if [ "$GOT" != "$EXPECT" ]; then
    echo "[doom] FATAL: doom1.wad md5 $GOT != expected $EXPECT" >&2
    echo "[doom]   $WAD is not the canonical shareware IWAD -- refusing to build." >&2
    echo "[doom]   Delete it to re-fetch, or point the URL above at a good copy." >&2
    exit 1
fi

# doomgeneric source set with the Win32/GDI backend (doomgeneric_win.c) instead
# of the X11 one (doomgeneric_xlib.c).
SRC=$(printf '%s\n' \
    dummy am_map doomdef doomstat dstrings d_event d_items d_iwad d_loop d_main \
    d_mode d_net f_finale f_wipe g_game hu_lib hu_stuff info i_cdmus i_endoom \
    i_joystick i_scale i_sound i_system i_timer memio m_argv m_bbox m_cheat \
    m_config m_controls m_fixed m_menu m_misc m_random p_ceilng p_doors p_enemy \
    p_floor p_inter p_lights p_map p_maputl p_mobj p_plats p_pspr p_saveg \
    p_setup p_sight p_spec p_switch p_telept p_tick p_user r_bsp r_data r_draw \
    r_main r_plane r_segs r_sky r_things sha1 sounds statdump st_lib st_stuff \
    s_sound tables v_video wi_stuff w_checksum w_file w_main w_wad z_zone \
    w_file_stdc i_input i_video doomgeneric doomgeneric_win | sed 's/$/.c/')

for arch in $ARCHES; do
    cc="${CC[$arch]:-}"
    [ -n "$cc" ] || { echo "[doom] unknown arch '$arch'"; exit 1; }
    out="$HERE/doom-$arch.exe"
    echo "[doom] building $arch -> $out"
    ( cd "$SRCDIR" && "$TC/$cc" -O0 -w -o "$out" $SRC \
        -lmsvcrt-os -luser32 -lgdi32 \
        -Wl,--subsystem,windows -Wl,-e,mainCRTStartup )
    # Guards (see doom-on-vibeos.md): must be GUI subsystem (2), must import
    # classic msvcrt.dll (not UCRT api-sets), must have NO `b .` self-branch
    # (the clang aarch64 -Os/-O2 pointer-chase miscompile; -O0 avoids it).
    objd="$TC/${cc%-gcc}-objdump"
    # NB: no early `exit` in awk — under `set -o pipefail` an early awk exit
    # SIGPIPEs objdump (141) and set -e kills the whole script (bit us
    # 2026-07-06: `build.sh all` silently truncated to just arm64).
    ss=$("$objd" -p "$out" | awk '$1=="Subsystem" && !p {print $2; p=1}')
    [ "$ss" = "00000002" ] || echo "[doom] WARN($arch): subsystem=$ss (want 00000002/GUI)"
    # NB: `|| true` — under `set -e` a no-match grep (the GOOD case) would
    # otherwise kill the script here, silently truncating an `all` run to
    # just the first arch (bit us 2026-07-06).
    "$objd" -p "$out" | grep -qi "api-ms-win-crt" \
        && echo "[doom] WARN($arch): links UCRT api-sets (want classic msvcrt.dll)" \
        || true
    if [ "$arch" = "arm64" ]; then
        n=$("$objd" -d "$out" | grep -cE '^\s+[0-9a-f]+:\s+14000000' || true)
        [ "$n" = "0" ] || echo "[doom] WARN($arch): $n 'b .' self-branch(es) — optimizer miscompile, rebuild -O0"
    fi
done

echo "[doom] done. Enable in an image with: VIBEOS_PAYLOAD=doom ./vibeos-cli build --arch <arch>"
