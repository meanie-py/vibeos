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

_DEFINE_FN_LIKE = re.compile(r"^[ \t]*#[ \t]*define\s+([A-Za-z_]\w*)\(", re.M)
# `#define NAME OTHER` where the whole body is one bare identifier -- an ALIAS. It is
# object-like, so `_DEFINE_FN_LIKE` cannot see it, but invoking it invokes whatever it
# names, so an alias to a function-like macro IS function-like at every call site.
_DEFINE_ALIAS = re.compile(
    r"^[ \t]*#[ \t]*define\s+([A-Za-z_]\w*)[ \t]+([A-Za-z_]\w*)[ \t]*$", re.M
)
_ARCHES = ("i386", "amd64", "arm64")

#: Headers translated THROUGH one of their includers (`translate --via`), header -> TU.
#: A shared header's inline helpers (identify_funcs.h's 44 `AtaDev*`, in sdk/include) have no
#: compile command, no includer in their own directory, and a different macro environment in
#: every module that pulls them in. The one that is right is the module being ported, so the
#: caller names a TU of it that reaches the header, and every question asked about the header's
#: compilation -- the macro set, constant probes, the c2rust transpile -- is asked of that TU
#: instead. Its preprocessed text contains the header's bodies by definition.
_VIA: dict[Path, Path] = {}


def register_via(header: Path, tu: Path) -> None:
    _VIA[header.resolve()] = tu.resolve()
    function_like_macros.cache_clear()
    string_literal_macros.cache_clear()


def flags_source(path: Path) -> Path:
    """The file whose compile command answers for `path`: its via TU, else itself."""
    return _VIA.get(path.resolve(), path)


def _function_like(text: str) -> set[str]:
    """Function-like macro names in a `-dM` dump, following object-like ALIASES.

    mingw's `stdarg.h` spells three of the four va_ macros as bare aliases:

        #define va_arg __crt_va_arg          <- object-like, invisible to the regex
        #define va_copy(d,s) __va_copy(...)  <- function-like, caught
        #define __crt_va_arg(v,l) __builtin_va_arg(v,l)

    so `va_copy` was excluded from the seal and `va_arg`/`va_start`/`va_end` were not.
    The seal then tokenized `va_arg(ap, int)` as a runtime callee of arity 2 -- and no
    legal Rust rendering has two arguments, because the TYPE is a turbofish, whose
    commas the arity check blanks. The envelope was therefore UNSATISFIABLE: the model
    returns the same correct body every time and the checker rejects it every time, so
    a retry re-bills a verdict that can never change. msvcrt's `_sopen` and `_open`
    were rejected together in one run on `__RUSTCONV_CALL_0001__=va_arg wants 2, got 1`.
    """
    names = set(_DEFINE_FN_LIKE.findall(text))
    aliases = dict(_DEFINE_ALIAS.findall(text))
    for name in aliases:
        seen = {name}
        target = aliases[name]
        # Bounded walk: an alias may name another alias, and a pathological header
        # could make the chain cyclic. `seen` terminates both.
        while target in aliases and target not in seen:
            seen.add(target)
            target = aliases[target]
        if target in names:
            names.add(name)
    return names


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


#: The kernel-mode C TU whose compile command lends its compiler, target and codegen flags to
#: a synthesized ntoskrnl command. Its `-D`s are all the repo-wide defaults (DBG, _M_*,
#: WINVER, __REACTOS__ ...), so they are kept; only its `-I` list is module-specific.
_KERNEL_BASE_TU = "drivers/bus/pcix/usage.c"


