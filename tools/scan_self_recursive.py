#!/usr/bin/env python3
"""scan_self_recursive.py — find functions that are an INFINITE SELF-JUMP.

A function whose FIRST instruction jumps unconditionally to its own entry is an
infinite loop. Nothing legitimate looks like that: it is always a bug, and always
a *silent* one — the CPU spins forever with no bugcheck, no serial output, no
panic. On a kernel that means a dead hang with an empty log.

WHY THIS LINT EXISTS (2026-07-12, stable-toolchain Axis B)
----------------------------------------------------------
`bootvid-rust` defined its own libc `memcpy`/`memset`:

    #[no_mangle] pub unsafe extern "C" fn memcpy(dst, src, n) -> *mut c_void {
        core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
        dst
    }

which looks obviously correct and is a TRAP. `copy_nonoverlapping` lowers to
`llvm.memcpy`, and the backend lowers *that* back into a **call to `memcpy`** —
the function itself. Tail-call it, and the whole body becomes `jmp .`.

This is unfixable in plain safe Rust: ANY byte-copy loop you write instead is
re-recognised by LLVM's loop-idiom pass and folded back into the same self-call.
The only correct answers are (a) don't define mem* in Rust at all — import the
kernel's (ntoskrnl.exe exports memcpy/memset/memmove) — or (b) use a crate built
with `#![no_builtins]`, i.e. `compiler_builtins`.

It stayed invisible for months because `-Z build-std-features=compiler-builtins-mem`
linked compiler_builtins' own correct mem* alongside the broken twins, and under
`--allow-multiple-definition` lld picked THAT one. The broken code was compiled,
shipped in every bootvid.dll, and simply never selected. The moment build-std went
away the competitor vanished, the broken twin became the only definition, and amd64
hung the instant the kernel drew the boot screen — spinning inside `memset`.

So this is a CLOAK bug (cf. the cdrom `TracePrint` no-op-fn cloak): a real defect
held harmless by a link-order accident. A source-level lint would not have caught
it — the Rust reads fine. Only the emitted code tells the truth, which is why this
scanner works on the LINKED PEs.

PRE-EXISTING DEBT
-----------------
This scanner immediately found ~30 OTHER self-jumps that predate it (see
`tools/lint-baseline/scan_self_recursive.txt`) — the same LLVM idiom-recursion trap
in other clothes:
  * 23× `exp2` (+ exp2f/exp2l) in user-mode DLLs — a C `exp2()` whose body LLVM
    recognised and folded into a call to `exp2`. Same bug, C instead of Rust.
  * `KeRaiseIrql` / `KeLowerIrql` / `IoWMIDeviceObjectToProviderId` in classpnp.sys —
    local stubs shadowing the ntoskrnl imports; they hang if ever called.
  * `ftfd.dll: rust_begin_unwind` — a `#[panic_handler]` written as `loop {}`, i.e. a
    panic that hangs SILENTLY instead of reporting (adopt `panic-common`).
  * `cdrom.sys: abort`.
Those are tracked, owed, and NOT fixed by this lane. `--check` fails only on findings
NOT in the baseline, so a NEW self-recursion (like bootvid's) is caught immediately
while the existing debt stays visible instead of being silently swallowed.

Usage:
    python3 tools/scan_self_recursive.py build-amd64          # report all
    python3 tools/scan_self_recursive.py --check build-amd64  # exit 1 on NEW ones
"""

from __future__ import annotations

import argparse
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

# The first instruction of the function jumps to the function's own address.
_SYM = re.compile(r"^([0-9a-f]+) <([^>]+)>:")
_JMP = re.compile(r"^\s*[0-9a-f]+:\s+(?:jmp|jmpq|b)\s+(?:0x)?([0-9a-f]+)\b")

_SUFFIXES = (".sys", ".dll", ".exe", ".drv")
# Directories of build INTERMEDIATES / staged copies — the same module appears
# there under a hashed name, so scanning them double-reports every finding.
_SKIP_DIRS = (
    "CMakeFiles",
    "host-tools",
    "/boot/stripped/",
    "/boot/staging/",
    "/boot/ramdisk/",
    "/boot/ramdisk-overlay/",
)


