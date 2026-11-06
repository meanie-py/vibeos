#!/usr/bin/env bash
# tools/mesa/build-mesa.sh — a modern software OpenGL for the guest: Mesa's gallium softpipe
# behind its WGL frontend, cross-built for i386/amd64/arm64 Windows. OUT-OF-BAND from the
# ReactOS cmake graph, like tools/fex/build-fex.sh.
#
# Why: the in-tree opengl32 is Mesa 2.6 (GL 1.1, CORE-7499), and wined3d (Wine 10.1) refuses
# any GL without GLSL, so d3d8/d3d9 CreateDevice can only ever fail. This is a TEST
# dependency: modrun stages it beside the d3d probes (opengl32 is not a KnownDLL, so the exe's
# directory wins), and nothing in the OS image changes.
#
# Usage: tools/mesa/build-mesa.sh {i386|amd64|arm64}
# Output: .cache/mesa/<arch>/{opengl32,libgallium_wgl,libz-1}.dll in this worktree.
#
# Every input is pinned by sha256. The built DLLs are cached under ~/.cache/vibeos-mesa/<key>,
# where <key> hashes this script and every input, so a second worktree copies instead of
# rebuilding (~10 min per arch) and any edit here builds afresh under a new key.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
A="${1:?arch: i386|amd64|arm64}"
declare -A TRIPLE=( [i386]=i686 [amd64]=x86_64 [arm64]=aarch64 )
declare -A FAMILY=( [i386]=x86 [amd64]=x86_64 [arm64]=aarch64 )
T="${TRIPLE[$A]:?unknown arch $A}"

MESA_VER=26.2.4
MESA_URL="https://archive.mesa3d.org/mesa-$MESA_VER.tar.xz"
MESA_SHA=bce5f7fbebb934373b86c999a064d52fb5065878dc57f287f95346648ec832e9
# The msvcrt flavour, not the repo's ucrt llvm-mingw: ReactOS has no api-ms-win-crt set, and
# libc++ built for ucrt imports from it.
TC_VER=20261006
TC_URL="https://github.com/mstorsjo/llvm-mingw/releases/download/$TC_VER/llvm-mingw-$TC_VER-msvcrt-ubuntu-22.04-x86_64.tar.xz"
TC_SHA=9dd3bbff7f746937c99b38322c8b1fcab8054b7d8d31ec840b3457234dff7e52
PY_PKGS=(meson==1.12.1 Mako==1.4.3 MarkupSafe==3.0.4 PyYAML==6.0.3 packaging==26.3)
DLLS=(opengl32.dll libgallium_wgl.dll libz-1.dll)

CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/vibeos-mesa"
KEY="$( { sha256sum "${BASH_SOURCE[0]}"; echo "$MESA_SHA $TC_SHA ${PY_PKGS[*]}"; } | sha256sum | cut -c1-16)"
OUT="$ROOT/.cache/mesa/$A"
DONE="$CACHE/$KEY/$A"

install_out() {
    mkdir -p "$OUT"
    for d in "${DLLS[@]}"; do cp -f "$DONE/$d" "$OUT/$d"; done
    echo "[mesa] $A: $OUT ($(cat "$DONE/KEY"))"
}
if [ -f "$DONE/KEY" ]; then install_out; exit 0; fi

mkdir -p "$CACHE/dl" "$CACHE/$KEY"
exec 9>"$CACHE/$KEY/.lock"
flock 9
if [ -f "$DONE/KEY" ]; then install_out; exit 0; fi

fetch() { # url sha dest
    if [ ! -f "$3" ] || ! echo "$2  $3" | sha256sum -c --quiet - 2>/dev/null; then
        curl -fsSL -o "$3.part" "$1"
        echo "$2  $3.part" | sha256sum -c --quiet - || { echo "FATAL: sha256 mismatch for $1" >&2; exit 1; }
        mv "$3.part" "$3"
    fi
}
fetch "$MESA_URL" "$MESA_SHA" "$CACHE/dl/mesa-$MESA_VER.tar.xz"
fetch "$TC_URL" "$TC_SHA" "$CACHE/dl/llvm-mingw-$TC_VER-msvcrt.tar.xz"

W="$CACHE/$KEY/work"
if [ ! -x "$W/tc/bin/clang" ]; then
    rm -rf "$W/tc" && mkdir -p "$W/tc"
    tar -xJf "$CACHE/dl/llvm-mingw-$TC_VER-msvcrt.tar.xz" -C "$W/tc" --strip-components=1
