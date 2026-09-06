"""Symbols a module's exports can be satisfied by that live in ASSEMBLY.

`export_audit` asks "does the translated Rust DEFINE this export?", and `c_defs`
answers the second half — "…or does a still-compiled .c?".  There is a third
supplier, and until it was modelled every one of its symbols read as MISSING:

    ntdll.spec: 649 MISSING, of which 593 are Nt*/Zw*

Not one of those 593 is a defect.  ntdll's syscall stubs are *generated at
assembly time*: `ntoskrnl/ntdll.S` sets `#define SVC_(name, argcount) STUB_U
name, argcount` and then `#include <sysfuncs.h>`, a 296-line table of
`SVC_(AcceptConnectPort, 6)` rows.  `STUB_U` (sdk/include/asm/syscalls.inc)
emits a PUBLIC `Nt<name>` *and* a PUBLIC `Zw<name>` for each row — 592 symbols
that exist in no .c and no .rs anywhere.  `ntoskrnl/ex/zw.S` expands the same
table through `STUB_K`, which emits the `Zw` half only.

Swamping a real drop is precisely the failure the MISSING verdict exists to
prevent, so those rows have to be classified, not filtered by name: a rule like
"ignore Nt*/Zw*" would also swallow a genuinely dropped `NtQueryInformationFile`.
The table IS the build's declaration of what it generates, so read it.

Hand-written assembly is the same fact wearing different clothes — `_alldiv`,
`KiFastSystemCall` and `RtlCaptureContext` are also symbols the module links but
no Rust defines — so the literal `PUBLIC` / `.globl` labels are collected too.
Both land under one verdict; the finding text says which mechanism supplied it.

    from rustconv.asm_defs import module_asm_sources, asm_provided_names
    srcs = module_asm_sources(Path("dll/ntdll"), repo_root, workspace)
    names = asm_provided_names(srcs, repo_root, arch="i386")
"""

from __future__ import annotations

import re
from pathlib import Path

__all__ = [
    "AsmDefsError",
    "asm_provided_names",
    "cmake_texts",
    "module_asm_sources",
    "syscall_table_names",
]


class AsmDefsError(Exception):
    """A syscall table the assembly plainly expands could not be read."""


# ---------------------------------------------------------------------------
# Finding the assembly a module links
# ---------------------------------------------------------------------------

# CMake names asm through the two source-dir variables; a cargo crate's build.rs
# names it as a plain repo-relative string literal (ntdll-cargo: "ntoskrnl/ntdll.S").
_RE_ASM_PATH = re.compile(r'["\']([^"\'$\n]*?[\w./-]+\.[sS])["\']')
_RE_CMAKE_ASM = re.compile(r"\$\{(CMAKE_SOURCE_DIR|CMAKE_CURRENT_SOURCE_DIR)\}/([\w./-]+\.[sS])\b")
_RE_INCLUDE_CMAKE = re.compile(r"^\s*include\s*\(\s*([\w./-]+\.cmake)\s*\)", re.M)


def cmake_texts(cmake_dir: Path) -> list[tuple[Path, str]]:
    """The module's CMakeLists plus every `.cmake` it includes, comments stripped.

    freeldr declares both of its cargo links (`rosload.cmake`, `uefi.cmake`) behind
    `include()`, so a reader that stops at CMakeLists.txt sees a module with no Rust
    and no assembly at all.
    """
    out: list[tuple[Path, str]] = []
    seen: set[Path] = set()
    queue = [cmake_dir / "CMakeLists.txt"]
    while queue:
        p = queue.pop(0)
        if p in seen or not p.is_file():
            continue
        seen.add(p)
        text = "\n".join(ln.split("#", 1)[0] for ln in p.read_text(errors="replace").splitlines())
        out.append((p, text))
        queue += [p.parent / inc for inc in _RE_INCLUDE_CMAKE.findall(text)]
    return out


def module_asm_sources(cmake_dir: Path, repo_root: Path, crates: list[Path]) -> list[Path]:
    """Every assembly source this module's build declares, from CMake and build.rs.

    `crates` is what `export_audit.module_rust_crates` resolved: their `build.rs` is
    where a cargo-driven link names its asm, and it is the only place `ntdll.S` —
    which lives under `ntoskrnl/`, nowhere near `dll/ntdll/` — is mentioned.

    Paths built by `format!`/generator expressions do not resolve and are skipped;
    a file that is not on disk cannot be defining symbols either way.
    """
    found: set[Path] = set()

    for cml, text in cmake_texts(cmake_dir):
        for var, rel in _RE_CMAKE_ASM.findall(text):
            base = repo_root if var == "CMAKE_SOURCE_DIR" else cml.parent
            found.add(base / rel)

    for crate in crates:
        build_rs = crate / "build.rs"
        if not build_rs.is_file():
            continue
        for rel in _RE_ASM_PATH.findall(build_rs.read_text(errors="replace")):
            if "{" in rel:
                # A `format!` path with an interpolated segment is the per-arch spelling
                # (ntdll-cargo: "dll/ntdll/dispatch/{stem}/dispatch.S"). Take every arch's
                # copy — they define the same exported names, which is all this asks.
                found.update(repo_root.glob(re.sub(r"\{[^}]*\}", "*", rel)))
                continue
            found.add(repo_root / rel)

    return sorted(p for p in found if p.is_file())


