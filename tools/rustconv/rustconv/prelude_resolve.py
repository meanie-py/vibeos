"""Fill a new crate's prelude from what the rest of the tree already agrees on.

WHY THIS EXISTS
---------------
Every user-mode port meets the same unresolved names -- `_wcsicmp`, `UNICODE_NULL`,
`FILE_GENERIC_READ`, `FSCTL_LOCK_VOLUME`, `RtlStringCbCopyW` -- and every lane has
hand-declared them again in its own prelude: on 2026-10-05 `_wcsicmp` alone was declared
in advapi32, desk, dnsrslvr and more, `UNICODE_NULL` in a dozen crates. That hand work
is the "write the prelude" step a desk-sized port spent its afternoon on.

WHAT IT DOES
------------
Takes the names `cargo check` cannot find, finds every definition of each in the TRACKED
workspace (`git grep`, so the crate being built is never its own evidence), and copies
one into the prelude ONLY when every definition in the tree normalises to the same text:
a `pub const`, a `pub type`, or an extern `fn` declaration WITH its block's ABI (an
`extern "C"` copy of an `extern "system"` decl is an i386 link bug). Anything else is
reported, never guessed: definitions that disagree, names nothing defines, and names this
module's own C defines (a TU not translated yet -- an extern for it would shadow the real
body later). Re-run to a fixpoint: a copied constant can name another missing one.

The copies land between generated markers in src/prelude.rs, so a re-run replaces them.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

from . import variadic_shim
from .crate_integrate import _LITERAL_OR_COMMENT as _LITERAL_OR_COMMENT_RE
from .crate_integrate import extern_block_spans, split_decls

BEGIN = "// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ===="
END = "// ==== end port resolve ===="

# E0422 ("cannot find struct, variant or union type") is a struct LITERAL's type: registry.c's
# file-local `_ROOT_KEY` / `_HIVE_LIST_ENTRY` tables reach rustc only in that spelling.
_MISSING = re.compile(
    r"^cannot find (?:function|value|type|constant|macro|function, tuple struct or tuple variant"
    r"|struct, variant or union type)"
    r"(?: or [a-z ]+)? `([A-Za-z_][A-Za-z0-9_]*)` in this scope"
)


@dataclass(frozen=True)
class Item:
    kind: str  # const | type | fn
    text: str  # the item as written, normalised
    abi: str | None
    path: str


@dataclass
class Resolution:
    copied: dict[str, Item] = field(default_factory=dict)
    conflicting: dict[str, str] = field(default_factory=dict)  # name -> why not
    undefined: list[str] = field(default_factory=list)
    pending: list[str] = field(default_factory=list)


#: rustc found a VALUE where the code names a type: setuplib's `FILE_SYSTEM` typedef, after
#: tree consensus had copied bugcodes.h's `FILE_SYSTEM: ULONG = 0x22` for the missing name.
_VALUE_FOR_TYPE = re.compile(r"^expected type, found (?:constant|static|function) `(\w+)`")
_TYPE_WORDS = ("cannot find type ", "cannot find struct, variant or union type ")


def unresolved_names(crate: str, triple: str, workspace: Path, target_dir: Path) -> set[str]:
    return set(unresolved(crate, triple, workspace, target_dir))


def unresolved(crate: str, triple: str, workspace: Path, target_dir: Path) -> dict[str, set[str]]:
    """Each name rustc cannot find -> the namespaces it was wanted in ("type" / "value").

    The namespace matters: a missing TYPE is never satisfied by a constant that happens to
    share its name, which is what name-only resolution did to `FILE_SYSTEM`.
    """
    proc = subprocess.run(
        ["cargo", "check", "-p", crate, "--target", triple, "--message-format=json"],
        cwd=workspace,
        env=dict(os.environ, CARGO_TARGET_DIR=str(target_dir)),
        capture_output=True,
        text=True,
    )
    names: dict[str, set[str]] = {}
    suspects: set[str] = set()
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        text = str((msg.get("message") or {}).get("message") or "")
        m = _MISSING.match(text)
        if m:
            ns = "type" if text.startswith(_TYPE_WORDS) else "value"
            names.setdefault(m.group(1), set()).add(ns)
        m = _VALUE_FOR_TYPE.match(text)
        if m:
            names.setdefault(m.group(1), set()).add("type")
        for name in pattern_bindings(msg.get("message") or {}):
            names.setdefault(name, set()).add("value")
        suspects |= _shared_value_patterns(msg.get("message") or {})
    # Newer rustc words a catch-all arm among several as "matches SOME of the same values",
    # naming real constants and the binding alike. A real constant is DEFINED somewhere the
    # crate can see; the catch-all is not (input's TVN_ITEMEXPANDING, after TVN_SELCHANGED).
    if suspects:
        defined = _visible_value_names(workspace, crate)
        for name in suspects - defined:
            names.setdefault(name, set()).add("value")
    return names


def _shared_value_patterns(diag: dict) -> set[str]:
    """UPPER_CASE pattern names an `unreachable pattern` note labels "matches some of the
    same values"."""
    if not str(diag.get("message") or "").startswith("unreachable pattern"):
        return set()
    out: set[str] = set()
    for span in [
        *(diag.get("spans") or []),
        *(sp for c in diag.get("children") or [] for sp in c.get("spans") or []),
    ]:
        if "matches some of the same values" not in str(span.get("label") or ""):
            continue
        got = (span.get("text") or [{}])[0]
        line = str(got.get("text") or "")
        word = line[
            int(got.get("highlight_start") or 1) - 1 : int(got.get("highlight_end") or 1) - 1
        ]
        if re.fullmatch(r"[A-Z][A-Z0-9_]*", word.strip()):
            out.add(word.strip())
    return out


def _visible_value_names(workspace: Path, crate: str) -> set[str]:
    """`const`/`static` names in the crate, ntdll-sys's bindgen, and the shared crates."""
    try:
        from tools.source_walk import source_files
    except ModuleNotFoundError:
        import sys

        sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
        from tools.source_walk import source_files

    roots = [workspace / crate / "src", workspace / "ntdll-sys" / "src"]
    roots += [workspace / d / "src" for _ident, d, _paths in SHARED_CRATES]
    out: set[str] = set()
    for root in roots:
        for p in source_files(root, "*.rs") if root.is_dir() else []:
            text = p.read_text(errors="replace")
            out |= set(re.findall(r"\b(?:const|static(?:\s+mut)?)\s+([A-Z][A-Z0-9_]*)\s*:", text))
            if root == roots[0]:
                out |= _explicit_use_names(text)
    return out


def _explicit_use_names(text: str) -> set[str]:
    """UPPER_CASE names a crate imports BY NAME (`pub use ntkrnl_sys::driver_ddk::{IRP_MN_..}`).

    The kernel prelude brings wdm.h's IRP_MN_* in this way because the `ddk` glob does not
    carry them; without this every ndis/pciidex PnP switch read as a catch-all. A glob is
    deliberately NOT counted: a name a glob does not reach is exactly the catch-all.
    """
    out: set[str] = set()
    for m in re.finditer(r"\buse\s+[\w:]+::\{([^}]*)\}", text):
        out |= set(re.findall(r"\b([A-Z][A-Z0-9_]*)\b(?!\s*as\b)", m.group(1)))
        out |= set(re.findall(r"\bas\s+([A-Z][A-Z0-9_]*)\b", m.group(1)))
    for m in re.finditer(r"\buse\s+[\w:]+::([A-Z][A-Z0-9_]*)\s*;", text):
        out.add(m.group(1))
    for m in re.finditer(r"\buse\s+[\w:]+\s+as\s+([A-Z][A-Z0-9_]*)\s*;", text):
        out.add(m.group(1))
    return out


_UNBOUND = re.compile(r"^variable `([A-Za-z_]\w*)` is not bound in all patterns")


def pattern_bindings(diag: dict) -> set[str]:
    """Names a `match` arm BINDS where the C `case` named a constant.

    A constant missing from scope is never "cannot find" in a pattern: it silently becomes a
    fresh binding that matches everything (the shape that shipped BUGCHECK 0x1E). rustc only
    objects indirectly -- E0408 when an or-pattern's halves bind different names (atapi's
    `IOCTL_SCSI_PASS_THROUGH | IOCTL_SCSI_PASS_THROUGH_DIRECT`), or an `unreachable pattern`
    warning whose culprit "matches any value". Both are read as wanted VALUES; one that starts
    lowercase is a real Rust binding and is left alone.
    """
    text = str(diag.get("message") or "")
    out: set[str] = set()
    m = _UNBOUND.match(text)
    if m:
        out.add(m.group(1))
    if text.startswith("unreachable pattern"):
        for span in [
            *(diag.get("spans") or []),
            *(sp for c in diag.get("children") or [] for sp in c.get("spans") or []),
        ]:
            if "matches any value" in str(span.get("label") or ""):
                got = (span.get("text") or [{}])[0]
                line = str(got.get("text") or "")
                word = line[
                    int(got.get("highlight_start") or 1) - 1 : int(got.get("highlight_end") or 1)
                    - 1
                ]
                if re.fullmatch(r"[A-Z]\w*", word.strip()):
                    out.add(word.strip())
    return {n for n in out if n[:1].isupper()}


def _norm(text: str) -> str:
    text = re.sub(r"//[^\n]*", "", text)
    text = re.sub(r"#\[[^\]]*\]", "", text)
    text = re.sub(r"\bpub(\([^)]*\))?\s+", "", text)
    # `crate::island::LPCWSTR` names the DEFINING crate's module; copied into another
    # crate it resolves to nothing (notepad got shell32's IDC_WAIT that way). Bare names
    # resolve through the receiving prelude's island glob.
    text = re.sub(r"\bcrate::(?:\w+::)+", "", text)
    return " ".join(text.split())


