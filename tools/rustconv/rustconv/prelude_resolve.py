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
    return names


def _norm(text: str) -> str:
    text = re.sub(r"//[^\n]*", "", text)
    text = re.sub(r"#\[[^\]]*\]", "", text)
    text = re.sub(r"\bpub(\([^)]*\))?\s+", "", text)
    return " ".join(text.split())


_CONST = r"^[ \t]*pub[ \t]+const[ \t]+{n}[ \t]*:[^=;]+=[^;]+;"
_TYPE = r"^[ \t]*pub[ \t]+type[ \t]+{n}[ \t]*=[^;]+;"


def is_test_source(rel: str) -> bool:
    """A test harness file (`tests/`, a `*-difftest` crate): never tree consensus."""
    return "/tests/" in f"/{rel}" or bool(re.search(r"(?:^|/)[\w-]*-difftest/", rel))


def workspace_items(names: set[str], workspace: Path, exclude: Path) -> dict[str, list[Item]]:
    """Every tracked definition of each name: consts, type aliases, extern fn decls."""
    if not names:
        return {}
    pattern = "|".join(sorted(names))
    proc = subprocess.run(
        ["git", "grep", "-l", "-w", "-E", pattern, "--", "*.rs"],
        cwd=workspace,
        capture_output=True,
        text=True,
    )
    found: dict[str, list[Item]] = {n: [] for n in names}
    for rel in proc.stdout.split():
        path = workspace / rel
        if exclude in path.parents or "/target/" in rel:
            continue
        # Test harnesses are not the tree's declarations: diskpart got `_wfopen` and `fgetws`
        # from msvcrt-difftest/tests/file.rs, typed with that file's private `FILEP`.
        if is_test_source(rel):
            continue
        try:
            src = path.read_text(errors="replace")
        except OSError:
            continue
        # One identifier scan per file, not one regex per name: with 544 unresolved names
        # over the tree that loop was a 40-minute CPU stall inside `port replay`.
        present = names & set(re.findall(r"[A-Za-z_]\w*", src))
        for n in present:
            for kind, pat in (("const", _CONST), ("type", _TYPE)):
                for m in re.finditer(pat.format(n=re.escape(n)), src, re.M):
                    found[n].append(Item(kind, _norm(m.group(0)), None, rel))
        for _start, _end, body, abi in extern_block_spans(src):
            for decl in split_decls(body):
                m = re.search(r"\bfn\s+([A-Za-z_]\w*)\s*\(", decl)
                if m and m.group(1) in present:
                    found[m.group(1)].append(Item("fn", _norm(decl), abi, rel))
        for name, decl, abi in exported_definitions(src, present):
            found[name].append(Item("fn", _norm(decl), abi, rel))
    return found


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
    if consts and not fns:
        literal = [i for i in consts if const_value(i) is not None]
        values = {const_value(i) for i in literal}
        if len(values) != 1:
            return None, (f"{len(values)} different values" if values else "no literal definition")
        # An expression definition (`CTL_CODE(...)`) is not evaluated here; it neither
        # vetoes nor is copied -- the literals that agree are the evidence.
        return _most_common(literal), ""
    if fns and not consts:
        classes = {_abi_class(i.abi) for i in fns}
        if len(classes) != 1:
            return None, f"ABI disagreement ({', '.join(sorted(classes))})"
        return _most_common(fns), ""
    if any(i.kind == "type" for i in items):
        return None, "a type: add its header to the island allowlist"
    return None, "mixed kinds"


def _most_common(items: list[Item]) -> Item:
    counts: dict[tuple[str, str | None], int] = {}
    for i in items:
        counts[(i.text, i.abi)] = counts.get((i.text, i.abi), 0) + 1
    best = max(counts.items(), key=lambda kv: (kv[1], -len(kv[0][0])))[0]
    return next(i for i in items if (i.text, i.abi) == best)


def island_enum_aliases(names: set[str], src: Path) -> dict[str, Item]:
    """`FreeLdr` -> `pub const FreeLdr: _BOOT_STORE_TYPE = _BOOT_STORE_TYPE_FreeLdr;`.

    bindgen prefixes every enumerator with its enum's name, while C code -- and so the
    sealed translation -- spells it bare. netid, samlib and others carried these aliases by
    hand. The island is the authority, so this is exact: a name gets an alias only when
    every arch's island has exactly one `<E>_<name>: <E>` constant, and the same E.
    """
    islands = sorted(src.glob("types_*.rs"))
    if not islands or not names:
        return {}
    texts = [p.read_text(errors="replace") for p in islands]
    out: dict[str, Item] = {}
    for name in sorted(names):
        pat = re.compile(
            rf"^pub const (_?[A-Za-z][A-Za-z0-9_]*)_{re.escape(name)}\s*:\s*([A-Za-z_]\w*)\s*=",
            re.M,
        )
        enums: set[str] = set()
        ok = True
        for text in texts:
            hits = {m.group(1) for m in pat.finditer(text) if m.group(1) == m.group(2)}
            if len(hits) != 1:
                ok = False
                break
            enums |= hits
        if ok and len(enums) == 1:
            e = enums.pop()
            out[name] = Item("const", f"const {name}: {e} = {e}_{name};", None, "island enum")
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
)


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
    changed: list[str] = []
    for (ident, module), names in shared_provides(workspace).items():
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
    r"(?P<kind>fn|static(?:\s+mut)?|const)\s+(?P<name>[A-Za-z_]\w*)\b",
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


