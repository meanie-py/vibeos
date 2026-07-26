"""Function-like macros visible to a TU, from the REAL preprocessor.

The call seal replaces each runtime-callee occurrence with an opaque token and
demands the model reproduce it "exactly once as the called function identifier".
That contract is only satisfiable when the C occurrence actually LOWERS to one
Rust call. A function-like macro does not:

    CoCreateInstance(CLSID_ProgressDialog, NULL, CLSCTX_INPROC_SERVER,
                     IID_PPV_ARG(IProgressDialog, &op.progress));

`IID_PPV_ARG` expands to *two arguments* (`IID_IProgressDialog, (void**)ppv`).
There is no Rust call for the model to put the token on, so the envelope check
fails through no fault of the model and the whole TU assembles nothing — one bad
worker voids the file. Sealing such a name also protects nothing: the macro name
never appears in the output, so there is no name for the model to get wrong.

Until now the exclusion list was hand-maintained (`TRACE`, `SUCCEEDED`, `FAILED`,
`_countof`, ...) — every entry a function-like macro someone added after eating a
reject. A denylist that must be remembered is a rule that gets broken, so this
derives the set instead: run the module's own compile command with `-E -dM` and
take every `#define NAME(`.

Fleet isolation (AGENTS.md §worktree-discipline 8): `compile_commands.json` is a
generated, per-commit artifact. It is read from THIS worktree only, and a missing
one is a HARD FAIL with the build command to fix it — never a silent fallback to
"seal everything", which is the degradation this module exists to end.
"""

from __future__ import annotations

import json
import re
import shlex
import subprocess
from functools import lru_cache
from pathlib import Path

# Builds run at a canonical bind-mount path (tools/canon-run.sh), so every path in
# compile_commands.json is spelled under it regardless of which worktree produced
# the entry. Rewriting the prefix to THIS worktree is what keeps the reads local.
CANONICAL_BUILD_ROOT = "/tmp/vibeos"

_DEFINE_FN_LIKE = re.compile(r"^#define\s+([A-Za-z_]\w*)\(", re.M)
_ARCHES = ("i386", "amd64", "arm64")


class MacroAuthorityError(RuntimeError):
    pass


def _compile_entry(repo_root: Path, arch: str, source: Path) -> list[str] | None:
    db_path = repo_root / f"build-{arch}" / "compile_commands.json"
    if not db_path.is_file():
        return None
    try:
        db = json.loads(db_path.read_text())
    except (OSError, ValueError) as error:
        raise MacroAuthorityError(f"unreadable {db_path}: {error}") from error
    wanted = source.name
    for entry in db:
        recorded = entry.get("file", "")
        if not recorded.endswith(wanted):
            continue
        # Match the whole trailing path, not just the basename: several modules
        # ship a `shell.c`, and the wrong module's flags give the wrong macros.
        if not recorded.replace(CANONICAL_BUILD_ROOT, str(repo_root)).endswith(
            str(source.relative_to(repo_root))
        ):
            continue
        argv = shlex.split(entry.get("command") or " ".join(entry.get("arguments", [])))
        return [a.replace(CANONICAL_BUILD_ROOT, str(repo_root)) for a in argv]
    return None


def _includer_entry(repo_root: Path, arch: str, header: Path) -> list[str] | None:
    """A compiled TU in the header's own directory that `#include`s it.

    A HEADER has no compile command of its own, but its macro environment is by
    definition whatever its includers see — so borrowing an includer's flags is exact,
    not a guess. This is what makes a `static inline` helper header translatable at all:
    wldap32.h is 62 marshalling helpers that every TU inlines a private copy of, so no
    .c defines them and the translated modules call 300-odd functions that would
    otherwise have no Rust definition anywhere.

    Restricted to the header's own directory: a module-private header is included by its
    own TUs, and matching tree-wide would pick an arbitrary module's flags.
    """
    db_path = repo_root / f"build-{arch}" / "compile_commands.json"
    if not db_path.is_file():
        return None
    try:
        db = json.loads(db_path.read_text())
    except (OSError, ValueError) as error:
        raise MacroAuthorityError(f"unreadable {db_path}: {error}") from error
    pattern = re.compile(r'^\s*#\s*include\s+[<"]' + re.escape(header.name) + r'[>"]', re.M)
    for entry in sorted(db, key=lambda e: e.get("file", "")):
        recorded = entry.get("file", "").replace(CANONICAL_BUILD_ROOT, str(repo_root))
        tu = Path(recorded)
        if tu.parent != header.parent or tu.suffix not in (".c", ".cpp"):
            continue
        try:
            if not pattern.search(tu.read_text(encoding="utf-8", errors="replace")):
                continue
        except OSError:
            continue
        argv = shlex.split(entry.get("command") or " ".join(entry.get("arguments", [])))
        return [a.replace(CANONICAL_BUILD_ROOT, str(repo_root)) for a in argv]
    return None


def _preprocess_defines(argv: list[str]) -> str:
    stripped: list[str] = []
    skip = False
    for arg in argv:
        if skip:
            skip = False
            continue
        if arg in ("-o", "-MF", "-MT", "-MQ"):
            skip = True
            continue
        if arg in ("-c", "-MD", "-MMD"):
            continue
        stripped.append(arg)
    result = subprocess.run([*stripped, "-E", "-dM"], capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise MacroAuthorityError(
            "preprocessor failed while deriving the function-like macro set:\n"
            + result.stderr.strip()[:600]
        )
    return result.stdout


@lru_cache(maxsize=64)
def function_like_macros(repo_root: Path, source: Path) -> frozenset[str]:
    """Every `#define NAME(...)` visible to ``source``, unioned over built arches.

    Unioned deliberately: a name that is a macro on ANY arch has no single call
    lowering, and excluding it from the seal can only ever *relax* a check. Sealing
    it on the one arch where it happens not to be a macro would reintroduce the
    unsatisfiable envelope for that build.
    """
    names: set[str] = set()
    found: list[str] = []
    is_header = source.suffix in (".h", ".hpp")
    for arch in _ARCHES:
        argv = _compile_entry(repo_root, arch, source)
        if argv is None and is_header:
            # See _includer_entry: a header's macro set is its includers' macro set.
            argv = _includer_entry(repo_root, arch, source)
        if argv is None:
            continue
        names |= set(_DEFINE_FN_LIKE.findall(_preprocess_defines(argv)))
        found.append(arch)
    if not found:
        via = " and no compiled TU in its own directory #includes it" if is_header else ""
        raise MacroAuthorityError(
            f"no compile command for {source} in any build-<arch>/compile_commands.json "
            f"under {repo_root}{via}. The call seal needs the real function-like macro set; "
            f"without it every macro occurrence becomes an unsatisfiable envelope and "
            f"the TU assembles nothing.\n"
            f"fix: ./vibeos-cli build --arch i386"
        )
    return frozenset(names)
