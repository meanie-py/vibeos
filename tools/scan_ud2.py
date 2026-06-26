#!/usr/bin/env python3
"""Scan ntoskrnl.exe for ud2 instructions in named functions.

ud2 (0x0f 0x0b) in a named function means LLVM proved the function body
contains undefined behavior and replaced it with an illegal instruction.
This is almost always a miscompilation caused by captures(none) or ABI
mismatch on i386.

Expected ud2 sites (not bugs):
- core::option::Option::unwrap_unchecked — unreachable panic stubs
- core::panicking::* — panic infrastructure

Usage:
    python3 tools/scan_ud2.py [build-dir]
    python3 tools/scan_ud2.py build-i386
    python3 tools/scan_ud2.py --check build-i386  # exit 1 if unexpected ud2 found
"""

import re
import subprocess
import sys
from pathlib import Path

# Functions where ud2 is expected (unreachable panic/abort paths)
EXPECTED_UD2 = {
    "unwrap_unchecked",
    # `core::hint::unreachable_unchecked` — the intrinsic whose entire body IS
    # the `ud2` (reaching it is UB by contract). It is the callee of the
    # already-trusted `unwrap_unchecked`: `Option::unwrap_unchecked` tail-calls
    # `unreachable_unchecked` on the `None` arm. Normally it inlines to nothing,
    # but with `-C debug-assertions` it gains a `check_language_ub` +
    # `precondition_check` wrapper, so LLVM keeps an out-of-line copy (plus a
    # `.llvm.<hash>` CGU-local clone). A cnt-rust CGU partition shift surfaced
    # those copies into the linked image as standalone symbols — same mechanism
    # as `NeverShortCircuit`/`copy_from_slice_impl` below. Verified i386: all
    # callers are `Option::unwrap_unchecked` monomorphizations (avltable
    # RtlAvlTable types); no kernel function tail-calls it. Unreachable-by-
    # construction, NOT a captures(none) miscompile.
    "unreachable_unchecked",
    "panic_cold",
    "panic_cold_display",
    "panic_nounwind_cold",
    "panic_nounwind",
    "abort",
    "rust_begin_unwind",
    # `core::slice::copy_from_slice_impl` is a length-checked memcpy; the ud2
    # is the unreachable "lengths don't match" assertion at the function tail.
    # Only appears in `-Z build-std` cargo binaries because rustup's pre-built
    # compiler_builtins inlines this monomorphization away.
    "copy_from_slice_impl",
    # `<NeverShortCircuit<T> as FromResidual<NeverShortCircuitResidual>>::
    # from_residual` — its input type `NeverShortCircuitResidual` is an EMPTY
    # enum (uninhabited), so the body is `match r {}` and ud2 is the ONLY
    # correct codegen: the function is uncallable (no value of the input type
    # can exist). It is the internal-iteration (`try_fold`) machinery behind
    # iterator adapters in cnt-rust's `core_rs`. Unreachable-by-construction,
    # NOT a miscompile — same class as `unwrap_unchecked`. Surfaced into the
    # linked image when cnt-rust's CGU partition shifts (a new crate module
    # relocates which CGU carries it); see feedback_workspace_member_cgu_shift.
    "NeverShortCircuit",
}

# Functions with known ud2 from rustc CGU partitioning bug (codegen-units=16).
# rustc places function definitions in one CGU but replaces their body with
# `unreachable` -> ud2. With CGU=16 only a few functions are affected.
# Most are in dead unnamed functions placed after the labeled function's ret;
# PopSetSystemPowerState has its actual body replaced but is only called for
# power state transitions (not during boot). Tracked as a known issue.
# Fix requires coordinated CGU=1 migration (see feedback_cgu1.md).
KNOWN_CGU_UD2 = {
    "IoInitializeCrashDump",
    "NtQuerySemaphore",
    "NtSetThreadExecutionState",
    "NtStopProfile",
    "PopSetSystemPowerState",
    "MmPageOutPhysicalAddress",
    "IopAcquireFileObjectLock",
    "are_any_accesses_granted",
    "KiSystemStartup",
    "KeAcquireInStackQueuedSpinLockForDpc",
}