_CONST = r"^[ \t]*pub[ \t]+const[ \t]+{n}[ \t]*:[^=;]+=[^;]+;"
_TYPE = r"^[ \t]*pub[ \t]+type[ \t]+{n}[ \t]*=[^;]+;"
_CONST_ANY = re.compile(_CONST.replace("{n}", r"([A-Za-z_]\w*)"), re.M)
_TYPE_ANY = re.compile(_TYPE.replace("{n}", r"([A-Za-z_]\w*)"), re.M)


def is_test_source(rel: str) -> bool:
    """A test harness file (`tests/`, a `*-difftest` crate): never tree consensus."""
    return "/tests/" in f"/{rel}" or bool(re.search(r"(?:^|/)[\w-]*-difftest/", rel))


def crate_mode(crate_dir: Path) -> str | None:
    """ "kernel" / "user" from what the crate builds against; None for a mode-neutral crate."""
    if crate_dir.name == "ntkrnl-sys":
        return "kernel"
    if crate_dir.name == "ntdll-sys":
        return "user"
    try:
        manifest = (crate_dir / "Cargo.toml").read_text(errors="replace")
    except OSError:
        return None
    if re.search(r"^ntkrnl-sys\s*=", manifest, re.M):
        return "kernel"
    if re.search(r"^ntdll-sys\s*=", manifest, re.M):
        return "user"
    return None


def workspace_items(names: set[str], workspace: Path, exclude: Path) -> dict[str, list[Item]]:
    """Every tracked definition of each name: consts, type aliases, extern fn decls.

    Never across the kernel/user line: gdi32_vista and opengl32 (user-mode DLLs) were handed
    `_InterlockedExchange` from disk-rust's DDK shims, an extern no user-mode image can link
    (i386 LINK_FAIL). A donor built against the other -sys crate is skipped; mode-neutral
    shared crates (win32macros-rust, strsafe-rust) donate to both.
    """
    if not names:
        return {}
    mine = crate_mode(exclude)
    modes: dict[str, str | None] = {}
    pattern = "|".join(sorted(names))
    proc = subprocess.run(
        ["git", "grep", "-l", "-w", "-E", pattern, "--", "*.rs"],
        cwd=workspace,
        capture_output=True,
        text=True,
    )
    found: dict[str, list[Item]] = {n: [] for n in names}
    aliases: dict[Path, dict[str, str]] = {}
    for rel in proc.stdout.split():
        path = workspace / rel
        if exclude in path.parents or "/target/" in rel:
            continue
        # Test harnesses are not the tree's declarations: diskpart got `_wfopen` and `fgetws`
        # from msvcrt-difftest/tests/file.rs, typed with that file's private `FILEP`.
        if is_test_source(rel):
            continue
        top = rel.split("/", 1)[0]
        if top not in modes:
            modes[top] = crate_mode(workspace / top)
        if mine is not None and modes[top] is not None and modes[top] != mine:
            continue
        try:
            src = path.read_text(errors="replace")
        except OSError:
            continue
        # One identifier scan per file, not one regex per name: with 544 unresolved names
        # over the tree that loop was a 40-minute CPU stall inside `port replay`.
        present = names & set(re.findall(r"[A-Za-z_]\w*", src))
        # One pass per KIND, not one per name: a bindgen island is megabytes and holds
        # hundreds of the wanted names, so the per-name regex was names x file size. comctl32
        # (825 unresolved names) spent most of a 30-minute resolve pass here.
        for kind, pat in (("const", _CONST_ANY), ("type", _TYPE_ANY)):
            hits: dict[str, list[Item]] = {}
            for m in pat.finditer(src):
                if m.group(1) in present:
                    hits.setdefault(m.group(1), []).append(Item(kind, _norm(m.group(0)), None, rel))
            for n, items in hits.items():  # per name: consts first, then types, as before
                found[n].extend(items)
        for _start, _end, body, abi in extern_block_spans(src):
            for decl in split_decls(body):
                m = re.search(r"\bfn\s+([A-Za-z_]\w*)\s*\(", decl)
                if m and m.group(1) in present:
                    decl = _inline_core_aliases(decl, workspace / top, aliases)
                    found[m.group(1)].append(Item("fn", _norm(decl), abi, rel))
                    continue
                # An extern DATA symbol (uuid's `static GUID_NULL: GUID;`).
                m = re.search(r"\bstatic\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:[^;]+;", decl)
                if m and m.group(1) in present:
                    text = re.sub(r"^.*?\bstatic\b", "static", _norm(decl))
                    found[m.group(1)].append(Item("static", text, abi, rel))
        for name, decl, abi in exported_definitions(src, present):
            decl = _inline_core_aliases(decl, workspace / top, aliases)
            found[name].append(Item("fn", _norm(decl), abi, rel))
    return found


# Spelled entirely in `core` terms, so an alias of this shape means the same in any crate.
_CORE_ALIAS_RHS = re.compile(
    r"(?:\s|[(),<>*\-]|->|::|Option|unsafe|extern|\"(?:C|system)\"|fn|const|mut|core|ffi|"
    r"c_(?:void|char|schar|uchar|short|ushort|int|uint|long|ulong|longlong|ulonglong)|"
    r"[iu](?:8|16|32|64|size))*"
)


_ALIAS_DEF = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?type ([A-Za-z_]\w*)\s*=\s*([^;]+);", re.M)


def _inline_core_aliases(decl: str, crate_dir: Path, cache: dict[Path, dict[str, str]]) -> str:
    """A copied declaration names its source crate's private aliases: ucrt-rust's
    `qsort(.., comp: Compare)` reached cmd and net with `Compare` undefined, because types
    are never copied. Where the alias is spelled purely in `core` terms (`Option<unsafe
    extern "C" fn(*const c_void, *const c_void) -> c_int>`) it means the same everywhere, so
    it is substituted into the declaration; anything else is left for the island, as before.
    """
    if crate_dir not in cache:
        found: dict[str, str] = {}
        from tools.source_walk import source_files  # repo-root package: imported at use

        for rs in source_files(crate_dir / "src", "*.rs"):
            try:
                text = rs.read_text(errors="replace")
            except OSError:
                continue
            for m in _ALIAS_DEF.finditer(text):
                rhs = " ".join(m.group(2).split())
                if _CORE_ALIAS_RHS.fullmatch(rhs) and found.get(m.group(1), rhs) == rhs:
                    found[m.group(1)] = rhs
                else:
                    found[m.group(1)] = ""  # disagreeing or not core-only: never inline
        cache[crate_dir] = {k: v for k, v in found.items() if v}
    table = cache[crate_dir]
    if not table:
        return decl
    return re.sub(
        r"(:\s*)([A-Za-z_]\w*)\b(?!\s*::)",
        lambda m: m.group(1) + table.get(m.group(2), m.group(2)),
        decl,
    )


_EXPORTED_FN = re.compile(
    r"#\[no_mangle\]\s*(?:#\[[^\]]*\]\s*)*pub\s+(?:unsafe\s+)?extern\s+\"(\w+)\"\s+fn\s+"
    r"([A-Za-z_]\w*)\s*\("
)


def exported_definitions(src: str, names: set[str]) -> list[tuple[str, str, str]]:
    """``(name, extern decl, abi)`` for each ``#[no_mangle]`` Rust DEFINITION of a wanted name.

    A module that links a library already ported to Rust calls functions the tree only
    DEFINES: usetup imports setuplib.dll, whose 52 exports are bodies in setuplib-rust, and
    resolve found "no definition" for every one of them (it read only extern blocks). The
    exported definition is the symbol itself, so its signature and ABI are the declaration.
    """
    out = []
    for m in _EXPORTED_FN.finditer(src):
        abi, name = m.group(1), m.group(2)
        if name not in names:
            continue
        depth, i = 1, m.end()
        while i < len(src) and depth:
            depth += {"(": 1, ")": -1}.get(src[i], 0)
            i += 1
        params = src[m.end() : i - 1]
        ret = re.match(r"\s*(->\s*[^{]+?)\s*(?:where\b[^{]*)?\{", src[i:])
        if ret is None and not re.match(r"\s*\{", src[i:]):
            continue
        # A binding pattern (`mut x: T`) is not allowed in a foreign declaration.
        params = re.sub(r"(^|,)(\s*)mut\s+(?=[A-Za-z_]\w*\s*:)", r"\1\2", params)
        tail = f" {ret.group(1).strip()}" if ret else ""
        # Quoted, as extern_block_spans gives it: render writes `extern {abi} {`, and a bare
        # `C` there is a parse error that hid every name in diskpart (4,867 errors).
        out.append((name, f"pub fn {name}({params.strip()}){tail};", f'"{abi}"'))
    return out


_INT = re.compile(
    r"^\(?\s*(0[xX][0-9A-Fa-f_]+|[0-9][0-9_]*)(?:_?[iu](?:8|16|32|64|size))?\s*\)?"
    r"(?:\s+as\s+[A-Za-z_][\w:]*)?$"
)


def const_value(item: Item) -> int | None:
    """The integer a `const NAME: T = <literal>[ as T];` declares, else None."""
    m = re.search(r"=\s*(.+?)\s*;\s*$", item.text)
    if not m:
        return None
    lit = _INT.match(m.group(1))
    if not lit:
        return None
    digits = lit.group(1).replace("_", "")
    return int(digits, 16) if digits[:2] in ("0x", "0X") else int(digits)


def _abi_class(abi: str | None) -> str:
    a = (abi or '"C"').strip('"')
    return "system" if a in ("system", "stdcall") else a


