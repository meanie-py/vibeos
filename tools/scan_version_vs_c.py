#!/usr/bin/env python3
"""
scan_version_vs_c.py — Rust that runs code its C twin's build compiles out by VERSION.

The C decides which Windows version it implements with `#if (NTDDI_VERSION >= NTDDI_WIN7)`
and friends, evaluated at the module's real `_WIN32_WINNT` (0x502, Server 2003, unless its
CMakeLists says otherwise). A translation of the whole function body, both branches
lowered as ordinary code, runs the Win7/Win8 path on a WS03 driver: oplock keys,
`FsRtlChangeBackingFileObject`, the Win8 sector-size query. It builds and boots, and
nothing notices. fastfat had about 40 such sites (b8eac0dccad); `scan_cfg_vs_c` cannot see
them, because it judges arch conditionals only. `#ifndef __REACTOS__` blocks are the same
class (the C build always defines `__REACTOS__`) and are judged here too.

HOW.
  1. Each tracked C file with a version conditional inside a function: resolve the module's
     `_WIN32_WINNT` / `NTDDI_VERSION` from the source (module_version): the nearest
     CMakeLists.txt / *.cmake up the tree that sets them, else the root default. NTDDI
     follows sdkddkver.h (0x502 -> NTDDI_WS03SP1, otherwise `_WIN32_WINNT << 16`).
  2. Evaluate every `#if`/`#elif`/`#else` chain whose conditions are made only of version
     symbols, numbers and `defined(__REACTOS__)` / `defined(<version symbol>)`. A chain with
     anything else in it (DBG, an arch macro, a feature flag) is UNKNOWN from that branch
     on, and never judged: this tool never guesses.
  3. DEAD-ONLY names: identifiers called in a dead branch that the function's live text
     never mentions at all.
  4. The Rust function of the same name (tools/c_twin.py, same module), with every region
     gated on a version cfg masked out (`#[cfg(..ntddi..)]` / `#[cfg(..reactos..)]` items
     and statements, and `if cfg!(..)` chains naming one). A dead-only name still CALLED in
     what is left is the finding: the Rust runs a branch the C compiles out.

What it cannot see: a dead branch that calls nothing the live text does not (pure
arithmetic, an extra flag test), and a Rust body that inlined the dead branch's callee.
Those need a human; the fastfat sweep found most of its sites by the call they made.

Usage:
    python3 tools/scan_version_vs_c.py [paths]   # report
    python3 tools/scan_version_vs_c.py --lint    # terse, for the baseline gate
    python3 tools/scan_version_vs_c.py --stats   # how the conditionals split
    python3 tools/scan_version_vs_c.py --self-test

Suppress a reviewed site with `// nolint: version-vs-c` on the Rust fn's line. Exit 1 if
any finding.
"""

from __future__ import annotations

import re
import subprocess
import sys
from functools import cache, lru_cache
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.c_twin import (
    CIndex,
    Func,
    blank_c,
    blank_rust,
    c_functions,
    defined_names,
    match_brace,
    prefer,
    rust_functions,
    same_module,
)
from tools.source_walk import source_files

try:
    import tomllib  # type: ignore[reportMissingImports]  # Python 3.11+ stdlib
except ImportError:  # Python 3.10 in the repository tooling environment.
    import tomli as tomllib

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
SDKDDKVER = REPO / "sdk" / "include" / "psdk" / "sdkddkver.h"
NOLINT = "nolint: version-vs-c"

VERSION_SYMS = ("NTDDI_VERSION", "_WIN32_WINNT", "WINVER")
# Defined by every C build in the tree (sdk/cmake/config.cmake adds -D__REACTOS__).
ALWAYS_DEFINED = ("__REACTOS__",)

_C_KEYWORDS = {
    "if", "while", "for", "switch", "return", "sizeof", "do", "else", "case", "defined",
    "goto", "break", "continue", "typedef", "struct", "union", "enum",
}  # fmt: skip


@lru_cache(maxsize=1)
def version_constants() -> dict[str, int]:
    """Every `NTDDI_*` / `_WIN32_WINNT_*` constant sdkddkver.h defines, aliases resolved."""
    raw: dict[str, str] = {}
    for m in re.finditer(
        r"^\s*#\s*define\s+((?:NTDDI|_WIN32_WINNT)_\w+)\s+(0x[0-9A-Fa-f]+|\w+)",
        SDKDDKVER.read_text(errors="replace"),
        re.M,
    ):
        raw.setdefault(m.group(1), m.group(2))
    out: dict[str, int] = {}
    for name in raw:
        v = raw[name]
        for _ in range(8):
            if v in raw and v != name:
                v = raw[v]
        if re.fullmatch(r"0x[0-9A-Fa-f]+", v):
            out[name] = int(v, 16)
    return out


_DEF = re.compile(
    r"(?:add_definitions|add_compile_definitions|target_compile_definitions)\s*\(([^)]*)\)", re.S
)
_REMOVE = re.compile(r"remove_definitions\s*\(([^)]*)\)", re.S)


def _hexval(s: str) -> int | None:
    m = re.fullmatch(r"(0x[0-9A-Fa-f]+|\d+)[uUlL]*", s.strip())
    return int(m.group(1), 0) if m else None


