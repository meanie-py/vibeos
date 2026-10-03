"""Language-neutral substrate dependency and compiler-root analysis.

The translation frontend may be C or C++; this module only consumes compiler
diagnostics, generated Rust, and an optional source file.  It answers the useful
question behind a large rustc cascade: which distinct authorities are missing,
where can each one be derived, and which deterministic route owns the fix?
"""

from __future__ import annotations

import json
import re
import subprocess
from collections import Counter, defaultdict
from pathlib import Path

from .config import REACTOS_ROOT

_ROOT_PATTERNS = (
    ("type", re.compile(r"cannot find type `([^`]+)`")),
    ("value", re.compile(r"cannot find value `([^`]+)`")),
    ("function", re.compile(r"cannot find function `([^`]+)`")),
    ("constructor", re.compile(r"cannot find (?:tuple struct|tuple variant) `([^`]+)`")),
    ("macro", re.compile(r"cannot find macro `([^`]+)`")),
    (
        "module",
        re.compile(r"failed to resolve: use of unresolved module or unlinked crate `([^`]+)`"),
    ),
)
_IDENTIFIER = re.compile(r"^[A-Za-z_]\w*$")


def diagnostic_root(diagnostic: dict) -> tuple[str, str] | None:
    """Return ``(kind, symbol)`` for an unresolved rustc diagnostic."""
    message = str(diagnostic.get("message") or "")
    for kind, pattern in _ROOT_PATTERNS:
        match = pattern.search(message)
        if match is not None and _IDENTIFIER.fullmatch(match.group(1)):
            return kind, match.group(1)
    return None


def _chunks(items: set[str], size: int = 80):
    ordered = sorted(items)
    for start in range(0, len(ordered), size):
        yield ordered[start : start + size]


def _searchable_texts(root: Path, glob: str) -> tuple[tuple[Path, str], ...]:
    """Every file under `root` matching `glob` that ripgrep would have searched: tracked
    plus untracked-but-not-ignored (a crate's `target/` stays out, as it did under rg).

    Read here, in Python, as latin-1 -- lossless for any byte, so a stray cp1252 byte in a
    header can never turn a file into 'binary' and silently drop it. That, and rg's exit 2
    on ANY unreadable file (which discarded every real match of the call), were the two
    ways the old `rg` shell-out could answer 'no authority' for a symbol that had one."""
    inside = subprocess.run(
        ["git", "rev-parse", "--is-inside-work-tree"], cwd=root, capture_output=True
    )
    if inside.returncode == 0:
        listing = subprocess.run(
            ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", glob],
            cwd=root,
            capture_output=True,
            check=True,
        )
        rels = [r for r in listing.stdout.decode("utf-8", "surrogateescape").split("\0") if r]
        paths = [(root / rel).resolve() for rel in rels]
    else:
        # Outside any repository rg applied no ignore rules either: every matching file.
        paths = sorted(p.resolve() for p in root.rglob(glob))
    return tuple((p, p.read_bytes().decode("latin-1")) for p in paths if p.is_file())


_Texts = dict[tuple[Path, str], tuple[tuple[Path, str], ...]]


def _rg_matches(
    pattern: str, roots: list[Path], glob: str, texts: _Texts | None = None
) -> list[tuple[Path, str]]:
    """(file, matched text) for every match of `pattern`, line-scoped like `rg -o`.

    `texts` caches file contents across the chunked calls of ONE authorities pass --
    never longer, so a crate rewritten later in the same process is read afresh."""
    texts = {} if texts is None else texts
    regex = re.compile(pattern)
    matches = []
    for root in roots:
        if not root.exists():
            continue
        key = (root.resolve(), glob)
        if key not in texts:
            texts[key] = _searchable_texts(*key)
        for path, text in texts[key]:
            for m in regex.finditer(text):
                if "\n" not in m.group(0):  # rg matches within a line, never across
                    matches.append((path, m.group(0)))
    return matches