# ---------------------------------------------------------------------------
# Reading the symbols out of it
# ---------------------------------------------------------------------------

_RE_SVC_DEFINE = re.compile(r"^\s*#\s*define\s+SVC_\s*\([^)]*\)\s*(STUB_[UK])\b", re.M)
_RE_SVC_ENTRY = re.compile(r"^\s*SVC_\s*\(\s*([A-Za-z_]\w*)\s*[,)]", re.M)
_RE_ASM_INCLUDE = re.compile(r"^\s*#\s*include\s*[<\"]([\w./-]+)[>\"]", re.M)
# MASM-flavour `PUBLIC _Foo@8` and GAS-flavour `.globl Foo` both appear in this tree.
_RE_ASM_PUBLIC = re.compile(r"^\s*(?:PUBLIC|\.globa?l)\s+([A-Za-z_$.?][\w$.?]*(?:@\d+)?)", re.M)

# The asm `-I` chain, in the order the build searches it. Resolution is explicit
# rather than a repo-wide glob for the basename: `w32ksvc32.h` also exists under
# modules/rostests/apitests/win32u/, and picking a test fixture's copy of a
# syscall table would silently answer with the wrong service list.
_ASM_INCLUDE_ROOTS = ("ntoskrnl/include", "win32ss", "sdk/include/asm", "sdk/include")

# Which symbols each stub macro publishes per table row (sdk/include/asm/syscalls.inc).
_STUB_PREFIXES = {"STUB_U": ("Nt", "Zw"), "STUB_K": ("Zw",)}


def _undecorate(sym: str, arch: str) -> str:
    """The .spec spelling of an i386 asm label: `_KiFastSystemCall@0` -> `KiFastSystemCall`.

    i386 asm carries the decoration the linker sees; the .spec carries the bare name and
    spec2def adds `_`/`@N` itself. amd64 and arm64 labels are already undecorated.
    """
    if arch != "i386":
        return sym
    sym = re.sub(r"@\d+$", "", sym)
    return sym[1:] if sym.startswith("_") else sym


def _resolve_include(inc: str, from_file: Path, repo_root: Path) -> Path | None:
    local = from_file.parent / inc
    if local.is_file():
        return local
    for root in _ASM_INCLUDE_ROOTS:
        cand = repo_root / root / inc
        if cand.is_file():
            return cand
    return None


def syscall_table_names(asm_path: Path, repo_root: Path) -> set[str]:
    """The Nt*/Zw* symbols `asm_path` generates by expanding a `SVC_` table.

    Empty unless the file actually defines `SVC_`. If it does and the table it then
    includes cannot be resolved, raise: a silently empty expansion would hand back
    "593 exports are MISSING" as a confident answer.
    """
    text = asm_path.read_text(errors="replace")
    stubs = set(_RE_SVC_DEFINE.findall(text))
    if not stubs:
        return set()

    names: set[str] = set()
    for inc in _RE_ASM_INCLUDE.findall(text):
        table = _resolve_include(inc, asm_path, repo_root)
        if table is None:
            continue
        entries = _RE_SVC_ENTRY.findall(table.read_text(errors="replace"))
        if not entries:
            continue
        for stub in stubs:
            for prefix in _STUB_PREFIXES.get(stub, ()):
                names.update(prefix + e for e in entries)

    if not names:
        raise AsmDefsError(
            f"{asm_path}: expands a SVC_ syscall table ({', '.join(sorted(stubs))}) but no "
            f"table was found on the include path ({', '.join(_ASM_INCLUDE_ROOTS)}). "
            f"Every generated Nt*/Zw* would be reported MISSING."
        )
    return names


def asm_provided_names(asm_srcs: list[Path], repo_root: Path, arch: str = "i386") -> dict[str, str]:
    """Symbol -> how the assembly supplies it, for every asm source given.

    The value is the human half of the finding: the generated stubs name their table,
    the literal labels name their file.
    """
    provided: dict[str, str] = {}
    for src in asm_srcs:
        for name in syscall_table_names(src, repo_root):
            provided.setdefault(name, f"a generated syscall stub ({src.name} SVC_ table)")
        text = src.read_text(errors="replace")
        try:
            rel = str(src.relative_to(repo_root))
        except ValueError:
            rel = str(src)
        for sym in _RE_ASM_PUBLIC.findall(text):
            provided.setdefault(_undecorate(sym, arch), f"assembly defines it ({rel})")
    return provided
