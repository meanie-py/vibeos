#!/usr/bin/env python3
"""Triage upstream ReactOS commits we have not merged yet: which ones are BUG ORACLES.

WHY THIS EXISTS
---------------
The lane's standing job is "get the upstream commits since the fork triaged and
semantically ported over, or merged in if it's pure C". The decision per commit is the
same question every time — *does it touch a file whose C we no longer compile?* — and
answering it by hand means reading a file list, then remembering which surfaces are
translated, for every commit. That is a recipe, and a recipe is a thing that belongs in
a tool.

The framing this encodes: **upstream is a BUG ORACLE, not a patch queue.** We translated
pre-fix C, so an upstream fix to a file we translated is evidence our Rust carries that
bug. A fix to a file we still COMPILE is just a merge.

THE LIVE/OUT-OF-BUILD SPLIT COMES FROM port_audit, NOT FROM A SECOND GUESS
-------------------------------------------------------------------------
`port_audit.compiled_sources()` already derives what the build actually compiles from the
ninja graph, after every `if(ARCH)` and variable. Re-deriving it here — by reading
CMakeLists for `# replaced by Rust`, say — would give a second answer that drifts from
the first, which is the exact failure the repo keeps re-learning. So this imports it.
Same for `vendored.Registry`: `vendor.yaml` is the only list of what is third-party.

HEADERS ARE THEIR OWN VERDICT, AND THE INTERESTING ONE
------------------------------------------------------
A header is not a compile edge, so it is never in the LIVE set and a naive classifier
calls every header change "translated". But a header change can be the most invasive kind
there is: `sdk/lib/cmlib/cmlib.h` adding a `UNICODE_STRING` member to `CMHIVE` moves
every field after it, and our Rust binds that struct through bindgen. So headers are
reported separately, flagged for layout review, and never silently counted as either
bucket.

WHAT THIS DOES NOT DO
---------------------
It does not decide anything. A commit landing in ORACLE means "read this one against our
Rust", not "port it": upstream also refactors, reformats and fixes things in code paths we
translated differently or not at all. The verdict is a reading order, not a work order.
"""

from __future__ import annotations

import argparse
import importlib.util
import posixpath
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


class TriageError(Exception):
    pass


def git(*args: str, repo: Path = REPO) -> str:
    out = subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True, text=True, check=False
    )
    if out.returncode != 0:
        raise TriageError(f"git {' '.join(args)} failed: {out.stderr.strip()}")
    return out.stdout


SOURCE_SUFFIXES = (".c", ".cpp", ".cc", ".cxx")
HEADER_SUFFIXES = (".h", ".hpp", ".hxx", ".inl")

# Buckets, most-significant first: a commit's verdict is the first one it hits.
ORACLE = "ORACLE"
HEADER = "HEADER"
MERGE = "MERGE"
VENDORED = "VENDORED"
TESTS = "TESTS"
UNBUILT = "UNBUILT"
OTHER = "OTHER"

# Trees whose sources this build does not compile by CONFIGURATION, not because they were
# translated. Without this the winetests alone make every Wine-sync commit look like an
# oracle hit: `[USP10] Sync with WINE-11.17` came back ORACLE purely on
# `modules/rostests/winetests/usp10/usp10.c`, while all 14 actual DLL sources in the same
# commit were correctly MERGE. "Not in the build graph" is only evidence of translation
# for code the build is SUPPOSED to compile.
TEST_TREES = ("modules/rostests/", "modules/rosapps/")


def classify_file(
    rel: str, live: set[str], registry, on_disk: bool, disabled: frozenset[str] = frozenset()
) -> str:
    """One file -> one bucket. See the module docstring for why headers are separate."""
    if registry.is_vendored(rel):
        return VENDORED
    if rel.endswith(HEADER_SUFFIXES):
        return HEADER
    if rel.startswith(TEST_TREES):
        return TESTS
    if rel.endswith(SOURCE_SUFFIXES):
        if rel in live:
            return MERGE
        # Not compiled by ANY arch. A file upstream has and we do not is new code, not a
        # translation of ours. A file we have that a CMake COMMENT switches off was
        # deliberately taken out of the build -- almost always a cohab flip, so an
        # upstream change there is a change our Rust never saw. A file we have that no
        # comment mentions was simply never built by this port (PCIX, say): reading it
        # against the Rust finds nothing, because there is no Rust. Folding those two
        # together is what made the already-merged range read as 465 oracle hits.
        if not on_disk:
            return OTHER
        return ORACLE if rel in disabled else UNBUILT
    return OTHER


# One source name in a CMake comment line: a `${REACTOS_SOURCE_DIR}/...` path, a
# `${CMAKE_CURRENT_SOURCE_DIR}/...` path, or a path relative to the CMake file.
_SRC_TOKEN = re.compile(r"[A-Za-z0-9_${}/.+-]*[A-Za-z0-9_+-]\.(?:c|cpp|cxx|cc)\b")


