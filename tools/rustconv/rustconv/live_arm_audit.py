"""Which shipped bodies dropped a call from a LIVE `#if` arm of their C function ($0).

The call seal (call_seal.py) makes every call inside any `#if` arm optional, because
`dead_regions.preprocessor_conditional_spans` treats every arm as possibly dead. So on
rundll32's `#ifdef UNICODE WideCharToMultiByte #else strncpy`, a model that drops the live
call passes. Tightening the seal (live arm required) would turn a `port replay` red for any
banked body that already did this, so the tightening is measured first: this walks every
flipped C file the cohab census lists (`# x.c # replaced by Rust (y.rs)`), decides each
conditional with `dead_regions.decided_live_lines` -- the walker the seal itself uses --
under the module's own `-D`s, and reports each call in a decidably
live arm that the Rust function of the same name never names.

A miss is a lead, not a verdict: the Rust may reach the callee through a wrapper, a macro
or a different spelling (`lstrcpyW` for a TCHAR `lstrcpy`). Read the body against the C.

    python -m rustconv.live_arm_audit [--census tools/cohab-census.txt] [--json OUT]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

from .dead_regions import KNOWN_DEFINED, decided_live_lines, pinned_int_macros
from .lex_mask import mask_code

_FN_LIKE_DEFINE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)\(", re.M)
_CALL = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
_KEYWORDS = frozenset(
    [
        "if",
        "while",
        "for",
        "switch",
        "return",
        "sizeof",
        "do",
        "else",
        "case",
        "defined",
        "__declspec",
        "__attribute__",
        "_SEH2_TRY",
        "_SEH2_EXCEPT",
        "_SEH2_FINALLY",
        "_SEH2_END",
        "_SEH2_LEAVE",
    ]
)
_CENSUS = re.compile(r"^(\S+CMakeLists\.txt)\t\d+\t#\s*(\S+\.c)\s+#\s*replaced by Rust \((\S+\.rs)")


@dataclass
class Miss:
    c_file: str
    rs_file: str
    function: str
    callee: str
    line: int
    guard: str


def _brace_end(text: str, opening: int) -> int:
    depth = 0
    for i in range(opening, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    return len(text)


def c_functions(path: Path) -> dict[str, tuple[int, int]]:
    """Function name -> (first, last) line of each definition in the RAW file (tree-sitter).

    Not scanner.scan_file: its line numbers are of the text it preprocessed, and a brace
    count cannot survive braces inside `#ifdef`s (edit.c's ran one function over 1,600
    lines of its neighbours). tree-sitter keeps a body's `#ifdef`s as nodes of the body.
    Its error recovery can still lose a function after one whose OPENING brace is split by
    an `#ifdef`; that function is then not audited (an under-report, never a false lead)."""
    import tree_sitter_c as tsc
    from tree_sitter import Language, Parser

    source = path.read_bytes()
    tree = Parser(Language(tsc.language())).parse(source)
    out: dict[str, tuple[int, int]] = {}
    stack = [tree.root_node]
    while stack:
        node = stack.pop()
        if node.type == "function_definition":
            decl = node.child_by_field_name("declarator")
            while decl is not None and decl.type != "identifier":
                decl = decl.child_by_field_name("declarator")
            if decl is not None:
                name = source[decl.start_byte : decl.end_byte].decode(errors="replace")
                out.setdefault(name, (node.start_point[0] + 1, node.end_point[0] + 1))
            continue
        stack.extend(node.children)
    return out


def rust_body(text: str, name: str) -> str | None:
    m = re.search(rf"\bfn\s+{re.escape(name)}\s*[<(]", text)
    if m is None:
        return None
    opening = text.find("{", m.end())
    return text[opening : _brace_end(text, opening) + 1] if opening >= 0 else None


def _pinned(defs: list[str]) -> dict[str, int]:
    """`-D`s as `decided_live_lines` takes them: a bare `-DNAME` is `NAME=1`, as in C."""
    pinned = pinned_int_macros([d if "=" in d else f"{d}=1" for d in defs if d.startswith("-D")])
    return {**pinned, **{name: 1 for name in KNOWN_DEFINED}}


def module_defines(cmake: Path, repo: Path) -> dict[str, int]:
    """The module's own `-D`s (profile_derive's VER_DEFS for it), plus the tree's DBG=1."""
    from .profile_derive import _default_ver_defs, parse_cmakelists

    module = parse_cmakelists(cmake, repo, flipped_sources=True)
    return _pinned([*_default_ver_defs(module, None), "-DDBG=1"])


