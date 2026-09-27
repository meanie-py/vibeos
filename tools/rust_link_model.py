#!/usr/bin/env python3
"""What a Rust crate actually DEFINES for the linker, per arch — a source model.

Shared by the link-level scanners (`scan_duplicate_symbols`). It answers the
questions a grep cannot:

  * Which `.rs` files does the crate COMPILE? Only those reachable from
    `lib.rs`/`main.rs` through `mod` declarations (honouring `#[path]`, inline
    `mod x { }` blocks and `include!`). An orphan file on disk defines nothing.
  * Under what cfg is each definition live? The conjunction of every
    `#[cfg]` stacked on the item, on each enclosing inline module, on every
    `mod` declaration on the path from the crate root, and every file-level
    `#![cfg]`. Stacked cfgs AND together — which is how an orphan cfg left behind
    by a deleted twin compiles an item out on every arch.
  * Which exported names does it define? `#[no_mangle]` / `#[export_name]`
    fns and statics, AND `.globl` labels inside `global_asm!` (inline strings or
    `include_str!`). The assembler is opaque to rustc, so an asm label and a
    Rust fn of the same name in one crate is never a compile error — the i386
    `RtlPrefetchMemoryNonTemporal` empty stub beat its own crate's
    `prefetch_asm.rs` this way and crashed kmtest (88b04bf7f86).

`Cond.satisfiable_with(other, arch)` decides whether two definitions can be
live in the same build: arch is fixed per build, every other cfg atom
(`feature = "smp"`, `debug_assertions`, …) is a free boolean, so two
definitions under `feature="smp"` / `not(feature="smp")` never collide.
"""

from __future__ import annotations

import itertools
import re
from dataclasses import dataclass, field
from functools import cache
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ARCHES: tuple[str, ...] = ("x86", "x86_64", "aarch64")

# --- cfg predicates -----------------------------------------------------------------------


def split_args(s: str) -> list[str]:
    out, depth, cur, in_str = [], 0, "", False
    for c in s:
        if c == '"':
            in_str = not in_str
        elif not in_str and c == "(":
            depth += 1
        elif not in_str and c == ")":
            depth -= 1
        if c == "," and depth == 0 and not in_str:
            out.append(cur.strip())
            cur = ""
        else:
            cur += c
    if cur.strip():
        out.append(cur.strip())
    return out


# Atoms fixed by the target: everything our three targets agree on or derive from arch.
_FIXED: dict[str, dict[str, bool]] = {
    'target_pointer_width="64"': {"x86": False, "x86_64": True, "aarch64": True},
    'target_pointer_width="32"': {"x86": True, "x86_64": False, "aarch64": False},
    'target_os="windows"': dict.fromkeys(ARCHES, True),
    'target_os="none"': dict.fromkeys(ARCHES, False),
    'target_os="linux"': dict.fromkeys(ARCHES, False),
    'target_endian="little"': dict.fromkeys(ARCHES, True),
    'target_endian="big"': dict.fromkeys(ARCHES, False),
    'target_family="windows"': dict.fromkeys(ARCHES, True),
    "windows": dict.fromkeys(ARCHES, True),
    "unix": dict.fromkeys(ARCHES, False),
    "test": dict.fromkeys(ARCHES, False),
    "doc": dict.fromkeys(ARCHES, False),
    "miri": dict.fromkeys(ARCHES, False),
}


# cfgs a build.rs emits with `cargo:rustc-cfg=NAME`, spelled as the predicate that
# build script evaluates. Without this they read as free atoms and hal's APIC and PIC
# backends look like they can both link. `unknown_build_cfgs()` fails loud on a new one.
BUILD_CFGS = {
    "hal-rust": {
        # hal-rust/build.rs: amd64 always APIC; i386 APIC iff feature "apic", else PIC.
        "hal_apic": 'any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic"))',
        "hal_pic": 'all(target_arch = "x86", not(feature = "apic"))',
    },
}


def _norm_atom(p: str) -> str:
    return re.sub(r"\s*=\s*", "=", p.strip())