def resolve_comment_sources(cmake_rel: str, line: str, exists) -> set[str]:
    """Repo-relative sources a commented CMake `line` names, keeping only paths that
    `exists`. Generator expressions are why this is not just a basename match:
    ntos.cmake disables pnpmgr.c as
    `#$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpmgr.c>`,
    and the cohab registry missed exactly that spelling -- along with the only
    translated file whose upstream fix had already cost a day."""
    here = posixpath.dirname(cmake_rel)
    out: set[str] = set()
    for tok in _SRC_TOKEN.findall(line):
        cands = []
        if "${REACTOS_SOURCE_DIR}/" in tok:
            cands.append(tok.split("${REACTOS_SOURCE_DIR}/", 1)[1])
        elif "${CMAKE_CURRENT_SOURCE_DIR}/" in tok:
            cands.append(posixpath.join(here, tok.split("${CMAKE_CURRENT_SOURCE_DIR}/", 1)[1]))
        elif "$" not in tok:
            cands.append(posixpath.join(here, tok))
            cands.append(tok)
        for c in cands:
            c = posixpath.normpath(c)
            if not c.startswith("..") and exists(c):
                out.add(c)
                break
    return out


def disabled_sources(root: Path = REPO) -> frozenset[str]:
    """Every existing source a commented-out CMake line names, across the tree. Uses
    cohab_census's walker and comment pattern, so the two tools agree on what a
    disabling comment is."""
    cc = _load("cohab_census", HERE / "cohab_census.py")
    out: set[str] = set()
    for p in cc.cmake_files(root):
        rel = p.relative_to(root).as_posix()
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line in text.splitlines():
            if cc.SRC_IN_COMMENT.match(line):
                out |= resolve_comment_sources(rel, line, lambda r: (root / r).is_file())
    return frozenset(out)


def triage(
    rev_range: str,
    live: set[str],
    registry,
    repo: Path = REPO,
    disabled: frozenset[str] = frozenset(),
):
    """Yield (sha, subject, verdict, {bucket: [files]}) oldest-first."""
    log = git("log", "--reverse", "--format=%H%x1f%s", rev_range, repo=repo)
    for line in log.splitlines():
        if not line.strip():
            continue
        sha, subject = line.split("\x1f", 1)
        files = [
            f
            for f in git("show", "--pretty=", "--name-only", sha, repo=repo).splitlines()
            if f.strip()
        ]
        buckets: dict[str, list[str]] = {}
        for rel in files:
            on_disk = (repo / rel).exists()
            verdict = classify_file(rel, live, registry, on_disk, disabled)
            buckets.setdefault(verdict, []).append(rel)
        for verdict in (ORACLE, HEADER, MERGE, VENDORED, TESTS, UNBUILT, OTHER):
            if verdict in buckets:
                break
        else:
            verdict = OTHER
        yield sha[:11], subject, verdict, buckets


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--range",
        dest="rev_range",
        default=None,
        help="commit range (default: merge-base(HEAD, upstream/master)..upstream/master)",
    )
    ap.add_argument(
        "--only", choices=[ORACLE, HEADER, MERGE, VENDORED, TESTS, UNBUILT, OTHER], default=None
    )
    ap.add_argument("--files", action="store_true", help="list each commit's files")
    args = ap.parse_args(argv)

    pa = _load("port_audit", HERE / "port_audit.py")
    vendored = _load("vendored", HERE / "vendored.py")
    registry = vendored.load(REPO)

    rev_range = args.rev_range
    if rev_range is None:
        base = git("merge-base", "HEAD", "upstream/master").strip()
        rev_range = f"{base}..upstream/master"

    per_arch = pa.compiled_sources(REPO)
    live: set[str] = set()
    for srcs in per_arch.values():
        live |= srcs
    if not live:
        print(
            "upstream_triage: the build graph is empty, so every .c would look "
            "translated and every commit an ORACLE.\n"
            "  fix: ./vibeos-cli build --arch amd64   (port_audit needs build-*/ )",
            file=sys.stderr,
        )
        return 1

    rows = list(triage(rev_range, live, registry, disabled=disabled_sources(REPO)))
    counts: dict[str, int] = {}
    for _sha, _subj, verdict, _b in rows:
        counts[verdict] = counts.get(verdict, 0) + 1

    for sha, subject, verdict, buckets in rows:
        if args.only and verdict != args.only:
            continue
        print(f"{verdict:8} {sha}  {subject}")
        if args.files:
            for bucket in (ORACLE, HEADER, MERGE, VENDORED, TESTS, UNBUILT, OTHER):
                for rel in buckets.get(bucket, []):
                    print(f"           {bucket:8} {rel}")

    order = [ORACLE, HEADER, MERGE, VENDORED, TESTS, UNBUILT, OTHER]
    summary = ", ".join(f"{counts.get(k, 0)} {k}" for k in order if counts.get(k))
    print(f"\nupstream_triage: {len(rows)} commit(s) in {rev_range} — {summary}")
    print(
        "  ORACLE   = touches C a CMake comment switched off (a cohab flip) → read it\n"
        "             against our Rust; a fix there is a bug we translated.\n"
        "  HEADER   = header-only reach; check struct layout, our Rust binds these.\n"
        "  MERGE    = C we still compile → ordinary merge.\n"
        "  VENDORED = vendor.yaml third-party → sync, not translate.\n"
        "  TESTS    = rostests/rosapps; not compiled here by CONFIG, not translated.\n"
        "  UNBUILT  = C on disk that nothing builds OR disables; no Rust to read it against.\n"
        "  A verdict is a READING ORDER, not a work order."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