def _parse_override(
    texts: list[tuple[str, str]],
) -> tuple[int | None, int | None, bool, str] | None:
    """(winnt, ntddi, removed-only, source) the CMake `texts` [(path, text)] set, or None."""
    winnt = ntddi = None
    removed = False
    src = ""
    for where, text in texts:
        text = re.sub(r"#[^\n]*", "", text)
        for m in _DEF.finditer(text):
            w = re.search(r"_WIN32_WINNT=(\w+)", m.group(1))
            n = re.search(r"NTDDI_VERSION=(\w+)", m.group(1))
            if w and _hexval(w.group(1)) is not None:
                winnt, src = _hexval(w.group(1)), where
            if n and _hexval(n.group(1)) is not None:
                ntddi, src = _hexval(n.group(1)), where
        if any("_WIN32_WINNT" in m.group(1) for m in _REMOVE.finditer(text)):
            removed, src = True, src or where
    if winnt is None and ntddi is None and not removed:
        return None
    return winnt, ntddi, removed and winnt is None, src


@cache
def _dir_override(d: Path) -> tuple[int | None, int | None, bool, str] | None:
    """What a directory's CMake sets today."""
    files = [d / "CMakeLists.txt", *sorted(d.glob("*.cmake"))]
    return _parse_override(
        [
            (f.relative_to(REPO).as_posix(), f.read_text(errors="replace"))
            for f in files
            if f.is_file()
        ]
    )


@lru_cache(maxsize=1)
def _ever_versioned() -> frozenset[str]:
    """CMakeLists.txt paths whose history ever mentions `_WIN32_WINNT` (one `git log`)."""
    proc = subprocess.run(
        ["git", "-C", str(REPO), "log", "--format=", "--name-only", "-S_WIN32_WINNT", "--",
         "*CMakeLists.txt"],
        capture_output=True, text=True, check=False,
    )  # fmt: skip
    return frozenset(proc.stdout.split()) if proc.returncode == 0 else frozenset()


@cache
def _historical_override(d: Path) -> tuple[int | None, int | None, bool, str] | None:
    """What a directory's CMakeLists set before a cutover rewrote it. The cargo-link cutovers
    replaced whole CMakeLists files, and the module's version went with them: classpnp was
    `_WIN32_WINNT=0x602 NTDDI_VERSION=0x06020000` until 7a67eeddf88, and its translation
    still implements exactly that build."""
    rel = (d / "CMakeLists.txt").relative_to(REPO).as_posix()
    if rel not in _ever_versioned():
        return None
    last = subprocess.run(
        ["git", "-C", str(REPO), "log", "-1", "--format=%h", "-S_WIN32_WINNT", "--", rel],
        capture_output=True, text=True, check=False,
    ).stdout.strip()  # fmt: skip
    if not last:
        return None
    old = subprocess.run(
        ["git", "-C", str(REPO), "show", f"{last}^:{rel}"],
        capture_output=True,
        text=True,
        check=False,
    )
    if old.returncode != 0:
        return None
    o = _parse_override([(f"{rel}@{last}^", old.stdout)])
    return o


def root_default() -> int:
    """The tree-wide `-D_WIN32_WINNT=` the root CMakeLists.txt passes."""
    m = re.search(r"-D_WIN32_WINNT=(0x[0-9A-Fa-f]+)", (REPO / "CMakeLists.txt").read_text())
    if not m:
        raise SystemExit("scan_version_vs_c: no -D_WIN32_WINNT in the root CMakeLists.txt")
    return int(m.group(1), 16)


@lru_cache(maxsize=1)
def _ninja_versions() -> tuple[dict[str, set[tuple[int, int | None]]], str]:
    """({repo-relative source: (_WIN32_WINNT, NTDDI_VERSION or None)}, build dir) from THIS
    worktree's own build.ninja: the defines each object was really compiled with. Never a
    sibling worktree's (generated, per-commit state). Empty when no build dir exists."""
    for arch in ("amd64", "i386", "arm64"):
        ninja = REPO / f"build-{arch}" / "build.ninja"
        if not ninja.is_file():
            continue
        out: dict[str, set[tuple[int, int | None]]] = {}
        cur = None
        with ninja.open(errors="replace") as fh:
            for line in fh:
                m = re.match(
                    r"build \S+\.(?:c|cpp)\.obj: C(?:XX)?_COMPILER\S* /tmp/vibeos/(\S+?\.(?:c|cpp))\b",  # noqa: E501
                    line,
                )
                if m:
                    cur = m.group(1)
                elif cur and line.startswith("  DEFINES ="):
                    w = re.findall(r"-D_WIN32_WINNT=(\w+)", line)
                    n = re.findall(r"-DNTDDI_VERSION=(\w+)", line)
                    # No -D_WIN32_WINNT at all: sdkddkver.h's own default applies.
                    v = (_hexval(w[-1]) or 0x600 if w else 0x600, _hexval(n[-1]) if n else None)
                    out.setdefault(cur, set()).add(v)
                    cur = None
        return out, f"build-{arch}/build.ninja"
    return {}, ""


def _env(winnt: int, ntddi: int | None) -> dict[str, int]:
    if ntddi is None:
        ntddi = version_constants()["NTDDI_WS03SP1"] if winnt == 0x502 else winnt << 16
    return {"NTDDI_VERSION": ntddi, "_WIN32_WINNT": winnt, "WINVER": winnt}


