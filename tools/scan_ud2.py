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

if __package__:
    from .binary_corpus import governed_binaries
    from .binary_facts import BinaryFacts
else:
    from binary_corpus import governed_binaries
    from binary_facts import BinaryFacts

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
    # compiler_builtins int/big (u256/i256 soft-float bigint machinery, pulled
    # in by the `+soft-float` i686 kernel targets' f128 builtins via build-std).
    # Two ud2 shapes, both unreachable-by-construction (verified i386
    # 2026-07-06 by disassembly):
    #   * `u256::shr` / `u256::shl` / `i256::shl` — ud2 is the guarded
    #     `shift >= 256` panic tail (cmp $0x100 / jae → abort); the main body
    #     is real code.
    #   * `HInt::widen_hi` / `widen_mul` for the WIDEST int (i256/u256) — the
    #     body IS ud2: there is no wider type, the impl is `unimplemented!()`
    #     and nothing can call it (kept because build-std retains the impl).
    # NOT a captures(none)/ABI miscompile — panic/unimplemented codegen.
    "17compiler_builtins3int3big",
}

# RETIRED 2026-07-18: the "rustc CGU partitioning bug" allow-set is dead lore.
# The one entry still firing (MmPageOutPhysicalAddress, i386 + ntkrnlmp) was a
# §9.99 same-crate CC fold: mm/rmap.rs (where MmPageOutPhysicalAddress lives)
# IMPORTED MmDeleteVirtualMapping from the bindgen ddk module (extern "C" = cdecl
# on i386) while the def + every hot caller are extern "system" (stdcall @16 —
# the REAL ABI; the no-NTAPI decl in internal/mm.h is misleading, §9.107). That
# intra-crate cdecl-vs-stdcall disagreement folds MmPageOutPhysicalAddress to a
# ud2 (cold balance-manager path → tolerated latent). Fixed by giving rmap.rs a
# LOCAL extern "system" decl (matching marea.rs/section.rs), NOT by flipping the
# ABI to cdecl — cdecl desyncs ESP at the marea.rs VirtFn stdcall call and
# exec-at-0s during i386 driver init (§9.107). The other nine were already 0-hit
# on every kernel (i386/amd64/arm64 UP + amd64 ntkrnlmp SMP, verified by direct
# sweep). There was never a CGU partitioning bug — these were fold-class
# landmines all along. Any future hit here must be root-caused with
# tools/scan_ir_mismatch.py (field manual §9.99/§9.102/§9.107), NOT re-baselined.
KNOWN_CGU_UD2: set[str] = set()

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
    # kernel32 process-termination tails: C's `for(;;);` after
    # NtTerminateProcess — the process is gone, the spin is unreachable
    # belt-and-suspenders and matches upstream. Deliberate, not a regression.
    "ExitProcess",
    "FatalAppExit",
    "FatalExit",
}
# Matched by EXACT name: a substring "exit" would excuse every function that has one.
# vendor/dosfstools/common.c's `exit()` (linked from vfatlib) is `NtTerminateProcess(); while(1);`
# for the native setup programs -- the same unreachable tail as ExitProcess above.
EXPECTED_SPIN_EXACT = {"exit"}


# ---------------------------------------------------------------------------
# Driver-fleet sweep (field manual §9.99). scan_ud2 was ntoskrnl-only for two
# years while an entire class of CC/signature-mismatch folds (`ud2` bombs that
# make the LINK pass precisely because the code is broken) landed silently in
# driver/DLL binaries. The sweep scans every Rust-containing .sys/.dll/.exe in
# the build dir (cheap llvm-nm `rust_begin_unwind` filter first, so ~1000
# binaries reduce to ~100).
#
# Scope note: only UNMANGLED named functions are judged. ud2 inside mangled
# Rust internals is dominated by legitimate panic/unreachable machinery; the
# same-module mismatches that poison internal fns are gated at source level by
# tools/scan_ir_mismatch.py instead.