def _placeholder_decl(item: Item) -> bool:
    """`fn NtClose(_: usize);` -- a link-only stub some crates keep. Not evidence of a signature."""
    params = re.search(r"\((.*)\)", item.text)
    if params is None:
        return False
    parts = [p for p in params.group(1).split(",") if p.strip()]
    # A function with NO parameters is not a placeholder: `all([])` called usetup's
    # CreatePartitionList() one, and every zero-argument extern was discarded as "mixed".
    return bool(parts) and all(re.fullmatch(r"\s*_\s*:\s*usize\s*", p) for p in parts)


_SIG_SKIP = frozenset(
    {
        "pub",
        "fn",
        "unsafe",
        "extern",
        "mut",
        "const",
        "Option",
        "dyn",
        "impl",
        "self",
        "c_void",
        "c_char",
        "c_schar",
        "c_uchar",
        "c_short",
        "c_ushort",
        "c_int",
        "c_uint",
        "c_long",
        "c_ulong",
        "c_longlong",
        "c_ulonglong",
        "c_float",
        "c_double",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
        "f32",
        "f64",
        "bool",
        "char",
        "str",
    }
)


def foreign_types(text: str, visible: set[str]) -> set[str]:
    """Type names an extern fn's signature uses that `visible` does not have.

    A copy is the OTHER crate's text: ucrt-rust declares `qsort(.., comp: Compare)` with its
    own private `Compare`, and regedit got that line verbatim (E0412). Parameter names,
    path segments (`core::ffi::c_void`) and primitives are not candidates."""
    head = text.split("{", 1)[0]
    head = re.sub(r'"[^"]*"', "", head)
    head = re.sub(r"\bfn\s+[A-Za-z_]\w*", "fn", head, count=1)
    out = set()
    for m in re.finditer(r"(?<![\w:])([A-Za-z_]\w*)\b(?!\s*::)(?!\s*:(?!:))", head):
        word = m.group(1)
        if word not in _SIG_SKIP and word not in visible:
            out.add(word)
    return out


def consensus(name: str, items: list[Item]) -> tuple[Item | None, str]:
    """The definition the tree agrees on, or (None, why not).

    A constant needs every literal definition to agree on the VALUE (`0x80`, `0x0000_0080`
    and `128u32 as T` are one value); the most common declaration of it is copied. An extern
    fn needs every real declaration to agree on the ABI class -- `system`/`stdcall` vs `C` is
    an i386 link bug, not a spelling -- and the most common declaration is copied. Types are
    never copied: a missing type belongs in the bindgen island's allowlist.
    """
    consts = [i for i in items if i.kind == "const"]
    fns = [i for i in items if i.kind == "fn" and not _placeholder_decl(i)]
    statics = [i for i in items if i.kind == "static"]
    if statics and not fns:
        # An extern DATA symbol: the island's declaration is the header's own transcript. It
        # outranks a crate's `const GUID_NULL: GUID = ..` copy, which is not the symbol.
        voters = [i for i in statics if _is_bindgen(i.path)] or statics
        types = {re.sub(r"\s+", "", i.text.split(":", 1)[1]) for i in voters}
        if len(types) != 1:
            return None, f"extern static type disagreement ({len(types)} types)"
        return _most_common(voters), ""
    if consts and not fns:
        literal = [i for i in consts if const_value(i) is not None]
        values = {const_value(i) for i in literal}
        if len(values) != 1:
            return None, (f"{len(values)} different values" if values else "no literal definition")
        # An expression definition (`CTL_CODE(...)`) is not evaluated here; it neither
        # vetoes nor is copied -- the literals that agree are the evidence.
        return _most_common(literal), ""
    if fns and not consts:
        # A bindgen island is the header's own transcript; a hand-written `extern` is a copy
        # someone typed. But only an i386 island KNOWS a calling convention: x86_64 and arm64
        # have one, so bindgen writes every WINAPI function there as `extern "C"`. So an i386
        # island declaration outvotes everything, and a 64-bit one never votes on the ABI.
        # Before this, one crate's `extern "C" { fn CloseHandle(..) }` (a stdcall import --
        # the i386 link bug) vetoed CloseHandle/GetLastError for every new module.
        island32 = [i for i in fns if _is_bindgen(i.path) and "i386" in i.path]
        voters = island32 or [i for i in fns if not _is_bindgen(i.path)]
        if voters:
            fns = voters
        classes = {_abi_class(i.abi) for i in fns}
        if len(classes) != 1:
            return None, f"ABI disagreement ({', '.join(sorted(classes))})"
        # Same name, different functions: diskpart's `SelectPartition(argc, argv)` command
        # handler and setuplib's `SelectPartition(List, DiskNumber, PartitionNumber)` live in
        # different binaries, agree on the ABI, and the shorter text won -- usetup called
        # setuplib's with diskpart's signature. Only the module's own prototype can choose.
        arities = {_arity(i.text) for i in fns} - {None}
        if len(arities) > 1:
            return None, f"signature disagreement ({len(arities)} arities)"
        return _most_common(fns), ""
    if any(i.kind == "type" for i in items):
        return None, "a type: add its header to the island allowlist"
    return None, "mixed kinds"


def _is_bindgen(path: str) -> bool:
    name = path.replace("\\", "/").rsplit("/", 1)[-1]
    # shell32's islands are `island_<arch>.rs`; every other generator writes bindgen_/types_.
    return name.startswith(("bindgen_", "hal_bindgen_")) or bool(
        re.match(r"(?:types|island)_\w+\.rs$", name)
    )


def _arity(decl: str) -> int | None:
    """Parameter count of a `fn NAME(...)` declaration (`...` counts), None if unreadable."""
    m = re.search(r"\bfn\s+\w+\s*\(", decl)
    if m is None:
        return None
    depth, i, n = 1, m.end(), 0
    seen = False
    while i < len(decl) and depth:
        c = decl[i]
        if c in "([<":
            depth += 1
        elif c in ")]>" and not (c == ">" and decl[i - 1] == "-"):
            depth -= 1
        elif c == "," and depth == 1:
            n += 1
            seen = False
            i += 1
            continue
        if depth and not c.isspace():
            seen = True
        i += 1
    if depth:
        return None
    return n + (1 if seen else 0)


def _most_common(items: list[Item]) -> Item:
    counts: dict[tuple[str, str | None], int] = {}
    for i in items:
        counts[(i.text, i.abi)] = counts.get((i.text, i.abi), 0) + 1
    best = max(counts.items(), key=lambda kv: (kv[1], -len(kv[0][0])))[0]
    return next(i for i in items if (i.text, i.abi) == best)


def island_enum_aliases(
    names: set[str], src: Path, substrate: Path | None = None
) -> dict[str, Item]:
    """`FreeLdr` -> `pub const FreeLdr: _BOOT_STORE_TYPE = _BOOT_STORE_TYPE_FreeLdr;`.

    bindgen prefixes every enumerator with its enum's name, while C code -- and so the
    sealed translation -- spells it bare. netid, samlib and others carried these aliases by
    hand. The island is the authority, so this is exact: a name gets an alias only when
    every arch's island has exactly one `<E>_<name>: <E>` constant, and the same E.

    `substrate`: the src dir of the -sys crate the island globs (a kernel island's
    `use ntkrnl_sys::ddk::*`). Its bindgen is part of each arch's island: atapi's
    `PowerSystemWorking`, `BusRelations` and `MmCached` are wdm.h enums, defined there and
    nowhere in the driver's own types, and fell through to "no literal definition".
    """
    islands = sorted(src.glob("types_*.rs"))
    if not islands or not names:
        return {}
    texts = [p.read_text(errors="replace") for p in islands]
    if substrate is not None:
        for k, p in enumerate(islands):
            arch = p.stem.removeprefix("types_")
            for extra in (substrate / f"bindgen_{arch}.rs", substrate / "bindgen_common.rs"):
                if extra.is_file():
                    texts[k] += "\n" + extra.read_text(errors="replace")
    out: dict[str, Item] = {}
    for name in sorted(names):
        pat = re.compile(
            rf"^pub const (_?[A-Za-z][A-Za-z0-9_]*)_{re.escape(name)}\s*:\s*([A-Za-z_]\w*)\s*=",
            re.M,
        )
        enums: set[tuple[str, str]] = set()
        ok = True
        for text in texts:
            # ...or an anonymous enum nested in a struct: bindgen prefixes the STRUCT's name
            # and types it `<struct>__bindgen_ty_N` (winhlp32's `enum {hlp_link_link, ..}
            # cookie;` inside tagHlpFileLink).
            hits = {
                (m.group(1), m.group(2))
                for m in pat.finditer(text)
                if m.group(1) == m.group(2)
                or re.fullmatch(rf"{re.escape(m.group(1))}__bindgen_ty_\d+", m.group(2))
            }
            if len(hits) != 1:
                ok = False
                break
            enums |= hits
        if ok and len(enums) == 1:
            prefix, ty = enums.pop()
            out[name] = Item("const", f"const {name}: {ty} = {prefix}_{name};", None, "island enum")
    return out


_WIN32MACROS_USE = re.compile(r"^pub use win32macros_rust::\{[^}]*\};$", re.M)