def promote_pub(path: Path, name: str) -> bool:
    """Make a top-level definition of `name` in `path` `pub`. True if it changed."""
    text = path.read_text()
    pat = re.compile(
        r"^(?!pub)((?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?(?:fn|static(?:\s+mut)?|const)\s+"
        + re.escape(name)
        + r"\b)",
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


def render(copied: dict[str, Item]) -> str:
    consts = [i for i in copied.values() if i.kind in ("const", "type")]
    fns: dict[str, list[Item]] = {}
    for i in copied.values():
        if i.kind == "fn":
            fns.setdefault(i.abi or '"C"', []).append(i)
    out = [BEGIN]
    for i in sorted(consts, key=lambda i: i.text):
        out.append(f"pub {i.text} // {i.path}")
    for abi, items in sorted(fns.items()):
        out.append(f"extern {abi} {{")
        for i in sorted(items, key=lambda i: i.text):
            text = i.text if i.text.startswith("fn") else re.sub(r"^.*?\bfn\b", "fn", i.text)
            out.append(f"    pub {text} // {i.path}")
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
        n = re.search(r"\b(?:const|type|fn)\s+([A-Za-z_]\w*)", body)
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
            return sugar
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


def render_facts(facts: dict[str, tuple[str, int, str]]) -> str:
    """name -> (rust type, value, source TU)."""
    out = [FACTS_BEGIN]
    for name, (ty, value, src) in sorted(facts.items()):
        literal = str(value)
        if value < 0 and ty in ("i32", "NTSTATUS", "LONG", "HRESULT"):
            literal = f"0x{value & 0xFFFFFFFF:08X}_u32 as {ty}"
        elif value >= 10:
            literal = f"0x{value:X}"
        out.append(f"pub const {name}: {ty} = {literal}; // clang, {src}")
    out.append(FACTS_END)
    return "\n".join(out) + "\n"


def existing_facts(prelude: Path) -> dict[str, tuple[str, int, str]]:
    text = prelude.read_text()
    m = re.search(re.escape(FACTS_BEGIN) + r"(.*?)" + re.escape(FACTS_END), text, re.S)
    if not m:
        return {}
    out = {}
    for line in m.group(1).splitlines():
        f = re.match(
            r"^pub const (\w+): (\w+) = (0x[0-9A-F]+)?(?:_u32 as \w+)?(-?\d+)?; // clang, (\S+)",
            line.strip(),
        )
        if not f:
            continue
        name, ty, hexv, decv, src = f.groups()
        value = int(hexv, 16) if hexv else int(decv)
        if hexv and line.find("_u32 as") > 0 and value >= 2**31:
            value -= 2**32
        out[name] = (ty, value, src)
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
    out = [LOCAL_TYPES_BEGIN]
    for name, (text, src) in sorted(items.items()):
        out.append(f"// {name}: c2rust, {src}")
        out.append(text)
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
        by_abi.setdefault(abi, []).append((name, sig, src))
    for abi, items in sorted(by_abi.items()):
        if abi == "use":
            out += [f"{sig} // {src}" for _, sig, src in items]
            continue
        out.append(f'extern "{abi}" {{')
        for _, sig, src in items:
            # c2rust's text is already indented for an extern block (params at 8).
            out.append(f"    pub {sig} // {src}")
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


def declared_outside_externs(prelude: Path) -> set[str]:
    """Functions the prelude declares ANYWHERE except the port-externs block itself.

    `hand_written_names` drops every generated block, the tree-consensus one included, so a
    name that block already declares (appwiz: `RegQueryValueExW`, copied from desk) looked
    undeclared and the externs pass declared it again -- E0428 on a crate nobody touched.
    """
    if not prelude.is_file():
        return set()
    text = re.sub(
        re.escape(EXTERNS_BEGIN) + r".*?" + re.escape(EXTERNS_END) + r"\n?",
        "",
        prelude.read_text(),
        flags=re.S,
    )
    return set(re.findall(r"^\s*(?:pub\s+)?(?:unsafe\s+)?fn\s+(\w+)\s*\(", text, re.M))


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