def eval_pred(pred: str, arch: str, free: dict[str, bool]) -> bool:
    p = pred.strip()
    m = re.fullmatch(r"(not|any|all)\s*\((.*)\)", p, re.S)
    if m:
        args = split_args(m.group(2))
        if m.group(1) == "not":
            return not eval_pred(args[0], arch, free) if args else True
        vals = (eval_pred(a, arch, free) for a in args)
        return any(vals) if m.group(1) == "any" else all(vals)
    a = _norm_atom(p)
    m = re.fullmatch(r'target_arch="(\w+)"', a)
    if m:
        return m.group(1) == arch
    if a in _FIXED:
        return _FIXED[a][arch]
    return free.get(a, False)


def feature_atom(crate: str, feat: str) -> str:
    return f'feature="{crate}/{feat}"'


def namespace_features(pred: str, crate: str) -> str:
    """`feature = "x"` means THIS crate's feature x; keep two crates' features apart.
    A cfg this crate's build.rs sets is first replaced by what the script evaluates."""
    for name, meaning in BUILD_CFGS.get(crate, {}).items():
        pred = re.sub(rf"\b{name}\b(?!\s*=)", f"all({meaning})", pred)
    return re.sub(r'\bfeature\s*=\s*"', f'feature="{crate}/', pred)


def free_atoms(pred: str) -> set[str]:
    out: set[str] = set()
    for a in re.findall(r'[A-Za-z_]\w*(?:\s*=\s*"[^"]*")?', pred):
        a = _norm_atom(a)
        if a in ("not", "any", "all") or a in _FIXED or a.startswith("target_arch="):
            continue
        out.add(a)
    return out


@dataclass(frozen=True)
class Cond:
    """Conjunction of cfg predicates (empty = always live)."""

    preds: tuple[str, ...] = ()

    def and_(self, *preds: str) -> Cond:
        return Cond(self.preds + tuple(preds))

    def arches(self) -> set[str]:
        return {a for a in ARCHES if self.satisfiable_with(Cond(), a)}

    def satisfiable_with(
        self, other: Cond, arch: str, fixed: frozenset[tuple[str, bool]] = frozenset()
    ) -> bool:
        """Can both be live in one build of `arch`? `fixed` pins atoms the link unit decides
        (a feature list passed literally by a *-cargo build.rs); all others are free."""
        return _sat(self.preds + other.preds, arch, fixed)


@cache
def _sat(
    preds: tuple[str, ...], arch: str, fixed: frozenset[tuple[str, bool]] = frozenset()
) -> bool:
    pinned = dict(fixed)
    atoms = sorted(set().union(*(free_atoms(p) for p in preds)) - pinned.keys()) if preds else []
    if len(atoms) > 12:  # never seen; don't blow up — treat as satisfiable
        return True
    for bits in itertools.product((False, True), repeat=len(atoms)):
        free = {**pinned, **dict(zip(atoms, bits, strict=True))}
        if all(eval_pred(p, arch, free) for p in preds):
            return True
    return False


# --- crate graph --------------------------------------------------------------------------


@dataclass
class Crate:
    name: str
    dir: Path
    roots: list[Path]
    deps: list[str] = field(default_factory=list)  # package names (path deps only)
    staticlib: bool = False  # crate-type staticlib/cdylib: a link root of its own
    features: dict[str, list[str]] = field(default_factory=dict)  # [features] table
    edges: dict[str, tuple[list[str], bool]] = field(
        default_factory=dict
    )  # dep -> (features, defaults?)
    optional: set[str] = field(default_factory=set)  # deps only a feature turns on


_TOML_TABLE = re.compile(r"^\s*\[(\[)?\s*([^\]]+?)\s*\]\]?\s*(?:#.*)?$")
_TOML_KV = re.compile(r'^\s*([\w."-]+)\s*=\s*(.*)$')
_TOML_STR = re.compile(r'"([^"]*)"')


def _dep_edge(path: str, val: str) -> dict:
    fm = re.search(r"\bfeatures\s*=\s*\[([^\]]*)\]", val)
    return {
        "path": path,
        "features": _TOML_STR.findall(fm.group(1)) if fm else [],
        "default": not re.search(r"default-features\s*=\s*false", val),
        "optional": bool(re.search(r"optional\s*=\s*true", val)),
    }