def module_version(c_path: Path) -> tuple[dict[str, int] | None, str]:
    """({NTDDI_VERSION, _WIN32_WINNT, WINVER}, where it came from) for one C file, or None
    when targets compile the file at different versions (dmusic's band.c): never guessed.

    This file's own object in the build first (measured 2,743 of 2,753 objects agreeing with
    the CMake reading below, 2026-10-16; the rest were `add_subdirectory` order and
    per-target defines a source walk cannot see). A cohab-flipped `.c` is not compiled, so
    then the nearest directory whose CMake sets a version, today or before a cutover, then
    the objects its directory still compiles (its shims), then the root default."""
    versions, ninja = _ninja_versions()
    try:
        rel = c_path.resolve().relative_to(REPO).as_posix()
    except ValueError:
        rel = c_path.as_posix()
    if rel in versions:
        if len(versions[rel]) > 1:
            return None, f"{ninja} ({rel}: {len(versions[rel])} targets, different versions)"
        return _env(*next(iter(versions[rel]))), f"{ninja} ({rel})"
    # Not compiled any more (cohab-flipped): what its directory's CMake says, today or
    # before a cutover rewrote it, nearest directory first; the root's default comes last.
    d = c_path.resolve().parent
    while d != REPO and REPO in d.parents:
        o = _dir_override(d) or _historical_override(d)
        if o is not None:
            w, n, removed_only, where = o
            if removed_only:
                return _env(0x600, None), f"{where} (removed; sdkddkver.h default)"
            return _env(w if w is not None else (n or 0) >> 16, n), where
        d = d.parent
    dprefix = rel.rsplit("/", 1)[0] + "/"
    same_dir = [
        x
        for k, v in versions.items()
        if k.startswith(dprefix) and "/" not in k[len(dprefix) :]
        for x in v
    ]
    if same_dir:
        best = max(set(same_dir), key=same_dir.count)
        return _env(*best), f"{ninja} ({dprefix}*, {same_dir.count(best)}/{len(same_dir)} objects)"
    return _env(root_default(), None), "CMakeLists.txt (root default)"


# --- the preprocessor-expression evaluator ---------------------------------------------

_TOK = re.compile(r"\s*(0x[0-9A-Fa-f]+[uUlL]*|\d+[uUlL]*|[A-Za-z_]\w*|&&|\|\||[<>=!]=|[<>!()])")


class Unknown(Exception):
    pass


def evaluate(expr: str, env: dict[str, int]) -> bool | None:
    """A version-only `#if` expression's truth at `env`, or None if it is not version-only
    (names anything but version symbols, sdkddkver constants and `defined(__REACTOS__)`)."""
    expr = re.sub(r"/\*.*?\*/|//.*$", "", expr).strip()
    toks: list[str] = []
    pos = 0
    while pos < len(expr):
        m = _TOK.match(expr, pos)
        if not m:
            return None
        toks.append(m.group(1))
        pos = m.end()
        if not expr[pos:].strip():
            break
    consts = version_constants()
    mentions = False
    i = 0

    def peek() -> str | None:
        return toks[i] if i < len(toks) else None

    def take() -> str:
        nonlocal i
        if i >= len(toks):
            raise Unknown
        i += 1
        return toks[i - 1]

    def primary() -> int:
        nonlocal mentions
        t = take()
        if t == "!":
            return int(not primary())
        if t == "(":
            v = orx()
            if take() != ")":
                raise Unknown
            return v
        if t == "defined":
            paren = peek() == "("
            if paren:
                take()
            name = take()
            if paren and take() != ")":
                raise Unknown
            if name in ALWAYS_DEFINED or name in VERSION_SYMS:
                mentions = True
                return 1
            raise Unknown
        if (v := _hexval(t)) is not None:
            return v
        if t in env:
            mentions = True
            return env[t]
        if t in consts:
            return consts[t]
        raise Unknown

    def cmp() -> int:
        v = primary()
        while peek() in ("<", ">", "<=", ">=", "==", "!="):
            op = take()
            w = primary()
            v = int(
                {"<": v < w, ">": v > w, "<=": v <= w, ">=": v >= w, "==": v == w, "!=": v != w}[op]
            )
        return v

    def andx() -> int:
        v = cmp()
        while peek() == "&&":
            take()
            w = cmp()
            v = int(bool(v) and bool(w))
        return v

    def orx() -> int:
        v = andx()
        while peek() == "||":
            take()
            w = andx()
            v = int(bool(v) or bool(w))
        return v

    try:
        v = orx()
    except Unknown:
        return None
    if i != len(toks) or not mentions:
        return None
    return bool(v)


_PP = re.compile(r"^[ \t]*#[ \t]*(if|ifdef|ifndef|elif|else|endif)\b([^\n]*)$", re.M)