def refresh_shared_macros(prelude: Path, macro_src: Path) -> bool:
    """Re-list win32macros-rust's macros in a prelude's by-name re-export.

    Scaffold writes the list once; a macro added to the shared crate later (setuplib's port
    added RTL_NUMBER_OF_FIELD and InitializeObjectAttributes) would otherwise stay invisible
    to every crate scaffolded before it.
    """
    text = prelude.read_text()
    m = _WIN32MACROS_USE.search(text)
    # Its folded CONSTANTS too (INVALID_HANDLE_VALUE, UNICODE_NULL): setuplib called
    # INVALID_HANDLE_VALUE five times and the tree has no literal definition of it. A name the
    # prelude defines itself is left out -- a named import beside a definition is E0255.
    own = set(
        re.findall(r"^\s*(?:pub\s+)?(?:const|static|fn|type)\s+([A-Za-z_]\w*)", text, re.M)
    ) | set(re.findall(r"macro_rules!\s+([A-Za-z_]\w*)", text))
    names = sorted(
        {
            n
            for f in (macro_src.glob("*.rs") if macro_src.is_dir() else [])
            for n in re.findall(
                r"macro_rules!\s+([A-Za-z_]\w*)|^pub const ([A-Za-z_]\w*)", f.read_text(), re.M
            )
            for n in n
            if n
        }
        - own
    )
    if not m or not names:
        return False
    line = f"pub use win32macros_rust::{{{', '.join(names)}}};"
    if m.group(0) == line:
        return False
    prelude.write_text(text[: m.start()] + line + text[m.end() :])
    return True


#: Shared crates a usermode prelude re-exports BY NAME (a glob would also pull their type
#: aliases next to the island's: E0659). Each entry: Rust ident, crate dir, and per module
#: path ("" = crate root) the pattern whose groups name what it provides.
SHARED_CRATES: tuple[tuple[str, str, tuple[tuple[str, str], ...]], ...] = (
    (
        "win32macros_rust",
        "win32macros-rust",
        (
            (
                "",
                r"macro_rules!\s+([A-Za-z_]\w*)|^pub const ([A-Za-z_]\w*)"
                r"|^pub (?:unsafe )?fn ([A-Za-z_]\w*)",
            ),
        ),
    ),
    (
        "strsafe_rust",
        "strsafe-rust",
        (
            (
                "",
                r"macro_rules!\s+([A-Za-z_]\w*)"
                r'|^pub unsafe (?:extern "\w+" )?fn (String[A-Za-z]+)\b',
            ),
            ("ntstrsafe", r"^pub unsafe fn (RtlString[A-Za-z]+)\b"),
        ),
    ),
    (
        "conutils_h",
        "conutils-h",
        (
            (
                "",
                r"^\s*pub (?:unsafe )?fn ([A-Za-z_]\w*)|^\s*pub static mut ([A-Za-z_]\w*)"
                r"|^pub const ([A-Za-z_]\w*)|^pub type ([A-Za-z_]\w*)|^pub struct ([A-Za-z_]\w*)",
            ),
        ),
    ),
    (
        "winelist_rust",
        "winelist-rust",
        (("", r"^pub unsafe fn (list_\w+)\b"),),
    ),
)

#: Shared crates a prelude takes only when the crate CALLS one of their names (or already
#: depends on them). wine/list.h's helpers operate on winelist-rust's `struct list`, so a
#: crate takes them together with that type: its island blocklists `list` and re-exports
#: `winelist_rust::*` (profile_derive emits both when wine/list.h is in the include closure).
#: Without the entry every wine-derived crate re-ported the header -- six copies -- and rpcss
#: got cabinet-rust's crate-internal #[no_mangle] copies as externs, which do not link.
OPT_IN_SHARED = frozenset({"winelist_rust"})


def _crate_calls(crate_dir: Path, names: set[str]) -> set[str]:
    """Which of `names` the crate's modules call (the prelude itself excluded)."""
    if not names:
        return set()
    call = re.compile(rf"(?<![\w:])({'|'.join(map(re.escape, sorted(names)))})\s*\(")
    out: set[str] = set()
    for rs in (crate_dir / "src").rglob("*.rs"):
        if rs.name == "prelude.rs":
            continue
        out |= set(call.findall(rs.read_text(errors="replace")))
    return out


def shared_provides(workspace: Path) -> dict[tuple[str, str], set[str]]:
    """`{(crate ident, module path): names}` the shared crates export, read from their source."""
    out: dict[tuple[str, str], set[str]] = {}
    for ident, dirname, paths in SHARED_CRATES:
        src = workspace / dirname / "src"
        files = sorted(src.glob("*.rs")) if src.is_dir() else []
        for module, pattern in paths:
            names = out.setdefault((ident, module), set())
            for f in files:
                if module and f.stem != module:
                    continue
                for groups in re.findall(pattern, f.read_text(), re.M):
                    names.update(
                        g for g in (groups if isinstance(groups, tuple) else (groups,)) if g
                    )
    return out


def refresh_shared_crates(crate_dir: Path, workspace: Path) -> list[str]:
    """Write one `pub use <crate>[::module]::{...};` per shared path into the prelude.

    Re-lists what each shared crate provides NOW (a crate scaffolded before a macro was added
    would never see it), inserts a missing line, and adds the Cargo dependency it needs.
    A name the prelude defines itself is left out: a named import beside a definition is
    E0255. Returns the lines changed.
    """
    prelude = crate_dir / "src" / "prelude.rs"
    text = prelude.read_text()
    own = set(
        re.findall(r"^\s*(?:pub\s+)?(?:const|static|fn|type)\s+([A-Za-z_]\w*)", text, re.M)
    ) | set(re.findall(r"macro_rules!\s+([A-Za-z_]\w*)", text))
    # ...and a name the prelude already IMPORTS from somewhere else (a sibling module's own
    # `heap_alloc`, re-exported by the cross-TU block): a second named import is E0252.
    shared_idents = {ident for ident, _d, _p in SHARED_CRATES}
    for m in re.finditer(r"^\s*pub use ([\w:]+?)::(?:\{([^}]*)\}|(\w+));", text, re.M):
        if m.group(1).split("::")[0] in shared_idents:
            continue
        own.update(n.strip().split(" as ")[-1] for n in (m.group(2) or m.group(3)).split(","))
    own.discard("")
    changed: list[str] = []
    manifest = (crate_dir / "Cargo.toml").read_text()
    for (ident, module), names in shared_provides(workspace).items():
        if (
            ident in OPT_IN_SHARED
            and not re.search(rf"^{re.escape(ident.replace('_', '-'))}\s*=", manifest, re.M)
            and not _crate_calls(crate_dir, names - own)
        ):
            continue
        keep = sorted(names - own)
        path = f"{ident}::{module}" if module else ident
        line_re = re.compile(rf"^pub use {re.escape(path)}::\{{[^}}]*\}};$", re.M)
        m = line_re.search(text)
        if not keep:
            continue
        line = f"pub use {path}::{{{', '.join(keep)}}};"
        if m:
            if m.group(0) == line:
                continue
            text = text[: m.start()] + line + text[m.end() :]
        else:
            text = text.rstrip("\n") + "\n" + line + "\n"
        changed.append(path)
        _ensure_dependency(crate_dir / "Cargo.toml", ident.replace("_", "-"))
    if changed:
        prelude.write_text(text)
    return changed


def _ensure_dependency(manifest: Path, dep: str) -> None:
    text = manifest.read_text()
    if re.search(rf"^{re.escape(dep)}\s*=", text, re.M):
        return
    m = re.search(r"^\[dependencies\]\s*\n", text, re.M)
    if m is None:
        return
    manifest.write_text(text[: m.end()] + f'{dep} = {{ path = "../{dep}" }}\n' + text[m.end() :])


SIBLING_BEGIN = "// ==== port resolve: cross-TU definitions (generated) ===="
SIBLING_END = "// ==== end cross-TU definitions ===="
_DEF = re.compile(
    r"^(?P<pub>pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?"
    # Types too: a local-type alias resolve hoists into the prelude (cmd's LPDIRSWITCHFLAGS =
    # *mut _DirSwitchesFlags) names a struct one module defines, and was "defined nowhere".
    r"(?P<kind>fn|static(?:\s+mut)?|const|struct|union|enum|type)\s+(?P<name>[A-Za-z_]\w*)\b",
    re.M,
)


def sibling_definitions(names: set[str], modules_dir: Path) -> dict[str, list[tuple[str, bool]]]:
    """`{name: [(module, is_pub), ...]}` for each name a translated module DEFINES at top level.

    C has one namespace across TUs: `INF_GetData` is declared in infsupp.h and defined in
    infsupp.c, and every other TU just calls it. The translation keeps the call and nothing
    imports the definition -- setuplib had ~100 such errors once every TU was translated.
    Only column-0 items count (an `extern { fn X(); }` declaration is indented).
    """
    out: dict[str, list[tuple[str, bool]]] = {}
    for rs in sorted(modules_dir.glob("*.rs")):
        text = rs.read_text(errors="replace")
        for m in _DEF.finditer(text):
            if m.group("name") in names:
                out.setdefault(m.group("name"), []).append((rs.stem, bool(m.group("pub"))))
    return out


def defined_outside_siblings(prelude: Path) -> set[str]:
    """Names the prelude itself defines at top level, outside the cross-TU import block.

    A sibling import of such a name collides with it (E0255). It happens when resolve runs
    before a stage that defines the name in the prelude: e1000's replay resolved nic.h's
    inline helpers to the `port headers` module, then `port macros` defined the same names.
    """
    text = prelude.read_text()
    m = re.search(re.escape(SIBLING_BEGIN) + r".*?" + re.escape(SIBLING_END), text, re.S)
    if m:
        text = text[: m.start()] + text[m.end() :]
    return {d.group("name") for d in _DEF.finditer(text)}


def promote_pub(path: Path, name: str) -> bool:
    """Make a top-level definition of `name` in `path` `pub`. True if it changed."""
    text = path.read_text()
    pat = re.compile(
        r"^(?!pub)((?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?"
        r"(?:fn|static(?:\s+mut)?|const|struct|union|enum|type)\s+" + re.escape(name) + r"\b)",
        re.M,
    )
    new, n = pat.subn(r"pub \1", text, count=1)
    if n:
        path.write_text(new)
    return bool(n)


def _raw_ident(seg: str) -> str:
    from .profile_derive import _RUST_KEYWORDS

    return f"r#{seg}" if seg in _RUST_KEYWORDS else seg


