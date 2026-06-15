"""c2rust behavior anchors — feed a compiler-exact transpilation into the prompt.

The mechanical LLM pass silently STUBS/drops hard function bodies and turns
function-like macros (e.g. `RosSymAllocMem`, `DPRINT`) into phantom externs. c2rust
is a *compiler*, not an LLM — it physically cannot drop, stub, or summarize logic, so
its transpilation of a function is a behavior-correct-by-construction checklist the
LLM must preserve. We don't ship c2rust's raw output (it's ugly: libc casts, generated
names, `.expect("non-null function pointer")`); we hand it to the model as an anchor and
let it restyle the surface while preserving every statement/branch/call.

This module produces, for one C file, a `{function_name: rust_body}` map. It is
strictly best-effort: any failure (no compile_commands.json, c2rust panic, parse miss)
returns an empty/partial map and translation proceeds anchor-less — never blocks.

Validated recipe (2026-06-07): c2rust chokes on a bare compile_commands.json path and
on the mingw cross `command` (multi-job), but the *direct-source* form works:
    c2rust transpile --output-dir DIR SOURCE.c -- <--target + -D + -I from the db entry>
Output lands at DIR/src/<stem>.rs.
"""

from __future__ import annotations

import json
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from .config import REACTOS_ROOT

# Build dirs that carry a compile_commands.json (i386's cmake has export OFF).
# Order = preference; the anchor only needs logic/structure, so an arch mismatch
# vs the translation target is harmless.
_DB_BUILD_DIRS = ["build-amd64", "build-arm64", "build-i386"]


def _find_db_entry(c_file: Path) -> tuple[dict, Path] | None:
    """Return (entry, build_dir) for c_file from the first build db that has it."""
    cf = str(c_file.resolve())
    for bd in _DB_BUILD_DIRS:
        db_path = REACTOS_ROOT / bd / "compile_commands.json"
        if not db_path.exists():
            continue
        try:
            with open(db_path) as f:
                db = json.load(f)
        except Exception:
            continue
        for e in db:
            if str(Path(e["file"]).resolve()) == cf:
                return e, (REACTOS_ROOT / bd)
    return None


def _extract_clang_flags(entry: dict) -> list[str]:
    """Pull --target, -D, -I, -isystem/-include from the db command; drop the
    multi-job triggers (-o/-c/-M*) that make clang tooling reject the command."""
    cmd = entry.get("command") or " ".join(entry.get("arguments", []))
    toks = shlex.split(cmd)
    keep: list[str] = []
    i = 1  # skip argv[0] (the compiler)
    while i < len(toks):
        t = toks[i]
        if t.startswith("-D") or t.startswith("-I") or t.startswith("--target"):
            keep.append(t)
        elif t in ("-isystem", "-include", "-imacros") and i + 1 < len(toks):
            keep.append(t)
            keep.append(toks[i + 1])
            i += 1
        i += 1
    return keep


# c2rust emits top-level fns rustfmt-style: signature line contains `fn NAME(`,
# body closes with `}` at column 0. Capture NAME → full text.
_FN_SIG = re.compile(r"\bfn\s+([A-Za-z_]\w*)\s*[(<]")


def _parse_functions(rs_text: str) -> dict[str, str]:
    lines = rs_text.splitlines()
    out: dict[str, str] = {}
    i = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        m = _FN_SIG.search(line)
        # Only treat as a fn DEFINITION if the signature region is a real item
        # (preceded by fn-item keywords on this or a recent attribute line) and
        # the block eventually opens a brace. c2rust prefixes with pub/unsafe/extern.
        is_item = bool(m) and (
            line.lstrip().startswith(("fn ", "pub ", "unsafe ", "extern ", "pub("))
        )
        if is_item:
            assert m is not None  # is_item is True only when m matched
            name = m.group(1)
            start = i
            # walk to the line that opens the body `{`, then to the matching `}`
            # at column 0 (c2rust formatting guarantees this for top-level items).
            depth = 0
            opened = False
            j = i
            while j < n:
                depth += lines[j].count("{") - lines[j].count("}")
                if "{" in lines[j]:
                    opened = True
                if opened and depth <= 0:
                    break
                j += 1
            if opened:
                body = "\n".join(lines[start : j + 1])
                # last writer wins is fine; c2rust won't emit duplicate defs
                out[name] = body
                i = j + 1
                continue
        i += 1
    return out


# Transpile is the slow step (~seconds); a file's anchors AND preamble both need it.
# Cache the raw .rs text per (path, mtime) so the second consumer is free.
_TRANSPILE_CACHE: dict[tuple[str, float], str | None] = {}


