"""Which C typedefs are provably POINTERS — the authority ``forwarder_with_null`` needs.

``trivial_c`` may only spell a literal ``NULL`` as ``core::ptr::null_mut()`` at a parameter
whose type is a pointer, and it proves that from the callee's signature rather than assuming
it. For ``T *p`` the star is in the spelling and the proof is free. For ``_locale_t`` it is
not: the star lives in a ``typedef`` in a header, so the position is unprovable and the whole
function refuses to lower.

That one typedef was the entire gap. Across msvcrt's 1,792 functions, ``_locale_t`` is the
**only** type ever seen at a refused literal-``NULL`` position, and it blocks **145** free
lowerings — 76 of them in ``mbcs.c``, 40% of that file. The CRT is written as locale pairs
(``X(args)`` delegating to ``X_l(args, NULL)``), so the idiom is the file, not an edge case.

## Where the proof comes from

The module's real include closure, parsed with tree-sitter — the same headers the C compiler
reads. Not the bindgen island: an island carries only what its allowlist names, and a tight
one (``msvcrt-sys`` is 312 lines of deliberately-curated state structs) would answer "unknown"
for almost everything. Headers answer for every typedef the TU can actually see.

The search path is **source headers only** — no ``build-<arch>`` roots. A typedef that exists
only in a generated header therefore stays unproven, which costs a lowering and never buys a
wrong one. That is the right side to fail on, and it keeps this free of the fleet-isolation
hazard in AGENTS.md §8 (a generated header from a *sibling* worktree is another commit's
state, and a verdict that depends on which siblings exist is not a verdict).

Concretely, and before anyone re-derives it: ``HANDLE``, ``PVOID`` and ``LPCSTR`` come back
**unproven**, because ``psdk/winnt.h`` is generated from ``sdk/include/xdk/winnt.template.h``
into ``build-<arch>/`` and is not on this path. That is deliberate. Reaching for a build dir
would make a $0 layer's output depend on whether one happens to exist, and it buys nothing
measurable: of the 145 literal-``NULL`` positions msvcrt refuses, **every single one** is
``_locale_t``. Widen this only against a measurement that says a Win32 handle type actually
blocks lowerings somewhere.

## Three rules keep it honest

* **Dead preprocessor branches are not evidence.** ``#ifndef __REACTOS__`` arms never compile
  in this tree, so a typedef inside one is skipped — the same fixed-truth rule the ``case``
  label audit uses, and for the same reason it was added there (a false alarm read out of an
  arm nobody builds).
* **Disagreement refuses.** A name typedef'd as a pointer in one reachable header and as a
  non-pointer in another is dropped rather than resolved by precedence. Header order is a
  property of the preprocessor state, and we do not run the preprocessor.
* **Only plain pointer declarators count.** A function-pointer typedef lowers to
  ``Option<unsafe extern "C" fn(..)>``, whose null is ``None`` and not ``null_mut()``; an
  array typedef is not a pointer at all. Both are refused rather than guessed.

Being wrong here would be a rustc type error at candidate-check, never a silent miscompile —
but the asymmetry from ``trivial_c`` still holds (nobody reviews a free translation), so
anything unproven simply does not lower and the model gets paid as normal.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping
from pathlib import Path

from tree_sitter import Node, Parser

from .config import REACTOS_ROOT

# The dead-branch rule and the include-path walker both already exist, are already tested,
# and are already the tree's answer to "what does the compiler actually see". A second copy
# here would be a second thing to keep true.
from .profile_derive import (
    _SDK_SEARCH,
    IncludeEnv,
    _in_dead_preproc_branch,
    walk_includes,
)
from .scanner import CPP_LANG

REPO_ROOT = REACTOS_ROOT  # the invoked tree, not this file's checkout

NULL_MUT = "core::ptr::null_mut()"
NULL_CONST = "core::ptr::null()"

#: Headers reached from one ``.c``. Deep CRT closures run ~150 headers; the cap only exists
#: so a pathological include cycle cannot turn a translate into a hang.
_MAX_HEADERS = 2000

_DECL_NAME_NODES = frozenset({"type_identifier", "identifier", "primitive_type"})


def _text(node: Node, src: bytes) -> str:
    return src[node.start_byte : node.end_byte].decode("utf-8", "replace")


def _declarator(node: Node, src: bytes) -> tuple[str, int] | None:
    """``(name, star_count)`` for a typedef declarator, or ``None`` if it is not our shape.

    Peels ``pointer_declarator`` chains (``char **strv_t`` is two stars and still a pointer)
    down to a bare name. Anything else under the stars — a ``function_declarator``, an
    ``array_declarator``, a parenthesised declarator — returns ``None``: see the module
    docstring for why those must refuse rather than round to "pointer".
    """
    stars = 0
    cur: Node | None = node
    while cur is not None and cur.type == "pointer_declarator":
        stars += 1
        cur = cur.child_by_field_name("declarator")
    if cur is None or cur.type not in _DECL_NAME_NODES:
        return None
    return _text(cur, src), stars


def _pointee_is_const(node: Node, src: bytes) -> bool:
    """True when ``const`` binds to the POINTEE, i.e. ``typedef const char *PCSTR``.

    tree-sitter puts that qualifier among the ``type_definition``'s own children, before the
    ``type`` field. The other spelling, ``typedef char *const CPTR``, parks its ``const``
    *inside* the ``pointer_declarator`` — there the pointer is const and the pointee is not,
    so the null stays ``null_mut()``. Reading only the outer children is what keeps those two
    apart.
    """
    for child in node.children:
        if child.type == "declarator":  # pragma: no cover - defensive; fields are unnamed here
            break
        if child.type == "type_qualifier" and _text(child, src) == "const":
            return True
    return False


def _scan_header(text: str) -> tuple[dict[str, str | None], dict[str, str]]:
    """``({name: null spelling or None}, {alias: underlying type name})`` for one file.

    ``None`` records a name that IS typedef'd here and is NOT a pointer — kept, not dropped,
    because it is exactly what convicts a conflicting definition elsewhere.
    """
    src = text.encode("utf-8", "replace")
    root = Parser(CPP_LANG).parse(src).root_node
    verdicts: dict[str, str | None] = {}
    aliases: dict[str, str] = {}

    stack = [root]
    while stack:
        node = stack.pop()
        if node.type != "type_definition":
            stack.extend(node.children)
            continue
        if _in_dead_preproc_branch(node, src):
            continue
        type_node = node.child_by_field_name("type")
        const_pointee = _pointee_is_const(node, src)
        for index in range(node.child_count):
            if node.field_name_for_child(index) != "declarator":
                continue
            child = node.child(index)
            if child is None:  # pragma: no cover - the index came from child_count
                continue
            parsed = _declarator(child, src)
            if parsed is None:
                continue
            name, stars = parsed
            if stars:
                # `const` only reaches the pointee through ONE star: for `const char **`
                # the pointee is `const char *`, itself mutable, so the null is null_mut().
                verdicts[name] = NULL_CONST if (stars == 1 and const_pointee) else NULL_MUT
            else:
                verdicts[name] = None
                if type_node is not None and type_node.type == "type_identifier":
                    aliases[name] = _text(type_node, src)
    return verdicts, aliases


_CACHE: dict[tuple[Path, float, int], tuple[dict[str, str | None], dict[str, str]]] = {}


def _scan_cached(path: Path) -> tuple[dict[str, str | None], dict[str, str]]:
    try:
        stat = path.stat()
        key = (path, stat.st_mtime, stat.st_size)
    except OSError:
        return {}, {}
    hit = _CACHE.get(key)
    if hit is None:
        try:
            hit = _scan_header(path.read_text(encoding="utf-8", errors="replace"))
        except OSError:
            hit = ({}, {})
        _CACHE[key] = hit
    return hit


def pointer_typedefs(paths: Iterable[Path]) -> dict[str, str]:
    """``{typedef name: null spelling}`` for every name *provably* a plain pointer.

    A name defined inconsistently across the given files is omitted entirely (see the
    module docstring's disagreement rule), as is every name whose only definitions are
    non-pointers.
    """
    verdicts: dict[str, set[str | None]] = {}
    aliases: dict[str, set[str]] = {}
    for path in paths:
        file_verdicts, file_aliases = _scan_cached(path)
        for name, spelling in file_verdicts.items():
            verdicts.setdefault(name, set()).add(spelling)
        for name, underlying in file_aliases.items():
            aliases.setdefault(name, set()).add(underlying)

    resolved: dict[str, str] = {}
    for name, spellings in verdicts.items():
        if len(spellings) != 1:
            continue  # defined inconsistently across the closure — refuse it
        only = next(iter(spellings))
        if only is not None:  # a proven NON-pointer is recorded, but is not an answer
            resolved[name] = only

    # `typedef _locale_t handle_t;` is a pointer too. Iterate to a fixpoint rather than
    # once, so a chain of aliases resolves; an alias with two different underlying types,
    # or one whose target is not (yet) a proven pointer, stays out.
    changed = True
    while changed:
        changed = False
        for name, underlying in aliases.items():
            if name in resolved or len(underlying) != 1:
                continue
            target = next(iter(underlying))
            if target in resolved:
                resolved[name] = resolved[target]
                changed = True
    return resolved


def source_include_env(c_file: Path, repo_root: Path = REPO_ROOT) -> IncludeEnv:
    """Include search path for *c_file*, from checked-in headers only.

    Deliberately not :func:`profile_derive.build_include_env`: that one needs a parsed
    ``CMakeLists`` and reaches into ``build-<arch>`` for generated headers. Neither is
    available or wanted on the translate path — see the module docstring.
    """
    module_dir = c_file.resolve().parent
    search: list[tuple[Path, str]] = [(module_dir, "module")]
    if (module_dir / "include").is_dir():
        search.append((module_dir / "include", "module"))
    search.extend((repo_root / rel, "sdk") for rel in _SDK_SEARCH)
    return IncludeEnv(
        repo_root=repo_root,
        arch="i386",
        module_dir=module_dir,
        build_dir=None,
        search=[(d, kind) for d, kind in search if d.is_dir()],
    )


def pointer_typedefs_for_source(
    c_file: Path,
    repo_root: Path = REPO_ROOT,
) -> dict[str, str]:
    """``{typedef name: null spelling}`` visible from *c_file*'s own include closure."""
    c_file = c_file.resolve()
    env = source_include_env(c_file, repo_root)
    closure = walk_includes([c_file], env)
    headers = list(closure.headers)[:_MAX_HEADERS]
    return pointer_typedefs([c_file, *headers])


def describe(found: Mapping[str, str], c_file: Path) -> str:
    """One status line for the translate log — silence here would hide a $0 layer failing."""
    if not found:
        return f"[typedefs] {c_file.name}: no pointer typedefs proven from the include closure"
    return (
        f"[typedefs] {c_file.name}: {len(found)} pointer typedefs proven (NULL is spellable there)"
    )
