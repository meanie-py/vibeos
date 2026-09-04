"""Export-coverage audit: does the translated Rust actually DEFINE the DLL's exports?

Motivation (a real ole32 defect): `StgCreateDocfile` — a genuine ole32 export —
came out of the translation pipeline as a module-private, Rust-ABI
`unsafe fn StgCreateDocfile(...)`.  No `#[no_mangle]`, no `extern "system"`.
The symbol `_StgCreateDocfile@16` was therefore simply ABSENT from the linked
DLL, and NOTHING noticed until the i386 link failed.  On an arch where some
other translation unit happened to satisfy the reference it could have linked
and just been silently missing at runtime.

Two failure shapes, both caught here:

  MISSING          the spec exports it; nothing defines it — not the Rust, and not any
                   .c the build still compiles (needs `c_dir=`; without it every
                   still-C export lands here and a real drop hides among thousands)
  STILL_C          no Rust definition, but a still-compiled .c has one — cohabitation
                   working as intended, and the reason MISSING is trustworthy
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

from .c_defs import c_provided_names, compiled_tus
from .profile_derive import ProfileDeriveError
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

# The modifier run on the signature line itself; `abi` is the one thing read from it.
_RE_SIG_MODS = re.compile(
    r"(?:pub(?:\s*\([^)]*\))?\s+)?"
    r"(?:default\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r'(?:extern\s+(?:"(?P<abi>[^"]*)"\s+)?)?$'
)


def _prefix_of(text: str, fn_at: int) -> tuple[str, str | None]:
    """The attributes attached to the `fn` at `fn_at`, and its declared ABI.

    Read by walking BACK over whole lines rather than with one regex anchored at the end of
    the preceding text. That regex could satisfy its anchor by letting `#\[[\s\S]*?\]` swallow
    the code between an EARLIER function's attributes and this signature, so a definition
    inherited its neighbour's attribute set: ex-rust's rundown twins both reported the
    attributes of the `_impl` helper above them, which loses both the `#[no_mangle]` (an
    export reads as private) and the `#[cfg]` (the two arch twins become indistinguishable).
    """
    line_start = text.rfind("\n", 0, fn_at) + 1
    head = text[line_start:fn_at]
    m = _RE_SIG_MODS.search(head)
    abi = m.group("abi") if m else None

    # Attributes may share the signature's line (`#[no_mangle] pub unsafe extern "C" fn X(`).
    attrs = [head[: m.start()] if m else head]
    lines = text[:line_start].splitlines()
    i = len(lines)
    depth = 0
    while i > 0:
        prev = lines[i - 1]
        stripped = prev.strip()
        if depth:
            # Inside a multi-line attribute, walking upward toward its `#[`.
            attrs.append(prev)
            depth += stripped.count("]") - stripped.count("[")
            i -= 1
            if stripped.startswith("#["):
                depth = 0
            continue
        if not stripped or stripped.startswith("//"):
            # Blank lines and comments sit inside an attribute run without ending it.
            i -= 1
            continue
        if stripped.startswith("#["):
            attrs.append(prev)
            i -= 1
            continue
        if stripped.endswith("]") and "[" not in stripped:
            attrs.append(prev)
            depth = 1
            i -= 1
            continue
        break
    return "\n".join(reversed(attrs)), abi


_RE_NO_MANGLE = re.compile(r"#\[\s*(?:unsafe\s*\(\s*)?no_mangle\s*\)?\s*\]")
# Which arch's twin this definition is. A per-arch fork is NOT a defect — `fastcall` is a real
# convention on i386 and is spelled `C` on amd64/arm64, so ex-rust's 40 rundown exports are
# correct on both sides. Indexing by name alone made the scan keep whichever twin it met first
# and report the other arch's spelling as an ABI_MISMATCH: 33 false findings, all of them the
# shape the audit exists to distinguish from a real one.
_RE_CFG_X86 = re.compile(r'#\[\s*cfg\s*\(\s*target_arch\s*=\s*"x86"\s*\)\s*\]')
_RE_CFG_NOT_X86 = re.compile(r'#\[\s*cfg\s*\(\s*not\s*\(\s*target_arch\s*=\s*"x86"\s*\)\s*\)\s*\]')
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
    cfg_arch: str | None = None  # "x86" | "not-x86" | None (unconditional)


@dataclass
class Finding:
    kind: str  # MISSING | STILL_C | NOT_EXPORTED | ABI_MISMATCH
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


def scan_rust_fns(paths: list[Path], arch: str = "i386") -> dict[str, RustFn]:
    """Index every `fn` DEFINITION in the given Rust sources by name.

    A `#[no_mangle]` definition always wins over a non-exported one of the same
    name (the same name can legitimately appear as a private helper in another
    module).  Bodiless declarations (extern blocks) are skipped entirely.

    Where a name has per-arch twins, the one THIS arch compiles wins — the other is not in the
    binary being audited, and judging its convention is judging code that is not there.
    """
    want_cfg = "x86" if arch == "i386" else "not-x86"
    fns: dict[str, RustFn] = {}

    def better(fn: RustFn, prev: RustFn) -> bool:
        if fn.no_mangle != prev.no_mangle:
            return fn.no_mangle
        # This arch's twin beats the other arch's; either beats nothing in particular.
        rank = lambda f: 2 if f.cfg_arch == want_cfg else (0 if f.cfg_arch else 1)  # noqa: E731
        return rank(fn) > rank(prev)

    def record(key: str, fn: RustFn) -> None:
        prev = fns.get(key)
        if prev is None or better(fn, prev):
            fns[key] = fn

    for p in paths:
        text = p.read_text(errors="replace")
        for name, fn in scan_macro_generated_fns(text, p).items():
            record(name, fn)
        for m in _RE_FN_TOKEN.finditer(text):
            if _is_declaration(text, m.end() - 1):
                continue
            attrs, abi = _prefix_of(text, m.start())
            en = _RE_EXPORT_NAME.search(attrs)
            fn = RustFn(
                name=m.group("name"),
                abi=abi,
                no_mangle=bool(_RE_NO_MANGLE.search(attrs)),
                export_name=en.group("sym") if en else None,
                path=p,
                line=text.count("\n", 0, m.start("name")) + 1,
                cfg_arch=(
                    "not-x86"
                    if _RE_CFG_NOT_X86.search(attrs)
                    else ("x86" if _RE_CFG_X86.search(attrs) else None)
                ),
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


_CARGO_PATH_DEP = re.compile(r'path[ \t]*=[ \t]*"([^"]+)"')
# How a CMakeLists names the Rust that goes into the module. Three spellings, all explicit:
#   rust_crate_sources(X_RUST_SRCS "${WS}" <crate>)   — a cohab module linking one .a
#   add_cargo_module(<mod> <cargo-crate> <out> ... BODY_CRATES a b c ...)
#   .rust_lib("<crate>", ..)                          — in the cargo crate's build.rs
_CMAKE_RUST_CRATE = re.compile(r"rust_crate_sources\([^)]*?\s([\w.-]+)\s*\)")
_ADD_CARGO_MODULE = re.compile(r"add_cargo_module\(\s*([\w.-]+)\s+([\w.-]+)(?P<rest>[^)]*)\)", re.S)
# Both spellings are in the tree: `BODY_CRATE kernel-rust` (ntoskrnl) and
# `BODY_CRATES win32k-kernel-rust win32k-rust` (win32k).
_BODY_CRATES = re.compile(r"BODY_CRATES?(?P<names>(?:[ \t]+[\w.-]+)+)")
_RUST_LIB_CALL = re.compile(r'\.rust_lib\(\s*"([\w.-]+)"')


def module_rust_crates(cmake_dir: Path, workspace: Path) -> list[Path]:
    """Every crate directory whose symbols end up in this module's binary.

    A module's Rust is NOT one directory. ntoskrnl.exe is assembled by `add_cargo_module` from
    a driver crate whose build.rs names ~20 more with `.rust_lib(..)`, and auditing against any
    one of them reports the other nineteen's exports as dropped. Each of these three spellings
    is the build's own declaration of what it links, so they are read rather than guessed at.
    """
    cml = cmake_dir / "CMakeLists.txt"
    if not cml.is_file():
        return []
    text = cml.read_text(encoding="utf-8", errors="replace")
    text = "\n".join(ln.split("#", 1)[0] for ln in text.splitlines())

    names: set[str] = set(_CMAKE_RUST_CRATE.findall(text))
    for m in _ADD_CARGO_MODULE.finditer(text):
        names.add(m.group(2))
        body = _BODY_CRATES.search(m.group("rest"))
        if body:
            names.update(body.group("names").split())

    # A cargo crate's build.rs is where the rest of the composition lives.
    for name in list(names):
        build_rs = workspace / name / "build.rs"
        if build_rs.is_file():
            names.update(
                _RUST_LIB_CALL.findall(build_rs.read_text(encoding="utf-8", errors="replace"))
            )

    return [workspace / n for n in sorted(names) if (workspace / n / "Cargo.toml").is_file()]


def crate_closure(crate_dir: Path) -> list[Path]:
    """Every .rs in `crate_dir` AND in its transitive `path = "…"` dependencies.

    ONE CRATE IS NOT THE BINARY. ntoskrnl.exe is assembled from ~20 crates — cc-rust, ke-rust,
    mm-rust, ob-rust, … — so auditing ntoskrnl.spec against `ntoskrnl-rust/src` alone called
    1,298 exports MISSING that are defined, exported and linked, just from a sibling crate:
    CcCopyRead is cc-rust's, KeInitializeDpc is ke-rust's. ntdll was the same story with 1,326.
    A dependency's `#[no_mangle]` symbols land in the final image exactly like the root
    crate's, so the closure — not the directory — is the set that can satisfy an export.
    """
    seen: set[Path] = set()
    pending = [Path(crate_dir).resolve()]
    srcs: list[Path] = []
    while pending:
        d = pending.pop()
        if d in seen:
            continue
        seen.add(d)
        toml = d / "Cargo.toml"
        if not toml.is_file():
            continue
        srcs.extend(sorted((d / "src").rglob("*.rs")) if (d / "src").is_dir() else [])
        for m in _CARGO_PATH_DEP.finditer(toml.read_text(encoding="utf-8", errors="replace")):
            dep = (d / m.group(1)).resolve()
            if dep not in seen:
                pending.append(dep)
    return srcs


def _repo_root_of(start: Path) -> Path:
    """Nearest ancestor holding a top-level CMakeLists.txt + sdk/, i.e. the checkout root.

    Resolved from the module's own path rather than the cwd: `parse_cmakelists` substitutes it
    for CMAKE_SOURCE_DIR, and a wrong root silently resolves every `${REACTOS_SOURCE_DIR}/…`
    source to a path that does not exist — which reads as "the build compiles nothing" and
    turns every export into MISSING.
    """
    for d in [start.resolve(), *start.resolve().parents]:
        if (d / "CMakeLists.txt").is_file() and (d / "sdk").is_dir():
            return d
    raise ProfileDeriveError(f"no repo root above {start} (looked for CMakeLists.txt + sdk/)")


def audit_exports_detailed(
    spec_path: Path,
    rust_srcs: list[Path],
    c_dir: Path | None = None,
    repo_root: Path | None = None,
    arch: str = "i386",
    target: str | None = None,
) -> list[Finding]:
    """Structured form of :func:`audit_exports`.

    Pass `c_dir` (the module's C directory) to separate the two reasons an export has no Rust
    definition. Without it every export still living in C reads as MISSING, which on a
    partially-flipped module is thousands of rows — ntdll alone contributed 1,326 — and a
    genuinely dropped export hides among them. With it, a name that a STILL-COMPILED .c
    defines is reported STILL_C, and MISSING means what it says.
    """
    spec = parse_spec(Path(spec_path))
    fns = scan_rust_fns(collect_rust_sources(list(rust_srcs)), arch=arch)

    still_c: set[str] = set()
    if c_dir is not None:
        root = Path(repo_root) if repo_root is not None else _repo_root_of(Path(c_dir))
        still_c = c_provided_names(compiled_tus(Path(c_dir), root, arch=arch, target=target))

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
            # A name the module's still-compiled C defines is COVERED — by C, which is the
            # expected state mid-flip. Checked against `entry.name` too: a spec line that
            # renames the symbol (`ldap_first_entry` -> `WLDAP32_ldap_first_entry`) is
            # implemented in the C under one spelling and exported under the other.
            if want in still_c or entry.name in still_c:
                findings.append(
                    Finding("STILL_C", label, expected, "a still-compiled .c defines it", "")
                )
                continue
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
    c_dir: Path | None = None
    arch, target = "i386", None
    crates: list[Path] = []
    module_dir: Path | None = None
    rest: list[str] = []
    it = iter(argv)
    for a in it:
        if a == "--c-dir":
            c_dir = Path(next(it))
        elif a == "--arch":
            arch = next(it)
        elif a == "--target":
            target = next(it)
        elif a == "--crate":
            crates.append(Path(next(it)))
        elif a == "--module":
            module_dir = Path(next(it))
        else:
            rest.append(a)
    argv = rest
    if module_dir is not None:
        # `--module` is the one-flag form: the module's C dir AND every crate it links.
        c_dir = c_dir or module_dir
        crates += module_rust_crates(module_dir, _repo_root_of(module_dir) / "sdk/lib/rust")
    if len(argv) < 1 or (len(argv) < 2 and not crates):
        print(
            "usage: python -m rustconv.export_audit [--module CMAKE_DIR] [--c-dir DIR] "
            "[--crate CRATE_DIR] [--arch A] [--target T] <file.spec> [rust-src-dir|file.rs]..."
        )
        return 2

    spec_path = Path(argv[0])
    srcs = [Path(a) for a in argv[1:]]
    for c in crates:
        srcs.extend(crate_closure(c))
    findings = audit_exports_detailed(spec_path, srcs, c_dir=c_dir, arch=arch, target=target)

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