# What every TU in the tree compiles with, for a module profile_derive cannot parse.
_TREE_DEFS = ["-DDBG=1", "-DWINVER=0x502", "-D_WIN32_WINNT=0x502", "-D_WIN32_IE=0x603"]


def _find_c(module_dir: Path, repo: Path, rel: str) -> Path | None:
    """The census's C path: module-relative, repo-relative, or (`${var}/x.c`, `../x.c`)
    the one file of that name under the module directory."""
    for cand in (module_dir / rel, repo / rel):
        if "${" not in rel and cand.is_file():
            return cand
    hits = [p for p in module_dir.parent.rglob(Path(rel).name) if p.is_file()]
    return hits[0] if len(hits) == 1 else None


def _find_rs(repo: Path, rel: str) -> Path | None:
    for cand in (repo / rel, repo / "sdk" / "lib" / "rust" / rel):
        if cand.is_file():
            return cand
    return None


def audit(repo: Path, census: Path) -> tuple[list[Miss], dict[str, int]]:
    misses: list[Miss] = []
    stats = {
        "files": 0,
        "functions_with_live_arms": 0,
        "live_arm_calls": 0,
        "skipped": 0,
        "tree_defines_only": 0,
    }
    by_module: dict[Path, dict[str, int] | None] = {}
    for line in census.read_text(errors="replace").splitlines():
        m = _CENSUS.match(line)
        if m is None:
            continue
        cmake, c_rel, rs_rel = repo / m.group(1), m.group(2), m.group(3)
        c_path, rs_path = _find_c(cmake.parent, repo, c_rel), _find_rs(repo, rs_rel)
        if c_path is None or rs_path is None:
            stats["skipped"] += 1
            continue
        rs_rel = str(rs_path.relative_to(repo))
        if cmake not in by_module:
            try:
                by_module[cmake] = module_defines(cmake, repo)
            except Exception:  # profile_derive refuses it: decide with the tree's defines only
                by_module[cmake] = None
        defines = by_module[cmake]
        if defines is None:
            # Fewer arms decidable, so fewer calls required: this side can only UNDER-report.
            stats["tree_defines_only"] += 1
            defines = _pinned(_TREE_DEFS)
        stats["files"] += 1
        c_text = c_path.read_text(errors="replace")
        rs_text = rs_path.read_text(errors="replace")
        live = decided_live_lines(c_text, defines)
        if not live:
            continue
        masked_lines = mask_code(c_text).splitlines()
        fn_like = set(_FN_LIKE_DEFINE.findall(c_text))
        try:
            functions = c_functions(c_path)
        except Exception:
            stats["skipped"] += 1
            continue
        for name, (first, last) in functions.items():
            # Only a conditional opened INSIDE the body: the seal sees the body alone, so a
            # function wrapped whole in a file-scope `#ifdef` already has its calls required.
            rows = [n for n in range(first, last + 1) if n in live and live[n][1] >= first]
            if not rows:
                continue
            body = rust_body(rs_text, name)
            if body is None:
                continue
            stats["functions_with_live_arms"] += 1
            for n in rows:
                for call in _CALL.finditer(masked_lines[n - 1]):
                    callee = call.group(1)
                    if callee in _KEYWORDS or callee in fn_like or callee.isupper():
                        continue
                    stats["live_arm_calls"] += 1
                    if not re.search(rf"\b{re.escape(callee)}\b", body):
                        misses.append(
                            Miss(str(c_path.relative_to(repo)), rs_rel, name, callee, n, live[n][0])
                        )
    return misses, stats


def main(argv: list[str] | None = None) -> int:
    from .config import REACTOS_ROOT

    ap = argparse.ArgumentParser(
        prog="rustconv.live_arm_audit", description=(__doc__ or "").split("\n")[0]
    )
    ap.add_argument("--census", type=Path, default=REACTOS_ROOT / "tools" / "cohab-census.txt")
    ap.add_argument("--json", type=Path, help="write the misses here as JSON")
    args = ap.parse_args(argv)
    misses, stats = audit(REACTOS_ROOT, args.census)
    for x in misses:
        print(f"{x.c_file}:{x.line}: {x.function}: `{x.callee}` ({x.guard}) not in {x.rs_file}")
    print(
        f"live_arm_audit: {stats['files']} C files, {stats['functions_with_live_arms']} "
        f"functions with live arms, {stats['live_arm_calls']} live-arm calls, "
        f"{len(misses)} not named by the Rust ({stats['skipped']} census rows unresolved, "
        f"{stats['tree_defines_only']} decided with the tree's defines only)",
        file=sys.stderr,
    )
    if args.json:
        args.json.write_text(json.dumps([asdict(x) for x in misses], indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
