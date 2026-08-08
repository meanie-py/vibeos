"""c_defs.py — which functions does a C translation unit DEFINE?

Extracted from `fn_coverage.py` so more than one auditor can use it. `export_audit` needs
exactly this oracle: an export with no Rust definition is only a defect if the C that used to
define it is no longer compiled, and answering that means reading the C.

The scanner is deliberately textual and deliberately not a preprocessor — see `_strip_if_zero`
for the one residual false positive that follows from that.
"""

from __future__ import annotations

import re
from pathlib import Path

from .profile_derive import ProfileDeriveError, parse_cmakelists

# Words that begin a statement, not a definition, when they sit at column 0 before a `(`.
_NOT_A_FN = {
    "if",
    "else",
    "for",
    "while",
    "switch",
    "return",
    "do",
    "sizeof",
    "case",
    "default",
    "typedef",
    "struct",
    "union",
    "enum",
    "static",
    "extern",
    "const",
    "void",
    "break",
    "continue",
    "goto",
}

_C_COMMENT = re.compile(r"/\*.*?\*/", re.S)
_C_LINE_COMMENT = re.compile(r"//[^\n]*")
_CPP_DIRECTIVE = re.compile(r"^[ \t]*#.*(?:\\\n.*)*$", re.M)
# ReactOS style puts the return type on its own line, so a definition's NAME starts a line;
# the one-line form (`static BOOL Foo(`, `VOID WINAPI Bar(`) puts it last before the `(`.
_C_CANDIDATE = re.compile(r"^(?:[A-Za-z_][\w \t\*]*?[ \t\*])?([A-Za-z_]\w*)[ \t]*\(", re.M)


def _strip_if_zero(text: str) -> str:
    """Drop `#if 0 ... #endif` regions — the C's own "commented out" idiom.

    Not a preprocessor: only the unambiguous case is handled. This tool deliberately does NOT
    evaluate `#if`/`#ifdef` in general, so a function that exists only under a configuration
    this build does not select can still read as missing. That is the known residual false
    positive, and the reason a finding here is a REVIEW item rather than an automatic bug.
    """
    out, depth, lines = [], 0, text.splitlines(keepends=True)
    for line in lines:
        s = line.strip()
        if depth:
            if s.startswith("#if"):
                depth += 1
            elif s.startswith("#endif"):
                depth -= 1
            elif depth == 1 and s.startswith("#else"):
                depth = 0  # the live branch starts here
            continue
        if re.match(r"#\s*if\s+0\b", s):
            depth = 1
            continue
        out.append(line)
    return "".join(out)


_IF_DBG = re.compile(r"#\s*if\s+DBG\b")


def _inside_if_dbg(text: str) -> str:
    """Just the text INSIDE `#if DBG ... #endif` regions.

    A body that exists only under DBG is not in the build the smoke tests gate on, so calling it
    "dropped" reads as a defect when it is a configuration. It is not nothing either — a DBG
    build would want it — so it gets its own bucket instead of being silently discarded.
    """
    out, depth = [], 0
    for line in text.splitlines(keepends=True):
        s = line.strip()
        if depth:
            if s.startswith("#if"):
                depth += 1
            elif s.startswith("#endif"):
                depth -= 1
                continue
            elif depth == 1 and s.startswith("#else"):
                depth = 0  # the non-DBG branch starts here
                continue
            if depth:
                out.append(line)
            continue
        if _IF_DBG.match(s):
            depth = 1
    return "".join(out)


def _after_params(text: str, open_paren: int) -> str:
    """The first meaningful characters after the parameter list that opens at `open_paren`."""
    depth = 0
    i = open_paren
    while i < len(text):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                break
        i += 1
    else:
        return ""
    return text[i + 1 : i + 400].lstrip()


def c_functions(text: str) -> set[str]:
    """Names of functions DEFINED in this C translation unit.

    A name at column 0 followed by `(` is not enough: SAL annotations
    (`_Requires_lock_held_(..)`, `_Function_class_(..)`) and Wine's
    `WINE_DEFAULT_DEBUG_CHANNEL(..)` sit exactly there, immediately before the definition they
    decorate, and would otherwise be reported as functions the translation dropped. What
    separates a definition from a macro invocation is what FOLLOWS its parameter list: a body.
    A `;` is a prototype; anything else means the line was an annotation and the real
    definition is further down.
    """
    text = _C_COMMENT.sub("", text)
    text = _C_LINE_COMMENT.sub("", text)
    text = _strip_if_zero(text)
    text = _CPP_DIRECTIVE.sub("", text)
    names = set()
    for m in _C_CANDIDATE.finditer(text):
        n = m.group(1)
        if n in _NOT_A_FN:
            continue
        tail = _after_params(text, m.end() - 1)
        # A K&R-ish definition may carry a trailing attribute or annotation before its brace.
        if not tail.startswith("{"):
            continue
        # `CSR_API(SrvFoo)` / `CON_API(SrvBar, ..)` — an ALL-CAPS macro that EXPANDS to a
        # definition. The macro is followed by a body, so it passes the test above, but the
        # function the C actually defines is its first argument. winsrv writes every one of its
        # ~40 server routines this way.
        # The first argument must be a BARE identifier that ends right there (`,` or `)`).
        # An ordinary function whose name is merely SHOUTY looks identical up to this point,
        # and its first parameter is a DECLARATION — `GDI_ROUND(FLOAT val)`,
        # `INTERNAL_YDSTOWS(XFORM *xForm, ..)`. Without the terminator test those two report
        # `FLOAT` and `XFORM` as dropped functions, and the real name is never checked at all.
        if n.isupper() and "_" in n:
            arg = re.match(r"\([ \t\n]*(\w+)[ \t\n]*[,)]", text[m.end() - 1 :])
            if arg and arg.group(1) not in ("void", "VOID"):
                names.add(arg.group(1))
                continue
        names.add(n)
    return names