# Substring-skip for symbols that legitimately contain ud2 in driver binaries:
# compiler-builtins soft-float/bigint intrinsics (guarded shift/unimplemented
# tails), libm pulls, C++ deleting dtors, LLVM-cloned locals.
DRIVER_INTRINSIC_SKIP = {
    "compiler_builtins",
    "__add",
    "__sub",
    "__mul",
    "__div",
    "__mod",
    "__udiv",
    "__umod",
    "__divmod",
    "__udivmod",
    "__fix",
    "__float",
    "__trunc",
    "__extend",
    "fmodf",
    "lgamma",
    "D0Ev",
    "D1Ev",
    "D2Ev",
    ".llvm.",
}

# ud2 immediately after a call to one of these is the unreachable-after-noreturn
# terminator — expected codegen, not a bomb.
NORETURN_CALLEES = {
    "panic",
    "KeBugCheck",
    "FatBugCheck",
    "RtlAssert",
    "abort",
    "_assert",
    "ExRaiseStatus",
    "RaiseStatus",
    "ExRaiseAccessViolation",
    "KiBugCheck",
    "HalHaltSystem",
    "rust_begin_unwind",
    "Terminate",
    "unreachable",
}

# Known, root-caused residual ud2 sites in driver binaries: basename ->
# fn-name substrings. Keep SHORT and only with a written justification; a NEW
# finding fails --check. (Populated 2026-07-18 after the §9.99 fix campaign.)
KNOWN_DRIVER_UD2: dict[str, set[str]] = {
    # LoadAndBootWindowsCommon tail-calls the kernel entry point through a fn
    # pointer that never returns (OS handoff); the post-call ud2 is the
    # unreachable-after-noreturn terminator, unnameable by the indirect-call
    # heuristic. Boots green daily.
    "rosload.exe": {"LoadAndBootWindowsCommon"},
    "freeldr.exe": {"LoadAndBootWindowsCommon"},
    # pcix-rust's arbiters are `UNIMPLEMENTED; loop {}` because they were translated
    # BEFORE upstream #9502 (023d8af5566, "Implement the PCIX Arbiters") gave the C
    # real bodies; the Rust never absorbed it. Seen on arm64 since 2026-10-03, when
    # its corpus became the livecd set (x86's sweep reads ud2, not `b .`). Unreached
    # there (arm64 binds pci.sys, not pcix.sys). OWED: port #9502 into pcix-rust
    # (roadmap, "pcix-rust predates upstream's arbiters"), then delete this entry.
    "pcix.sys": {
        "PciArbiterDestructor",
        "ario_ApplyBrokenVideoHack",
        "ario_Constructor",
        "armem_Constructor",
        "arbusno_Constructor",
    },
    # Do NOT add new entries without a written root-cause attempt; every
    # 2026-07-18 first-sweep latent (user32 SwitchWndProc_common §9.103
    # null-blind deref, kernel32 seqlock spins §9.106, win32k
    # EngInitializeSafeSemaphore wait-loop) was root-caused and FIXED the
    # same day — the baseline is for justified terminal shapes only.
    # (kernel32 time/locale spins RESOLVED 2026-07-18: they were non-volatile
    # KUSER_SHARED_DATA seqlock capture loops — LLVM hoisted the plain loads,
    # leaving an unconditional `b .` if entered mid-update. Fixed with
    # read_volatile capture across kernel32/vista/ke-inlines/ntdll/clock.rs,
    # plus write_volatile on the kernel writer side. Field manual §9.106.)
}
# the SMP kernel image shares ntoskrnl's known-CGU baselines


def _is_mangled(fn: str) -> bool:
    base = fn.lstrip("_@")
    return base.startswith("ZN") or (base.startswith("R") and not base.startswith("Rtl"))


def find_rust_binaries(build_dir: Path, nm: str, cache=None, repo: Path | None = None):
    """Every PE in the build dir's GOVERNED corpus that links Rust code.

    The corpus comes from tools/binary_corpus.py: the build tree minus staging
    copies (strip caches, the arm64 ramdisk's symlink tree), which every arch's
    `livecd` DAG re-rolls on each build. ntoskrnl.exe is dropped here because
    main() scans it separately against its own kernel baselines. The llvm-nm
    `rust_begin_unwind` probe keeps this fast (~6s over ~1000 files).

    Returns (paths, corpus) so the caller can report what it actually judged.
    """
    corpus = governed_binaries(build_dir, repo)
    out: list[Path] = []
    for p in corpus.paths:
        if p.name == "ntoskrnl.exe":
            continue

        def contains_rust(p=p):
            r = subprocess.run([nm, str(p)], capture_output=True, text=True, timeout=60, check=True)
            return "rust_begin_unwind" in r.stdout

        if cache.get(p, contains_rust) if cache else contains_rust():
            out.append(p)
    return sorted(out), corpus