def _rust_authorities(symbols: set[str], roots: list[Path]) -> dict[str, set[str]]:
    found: dict[str, set[str]] = defaultdict(set)
    texts: _Texts = {}
    for chunk in _chunks(symbols):
        alternatives = "|".join(re.escape(item) for item in chunk)
        pattern = (
            rf"\b(?:struct|union|enum|type|const|static|fn)\s+(?:r#)?(?:{alternatives})\b"
            rf"|\bmacro_rules!\s+(?:{alternatives})\b"
        )
        for path, matched in _rg_matches(pattern, roots, "*.rs", texts):
            for symbol in chunk:
                if re.search(rf"(?<![A-Za-z0-9_])(?:r#)?{re.escape(symbol)}\b", matched):
                    found[symbol].add(str(path))
    return found


def _sdk_authorities(symbols: set[str], root: Path) -> dict[str, set[str]]:
    found: dict[str, set[str]] = defaultdict(set)
    texts: _Texts = {}
    for chunk in _chunks(symbols):
        alternatives = "|".join(re.escape(item) for item in chunk)
        pattern = rf"\b(?:{alternatives})\b"
        for glob in ("*.h", "*.idl"):
            for path, matched in _rg_matches(pattern, [root], glob, texts):
                if matched in symbols:
                    found[matched].add(str(path))
    return found


def _source_authorities(symbols: set[str], source: Path | None) -> dict[str, set[str]]:
    if source is None or not source.is_file():
        return {}
    text = source.read_text(encoding="utf-8", errors="replace")
    found: dict[str, set[str]] = defaultdict(set)
    for symbol in symbols:
        escaped = re.escape(symbol)
        definition = (
            rf"^\s*#\s*define\s+{escaped}\b"
            rf"|\b(?:struct|union|enum|class)\s+{escaped}\b"
            rf"|\b{escaped}\s*\([^;{{}}]*\)\s*\{{"
        )
        if re.search(definition, text, re.M):
            found[symbol].add(str(source.resolve()))
    return found


def _route(authority: str, kind: str) -> str:
    if authority == "candidate":
        return "repair_scope_or_import"
    if authority == "shell32_shared":
        return "import_existing"
    if authority == "shell32_module":
        return "promote_shared_authority"
    if authority == "project_rust":
        return "reuse_project_binding"
    if authority == "source_local":
        return "generate_local_or_bridge"
    if authority == "sdk_header":
        return "generate_from_sdk"
    if kind == "module":
        return "repair_model_path"
    return "authority_backlog"