def render_siblings(uses: dict[str, str], stem: str) -> str:
    lines = [SIBLING_BEGIN]
    for name in sorted(uses):
        # `enum.c` -> module `r#enum`: a keyword path segment must be a raw identifier.
        lines.append(f"pub use crate::{stem}::{_raw_ident(uses[name])}::{name};")
    lines.append(SIBLING_END)
    return "\n".join(lines) + "\n"


def existing_siblings(prelude: Path) -> dict[str, str]:
    m = re.search(
        re.escape(SIBLING_BEGIN) + r"(.*?)" + re.escape(SIBLING_END), prelude.read_text(), re.S
    )
    if not m:
        return {}
    return {
        name: mod
        for mod, name in re.findall(r"pub use crate::\w+::(?:r#)?(\w+)::(\w+);", m.group(1))
    }


def write_siblings(prelude: Path, block: str) -> bool:
    text = prelude.read_text()
    m = re.search(re.escape(SIBLING_BEGIN) + r".*?" + re.escape(SIBLING_END) + r"\n?", text, re.S)
    new = text[: m.start()] + block + text[m.end() :] if m else text.rstrip("\n") + "\n\n" + block
    if new == text:
        return False
    prelude.write_text(new)
    return True


SUBSTRATE_BEGIN = "// ==== port resolve: substrate helpers (generated) ===="
SUBSTRATE_END = "// ==== end substrate helpers ===="
#: Rust `target_arch` -> the island file suffix.
ARCHES = {"x86": "i386", "x86_64": "amd64", "aarch64": "arm64"}
_ALL_ARCHES = frozenset(ARCHES)
#: Generated layout/assert files are not helpers anyone imports by name.
# Not header ports: generated bindings and layout checks -- and `base`/`rtl`, ntoskrnl's own
# Rust-flavoured core (its own LIST_ENTRY/SLIST_HEADER, `bool` returns): atapi's
# `IsListEmpty(..) != 0` and `InitializeSListHead(&ddk_header)` stopped compiling on them.
_NOT_HELPERS = re.compile(r"^(?:hal_)?bindgen_|^size_check|^offset_check|^base$|^rtl$")
_SUBSTRATE_TOKEN = re.compile(
    r"(?P<cfg>#!?\[cfg\()"
    # Matched on MASKED text, where `extern "system" {` reads `extern          {`.
    r"|(?P<extern>\b(?:unsafe\s+)?extern\s*\{)"
    r"|(?P<export>#\[macro_export\])"
    r"|\bmacro_rules!\s*(?P<macro>[A-Za-z_]\w*)"
    r"|\bpub\s+(?:const\s+)?(?:unsafe\s+)?(?:extern\s+)?fn\s+(?P<fn>[A-Za-z_]\w*)"
    r"|(?P<open>\{)|(?P<close>\})|(?P<semi>;)"
)


def _split_top(text: str) -> list[str]:
    parts, depth, cur = [], 0, []
    for ch in text:
        depth += ch == "("
        depth -= ch == ")"
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    if "".join(cur).strip():
        parts.append("".join(cur))
    return parts


def cfg_arches(expr: str, features: frozenset[str] = frozenset()) -> frozenset[str] | None:
    """The arches a `cfg(...)` predicate admits; None when it tests anything else.

    `feature = "x"` is decided by `features`, the dependency's DEFAULT feature set.
    """
    expr = expr.strip()
    m = re.fullmatch(r'target_arch\s*=\s*"(\w+)"', expr)
    if m:
        return frozenset({m.group(1)}) & _ALL_ARCHES
    m = re.fullmatch(r'feature\s*=\s*"([\w-]+)"', expr)
    if m:
        return _ALL_ARCHES if m.group(1) in features else frozenset()
    m = re.fullmatch(r"(not|any|all)\s*\((.*)\)", expr, re.S)
    if not m:
        return None
    parts = [cfg_arches(p, features) for p in _split_top(m.group(2))]
    if any(p is None for p in parts):
        return None
    sets = [p for p in parts if p is not None]
    if m.group(1) == "not":
        return _ALL_ARCHES - sets[0] if len(sets) == 1 else None
    if m.group(1) == "any":
        return frozenset().union(*sets)
    return _ALL_ARCHES.intersection(*sets)


def _balanced(text: str, start: int) -> int:
    """Index just past the `)` closing the `(` before `start`."""
    depth = 1
    i = start
    while i < len(text) and depth:
        depth += text[i] == "("
        depth -= text[i] == ")"
        i += 1
    return i


def substrate_items(
    text: str, features: frozenset[str] = frozenset()
) -> list[tuple[str, str, frozenset[str]]]:
    """`(name, kind, arches)` for each helper a substrate source file makes importable.

    Kind `fn`: a `pub fn` at the file's top level, or one declared in a top-level `extern`
    block. Kind `macro`: a `#[macro_export]` `macro_rules!` (it lives at the crate root).
    `arches` is what every enclosing `#[cfg(target_arch ...)]` admits, the file's own
    `#![cfg]` included. An item behind any other cfg (a feature) is left out: whether it
    exists is not a property of the target -- unless it is one of `features`, the -sys crate's
    DEFAULT features, which every plain path dependency gets (ntkrnl-sys's whole `inline`
    module is `#[cfg(feature = "inline_exports")]`).
    """
    masked = _LITERAL_OR_COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)
    file_arches: frozenset[str] | None = _ALL_ARCHES
    stack: list[tuple[str, frozenset[str] | None]] = []
    pending: frozenset[str] | None = _ALL_ARCHES
    export = False
    out: list[tuple[str, str, frozenset[str]]] = []
    pos = 0
    while True:
        m = _SUBSTRATE_TOKEN.search(masked, pos)
        if m is None:
            break
        pos = m.end()
        here = stack[-1][1] if stack else file_arches
        if m.group("cfg"):
            end = _balanced(masked, m.end())
            # The predicate from the ORIGINAL text: masking blanked its `"x86"`.
            arches = cfg_arches(text[m.end() : end - 1], features)
            pos = end
            if m.group("cfg").startswith("#!"):
                file_arches = (
                    None if arches is None or file_arches is None else (file_arches & arches)
                )
            else:
                pending = None if arches is None or pending is None else pending & arches
            continue
        if m.group("export"):
            export = True
            continue
        both = None if here is None or pending is None else here & pending
        if m.group("extern"):
            stack.append(("extern", both))
        elif m.group("open"):
            stack.append(("other", both))
        elif m.group("close"):
            if stack:
                stack.pop()
        elif m.group("macro"):
            if export and both:
                out.append((m.group("macro"), "macro", both))
        elif m.group("fn"):
            if all(kind == "extern" for kind, _ in stack) and both:
                # "decl": declared in an extern block, no body -- another declaration of an
                # import, not a helper with an implementation of its own.
                out.append((m.group("fn"), "decl" if stack else "fn", both))
            # The body's `{` opens under the fn's own cfg, already applied.
            pending = _ALL_ARCHES
            export = False
            continue
        pending = _ALL_ARCHES
        export = False
    return out


def _module_paths(
    src: Path, features: frozenset[str] = frozenset()
) -> dict[Path, tuple[str, frozenset[str] | None]]:
    """Each `pub mod` source file -> (its path from the crate root, the arches it builds on)."""
    out: dict[Path, tuple[str, frozenset[str] | None]] = {}

    def walk(parent: Path, prefix: str, arches: frozenset[str] | None, here: Path) -> None:
        if not parent.is_file():
            return
        raw = parent.read_text(errors="replace")
        masked = _LITERAL_OR_COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), raw)
        for m in re.finditer(r"((?:#\[cfg\([^\]]*\)\]\s*)*)pub\s+mod\s+([A-Za-z_]\w*)\s*;", masked):
            mod_arches = arches
            attrs = raw[m.start(1) : m.end(1)]  # the original: masking blanked `"x86"`
            for c in re.findall(r"#\[cfg\((.*?)\)\]\s*", attrs, re.S):
                a = cfg_arches(c, features)
                mod_arches = None if a is None or mod_arches is None else mod_arches & a
            name = m.group(2)
            path = f"{prefix}::{name}" if prefix else name
            for f in (here / f"{name}.rs", here / name / "mod.rs"):
                if f.is_file():
                    out[f] = (path, mod_arches)
                    walk(f, path, mod_arches, here / name)
                    break

    lib = src / "lib.rs"
    out[lib] = ("", _ALL_ARCHES)
    walk(lib, "", _ALL_ARCHES, src)
    return out


def default_features(manifest: Path) -> frozenset[str]:
    """A crate's `[features] default = [...]`, expanded through the features it names."""
    if not manifest.is_file():
        return frozenset()
    m = re.search(r"^\[features\](.*?)(?=^\[|\Z)", manifest.read_text(), re.M | re.S)
    if not m:
        return frozenset()
    table = {
        k: re.findall(r'"([\w-]+)"', v)
        for k, v in re.findall(r"^([\w-]+)\s*=\s*(\[[^\]]*\])", m.group(1), re.M | re.S)
    }
    out: set[str] = set()
    todo = list(table.get("default", []))
    while todo:
        f = todo.pop()
        if f not in out:
            out.add(f)
            todo += table.get(f, [])
    return frozenset(out)


