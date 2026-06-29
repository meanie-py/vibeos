#!/usr/bin/env python3
"""Scan ntoskrnl.exe for illegal/degenerate codegen in named functions.

x86 (i386/amd64): ud2 (0x0f 0x0b) in a named function means LLVM proved the
function body contains undefined behavior and replaced it with an illegal
instruction. Almost always a miscompilation from captures(none) or an i386 ABI
mismatch.

arm64: there is NO x86-style trap for UB. The aarch64-pc-windows-gnullvm target
builds with trap-unreachable OFF — a UB/`unreachable` body emits no instruction
(it falls through silently) and a Rust panic lowers to `rust_begin_unwind`, which
is a branch-to-self `b .` spin (the "fake arm64 idle hang"). So:
  * The captures(none)/UB class is SILENT on arm64 (no binary signal) — it is
    caught at SOURCE by tools/lint_kernel.py, which is the primary arm64 defense.
  * The one detectable binary signal is an *unexpected branch-to-self spin* in a
    named function (a body that is just `b .`). The known-good spins are panic /
    bugcheck / halt primitives; a NEW one means a function regressed into an
    infinite loop (a pre-boot catch for an otherwise-mysterious arm64 hang).
This tool scans ud2 on x86 and branch-to-self on arm64, so the binary gate is
honest on all three arches instead of silently "passing" on arm64.

Expected x86 ud2 sites (not bugs):
- core::option::Option::unwrap_unchecked — unreachable panic stubs
- core::panicking::* — panic infrastructure

Usage:
    python3 tools/scan_ud2.py [build-dir]
    python3 tools/scan_ud2.py build-i386
    python3 tools/scan_ud2.py --check build-i386  # exit 1 if unexpected found
    python3 tools/scan_ud2.py --check build-arm64 # arm64 branch-to-self scan
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

# arm64: named functions whose body is legitimately a branch-to-self `b .` spin
# (the arm64 analogue of x86's EXPECTED_UD2). These are intentional terminal
# loops — panic handler, bugcheck, and halt primitives — not miscompiles. A spin
# in a function NOT matching one of these is flagged: a body that regressed into
# an infinite loop, which boots as a hard-to-diagnose "idle hang". Substring
# match (mangled Rust names embed the symbol), same as EXPECTED_UD2.
EXPECTED_SPIN = {
    "rust_begin_unwind",  # Rust panic handler (panic=abort → loop {})
    "KeBugCheckWithTf",  # bugcheck terminal halt
    "KeBugCheck",  # bugcheck family
    "xHalHaltSystem",  # HAL halt
    "HvValidateHive",  # fatal hive-corruption terminal path
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


def scan_ud2(ntoskrnl_path: Path, objdump: str, arch: str = "i386") -> list[dict]:
    """Scan binary for illegal/degenerate codegen in named functions.

    x86: ud2 (0f 0b). arm64: branch-to-self `b .` spins. Returns a list of
    {address, function, expected, known_cgu}. The arm64 path classifies against
    EXPECTED_SPIN (and never sets known_cgu — that bug is x86-CGU specific)."""
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

        if not current_function:
            continue

        if arch == "arm64":
            # Branch-to-self: `<addr>: <word>  b 0x<addr>` where the branch
            # target equals the instruction's own address. That is `b .` — an
            # infinite loop, the arm64 panic/halt idiom (and the regression
            # signature for a function that became an unintended spin).
            m = re.search(r"^\s+([0-9a-f]+):\s+[0-9a-f]{8}\s+b\s+0x([0-9a-f]+)\b", line)
            if m and m.group(1) == m.group(2):
                addr = m.group(1)
                expected = any(e in current_function for e in EXPECTED_SPIN)
                findings.append(
                    {
                        "address": addr,
                        "function": current_function,
                        "expected": expected,
                        "known_cgu": False,
                    }
                )
        else:
            # x86: ud2 instruction (0f 0b)
            ud2_match = re.search(r"^\s+([0-9a-f]+):\s+0f 0b\s+ud2", line)
            if ud2_match:
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

    # Per-arch vocabulary: x86 looks for ud2, arm64 for branch-to-self spins.
    kind = "branch-to-self spins" if arch == "arm64" else "ud2 instructions"
    bug_hint = (
        "unintended infinite loop (arm64 'idle hang' signature)"
        if arch == "arm64"
        else "likely LLVM miscompilation"
    )
    print(f"Scanning {ntoskrnl} ({arch}) for {kind}...")
    findings = scan_ud2(ntoskrnl, objdump, arch)

    expected = [f for f in findings if f["expected"]]
    known_cgu = [f for f in findings if not f["expected"] and f["known_cgu"]]
    unexpected = [f for f in findings if not f["expected"] and not f["known_cgu"]]

    if expected:
        label = "panic/halt spins" if arch == "arm64" else "unreachable panic paths"
        print(f"\n  Expected ({len(expected)} — {label}):")
        for f in expected:
            print(f"    0x{f['address']}: {f['function']}")

    if known_cgu:
        print(f"\n  Known CGU ud2 ({len(known_cgu)} — rustc CGU partitioning bug, non-blocking):")
        for f in known_cgu:
            print(f"    0x{f['address']}: {f['function']}  (known)")

    if unexpected:
        print(f"\n  UNEXPECTED ({len(unexpected)} — {bug_hint}):")
        for f in unexpected:
            print(f"    0x{f['address']}: {f['function']}  *** BUG ***")

    if not findings:
        print(f"  No {kind} found.")

    if arch == "arm64":
        # Be explicit that a clean spin-scan does NOT mean arm64 UB is clean —
        # the captures(none)/UB class is invisible in arm64 binaries (no trap).
        print(
            "\n  NOTE: arm64 has no instruction-level UB trap (trap-unreachable off);"
            "\n  the captures(none)/UB class is SILENT here and is gated at source by"
            "\n  tools/lint_kernel.py — this scan only covers branch-to-self spins."
        )

    print(
        f"\nTotal: {len(findings)} {kind.split()[0]} ({len(expected)} expected, "
        f"{len(known_cgu)} known CGU, {len(unexpected)} unexpected)"
    )

    if check_mode and unexpected:
        noun = "branch-to-self spin" if arch == "arm64" else "ud2"
        print(f"\nFAILED: Unexpected {noun} found — {bug_hint}!")
        sys.exit(1)
    elif not unexpected:
        print("\nPASSED: All findings are in expected or known-CGU locations.")


if __name__ == "__main__":
    main()
