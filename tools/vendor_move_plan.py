#!/usr/bin/env python3
"""What pins each vendored tree to its current path: the blast radius of moving it.

WHY THIS EXISTS
---------------
Vendored code is scattered through the ReactOS layout -- ACPICA under drivers/, FreeType
under sdk/lib/3rdparty/, FEX under external/, dosfstools inside vfatlib, two loose
files in setup -- because upstream put each drop wherever its consumer lived. The plan
is to gather it into one directory, so that "is this ours to translate" becomes a fact
about the path instead of a lookup in vendor.yaml.

A move is only as safe as its reference list. A tree is referenced from its consumers'
CMakeLists (often RELATIVELY: `add_subdirectory(acpica)` never spells the full path),
from `include_directories`, from Rust build scripts and bindgen wrappers, from tools
with hard-coded paths, from the registry and its patches, and from the tree's own
sources reaching OUT with `#include "../../x.h"`. Miss one and the break is a build
failure on one arch, or worse, a bindgen island silently generated from a stale copy.
So this enumerates them rather than leaving the next agent to grep and hope.

It does not move anything. The destination is a separate decision; this answers "what
would have to change" for every tree at once, so the move can be sequenced cheapest
first and each step reviewed against a list instead of a guess.

WHAT EACH COLUMN MEANS
----------------------
  build     CMake files naming the tree -- by full path, or relative to the CMake
            file's own directory (the add_subdirectory case). MUST change.
  rust      files under sdk/lib/rust naming it: build.rs, bindgen wrappers, Cargo.toml.
            MUST change, and bindgen islands must be regenerated, not hand-edited.
  tooling   tools/, tests/, top-level scripts. MUST change or the gates misread.
  registry  vendor.yaml, vendor-patches/, THIRD-PARTY-NOTICES.md, upstream's ledger.
            Mechanical, and `vendored.py` regenerates the generated ones.
  docs      prose. Should change; nothing breaks if it lags.
  source    C/C++ OUTSIDE the tree spelling a path into it (a relative #include).
  include   code reaching a header tree under sdk/include through an INCLUDE ROOT
            (`#include <GL/gl.h>`). It never spells the tree's path, so a path grep
            reports zero -- the first cut of this tool did, for GL and d3dkmthk.h. These
            consumers need not change if the new location is added as an include root,
            so they are reported but not counted as must-change: the count says how
            much depends on that one include_directories line. Blind to a header dir
            that is ITSELF an include root (`#include <zlib.h>`): matching every bare
            header name would drown the column in `config.h`.
  review    scripts that assemble the path from a VARIABLE -- acpica-sys's bindgen
            script says `${ACPI_DIR}/acpica/include`, which no full-path search can
            see, and which the first cut of this tool missed. Matched by the tree's
            shortest distinctive tail after a `/` or `}`. Heuristic, so it is a list to
            READ before moving, not a count to trust.
  outbound  files INSIDE the tree reaching out of it with a relative path. These break
            on a move even when nothing references the tree, and are the ones a
            reference grep of the rest of the repo can never see.

Reads only this worktree (fleet isolation): build dirs, cargo targets, .git and
`.claude/` subagent checkouts are pruned, never entered.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import vendored

REPO = Path(__file__).resolve().parents[1]

# `.venv` is third-party Python, `CMakeFiles` is configure output that leaks into the
# source tree when someone runs cmake in place: neither is a reference we would edit.
_SKIP_DIRS = {
    ".git",
    "target",
    "__pycache__",
    ".claude",
    "node_modules",
    "output",
    ".venv",
    "CMakeFiles",
}
_SELF = "tools/vendor_move_plan.py"
_TEXT_SUFFIXES = {
    ".txt",
    ".cmake",
    ".py",
    ".rs",
    ".toml",
    ".yaml",
    ".yml",
    ".md",
    ".c",
    ".h",
    ".cpp",
    ".hpp",
    ".cc",
    ".cxx",
    ".inc",
    ".s",
    ".asm",
    ".spec",
    ".rc",
    ".inf",
    ".patch",
    ".sh",
    ".json",
    ".ps1",
    ".in",
    ".def",
}
_CODE_SUFFIXES = {".c", ".h", ".cpp", ".hpp", ".cc", ".cxx", ".inc", ".s", ".asm", ".rc"}
_INCLUDE = re.compile(r'^\s*#\s*include\s*"([^"]+)"', re.M)
_REL_IN_CMAKE = re.compile(r'(?<![\w$}/.-])(\.\./[^\s)"\']+)')

CATEGORIES = (
    "build",
    "rust",
    "tooling",
    "registry",
    "docs",
    "source",
    "outbound",
    "include",
    "review",
)
_SCRIPTY = {".sh", ".py", ".cmake", ".toml", ".ps1", ".in"}
# Basenames too generic to identify a tree on their own; lengthen the tail instead.
_GENERIC = {"src", "include", "check", "lib", "ms", "gl", "wine", "common", "drivers"}


def _tail(prefix: str) -> str:
    """Shortest trailing path of the prefix that is distinctive enough to search for."""
    parts = prefix.split("/")
    for i in range(len(parts) - 1, -1, -1):
        tail = "/".join(parts[i:])
        if len(tail) >= 5 and parts[i].lower() not in _GENERIC:
            return tail
    return prefix


@dataclass
class Plan:
    path: str
    name: str
    loc: int
    refs: dict[str, set[str]] = field(default_factory=lambda: defaultdict(set))

    @property
    def must_change(self) -> int:
        return sum(len(self.refs[c]) for c in ("build", "rust", "tooling", "source", "outbound"))


def _walk(repo: Path):
    for dirpath, dirnames, filenames in os.walk(repo, onerror=lambda _e: None):
        dirnames[:] = [d for d in dirnames if d not in _SKIP_DIRS and not d.startswith("build-")]
        for name in filenames:
            p = Path(dirpath, name)
            if name in ("CMakeLists.txt", "VENDORED") or p.suffix.lower() in _TEXT_SUFFIXES:
                yield p


def _category(rel: str) -> str:
    name = rel.rsplit("/", 1)[-1]
    if name == "CMakeLists.txt" or rel.endswith(".cmake"):
        return "build"
    if rel.startswith("sdk/lib/rust/"):
        return "rust"
    if rel in ("vendor.yaml", "THIRD-PARTY-NOTICES.md", "media/doc/3rd Party Files.txt"):
        return "registry"
    if rel.startswith("vendor-patches/"):
        return "registry"
    if rel.startswith(("tools/", "tests/")) or "/" not in rel:
        return "docs" if rel.endswith(".md") else "tooling"
    if rel.endswith((".md", ".txt")):
        return "docs"
    if Path(rel).suffix.lower() in _CODE_SUFFIXES:
        return "source"
    return "tooling"


_ANY_INCLUDE = re.compile(r'^\s*#\s*include\s*[<"]([^">]+)[">]', re.M)


def _root_spellings(prefix: str) -> list[str]:
    """How code spells a path under sdk/include through an include root.

    `sdk/include/GL` -> ["GL"]; `sdk/include/ddk/d3dkmthk.h` -> ["d3dkmthk.h",
    "ddk/d3dkmthk.h"]. Only sdk/include: that is where the tree's global roots live, and
    a spelling taken from a deeper root elsewhere (a bare "check") would match anything.
    """
    base = "sdk/include/"
    if not prefix.startswith(base):
        return []
    parts = prefix[len(base) :].split("/")
    return ["/".join(parts[i:]) for i in range(len(parts) - 1, -1, -1)]


def _inside(rel: str, prefixes: list[str]) -> bool:
    return any(rel == p or rel.startswith(p + "/") for p in prefixes)


def _escapes(repo: Path, src: Path, target: str, prefixes: list[str]) -> bool:
    """Does a relative reference from `src` resolve OUTSIDE the tree? Textual only."""
    joined = os.path.normpath(os.path.join(src.parent.relative_to(repo).as_posix(), target))
    return not joined.startswith("..") and not _inside(joined.replace(os.sep, "/"), prefixes)


def plan(repo: Path = REPO) -> list[Plan]:
    reg = vendored.load(repo)
    plans = [Plan(t.path, t.name, reg.loc(t)) for t in reg.trees]
    by_path = {p.path: p for p in plans}
    prefixes = {t.path: t.prefixes for t in reg.trees}
    # Longest first, so a nested entry (`sdk/include/reactos/libs/x` inside a broader
    # one) is credited to the tree that owns it rather than to both.
    needles = sorted(
        ((pre, t.path) for t in reg.trees for pre in t.prefixes), key=lambda x: -len(x[0])
    )

    for f in _walk(repo):
        rel = f.relative_to(repo).as_posix()
        if rel == _SELF:
            continue  # its docstring names trees as examples
        try:
            text = f.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        owner = next((tp for pre, tp in needles if _inside(rel, [pre])), None)
        if owner is not None and rel.rsplit("/", 1)[-1] == "VENDORED":
            continue
        cat = _category(rel)
        is_cmake = cat == "build"
        # A .cmake fragment is include()d, so its relative paths resolve against the
        # INCLUDING CMakeLists' directory, not its own: sdk/lib/crt/math/math.cmake lists
        # `math/ieee754/j0_y0.c` relative to sdk/lib/crt. So try every ancestor.
        parts = rel.split("/")[:-1]
        cmake_dirs = ["/".join(parts[:i]) for i in range(len(parts), 0, -1)] if is_cmake else []
        is_code = f.suffix.lower() in _CODE_SUFFIXES
        includes = _ANY_INCLUDE.findall(text) if is_code and owner is None else []

        for pre, tpath in needles:
            if owner == tpath:
                continue  # a tree mentioning itself moves with it
            hit = pre in text
            for d in cmake_dirs:
                if hit:
                    break
                if pre.startswith(d + "/"):
                    # `add_subdirectory(acpica)` from drivers/bus/acpi/CMakeLists.txt:
                    # relative to the CMake file and never spelling the full path.
                    tail = pre[len(d) + 1 :]
                    hit = re.search(rf"(?<![\w/.-]){re.escape(tail)}(?![\w.-])", text) is not None
            if hit:
                by_path[tpath].refs[cat].add(rel)
            elif (
                f.suffix.lower() in _SCRIPTY
                or f.name in ("CMakeLists.txt", "build.rs")
                or (cat == "rust" and f.suffix.lower() == ".h")
            ) and re.search(rf"[}}/]{re.escape(_tail(pre))}(?=[/\"'\s)]|$)", text, re.M):
                by_path[tpath].refs["review"].add(rel)
            elif includes and any(
                inc == sp or inc.startswith(sp + "/")
                for sp in _root_spellings(pre)
                for inc in includes
            ):
                by_path[tpath].refs["include"].add(rel)

        if owner is None:
            continue
        # Outbound: the tree's own files reaching out by relative path.
        tpre = prefixes[owner]
        targets: list[str] = []
        if f.suffix.lower() in _CODE_SUFFIXES:
            targets = [t for t in _INCLUDE.findall(text) if t.startswith("../")]
        elif is_cmake:
            targets = _REL_IN_CMAKE.findall(text)
        if any(_escapes(repo, f, t, tpre) for t in targets):
            by_path[owner].refs["outbound"].add(rel)
    return plans


def render(plans: list[Plan]) -> str:
    rows = sorted(plans, key=lambda p: (p.must_change, p.loc))
    out = [
        "| Tree | LOC | build | rust | tooling | source | outbound | include | review "
        "| registry | docs |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for p in rows:
        c = {k: len(p.refs[k]) for k in CATEGORIES}
        out.append(
            f"| `{p.path}` | {p.loc:,} | {c['build']} | {c['rust']} | {c['tooling']} | "
            f"{c['source']} | {c['outbound']} | {c['include']} | {c['review']} | "
            f"{c['registry']} | {c['docs']} |"
        )
    easy = [p.path for p in rows if p.must_change <= 1]
    out.append("")
    out.append(
        f"*{len(plans)} trees. {len(easy)} have at most one must-change reference "
        f"(build/rust/tooling/source/outbound) and are the natural first movers. "
        f"Rows are sorted cheapest first.*"
    )
    return "\n".join(out)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="What pins each vendored tree to its current path (move blast radius)."
    )
    ap.add_argument("--json", action="store_true", help="every reference, machine-readable")
    ap.add_argument("--tree", metavar="PATH", help="list every reference for one tree")
    args = ap.parse_args(argv)
    plans = plan()
    if args.tree:
        match = [p for p in plans if p.path == args.tree.rstrip("/")]
        if not match:
            sys.stderr.write(f"not a registered tree: {args.tree}\n")
            return 2
        for cat in CATEGORIES:
            for rel in sorted(match[0].refs[cat]):
                print(f"{cat:9} {rel}")
        return 0
    if args.json:
        print(
            json.dumps(
                {p.path: {c: sorted(p.refs[c]) for c in CATEGORIES} for p in plans},
                indent=2,
                sort_keys=True,
            )
        )
        return 0
    print(render(plans))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