def _derived_kernel_entry(repo_root: Path, arch: str, source: Path) -> list[str] | None:
    """A compile command for an ntoskrnl .c, assembled from the derived kernel profile.

    ntoskrnl.exe is cargo-linked and every one of its .c is flipped, so not one ntoskrnl TU
    is in compile_commands.json and there is no sibling to borrow from either. The include
    path and module defines the real kernel compile used are exactly what profile_derive
    reconstructs for bindgen (ntos.cmake's include_directories/add_definitions, plus linked
    targets' INTERFACE includes such as cmlib). The compiler, target and global flags come from a
    real kernel-mode driver TU. Narrow on purpose: only ntoskrnl/ reaches here.
    """
    try:
        rel = source.resolve().relative_to(repo_root.resolve())
    except ValueError:
        return None
    if not rel.parts or rel.parts[0] != "ntoskrnl":
        return None
    base = _compile_entry(repo_root, arch, repo_root / _KERNEL_BASE_TU)
    if base is None:
        return None
    from . import profile_derive  # local: profile_derive is heavy and only needed here

    derived = profile_derive.derive_source(
        source, repo_root, arch=arch, preferred_target="ntoskrnl", usermode=False
    )
    kept: list[str] = []
    skip_next = False
    for arg in base[1:]:
        if skip_next:
            skip_next = False
            continue
        if arg in ("-I", "-o", "-MF", "-MT", "-MQ", "-include", "-iquote"):
            skip_next = True
            continue
        if arg.startswith(("-I", "-iquote")) or arg in ("-c", "-MD", "-MMD"):
            continue
        if arg.endswith(_KERNEL_BASE_TU):
            continue
        kept.append(arg)
    includes = [f"-I{d}" for d, _kind in derived.env.search]
    defines = [*derived.module.definitions, *derived.ver_defs]
    return [base[0], *kept, *includes, *defines, str(source)]


def _without_outputs(argv: list[str]) -> list[str]:
    """A compile command minus what makes it write files (`-c`, `-o`, dependency outputs)."""
    out: list[str] = []
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
        out.append(arg)
    return out


def compile_argv(repo_root: Path, arch: str, source: Path) -> list[str] | None:
    """`source`'s compile command on `arch`: its own, else the borrowed one the seal uses."""
    source = flags_source(source)
    return (
        _compile_entry(repo_root, arch, source)
        or _sibling_target_entry(repo_root, arch, source)
        or _derived_kernel_entry(repo_root, arch, source)
    )


