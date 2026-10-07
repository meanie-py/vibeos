#!/usr/bin/env bash
# Build the rung-2 "hello compute" FEX harness (fexprobe.exe) — a native arm64
# Windows PE linking the prebuilt vendored FEXCore archives. Requires the FEX
# vendor build to exist first (tools/fex/build-fex.sh).
#
# Usage: tools/fex/build-hello-compute.sh [fex-builddir] [harness-builddir]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FEX_SRC="$REPO_ROOT/vendor/fex"
FEX_BUILD="${1:-$REPO_ROOT/build-fex-arm64}"
HARNESS_SRC="$REPO_ROOT/tools/fex/hello-compute"
HARNESS_BUILD="${2:-$REPO_ROOT/build-fex-hello}"
LLVM_MINGW="$REPO_ROOT/.build-tools/llvm-mingw/bin"

export PATH="$LLVM_MINGW:$PATH"

mkdir -p "$HARNESS_BUILD"

# fexprobe/wow64probe are the ARM64 half: native arm64 PEs linking the vendored
# FEXCore archives, so they genuinely need the FEX vendor build. The nine i386
# guests below do NOT -- they are plain i386 PEs from llvm-mingw, and on amd64 the
# CPU layer is our own wow64cpu.dll (heaven's gate, no emulation), so FEX is not
# in that picture at all. This used to be one `exit 1` at the top of the file,
# which meant an amd64-only workflow could not produce a single ladder guest
# without first building an arm64 emulator it never loads. Skipping loudly, in
# the same shape as the per-guest skips below, is what lets `smoke --ladder` and
# the gate's ladder stage work on an amd64-only tree.
if [ -f "$FEX_BUILD/FEXCore/Source/libFEXCore.a" ]; then
    cmake -S "$HARNESS_SRC" -B "$HARNESS_BUILD" -G Ninja \
        -DCMAKE_TOOLCHAIN_FILE="$FEX_SRC/Data/CMake/toolchain_mingw.cmake" \
        -DMINGW_TRIPLE=aarch64-w64-mingw32 \
        -DCMAKE_BUILD_TYPE=Release \
        -DFEX_SRC="$FEX_SRC" \
        -DFEX_BUILD="$FEX_BUILD"

    ninja -C "$HARNESS_BUILD" fexprobe wow64probe

    # libwowx64fex.dll — the BTCpu64 CPU layer for x86-64 guests (Fork B of
    # docs/engineering/wow64-x64-guest-design.md). A shipping DLL rather than a
    # probe, but it links the same prebuilt FEX archives, so it builds where
    # they are. Its own project: it is a DLL with FEX's private CRT, not an exe.
    cmake -S "$REPO_ROOT/tools/fex/wowx64fex" -B "$HARNESS_BUILD/wowx64fex" -G Ninja \
        -DCMAKE_TOOLCHAIN_FILE="$FEX_SRC/Data/CMake/toolchain_mingw.cmake" \
        -DMINGW_TRIPLE=aarch64-w64-mingw32 \
        -DCMAKE_BUILD_TYPE=Release \
        -DFEX_SRC="$FEX_SRC" \
        -DFEX_BUILD="$FEX_BUILD" \
        -DVIBEOS_SRC="$REPO_ROOT"
    ninja -C "$HARNESS_BUILD/wowx64fex"
else
    echo "[!] fexprobe/wow64probe skipped: FEX not built at $FEX_BUILD"
    echo "    (run tools/fex/build-fex.sh if you need the arm64 probes)"
    echo "    The nine i386 WoW64 guests do not need FEX and are built below."
fi

# hello32.exe — the i386 WoW64 Phase C guest (no CRT, no imports). Built with
# a direct clang call because the harness CMake is aarch64-target; this is the
# only i386 artifact here.
# GUI subsystem, NOT console: a CUI process makes kernel32's ConDllInitialize
# allocate a console via csrss, which hangs for a winlogon-spawned WoW64 process
# (no console to inherit). The GUI probes (fexprobe/wow64probe) work precisely
# because they skip console init. See wow64-plan.md Phase C M2.
"$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
    -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -Wl,--no-insert-timestamp \
    -o "$HARNESS_BUILD/hello32.exe" "$HARNESS_SRC/Hello32.c"

