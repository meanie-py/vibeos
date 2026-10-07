#!/usr/bin/env python3
"""Delete what ninja built that the requested targets no longer depend on.

`ninja -t cleandead` (run by `vibeos-cli build`) removes the outputs of edges
that have LEFT build.ninja. It cannot touch an edge that is still declared but
that nothing we build depends on any more -- a library taken off every link line
but kept "compiled, as a reference". A warm lane keeps that library's objects
forever; a fresh integration tree never builds them. msvcrt_static did exactly
that on 2026-10-04: green in every lane, red on master (the difftest oracle read
objects only warm trees had).

So this walks the dependency closure of the targets a build asks for, takes
EVERY output of every edge in it (side outputs included -- deleting an import
library or a .map an edge also writes would make ninja re-run that edge on every
build), and removes the files ninja recorded building (.ninja_log) that fall
outside it. A warm build dir then holds what a fresh one would.

Only .ninja_log entries are candidates, so a source file, a cargo target dir, or
anything ninja never wrote is never touched. Paths outside the build dir are
refused rather than followed.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from collections import deque
from pathlib import Path

_ESC = re.compile(r"\$(.)")


def _unescape(tok: str) -> str:
    return _ESC.sub(lambda m: m.group(1) if m.group(1) in " :$" else "$" + m.group(1), tok)


def _split(s: str) -> list[str]:
    """Split a ninja path list on unescaped spaces."""
    out, cur, i = [], [], 0
    while i < len(s):
        c = s[i]
        if c == "$" and i + 1 < len(s):
            cur.append(s[i : i + 2])
            i += 2
            continue
        if c == " ":
            if cur:
                out.append("".join(cur))
                cur = []
        else:
            cur.append(c)
        i += 1
    if cur:
        out.append("".join(cur))
    return [_unescape(t) for t in out]


def _logical_lines(text: str):
    buf = ""
    for line in text.split("\n"):
        if line.endswith("$") and not line.endswith("$$"):
            buf += line[:-1]
            continue
        yield buf + line
        buf = ""
    if buf:
        yield buf


def _colon(s: str) -> int:
    """Index of the first unescaped ':' (the outputs/rule separator)."""
    i = 0
    while i < len(s):
        if s[i] == "$":
            i += 2
            continue
        if s[i] == ":":
            return i
        i += 1
    return -1


def parse_edges(manifest: Path) -> list[tuple[list[str], list[str]]]:
    """[(outputs, inputs)] for every `build` statement, includes followed.

    inputs = explicit + implicit + order-only (ninja builds all three before the
    edge, so all three are part of what a target needs)."""
    edges = []
    seen: set[Path] = set()
    todo = [manifest]
    while todo:
        path = todo.pop()
        if path in seen or not path.is_file():
            continue
        seen.add(path)
        for line in _logical_lines(path.read_text(errors="replace")):
            if line.startswith(("include ", "subninja ")):
                todo.append(manifest.parent / _unescape(line.split(None, 1)[1].strip()))
                continue
            if not line.startswith("build "):
                continue
            body = line[len("build ") :]
            c = _colon(body)
            if c < 0:
                continue
            outs = _split(body[:c].replace(" | ", " "))
            rest = body[c + 1 :].strip()
            rest = rest.split(" |@ ", 1)[0]  # validations are not dependencies
            parts = rest.split(None, 1)
            ins_txt = parts[1] if len(parts) > 1 else ""
            ins = [t for t in _split(ins_txt) if t not in ("|", "||")]
            edges.append(([o for o in outs if o != "|"], ins))
    return edges


def closure_outputs(edges, targets: list[str], norm) -> tuple[set[str], list[str]]:
    """(every output of every edge reachable from `targets`, unknown targets)."""
    by_out: dict[str, int] = {}
    for i, (outs, _ins) in enumerate(edges):
        for o in outs:
            by_out[norm(o)] = i
    keep: set[str] = set()
    visited: set[int] = set()
    unknown = []
    q: deque[str] = deque()
    for t in targets:
        if norm(t) in by_out:
            q.append(norm(t))
        else:
            unknown.append(t)
    while q:
        node = q.popleft()
        e = by_out.get(node)
        if e is None or e in visited:
            continue
        visited.add(e)
        outs, ins = edges[e]
        keep.update(norm(o) for o in outs)
        q.extend(norm(i) for i in ins)
    return keep, unknown


def logged_outputs(build: Path) -> list[str]:
    log = build / ".ninja_log"
    if not log.is_file():
        return []
    out = []
    for line in log.read_text(errors="replace").splitlines():
        if line.startswith("#"):
            continue
        f = line.split("\t")
        if len(f) >= 4:
            out.append(f[3])
    return list(dict.fromkeys(out))


def plan(build: Path, targets: list[str], canon_root: str | None = None):
    """(paths to delete relative to `build`, unknown targets)."""
    roots = [str(build.resolve()) + "/"]
    if canon_root:
        roots.append(canon_root.rstrip("/") + "/")

    def norm(p: str) -> str:
        for r in roots:
            if p.startswith(r):
                return p[len(r) :]
        return os.path.normpath(p) if not p.startswith("/") else p

    edges = parse_edges(build / "build.ninja")
    keep, unknown = closure_outputs(edges, targets, norm)
    victims: list[str] = []
    for p in logged_outputs(build):
        rel = norm(p)
        if rel.startswith("/") or rel.startswith(".."):
            continue  # outside the build dir: never ours to delete
        if rel in keep:
            continue
        if (build / rel).is_file():
            victims.append(rel)
    return sorted(set(victims)), unknown


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("build", type=Path)
    ap.add_argument("targets", nargs="+")
    ap.add_argument("--canon-root", help="the canonical path build.ninja was generated at")
    ap.add_argument("--delete", action="store_true", help="delete (default: list only)")
    a = ap.parse_args()
    victims, unknown = plan(a.build, a.targets, a.canon_root)
    if unknown:
        print(f"ninja_closure_prune: unknown target(s): {' '.join(unknown)}", file=sys.stderr)
        return 2
    for v in victims:
        if a.delete:
            (a.build / v).unlink(missing_ok=True)
        else:
            print(v)
    print(f"{len(victims)} file(s) outside the closure{' removed' if a.delete else ''}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