def dead_ranges(body: str, env: dict[str, int]) -> tuple[list[tuple[int, int, str]], int]:
    """([(start, end, directive)] of every branch compiled out at `env`, number of chains
    judged). Directive lines themselves are not in any range."""
    dead: list[tuple[int, int, str]] = []
    judged = 0
    # Each open chain: [decided: bool|None (a branch already taken / unknown), cur_alive,
    #                   cur_start, cur_directive, any_version]
    stack: list[list] = []
    for m in _PP.finditer(body):
        kind, rest = m.group(1), m.group(2)
        if kind in ("if", "ifdef", "ifndef"):
            if kind == "ifdef":
                v = evaluate(f"defined({rest.split()[0]})", env) if rest.split() else None
            elif kind == "ifndef":
                v = evaluate(f"!defined({rest.split()[0]})", env) if rest.split() else None
            else:
                v = evaluate(rest, env)
            # taken: True once a branch was alive; None once the chain became unknown.
            stack.append(
                [v if v is not None else None, v, m.end(), f"#{kind}{rest}", v is not None]
            )
            continue
        if not stack:
            continue
        top = stack[-1]
        taken, alive, start, directive, known = top
        if alive is False:
            dead.append((start, m.start(), directive.strip()))
        if kind == "endif":
            judged += 1 if known else 0
            stack.pop()
            continue
        if kind == "elif":
            v = evaluate(rest, env) if taken is not None else None
            if taken is None or v is None:
                top[:] = [None, None, m.end(), f"#elif{rest}", known]
            elif taken:
                top[:] = [True, False, m.end(), f"#elif{rest}", True]
            else:
                top[:] = [v, v, m.end(), f"#elif{rest}", True]
        else:  # else
            if taken is None:
                top[:] = [None, None, m.end(), "#else", known]
            else:
                top[:] = [True, not taken, m.end(), "#else", known]
    return dead, judged


_CALL = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
_IDENT = re.compile(r"\b[A-Za-z_]\w*\b")


def dead_only_calls(body: str, env: dict[str, int]) -> tuple[dict[str, str], int]:
    """({name called only in a dead branch: that branch's directive}, chains judged)."""
    ranges, judged = dead_ranges(body, env)
    if not ranges:
        return {}, judged
    blanked = blank_c(body)
    keep = list(blanked)
    for s, e, _ in ranges:
        for k in range(s, e):
            if keep[k] != "\n":
                keep[k] = " "
    live = _PP.sub("", "".join(keep))
    live_idents = set(_IDENT.findall(live))
    out: dict[str, str] = {}
    for s, e, directive in ranges:
        for name in _CALL.findall(blanked[s:e]):
            if name not in live_idents and name not in _C_KEYWORDS:
                out.setdefault(name, directive)
    return out, judged


# --- the Rust side ----------------------------------------------------------------------

_CFG_ATTR = re.compile(r"#\[cfg\(")
_CFG_MACRO_IF = re.compile(r"\bif\s+(!\s*)?cfg!\s*\(")
_CFG_MACRO = re.compile(r"(!\s*)?\bcfg!\s*\(")
# `if false {`, `if REACTOS_BUILD {`, `if !__REACTOS__ {`: a constant condition (the name an
# immutable `const`/`let` bool literal in the same file -- wscript's and comctl32's spelling).
_BOOL_IF = re.compile(r"\bif\s+(!\s*)?(\w+)\s*\{")
_BOOL_CONST = re.compile(r"\b(?:const|let)\s+(\w+)\s*:\s*bool\s*=\s*(true|false)\s*;")
# Built-in cfgs whose value this tool cannot know for the shipped build.
_BUILTIN_CFGS = {"test", "debug_assertions", "doc", "doctest", "miri", "unix", "windows",
                 "panic", "overflow_checks", "rustfmt", "clippy"}  # fmt: skip


class CrateCfg:
    """What a crate's `#[cfg]`s can be decided against: the features the shipped build turns
    on (its defaults, plus whatever any workspace crate requests of it), the features it
    declares, and the cfgs its build script emits (whose value is unknown here)."""

    def __init__(self, on: set[str], declared: set[str], emitted: set[str]):
        self.on, self.declared, self.emitted = on, declared, emitted


@lru_cache(maxsize=1)
def _requested_features() -> dict[str, set[str]]:
    """crate name -> features some workspace manifest asks of it as a dependency."""
    out: dict[str, set[str]] = {}
    for toml in RUST_ROOT.glob("*/Cargo.toml"):
        try:
            data = tomllib.loads(toml.read_text(errors="replace"))
        except (tomllib.TOMLDecodeError, OSError):
            continue
        tables = [data.get("dependencies", {}), data.get("build-dependencies", {})]
        tables += [t.get("dependencies", {}) for t in data.get("target", {}).values()]
        for deps in tables:
            for name, spec in deps.items():
                if isinstance(spec, dict):
                    out.setdefault(spec.get("package", name), set()).update(
                        spec.get("features", [])
                    )
    return out


@cache
def crate_cfg(crate_dir: Path) -> CrateCfg:
    manifest = crate_dir / "Cargo.toml"
    try:
        data = tomllib.loads(manifest.read_text(errors="replace"))
    except (tomllib.TOMLDecodeError, OSError):
        data = {}
    feats: dict[str, list[str]] = data.get("features", {})
    name = data.get("package", {}).get("name", crate_dir.name)
    on: set[str] = set()
    todo = list(feats.get("default", [])) + sorted(_requested_features().get(name, ()))
    while todo:
        f = todo.pop()
        if "/" in f or f.startswith("dep:") or f in on:
            continue
        on.add(f)
        todo.extend(feats.get(f, []))
    build = crate_dir / "build.rs"
    emitted = (
        set(re.findall(r"rustc-cfg=(\w+)", build.read_text(errors="replace")))
        if build.is_file()
        else set()
    )
    return CrateCfg(on, set(feats) - {"default"}, emitted)