fi
if [ ! -f "$W/src/.patched" ]; then
    rm -rf "$W/src" && mkdir -p "$W/src"
    tar -xJf "$CACHE/dl/mesa-$MESA_VER.tar.xz" -C "$W/src" --strip-components=1
    # Mesa's MinGW setjmp workaround passes a jmp_buf to __builtin_setjmp, whose parameter is
    # void **, which this clang makes -Werror=incompatible-pointer-types; and on aarch64 clang
    # has no __builtin_setjmp at all, so that target keeps the plain setjmp (vtn only longjmps
    # on a malformed SPIR-V module, and wined3d feeds GL GLSL, not SPIR-V).
    H="$W/src/src/compiler/spirv/vtn_private.h"
    sed -i 's/^#if defined(__MINGW32__) && !defined(_UCRT)$/#if defined(__MINGW32__) \&\& !defined(_UCRT) \&\& !defined(__aarch64__)/;
            s/#define vtn_setjmp __builtin_setjmp$/#define vtn_setjmp(b) __builtin_setjmp((void **)(b))/;
            s/#define vtn_longjmp __builtin_longjmp$/#define vtn_longjmp(b, v) __builtin_longjmp((void **)(b), (v))/' "$H"
    grep -q '__builtin_setjmp((void \*\*)' "$H" && grep -q '!defined(__aarch64__)' "$H" \
        || { echo "FATAL: vtn_private.h patch did not apply" >&2; exit 1; }
    touch "$W/src/.patched"
fi
if [ ! -x "$W/py/bin/meson" ]; then
    rm -rf "$W/py"
    python3 -m venv "$W/py"
    "$W/py/bin/pip" -q install "${PY_PKGS[@]}"
fi

# arm64: QEMU runs -cpu cortex-a72 (ARMv8.0), so no newer baseline may leak in.
MARCH=""
[ "$A" = arm64 ] && MARCH="'-march=armv8-a'"
cat > "$W/cross-$A.txt" <<EOF
[binaries]
c = '$W/tc/bin/$T-w64-mingw32-clang'
cpp = '$W/tc/bin/$T-w64-mingw32-clang++'
ar = '$W/tc/bin/llvm-ar'
strip = '$W/tc/bin/llvm-strip'
windres = '$W/tc/bin/$T-w64-mingw32-windres'
pkg-config = '/bin/false'
[built-in options]
c_args = [$MARCH]
cpp_args = [$MARCH]
[host_machine]
system = 'windows'
cpu_family = '${FAMILY[$A]}'
cpu = '$T'
endian = 'little'
EOF

B="$W/build-$A"
rm -rf "$B"
PATH="$W/py/bin:$PATH" meson setup "$B" "$W/src" --cross-file "$W/cross-$A.txt" \
    --buildtype=release -Db_ndebug=true \
    -Dplatforms=windows -Dgallium-drivers=softpipe -Dvulkan-drivers= -Dllvm=disabled \
    -Dopengl=true -Dgles1=disabled -Dgles2=disabled -Degl=disabled -Dglx=disabled -Dgbm=disabled \
    -Dzstd=disabled -Dexpat=disabled -Dvideo-codecs= -Dshader-cache=disabled \
    -Dspirv-tools=disabled -Dmicrosoft-clc=disabled -Dgallium-d3d12-graphics=disabled \
    -Dbuild-tests=false -Dmin-windows-version=7
PATH="$W/py/bin:$PATH" nice ninja -C "$B"

STAGE="$CACHE/$KEY/.$A.tmp"
rm -rf "$STAGE" && mkdir -p "$STAGE"
cp "$B/src/gallium/targets/libgl-gdi/opengl32.dll" "$B/src/gallium/targets/wgl/libgallium_wgl.dll" \
   "$B/subprojects/zlib-1.3.1/libz-1.dll" "$STAGE/"
for d in "${DLLS[@]}"; do "$W/tc/bin/llvm-strip" "$STAGE/$d"; done
# Only the system DLLs ReactOS ships, and never the ucrt api set.
python3 "$ROOT/tools/payloads/pecheck.py" "$STAGE/libgallium_wgl.dll" --tag mesa --forbid-dll api-ms-win-crt
echo "$KEY mesa-$MESA_VER softpipe llvm-mingw-$TC_VER-msvcrt" > "$STAGE/KEY"
rm -rf "$DONE" && mv "$STAGE" "$DONE"
rm -rf "$B"
install_out