def included_headers(repo_root: Path, source: Path, arch: str = "amd64") -> set[Path]:
    """Every header `source` really includes on `arch`, by its own preprocessor (`-M`).

    Exact where a lexical include walk is not (conditional and macro-spelled includes), and
    it needs only a compile command -- not a CMakeLists that still lists the TU.
    """
    argv = compile_argv(repo_root, arch, source)
    if argv is None:
        raise MacroAuthorityError(
            f"no compile command for {source} on {arch}\nfix: ./vibeos-cli build --arch {arch}"
        )
    result = subprocess.run(
        [*_without_outputs(argv), "-E", "-M"], capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        raise MacroAuthorityError(
            f"preprocessor failed listing {source.name}'s headers:\n" + result.stderr[:600]
        )
    deps = result.stdout.replace("\\\n", " ").split(":", 1)[-1].split()
    return {
        Path(d.replace(CANONICAL_BUILD_ROOT, str(repo_root))).resolve()
        for d in deps
        if d.endswith((".h", ".hpp", ".inl"))
    }


def _preprocess_defines(argv: list[str]) -> str:
    stripped = _without_outputs(argv)
    result = subprocess.run([*stripped, "-E", "-dM"], capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise MacroAuthorityError(
            "preprocessor failed while deriving the function-like macro set:\n"
            + result.stderr.strip()[:600]
        )
    return result.stdout


_SOURCE_SUFFIXES = (".c", ".cpp", ".cc", ".cxx")
_CXX_SUFFIXES = (".cpp", ".cc", ".cxx")


def _is_cxx(path: Path) -> bool:
    return path.suffix in _CXX_SUFFIXES


def _drop_language_flags(argv: list[str]) -> list[str]:
    """Strip the flags that select a SOURCE LANGUAGE from a borrowed command.

    Only meaningful when borrowing across languages. `-std=` and `-x <lang>` describe the
    sibling's own dialect, not the module's include environment, and clang hard-errors when a
    C dialect meets a C++ input. Everything else (`-I`, `-D`, sysroot, target) is the module's
    and must survive untouched — that closure is the entire reason to borrow.
    """
    out: list[str] = []
    skip_next = False
    for i, arg in enumerate(argv):
        if skip_next:
            skip_next = False
            continue
        if arg.startswith("-std="):
            continue
        if arg == "-x":
            skip_next = True
            continue
        if arg.startswith("-x") and len(arg) > 2 and i:
            continue
        out.append(arg)
    return out


def _sibling_target_entry(repo_root: Path, arch: str, source: Path) -> list[str] | None:
    """A compiled TU from the SAME module directory, with `source` substituted in.

    A cohab flip comments the translated .c out of the source list, so it has no compile
    command of its own — and a flipped module is exactly the one you want to re-translate or
    audit against its C. The module's surviving TUs carry the same `-I`/`-D` environment, so
    borrowing their flags and preprocessing THIS file is exact rather than a guess: the
    include closure is the target's real one and the source is the real source. Only the
    flags are borrowed; nothing is inferred from the sibling's own text.

    A fully-flipped module often has only its GENERATED MIDL `*_s.c` left, which lives under
    `build-<arch>/<same relative dir>` — so both spellings of the module directory count.

    LANGUAGE MUST MATCH FIRST. The "borrowing is exact" argument above holds only within one
    language: a C sibling's command carries `-std=gnu99`, which clang REFUSES on a C++ input
    ("invalid argument '-std=gnu99' not allowed with 'C++'"), so a C++ TU in a module whose
    other TUs are C used to fail the preprocess on every arch and abort the run. Prefer a
    same-language sibling; fall back to a cross-language one only after dropping the
    language-selecting flags, which are the sibling's alone — the `-I`/`-D` closure being
    borrowed is the module's and stays intact.
    """
    db_path = repo_root / f"build-{arch}" / "compile_commands.json"
    if not db_path.is_file():
        return None
    try:
        db = json.loads(db_path.read_text())
    except (OSError, ValueError) as error:
        raise MacroAuthorityError(f"unreadable {db_path}: {error}") from error
    # The module directory first; then the directory of the CMake target that LISTS this file,
    # which is not always its parent. shell32's CMakeLists lists `dialogs/filetypes.cpp`, so
    # every dialogs/*.cpp is compiled with shell32's flags — and once the last of them flipped,
    # dialogs/ had no compiled sibling left, and a bank could no longer be REASSEMBLED after
    # its TU was flipped (the flip is when you most want to fix a translated body). The owning
    # target's surviving TUs (shell32.cpp) carry exactly those flags.
    for rel_dir in _borrow_dirs(repo_root, source):
        argv = _sibling_in(repo_root, arch, source, db, rel_dir)
        if argv is not None:
            return argv
    return None


def _borrow_dirs(repo_root: Path, source: Path) -> list[Path]:
    """The TU's own directory, then the nearest ancestor whose CMakeLists.txt names it."""
    dirs = [source.parent.relative_to(repo_root)]
    for ancestor in source.parent.parents:
        if ancestor == repo_root or repo_root not in ancestor.parents:
            break
        cml = ancestor / "CMakeLists.txt"
        if not cml.is_file():
            continue
        # dll/keyboard names all 89 layouts through `foreach(_keyboard_layout ...)`, so
        # `kbdus/kbdus.c` appears only once the loop is unrolled (the flipped line is still
        # there, commented); once every layout is flipped, the stub TU beside that
        # CMakeLists is the one left carrying their flags.
        from .profile_derive import _unroll_target_foreach

        if str(source.relative_to(ancestor)) in _unroll_target_foreach(cml.read_text(errors="replace")):
            dirs.append(ancestor.relative_to(repo_root))
            break
    return dirs


def _sibling_in(
    repo_root: Path, arch: str, source: Path, db: list[dict], rel_dir: Path
) -> list[str] | None:
    wanted_dirs = {repo_root / rel_dir, repo_root / f"build-{arch}" / rel_dir}
    want_cxx = _is_cxx(source)
    # Same-language siblings first, then the rest. Within each group, sorted so the choice is
    # deterministic across runs; the resume cache keys on the derived macro set, and a set that
    # flip-flops with dict order would thrash it.
    candidates = sorted(db, key=lambda e: e.get("file", ""))
    candidates.sort(key=lambda e: _is_cxx(Path(e.get("file", ""))) != want_cxx)
    for entry in candidates:
        recorded = entry.get("file", "").replace(CANONICAL_BUILD_ROOT, str(repo_root))
        tu = Path(recorded)
        if tu.parent not in wanted_dirs or tu.suffix not in _SOURCE_SUFFIXES:
            continue
        argv = shlex.split(entry.get("command") or " ".join(entry.get("arguments", [])))
        argv = [a.replace(CANONICAL_BUILD_ROOT, str(repo_root)) for a in argv]
        if _is_cxx(tu) != want_cxx:
            argv = _drop_language_flags(argv)
        # Point the borrowed command at OUR file. Match on the sibling's full recorded path
        # as well as its basename: `-o …/wkssvc_s.c.obj` also ends in the basename, and
        # rewriting the output path instead of the input silently preprocesses the sibling.
        swapped = False
        for i, arg in enumerate(argv):
            if i and argv[i - 1] in ("-o", "-MF", "-MT", "-MQ"):
                continue
            if Path(arg.replace(CANONICAL_BUILD_ROOT, str(repo_root))) == tu:
                argv[i] = str(source)
                swapped = True
        if not swapped:
            continue
        return argv
    return None


_STRING_MACRO = re.compile(
    r'^#define[ \t]+([A-Za-z_]\w*)[ \t]+((?:L?"(?:[^"\\\n]|\\.)*"[ \t]*)+)$', re.M
)


@lru_cache(maxsize=64)
def string_literal_macros(repo_root: Path, source: Path) -> dict[str, dict[str, str]]:
    """`{NAME: {arch: literal text}}` for every object-like macro whose whole value is
    string literal(s), per built arch, from the TU's OWN compile command.

    The wide seal folds `L"AddReg.NT" INF_ARCH` with these: C concatenates the literal
    with the macro's expansion, and spapisup.h defines INF_ARCH differently per arch, so
    only the real preprocessor of each build can say what the string is. Empty when the
    TU has no command of its own; the seal then skips the run as it always did (reported).
    """
    out: dict[str, dict[str, str]] = {}
    via = flags_source(source)
    for arch in _ARCHES:
        argv = _compile_entry(repo_root, arch, via)
        if argv is None and via != source:
            argv = _sibling_target_entry(repo_root, arch, via)
        if argv is None:
            continue
        try:
            dump = _preprocess_defines(argv)
        except MacroAuthorityError:
            continue
        for m in _STRING_MACRO.finditer(dump):
            out.setdefault(m.group(1), {})[arch] = m.group(2).strip()
    return out


@lru_cache(maxsize=64)
def identifier_aliases(repo_root: Path, source: Path) -> dict[str, str]:
    """`{NAME: TARGET}` for each object-like macro whose whole value is one identifier,
    the same on every built arch, from the TU's OWN compile command.

    The SDK's A/W switch: `#define GetModuleHandle GetModuleHandleW` under UNICODE. A
    translation calls `GetModuleHandle` as the C spells it, and only the preprocessor of the
    real build knows which function that is (diskpart: 22 calls left unresolved).
    """
    # compile_argv, not the TU's own entry: after a cohab flip a TU HAS no entry, and this
    # returned {} without a word -- every A/W-spelled definition in usetup's console.c
    # (WriteConsole, ReadConsoleInput, PeekConsoleInput) was then dropped as a dead branch
    # by the next `port replay`, the very failure the aliases exist to prevent.
    per_arch: list[dict[str, str]] = []
    commands = 0
    for arch in _ARCHES:
        argv = compile_argv(repo_root, arch, source)
        if argv is None:
            continue
        commands += 1
        try:
            dump = _preprocess_defines(argv)
        except MacroAuthorityError:
            continue
        per_arch.append(dict(re.findall(r"^#define ([A-Za-z_]\w*) ([A-Za-z_]\w*)\s*$", dump, re.M)))
    if not commands:
        raise MacroAuthorityError(
            f"no compile command for {source} (own or borrowed) in any build-<arch>: "
            "build an arch first"
        )
    if not per_arch:
        return {}
    agreed = per_arch[0]
    for other in per_arch[1:]:
        agreed = {k: v for k, v in agreed.items() if other.get(k) == v}
    return agreed


def arch_only_aliases(repo_root: Path, source: Path) -> dict[str, dict[str, str]]:
    """`{NAME: {arch: TARGET}}` for an identifier alias that holds on SOME built arches only.

    winuser.h makes `GetWindowLongPtrW` a macro for `GetWindowLongW` on i386 and a real
    export on the 64-bit targets. identifier_aliases keeps only what every arch agrees on, so
    the extern resolve declares links on amd64/arm64 and is `undefined symbol
    _GetWindowLongPtrW@8` on i386 -- appwiz, then timedate, each patched by hand.
    """
    per_arch: dict[str, dict[str, str]] = {}
    for arch in _ARCHES:
        argv = compile_argv(repo_root, arch, source)
        if argv is None:
            return {}
        try:
            dump = _preprocess_defines(argv)
        except MacroAuthorityError:
            return {}
        per_arch[arch] = dict(re.findall(r"^#define ([A-Za-z_]\w*) ([A-Za-z_]\w*)\s*$", dump, re.M))
    names = set().union(*per_arch.values())
    out: dict[str, dict[str, str]] = {}
    for name in names:
        targets = {a: d[name] for a, d in per_arch.items() if name in d}
        if len(targets) != len(_ARCHES):
            out[name] = targets
    return out


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
    borrowed_failures: list[str] = []
    flag_src = flags_source(source)
    is_header = flag_src.suffix in (".h", ".hpp")
    for arch in _ARCHES:
        borrowed = False
        argv = _compile_entry(repo_root, arch, flag_src)
        if argv is None and is_header:
            # See _includer_entry: a header's macro set is its includers' macro set.
            argv = _includer_entry(repo_root, arch, flag_src)
            borrowed = argv is not None
        if argv is None and not is_header:
            # See _sibling_target_entry: a FLIPPED .c has no command of its own.
            argv = _sibling_target_entry(repo_root, arch, flag_src)
            borrowed = argv is not None
        if argv is None and not is_header:
            # See _derived_kernel_entry: ntoskrnl compiles no C through CMake at all.
            argv = _derived_kernel_entry(repo_root, arch, flag_src)
            borrowed = argv is not None
        if argv is None:
            continue
        if borrowed:
            # A BORROWED command can fail on an arch whose tree is built less far than the
            # one lent it — arm64 may not have run the MIDL step that generates the `*_s.h`
            # this TU includes. That is the same situation as "this arch has no entry", which
            # the loop already skips, so skip it rather than failing the whole derivation.
            # It stays loud: every skipped arch is reported below if NO arch succeeds, and a
            # file's OWN command still raises on the spot, because that is a real breakage.
            try:
                dump = _preprocess_defines(argv)
            except MacroAuthorityError as error:
                borrowed_failures.append(f"{arch}: {str(error).splitlines()[-1][:200]}")
                continue
        else:
            dump = _preprocess_defines(argv)
        names |= _function_like(dump)
        found.append(arch)
    # `-dM` dumps only macros STILL DEFINED when preprocessing ends, so one that the TU
    # `#undef`s again is invisible to it — and a function-like macro missing from this set
    # gets its invocations sealed as runtime calls, which no translation can satisfy.
    # shell32's `DumpEvent` is exactly that shape: `#define DUMPEV(x)`, 22 invocations,
    # `#undef DUMPEV`. It cost four rejected paid attempts. Scanning the TU's own text is
    # cheap and catches the define/undef pair the preprocessor dump erases.
    try:
        names |= _function_like(source.read_text(errors="replace"))
    except OSError:
        pass
    if not found:
        via = " and no compiled TU in its own directory #includes it" if is_header else ""
        tried = (
            "\nborrowed a sibling TU's flags but the preprocessor failed on every arch:\n  "
            + "\n  ".join(borrowed_failures)
            if borrowed_failures
            else ""
        )
        raise MacroAuthorityError(
            f"no compile command for {source} in any build-<arch>/compile_commands.json "
            f"under {repo_root}{via}. The call seal needs the real function-like macro set; "
            f"without it every macro occurrence becomes an unsatisfiable envelope and "
            f"the TU assembles nothing.{tried}\n"
            f"fix: ./vibeos-cli build --arch i386"
        )
    return frozenset(names)