def _transpile_text(c_file: Path, timeout: int = 180, verbose: bool = True) -> str | None:
    """Run c2rust on c_file and return the raw transpiled .rs text (None on any failure).
    Best-effort + cached: anchors and preamble extraction share one transpile."""
    try:
        key = (str(c_file.resolve()), c_file.stat().st_mtime)
    except OSError:
        return None
    if key in _TRANSPILE_CACHE:
        return _TRANSPILE_CACHE[key]
    result: str | None = None
    if shutil.which("c2rust") is None:
        if verbose:
            print("  [c2rust] binary not on PATH — skipping", file=sys.stderr)
        _TRANSPILE_CACHE[key] = None
        return None
    found = _find_db_entry(c_file)
    if found is None:
        if verbose:
            print(
                f"  [c2rust] no compile_commands entry for {c_file.name} — skipping",
                file=sys.stderr,
            )
        _TRANSPILE_CACHE[key] = None
        return None
    entry, build_dir = found
    flags = _extract_clang_flags(entry)
    out_dir = Path(tempfile.mkdtemp(prefix="c2rust_anchor_"))
    try:
        cmd = [
            "c2rust",
            "transpile",
            "--output-dir",
            str(out_dir),
            str(c_file.resolve()),
            "--",
            *flags,
        ]
        try:
            subprocess.run(cmd, cwd=str(build_dir), capture_output=True, text=True, timeout=timeout)
        except subprocess.TimeoutExpired:
            if verbose:
                print(f"  [c2rust] timeout on {c_file.name}", file=sys.stderr)
            _TRANSPILE_CACHE[key] = None
            return None
        # c2rust panics on some kernel ASTs (e.g. the `Could not find CDeclId` TU-abort);
        # we don't care about exit code, only whether it produced output for our file.
        rs = out_dir / "src" / f"{c_file.stem}.rs"
        if not rs.exists():
            alt = out_dir / f"{c_file.stem}.rs"  # some layouts drop it at the top level
            rs = alt if alt.exists() else rs
        if rs.exists():
            result = rs.read_text(errors="replace")
        elif verbose:
            print(
                f"  [c2rust] no .rs produced for {c_file.name} (AST unsupported)", file=sys.stderr
            )
    finally:
        shutil.rmtree(out_dir, ignore_errors=True)
    _TRANSPILE_CACHE[key] = result
    return result


def build_anchor_map(c_file: Path, timeout: int = 180, verbose: bool = True) -> dict[str, str]:
    """Transpile c_file with c2rust and return {fn_name: rust_body}. Best-effort:
    returns {} on any failure (anchor is optional context, never a hard dep)."""
    text = _transpile_text(c_file, timeout=timeout, verbose=verbose)
    if text is None:
        return {}
    fns = _parse_functions(text)
    if verbose:
        print(f"  [c2rust] anchors: {len(fns)} fn(s) from {c_file.name}", file=sys.stderr)
    return fns


# File-scope DATA tables c2rust transcribes compiler-exact: the lookup tables / S-boxes /
# dispatch tables / quirk lists the LLM preamble pass FABRICATES or truncates. We take ONLY
# `static [mut]` items and `const` items with an AGGREGATE initializer (`[`/`{`) — i.e. real
# data. We deliberately EXCLUDE scalar `const NAME: _ENUM = 5;` items: c2rust floods those from
# every header enum it touched (PCIBus/PowerAction…, hundreds per file), they are NOT the file's
# own data, and they COLLIDE with the enums bindgen/ntkrnl_sys already define. Struct/type/union
# defs are likewise excluded — drivers get those exact from bindgen `types_*.rs`; only DATA is
# fabrication-prone. Scalar #define consts stay the harvest_consts/LLM-preamble job (low risk).
_DATA_SIG = re.compile(r"^(?:pub\s+)?(static(?:\s+mut)?|const)\s+[A-Za-z_]\w*\s*:", re.M)


def build_preamble(c_file: Path, timeout: int = 180, verbose: bool = True) -> str:
    """Return c2rust's file-scope DATA tables (`static [mut]` + aggregate `const`) as exact
    Rust text, to prepend deterministically instead of LLM-translating the preamble (which
    fabricates table values). Excludes header enum-variant scalar consts. Best-effort: '' on
    failure."""
    text = _transpile_text(c_file, timeout=timeout, verbose=verbose)
    if text is None:
        return ""
    lines = text.splitlines()
    out: list[str] = []
    i, n = 0, len(lines)
    while i < n:
        m = _DATA_SIG.match(lines[i])
        if m:
            start = i
            # an item runs until brackets balance and the line ends in `;` (handles both
            # one-line scalars and multi-line array/struct initialisers).
            depth = 0
            j = i
            while j < n:
                depth += lines[j].count("[") - lines[j].count("]")
                depth += lines[j].count("{") - lines[j].count("}")
                if depth <= 0 and lines[j].rstrip().endswith(";"):
                    break
                j += 1
            block = "\n".join(lines[start : j + 1])
            # keep `static*` always; keep `const` ONLY if it carries an aggregate (table/struct),
            # never a bare scalar enum-variant const.
            if m.group(1).startswith("static") or ("[" in block or "{" in block):
                out.append(block)
            i = j + 1
            continue
        i += 1
    if verbose:
        print(f"  [c2rust] preamble: {len(out)} data item(s) from {c_file.name}", file=sys.stderr)
    return "\n\n".join(out)