def cfg_value(pred: str, env: CrateCfg) -> bool | None:
    """A `cfg(..)` predicate's value in the shipped build, or None if this tool cannot know.
    `feature = "x"`: on iff the build enables it. A bare custom cfg nothing emits
    (`ntddi_ge_ntddi_win7`, `not_reactos`): never set, so false."""
    toks = re.findall(r'[A-Za-z_]\w*|"[^"]*"|[(),=]', pred)
    i = 0

    def take() -> str:
        nonlocal i
        if i >= len(toks):
            raise Unknown
        i += 1
        return toks[i - 1]

    def item() -> bool | None:
        t = take()
        if t in ("any", "all", "not") and i < len(toks) and toks[i] == "(":
            take()
            vals: list[bool | None] = []
            while toks[i] != ")":
                vals.append(item())
                if toks[i] == ",":
                    take()
            take()
            if t == "not":
                return None if len(vals) != 1 or vals[0] is None else not vals[0]
            if t == "any":
                return True if True in vals else (None if None in vals else False)
            return False if False in vals else (None if None in vals else True)
        if i < len(toks) and toks[i] == "=":
            take()
            value = take().strip('"')
            if t == "feature":
                return value in env.on
            return None  # target_arch, target_os, ...: not this tool's question
        if t in _BUILTIN_CFGS or t in env.emitted:
            return None
        return False

    try:
        v = item()
    except (Unknown, IndexError):
        return None
    return v if i == len(toks) else None


def _paren_end(text: str, open_at: int) -> int:
    depth = 0
    for k in range(open_at, len(text)):
        if text[k] == "(":
            depth += 1
        elif text[k] == ")":
            depth -= 1
            if depth == 0:
                return k
    return -1


def _stmt_end(text: str, at: int) -> int:
    """End of the item/statement starting at `at`: through its first `{..}` block (and a
    trailing `else ..` chain), or its `;`, whichever comes first at depth 0."""
    k = at
    depth = 0
    while k < len(text):
        c = text[k]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif c == ";" and depth == 0:
            return k + 1
        elif c == "{" and depth == 0:
            end = match_brace(text, k)
            if end < 0:
                return len(text)
            nxt = re.match(r"\s*else\b", text[end:])
            if nxt:
                k = end + nxt.end()
                continue
            return end
        k += 1
    return len(text)


def _expr_end(text: str, at: int, op: str) -> int:
    """End of the operand chain right of `op` at `at`: up to the enclosing `)`, or a `{`,
    `;` or `,` at depth 0; for `&&`, also a `||` at depth 0 (which binds looser)."""
    depth = 0
    k = at
    while k < len(text):
        c = text[k]
        if c in "([":
            depth += 1
        elif c in ")]":
            if depth == 0:
                return k
            depth -= 1
        elif depth == 0 and (c in "{;," or (op == "&&" and text.startswith("||", k))):
            return k
        k += 1
    return k


def mask_gated(
    body: str,
    raw: str | None = None,
    env: CrateCfg | None = None,
    consts: dict[str, bool] | None = None,
) -> str:
    """`body` (blanked Rust) with every region the shipped build compiles out blanked.
    `raw` is the same span unblanked (cfg predicates are string literals, which blanking
    empties); a predicate whose value is unknown masks nothing."""
    raw = body if raw is None else raw
    env = env or CrateCfg(set(), set(), set())
    spans: list[tuple[int, int]] = []

    def off(open_paren: int) -> tuple[int, bool]:
        close = _paren_end(body, open_paren)
        return close, close >= 0 and cfg_value(raw[open_paren + 1 : close], env) is False

    for m in _CFG_ATTR.finditer(body):
        close, dead = off(m.end() - 1)
        if dead:
            spans.append((m.start(), _stmt_end(body, body.find("]", close) + 1)))
    for m in _CFG_MACRO_IF.finditer(body):
        close, dead = off(m.end() - 1)
        # Only when `cfg!(..)` IS the condition; a compound one is the operand rule's below.
        if not dead or not re.match(r"\s*\{", body[close + 1 :]):
            continue
        then_open = body.index("{", close)
        then_end = match_brace(body, then_open)
        if not m.group(1):  # `if cfg!(off) { A }`: A never runs
            spans.append((m.start(), then_end))
        else:  # `if !cfg!(off) { A } else { B }`: B never runs
            spans.append((then_end, _stmt_end(body, then_end)))
    known = {"true": True, "false": False, **(consts or {})}
    for m in _BOOL_IF.finditer(body):
        value = known.get(m.group(2))
        if value is None:
            continue
        then_open = m.end() - 1
        then_end = match_brace(body, then_open)
        if then_end < 0:
            continue
        if value != bool(m.group(1)):  # the then-block runs; an `else` never does
            spans.append((then_end, _stmt_end(body, then_end)))
        else:
            spans.append((m.start(), then_end))
    # `cfg!(off) && rest` and `!cfg!(off) || rest`: `rest` is never evaluated.
    for m in _CFG_MACRO.finditer(body):
        close, dead = off(m.end() - 1)
        if not dead:
            continue
        after = re.match(r"\s*(&&|\|\|)", body[close + 1 :])
        if not after or (after.group(1) == "||") != bool(m.group(1)):
            continue
        end = _expr_end(body, close + 1 + after.end(), after.group(1))
        # `if cfg!(off) && a && b { .. }`: the leading conjunct gates the block too.
        if (
            after.group(1) == "&&"
            and not m.group(1)
            and re.search(r"\bif\s*$", body[: m.start()])
            and body.startswith("{", end)
        ):
            end = match_brace(body, end)
        spans.append((m.start(), end if end > 0 else len(body)))
    if not spans:
        return body
    out = list(body)
    for s, e in spans:
        for k in range(s, min(e, len(out))):
            if out[k] != "\n":
                out[k] = " "
    return "".join(out)


