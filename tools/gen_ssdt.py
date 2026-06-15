#!/usr/bin/env python3
"""Generate Rust SSDT (MainSSDT / MainSSPT / MainNumberOfSysCalls) from sysfuncs.h.

Reads ntoskrnl/include/sysfuncs.h, parses the SVC_(Name, argcount) entries,
and emits sdk/lib/rust/ntoskrnl-rust/src/ke/ssdt.rs with extern "system" forward
declarations and the three exported statics.

The arg counts in sysfuncs.h are syscall arg counts (each arg pointer-sized,
even on i386 where smaller types get widened to ULONG). MainSSPT is
`argcount * sizeof(void*)` per entry — we encode that via core::mem::size_of.

Run after editing sysfuncs.h (rare). Idempotent — overwrites the output file.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
SYSFUNCS = REPO / "ntoskrnl" / "include" / "sysfuncs.h"
OUTPUT = REPO / "sdk" / "lib" / "rust" / "ntoskrnl-rust" / "src" / "ke" / "ssdt.rs"

SVC_RE = re.compile(r"^\s*SVC_\(\s*(\w+)\s*,\s*(\d+)\s*\)\s*$")


def parse() -> list[tuple[str, int]]:
    entries = []
    for lineno, line in enumerate(SYSFUNCS.read_text().splitlines(), 1):
        line = line.split("//")[0].rstrip()
        if not line.strip():
            continue
        m = SVC_RE.match(line)
        if not m:
            sys.exit(f"{SYSFUNCS}:{lineno}: unparsed line: {line!r}")
        entries.append((m.group(1), int(m.group(2))))
    return entries


def render(entries: list[tuple[str, int]]) -> str:
    n = len(entries)
    out = []
    out.append(
        "// AUTO-GENERATED from ntoskrnl/include/sysfuncs.h by tools/gen_ssdt.py.\n"
        "// Do not edit by hand — re-run the generator if sysfuncs.h changes.\n"
        "//\n"
        "// Materializes MainSSDT (function pointer table), MainSSPT (per-syscall\n"
        "// argument byte counts), and MainNumberOfSysCalls in Rust. Replaces the\n"
        "// macro-expansion of <internal/napi.h> inside ke_rust_shims.c so the shim\n"
        "// can drop its <ntoskrnl.h> include.\n"
        "//\n"
        "// MainSSDT entries are taken as `<name> as usize` so the ABI of the\n"
        "// underlying Rust impl (NTAPI = stdcall on i386, cdecl on amd64/arm64)\n"
        "// is opaque here — KiFastCallEntry asm does the real call. The forward\n"
        "// declarations below are sized with `usize` arguments so on i386 the\n"
        "// stdcall name decoration (_Name@N) matches the real impl.\n"
    )

    out.append("#![allow(non_upper_case_globals)]\n")
    out.append("#![allow(improper_ctypes)]\n")
    out.append("#![allow(unused_imports)]\n\n")
    out.append(
        "/// SSDT entry — a syscall implementation. The arg count + ABI is\n"
        "/// captured by the per-syscall extern decl below; here we erase to\n"
        "/// `*const ()` because Rust forbids `fn() as fn()` casts between\n"
        "/// different signatures and forbids `fn() as integer` in const\n"
        "/// init, but allows `fn() as *const ()`. On every supported arch a\n"
        "/// fn pointer is the same size as `ULONG_PTR`, so consumers re-cast\n"
        "/// `addr_of_mut!(MainSSDT)` to `*mut ULONG_PTR` to match the\n"
        "/// historical C `ULONG_PTR MainSSDT[]` signature without changing\n"
        "/// the bytes.\n"
        "pub type SSDTEntry = *const ();\n\n"
    )

    # Forward declarations grouped by argcount so we don't repeat the (`_:usize, ...`)
    # block 296 times unnecessarily — Rust still tolerates one block per fn but
    # this keeps it tidy.
    out.append('extern "system" {\n')
    for name, argc in entries:
        args = ", ".join(["_: usize"] * argc) if argc > 0 else ""
        out.append(f"    pub fn Nt{name}({args});\n")
    out.append("}\n\n")

    # Note: ke_defs.rs has its own NUMBER_OF_SYSCALLS const (0x1000, used as
    # KiServiceLimit) — we don't redefine it here. The actual table length is
    # captured in the array sizes below.

    out.append(
        "// `static mut` (not `static`) so consumers can `addr_of_mut!` to match\n"
        "// the original C `ULONG_PTR MainSSDT[]` declaration.\n"
        "#[no_mangle]\n"
        f"pub static mut MainSSDT: [SSDTEntry; {n}] = [\n"
    )
    for name, _argc in entries:
        out.append(f"    Nt{name} as SSDTEntry,\n")
    out.append("];\n\n")

    out.append(
        "// `argcount * sizeof(void*)` per entry — pointer size differs per arch\n"
        "// so the multiplier is computed at const-eval time.\n"
        "#[no_mangle]\n"
        f"pub static mut MainSSPT: [u8; {n}] = [\n"
    )
    for _name, argc in entries:
        out.append(f"    ({argc} * core::mem::size_of::<usize>()) as u8,\n")
    out.append("];\n\n")

    out.append(f"#[no_mangle]\npub static mut MainNumberOfSysCalls: u32 = {n};\n")
    return "".join(out)


def main() -> int:
    entries = parse()
    OUTPUT.write_text(render(entries))
    print(f"wrote {OUTPUT} ({len(entries)} syscalls)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