def scan_driver_binary(path: Path, objdump: str, arch: str) -> list[dict]:
    """ud2 (x86) / branch-to-self (arm64) in named, unmangled functions of one
    driver/DLL binary, with the noreturn-tail heuristic applied."""
    result = subprocess.run(
        [objdump, "-d", str(path)], capture_output=True, text=True, timeout=300, check=True
    )
    findings = []
    current_function = None
    prev_call_target = None
    known = KNOWN_DRIVER_UD2.get(path.name, set())
    for line in result.stdout.splitlines():
        func_match = re.match(r"^([0-9a-f]+)\s+<(.+)>:", line)
        if func_match:
            current_function = func_match.group(2)
            prev_call_target = None
            continue
        if not current_function:
            continue
        if arch == "arm64":
            m = re.search(r"^\s*([0-9a-f]+):\s+[0-9a-f]{8}\s+b\s+0x([0-9a-f]+)\b", line)
            hit = bool(m and m.group(1) == m.group(2))
            addr = m.group(1) if m else ""
        else:
            m = re.search(r"^\s*([0-9a-f]+):\s+0f 0b\s+ud2", line)
            hit = bool(m)
            addr = m.group(1) if m else ""
        if not hit:
            cm = re.search(r"\b(?:call\w*|bl)\s+.*<(.+?)>", line)
            if cm:
                prev_call_target = cm.group(1)
            elif line.strip() and not re.search(r"\baddl?\s+\$0x[0-9a-f]+, %esp", line):
                # any other instruction breaks call-adjacency EXCEPT pure
                # stack-cleanup (addl $N, %esp) and nop padding
                prev_call_target = prev_call_target if "nop" in line else None
            continue
        fn = current_function
        expected = (
            any(e in fn for e in EXPECTED_UD2)
            or (
                arch == "arm64"
                and (any(e in fn for e in EXPECTED_SPIN) or fn in EXPECTED_SPIN_EXACT)
            )
            or any(e in fn for e in DRIVER_INTRINSIC_SKIP)
            or _is_mangled(fn)
            or (prev_call_target and any(n in prev_call_target for n in NORETURN_CALLEES))
        )
        known_hit = any(k in fn for k in known)
        findings.append(
            {
                "address": addr,
                "function": fn,
                "binary": path.name,
                "expected": expected,
                "known_cgu": known_hit,
            }
        )
    return findings


def scan_drivers(build_dir: Path, objdump: str, nm: str, arch: str):
    """Sweep the governed Rust-containing driver/DLL binaries; returns
    (binaries, corpus, unexpected+known findings) — expected ones are dropped
    here (volume)."""
    from concurrent.futures import ThreadPoolExecutor

    repo = build_dir.resolve().parent
    symbols = BinaryFacts(
        repo, namespace="rust-symbols", implementation=[Path(__file__)], tools=[nm]
    )
    facts = BinaryFacts(
        repo,
        namespace="driver-ud2",
        implementation=[Path(__file__)],
        tools=[objdump],
        options={"arch": arch},
    )
    binaries, corpus = find_rust_binaries(build_dir, nm, symbols, repo)
    keep: list[dict] = []
    with ThreadPoolExecutor(max_workers=8) as pool:
        for findings in pool.map(
            lambda p: facts.get(p, lambda: scan_driver_binary(p, objdump, arch)), binaries
        ):
            keep.extend(f for f in findings if not f["expected"])
    return binaries, corpus, keep


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