# gui32.exe — the wow64win guest (M4): no CRT, but REAL kernel32+gdi32 imports
# resolved from the build-i386 spec2def importlibs (the guest loader pulls the
# actual i386 DLLs from syswow64 at run time). Exercises the win32k thunk
# table via a direct display DC — no user-mode callbacks.
I386_BUILD="${VIBEOS_I386_BUILD:-$REPO_ROOT/build-i386}"
if [ -f "$I386_BUILD/win32ss/gdi/gdi32/CMakeFiles/libgdi32.dir/libgdi32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -Wl,--no-insert-timestamp \
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
        -Wl,--no-insert-timestamp \
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
        -Wl,--no-insert-timestamp \
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
        -Wl,--no-insert-timestamp \
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
        -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/fault32.exe" "$HARNESS_SRC/Fault32.c" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] fault32.exe skipped: build-i386 kernel32 importlib missing ($I386_BUILD)"
fi

# thread32.exe — the #64 threading guest: CreateThread/join/exit-code, the
# thread parameter (CONTEXT32.Ebx), TLS on the new thread (proves the guest
# loader's per-thread init ran), guest SEH on the new thread (proves TEB32's
# NtTib stack bounds were set), and 4 concurrent threads with distinct stacks
# (proves the CPU area is per-thread, not the old process-wide static).
if [ -f "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/thread32.exe" "$HARNESS_SRC/Thread32.c" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] thread32.exe skipped: build-i386 kernel32 importlib missing ($I386_BUILD)"
fi

# selfmod32.exe — the E17 runtime-code-generation guest: execute VirtualAlloc'd
# code, rewrite it + FlushInstructionCache, RW->RX publication, free+realloc at
# the same base, ReadFile straight into live code, and WriteProcessMemory
# hot-patching (which doubles as the 4-vs-8 *ReturnSize marshalling oracle).
# The stale-translation rungs fail SILENTLY WRONG, which is the point.
if [ -f "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/selfmod32.exe" "$HARNESS_SRC/Selfmod32.c" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] selfmod32.exe skipped: build-i386 kernel32 importlib missing ($I386_BUILD)"
fi

# reg32.exe — the registry-view guest: Wow6432Node redirection, isolation, the
# no-double-splice exemption, the 64-bit fallback, and the two-view delete. It
# is the only ladder guest that links advapi32, and the only one whose failures
# are SILENT -- a redirection bug returns SUCCESS and the wrong key.
if [ -f "$I386_BUILD/dll/win32/advapi32/CMakeFiles/libadvapi32.dir/libadvapi32.a" ] && \
   [ -f "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/reg32.exe" "$HARNESS_SRC/Reg32.c" \
        "$I386_BUILD/dll/win32/advapi32/CMakeFiles/libadvapi32.dir/libadvapi32.a" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] reg32.exe skipped: build-i386 advapi32/kernel32 importlib missing ($I386_BUILD)"
fi

# spawn32.exe — the process-creation guest: can a 32-bit guest start another
# 32-bit process at all. Links kernel32 only.
#
# Built UNCONDITIONALLY since 2026-09-19, when it first reported 0x600D and was
# promoted into ladder.LADDER. It was opt-in twice over and both reasons are
# spent: first it BUGCHECKed the machine (0x1E, a dropped _SEH2_EXCEPT at four
# probe sites in ARM3/virtual.rs), then it was merely red-by-design as a
# DIAGNOSTIC measuring an open gap. The gap is closed, so it is a parity guest
# now and has to be present on every tree or it guards nothing -- and
# _wow64_ladder is all-or-nothing, so an absent spawn32 costs the WHOLE ladder
# its staging rather than just itself.
if [ -f "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/spawn32.exe" "$HARNESS_SRC/Spawn32.c" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] spawn32.exe skipped: build-i386 kernel32 importlib missing ($I386_BUILD) -- the ladder will NOT stage"
fi

# apc32.exe — user APCs into 32-bit code (QueueUserAPC, ReadFileEx, timers) and
# per-thread cleanup: Apc64.c, the x86-64 ladder's apc64, built at 32 bits.
if [ -f "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/apc32.exe" "$HARNESS_SRC/Apc64.c" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
else
    echo "[!] apc32.exe skipped: build-i386 kernel32 importlib missing ($I386_BUILD) -- the ladder will NOT stage"
fi

# marshal32.exe — the M18 marshaller-verification guest: NtQueryInformationThread
# TBI (verified against the guest's OWN TEB32), IOCP post/get sentinel round-trip
# (NtRemoveIoCompletion out-slots + IOSB), EnumDisplayMonitors/GetMonitorInfo/
# MonitorFromPoint (count-aware HMONITOR array + the new flat thunks), and
# select() on a fresh UDP socket — the first rung to reach an AFD DRIVER payload
# (AFD_POLL_INFO), which is why it links ws2_32. That link is also what pulls
# ws2_32 into the staged import closure, and with it msafd via the Winsock
# catalog, so the AFD marshallers become reachable by a guest at all.
if [ -f "$I386_BUILD/win32ss/user/user32/CMakeFiles/libuser32.dir/libuser32.a" ] && \
   [ -f "$I386_BUILD/dll/win32/ws2_32/CMakeFiles/libws2_32.dir/libws2_32.a" ]; then
    "$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
        -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
        -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/marshal32.exe" "$HARNESS_SRC/Marshal32.c" \
        "$I386_BUILD/win32ss/user/user32/CMakeFiles/libuser32.dir/libuser32.a" \
        "$I386_BUILD/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" \
        "$I386_BUILD/dll/ntdll/CMakeFiles/libntdll.dir/libntdll.a" \
        "$I386_BUILD/dll/win32/ws2_32/CMakeFiles/libws2_32.dir/libws2_32.a"
else
    echo "[!] marshal32.exe skipped: build-i386 importlibs missing ($I386_BUILD)"
fi

# hello64.exe — the first x86-64 guest (Fork B milestone M1): no CRT, no
# imports, raw syscalls. Their numbers are read out of ntoskrnl/sysfuncs.lst
# here — the list is arch-independent, so the guest's number IS the arm64
# kernel's, which is what lets wowx64.dll forward a syscall by register
# shuffle alone. Same binary runs natively on amd64 as the oracle.
SYSFUNCS="$REPO_ROOT/ntoskrnl/sysfuncs.lst"
svc_index() {
    local n
    n="$(awk -v f="$1" '$1==f {print NR-1; exit}' "$SYSFUNCS")"
    [ -n "$n" ] || { echo "FATAL: $1 not in $SYSFUNCS" >&2; exit 1; }
    echo "$n"
}
"$LLVM_MINGW/x86_64-w64-mingw32-clang" -O1 -nostdlib -ffreestanding -fno-stack-protector \
    -Wl,-e,start -Wl,--subsystem,windows:5.2 -Wl,--no-insert-timestamp \
    -DSVC_NTCLOSE="$(svc_index NtClose)" \
    -DSVC_NTTERMINATEPROCESS="$(svc_index NtTerminateProcess)" \
    -o "$HARNESS_BUILD/hello64.exe" "$HARNESS_SRC/Hello64.c"

# The x86-64 ladder (BTCpu64 M2): the 32-bit ladder's own sources, rebuilt for
# x86-64 against build-amd64's import libraries. One source per rung on purpose
# -- the same checks at both widths is what makes the two ladders comparable;
# the few width-specific bits (guest SEH has no fs:[0] chain on x64) are
# #ifdef _WIN64 inside the source. They run under FEX on arm64 and natively on
# amd64, the native run being the oracle that the guest itself is right.
AMD64_BUILD="${VIBEOS_AMD64_BUILD:-$REPO_ROOT/build-amd64}"
x64_lib() { # x64_lib <dir under build-amd64> <name>
    echo "$AMD64_BUILD/$1/CMakeFiles/lib$2.dir/lib$2.a"
}
x64_guest() { # x64_guest <out.exe> <source> <importlib...>
    local out="$1" src="$2" lib
    shift 2
    for lib in "$@"; do
        if [ ! -f "$lib" ]; then
            echo "[!] $out skipped: build-amd64 importlib missing ($lib)"
            return 0
        fi
    done
    "$LLVM_MINGW/x86_64-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,start \
        -Wl,--subsystem,windows:5.2 -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/$out" "$HARNESS_SRC/$src" "$@"
}
K64="$(x64_lib dll/win32/kernel32 kernel32)"
G64="$(x64_lib win32ss/gdi/gdi32 gdi32)"
U64="$(x64_lib win32ss/user/user32 user32)"
x64_guest gui64.exe Gui32.c "$G64" "$K64"
x64_guest win64.exe Win32App.c "$U64" "$G64" "$K64"
x64_guest file64.exe File32.c "$K64"
x64_guest async64.exe Async32.c "$K64"
x64_guest fault64.exe Fault32.c "$K64"
x64_guest thread64.exe Thread32.c "$K64"
x64_guest selfmod64.exe Selfmod32.c "$K64"
x64_guest reg64.exe Reg32.c "$(x64_lib dll/win32/advapi32 advapi32)" "$K64"
x64_guest spawn64.exe Spawn32.c "$K64"
x64_guest apc64.exe Apc64.c "$K64"
x64_guest marshal64.exe Marshal32.c "$U64" "$K64" "$(x64_lib dll/ntdll ntdll)" \
    "$(x64_lib dll/win32/ws2_32 ws2_32)"

# xinject.exe -- spawn64's NATIVE helper (rung 13, XInject.c): built for each
# host machine and staged as system32\xinject.exe on that arch's media. It is
# no guest, so it links llvm-mingw's own kernel32/ntdll import libraries (by
# name, nothing build-specific) and needs no build-* tree.
for M in aarch64:arm64 x86_64:amd64; do
    "$LLVM_MINGW/${M%%:*}-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,start \
        -Wl,--subsystem,windows -Wl,--no-insert-timestamp \
        -o "$HARNESS_BUILD/xinject-${M##*:}.exe" "$HARNESS_SRC/XInject.c" -lkernel32 -lntdll
done

echo
echo "Built:"
ls -la "$HARNESS_BUILD"/fexprobe.exe "$HARNESS_BUILD"/wow64probe.exe "$HARNESS_BUILD"/hello32.exe "$HARNESS_BUILD"/gui32.exe "$HARNESS_BUILD"/win32.exe "$HARNESS_BUILD"/file32.exe "$HARNESS_BUILD"/marshal32.exe "$HARNESS_BUILD"/hello64.exe "$HARNESS_BUILD"/wowx64fex/libwowx64fex.dll 2>/dev/null || true
