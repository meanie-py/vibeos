"""Export-coverage audit: does the translated Rust actually DEFINE the DLL's exports?

Motivation (a real ole32 defect): `StgCreateDocfile` — a genuine ole32 export —
came out of the translation pipeline as a module-private, Rust-ABI
`unsafe fn StgCreateDocfile(...)`.  No `#[no_mangle]`, no `extern "system"`.
The symbol `_StgCreateDocfile@16` was therefore simply ABSENT from the linked
DLL, and NOTHING noticed until the i386 link failed.  On an arch where some
other translation unit happened to satisfy the reference it could have linked
and just been silently missing at runtime.

Two failure shapes, both caught here:

  MISSING          the spec exports it; no Rust source defines it at all
  NOT_EXPORTED     a Rust fn by that name exists but has no #[no_mangle]
                   (→ name-mangled, invisible to the linker) — the StgCreateDocfile shape
  ABI_MISMATCH     it IS exported, but with the wrong calling convention.  A
                   `stdcall` spec entry emitted as `extern "C"` links fine and
                   then corrupts the i386 stack on every call: the callee pops
                   nothing, the caller pops nothing → silent stack leak/crash.

Exports may legitimately still live in the C that remains (cohabitation), so
this takes the list of Rust sources to check and reports what is missing FROM
THEM.  The caller decides whether a miss is expected (still-C) or a bug.

    from rustconv.export_audit import audit_exports
    for line in audit_exports(Path("dll/win32/ole32/ole32.spec"), rust_srcs):
        print(line)

CLI:
    python -m rustconv.export_audit dll/win32/ole32/ole32.spec sdk/lib/rust/ole32-rust/src
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

from .spec_parser import SpecEntry, parse_spec

# ---------------------------------------------------------------------------
# Rust fn definitions
# ---------------------------------------------------------------------------

# Every `fn NAME(` / `fn NAME<`.  The signature PREFIX (attributes + modifiers)
# is then parsed BACKWARDS from the `fn` token — attributes are not reliably on
# their own line (oleaut32's vartype.rs has `#[no_mangle] pub unsafe extern
# "system" fn VarI8FromStr(`, and a line-anchored regex silently missed it,
# reporting a perfectly good export as MISSING).
_RE_FN_TOKEN = re.compile(r"\bfn\s+(?P<name>[A-Za-z_]\w*)\s*[(<]")

# Anchored at the END of the text preceding `fn`: the contiguous attribute +
# modifier run that belongs to this signature.
_RE_FN_PREFIX = re.compile(
    r"(?P<attrs>(?:#\[[\s\S]*?\]\s*)*)"
    r"(?:pub(?:\s*\([^)]*\))?\s+)?"
    r"(?:default\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r'(?:extern\s+(?:"(?P<abi>[^"]*)"\s+)?)?'
    r"$"
)

_RE_NO_MANGLE = re.compile(r"#\[\s*(?:unsafe\s*\(\s*)?no_mangle\s*\)?\s*\]")
_RE_EXPORT_NAME = re.compile(r'#\[\s*export_name\s*=\s*"(?P<sym>[^"]+)"\s*\]')

# `extern "system"` and `extern "stdcall"` are the same thing on i386 (and
# `system` is what the codebase standardises on); either satisfies a stdcall
# spec entry.  `extern "C"` does NOT.
_OK_FOR: dict[str, frozenset[str]] = {
    "stdcall": frozenset({"system", "stdcall", "win64"}),
    "cdecl": frozenset({"C", "cdecl"}),
    "varargs": frozenset({"C", "cdecl"}),
    "fastcall": frozenset({"fastcall", "system"}),  # cfg'd per-arch by postprocess
    "thiscall": frozenset({"thiscall"}),
}


@dataclass
class RustFn:
    name: str
    abi: str | None  # None = Rust ABI (no `extern`)
    no_mangle: bool
    export_name: str | None
    path: Path
    line: int


@dataclass
class Finding:
    kind: str  # MISSING | NOT_EXPORTED | ABI_MISMATCH
    name: str
    expected: str  # expected Rust ABI, e.g. 'extern "system"'
    found: str  # what we actually found (or "nothing")
    where: str  # file:line, or "" when missing

    def __str__(self) -> str:
        loc = f" at {self.where}" if self.where else ""
        return f"{self.kind:<12} {self.name}: expected {self.expected}, found {self.found}{loc}"


def _expected_abi(entry: SpecEntry) -> str:
    if entry.convention == "stdcall":
        return 'extern "system"'
    if entry.convention in ("cdecl", "varargs"):
        return 'extern "C"'
    if entry.convention == "fastcall":
        return 'extern "fastcall"'
    if entry.convention == "thiscall":
        return 'extern "thiscall"'
    return 'extern "C"'


def _is_declaration(text: str, sig_start: int) -> bool:
    """True if this `fn` has NO body — i.e. it's a decl in an `extern "C" { .. }`
    block (or a trait method), not a definition.  Those must not be mistaken for
    an implementation: `fn StgCreateDocfileOnILockBytes(...);` inside an extern
    block would otherwise mask the real definition (or a real MISSING).
    """
    i = text.find("(", sig_start)
    if i < 0:
        return False
    depth = 0
    for j in range(i, len(text)):
        c = text[j]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                # scan past the return type / where-clause to the ';' or '{'
                for k in range(j + 1, len(text)):
                    if text[k] == ";":
                        return True
                    if text[k] == "{":
                        return False
                return True
    return False


def scan_rust_fns(paths: list[Path]) -> dict[str, RustFn]:
    """Index every `fn` DEFINITION in the given Rust sources by name.

    A `#[no_mangle]` definition always wins over a non-exported one of the same
    name (the same name can legitimately appear as a private helper in another
    module).  Bodiless declarations (extern blocks) are skipped entirely.
    """
    fns: dict[str, RustFn] = {}

    def record(key: str, fn: RustFn) -> None:
        prev = fns.get(key)
        if prev is None or (fn.no_mangle and not prev.no_mangle):
            fns[key] = fn

    for p in paths:
        text = p.read_text(errors="replace")
        for name, fn in scan_macro_generated_fns(text, p).items():
            record(name, fn)
        for m in _RE_FN_TOKEN.finditer(text):
            if _is_declaration(text, m.end() - 1):
                continue
            pm = _RE_FN_PREFIX.search(text[max(0, m.start() - 2000) : m.start()])
            attrs = (pm.group("attrs") or "") if pm else ""
            abi = pm.group("abi") if pm else None
            en = _RE_EXPORT_NAME.search(attrs)
            fn = RustFn(
                name=m.group("name"),
                abi=abi,
                no_mangle=bool(_RE_NO_MANGLE.search(attrs)),
                export_name=en.group("sym") if en else None,
                path=p,
                line=text.count("\n", 0, m.start("name")) + 1,
            )
            record(fn.export_name or fn.name, fn)
    return fns


# ---------------------------------------------------------------------------
# macro_rules!-generated exports
# ---------------------------------------------------------------------------
#
# ole32's usrmarshal.c generates its 24 H*_User{Size,Marshal,Unmarshal,Free}
# exports from a C macro (`IMPL_WIREM_HANDLE(HWND)`), and the translation kept
# that shape as a `macro_rules!`.  A regex that only sees literal `fn NAME`
# would call all 24 of them MISSING — 24 false positives on a DLL that links and
# boots.  So: resolve them.
#
#   macro_rules! IMPL_WIREM_HANDLE {
#       ($ty:ty, $size:ident, ...) => { #[no_mangle] pub unsafe extern "system" fn $size(..)
#   IMPL_WIREM_HANDLE!(HWND, HWND_UserSize, ...);
#
# → metavar `$size` is at matcher position 1 and is used as `fn $size`, so
#   invocation arg 1 (`HWND_UserSize`) is a real exported symbol.

_RE_MACRO_DEF = re.compile(r"macro_rules!\s*(?P<name>\w+)\s*\{")
_RE_METAVAR_DECL = re.compile(
    r"\$(\w+)\s*:\s*(?:ident|ty|expr|literal|tt|path|block|item|meta|pat|pat_param|stmt|vis|lifetime)\b"
)
_RE_FN_METAVAR = re.compile(
    r'(?:extern\s+"(?P<abi>[^"]*)"\s+)?fn\s+\$(?P<meta>\w+)',
)


def _brace_body(text: str, open_idx: int) -> tuple[str, int]:
    """Return the text between the brace at `open_idx` and its match, plus the end index."""
    depth = 0
    for i in range(open_idx, len(text)):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[open_idx + 1 : i], i
    return text[open_idx + 1 :], len(text)


def _split_top_level(args: str) -> list[str]:
    """Split a macro arg list on top-level commas (ignoring nested (), [], {}, <>)."""
    out: list[str] = []
    depth = 0
    cur = ""
    for c in args:
        if c in "([{<":
            depth += 1
        elif c in ")]}>":
            depth -= 1
        if c == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += c
    if cur.strip():
        out.append(cur.strip())
    return out


def scan_macro_generated_fns(text: str, path: Path) -> dict[str, RustFn]:
    """Resolve `#[no_mangle] fn $meta` definitions inside macro_rules! bodies."""
    fns: dict[str, RustFn] = {}

    # 1. index the macros defined in this file
    macros: dict[str, dict[str, tuple[str | None, bool]]] = {}
    positions: dict[str, list[str]] = {}
    for m in _RE_MACRO_DEF.finditer(text):
        body, _ = _brace_body(text, text.index("{", m.end() - 1))
        metavars = list(dict.fromkeys(_RE_METAVAR_DECL.findall(body)))
        if not metavars:
            continue
        gen: dict[str, tuple[str | None, bool]] = {}
        for fm in _RE_FN_METAVAR.finditer(body):
            # attrs precede the signature; #[no_mangle] within the previous ~200 chars
            prefix = body[max(0, fm.start() - 200) : fm.start()]
            gen[fm.group("meta")] = (fm.group("abi"), bool(_RE_NO_MANGLE.search(prefix)))
        if gen:
            macros[m.group("name")] = gen
            positions[m.group("name")] = metavars

    # 2. resolve the invocations
    for mac, gen in macros.items():
        for inv in re.finditer(rf"\b{re.escape(mac)}!\s*\(", text):
            depth = 0
            start = inv.end() - 1
            end = start
            for i in range(start, len(text)):
                if text[i] == "(":
                    depth += 1
                elif text[i] == ")":
                    depth -= 1
                    if depth == 0:
                        end = i
                        break
            args = _split_top_level(text[start + 1 : end])
            order = positions[mac]
            for meta, (abi, no_mangle) in gen.items():
                if meta not in order:
                    continue
                idx = order.index(meta)
                if idx >= len(args):
                    continue
                name = args[idx]
                if not re.fullmatch(r"[A-Za-z_]\w*", name):
                    continue
                line = text.count("\n", 0, inv.start()) + 1
                fns[name] = RustFn(name, abi, no_mangle, None, path, line)
    return fns


def collect_rust_sources(paths: list[Path]) -> list[Path]:
    """Expand directories to their *.rs files; pass .rs files through."""
    out: list[Path] = []
    for p in paths:
        p = Path(p)
        if p.is_dir():
            out.extend(sorted(p.rglob("*.rs")))
        elif p.suffix == ".rs":
            out.append(p)
    return out


def audit_exports_detailed(spec_path: Path, rust_srcs: list[Path]) -> list[Finding]:
    """Structured form of :func:`audit_exports`."""
    spec = parse_spec(Path(spec_path))
    fns = scan_rust_fns(collect_rust_sources(list(rust_srcs)))

    findings: list[Finding] = []
    for entry in spec.values():
        # Skip anything that needs no symbol in THIS module's object code:
        #   @ stub X / -stub   → spec2def synthesises the body (stubs.c)
        #   @ extern X         → data export, not a fn
        #   ... ) other.dll.Y  → the .def forwards it straight at another DLL
        if not entry.needs_implementation:
            continue

        expected = _expected_abi(entry)
        # `entry.symbol`, NOT `entry.name`: a bare trailing target renames the symbol the
        # linker looks for (`ldap_first_entry` is implemented as `WLDAP32_ldap_first_entry`),
        # and it is the RENAMED one that has to exist.
        want = entry.symbol
        label = want if want == entry.name else f"{want} (export {entry.name})"
        fn = fns.get(want)
        if fn is None:
            findings.append(
                Finding("MISSING", label, expected, "nothing (no Rust fn by that name)", "")
            )
            continue

        where = f"{fn.path}:{fn.line}"
        # `#[export_name = "X"]` also places the symbol (though AGENTS.md warns it
        # double-decorates on i686 stdcall — `#[no_mangle]` is the house style).
        if not (fn.no_mangle or fn.export_name):
            shape = f'extern "{fn.abi}"' if fn.abi else "Rust-ABI"
            findings.append(
                Finding(
                    "NOT_EXPORTED",
                    label,
                    expected,
                    f"a private {shape} fn with NO #[no_mangle]",
                    where,
                )
            )
            continue

        ok = _OK_FOR.get(entry.convention, frozenset({"C"}))
        if (fn.abi or "Rust") not in ok:
            found = f'extern "{fn.abi}"' if fn.abi else "the Rust ABI (no extern)"
            findings.append(Finding("ABI_MISMATCH", label, expected, found, where))

    findings.sort(key=lambda f: (f.kind, f.name))
    return findings


def audit_exports(spec_path: Path, rust_srcs: list[Path]) -> list[str]:
    """Every fn the .spec exports must be a `#[no_mangle]` fn in `rust_srcs`.

    Returns one actionable line per problem (name, expected convention, what was
    actually found, and where).  Empty list == clean.
    """
    return [str(f) for f in audit_exports_detailed(Path(spec_path), list(rust_srcs))]


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    if len(argv) < 2:
        print("usage: python -m rustconv.export_audit <file.spec> <rust-src-dir|file.rs>...")
        return 2

    spec_path = Path(argv[0])
    srcs = [Path(a) for a in argv[1:]]
    findings = audit_exports_detailed(spec_path, srcs)

    spec = parse_spec(spec_path)
    need = sum(1 for e in spec.values() if e.needs_implementation)
    skipped = len(spec) - need
    print(
        f"{spec_path.name}: {len(spec)} entries, {need} need a real symbol "
        f"({skipped} skipped: stub/extern/forwarded)"
    )

    if not findings:
        print("CLEAN — every implementable export is a #[no_mangle] fn with the right ABI.")
        return 0

    by_kind: dict[str, list[Finding]] = {}
    for f in findings:
        by_kind.setdefault(f.kind, []).append(f)
    for kind, group in sorted(by_kind.items()):
        print(f"\n{kind} ({len(group)}):")
        for f in group:
            print(f"  {f}")
    print(f"\n{len(findings)} finding(s)")
    return 1


if __name__ == "__main__":
    sys.exit(main())
