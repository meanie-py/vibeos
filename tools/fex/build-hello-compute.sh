#!/usr/bin/env bash
# Build the rung-2 "hello compute" FEX harness (fexprobe.exe) — a native arm64
# Windows PE linking the prebuilt vendored FEXCore archives. Requires the FEX
# vendor build to exist first (tools/fex/build-fex.sh).
#
# Usage: tools/fex/build-hello-compute.sh [fex-builddir] [harness-builddir]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FEX_SRC="$REPO_ROOT/external/fex"
FEX_BUILD="${1:-$REPO_ROOT/build-fex-arm64}"
HARNESS_SRC="$REPO_ROOT/tools/fex/hello-compute"
HARNESS_BUILD="${2:-$REPO_ROOT/build-fex-hello}"
LLVM_MINGW="$REPO_ROOT/.build-tools/llvm-mingw/bin"

[ -f "$FEX_BUILD/FEXCore/Source/libFEXCore.a" ] || {
    echo "FATAL: FEX not built at $FEX_BUILD — run tools/fex/build-fex.sh first" >&2
    exit 1; }

export PATH="$LLVM_MINGW:$PATH"

mkdir -p "$HARNESS_BUILD"
cmake -S "$HARNESS_SRC" -B "$HARNESS_BUILD" -G Ninja \
    -DCMAKE_TOOLCHAIN_FILE="$FEX_SRC/Data/CMake/toolchain_mingw.cmake" \
    -DMINGW_TRIPLE=aarch64-w64-mingw32 \
    -DCMAKE_BUILD_TYPE=Release \
    -DFEX_SRC="$FEX_SRC" \
    -DFEX_BUILD="$FEX_BUILD"

ninja -C "$HARNESS_BUILD" fexprobe wow64probe

# hello32.exe — the i386 WoW64 Phase C guest (no CRT, no imports). Built with
# a direct clang call because the harness CMake is aarch64-target; this is the
# only i386 artifact here.
# GUI subsystem, NOT console: a CUI process makes kernel32's ConDllInitialize
# allocate a console via csrss, which hangs for a winlogon-spawned WoW64 process
# (no console to inherit). The GUI probes (fexprobe/wow64probe) work precisely
# because they skip console init. See wow64-plan.md Phase C M2.
"$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
    -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
    -o "$HARNESS_BUILD/hello32.exe" "$HARNESS_SRC/Hello32.c"

# gui32.exe — the wow64win guest (M4): no CRT, but REAL kernel32+gdi32 imports
# resolved from the build-i386 spec2def importlibs (the guest loader pulls the
# actual i386 DLLs from syswow64 at run time). Exercises the win32k thunk
# table via a direct display DC — no user-mode callbacks.
I386_BUILD="${VIBEOS_I386_BUILD:-$REPO_ROOT/build-i386}"
if [ -f "$I386_BUILD/win32ss/gdi/gdi32/CMakeFiles/libgdi32.dir/libgdi32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -o "$HARNESS_BUILD/gui32.exe" "$HARNESS_SRC/Gui32.c" \
        "$I386_BUILD/win32ss/gdi/gdi32/CMakeFiles/libgdi32.dir/libgdi32.a" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] gui32.exe skipped: build-i386 importlibs missing ($I386_BUILD)"
fi

# win32.exe — the M8 window-creation guest: user32 RegisterClass/CreateWindow +
# a real message pump (forces user32 client connect + kernel→user callbacks).
if [ -f "$I386_BUILD/win32ss/user/user32/CMakeFiles/libuser32.dir/libuser32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -o "$HARNESS_BUILD/win32.exe" "$HARNESS_SRC/Win32App.c" \
        "$I386_BUILD/win32ss/user/user32/CMakeFiles/libuser32.dir/libuser32.a" \
        "$I386_BUILD/win32ss/gdi/gdi32/CMakeFiles/libgdi32.dir/libgdi32.a" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] win32.exe skipped: build-i386 user32 importlib missing ($I386_BUILD)"
fi

# file32.exe — the M10 file-I/O guest: ReadFile/WriteFile/GetFileSize/FindFirstFile.
# The oracle for the IO_STATUS_BLOCK 32<->64 marshalling (Information is 4 bytes
# in the guest, 8 on the host — ReadFile's byte count IS that field).
if [ -f "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -o "$HARNESS_BUILD/file32.exe" "$HARNESS_SRC/File32.c" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] file32.exe skipped: build-i386 kernel32 importlib missing ($I386_BUILD)"
fi

# async32.exe — the M15b overlapped-I/O guest: Event-based + polled ReadFile on a
# FILE_FLAG_OVERLAPPED handle. The oracle for kernel-side IO_STATUS_BLOCK32
# completion (OVERLAPPED.Internal/InternalHigh ARE the guest IOSB32; .Offset is
# the 16-byte-clobber detector).
if [ -f "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -o "$HARNESS_BUILD/async32.exe" "$HARNESS_SRC/Async32.c" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] async32.exe skipped: build-i386 kernel32 importlib missing ($I386_BUILD)"
fi

# fault32.exe — the E5 guest-SEH verification guest: hardware AV caught by a raw
# fs:[0] handler with the LIVE faulting context (the E5 flush oracle), software
# RaiseException round-trip (the NtRaiseException marshaller), and
# OutputDebugStringA consumption (DBG_PRINTEXCEPTION must not reach guest SEH).
if [ -f "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -o "$HARNESS_BUILD/fault32.exe" "$HARNESS_SRC/Fault32.c" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] fault32.exe skipped: build-i386 kernel32 importlib missing ($I386_BUILD)"
fi

# marshal32.exe — the M18 marshaller-verification guest: NtQueryInformationThread
# TBI (verified against the guest's OWN TEB32), IOCP post/get sentinel round-trip
# (NtRemoveIoCompletion out-slots + IOSB), EnumDisplayMonitors/GetMonitorInfo/
# MonitorFromPoint (count-aware HMONITOR array + the new flat thunks).
if [ -f "$I386_BUILD/win32ss/user/user32/CMakeFiles/libuser32.dir/libuser32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -o "$HARNESS_BUILD/marshal32.exe" "$HARNESS_SRC/Marshal32.c" \
        "$I386_BUILD/win32ss/user/user32/CMakeFiles/libuser32.dir/libuser32.a" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" \
        "$I386_BUILD/dll/ntdll/CMakeFiles/libntdll.dir/libntdll.a"
else
    echo "[!] marshal32.exe skipped: build-i386 importlibs missing ($I386_BUILD)"
fi

echo
echo "Built:"
ls -la "$HARNESS_BUILD"/fexprobe.exe "$HARNESS_BUILD"/wow64probe.exe "$HARNESS_BUILD"/hello32.exe "$HARNESS_BUILD"/gui32.exe "$HARNESS_BUILD"/win32.exe "$HARNESS_BUILD"/file32.exe "$HARNESS_BUILD"/marshal32.exe 2>/dev/null || true