def analyze_report(
    report: dict,
    *,
    candidate_sources: dict[str, Path | None] | None = None,
    repo_root: Path = REACTOS_ROOT,
) -> dict:
    """Cluster diagnostics and locate deterministic authorities for each root."""
    candidate_sources = candidate_sources or {}
    occurrences: dict[tuple[str, str, str], Counter[str]] = defaultdict(Counter)
    category_counts: dict[str, Counter[str]] = defaultdict(Counter)
    unattributed: dict[str, int] = defaultdict(int)
    candidate_text: dict[str, str] = {}

    for result in report.get("results", []):
        name = result["name"]
        candidate = Path(result["candidate"])
        candidate_text[name] = (
            candidate.read_text(encoding="utf-8", errors="replace") if candidate.is_file() else ""
        )
        for arch, arch_result in result.get("arches", {}).items():
            for diagnostic in arch_result.get("diagnostics", []):
                category_counts[arch][diagnostic.get("category", "other")] += 1
                root = diagnostic_root(diagnostic)
                if root is None:
                    unattributed[arch] += 1
                    continue
                kind, symbol = root
                occurrences[(name, kind, symbol)][arch] += 1

    symbols = {symbol for _name, _kind, symbol in occurrences}
    shell_root = repo_root / "sdk/lib/rust/shell32-rust/src"
    project_root = repo_root / "sdk/lib/rust"
    shell_authority = _rust_authorities(symbols, [shell_root])
    project_authority = _rust_authorities(symbols, [project_root])
    sdk_authority = _sdk_authorities(symbols, repo_root / "sdk/include")
    source_authority = {
        name: _source_authorities(symbols, source) for name, source in candidate_sources.items()
    }

    roots = []
    arches = sorted(category_counts)
    for (candidate, kind, symbol), by_arch in occurrences.items():
        paths: set[str] = set()
        if re.search(
            rf"\b(?:struct|enum|union|type|const|static|fn)\s+(?:r#)?{re.escape(symbol)}\b",
            candidate_text[candidate],
        ):
            authority = "candidate"
        elif symbol in shell_authority:
            paths = shell_authority[symbol]
            shared_names = {"types.rs", "shellcom.rs"}
            authority = (
                "shell32_shared"
                if any(Path(path).name in shared_names for path in paths)
                else "shell32_module"
            )
        elif symbol in project_authority:
            authority = "project_rust"
            paths = project_authority[symbol]
        elif symbol in source_authority.get(candidate, {}):
            authority = "source_local"
            paths = source_authority[candidate][symbol]
        elif symbol in sdk_authority:
            authority = "sdk_header"
            paths = sdk_authority[symbol]
        else:
            authority = "unknown"
        roots.append(
            {
                "candidate": candidate,
                "kind": kind,
                "symbol": symbol,
                "authority": authority,
                "route": _route(authority, kind),
                "authority_paths": sorted(paths)[:8],
                "occurrences_by_arch": {arch: by_arch.get(arch, 0) for arch in arches},
                "impact": max(by_arch.values(), default=0),
                "cross_arch_stable": len({by_arch.get(arch, 0) for arch in arches}) <= 1,
            }
        )
    roots.sort(key=lambda item: (-item["impact"], item["symbol"], item["candidate"]))
    authorities = Counter(item["authority"] for item in roots)
    routes = Counter(item["route"] for item in roots)
    generation_plan: dict[str, list[str]] = defaultdict(list)
    for item in roots:
        route = item["route"]
        kind = item["kind"]
        if route == "generate_from_sdk":
            bucket = {
                "type": "sdk_types",
                "constructor": "sdk_types",
                "function": "sdk_functions",
                "value": "sdk_values",
                "macro": "sdk_values",
            }.get(kind, "sdk_modules")
        elif route == "import_existing":
            bucket = "shell32_authorities"
        elif route == "promote_shared_authority":
            bucket = "shell32_promotions"
        elif route == "reuse_project_binding":
            bucket = "project_bindings"
        elif route == "generate_local_or_bridge":
            bucket = "local_generation_or_bridges"
        elif route == "repair_scope_or_import":
            bucket = "scope_repairs"
        else:
            bucket = "unknown_authorities"
        generation_plan[bucket].append(item["symbol"])
    return {
        "schema": 1,
        "unique_unresolved_roots": len(roots),
        "category_counts_by_arch": {
            arch: dict(sorted(counts.items())) for arch, counts in sorted(category_counts.items())
        },
        "unattributed_diagnostics_by_arch": dict(sorted(unattributed.items())),
        "authority_counts": dict(sorted(authorities.items())),
        "route_counts": dict(sorted(routes.items())),
        "generation_plan": {
            name: sorted(set(items)) for name, items in sorted(generation_plan.items())
        },
        "roots": roots,
    }


def print_analysis(analysis: dict, *, as_json: bool = False, limit: int = 25) -> None:
    if as_json:
        print(json.dumps(analysis, indent=2, sort_keys=True))
        return
    print(f"substrate-plan: {analysis.get('unique_unresolved_roots', 0)} unresolved roots")
    print(
        "  routes: "
        + ", ".join(f"{route}={count}" for route, count in analysis.get("route_counts", {}).items())
    )
    for root in analysis.get("roots", [])[:limit]:
        print(
            f"  {root['symbol']}: impact={root['impact']}, kind={root['kind']}, "
            f"authority={root['authority']}, route={root['route']}"
        )