def substrate_helpers(
    names: set[str], sys_src: Path, ident: str
) -> tuple[dict[str, dict[str, str | None]], dict[str, str]]:
    """Where a kernel -sys crate implements each wanted header helper, per arch.

    ntkrnl-sys ports wdm.h's inlines and macros in Rust outside its bindgen glob
    (`inline::mm::MmGetSystemAddressForMdlSafe`, `interlocked::InterlockedPushEntrySList`,
    `inline::ke::KeQuerySystemTime`), and a driver prelude used to re-export them by hand.
    Tree consensus cannot stand in: atapi's `InsertTailList` had 16 copies across the tree
    in two ABIs, so resolve refused it, and `MmGetMdlVirtualAddress` -- a C macro to the
    module -- was never even asked. The substrate is the authority, read per arch:

    - a module under `inline::` (the crate's documented home for these) beats any other;
    - two remaining paths for one arch is ambiguous, reported, never chosen;
    - an arch with no Rust helper is fine when its bindgen glob declares the name (i386's
      `KeQuerySystemTime` is a real export, wdm.h's macro is amd64/arm64-only) -> `None`;
    - otherwise the name is reported with the arch it is missing on.

    Returns `({name: {target_arch: module path | None}}, {name: why not})`.
    """
    features = default_features(sys_src.parent / "Cargo.toml")
    index: dict[str, list[tuple[str, frozenset[str]]]] = {}
    decls: set[tuple[str, str, str]] = set()  # (name, module, arch): an extern declaration
    bodied: set[tuple[str, str, str]] = set()  # ... an implementation (fn with a body, macro)
    for f, (mod, mod_arches) in _module_paths(sys_src, features).items():
        if mod_arches is None or _NOT_HELPERS.search(f.stem):
            continue
        for name, kind, arches in substrate_items(f.read_text(errors="replace"), features):
            if name in names and arches & mod_arches:
                path = "" if kind == "macro" else mod
                index.setdefault(name, []).append((path, arches & mod_arches))
                if kind == "decl":
                    for arch in arches & mod_arches:
                        decls.add((name, path, arch))
                else:
                    for arch in arches & mod_arches:
                        bodied.add((name, path, arch))
    globs = {
        arch: "\n".join(
            p.read_text(errors="replace")
            for p in (sys_src / f"bindgen_{suffix}.rs", sys_src / "bindgen_common.rs")
            if p.is_file()
        )
        for arch, suffix in ARCHES.items()
    }
    found: dict[str, dict[str, str | None]] = {}
    why: dict[str, str] = {}
    for name in sorted(set(index)):
        per: dict[str, str | None] = {}
        for arch in ARCHES:
            paths = {p for p, a in index[name] if arch in a}
            inline = {p for p in paths if p.startswith("inline::")}
            paths = inline or paths
            # A helper OUTSIDE inline:: for a function the bindgen glob already declares is
            # only a second extern declaration, written in ntkrnl-sys's hand `base` types
            # (`ULONG_PTR = usize`) where the glob, and so the island, has bindgen's
            # (`c_ulonglong` on amd64). Importing it over the glob made every `as SIZE_T` /
            # `as ULONG_PTR` argument the C wrote a mismatch (ndis: ExAllocatePoolWithTag,
            # KeBugCheckEx). The glob's declaration is the island's own universe: keep it.
            # (Only a DECLARATION: ke.rs's amd64 KeGetCurrentIrql is an implementation, `mov
            # cr8`, and the glob's extern of it links to nothing on amd64.)
            if (
                not inline
                and paths
                and all((name, p, arch) in decls and (name, p, arch) not in bodied for p in paths)
                and re.search(rf"^\s*pub\s+fn\s+{re.escape(name)}\b", globs[arch], re.M)
            ):
                paths = set()
            if len(paths) == 1:
                per[arch] = paths.pop()
            elif paths:
                why[name] = f"{ident} defines it in {', '.join(sorted(paths))} on {arch}"
                break
            elif re.search(
                rf"^\s*pub\s+(?:fn|const|static)\s+{re.escape(name)}\b", globs[arch], re.M
            ):
                per[arch] = None
            else:
                why[name] = f"{ident} has no {arch} definition"
                break
        else:
            found[name] = per
    return found, why


_EXPLICIT_DDK = re.compile(r"^\s*use\s+\w+::ddk::\{([^}]*)\}", re.M)
# An extern-block declaration: no body, no initializer (a bindgen accessor method has a body).
_DECLARED = r"^\s*pub\s+(?:fn|static(?:\s+mut)?)\s+([A-Za-z_]\w*)\b[^{;=]*;"


_IMPORTLIBS = re.compile(r"add_importlibs\(\s*\S+\s+([^)]*)\)")


def module_exports(module_dir: Path, repo: Path, every_arch: bool = True) -> set[str] | None:
    """Names the module's import libraries export on EVERY arch; None when unknowable.

    The authority for "can this be an extern at all". A wdm.h FORCEINLINE (InitializeListHead,
    ExAllocatePoolZero, KeGetCurrentIrql on amd64) is declared by the island like any export,
    resolves, compiles -- and is undefined at link, because no .spec exports it. atapi linked
    17 such names. A name some arch does not export is left to the substrate to decide per
    arch (RtlUlongByteSwap: a fastcall export on i386, an intrinsic elsewhere).
    """
    from .spec_parser import parse_spec

    cmake = module_dir / "CMakeLists.txt"
    if not cmake.is_file():
        return None
    libs = {w for m in _IMPORTLIBS.finditer(cmake.read_text()) for w in m.group(1).split()}
    specs: list[Path] = []
    for lib in sorted(libs):
        found = subprocess.run(
            ["git", "ls-files", f"*/{lib}.spec", f"{lib}.spec"],
            cwd=repo,
            capture_output=True,
            text=True,
            check=False,
        ).stdout.split()
        if not found:
            return None  # an import lib we cannot read: no authority, decide nothing
        specs += [repo / f for f in found]
    per_arch = [
        set().union(*(parse_spec(spec, arch).keys() for spec in specs))
        | _static_lib_symbols(cmake.read_text(), repo, arch)
        for arch in ("i386", "amd64", "arm64")
    ]
    if not per_arch:
        return None
    # every_arch=False: exported on SOME arch -- i386's GetWindowLongPtrW is a user32.h macro
    # for GetWindowLongW, which the prelude's per-arch link_name settles.
    return set.intersection(*per_arch) if every_arch else set.union(*per_arch)


_LINKLIBS = re.compile(r"target_link_libraries\(\s*\S+\s+([^)]*)\)")


def _static_lib_symbols(cmake_text: str, repo: Path, arch: str) -> set[str]:
    """What the module's STATIC libraries define (`target_link_libraries(mspaint uuid ..)`):
    uuid's GUID_NULL is a data symbol no .spec exports, yet it links. Read off THIS tree's
    built archive with llvm-nm, never guessed; an archive not built yet contributes nothing
    (the name then stays unresolved, loudly)."""
    nm = repo / ".build-tools" / "llvm-mingw" / "bin" / "llvm-nm"
    if not nm.is_file():
        return set()
    out: set[str] = set()
    for m in _LINKLIBS.finditer(cmake_text):
        for lib in m.group(1).split():
            if not re.fullmatch(r"[\w-]+", lib):
                continue
            hits = sorted((repo / f"build-{arch}" / "sdk" / "lib").glob(f"**/lib{lib}.a"))
            if not hits:
                continue
            res = subprocess.run(
                [str(nm), "--defined-only", "--extern-only", str(hits[0])],
                capture_output=True,
                text=True,
                check=False,
            )
            for line in res.stdout.splitlines():
                parts = line.split()
                if len(parts) == 3:
                    sym = parts[2]
                    # i386 decorates C symbols with one leading underscore.
                    out.add(sym[1:] if arch == "i386" and sym.startswith("_") else sym)
    return out


def called_names(modules_dir: Path) -> set[str]:
    """Every identifier the translated modules CALL (`X(`, not `X!(`)."""
    out: set[str] = set()
    for rs in sorted(modules_dir.glob("*.rs")):
        out |= set(re.findall(r"(?<![!\w])([A-Za-z_]\w*)\s*\(", rs.read_text(errors="replace")))
    return out


def unpin_substrate(modules_dir: Path, names: set[str], ident: str) -> dict[str, int]:
    """Point every module at the prelude's substrate import of `names`. Returns {file: edits}.

    The translate stage wrote per-file `use ntkrnl_sys::ddk::{.., KeInitializeSpinLock, ..}`
    lists and some bodies call `ntkrnl_sys::ddk::KeGetCurrentIrql()` by path. Either one binds
    the bindgen EXTERN and beats the prelude's import of the real helper -- which for a header
    inline means an undefined symbol at link (atapi: 7 of them, after resolve had already
    imported every one).
    """
    out: dict[str, int] = {}
    if not names:
        return out
    alt = "|".join(sorted(map(re.escape, names), key=len, reverse=True))
    qualified = re.compile(rf"\b{re.escape(ident)}::ddk::({alt})\b")
    lists = re.compile(rf"^(\s*use\s+{re.escape(ident)}::ddk::\{{)([^}}]*)(\}};[ \t]*\n?)", re.M)
    for rs in sorted(modules_dir.glob("*.rs")):
        text = rs.read_text(errors="replace")
        n = 0

        def trim(m: re.Match[str]) -> str:
            nonlocal n
            items = [i.strip() for i in m.group(2).split(",") if i.strip()]
            kept = [i for i in items if i.split(" as ")[0].strip() not in names]
            n += len(items) - len(kept)
            return f"{m.group(1)}{', '.join(kept)}{m.group(3)}" if kept else ""

        new = lists.sub(trim, text)
        new, k = qualified.subn(r"\1", new)
        n += k
        if new != text:
            rs.write_text(new)
            out[rs.name] = n
    return out


