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
            db = json.load(open(db_path))
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
        elif t in ("-isystem", "-include", "-imacros"):
            if i + 1 < len(toks):
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


def build_anchor_map(c_file: Path, timeout: int = 180,
                     verbose: bool = True) -> dict[str, str]:
    """Transpile c_file with c2rust and return {fn_name: rust_body}. Best-effort:
    returns {} on any failure (anchor is optional context, never a hard dep)."""
    if shutil.which("c2rust") is None:
        if verbose:
            print("  [c2rust] binary not on PATH — skipping anchors", file=sys.stderr)
        return {}
    found = _find_db_entry(c_file)
    if found is None:
        if verbose:
            print(f"  [c2rust] no compile_commands entry for {c_file.name} — "
                  f"skipping anchors", file=sys.stderr)
        return {}
    entry, build_dir = found
    flags = _extract_clang_flags(entry)
    out_dir = Path(tempfile.mkdtemp(prefix="c2rust_anchor_"))
    try:
        cmd = ["c2rust", "transpile", "--output-dir", str(out_dir),
               str(c_file.resolve()), "--"] + flags
        try:
            subprocess.run(cmd, cwd=str(build_dir), capture_output=True,
                           text=True, timeout=timeout)
        except subprocess.TimeoutExpired:
            if verbose:
                print(f"  [c2rust] timeout on {c_file.name}", file=sys.stderr)
            return {}
        # c2rust panics on some kernel ASTs; we don't care about exit code, only
        # whether it produced output for our file.
        rs = out_dir / "src" / f"{c_file.stem}.rs"
        if not rs.exists():
            # some layouts drop it at the top level
            alt = out_dir / f"{c_file.stem}.rs"
            rs = alt if alt.exists() else rs
        if not rs.exists():
            if verbose:
                print(f"  [c2rust] no .rs produced for {c_file.name} (AST "
                      f"unsupported) — skipping anchors", file=sys.stderr)
            return {}
        fns = _parse_functions(rs.read_text(errors="replace"))
        if verbose:
            print(f"  [c2rust] anchors: {len(fns)} fn(s) from {c_file.name}",
                  file=sys.stderr)
        return fns
    finally:
        shutil.rmtree(out_dir, ignore_errors=True)