def find_nm(arch: str) -> str | None:
    """Find a working llvm-nm / mingw nm (same search order as find_objdump)."""
    names = {
        "arm64": ["aarch64-w64-mingw32-nm", "llvm-nm"],
        "amd64": ["x86_64-w64-mingw32-nm", "llvm-nm"],
        "i386": ["i686-w64-mingw32-nm", "llvm-nm"],
    }.get(arch, ["llvm-nm"])
    bundled = Path(__file__).resolve().parent.parent / ".build-tools" / "llvm-mingw" / "bin"
    for c in list(names) + [str(bundled / n) for n in names]:
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
        check=True,
    )

    findings = []
    current_function = None
    prev_insn = ""

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
                expected = (
                    any(e in current_function for e in EXPECTED_SPIN)
                    or current_function in EXPECTED_SPIN_EXACT
                )
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
                # `int $0x29` is __fastfail (RtlFailFast: a corrupt LIST_ENTRY, a failed
                # /GS check). It never returns, so the compiler ends the path with ud2 --
                # a deliberate stop, as in the C header inlines, not a signature fold.
                fastfail = re.search(r"\bint\s+\$0x29\b", prev_insn) is not None
                expected = fastfail or any(e in current_function for e in EXPECTED_UD2)
                known_cgu = any(e in current_function for e in KNOWN_CGU_UD2)
                findings.append(
                    {
                        "address": addr,
                        "function": current_function,
                        "expected": expected,
                        "known_cgu": known_cgu,
                    }
                )
            if re.match(r"^\s+[0-9a-f]+:", line):
                prev_insn = line

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
    facts = BinaryFacts(
        build_dir.resolve().parent,
        namespace="kernel-ud2",
        implementation=[Path(__file__)],
        tools=[objdump],
        options={"arch": arch},
    )
    findings = facts.get(ntoskrnl, lambda: scan_ud2(ntoskrnl, objdump, arch))

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

    # ---- driver-fleet sweep (§9.99) ----
    driver_bad: list[dict] = []
    if "--skip-drivers" not in sys.argv:
        nm = find_nm(arch)
        if not nm:
            raise RuntimeError("no nm found — driver sweep cannot run")
        else:
            bins, corpus, flagged = scan_drivers(build_dir, objdump, nm, arch)
            known = [f for f in flagged if f["known_cgu"]]
            driver_bad = [f for f in flagged if not f["known_cgu"]]
            noun = "spins" if arch == "arm64" else "ud2"
            # Print the corpus and its BASIS, not just a count: this sweep used
            # to grade whatever binaries happened to sit in the build dir, so a
            # silent corpus change read as a code change. Say what was judged.
            print(
                f"\nDriver sweep: {len(bins)} Rust-containing binaries scanned "
                f"of {len(corpus.paths)} in corpus\n  corpus basis: {corpus.basis}"
            )
            if corpus.missing:
                print(f"  declared but absent from the build dir ({len(corpus.missing)}):")
                for rel in corpus.missing:
                    print(f"    {rel}")
            if known:
                print(f"  Known residuals ({len(known)}):")
                for f in known:
                    print(f"    {f['binary']}: 0x{f['address']}: {f['function']}  (known)")
            if driver_bad:
                print(
                    f"  UNEXPECTED driver {noun} ({len(driver_bad)} — §9.99 CC/sig-mismatch fold):"
                )
                for f in driver_bad:
                    print(f"    {f['binary']}: 0x{f['address']}: {f['function']}  *** BUG ***")
            if not flagged:
                print(f"  No unexpected driver {noun}.")
            if not corpus.paths:
                # An empty corpus is not a clean sweep, it is no sweep. The whole
                # point of declaring the corpus is that it can now be declared
                # EMPTY by a refactor of mkramdisk.sh, and a gate that silently
                # grades nothing is worse than the directory listing it replaced.
                raise RuntimeError(
                    f"empty binary corpus for {build_dir} — nothing was swept.\n"
                    f"  corpus basis: {corpus.basis}"
                )

    if check_mode and (unexpected or driver_bad):
        noun = "branch-to-self spin" if arch == "arm64" else "ud2"
        print(f"\nFAILED: Unexpected {noun} found — {bug_hint}!")
        sys.exit(1)
    elif not unexpected and not driver_bad:
        print("\nPASSED: All findings are in expected or known-CGU locations.")


if __name__ == "__main__":
    main()