def glob_ties(crate_src: Path, modules_dir: Path, sys_src: Path) -> dict[str, list[str]]:
    """Names two prelude globs both export on some arch -> the arches (E0659 there).

    The island (`crate::types::*`) and the kernel -sys glob (`ntkrnl_sys::ddk::*`) declare
    many of the same functions, and a use through both is ambiguous. Which ones collide is
    per ARCH: atapi's i386 island declares KeGetCurrentIrql (a real export there) where the
    amd64 island does not (an inline there), so resolve -- which compiles amd64 -- never saw
    it. Read from the islands rather than from rustc, because the import this produces
    removes the very error that would justify it: a diagnostic-driven rule would drop it on
    the next run and add it back on the one after. The -sys crate's declaration wins, the
    one every other arch already resolves to.
    """
    # A name the prelude already IMPORTS or DEFINES is settled there (a second explicit import
    # is E0252). Only those: a name the prelude merely CALLS -- pciidex's ATA_IO_WAIT macro
    # calling KeStallExecutionProcessor -- is a use, and ambiguous like any other.
    prelude = crate_src / "prelude.rs"
    settled: set[str] = set()
    used: set[str] = set()
    if prelude.is_file():
        text = prelude.read_text(errors="replace")
        m = re.search(re.escape(SUBSTRATE_BEGIN) + r".*?" + re.escape(SUBSTRATE_END), text, re.S)
        rest = text[: m.start()] + text[m.end() :] if m else text
        for imp in re.finditer(r"(?m)^\s*pub(?:\([^)]*\))?\s+use\s+([^;]+);", rest):
            body = imp.group(1)
            names = re.findall(r"\{([^}]*)\}", body)
            parts = names[0].split(",") if names else [body.rsplit("::", 1)[-1]]
            for part in parts:
                word = part.strip().split(" as ")[-1].strip()
                if word and word != "*":
                    settled.add(word)
        settled |= set(
            re.findall(
                r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?"
                r"(?:fn|const|static(?:\s+mut)?|struct|union|enum|type|trait)\s+([A-Za-z_]\w*)",
                rest,
            )
        )
        settled |= set(re.findall(r"macro_rules!\s+([A-Za-z_]\w*)", rest))
        used |= set(re.findall(r"\b([A-Za-z_]\w*)\s*\(", rest))
    for rs in sorted(modules_dir.glob("*.rs")):
        text = rs.read_text(errors="replace")
        explicit = {
            w.strip().split(" as ")[0]
            for m in _EXPLICIT_DDK.finditer(text)
            for w in m.group(1).split(",")
        }
        used |= set(re.findall(r"\b([A-Za-z_]\w*)\s*\(", text)) - explicit
    ties: dict[str, list[str]] = {}
    for arch, suffix in ARCHES.items():
        island = crate_src / f"types_{suffix}.rs"
        if not island.is_file():
            continue
        mine = set(re.findall(_DECLARED, island.read_text(errors="replace"), re.M)) & used - settled
        theirs: set[str] = set()
        for p in (sys_src / f"bindgen_{suffix}.rs", sys_src / "bindgen_common.rs"):
            if p.is_file():
                theirs |= set(re.findall(_DECLARED, p.read_text(errors="replace"), re.M))
        for name in sorted(mine & theirs):
            ties.setdefault(name, []).append(arch)
    return ties


def with_glob_ties(
    found: dict[str, dict[str, str | None]], ties: dict[str, list[str]]
) -> dict[str, dict[str, str | None]]:
    """`found` plus an explicit `ddk` import wherever a tie has no helper on that arch."""
    out = {k: dict(v) for k, v in found.items()}
    for name, arches in ties.items():
        per = out.setdefault(name, {a: None for a in ARCHES})
        for arch in arches:
            if per.get(arch) is None:
                per[arch] = "ddk"
    return out


def island_substrate(src: Path, workspace: Path) -> Path | None:
    """The src dir of the -sys crate a bindgen island globs (`use ntkrnl_sys::ddk::*;`)."""
    for island in sorted(src.glob("types_*.rs")):
        m = re.search(
            r"^(?:pub\s+)?use\s+(\w+)::\w+::\*;", island.read_text(errors="replace"), re.M
        )
        if m:
            sys_src = workspace / m.group(1).replace("_", "-") / "src"
            return sys_src if sys_src.is_dir() else None
    return None


def render_substrate(found: dict[str, dict[str, str | None]], ident: str) -> str:
    lines = [SUBSTRATE_BEGIN]
    for name in sorted(found):
        by_path: dict[str, list[str]] = {}
        for arch, path in found[name].items():
            if path is not None:
                by_path.setdefault(path, []).append(arch)
        for path, arches in sorted(by_path.items()):
            if len(arches) == 2:
                (other,) = set(ARCHES) - set(arches)
                lines.append(f'#[cfg(not(target_arch = "{other}"))]')
            elif len(arches) == 1:
                lines.append(f'#[cfg(target_arch = "{arches[0]}")]')
            lines.append(f"pub use {ident}::{path + '::' if path else ''}{name};")
    lines.append(SUBSTRATE_END)
    return "\n".join(lines) + "\n"


def existing_substrate(prelude: Path) -> set[str]:
    m = re.search(
        re.escape(SUBSTRATE_BEGIN) + r"(.*?)" + re.escape(SUBSTRATE_END), prelude.read_text(), re.S
    )
    return set(re.findall(r"pub use [\w:]+::(\w+);", m.group(1))) if m else set()


def write_substrate(prelude: Path, block: str) -> bool:
    text = prelude.read_text()
    m = re.search(
        re.escape(SUBSTRATE_BEGIN) + r".*?" + re.escape(SUBSTRATE_END) + r"\n?", text, re.S
    )
    new = text[: m.start()] + block + text[m.end() :] if m else text.rstrip("\n") + "\n\n" + block
    if new == text:
        return False
    prelude.write_text(new)
    return True


_LONG_PTR_ALIAS = re.compile(r"\bfn ((?:Get|Set)(?:Window|Class)Long)Ptr([AW]?)\b")


def _x86_alias(sig: str) -> str:
    """winuser.h `#define`s Get/SetWindowLongPtrW and Get/SetClassLongPtrW to the non-`Ptr`
    function on 32-bit, so i386 user32 exports no `_GetWindowLongPtrW@8`. A declaration copied
    from another crate loses that crate's `link_name`, and comctl32's i386 link failed on all
    four. The attribute line, indented for an extern block, or "" when none applies."""
    m = _LONG_PTR_ALIAS.search(sig)
    if m is None or re.search(
        r'target_arch\s*=\s*"x86"[^\]]*link_name|^\s*#\[link_name', sig, re.M
    ):
        return ""
    # The bare TCHAR spelling (`GetWindowLongPtr`, copied with only 64-bit link_names, as
    # mspaint's prelude got from shell32) is the W function in this tree's UNICODE modules.
    return f'    #[cfg_attr(target_arch = "x86", link_name = "{m.group(1)}{m.group(2) or "W"}")]\n'


def render(copied: dict[str, Item]) -> str:
    consts = [i for i in copied.values() if i.kind in ("const", "type")]
    fns: dict[str, list[Item]] = {}
    for i in copied.values():
        if i.kind in ("fn", "static"):
            fns.setdefault(i.abi or '"C"', []).append(i)
    out = [BEGIN]
    for i in sorted(consts, key=lambda i: i.text):
        out.append(f"pub {i.text} // {i.path}")
    for abi, items in sorted(fns.items()):
        out.append(f"extern {abi} {{")
        for i in sorted(items, key=lambda i: i.text):
            text = (
                i.text
                if i.text.startswith(("fn", "static"))
                else re.sub(r"^.*?\bfn\b", "fn", i.text)
            )
            out.append(f"{_x86_alias(text)}    pub {text} // {i.path}")
        out.append("}")
    out.append(END)
    return "\n".join(out) + "\n"


def write_block(prelude: Path, block: str) -> bool:
    text = prelude.read_text()
    m = re.search(re.escape(BEGIN) + r".*?" + re.escape(END) + r"\n?", text, re.S)
    new = text[: m.start()] + block + text[m.end() :] if m else text.rstrip("\n") + "\n\n" + block
    if new == text:
        return False
    prelude.write_text(new)
    return True


def existing_block(prelude: Path) -> dict[str, Item]:
    """Items a previous run copied, so a re-run keeps them (cargo no longer reports them)."""
    text = prelude.read_text()
    m = re.search(re.escape(BEGIN) + r"(.*?)" + re.escape(END), text, re.S)
    if not m:
        return {}
    out: dict[str, Item] = {}
    abi = None
    for line in m.group(1).splitlines():
        s = line.strip()
        ab = re.match(r'^extern ("[^"]+") \{$', s)
        if ab:
            abi = ab.group(1)
            continue
        if s == "}":
            abi = None
            continue
        body, _, src = s.partition(" // ")
        body = re.sub(r"^pub\s+", "", body)
        n = re.search(r"\b(?:const|type|fn|static(?:\s+mut)?)\s+([A-Za-z_]\w*)", body)
        if n:
            kind = "fn" if body.startswith("fn") else body.split()[0]
            out[n.group(1)] = Item(kind, body, abi, src)
    return out


# --------------------------------------------------------------------------------------
# constants the C headers define and no Rust in the tree does
# --------------------------------------------------------------------------------------

FACTS_BEGIN = (
    "// ==== port facts: constants evaluated by clang from the C headers "
    "(generated -- `rustconv port resolve`) ===="
)
FACTS_END = "// ==== end port facts ===="

_CONSTANT_NAME = re.compile(r"^_*[A-Z][A-Z0-9_]*$")
# Windows is LLP64: C `long` is 32-bit on every target we build.
_C_PRIMITIVE = {
    "int": "i32",
    "unsigned int": "u32",
    "long": "i32",
    "unsigned long": "u32",
    "long long": "i64",
    "unsigned long long": "u64",
    "short": "i16",
    "unsigned short": "u16",
    "char": "i8",
    "signed char": "i8",
    "unsigned char": "u8",
    "_Bool": "u8",
}