# --- driver -----------------------------------------------------------------------------


def item_compiled_out(lines: list[str], fn_line: int, env: CrateCfg) -> bool:
    """The fn's own attributes (the lines just above its name) carry a cfg that is off:
    classpnp's Win10 firmware handlers are `#[cfg(any())]` whole."""
    k = fn_line - 2
    while k >= 0:
        t = lines[k].strip()
        if t.startswith("#[cfg("):
            m = re.match(r"#\[cfg\((.*)\)\]\s*$", t)
            if m and cfg_value(m.group(1), env) is False:
                return True
        elif not (t.startswith("#[") or t.startswith("///") or t.startswith("//")):
            break
        k -= 1
    return False


def _crate_dir(path: Path) -> Path:
    for parent in path.parents:
        if (parent / "Cargo.toml").is_file():
            return parent
    return path.parent


def c_candidates(repo: Path = REPO) -> dict[str, list[tuple[Func, dict[str, str], str]]]:
    """name -> [(C function, {dead-only call: directive}, version source)]."""
    proc = subprocess.run(
        ["git", "-C", str(repo), "grep", "-lE", r"NTDDI_VERSION|_WIN32_WINNT|__REACTOS__",
         "--", "*.c", "*.cpp", ":!sdk/lib/rust", ":!modules/rostests"],
        capture_output=True, text=True, check=False,
    )  # fmt: skip
    if proc.returncode not in (0, 1):
        raise SystemExit(f"scan_version_vs_c: git grep failed: {proc.stderr.strip()}")
    out: dict[str, list[tuple[Func, dict[str, str], str]]] = {}
    STATS["c_files"] = 0
    for rel in proc.stdout.split():
        path = repo / rel
        raw = path.read_text(errors="replace")
        if not re.search(r"^[ \t]*#[ \t]*(?:el)?if", raw, re.M):
            continue
        env, src = module_version(path)
        if env is None:
            STATS["ambiguous_version"] += 1
            continue
        blanked = blank_c(raw)
        names = set(defined_names(blanked))
        if not names:
            continue
        STATS["c_files"] += 1
        for f in c_functions(blanked, path, names):
            body = raw[f.body_start : f.body_start + len(f.body)]
            if "#" not in body:
                continue
            calls, judged = dead_only_calls(body, env)
            STATS["chains"] += judged
            if calls:
                STATS["c_fns_with_dead"] += 1
                out.setdefault(f.name, []).append((f, calls, src))
    return out


STATS: dict[str, int] = {
    "c_files": 0, "chains": 0, "c_fns_with_dead": 0, "rust_twins": 0, "ambiguous_version": 0,
    "fn_compiled_out": 0,
}  # fmt: skip


def scan(roots: list[Path]) -> list[tuple[str, int, str, str, str, list[str], str]]:
    """(rust relpath, line, fn, C twin path:line, dead directive, [names still called],
    where the module's version came from)."""
    cands = c_candidates()
    index = CIndex()
    index.load(set(cands))
    findings = []
    files = [f for r in roots for f in ([r] if r.is_file() else source_files(r))]
    wanted = re.compile(r"\bfn\s+(?:" + "|".join(map(re.escape, sorted(cands))) + r")\b")
    for f in files:
        if re.search(r"bindgen_\w+\.rs$", f.name):
            continue  # layouts and declarations, no bodies; most of the bytes in the tree
        raw = f.read_text(errors="replace")
        if not wanted.search(raw):
            continue
        blanked = blank_rust(raw)
        lines = raw.split("\n")
        consts = {k: v == "true" for k, v in _BOOL_CONST.findall(raw)}
        for fn in rust_functions(blanked, f):
            if fn.name not in cands:
                continue
            # Rank against EVERY C definition, not just the ones with a version block: amd64's
            # KiGetCacheInformation would otherwise be judged against ke/i386/cpu.c.
            twins = [t for t in index.get(fn.name) if same_module(f, t)]
            if not twins:
                continue
            best = prefer(f, twins)[0]
            twin = next(
                (t for t, _, _ in cands[fn.name] if t.path == best.path and t.line == best.line),
                None,
            )
            if twin is None:
                continue
            STATS["rust_twins"] += 1
            if NOLINT in lines[fn.line - 1]:
                continue
            env = crate_cfg(_crate_dir(f))
            if item_compiled_out(lines, fn.line, env):
                STATS["fn_compiled_out"] += 1
                continue
            calls, src = next((c, w) for t, c, w in cands[fn.name] if t is twin)
            raw_body = raw[fn.body_start : fn.body_start + len(fn.body)]
            live = mask_gated(fn.body, raw_body, env, consts)
            hit = sorted(n for n in calls if re.search(rf"\b{re.escape(n)}\s*!?\s*\(", live))
            if not hit:
                continue
            try:
                rel = f.relative_to(REPO).as_posix()
            except ValueError:
                rel = f.as_posix()
            directive = calls[hit[0]]
            findings.append(
                (
                    rel,
                    fn.line,
                    fn.name,
                    f"{twin.path.relative_to(REPO)}:{twin.line}",
                    directive,
                    hit,
                    src,
                )
            )
    return sorted(findings)


