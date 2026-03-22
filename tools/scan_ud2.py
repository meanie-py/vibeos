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
    "panic_cold",
    "panic_cold_display",
    "panic_nounwind_cold",
    "panic_nounwind",
    "abort",
    "rust_begin_unwind",
}


def find_objdump(arch: str) -> str:
    """Find a working objdump for the given architecture."""
    candidates = {
        "arm64": ["aarch64-w64-mingw32-objdump", "llvm-objdump"],
        "amd64": ["x86_64-w64-mingw32-objdump", "llvm-objdump"],
        "i386": ["i686-w64-mingw32-objdump", "llvm-objdump"],
    }
    for c in candidates.get(arch, ["llvm-objdump"]):
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
        capture_output=True, text=True, timeout=120,
    )

    findings = []
    current_function = None

    for line in result.stdout.splitlines():
        # Match function labels like "0046b350 <_IopAcquireFileObjectLock@16>:"
        func_match = re.match(r'^([0-9a-f]+)\s+<(.+)>:', line)
        if func_match:
            current_function = func_match.group(2)
            continue

        # Match ud2 instruction (0f 0b)
        ud2_match = re.search(r'^\s+([0-9a-f]+):\s+0f 0b\s+ud2', line)
        if ud2_match and current_function:
            addr = ud2_match.group(1)
            expected = any(e in current_function for e in EXPECTED_UD2)
            findings.append({
                "address": addr,
                "function": current_function,
                "expected": expected,
            })

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
    unexpected = [f for f in findings if not f["expected"]]

    if expected:
        print(f"\n  Expected ud2 ({len(expected)} — unreachable panic paths):")
        for f in expected:
            print(f"    0x{f['address']}: {f['function']}")

    if unexpected:
        print(f"\n  UNEXPECTED ud2 ({len(unexpected)} — likely LLVM miscompilation):")
        for f in unexpected:
            print(f"    0x{f['address']}: {f['function']}  *** BUG ***")

    if not findings:
        print("  No ud2 instructions found.")

    print(f"\nTotal: {len(findings)} ud2 ({len(expected)} expected, {len(unexpected)} unexpected)")

    if check_mode and unexpected:
        print("\nFAILED: Unexpected ud2 found — LLVM miscompilation detected!")
        sys.exit(1)
    elif not unexpected:
        print("\nPASSED: All ud2 are in expected locations.")


if __name__ == "__main__":
    main()