def looks_constant(name: str) -> bool:
    return bool(_CONSTANT_NAME.match(name))


def fact_rust_type(
    c_type: str | None, value: int, known_types: set[str], *, bare_int_unsigned: bool = False
) -> str:
    """The Rust spelling of a constant's C type: its typedef when the crate has it
    (`NTSTATUS`), else the LLP64 primitive, else (unprobed) the narrowest by value."""
    if c_type:
        sugar, _, canonical = c_type.partition("|")
        canonical = (canonical or sugar).removeprefix("const ").strip()
        if re.fullmatch(r"[A-Za-z_]\w*", sugar) and sugar in known_types:
            # A POINTER-typed constant (taskdialog.c's `TD_ERROR_ICON`, MAKEINTRESOURCEW(-2))
            # needs an int-to-pointer cast; the leading `*` tells render_facts so.
            return f"*{sugar}" if canonical.endswith("*") else sugar
        if bare_int_unsigned and sugar == "int" and value >= 0:
            # A bare `#define X 4` is an `int` to clang, and bindgen -- hence every island
            # and -sys constant these bodies mix it with -- spells the same macro `u32`.
            # Following the C type put atapi at 190 `u32 & i32` errors against flag fields;
            # an explicitly typed or negative constant still keeps its C type. Kernel crates
            # only (the caller decides): setuplib's banked fixes were made against `i32`.
            return "u32"
        if canonical in _C_PRIMITIVE:
            return _C_PRIMITIVE[canonical]
    if value < 0:
        return "i32" if value >= -(2**31) else "i64"
    return "u32" if value < 2**32 else "u64"


_UNSIGNED_BITS = {"u8": 8, "u16": 16, "u32": 32, "u64": 64}


def render_facts(facts: dict[str, tuple[str, int | str, str]]) -> str:
    """name -> (rust type, value, source TU)."""
    out = [FACTS_BEGIN]
    for name, (ty, value, src) in sorted(facts.items()):
        literal = str(value)
        if ty in ("f32", "f64"):
            out.append(f"pub const {name}: {ty} = {literal}; // clang, {src}")
            continue
        assert isinstance(value, int)  # only a floating fact carries its literal text
        if ty.startswith("*"):
            ty = ty[1:]
            literal = f"0x{value & 0xFFFFFFFFFFFFFFFF:X}usize as {ty}"
        elif value < 0 and ty in ("i32", "NTSTATUS", "LONG", "HRESULT"):
            literal = f"0x{value & 0xFFFFFFFF:08X}_u32 as {ty}"
        elif value < 0 and ty in _UNSIGNED_BITS:
            # clang's evaluator answers in a signed 64-bit: calc's `_UI64_MAX`
            # (0xffffffffffffffffui64) came back -1, and `u64 = -1` does not compile.
            literal = f"0x{value & ((1 << _UNSIGNED_BITS[ty]) - 1):X}"
        elif value >= 10:
            literal = f"0x{value:X}"
        out.append(f"pub const {name}: {ty} = {literal}; // clang, {src}")
    out.append(FACTS_END)
    return "\n".join(out) + "\n"


def existing_facts(prelude: Path) -> dict[str, tuple[str, int | str, str]]:
    text = prelude.read_text()
    m = re.search(re.escape(FACTS_BEGIN) + r"(.*?)" + re.escape(FACTS_END), text, re.S)
    if not m:
        return {}
    out: dict = {}
    for line in m.group(1).splitlines():
        fl = re.match(r"^pub const (\w+): (f32|f64) = ([-+0-9.eE]+); // clang, (\S+)", line.strip())
        if fl:
            out[fl.group(1)] = (fl.group(2), fl.group(3), fl.group(4))
            continue
        f = re.match(
            r"^pub const (\w+): (\w+) = (0x[0-9A-F]+)?(?:_u32 as \w+|usize as \w+)?(-?\d+)?;"
            r" // clang, (\S+)",
            line.strip(),
        )
        if not f:
            continue
        name, ty, hexv, decv, src = f.groups()
        value = int(hexv, 16) if hexv else int(decv)
        if hexv and line.find("_u32 as") > 0 and value >= 2**31:
            value -= 2**32
        out[name] = (f"*{ty}" if "usize as" in line else ty, value, src)
    return out


def write_facts(prelude: Path, block: str) -> bool:
    text = prelude.read_text()
    m = re.search(re.escape(FACTS_BEGIN) + r".*?" + re.escape(FACTS_END) + r"\n?", text, re.S)
    new = text[: m.start()] + block + text[m.end() :] if m else text.rstrip("\n") + "\n\n" + block
    if new == text:
        return False
    prelude.write_text(new)
    return True


LOCAL_TYPES_BEGIN = (
    "// ==== port local types: types the module's .c files define, laid out by c2rust "
    "(generated -- `rustconv port resolve`) ===="
)
LOCAL_TYPES_END = "// ==== end port local types ===="


def existing_local_types(prelude: Path) -> set[str]:
    text = prelude.read_text()
    m = re.search(re.escape(LOCAL_TYPES_BEGIN) + r"(.*?)" + re.escape(LOCAL_TYPES_END), text, re.S)
    if not m:
        return set()
    return set(re.findall(r"(?m)^pub (?:struct|union|type) (\w+)", m.group(1)))


def write_local_types(prelude: Path, items: dict[str, tuple[str, str]]) -> bool:
    """name -> (rust text, source TU)."""
    from .crate_repair import _BINDGEN_KEYWORDS

    out = [LOCAL_TYPES_BEGIN]
    for name, (text, src) in sorted(items.items()):
        out.append(f"// {name}: c2rust, {src}")
        # c2rust escapes a keyword-named C field as `type_0`, bindgen and crate-repair as
        # `type_`: reg.exe's `data_type` came out `type_0` while its body read `.type_`.
        text = re.sub(
            r"(\bpub\s+)(\w+)_0(\s*:)",
            lambda m: (
                m.group(1) + m.group(2) + "_" + m.group(3)
                if m.group(2) in _BINDGEN_KEYWORDS
                else m.group(0)
            ),
            text,
        )
        out.append(variadic_shim.lower_va_list_types(text)[0])
    out.append(LOCAL_TYPES_END)
    block = "\n".join(out) + "\n"
    text = prelude.read_text()
    m = re.search(
        re.escape(LOCAL_TYPES_BEGIN) + r".*?" + re.escape(LOCAL_TYPES_END) + r"\n?", text, re.S
    )
    new = text[: m.start()] + block + text[m.end() :] if m else text.rstrip("\n") + "\n\n" + block
    if new == text:
        return False
    prelude.write_text(new)
    return True


EXTERNS_BEGIN = (
    "// ==== port externs: C functions declared outside the island -- c2rust signature, "
    "clang calling convention (generated -- `rustconv port resolve`) ===="
)
EXTERNS_END = "// ==== end port externs ===="


def existing_externs(prelude: Path) -> set[str]:
    text = prelude.read_text()
    m = re.search(re.escape(EXTERNS_BEGIN) + r"(.*?)" + re.escape(EXTERNS_END), text, re.S)
    if not m:
        return set()
    # Both kinds, or a re-run drops the aliases its own first pass wrote.
    return set(re.findall(r"\bpub fn (\w+)\(", m.group(1))) | set(
        re.findall(r"\bpub use \S+ as (\w+);", m.group(1))
    )


def render_externs(sigs: dict[str, tuple[str, str, str]]) -> str:
    """name -> (abi, `fn ...;` signature, source TU); grouped by ABI. Aliases are `pub use`."""
    out = [EXTERNS_BEGIN]
    by_abi: dict[str, list[tuple[str, str, str]]] = {}
    for name, (abi, sig, src) in sorted(sigs.items()):
        if abi == "C" and _LONG_PTR_ALIAS.search(sig):
            # Get/SetWindowLongPtrW and Get/SetClassLongPtrW are WINAPI. On i386 they are
            # MACROS, so clang reports no calling convention for them and the default was "C":
            # sol and spider then imported `GetWindowLongW` -- stdcall -- as cdecl.
            abi = "system"
        by_abi.setdefault(abi, []).append((name, sig, src))
    for abi, items in sorted(by_abi.items()):
        if abi == "use":
            out += [f"{sig} // {src}" for _, sig, src in items]
            continue
        out.append(f'extern "{abi}" {{')
        for _, sig, src in items:
            # c2rust's text is already indented for an extern block (params at 8).
            out.append(f"{_x86_alias(sig)}    pub {sig} // {src}")
        out.append("}")
    out.append(EXTERNS_END)
    return "\n".join(out) + "\n"


def write_externs(prelude: Path, block: str) -> bool:
    text = prelude.read_text()
    m = re.search(re.escape(EXTERNS_BEGIN) + r".*?" + re.escape(EXTERNS_END) + r"\n?", text, re.S)
    new = text[: m.start()] + block + text[m.end() :] if m else text.rstrip("\n") + "\n\n" + block
    if new == text:
        return False
    prelude.write_text(new)
    return True


def hand_written_names(prelude: Path) -> set[str]:
    """Names the prelude defines OUTSIDE its generated `// ==== ... ====` blocks."""
    if not prelude.is_file():
        return set()
    from .profile_derive import prelude_provided_names

    text = re.sub(
        r"^// ==== (?!end)[^\n]*\n.*?^// ==== end [^\n]*\n?",
        "",
        prelude.read_text(),
        flags=re.S | re.M,
    )
    names: set[str] = set(re.findall(r"^\s*(?:pub\s+)?(?:unsafe\s+)?fn\s+(\w+)", text, re.M))
    names |= set(re.findall(r"macro_rules!\s+(\w+)", text))
    return names | prelude_provided_names(None, text)