def _toml(text: str) -> dict:
    """The few Cargo.toml facts we need, without a TOML dependency (the tooling venv is
    3.10: no tomllib). Tables: package/lib (string values), [[bin]] (list), and every
    `path` dependency in [dependencies]/[target.*.dependencies] as `@deps`."""
    data: dict = {"@deps": []}
    cur: dict | None = None
    table = ""
    lines = iter(text.splitlines())
    for line in lines:
        if table == "features" and (m := _TOML_KV.match(line)):
            val = m.group(2)
            while val.lstrip().startswith("[") and "]" not in re.sub(r"#.*", "", val):
                val += " " + next(lines, "]")
            data.setdefault("@features", {})[m.group(1)] = _TOML_STR.findall(
                re.sub(r"#[^\n]*", "", val)
            )
            continue
        if m := _TOML_TABLE.match(line):
            table = m.group(2).strip()
            if m.group(1):
                cur = {}
                data.setdefault(f"[{table}]", []).append(cur)
            else:
                cur = data.setdefault(table, {})
            continue
        m = _TOML_KV.match(line)
        if not m or cur is None:
            continue
        key, val = m.group(1), m.group(2)
        is_dep_table = table == "dependencies" or (
            table.startswith("target.") and table.endswith(".dependencies")
        )
        if is_dep_table and val.lstrip().startswith("{"):
            while "}" not in re.sub(r"#.*", "", val):
                val += " " + next(lines, "}")
        if is_dep_table and "path" in val and (pm := re.search(r'\bpath\s*=\s*"([^"]+)"', val)):
            data["@deps"].append(_dep_edge(pm.group(1), val))
        elif table.startswith("dependencies.") or ".dependencies." in table:
            if key == "path" and (sm := _TOML_STR.match(val.strip())):
                data["@deps"].append(
                    edge := {
                        "path": sm.group(1),
                        "features": [],
                        "default": True,
                        "optional": False,
                    }
                )
                cur["@edge"] = edge
            elif "@edge" in cur and key == "features":
                cur["@edge"]["features"] = _TOML_STR.findall(val)
            elif "@edge" in cur and key == "default-features":
                cur["@edge"]["default"] = "false" not in val
            elif "@edge" in cur and key == "optional":
                cur["@edge"]["optional"] = "true" in val
        elif sm := _TOML_STR.match(val.strip()):
            cur[key] = sm.group(1)
        elif key == "crate-type":
            cur["@crate-type"] = val
    return data