SELF_TEST_C = """
VOID FatSetup(PIRP Irp)
{
    Prep(Irp);
#if (NTDDI_VERSION >= NTDDI_WIN7)
    FsRtlChangeBackingFileObject(Irp);
#else
    OldPath(Irp);
#endif
#ifndef __REACTOS__
    NotOnReactos(Irp);
#endif
#if DBG
    DbgOnly(Irp);
#endif
#if (NTDDI_VERSION >= NTDDI_WIN7) && defined(_M_AMD64)
    Unknowable(Irp);
#endif
}
"""


def self_test() -> int:
    ws03 = {"NTDDI_VERSION": 0x05020100, "_WIN32_WINNT": 0x502, "WINVER": 0x502}
    win7 = {"NTDDI_VERSION": 0x06010000, "_WIN32_WINNT": 0x601, "WINVER": 0x601}
    checks = [
        ("WS03 compiles the WIN7 branch out", evaluate("(NTDDI_VERSION >= NTDDI_WIN7)", ws03), False),  # noqa: E501
        ("WIN7 keeps it", evaluate("NTDDI_VERSION >= NTDDI_WIN7", win7), True),
        ("an arch macro makes it unknowable", evaluate("NTDDI_VERSION >= NTDDI_WIN7 && _M_AMD64", ws03), None),  # noqa: E501
        ("a bare constant is not a version test", evaluate("1", ws03), None),
        ("!defined(__REACTOS__) is dead", evaluate("!defined(__REACTOS__)", ws03), False),
        ("! binds tighter than ==", evaluate("!defined(__REACTOS__) == 0", ws03), True),
    ]  # fmt: skip
    calls, judged = dead_only_calls(SELF_TEST_C[SELF_TEST_C.index("{") :], ws03)
    checks += [
        ("dead-only calls: the WIN7 branch and the non-ReactOS one", sorted(calls),
         ["FsRtlChangeBackingFileObject", "NotOnReactos"]),
        ("the DBG and arch chains are never judged", judged, 2),
        ("at WIN7 the #else is the dead one", sorted(dead_only_calls(SELF_TEST_C[SELF_TEST_C.index("{") :], win7)[0]),  # noqa: E501
         ["NotOnReactos", "OldPath"]),
    ]  # fmt: skip
    rust_ungated = "{ Prep(Irp); FsRtlChangeBackingFileObject(Irp); }"
    rust_gated = (
        "{ Prep(Irp);\n #[cfg(ntddi_ge_ntddi_win7)]\n { FsRtlChangeBackingFileObject(Irp); }\n"
        " if cfg!(not_reactos) { NotOnReactos(Irp); } else { x(); }\n Prep(Irp); }"
    )
    for desc, body, want in [
        ("ungated Rust still calls it", rust_ungated, True),
        ("a cfg'd block and an `if cfg!` chain are masked", rust_gated, False),
    ]:
        live = mask_gated(body)
        got = bool(re.search(r"\b(FsRtlChangeBackingFileObject|NotOnReactos)\s*\(", live))
        checks.append((desc, got, want))
    checks.append(
        ("masking keeps what follows the gate", "Prep(Irp); }" in mask_gated(rust_gated), True)
    )
    for desc, body, want in [
        ("`cfg!(g) && call` short-circuits the call away",
         "{ if a || (cfg!(ntddi_ge_ntddi_win7) && FsRtlChangeBackingFileObject(x) != 0) { y(); } }", False),  # noqa: E501
        ("`!cfg!(g) || call` too",
         "{ if !cfg!(ntddi_ge_ntddi_win8) || b || (c && FsRtlChangeBackingFileObject(x)) { y(); } }", False),  # noqa: E501
        ("but `!cfg!(g) && call` runs the call on the old build",
         "{ if !cfg!(ntddi_ge_ntddi_win8) && FsRtlChangeBackingFileObject(x) { y(); } }", True),
        ("an eager `let` before the gate still calls it (fastfat lockctrl)",
         "{ let w = FsRtlChangeBackingFileObject(x);\n if !cfg!(ntddi_ge_ntddi_win8) || w { y(); } }", True),  # noqa: E501
        ("`if cfg!(g) && a { call }` gates the block (fastfat close.rs)",
         "{ if cfg!(ntddi_ge_ntddi_win8)\n && !f.is_null()\n {\n FsRtlChangeBackingFileObject(x);\n }\n y(); }", False),  # noqa: E501
        ("`if a && cfg!(g) { call }` is not the leading-conjunct form; the block is judged",
         "{ if a && cfg!(ntddi_ge_ntddi_win8) { FsRtlChangeBackingFileObject(x); } }", True),
        ("`#[cfg(any())]` is never compiled",
         "{ #[cfg(any())]\n let s = FsRtlChangeBackingFileObject(x);\n y(); }", False),
    ]:  # fmt: skip
        live = mask_gated(body)
        checks.append((desc, bool(re.search(r"\bFsRtlChangeBackingFileObject\s*\(", live)), want))
    on = CrateCfg({"reactos"}, {"reactos", "smp"}, {"cnt_libm"})
    off = CrateCfg(set(), {"reactos"}, set())
    for desc, pred, env, want in [
        ("`not(feature)` with the feature ON is off (cdfs: default = [\"reactos\"])",
         'not(feature = "reactos")', on, False),
        ("...and with it OFF is LIVE: masking it would hide the finding",
         'not(feature = "reactos")', off, True),
        ("an undeclared feature is never on", 'feature = "ntddi_win8"', on, False),
        ("a declared, unrequested feature is off", 'feature = "smp"', on, False),
        ("an arch predicate is not this tool's to decide", 'target_arch = "x86"', on, None),
        ("any(unknown, off) is unknown", 'any(target_arch = "x86", ntddi_ge_ntddi_win7)', on, None),
        ("all(unknown, off) is off", 'all(target_arch = "x86", ntddi_ge_ntddi_win7)', on, False),
        ("a cfg the build script emits is unknown", "cnt_libm", on, None),
        ("`test` is unknown", "test", on, None),
    ]:  # fmt: skip
        checks.append((desc, cfg_value(pred, env), want))
    for desc, body, env, want in [
        ("`#[cfg(not(feature))]` masks only when the feature is on",
         '{ #[cfg(not(feature = "reactos"))]\n { FsRtlChangeBackingFileObject(x); } }', on, False),
        ("...feature off: the call is live",
         '{ #[cfg(not(feature = "reactos"))]\n { FsRtlChangeBackingFileObject(x); } }', off, True),
        ("`if !cfg!(off) { A } else { call }` masks the else",
         "{ if !cfg!(ntddi_ge_ntddi_win8) { y(); } else { FsRtlChangeBackingFileObject(x); } }", on, False),  # noqa: E501
        ("`if !cfg!(off) { call }` keeps the then-block",
         "{ if !cfg!(ntddi_ge_ntddi_win8) { FsRtlChangeBackingFileObject(x); } }", on, True),
        ("`if cfg!(off) { y } else { call }`: the else runs",
         "{ if cfg!(ntddi_ge_ntddi_win8) { y(); } else { FsRtlChangeBackingFileObject(x); } }", on, True),  # noqa: E501
    ]:  # fmt: skip
        live = mask_gated(body, body, env)
        checks.append((desc, bool(re.search(r"\bFsRtlChangeBackingFileObject\s*\(", live)), want))
    for desc, body, want in [
        ("`if false { call }` never runs", "{ if false {\n FsRtlChangeBackingFileObject(x); } y(); }", False),  # noqa: E501
        ("`if REACTOS_BUILD { a } else { call }`: the else never runs",
         "{ if REACTOS_BUILD { a(); } else { FsRtlChangeBackingFileObject(x); } }", False),
        ("`if !REACTOS_BUILD { call }` never runs", "{ if !REACTOS_BUILD { FsRtlChangeBackingFileObject(x); } }", False),  # noqa: E501
        ("`if flag { call }` on an unknown name is judged", "{ if flag { FsRtlChangeBackingFileObject(x); } }", True),  # noqa: E501
    ]:  # fmt: skip
        live = mask_gated(body, body, on, {"REACTOS_BUILD": True})
        checks.append((desc, bool(re.search(r"\bFsRtlChangeBackingFileObject\s*\(", live)), want))
    item = [
        "// dead: C is THRESHOLD-only",
        "#[cfg(any())]",
        "#[no_mangle]",
        'pub unsafe extern "C" fn F() {',
    ]
    checks.append(
        (
            "a fn whose own attributes compile it out is skipped",
            item_compiled_out(item, 4, on),
            True,
        )
    )
    checks.append(("...a live one is not", item_compiled_out(item[2:], 2, on), False))
    ok = 0
    for desc, got, want in checks:
        if got == want:
            ok += 1
        else:
            print(f"FAIL: {desc}\n  want {want!r}\n  got  {got!r}")
    print(f"self-test: {ok}/{len(checks)}")
    return 0 if ok == len(checks) else 1


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    lint = "--lint" in sys.argv
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    roots = [Path(a).resolve() for a in args] if args else [RUST_ROOT]
    findings = scan(roots)
    for rel, line, name, twin, directive, hit, src in findings:
        if lint:
            print(f"{rel}: {name} [VERSION_VS_C]")
        else:
            print(
                f"{rel}:{line}: {name} calls {', '.join(hit)} -- only `{directive}` in {twin} "
                f"calls it, and the module's build compiles that branch out [{src}]"
            )
    if "--stats" in sys.argv or (not lint and findings):
        print(
            f"\n{len(findings)} finding(s); {STATS['c_files']} C files, {STATS['chains']} "
            f"version chains judged, {STATS['c_fns_with_dead']} C fns with dead-only calls, "
            f"{STATS['rust_twins']} Rust twins compared, {STATS['ambiguous_version']} C files "
            "skipped (compiled at two versions)",
            file=sys.stderr,
        )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