_ADD_LIBRARY = re.compile(r"^[ \t]*add_(?:library|executable)\([ \t]*([A-Za-z_][\w.-]*)", re.M)
_LINK_LIBS = re.compile(
    r"^[ \t]*target_link_libraries\([ \t]*([A-Za-z_][\w.-]*)(?P<rest>[^)]*)\)", re.M | re.S
)
_TOKEN = re.compile(r"[A-Za-z_][\w.-]*")
# Keywords target_link_libraries() takes that are not library names.
_LINK_KEYWORDS = frozenset({"PUBLIC", "PRIVATE", "INTERFACE", "debug", "optimized", "general"})


def _target_index(repo_root: Path) -> dict[str, Path]:
    """Every `add_library`/`add_executable` target in the tree -> its CMakeLists.

    Built once and cached on the function: the walk is ~4k files and the audit resolves a link
    closure per module.
    """
    cached = getattr(_target_index, "_cache", {}).get(repo_root)
    if cached is not None:
        return cached
    index: dict[str, Path] = {}
    for cml in repo_root.rglob("CMakeLists.txt"):
        if any(part.startswith(("build-", ".git")) for part in cml.parts):
            continue
        text = _read_text(cml)
        for m in _ADD_LIBRARY.finditer(text):
            index.setdefault(m.group(1), cml)
    store = getattr(_target_index, "_cache", None)
    if store is None:
        store = {}
        _target_index._cache = store  # type: ignore[attr-defined]
    store[repo_root] = index
    return index


def _read_text(p: Path) -> str:
    return p.read_text(encoding="utf-8", errors="replace")


def _linked_targets(cml: Path, target: str) -> list[str]:
    """The library names `target_link_libraries(<target> …)` names, minus keywords and vars."""
    out: list[str] = []
    for m in _LINK_LIBS.finditer(_read_text(cml)):
        if m.group(1) != target:
            continue
        for line in m.group("rest").splitlines():
            code = line.split("#", 1)[0]
            # `${PSEH_LIB}` / `${GDI32_RUST_LIB}` — a variable, not a resolvable target name.
            code = re.sub(r"\$\{[^}]*\}", " ", code)
            out += [t for t in _TOKEN.findall(code) if t not in _LINK_KEYWORDS]
    return out


def compiled_tus(
    c_dir: Path, repo_root: Path, arch: str = "i386", target: str | None = None
) -> list[Path]:
    """Every .c that ends up IN this module's binary, following the link.

    Uses the CMake interpreter rather than a regex over the file: a cohab flip comments the
    translated TUs out of the source list, and telling a commented-out entry from a live one
    means resolving `if(ARCH)`, `set`, `list(APPEND)` and `add_pch` the way the build does.
    A conditional entry is KEPT — a source behind an unresolved guard may well be compiled,
    and treating it as dead is the direction that invents findings.

    THE MODULE'S OWN SOURCE LIST IS NOT THE ANSWER. gdi32's is EMPTY — fully flipped — while
    190 of its exports are still C, because they are compiled into the `winegdi` static library
    in its `wine/` subdirectory and into the sibling `gdi32_vista_static`. Reading only
    `gdi32/CMakeLists.txt` calls all 190 dropped exports. So resolve `target_link_libraries`
    transitively against a tree-wide index of `add_library` targets; a name that does not
    resolve (an import lib, a `${VAR}`) is simply not a source of C and is skipped.
    """
    index = _target_index(repo_root)
    root_cml = c_dir / "CMakeLists.txt"
    root_name = parse_cmakelists(root_cml, repo_root, arch=arch, target=target).name

    tus: list[Path] = []
    seen_targets: set[str] = set()
    pending: list[tuple[Path, str]] = [(root_cml, root_name)]
    while pending:
        cml, name = pending.pop()
        if name in seen_targets:
            continue
        seen_targets.add(name)
        try:
            mod = parse_cmakelists(cml, repo_root, arch=arch, target=name)
        except ProfileDeriveError:
            continue
        tus += [
            e.path
            for e in mod.entries
            if e.kind == "tu" and e.path is not None and e.path.suffix == ".c" and e.path.is_file()
        ]
        for dep in _linked_targets(cml, name):
            dep_cml = index.get(dep)
            if dep_cml is not None and dep not in seen_targets:
                pending.append((dep_cml, dep))

    # Same .c can arrive through two targets that both compile it.
    return sorted({p for p in tus})


def c_provided_names(paths: list[Path]) -> set[str]:
    """Every function name defined by the given C translation units."""
    names: set[str] = set()
    for p in paths:
        names |= c_functions(p.read_text(encoding="utf-8", errors="replace"))
    return names