def find_objdump(arch: str) -> str | None:
    """Find a working objdump for the given architecture.

    Tries the per-arch mingw objdump and llvm-objdump on PATH first, then
    falls back to the repo's bundled llvm-mingw toolchain
    (.build-tools/llvm-mingw/bin). The fallback matters for arm64: a stock
    Linux box has the x86_64/i686 mingw objdumps in /usr/bin (so i386/amd64
    lint fine) but neither aarch64-w64-mingw32-objdump nor llvm-objdump, and
    the bundled toolchain isn't necessarily on PATH — without this the arm64
    ud2 scan silently no-ops with "No objdump found".
    """
    names = {
        "arm64": ["aarch64-w64-mingw32-objdump", "llvm-objdump"],
        "amd64": ["x86_64-w64-mingw32-objdump", "llvm-objdump"],
        "i386": ["i686-w64-mingw32-objdump", "llvm-objdump"],
    }.get(arch, ["llvm-objdump"])

    # PATH candidates, then the same names under the bundled toolchain.
    bundled = Path(__file__).resolve().parent.parent / ".build-tools" / "llvm-mingw" / "bin"
    candidates = list(names) + [str(bundled / n) for n in names]

    for c in candidates:
        try:
            subprocess.run([c, "--version"], capture_output=True, check=True)
            return c
        except (FileNotFoundError, subprocess.CalledProcessError, PermissionError):
            continue
    return None


def scan_ud2(ntoskrnl_path: Path, objdump: str) -> list[dict]:
    """Scan binary for ud2 instructions, return list of {address, function, expected}."""
    result = subprocess.run(
        [objdump, "-d", str(ntoskrnl_path)],
        capture_output=True,
        text=True,
        timeout=120,
    )

    findings = []
    current_function = None

    for line in result.stdout.splitlines():
        # Match function labels like "0046b350 <_IopAcquireFileObjectLock@16>:"
        func_match = re.match(r"^([0-9a-f]+)\s+<(.+)>:", line)
        if func_match:
            current_function = func_match.group(2)
            continue

        # Match ud2 instruction (0f 0b)
        ud2_match = re.search(r"^\s+([0-9a-f]+):\s+0f 0b\s+ud2", line)
        if ud2_match and current_function:
            addr = ud2_match.group(1)
            expected = any(e in current_function for e in EXPECTED_UD2)
            known_cgu = any(e in current_function for e in KNOWN_CGU_UD2)
            findings.append(
                {
                    "address": addr,
                    "function": current_function,
                    "expected": expected,
                    "known_cgu": known_cgu,
                }
            )

    return findings


def main():
    check_mode = "--check" in sys.argv
    args = [a for a in sys.argv[1:] if a != "--check"]

    build_dir = Path(args[0]) if args else Path("build-i386")
    ntoskrnl = build_dir / "ntoskrnl" / "ntoskrnl.exe"

    if not ntoskrnl.exists():
        print(f"ERROR: {ntoskrnl} not found")
        sys.exit(1)

    # Detect arch from build dir name
    arch = "i386"
    if "arm64" in str(build_dir):
        arch = "arm64"
    elif "amd64" in str(build_dir):
        arch = "amd64"

    objdump = find_objdump(arch)
    if not objdump:
        print("ERROR: No objdump found")
        sys.exit(1)

    print(f"Scanning {ntoskrnl} for ud2 instructions...")
    findings = scan_ud2(ntoskrnl, objdump)

    expected = [f for f in findings if f["expected"]]
    known_cgu = [f for f in findings if not f["expected"] and f["known_cgu"]]
    unexpected = [f for f in findings if not f["expected"] and not f["known_cgu"]]

    if expected:
        print(f"\n  Expected ud2 ({len(expected)} — unreachable panic paths):")
        for f in expected:
            print(f"    0x{f['address']}: {f['function']}")

    if known_cgu:
        print(f"\n  Known CGU ud2 ({len(known_cgu)} — rustc CGU partitioning bug, non-blocking):")
        for f in known_cgu:
            print(f"    0x{f['address']}: {f['function']}  (known)")

    if unexpected:
        print(f"\n  UNEXPECTED ud2 ({len(unexpected)} — likely LLVM miscompilation):")
        for f in unexpected:
            print(f"    0x{f['address']}: {f['function']}  *** BUG ***")

    if not findings:
        print("  No ud2 instructions found.")

    print(
        f"\nTotal: {len(findings)} ud2 ({len(expected)} expected, "
        f"{len(known_cgu)} known CGU, {len(unexpected)} unexpected)"
    )

    if check_mode and unexpected:
        print("\nFAILED: Unexpected ud2 found — LLVM miscompilation detected!")
        sys.exit(1)
    elif not unexpected:
        print("\nPASSED: All ud2 are in expected or known-CGU locations.")


if __name__ == "__main__":
    main()
