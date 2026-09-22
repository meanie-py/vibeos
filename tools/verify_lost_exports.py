#!/usr/bin/env python3
"""Find exports a cohab flip DROPPED: in the .spec, in a commented-out .c, absent from Rust.

WHY THIS EXISTS
---------------
When a DLL is cohab-flipped to Rust, each .c is commented out of the cmake and a `#[no_mangle]`
Rust fn takes over. If the translation silently SKIPPED a function, the export just... vanishes.
Nothing catches it:

  * rustc is happy — nothing references the missing fn.
  * The link is happy — a .spec export with no implementation is not a link error; the DLL is
    built with a smaller export table.
  * The smoke boot is happy — unless the boot path happens to call that exact API.

ws2_32 shipped this way in translation: `bind`, `connect`, `sendto`, `WSAConnect`, `WSAJoinLeaf`
and `WSASendTo` were all silently dropped (tree-sitter cannot parse `IN CONST struct sockaddr *`,
so the scanner never put them in the plan — see scanner.py::_blank_memmodel). The crate compiled
clean on all three architectures. A socket client cannot bind or connect.

WHAT IT WILL NOT DO — AND WHY THAT MATTERS
------------------------------------------
A missing export is NOT automatically a bug:

  * PARTIAL COHAB: shell32 has 199 spec exports with no Rust fn — because 199 of its .c files
    are still compiled. Reporting those as defects is crying wolf (it is the exact false alarm
    a previous sweep produced).
  * DEAD CODE: kernel32's `IsValidLocaleName` sits inside `#if 0 // See kernel32_vista`. The C
    never compiled it either; the real one lives in another module. Reporting it as "lost" is
    also crying wolf — and this checker DID, until it learned to read #if 0.

  * A C TU THE FLIP DELIBERATELY KEPT: a module's `rust_varargs_shim.c` owns the ENTRY point of
    every C-variadic export, because stable Rust can declare but not define one (`va_start`,
    rust-lang/rust#44930) — the body lives in a fixed-arity `*_v` twin in Rust. syssetup's
    `pSetupDebugPrint` is `@ varargs` in the .spec and is implemented exactly there, so reading
    only Rust fns and .S files reported it LOST while the export was in fact served. Any live
    .c counts, not just the shim: if it still compiles, it still defines the symbol.

So a finding requires ALL FOUR: in the .spec, defined in a .c that is COMMENTED OUT in the
cmake, NOT inside a dead preprocessor branch, and not defined by anything that still links
(Rust, assembly, or a .c the cmake still compiles). Anything less is reported as context, not
as a defect.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

ROOT = Path(__file__).resolve().parent.parent

# `@ stdcall GetEnvironmentStringsA() GetEnvironmentStrings` — a FORWARDER. The name after the
# paren list is the real implementation, so the export is served and nothing is lost. Capture it
# so forwarders can be excluded; without this the checker reports them as missing (it did).
_SPEC_ENTRY = re.compile(
    r"^\s*(?:@|\d+)\s+(?:stdcall|cdecl|varargs|thiscall|fastcall)\s+"
    # The tail is: optional forwarder, then an optional trailing `# comment`. Both
    # optional parts must be spelled out. A `$`-anchored bare `(\S+)?` cannot
    # tolerate `... (ptr ptr ptr) # Vista:330` — the line fails to match ENTIRELY and
    # the symbol vanishes from the audit with no diagnostic. SHCreateItemFromIDList
    # was a genuine lost export hiding behind exactly that (found 2026-09-20).
    r"(?:-\S+[ \t]+)*([A-Za-z_]\w*)[ \t]*\([^)]*\)[ \t]*(?:(?!#)(\S+))?[ \t]*(?:#.*)?$",
    re.M,
)
# Plenty of exports are implemented in ASSEMBLY, not C or Rust (ntdll's KiUserExceptionDispatcher
# lives in dispatch/{i386,amd64,arm64}/dispatch.S). Absent-from-Rust does not mean absent.
_ASM_LABEL = re.compile(r"^\s*(?:\.globl|\.global|PUBLIC|FUNC_BEGIN|_?([A-Za-z_]\w*):)", re.M)
_CMAKE_COMMENTED_C = re.compile(r"^\s*#\s*(?:\$\{\w+\}/)?([\w./-]+\.c(?:pp)?)\b", re.M)
# ...and the same names on a line that is NOT a comment. A flip's comment block EXPLAINS which
# .c stayed — "rust_varargs_shim.c is NOT a translation reverted to C" — so the commented-out
# rule alone reads the explanation as the fact and declares a live file flipped. Listed live
# wins: that is the line cmake actually compiles.
_CMAKE_LIVE_C = re.compile(r"^(?![ \t]*#)[^#\n]*?(?:\$\{\w+\}/)?([\w./-]+\.c(?:pp)?)\b", re.M)
# Every `#[no_mangle]` fn DEFINITION. The previous spelling demanded `#[no_mangle]` immediately
# followed by `pub unsafe extern "..."` on the very next line, which three real shapes break:
# a comment between the attribute and the fn (snmpapi's SnmpUtilDbgPrint carries a six-line
# note), a second attribute, and an indented item. Each miss reads as a LOST export — the
# opposite of the truth — so this is the same tolerance `fn_coverage._RS_FN` already carries.
# `[\s\S]*?` spans the gap but the `#[no_mangle]` must still be the nearest attribute block.
# A FOURTH shape, found 2026-09-19 when shell32/utils.rs was cohab-flipped: the attribute and
# the fn on ONE line -- `#[no_mangle] pub unsafe extern "system" fn SHOpenEffectiveToken(..)`.
# The old spelling required the attribute line to END before the fn, so it reported
# SHOpenEffectiveToken (SHELL32.235) as a LOST export while utils.rs was defining it. A sweep
# of sdk/lib/rust found 209 such definitions across 46 files invisible to this regex; each one
# is a false LOST export waiting for its .c to be commented out. Hence the newline after the
# attribute is now OPTIONAL.
_RUST_FN = re.compile(
    r"#\[no_mangle\](?:[ \t]*(?://[^\n]*)?\n(?:[ \t]*(?://[^\n]*|#\[[^\n]*\])?\n)*)?"
    r"[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:unsafe[ \t]+)?"
    r"(?:extern[ \t]+(?:\"\w+\"|\$\w+)[ \t]+)?fn[ \t]+(\w+)"
)


def dead_regions(text: str) -> list[tuple[int, int]]:
    """Byte ranges inside `#if 0` / `#endif`. Nesting-aware."""
    out: list[tuple[int, int]] = []
    stack: list[int] = []
    for m in re.finditer(r"^\s*#\s*(if\s+0\b|if\b|ifdef\b|ifndef\b|endif\b)", text, re.M):
        kind = m.group(1)
        if kind.startswith("if 0"):
            stack.append(m.start())
        elif kind.startswith(("if", "ifdef", "ifndef")) and stack:
            stack.append(-1)
        elif kind.startswith("endif") and stack:
            opened = stack.pop()
            if not stack and opened != -1:
                out.append((opened, m.end()))
    return out


def defined_live(cfile: Path, sym: str) -> bool:
    """Is `sym` DEFINED (body and all) at column 0 in this .c, OUTSIDE any `#if 0` block?

    ReactOS puts the name at column 0 for a PROTOTYPE as well as a definition, so matching
    `^sym(` alone cannot tell them apart. kernel32's four Get/SetFirmwareEnvironmentVariableEx*
    are declared in client/sysinfo.c and defined NOWHERE — matching the declaration reported
    them as "lost exports on master", which they are not. Require the `{`.

    The name is at column 0 only in the house style where the return type sits on its own line.
    Hand-written companions do not follow it — `rust_varargs_shim.c` writes
    `void CDECL pSetupDebugPrint(...)` on one line — so the specifiers are allowed to precede
    the name as long as the LINE starts at column 0 and no `(` intervenes. That last part is
    what keeps a call inside an expression out; the `{` requirement does the rest.
    """
    try:
        text = cfile.read_text(errors="replace")
    except OSError:
        return False
    pattern = rf"^(?:[A-Za-z_][^;()\n]*?[\s*])?{re.escape(sym)}\s*\("
    for m in re.finditer(pattern, text, re.M):
        # walk to the matching close paren, then demand a body
        i, depth = m.end() - 1, 0
        while i < len(text):
            if text[i] == "(":
                depth += 1
            elif text[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        rest = text[i + 1 : i + 200].lstrip()
        if not rest.startswith("{"):
            continue  # a prototype, not a definition
        if not any(a <= m.start() <= b for a, b in dead_regions(text)):
            return True
    return False


def audit(name: str, ddir: Path, crate: Path) -> tuple[list[tuple[str, str]], int]:
    spec = next(iter(ddir.rglob(f"{name}.spec")), None)
    if spec is None or not crate.is_dir():
        return [], 0
    exports = set()
    for sym, fwd in _SPEC_ENTRY.findall(spec.read_text(errors="replace")):
        if fwd and not fwd.startswith(("#", "-")):
            continue  # forwarded to another symbol — the export is served
        exports.add(sym)

    rust = set()
    for rf in source_files(crate):
        rust |= set(_RUST_FN.findall(rf.read_text(errors="replace")))
    # Assembly implementations count too.
    asm = set()
    for af in list(ddir.rglob("*.S")) + list(ddir.rglob("*.s")) + list(ddir.rglob("*.asm")):
        asm |= set(re.findall(r"\b([A-Za-z_]\w*)\b", af.read_text(errors="replace")))
    rust |= asm

    cml = ddir / "CMakeLists.txt"
    cml_text = cml.read_text(errors="replace") if cml.is_file() else ""
    # Compare by BASENAME, not the raw captured path text. A "commented" hit can come from
    # ordinary prose in an explanatory comment above a live entry -- kernel32's dep.c is a
    # real example: "# dep.c is genuinely new code (extracted from proc.c upstream) ..."
    # sits directly above the live `client/dep.c` line. The prose captures the bare
    # basename "dep.c" while the live regex captures the full "client/dep.c" path, so a
    # raw string set-difference fails to cancel them and the file is wrongly flipped to
    # "commented out". `flipped` below already matches by basename, so do the same here.
    commented_names = {Path(f).name for f in _CMAKE_COMMENTED_C.findall(cml_text)}
    live_names = {Path(f).name for f in _CMAKE_LIVE_C.findall(cml_text)}
    commented = commented_names - live_names
    # `.cpp` too: shell32 and devmgr are C++ modules, and reading only `.c` made every export
    # they define in a .cpp look undefined — three of shell32's reported LOST were really
    # "the file this tool cannot see".
    all_c = set(ddir.rglob("*.c")) | set(ddir.rglob("*.cpp"))
    flipped = {c for c in all_c if any(Path(f).name == c.name for f in commented)}
    # Every .c the cmake still compiles still DEFINES its symbols, so it serves the export just
    # as Rust or assembly would. This is what `rust_varargs_shim.c` is: the entry point of a
    # C-variadic export, which stable Rust cannot define at all.
    live = all_c - flipped

    lost = []
    for sym in sorted(exports - rust):
        if any(defined_live(cf, sym) for cf in live):
            continue
        for cf in flipped:
            if defined_live(cf, sym):
                lost.append((sym, str(cf.relative_to(ddir))))
                break
    return lost, len(exports - rust)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dll", help="e.g. ws2_32 (default: every spec-paired cohab crate)")
    ap.add_argument("--check", action="store_true", help="exit 1 on any lost export")
    args = ap.parse_args()

    crates = []
    for cdir in sorted((ROOT / "sdk" / "lib" / "rust").glob("*-rust")):
        name = cdir.name[:-5]
        if args.dll and name != args.dll:
            continue
        ddir = next((d for d in (ROOT / "dll").rglob(name) if d.is_dir()), None)
        if ddir and next(iter(ddir.rglob(f"{name}.spec")), None):
            crates.append((name, ddir, cdir / "src"))

    total = 0
    for name, ddir, crate in crates:
        lost, missing = audit(name, ddir, crate)
        total += len(lost)
        for sym, cf in lost:
            print(
                f"{name}: LOST export {sym} — implemented in {cf}, "
                "which is commented out; no Rust fn"
            )
        print(f"{name}: {len(lost)} lost, {missing} missing-but-still-C/dead", file=sys.stderr)
    if total:
        print(f"{total} lost export(s) — the DLL would ship without them.", file=sys.stderr)
    return 1 if (total and args.check) else 0


if __name__ == "__main__":
    raise SystemExit(main())