def objdump(build_dir: Path) -> str:
    """Locate llvm-objdump next to the repo's toolchain (fall back to PATH)."""
    repo = build_dir.resolve().parent
    cand = repo / ".build-tools/llvm-mingw/bin/llvm-objdump"
    return str(cand) if cand.is_file() else "llvm-objdump"


def scan_pe(objd: str, pe: Path) -> list[tuple[str, str]]:
    try:
        out = subprocess.run(
            [objd, "-d", "--no-show-raw-insn", str(pe)],
            capture_output=True,
            text=True,
            timeout=180,
            check=True,
        ).stdout
    except (OSError, subprocess.SubprocessError) as error:
        detail = getattr(error, "stderr", None) or str(error)
        raise RuntimeError(f"cannot disassemble {pe}: {detail}") from error

    hits: list[tuple[str, str]] = []
    lines = out.split("\n")
    for i, line in enumerate(lines):
        m = _SYM.match(line)
        if not m or i + 1 >= len(lines):
            continue
        addr, sym = int(m.group(1), 16), m.group(2)
        j = _JMP.match(lines[i + 1])
        if j and int(j.group(1), 16) == addr:
            hits.append((pe.name, sym))
    return hits


def load_baseline(repo: Path) -> set[str]:
    """Known pre-existing self-jumps, one `module: symbol` per line. Anything NOT
    listed is a NEW regression and fails --check. Comments start with '#'."""
    f = repo / "tools" / "lint-baseline" / "scan_self_recursive.txt"
    if not f.is_file():
        return set()
    return {
        ln.strip()
        for ln in f.read_text().splitlines()
        if ln.strip() and not ln.lstrip().startswith("#")
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("build_dir", type=Path, help="e.g. build-amd64")
    ap.add_argument("--check", action="store_true", help="exit 1 on NEW (non-baselined) findings")
    args = ap.parse_args()

    if not args.build_dir.is_dir():
        print(f"scan_self_recursive: no such build dir: {args.build_dir}", file=sys.stderr)
        return 2

    repo = args.build_dir.resolve().parent
    objd = objdump(args.build_dir)
    hits: list[tuple[str, str]] = []
    scanned = 0
    facts = None
    # The corpus is DECLARED, not discovered (tools/binary_corpus.py). arm64 has
    # no livecd DAG, so a plain walk of build-arm64/ grades whatever an older
    # ninja left behind instead of what the tree builds and the media ships.
    corpus = governed_binaries(args.build_dir, repo, suffixes=_SUFFIXES, staging_dirs=_SKIP_DIRS)
    for pe in corpus.paths:
        if not pe.is_file():
            continue
        try:
            if facts is None:
                facts = BinaryFacts(
                    repo, namespace="self-recursive", implementation=[Path(__file__)], tools=[objd]
                )
            hits.extend(tuple(row) for row in facts.get(pe, lambda pe=pe: scan_pe(objd, pe)))
        except RuntimeError as error:
            print(f"scan_self_recursive: {error}", file=sys.stderr)
            return 2
        scanned += 1

    if not scanned:
        print(
            f"scan_self_recursive: no PE inputs in {args.build_dir} (corpus basis: {corpus.basis})",
            file=sys.stderr,
        )
        return 2

    baseline = load_baseline(repo)
    found = {f"{mod}: {sym}" for mod, sym in hits}
    new = sorted(found - baseline)

    if not found:
        print(f"scan_self_recursive: {args.build_dir} — clean (no self-jump functions)")
        return 0

    print(
        f"scan_self_recursive: {len(found)} self-jump function(s) "
        f"({len(new)} NEW, {len(found) - len(new)} baselined):\n"
    )
    for entry in sorted(found):
        tag = "NEW  " if entry in set(new) else "known"
        print(f"  [{tag}] {entry}")

    if not new:
        return 0

    print(
        f"\n{len(new)} NEW INFINITE SELF-JUMP(S) — these hang the CPU silently "
        "(no bugcheck, no output).\n"
        "A Rust `#[no_mangle] fn memcpy/memset/...` whose body uses "
        "`copy_nonoverlapping`/`write_bytes`\nis the usual cause: LLVM lowers the body "
        "back into a call to the function ITSELF.\nDon't define mem* in Rust — import "
        "ntoskrnl's exports. See this file's docstring."
    )
    return 1 if args.check else 0


if __name__ == "__main__":
    sys.exit(main())