@cache
def crates(root: Path | None = None) -> dict[str, Crate]:
    """Every Cargo package: the tracked ones in the repo, or all under a fixture `root`."""
    import subprocess

    if root is None:
        out = subprocess.run(
            ["git", "-C", str(REPO), "ls-files", "*Cargo.toml"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.split()
        tomls = [REPO / rel for rel in out]
    else:
        tomls = sorted(root.rglob("Cargo.toml"))
    by_dir: dict[Path, Crate] = {}
    raw_deps: dict[Path, list[tuple[Path, dict]]] = {}
    for toml in tomls:
        try:
            data = _toml(toml.read_text())
        except OSError:
            continue
        pkg = data.get("package")
        if not pkg or "name" not in pkg:
            continue
        d = toml.parent
        roots = []
        lib = data.get("lib", {})
        if lib.get("path"):
            roots.append(d / lib["path"])
        elif (d / "src" / "lib.rs").is_file():
            roots.append(d / "src" / "lib.rs")
        for b in data.get("[bin]", []):
            if b.get("path"):
                roots.append(d / b["path"])
        if not data.get("[bin]") and (d / "src" / "main.rs").is_file():
            roots.append(d / "src" / "main.rs")
        deps = [((d / e["path"]).resolve(), e) for e in data.get("@deps", [])]
        c = Crate(pkg["name"], d, [r for r in roots if r.is_file()], staticlib=_is_root_lib(data))
        c.features = data.get("@features", {})
        by_dir[d.resolve()] = c
        raw_deps[d.resolve()] = deps
    for d, deps in raw_deps.items():
        by_dir[d].deps = [by_dir[x].name for x, _ in deps if x in by_dir]
        by_dir[d].edges = {
            by_dir[x].name: (e["features"], e["default"]) for x, e in deps if x in by_dir
        }
        by_dir[d].optional = {by_dir[x].name for x, e in deps if x in by_dir and e.get("optional")}
    return {c.name: c for c in by_dir.values()}


def resolve_features(
    roots: dict[str, tuple[set[str], bool]], maybe: set[str], root: Path | None = None
) -> tuple[dict[str, set[str]], dict[str, set[str]]]:
    """Cargo feature unification over the path-dep graph, for one binary.

    `roots`: crate -> (features requested, default features on?). `maybe`: feature
    names the binary's build script may request conditionally (env, arch) — they are
    UNKNOWN, not off. Returns (on, unknown) per crate; every other feature is off."""
    allc = crates(root)
    on: dict[str, set[str]] = {}
    unknown: dict[str, set[str]] = {}
    defaults: dict[str, bool] = {}
    todo: list[tuple[str, str, bool]] = []  # (crate, feature, certain)

    def enable_crate(c: str, feats: list[str] | set[str], dflt: bool, certain: bool) -> None:
        if c not in allc:
            return
        first = c not in on
        on.setdefault(c, set())
        unknown.setdefault(c, set())
        if dflt and not defaults.get(c):
            defaults[c] = True
            todo.append((c, "default", certain))
        for f in feats:
            todo.append((c, f, certain))
        if first:
            for dep, (df, dd) in allc[c].edges.items():
                if dep not in allc[c].optional:
                    enable_crate(dep, df, dd, certain)
        for f in allc[c].features:
            if f in maybe and f not in on[c]:
                todo.append((c, f, False))

    for c, (feats, dflt) in roots.items():
        enable_crate(c, feats, dflt, True)
    seen: set[tuple[str, str, bool]] = set()
    while todo:
        c, f, certain = item = todo.pop()
        if item in seen:
            continue
        seen.add(item)
        (on if certain else unknown)[c].add(f)
        for sub in allc[c].features.get(f, []):
            if sub.startswith("dep:") or (sub in allc[c].optional and sub not in allc[c].features):
                dep = sub.removeprefix("dep:")
                if dep in allc[c].edges:
                    df, dd = allc[c].edges[dep]
                    enable_crate(dep, df, dd, certain)
                continue
            if "/" in sub:
                dep, df = sub.split("/", 1)
                weak = dep.endswith("?")
                dep = dep.rstrip("?")
                if dep in allc and (not weak or dep in on):
                    if dep in allc[c].edges and dep not in on:
                        enable_crate(dep, *allc[c].edges[dep], certain)
                    else:
                        enable_crate(dep, [], False, certain)
                    todo.append((dep, df, certain))
            elif sub in allc[c].features:
                todo.append((c, sub, certain))
    for c in unknown:
        unknown[c] -= on[c]
    return on, unknown


def _is_root_lib(data: dict) -> bool:
    return any(t in data.get("lib", {}).get("@crate-type", "") for t in ("staticlib", "cdylib"))


def closure(roots: list[str], root: Path | None = None) -> list[str]:
    allc = crates(root)
    seen: list[str] = []
    todo = [r for r in roots if r in allc]
    while todo:
        n = todo.pop()
        if n in seen:
            continue
        seen.append(n)
        todo.extend(allc[n].deps)
    return sorted(seen)


# --- definitions --------------------------------------------------------------------------


@dataclass(frozen=True)
class Def:
    name: str  # as written: Rust item / export_name / asm label
    kind: str  # "rust" | "asm"
    crate: str
    path: Path
    line: int
    cond: Cond
    abi: str = ""  # Rust `extern "<abi>"` ("" for a static or a Rust-ABI fn)

    def link_name(self, arch: str) -> str:
        """The symbol the linker matches, minus i386's stdcall/fastcall `@N` byte count.

        On i386 the decoration is part of the name: cdecl `_X`, stdcall `_X@N` and
        fastcall `@X@N` are three different symbols, and a reference to one never binds
        to another — so they must not be merged. Rust gets its decoration from the ABI
        (LLVM applies it to `#[no_mangle]` and `#[export_name]` alike); an asm label is
        already spelled decorated. `@N` is dropped because computing a Rust signature's
        byte count needs its types — a same-name pair that differs only there is still
        two providers of one API, which is what this is looking for."""
        if arch != "x86":
            return self.name
        n = self.name
        if self.kind == "asm":
            if m := re.fullmatch(r"@(\w+)@\d+", n):
                return f"@{m.group(1)}@"
            if m := re.fullmatch(r"_(\w+)@\d+", n):
                return f"_{m.group(1)}@"
            return n  # `_X` cdecl, or an undecorated label nothing C/Rust can name
        if self.abi == "fastcall":
            return f"@{n}@"
        if self.abi in ("stdcall", "system"):
            return f"_{n}@"
        return f"_{n}"

    @property
    def base(self) -> str:
        """The name with any i386 decoration an asm label is spelled with removed."""
        if self.kind == "asm" and (m := re.fullmatch(r"[_@](\w+?)(?:@\d+)?", self.name)):
            return m.group(1)
        return self.name

    def loc(self) -> str:
        try:
            return f"{self.path.relative_to(REPO)}:{self.line}"
        except ValueError:
            return f"{self.path}:{self.line}"


_ATTR_START = re.compile(r"^\s*#\s*\[")
_CFG_ATTR = re.compile(r"^\s*#\s*\[\s*cfg\s*\((.*)\)\s*\]\s*(?://.*)?$", re.S)
_INNER_CFG = re.compile(r"^\s*#!\s*\[\s*cfg\s*\((.*)\)\s*\]\s*(?://.*)?$", re.S)
_NO_MANGLE = re.compile(r"^\s*#\s*\[\s*(?:unsafe\s*\(\s*)?no_mangle\b")
_EXPORT = re.compile(r'^\s*#\s*\[\s*(?:unsafe\s*\(\s*)?export_name\s*=\s*"([^"]+)"')
_CFG_ATTR_PATH = re.compile(
    r'^\s*#\s*\[\s*cfg_attr\s*\((.*),\s*path\s*=\s*"([^"]+)"\s*\)\s*\]', re.S
)
_PATH_ATTR = re.compile(r'^\s*#\s*\[\s*path\s*=\s*"([^"]+)"\s*\]')
_MOD = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(r#)?(\w+)\s*(;|\{)")
_FN = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:(?:const|unsafe|async|safe)\s+)*"
    r'(?:extern\s*(?:"[^"]*")?\s*)?fn\s+(\w+)'
)
_STATIC = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?static\s+(?:mut\s+)?(\w+)")
_GLOBAL_ASM = re.compile(r"^\s*(?:(?:core|std)\s*::\s*arch\s*::\s*)?global_asm\s*!\s*[\(\{\[]")
_INCLUDE = re.compile(r'^\s*include\s*!\s*\(\s*"([^"]+)"\s*\)')
_INCLUDE_STR = re.compile(r'include_str\s*!\s*\(\s*"([^"]+)"\s*\)')
_GLOBL = re.compile(r'\.glob(?:a)?l\s+"?([@\w$.?]+)"?')


_LEX = re.compile(
    r"(?P<lc>//[^\n]*)"
    r"|(?P<bc>/\*.*?\*/)"
    r'|(?P<raw>\bb?r(?P<h>#*)")'
    r'|(?P<str>b?"(?:\\.|[^"\\])*")'
    r"|(?P<chr>'(?:\\.[^']*|[^\\'\n])')",
    re.S,
)


def _blank(s: str) -> str:
    return re.sub(r"[^\n]", " ", s)


def _mask(text: str) -> str:
    """Blank comments and string/char literal CONTENTS (keeping quotes and newlines),
    so brace counting and line matching never see code inside them."""
    out, pos = [], 0
    while m := _LEX.search(text, pos):
        out.append(text[pos : m.start()])
        k = m.lastgroup if m.lastgroup != "h" else "raw"
        if m.group("raw") is not None:
            end = '"' + m.group("h")
            j = text.find(end, m.end())
            j = len(text) if j < 0 else j
            out.append(m.group("raw") + _blank(text[m.end() : j]) + end)
            pos = j + len(end)
            continue
        tok = m.group(0)
        if k in ("lc", "bc"):
            out.append(_blank(tok))
        else:
            q = tok.index(tok[-1])
            out.append(tok[: q + 1] + _blank(tok[q + 1 : -1]) + tok[-1])
        pos = m.end()
    out.append(text[pos:])
    return "".join(out)


# A file that has none of these cannot define an export or pull in another file.
_INTERESTING = re.compile(r"no_mangle|export_name|global_asm|\bmod\s+\w+\s*;|include!|path\s*=")


def _segments(raw: str, masked: str) -> tuple[list[str], list[str], list[int]]:
    """Split each line into statements so `#[cfg(x)] #[path = "a.rs"] mod a;` and
    `pub mod a; pub mod b;` read as one item per entry. Splits after a line-level
    attribute's closing `]` and after a `;`, both at paren/bracket depth 0. Masking
    preserves offsets, so the raw line is sliced at the same columns."""
    rs, ms, ln = [], [], []
    for n, (r, m) in enumerate(zip(raw.split("\n"), masked.split("\n"), strict=True), 1):
        if "#" not in m and (m.count(";") == 0 or (m.count(";") == 1 and m.rstrip().endswith(";"))):
            rs.append(r)
            ms.append(m)
            ln.append(n)
            continue
        cuts, depth, attr_depth = [], 0, None
        for k, c in enumerate(m):
            if c == "#" and depth == 0 and re.match(r"#!?\[", m[k : k + 3]):
                attr_depth = 0
            if c in "([":
                depth += 1
            elif c in ")]":
                depth -= 1
                if depth == 0 and attr_depth is not None and c == "]":
                    attr_depth = None
                    if m[k + 1 :].strip():
                        cuts.append(k + 1)
            elif c == ";" and depth == 0 and m[k + 1 :].strip():
                cuts.append(k + 1)
        prev = 0
        for cut in [*cuts, len(m)]:
            rs.append(r[prev:cut])
            ms.append(m[prev:cut])
            ln.append(n)
            prev = cut
    return rs, ms, ln


def _attr_span(lines: list[str], i: int) -> tuple[str, int]:
    """Join a (possibly multi-line) `#[...]` starting at line i; return (text, last line)."""
    buf, depth, j = "", 0, i
    while j < len(lines):
        seg = lines[j]
        buf += seg + " "
        depth += seg.count("[") - seg.count("]")
        if depth <= 0:
            break
        j += 1
    return buf, j


class _CrateScan:
    def __init__(self, crate: Crate):
        self.crate = crate
        self.defs: list[Def] = []
        self.seen: set[Path] = set()
        self.lnos: dict[Path, list[int]] = {}

    def ns(self, cond: Cond) -> Cond:
        return Cond(tuple(namespace_features(p, self.crate.name) for p in cond.preds))

    def run(self) -> list[Def]:
        for r in self.crate.roots:
            self.file(r, Cond(), mod_rs=True)
        return self.defs

    def file(self, path: Path, cond: Cond, mod_rs: bool) -> None:
        path = path.resolve()
        if path in self.seen or not path.is_file():
            return
        self.seen.add(path)
        raw = path.read_text(errors="replace")
        if not _INTERESTING.search(raw):
            return  # no module tree, no export: nothing this file adds to the link
        masked = _mask(raw)
        rlines, mlines, lnos = _segments(raw, masked)
        # a file's own child-module dir: foo.rs → foo/, mod.rs/lib.rs → its dir
        child_dir = path.parent if mod_rs else path.parent / path.stem
        self.lnos[path] = lnos
        self.block(path, rlines, mlines, 0, len(mlines), cond, child_dir, top=True)

    def block(self, path, rlines, mlines, start, end, cond, child_dir, top=False):
        lnos = self.lnos[path]
        pending_cfg: list[str] = []
        pending_mangle = False
        export: str | None = None
        path_attr: str | None = None
        alt_paths: list[tuple[str, str]] = []
        i = start
        while i < end:
            ml = mlines[i]
            s = ml.strip()
            if not s:
                i += 1
                continue
            if top and s.startswith("#!["):
                text, j = _attr_span(rlines, i)
                m = _INNER_CFG.match(text)
                if m:
                    cond = cond.and_(m.group(1))
                i = j + 1
                continue
            if _ATTR_START.match(ml):
                text, j = _attr_span(rlines, i)
                if m := _CFG_ATTR.match(text):
                    pending_cfg.append(m.group(1))
                elif _NO_MANGLE.match(text):
                    pending_mangle = True
                elif m := _EXPORT.match(text):
                    export = m.group(1)
                elif m := _PATH_ATTR.match(text):
                    path_attr = m.group(1)
                elif m := _CFG_ATTR_PATH.match(text):
                    alt_paths.append((m.group(1), m.group(2)))
                i = j + 1
                continue
            item_cond = cond.and_(*pending_cfg)
            nxt = i + 1
            if m := _MOD.match(ml):
                name = m.group(2)
                if m.group(3) == ";":
                    for pred, alt in alt_paths:
                        self.file(path.parent / alt, item_cond.and_(pred), mod_rs=True)
                    if alt_paths:
                        pass
                    elif path_attr:
                        base = path.parent if child_dir == path.parent else child_dir
                        target = (path.parent / path_attr) if top else (base / path_attr)
                        self.file(target, item_cond, mod_rs=True)
                    else:
                        f1, f2 = child_dir / f"{name}.rs", child_dir / name / "mod.rs"
                        self.file(f1 if f1.is_file() else f2, item_cond, mod_rs=not f1.is_file())
                else:
                    close = _match_brace(mlines, i)
                    self.block(path, rlines, mlines, i + 1, close, item_cond, child_dir / name)
                    nxt = close + 1
            elif m := _INCLUDE.match(rlines[i]):
                self.file(path.parent / m.group(1), item_cond, mod_rs=False)
            elif _GLOBAL_ASM.match(ml):
                close = _match_paren(mlines, i)
                body = "\n".join(rlines[i : close + 1])
                for inc in _INCLUDE_STR.findall(body):
                    try:
                        body += "\n" + (path.parent / inc).read_text(errors="replace")
                    except OSError:
                        pass
                for lab in _GLOBL.findall(body):
                    self.defs.append(
                        Def(lab, "asm", self.crate.name, path, lnos[i], self.ns(item_cond))
                    )
                nxt = close + 1
            elif pending_mangle or export:
                m = _FN.match(ml) or _STATIC.match(ml)
                if m:
                    abi = re.search(r'\bextern\s*"(\w+)"', rlines[i]) if _FN.match(ml) else None
                    self.defs.append(
                        Def(
                            export or m.group(1),
                            "rust",
                            self.crate.name,
                            path,
                            lnos[i],
                            self.ns(item_cond),
                            abi.group(1) if abi else "",
                        )
                    )
            # skip past a braced item (fn/impl/extern block/macro) so nested
            # attributes inside bodies are never attached to the wrong item
            if nxt == i + 1 and "{" in ml and not _MOD.match(ml):
                close = _match_brace(mlines, i)
                if re.match(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?impl\b", ml):
                    pass  # impl blocks never hold #[no_mangle] items we care about
                nxt = close + 1
            pending_cfg, pending_mangle, export, path_attr, alt_paths = [], False, None, None, []
            i = nxt


def _match_brace(mlines: list[str], i: int) -> int:
    depth, j = 0, i
    started = False
    while j < len(mlines):
        for c in mlines[j]:
            if c == "{":
                depth += 1
                started = True
            elif c == "}":
                depth -= 1
                if started and depth == 0:
                    return j
        if not started and mlines[j].rstrip().endswith(";"):
            return j
        j += 1
    return len(mlines) - 1


def _match_paren(mlines: list[str], i: int) -> int:
    depth, j, started = 0, i, False
    while j < len(mlines):
        for c in mlines[j]:
            if c in "([{":
                depth += 1
                started = True
            elif c in ")]}":
                depth -= 1
                if started and depth == 0:
                    return j
        j += 1
    return len(mlines) - 1


def unknown_build_cfgs(root: Path | None = None) -> dict[str, str]:
    """build.rs-emitted cfg names BUILD_CFGS does not model -> the build.rs that emits it."""
    out = {}
    for c in crates(root).values():
        br = c.dir / "build.rs"
        if br.is_file():
            for n in re.findall(r"rustc-cfg=(\w+)", br.read_text(errors="replace")):
                if n not in BUILD_CFGS.get(c.name, {}):
                    out[n] = str(br)
    return out


@cache
def crate_defs(name: str, root: Path | None = None) -> tuple[Def, ...]:
    c = crates(root).get(name)
    return tuple(_CrateScan(c).run()) if c else ()


@cache
def crate_files(name: str, root: Path | None = None) -> frozenset[Path]:
    c = crates(root).get(name)
    if not c:
        return frozenset()
    s = _CrateScan(c)
    s.run()
    return frozenset(s.seen)
