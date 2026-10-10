#!/usr/bin/env python3
"""Derive a ``gen_driver_types.sh`` bindgen-island profile from a module's REAL
build inputs — its CMakeLists source list plus each compiled TU's own transitive
include closure — instead of hand-reading ``precomp.h``.

WHY THIS EXISTS
===============
Every C module we translate needs a per-module bindgen "island" of struct layouts
and constants, driven by a hand-authored profile in ``tools/rustconv/gen_driver_types.sh``.
Hand-authoring cost real debugging time twice, in two directions:

1. **An INCOMPLETE island is a CORRECTNESS bug, not a compile error**
   (field manual §9.112). A missing *type* is a loud ``E0412``. A missing
   *constant* is loud only in VALUE position — in ``match`` PATTERN position a bare
   unresolved identifier silently becomes a fresh irrefutable BINDING that matches
   EVERYTHING, collapsing a C ``switch`` into its first arm. That shipped a real
   BUGCHECK 0x1E, then recurred in ``browser``/``wlansvc`` (whose islands lacked
   ``winsvc.h``, so every SERVICE_CONTROL_* fell into the STOP arm). So: the header
   that owns a constant any translated ``switch`` dispatches on MUST be allowlisted.
   ``tools/verify_match_consts.py`` is the gated oracle for the *symptom*; this
   module attacks the *cause*, which is cheaper and durable.

2. **A multi-header redefinition wall is a claim about YOUR WRAPPER, not about the
   headers** (field manual §9.108). The recorded "6 mutually-conflicting MIDL
   interfaces" blocker in ``netapi32`` was a misdiagnosis that deferred real work
   twice: the actual conflict was the ``<lm.h>`` MASTER header versus the WIDL
   stubs. The fix is to mirror each translation unit's REAL include set — *if a
   ``.c`` compiles, its include set is bindgen-able* — and to split into separate
   ``TYPES_STEM`` islands only for the rare pair that genuinely collides.

Both failure modes are mechanizable, because the ground truth is already in the
tree: the CMakeLists says which TUs are actually compiled (files can exist on disk
and be dead source — last batch a translation paid for a file nobody compiles), and
each TU's ``#include`` prologue says what its island may legally contain.

Deriving the allowlist FROM the closure also buys a property the hand-written profiles
have to remember by hand: ``--allowlist-file`` can only FILTER types bindgen actually
parsed, never emit unreached ones (see the classpnp/disk/cdrom profile comments, which
pull extra headers explicitly for exactly this reason). Every header this module
allowlists is reachable from the emitted wrapper by construction.

WHAT IT DOES NOT DO
===================
It is not a C preprocessor. The include walker is lexical and deliberately
OVER-approximates (it follows both arms of a conditional), because for an allowlist
the safe direction is "too many headers", never "too few". It never silently drops
an include it cannot resolve: unresolved includes raise :class:`ProfileDeriveError`
(``--allow-unresolved`` demotes them to loud findings) rather than producing a
partial profile that looks complete.

CLI
===
    python -m rustconv.profile_derive dll/win32/wlanapi            # report + profile
    python -m rustconv.profile_derive dll/win32/wlanapi --emit profile
    python -m rustconv.profile_derive drivers/network/dd/rtl8139 --emit json
    python -m rustconv.profile_derive dll/win32/snmpapi --scaffold-dir /tmp/snmpapi-rust
"""

from __future__ import annotations

import argparse
import functools
import json
import re
import sys
from dataclasses import dataclass, field
from datetime import date
from pathlib import Path

from .dead_regions import (  # noqa: F401  (KNOWN_DEFINED re-exported)
    KNOWN_DEFINED,
    cond_truth,
    eval_int_guard,
    pinned_int_macros,
)

# ---------------------------------------------------------------------------
# Errors + findings
# ---------------------------------------------------------------------------


class ProfileDeriveError(RuntimeError):
    """A derivation could not be completed on inputs it cannot guess at.

    Raised instead of emitting a partial profile: a profile that looks complete but
    silently omits a header is exactly the §9.112 failure mode this module exists to
    prevent (AGENTS.md "fail loud, never silently degrade").
    """


@dataclass(frozen=True)
class Finding:
    """One reviewable observation. ``severity`` ∈ {error, warn, info}."""

    severity: str
    code: str
    detail: str
    #: The repo-relative .c this finding is ABOUT, when it is about one file
    #: rather than the module as a whole. `None` means module-wide.
    #:
    #: This exists so a caller can tell "you must not pay to translate THIS
    #: file" apart from "you must not pay to translate SOME file in this
    #: module". `derive_source` is module-scoped, so without it a per-file
    #: ERROR blocks every OTHER file in the module too — which is how msvcrt's
    #: two permanently-dead TUs (handler4.c, concurrency.c) came to block
    #: translating anything in msvcrt at all, with no waiver available.
    subject: str | None = None

    def __str__(self) -> str:
        mark = {"error": "✗", "warn": "!", "info": "·"}.get(self.severity, "?")
        return f"  {mark} {self.severity.upper():5s} {self.code}: {self.detail}"


# ---------------------------------------------------------------------------
# Header classification policy
# ---------------------------------------------------------------------------

# Headers whose contents are ALREADY a single identity in the sibling -sys crate
# (ntdll-sys for usermode, ntkrnl-sys for kernel), so allowlisting them would
# re-emit a second local copy of the base platform. Mirrors what every hand-written
# profile in gen_driver_types.sh does: blocklist the base, allowlist the domain.
BASE_PLATFORM_HEADERS = frozenset(
    {
        # Win32 base
        "windows.h",
        "windef.h",
        "winbase.h",
        "winnt.h",
        "wincon.h",
        "winuser.h",
        "wtypes.h",
        "wtypesbase.h",
        "basetsd.h",
        "guiddef.h",
        "specstrings.h",
        "sal.h",
        "excpt.h",
        "poppack.h",
        "pshpack1.h",
        "pshpack2.h",
        "pshpack4.h",
        "pshpack8.h",
        "winapifamily.h",
        "apiset.h",
        "concurrencysal.h",
        # NT / kernel base
        "ntdef.h",
        "ntstatus.h",
        "ntddk.h",
        "wdm.h",
        "ntifs.h",
        "ntoskrnl.h",
        "ntndk.h",
        "ndk.h",
        # debug channels — deliberately PARSED but never allowlisted; the no-op
        # TRACE/ERR/WARN macros live in the crate prelude (field manual §9.65).
        "debug.h",
    }
)

# Pure CRT/toolchain headers. Resolved for the closure when present in-tree, but
# never allowlisted and never an error when they resolve only inside the toolchain.
CRT_HEADERS = frozenset(
    {
        "assert.h",
        "ctype.h",
        "errno.h",
        "float.h",
        "limits.h",
        "locale.h",
        "math.h",
        "setjmp.h",
        "signal.h",
        "stdarg.h",
        "stdbool.h",
        "stddef.h",
        "stdint.h",
        "stdio.h",
        "stdlib.h",
        "string.h",
        "time.h",
        "wchar.h",
        "wctype.h",
        "intrin.h",
        "immintrin.h",
        "emmintrin.h",
        "xmmintrin.h",
        "malloc.h",
        "memory.h",
        "process.h",
        "io.h",
        "conio.h",
        "direct.h",
        "share.h",
        "sys/stat.h",
        "sys/types.h",
        "sys/timeb.h",
        "vadefs.h",
        "crtdefs.h",
        "corecrt.h",
    }
)

# CMake module types that build usermode images (USERMODE=1 in gen_driver_types.sh);
# everything else in VALID_MODULE_TYPES is kernel-side. From sdk/cmake/CMakeMacros.cmake.
KERNEL_MODULE_TYPES = frozenset(
    {"kernel", "kerneldll", "kernelmodedriver", "kmdfdriver", "wdmdriver"}
)
USERMODE_MODULE_TYPES = frozenset(
    {"nativecui", "nativedll", "win32cui", "win32gui", "win32dll", "win32ocx", "cpl", "module"}
)

ARCHES = ("i386", "amd64", "arm64")

_INCLUDE_RE = re.compile(r'^\s*#\s*include\s+(?:(<[^>]+>)|("[^"]+"))')
_INCLUDE_MACRO_RE = re.compile(r"^\s*#\s*include\s+([A-Za-z_]\w*)")
_DIRECTIVE_RE = re.compile(r"^\s*#\s*(\w+)")
_COND_DIRECTIVES = frozenset({"if", "ifdef", "ifndef", "else", "elif", "endif"})


# ---------------------------------------------------------------------------
# Data model
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Include:
    """One ``#include`` occurrence, as written."""

    spelling: str  # "<ndis.h>" / "\"nic.h\"" — reproduced verbatim in the wrapper
    name: str  # "ndis.h"
    angled: bool
    line: int
    # Inside an ``#if``/``#ifdef`` (the file's own include guard does NOT count). A
    # conditional include that fails to resolve is almost always a dead platform arm
    # (``rpcmac.h``, ``intrin_mips.h``); an UNCONDITIONAL one that fails to resolve means
    # our search path is wrong and the closure is genuinely incomplete.
    conditional: bool = False


@dataclass
class HeaderRef:
    """A header reached by the include walk, with the evidence for why it is in."""

    path: Path
    name: str
    kind: str  # module | generated | idl | sdk | crt | toolchain
    depth: int  # 1 = directly included by a TU
    includers: list[str] = field(default_factory=list)
    tus: set[str] = field(default_factory=set)


@dataclass(frozen=True)
class Unresolved:
    """An include the walker could not resolve — never silently dropped."""

    name: str
    includer: str
    line: int
    reason: str
    conditional: bool = False


@dataclass
class SourceEntry:
    """One entry of a CMake target's source list."""

    raw: str
    path: Path | None  # None for generated (binary-dir) entries
    kind: str  # tu | generated | header | resource | def | unknown
    condition: str | None = None  # unresolved `if(...)` guard, if any


@dataclass
class CMakeModule:
    """What the module's CMakeLists actually says is compiled."""

    cmake: Path
    module_dir: Path
    name: str
    target_kind: str  # library | executable
    module_type: str | None
    unicode: bool
    entries: list[SourceEntry] = field(default_factory=list)
    definitions: list[str] = field(default_factory=list)
    removed_definitions: list[str] = field(default_factory=list)
    include_dirs: list[Path] = field(default_factory=list)
    pch_header: Path | None = None
    # TUs add_pch() explicitly EXCLUDES — they do not compile with the PCH, so a wrapper
    # that force-includes it reproduces exactly the conflict the skip list avoids.
    pch_skip: set[Path] = field(default_factory=set)
    rpc_interfaces: list[str] = field(default_factory=list)
    dead_sources: list[Path] = field(default_factory=list)
    findings: list[Finding] = field(default_factory=list)
    # Set by --usermode/--kernel. A target with no set_module_type() cannot be
    # classified from the CMakeLists at all — a static library shared between DLLs
    # (msvcrt_static, feeding msvcrt.dll/msvcrt20.dll/msvcrt40.dll) never calls it,
    # and it is exactly such a target that owns the C in a cohab module.
    usermode_override: bool | None = None
    # What the Rust side is NAMED after, when that is not the CMake target: a static library
    # with exactly one consumer (setuplib_static -> setuplib) belongs to that module, so its
    # crate is setuplib-rust and its gen_driver_types.sh arm is `setuplib)`. The compile-db
    # lookup still keys on `name`, the real target.
    stem: str | None = None

    @property
    def crate_stem(self) -> str:
        return self.stem or self.name

    @property
    def compiled_tus(self) -> list[Path]:
        """In-tree TUs actually compiled — the translation surface."""
        return [e.path for e in self.entries if e.kind == "tu" and e.path is not None]

    @property
    def generated_sources(self) -> list[str]:
        """Binary-dir sources (MIDL/spec2def products). They stay C; never translate them."""
        return [
            e.raw.replace("${CMAKE_CURRENT_BINARY_DIR}/", "")
            for e in self.entries
            if e.kind == "generated"
        ]

    @property
    def is_usermode(self) -> bool:
        if self.usermode_override is not None:
            return self.usermode_override
        if self.module_type is None:
            raise ProfileDeriveError(
                f"{self.cmake}: no set_module_type() found for target {self.name!r}; cannot tell "
                "kernel from usermode (USERMODE= in the profile). Pass --usermode or --kernel."
            )
        if self.module_type in KERNEL_MODULE_TYPES:
            return False
        if self.module_type in USERMODE_MODULE_TYPES:
            return True
        raise ProfileDeriveError(
            f"{self.cmake}: unknown module type {self.module_type!r} for target {self.name!r}"
        )


@dataclass
class IncludeEnv:
    """Include search path, in the order the real compile uses it."""

    repo_root: Path
    arch: str
    module_dir: Path
    build_dir: Path | None
    search: list[tuple[Path, str]] = field(default_factory=list)  # (dir, kind)

    def classify(self, path: Path) -> str:
        """Kind of a resolved header, by which search root owns it.

        ``generated`` is deliberately scoped to the MODULE's own binary dir (its MIDL
        ``*_c.h``/``*_s.h``). ``build-<arch>/sdk/include`` is NOT module-generated —
        large parts of psdk (``winnt.h``, ``oaidl.h``, ``urlmon.h``, every WIDL product)
        are configure-generated there, and treating those as module-adjacent bloats the
        allowlist with the whole COM/OLE platform.
        """
        if _is_within(path, self.module_dir):
            return "module"
        if self.module_build_dir is not None and _is_within(path, self.module_build_dir):
            return "generated"
        if not _is_within(path, self.repo_root):
            # The compiler's own headers (llvm-mingw's libc++ via the compile db's
            # -isystem): never a ReactOS type surface.
            return "toolchain"
        parts = path.parts
        if "idl" in parts:
            return "idl"
        if Path(path.name).name in CRT_HEADERS or "crt" in parts or "vcruntime" in parts:
            return "crt"
        return "sdk"

    @property
    def module_build_dir(self) -> Path | None:
        if self.build_dir is None:
            return None
        return self.build_dir / self.module_dir.relative_to(self.repo_root)

    def resolvable_by_name(self, path: Path, spelled: str) -> bool:
        """True if ``#include <spelled>`` finds *path* from a standard search dir.

        Guards against emitting a pointless ``-I`` for a SUBDIR-qualified spelling
        (``sec_api/stdio_s.h``, ``psdk/winnt.h``) whose parent directory is not itself
        on the search path — and never needs to be.
        """
        return any((d / spelled).resolve() == path for d, _ in self.search)


@dataclass
class Closure:
    """The transitive include closure of one or more TUs."""

    headers: dict[Path, HeaderRef] = field(default_factory=dict)
    unresolved: list[Unresolved] = field(default_factory=list)
    per_tu: dict[str, set[Path]] = field(default_factory=dict)


@dataclass(frozen=True)
class SwitchConst:
    """An identifier used as a ``case`` label — the §9.112 hazard surface."""

    ident: str
    tu: str
    line: int


@dataclass
class ConstOwner:
    """Where a switch constant is defined, and whether the island will carry it."""

    ident: str
    owner: Path | None
    owner_kind: str  # header | cxx-const | tu-local | sys-crate | UNOWNED
    definition: str | None
    macro_kind: str  # literal | expression | function-like | enum | n/a
    allowlisted: bool = False
    # Already exported by the sibling -sys crate (ntdll-sys / ntkrnl-sys): it resolves
    # through that crate's glob, so its owning header needs NO allowlist entry. Checking
    # this is what keeps `winerror.h`-sized headers out of a profile that does not need
    # them, while still force-adding the ones the -sys crate genuinely lacks.
    in_sys: bool = False
    # which -sys island provides it: the primary `bindgen_<arch>`/`bindgen_common` (reachable
    # through the glob the generated island re-exports) or a SECONDARY one such as
    # `hal_bindgen_<arch>` (reachable only via its own module path, e.g. ntkrnl_sys::hal_ddk)
    sys_source: str | None = None
    # For an ENUM member: the tag/typedef bindgen prefixes it with, so the report can name the
    # spelling that actually exists and the `pub use` line that maps it back.
    enum_prefix: str | None = None


@dataclass
class Island:
    """One bindgen island: a group of TUs whose include sets are mutually legal.

    Each island carries its OWN wrapper: the union of every TU's prologue would re-create
    exactly the redefinition wall the split exists to avoid (§9.108).
    """

    stem: str
    tus: list[str]
    allow_files: list[str]
    reason: str
    wrapper_inc: list[str] = field(default_factory=list)
    label: str = ""  # the DRIVER argument gen_driver_types.sh is invoked with


@dataclass
class DerivedProfile:
    """Everything the integrator needs, plus the evidence for every decision."""

    module: CMakeModule
    env: IncludeEnv
    closure: Closure
    switch_consts: list[SwitchConst]
    const_owners: list[ConstOwner]
    islands: list[Island]
    wrapper_inc: list[str]
    allow_files: list[str]
    extra_inc: list[str]
    ver_defs: list[str]
    # Include dirs the REAL compile lists AFTER the standard SDK dirs. They must stay after
    # them in the island build too: `sdk/include/wine` carries an alternate surface for
    # headers the PSDK also owns (winioctl.h defines _FILE_FS_*_INFORMATION, which
    # ndk/iotypes.h then redefines), so hoisting it early silently swaps which definition the
    # island is built against. samlib: 23 redefinition errors when early, 0 when late.
    late_inc: list[str] = field(default_factory=list)
    allow_reasons: dict[str, str] = field(default_factory=dict)
    excluded: dict[str, str] = field(default_factory=dict)
    findings: list[Finding] = field(default_factory=list)
    compile_db_used: Path | None = None
    #: The BLOCKLIST_FUNCS evidence. None only for a kernel module, where the flag is not
    #: emitted at all.
    prototype_survey: PrototypeSurvey | None = None

    @property
    def crate_name(self) -> str:
        return f"{self.module.crate_stem}-rust"

    @property
    def errors(self) -> list[Finding]:
        return [f for f in self.findings if f.severity == "error"]

    def blocking_errors(self, c_file: str | None = None) -> list[Finding]:
        """The errors that must stop a paid run for `c_file` (repo-relative).

        Module-wide errors (``subject is None``) always block. A finding ABOUT a
        specific file blocks only that file: `DEAD_TU: concurrency.c is 98% dead`
        is advice not to buy concurrency.c, and has nothing to say about
        console.c. Blocking on it anyway made msvcrt untranslatable end to end,
        because its two dead TUs are permanent and there is no waiver flag.

        `c_file=None` keeps the old whole-module meaning, which is what a bare
        `profile_derive` report wants.
        """
        if c_file is None:
            return self.errors
        return [f for f in self.errors if f.subject is None or f.subject == c_file]

    def deferred_errors(self, c_file: str) -> list[Finding]:
        """Errors `blocking_errors(c_file)` declines to block on.

        Returned so the caller can PRINT them. They are still true, and a
        finding that stops blocking must not also stop being visible.
        """
        return [f for f in self.errors if f.subject is not None and f.subject != c_file]


# ---------------------------------------------------------------------------
# Small helpers
# ---------------------------------------------------------------------------


def _is_within(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _read_with_includes(path: Path, _seen: frozenset[Path] = frozenset()) -> str:
    """`_read`, but splicing `include(<file>.cmake)` in at the include site.

    A tree may define its targets in included `.cmake` fragments rather than inline --
    sdk/lib/crt does, and so do ntoskrnl, freeldr and winsrv; 36 CMakeLists.txt in this repo
    use the pattern. Without this, `--target crtmath` fails with "no target 'crtmath'" and
    the whole module is invisible to the profiler, which is a silent gap: the answer looks
    like "that target does not exist" rather than "this tool cannot see it".

    Textual splicing rather than a second parse pass, because CMake's `include()` runs the
    file in the CURRENT variable scope -- `crtmath.cmake` reads `${CRT_MATH_SOURCE}` that
    `math/math.cmake` appended to earlier. Pasting at the include site is exactly those
    semantics, and it keeps the single-pass parser below unchanged.

    Only literal relative paths are followed. A `${VAR}`-built include is left alone: the
    variables are not resolved yet at read time, and guessing would splice the wrong file.
    """
    text = _read(path)
    if path in _seen:  # include cycles are legal to write and fatal to recurse on
        return ""
    seen = _seen | {path}
    out, pos = [], 0
    for m in re.finditer(r"^[ \t]*include\s*\(\s*([^)${}\s\"]+\.cmake)\s*\)", text, re.M):
        inc = (path.parent / m.group(1)).resolve()
        out.append(text[pos : m.start()])
        if inc.is_file():
            out.append(f"\n# ---- spliced from {m.group(1)} ----\n")
            out.append(_read_with_includes(inc, seen))
            out.append(f"\n# ---- end {m.group(1)} ----\n")
        else:
            out.append(m.group(0))  # unresolvable: leave it, the caller may report it
        pos = m.end()
    out.append(text[pos:])
    return "".join(out)


def _rel(path: Path, root: Path) -> str:
    return str(path.relative_to(root)) if _is_within(path, root) else str(path)


def _hdr_regex(name: str) -> str:
    """``ntddndis.h`` -> ``.*[/\\]ntddndis\\.h`` — the ALLOW_FILES spelling in use."""
    return r".*[/\\]" + re.escape(name).replace("/", r"[/\\]")


# ---------------------------------------------------------------------------
# CMakeLists parsing — "which TUs are ACTUALLY compiled"
# ---------------------------------------------------------------------------


def _strip_cmake_comments(text: str) -> str:
    out: list[str] = []
    for line in text.splitlines():
        in_str = False
        cut = len(line)
        for i, ch in enumerate(line):
            if ch == '"':
                in_str = not in_str
            elif ch == "#" and not in_str:
                cut = i
                break
        out.append(line[:cut])
    return "\n".join(out)


#: A TU that exists only to hold what stable Rust cannot DEFINE — a `va_start`, a
#: `__forceinline` body, a layout assertion, a cohab stub. The tree spells every one of them
#: `rust_*.c` (12 distinct names as of 2026-08-07: rust_varargs_shim, rust_scanf_shim,
#: rust_strsafe_shim, rust_cohab_fat, rust_layout_assert, …). None is a translation surface, so
#: a module whose whole compiled source list is these is FLIPPED, not un-translated.
_SHIM_TU_RE = re.compile(r"^rust_\w*\.c$")


def is_shim_tu(path: Path) -> bool:
    """True for a cohab scaffolding TU — never something to derive a translation island from."""
    return bool(_SHIM_TU_RE.match(path.name))


#: Commands whose argument list is a source list. A flip comments entries out of one of
#: these — `add_library` included: wsock32's only `.c` is commented inside `add_library(...)`,
#: not inside a `list(APPEND SOURCE)` at all.
_SOURCE_LIST_OPEN = re.compile(
    r"^[ \t]*(?:list\s*\(\s*APPEND\s+\w+|set\s*\(\s*\w+"
    r"|add_(?:library|executable)\s*\(\s*\w+)"
)
#: A comment line inside such a list whose first token is a C source file. Four annotation
#: spellings are live in this tree and there is no point discriminating among them — being
#: INSIDE the list is what makes a line an entry:
#:     `#     stubs.c`                                                  (wsock32)
#:     `#     tcpsvcs.c    (→ tcpsvcs-rust src/tcpsvcs/tcpsvcs.rs — …)` (tcpsvcs)
#:     `#     gdi/eng/clip.c   # replaced by Rust (…) — cohabitation`   (win32ss, 158)
#:     `#     byteswap.c — replaced by Rust implementation (…)`         (rtl)
#: The precision comes from two other places instead: the depth tracking that decides
#: "inside", and the on-disk check below, which is what stops a HEADING that merely names a
#: file (shell32 writes one above each flipped block) from inventing an entry.
#: ntoskrnl's ntos.cmake spells every entry repo-rooted, `#${REACTOS_SOURCE_DIR}/ntoskrnl/ps/job.c`
#: (237 of them), so the prefix is accepted and resolved against the repo root.
_COMMENTED_ENTRY = re.compile(
    r"^[ \t]*#[ \t]*((?:\$\{REACTOS_SOURCE_DIR\}/)?[\w./+-]+\.(?:c|cpp|cc))(?:[ \t].*)?$"
)

#: `# list(APPEND SOURCE` / `# set(SOURCE` — what a cohab flip leaves behind.
_COMMENTED_LIST_OPEN = re.compile(r"^[ \t]*#[ \t]?(list\s*\(\s*APPEND\s+\w+|set\s*\(\s*\w+)")
_SOURCE_ARG = re.compile(r"^(?:\$\{[^}]+\}|[\w./+-]+\.(?:c|cpp|cc|h|hpp|idl|rc))$")


def uncomment_flipped_source_lists(
    text: str, module_dir: Path | None = None, repo_root: Path | None = None
) -> tuple[str, list[str]]:
    """Restore the source list a cohab flip commented out. Returns `(text, restored files)`.

    A flip does not DELETE the source list, it comments it out — so the pre-flip translation
    surface is still sitting in the CMakeLists, and a flipped module can be re-audited without
    going to git. That matters because the audits worth re-running (case labels, dead bodies,
    enum members) are exactly the ones that answer "was this flip correct".

    Deliberately narrow: the block must be a `list(APPEND X ...)`/`set(X ...)` whose every
    argument looks like a source file or a `${VAR}`, and its parens must balance before the
    commented run ends. Un-commenting an arbitrary `# set(FOO ...)` would resurrect a
    build decision somebody disabled on purpose.
    """
    lines = text.splitlines()
    out: list[str] = []
    restored: list[str] = []
    index = 0
    while index < len(lines):
        if _COMMENTED_LIST_OPEN.match(lines[index]) is None:
            out.append(lines[index])
            index += 1
            continue
        block: list[str] = []
        cursor = index
        depth = 0
        while cursor < len(lines) and lines[cursor].lstrip().startswith("#"):
            bare = lines[cursor].lstrip()[1:].lstrip()
            block.append(bare)
            depth += bare.count("(") - bare.count(")")
            cursor += 1
            if depth == 0:
                break
        joined = " ".join(block)
        args = _split_args(joined[joined.index("(") + 1 : joined.rindex(")")]) if depth == 0 else []
        # The first argument of `list(APPEND X ...)` is the variable, not a source.
        values = args[2:] if joined.lstrip().lower().startswith("list") else args[1:]
        if depth == 0 and values and all(_SOURCE_ARG.match(a) for a in values):
            out.extend(block)
            restored.extend(values)
            index = cursor
            continue
        out.append(lines[index])
        index += 1

    # SECOND shape, and the commoner one: the source-list command stays LIVE and the entries
    # inside it are commented out one per line. tcpsvcs (8 entries in `list(APPEND SOURCE)`)
    # and wsock32 (one entry inside `add_library`) are both this, and both are why
    # --flipped-sources first reported them as having no sources at all.
    final: list[str] = []
    depth = 0
    for line in out:
        if depth == 0 and _SOURCE_LIST_OPEN.match(line) is None:
            final.append(line)
            continue
        # The OPENING line's own parens are counted like any other, which is the whole trick:
        # `set(X "...")` opens and closes on one line, and skipping it left depth stuck at 1
        # for the rest of the file — wsock32 then restored a PROSE comment
        # (`#   stubs.c  → wsock32-rust`) sitting outside every list.
        entry = _COMMENTED_ENTRY.match(line) if depth > 0 else None
        # An entry names a file that is STILL THERE — a flip comments sources out, it never
        # deletes them (the never-delete rule). That is what separates an entry from the
        # heading shell32 writes above each flipped block, which names the same file in prose.
        # Deduped for the same reason, and because rtl legitimately lists `byteswap.c` in two
        # different `if(ARCH)` arms.
        if entry is not None and module_dir is not None:
            rel = entry.group(1)
            if rel.startswith("${REACTOS_SOURCE_DIR}/"):
                on_disk = (
                    repo_root is not None
                    and (repo_root / rel[len("${REACTOS_SOURCE_DIR}/") :]).is_file()
                )
            else:
                on_disk = (module_dir / rel).is_file()
            if not on_disk:
                entry = None
        if entry is not None:
            final.append("    " + entry.group(1))
            if entry.group(1) not in restored:
                restored.append(entry.group(1))
            continue
        code = line.split("#", 1)[0]
        depth = max(depth + code.count("(") - code.count(")"), 0)
        final.append(line)
    return "\n".join(final), restored


def _cmake_commands(text: str) -> list[tuple[str, str, int]]:
    """Yield ``(command, raw-args, line)`` honouring nested parens and strings."""
    cmds: list[tuple[str, str, int]] = []
    i = 0
    n = len(text)
    name_re = re.compile(r"([A-Za-z_]\w*)\s*\(")
    while i < n:
        m = name_re.search(text, i)
        if m is None:
            break
        depth = 1
        j = m.end()
        in_str = False
        while j < n and depth:
            ch = text[j]
            if ch == '"':
                in_str = not in_str
            elif not in_str and ch == "(":
                depth += 1
            elif not in_str and ch == ")":
                depth -= 1
            j += 1
        cmds.append((m.group(1).lower(), text[m.end() : j - 1], text.count("\n", 0, m.start()) + 1))
        i = j
    return cmds


def _split_args(raw: str) -> list[str]:
    return [a for a in re.split(r"\s+", raw.replace("\n", " ").strip()) if a]


def _expand(value: str, vars_: dict[str, list[str]]) -> list[str]:
    """Expand ``${VAR}`` (list-valued) with a depth cap; unknown vars stay literal."""
    for _ in range(8):
        m = re.search(r"\$\{(\w+)\}", value)
        if m is None:
            return [value.strip('"')] if value.strip('"') else []
        var = m.group(1)
        if var not in vars_:
            return [value.strip('"')] if value.strip('"') else []
        repl = vars_[var]
        head, tail = value[: m.start()], value[m.end() :]
        if len(repl) == 1:
            value = head + repl[0] + tail
            continue
        return [x for r in repl for x in _expand(head + r + tail, vars_)]
    return [value.strip('"')]


def _classify_entry(raw: str, module_dir: Path, cond: str | None) -> SourceEntry:
    low = raw.lower()
    if "cmake_current_binary_dir" in low or low.startswith("${cmake_binary_dir}"):
        kind = "generated"
        if low.endswith(".def"):
            kind = "def"
        return SourceEntry(raw, None, kind, cond)
    if low.endswith((".c", ".cpp", ".cc", ".cxx")):
        return SourceEntry(raw, (module_dir / raw).resolve(), "tu", cond)
    if low.endswith((".h", ".hpp")):
        return SourceEntry(raw, (module_dir / raw).resolve(), "header", cond)
    if low.endswith(".rc"):
        return SourceEntry(raw, (module_dir / raw).resolve(), "resource", cond)
    return SourceEntry(raw, None, "unknown", cond)


def _targets_compiling_c(
    cmake: Path, repo_root: Path, arch: str, exclude: str
) -> list[tuple[str, int]]:
    """Sibling targets in `cmake` that DO compile in-tree .c, as (name, count).

    Best-effort: a target that fails to parse is simply not offered as a suggestion.
    """
    out: list[tuple[str, int]] = []
    try:
        names = sorted(_target_names(cmake))
    except ProfileDeriveError:
        return out
    for name in names:
        if name == exclude:
            continue
        try:
            sibling = parse_cmakelists(cmake, repo_root, arch=arch, target=name)
        except ProfileDeriveError:
            continue
        count = sum(1 for p in sibling.compiled_tus if p.is_file())
        if count:
            out.append((name, count))
    out.sort(key=lambda item: -item[1])
    return out


_INTERFACE_INCLUDE_RE = re.compile(
    r"\btarget_include_directories\s*\(\s*(\w+)\s+INTERFACE\s+([^)]*)\)"
)
_INTERFACE_SCAN_ROOTS = ("sdk", "drivers", "dll", "win32ss", "ntoskrnl", "hal", "base", "boot")


@functools.lru_cache(maxsize=4)
def _interface_include_index(repo_root: Path) -> dict[str, tuple[Path, ...]]:
    """`target_include_directories(<name> INTERFACE <dirs>)` across the tree: name -> dirs."""
    index: dict[str, list[Path]] = {}
    for top in _INTERFACE_SCAN_ROOTS:
        for cm in (repo_root / top).rglob("CMakeLists.txt"):
            try:
                text = _strip_cmake_comments(cm.read_text(errors="replace"))
            except OSError:
                continue
            for m in _INTERFACE_INCLUDE_RE.finditer(text):
                for d in m.group(2).split():
                    d = d.replace("${CMAKE_CURRENT_SOURCE_DIR}", str(cm.parent)).replace(
                        "${REACTOS_SOURCE_DIR}", str(repo_root)
                    )
                    if "$" not in d:
                        path = Path(d) if Path(d).is_absolute() else cm.parent / d
                        index.setdefault(m.group(1), []).append(path)
    return {k: tuple(v) for k, v in index.items()}


_CARGO_KERNEL_RE = re.compile(r"\badd_cargo_module\s*\(\s*(\w+)[^)]*\bKIND\s+kernelexe\b")


def _cargo_kernel_targets(cmake: Path) -> set[str]:
    """`add_cargo_module(<name> ... KIND kernelexe)` targets in a CMakeLists (ntoskrnl)."""
    return set(_CARGO_KERNEL_RE.findall(_strip_cmake_comments(_read_with_includes(cmake))))


def _target_names(cmake: Path) -> set[str]:
    """Every add_library()/add_executable() target name in a CMakeLists, plus a cargo-linked
    kernel (`add_cargo_module(<name> ... KIND kernelexe)`, see parse_cmakelists)."""
    if not cmake.is_file():
        raise ProfileDeriveError(f"no CMakeLists.txt at {cmake}")
    text = _strip_cmake_comments(_read_with_includes(cmake))
    names = {
        m.group(2)
        for m in re.finditer(r"\b(add_library|add_executable)\s*\(\s*([A-Za-z0-9_${}]+)", text)
        if not m.group(2).startswith("$")
    }
    names.update(_CARGO_KERNEL_RE.findall(text))
    return names


def parse_cmakelists(
    cmake: Path,
    repo_root: Path,
    arch: str = "i386",
    target: str | None = None,
    flipped_sources: bool = False,
) -> CMakeModule:
    """Parse a module's CMakeLists into the set of TUs the build ACTUALLY compiles.

    Only a tiny CMake subset is interpreted (``set``/``list(APPEND)``,
    ``add_library``/``add_executable``, ``set_module_type``, ``add_definitions``,
    ``remove_definitions``, ``include_directories``, ``add_pch``, ``add_rpc_files``).
    ``if(ARCH STREQUAL "<a>")`` is resolved against *arch*; any OTHER conditional keeps
    its sources but records the guard on the entry, so a conditional source can never
    masquerade as unconditionally compiled.
    """
    if not cmake.is_file():
        raise ProfileDeriveError(f"no CMakeLists.txt at {cmake}")
    module_dir = cmake.parent
    raw_text = _read_with_includes(cmake)
    restored_sources: list[str] = []
    if flipped_sources:
        raw_text, restored_sources = uncomment_flipped_source_lists(raw_text, module_dir, repo_root)
    text = _strip_cmake_comments(raw_text)
    vars_: dict[str, list[str]] = {
        "CMAKE_CURRENT_SOURCE_DIR": [str(module_dir)],
        "REACTOS_SOURCE_DIR": [str(repo_root)],
        "CMAKE_SOURCE_DIR": [str(repo_root)],
        "ARCH": [arch],
    }
    findings: list[Finding] = []
    # (condition-text or None) stack; None = statically true.
    cond_stack: list[str | None] = []
    entries_by_var: dict[str, list[SourceEntry]] = {}
    targets: dict[str, tuple[str, list[str]]] = {}
    definitions: list[str] = []
    per_target_defs: dict[str, list[str]] = {}
    removed: list[str] = []
    include_dirs: list[Path] = []
    module_types: dict[str, tuple[str, bool]] = {}
    pch: Path | None = None
    pch_skip: set[Path] = set()
    rpc: list[str] = []
    link_deps: dict[str, list[str]] = {}

    def cond() -> str | None:
        live = [c for c in cond_stack if c]
        return " && ".join(live) if live else None

    for name, raw, _line in _cmake_commands(text):
        args_raw = _split_args(raw)
        if name == "if":
            m = re.match(r'ARCH\s+STREQUAL\s+"?(\w+)"?', raw.strip())
            if m:
                cond_stack.append(None if m.group(1) == arch else "__dead__")
            else:
                cond_stack.append(raw.strip())
            continue
        if name in ("elseif", "else"):
            if cond_stack:
                cond_stack[-1] = raw.strip() or "else"
            continue
        if name == "endif":
            if cond_stack:
                cond_stack.pop()
            continue
        if cond_stack and "__dead__" in cond_stack:
            continue  # a resolved-false ARCH branch: those sources are not compiled here
        args = [x for a in args_raw for x in _expand(a, vars_)]
        if name == "set" and args:
            vars_[args_raw[0]] = args[1:]
        elif name == "list" and len(args_raw) >= 2 and args_raw[0].upper() == "APPEND":
            var = args_raw[1]
            vals = [x for a in args_raw[2:] for x in _expand(a, vars_)]
            vars_.setdefault(var, []).extend(vals)
            entries_by_var.setdefault(var, []).extend(
                _classify_entry(v, module_dir, cond()) for v in vals
            )
        elif name in ("add_library", "add_executable") and args_raw:
            tgt = args[0]
            srcs = [a for a in args_raw[1:] if a not in ("MODULE", "SHARED", "STATIC", "WIN32")]
            targets[tgt] = ("library" if name == "add_library" else "executable", srcs)
        elif name == "add_cargo_module" and len(args_raw) >= 3 and "kernelexe" in args_raw:
            # ntoskrnl.exe graduated to a cargo-driven link: `add_cargo_module(ntoskrnl ...
            # KIND kernelexe ...)` takes no source list, and the kernel's C TUs are the
            # `${SOURCE}` list ntos.cmake builds (every one of them a flipped `#` entry now).
            # Without this, no ntoskrnl file had a provable owner and a sealed translate of
            # any of them refused outright.
            targets[args[0]] = ("executable", ["${SOURCE}"])
            module_types[args[0]] = ("kernel", False)
            if "EXTRA_DEPENDS" in args:
                link_deps.setdefault(args[0], []).extend(args[args.index("EXTRA_DEPENDS") + 1 :])
        elif name == "target_link_libraries" and len(args) >= 2:
            link_deps.setdefault(args[0], []).extend(
                a for a in args[1:] if a not in ("PUBLIC", "PRIVATE", "INTERFACE")
            )
        elif name == "set_module_type" and len(args) >= 2:
            module_types[args[0]] = (args[1], any(a.upper() == "UNICODE" for a in args[2:]))
        elif name == "add_definitions":
            definitions.extend(a for a in args if a.startswith("-D"))
        elif name == "target_compile_definitions" and len(args) >= 2:
            # `target_compile_definitions(disk PUBLIC _WIN32_WINNT=0x602 …)` — spelled WITHOUT
            # the -D and per target. `disk` carries its whole Win8 version profile this way, so
            # skipping it would silently derive the wrong struct layouts in a no-compile-db tree.
            per_target_defs.setdefault(args[0], []).extend(
                f"-D{a}" for a in args[1:] if a.upper() not in ("PUBLIC", "PRIVATE", "INTERFACE")
            )
        elif name == "remove_definitions":
            removed.extend(a for a in args if a.startswith("-D"))
        elif name == "include_directories":
            include_dirs.extend(Path(a) for a in args if not a.startswith(("BEFORE", "AFTER")))
        elif name == "add_pch" and len(args) >= 2:
            pch = (module_dir / args[1]).resolve()
            # add_pch(<target> <header> <SKIP-list>): the third argument names the TUs
            # that must NOT get the PCH — and they are skipped precisely because they do
            # not COMPILE with it. wldap32's ber.c is the case: it includes <winldap.h>
            # while precomp.h pulls "winldap_private.h", and the two declare 30-odd ldap_*
            # prototypes incompatibly. Injecting the PCH into its wrapper reproduces that
            # conflict exactly, so the skip list is load-bearing, not cosmetic.
            pch_skip.update((module_dir / a).resolve() for a in args[2:] if a.endswith(".c"))
        elif name == "add_rpc_files" and len(args) >= 2:
            rpc.extend(args[1:])

    if not targets:
        raise ProfileDeriveError(f"{cmake}: no add_library()/add_executable() found")
    if target is None:
        if len(targets) == 1:
            target = next(iter(targets))
        elif module_dir.name in targets:
            target = module_dir.name
        else:
            raise ProfileDeriveError(
                f"{cmake}: {len(targets)} targets ({', '.join(sorted(targets))}); "
                "pass --target to say which one owns the island"
            )
    if target not in targets:
        raise ProfileDeriveError(f"{cmake}: no target {target!r} (have: {', '.join(targets)})")

    def target_entries(name: str) -> list[SourceEntry]:
        out: list[SourceEntry] = []
        seen: set[str] = set()
        for a in targets[name][1]:
            m = re.fullmatch(r"\$\{(\w+)\}", a)
            expanded_entries = (
                entries_by_var.get(m.group(1), []) if m else [_classify_entry(a, module_dir, None)]
            )
            if m and m.group(1) not in entries_by_var and m.group(1) in vars_:
                expanded_entries = [_classify_entry(v, module_dir, None) for v in vars_[m.group(1)]]
            for e in expanded_entries:
                if e.raw in seen:
                    continue
                seen.add(e.raw)
                out.append(e)
        return out

    target_kind, _ = targets[target]
    entries = target_entries(target)
    # TUs a SIBLING target in this same CMakeLists compiles. setuplib's 24 .c are built by
    # `setuplib_static` and linked into `setuplib`; judged against `setuplib` alone they read
    # as dead source, and the finding said "do not pay to translate them" about live code.
    sibling_tus: dict[Path, str] = {}
    for other in targets:
        if other == target:
            continue
        for e in target_entries(other):
            if e.kind == "tu" and e.path is not None:
                sibling_tus.setdefault(e.path.resolve(), other)

    definitions.extend(per_target_defs.get(target, []))
    # A linked library's `target_include_directories(<lib> INTERFACE dir)` is on this target's
    # include path too. cmlib.h reaches ntoskrnl's internal/cm.h only this way, and upstream
    # moved evtlib/arbiter/infros/skiplist/concfg headers to the same mechanism.
    interface = _interface_include_index(repo_root)
    for dep in link_deps.get(target, []):
        include_dirs.extend(interface.get(dep, ()))
    mt = module_types.get(target)
    module = CMakeModule(
        cmake=cmake,
        module_dir=module_dir,
        name=target,
        target_kind=target_kind,
        module_type=mt[0] if mt else None,
        unicode=bool(mt and mt[1]),
        entries=entries,
        definitions=definitions,
        removed_definitions=removed,
        include_dirs=include_dirs,
        pch_header=pch,
        pch_skip=pch_skip,
        rpc_interfaces=rpc,
        findings=findings,
    )
    if module.module_type is None:
        # A static library has no set_module_type(); it runs in whatever image links it.
        # When every consumer in this CMakeLists agrees on the mode, inherit it rather than
        # make every caller remember --usermode (setuplib_static -> setuplib, a nativedll).
        consumers = sorted(t for t, deps in link_deps.items() if target in deps)
        kinds = {module_types[c][0] for c in consumers if c in module_types}
        modes = {k in KERNEL_MODULE_TYPES for k in kinds}
        if consumers and len(modes) == 1 and all(c in module_types for c in consumers):
            module.module_type = sorted(kinds)[0]
            if len(consumers) == 1:
                module.stem = consumers[0]
            findings.append(
                Finding(
                    "info",
                    "MODULE_TYPE_INHERITED",
                    f"{target!r} has no set_module_type(); it links into "
                    f"{', '.join(consumers)} ({', '.join(sorted(kinds))}), so it is "
                    f"{'kernel' if modes == {True} else 'usermode'} like them.",
                )
            )
    if restored_sources:
        findings.append(
            Finding(
                "warn",
                "FLIPPED_SOURCES_RESTORED",
                f"--flipped-sources re-enabled {len(restored_sources)} commented-out source(s) "
                f"({', '.join(sorted(restored_sources)[:8])}"
                f"{', …' if len(restored_sources) > 8 else ''}). This module is FLIPPED: the "
                "audit below describes the C the Rust replaced, not what the build compiles.",
            )
        )

    # Dead source: on disk, absent from the compiled list. Translating one of these
    # is paid-for work that nobody links (it happened last batch).
    compiled = {p.resolve() for p in module.compiled_tus}
    by_sibling: dict[str, list[Path]] = {}
    for cand in sorted(module_dir.rglob("*.c")) + sorted(module_dir.rglob("*.cpp")):
        if any(part in {"tests", "test", "build", ".git"} for part in cand.parts):
            continue
        if cand.resolve() in compiled:
            continue
        owner = sibling_tus.get(cand.resolve())
        if owner is not None:
            by_sibling.setdefault(owner, []).append(cand)
        else:
            module.dead_sources.append(cand)
    for owner, files in sorted(by_sibling.items()):
        findings.append(
            Finding(
                "warn",
                "SIBLING_TARGET",
                f"{len(files)} .c are compiled by sibling target {owner!r}, not {target!r}: "
                f"they are LIVE. To derive the island for them, pass --target {owner}: "
                + ", ".join(_rel(p, repo_root) for p in files[:6]),
            )
        )
    if module.dead_sources:
        findings.append(
            Finding(
                "warn",
                "DEAD_SOURCE",
                f"{len(module.dead_sources)} .c on disk are NOT in the source list: nothing "
                "compiles them, so do not pay to translate them (they are dead source, or "
                "already Rust-flipped and kept per the never-delete rule): "
                + ", ".join(_rel(p, repo_root) for p in module.dead_sources[:6]),
            )
        )
    _audit_dead_bodies(module, repo_root, findings)
    for e in entries:
        if e.kind == "tu" and e.condition:
            findings.append(
                Finding(
                    "warn",
                    "CONDITIONAL_SOURCE",
                    f"{e.raw} is compiled only under `{e.condition}` — confirm before translating",
                )
            )
        if e.kind == "tu" and e.path is not None and not e.path.is_file():
            findings.append(
                Finding("error", "MISSING_SOURCE", f"{e.raw} is in the source list but not on disk")
            )
    return module


# ---------------------------------------------------------------------------
# Include closure
# ---------------------------------------------------------------------------


def build_include_env(
    repo_root: Path,
    module: CMakeModule,
    arch: str = "i386",
    extra_dirs: tuple[Path, ...] = (),
    compile_dirs: tuple[Path, ...] = (),
) -> IncludeEnv:
    """Assemble the include search path in the order the real compile uses it.

    *compile_dirs* is the module's own ``-I`` list from the compile db, and when present it
    is searched FIRST, in its own order: it is the only authority on precedence. The fixed
    list below is the fallback for a module with no db entry. Appending the db list after it
    resolved comctl32's ``commctrl.h`` to psdk's when the compiler takes sdk/include/wine's
    (a wine module puts that dir ahead of psdk), so the two wine-only constants its switches
    use read as owned by no header at all.

    Order is taken from the module's actual compile command (see
    ``build-<arch>/compile_commands.json``) and from ``gen_driver_types.sh``'s own
    ``-I`` list, which is the proven-working one:

      1. ``build-<arch>/<module rel path>``  — generated headers (MIDL ``*_c.h``/``*_s.h``)
      2. the module's own directory (+ ``include/`` if it has one)
      3. ``include_directories()`` from the module's CMakeLists (e.g. reactos/idl)
      4. ``sdk/include`` and its psdk/ddk/ndk/reactos/crt/... children
      5. ``build-<arch>/sdk/include*`` — the configure-time GENERATED xdk/mc headers
      6. ``sdk/include/xdk`` as the fallback source of (5) when no build dir exists
    """
    build_dir = repo_root / f"build-{arch}"
    if not build_dir.is_dir():
        build_dir = None
    search: list[tuple[Path, str]] = [(d, "compile") for d in compile_dirs]
    if build_dir is not None:
        rel = module.module_dir.relative_to(repo_root)
        search.append((build_dir / rel, "generated"))
    search.append((module.module_dir, "module"))
    if (module.module_dir / "include").is_dir():
        search.append((module.module_dir / "include", "module"))
    # CMake resolves a relative include_directories() entry against the CMakeLists' own
    # directory, not the repo root (setuplib: `include_directories(spapisup utils)`).
    search.extend(
        (d if d.is_absolute() else module.module_dir / d, "idl") for d in module.include_dirs
    )
    search.extend((repo_root / p, "sdk") for p in _SDK_SEARCH)
    if build_dir is not None:
        search.extend((build_dir / p, "generated") for p in _BUILD_SEARCH)
    else:
        # xdk headers are GENERATED into build-*/sdk/include; without a build dir the
        # source templates are the only stand-in. Say so (the report prints it).
        search.append((repo_root / "sdk/include/xdk", "sdk"))
    search.extend((d, "extra") for d in extra_dirs)
    return IncludeEnv(
        repo_root=repo_root,
        arch=arch,
        module_dir=module.module_dir,
        build_dir=build_dir,
        search=[(d, k) for d, k in search if d.is_dir()],
    )


_SDK_SEARCH = (
    "sdk/include",
    "sdk/include/crt",
    "sdk/include/ddk",
    "sdk/include/ndk",
    "sdk/include/psdk",
    "sdk/include/reactos",
    "sdk/include/reactos/libs",
    "sdk/include/reactos/drivers",
    "sdk/include/vcruntime",
    "sdk/include/winrt",
    "sdk/include/dxsdk",
    "sdk/lib/pseh/include",
    "sdk/include/wine",
)
_BUILD_SEARCH = (
    "sdk/include",
    "sdk/include/psdk",
    "sdk/include/ddk",
    "sdk/include/dxsdk",
    "sdk/include/reactos",
    "sdk/include/reactos/mc",
)

# The -I set gen_driver_types.sh ALREADY passes to bindgen. An include dir we resolve
# through must be emitted into EXTRA_INC/LATE_INC unless it is in here.
#
# This is deliberately NOT `_SDK_SEARCH`. _SDK_SEARCH is where WE look to resolve a
# header; this is what the GENERATOR looks in. They differ, and treating them as one
# emits a profile that cannot generate: wldap32 includes "config.h", which lives only in
# sdk/include/wine, and the derived profile omitted that -I because _SDK_SEARCH lists it
# — bindgen then died with `'config.h' file not found`. samlib hit the same gap from the
# other side and was patched per-instance (a hand-added LATE_INC after a header bisect);
# that was the symptom, this is the cause. Keep in sync with the bindgen invocation.
_GENERATOR_BASE_INC = frozenset(
    {
        "sdk/include",
        "sdk/include/ddk",
        "sdk/include/ndk",
        "sdk/include/psdk",
        "sdk/include/xdk",
        "sdk/include/reactos",
        "sdk/include/reactos/libs",
        "sdk/include/reactos/drivers",
        "sdk/include/vcruntime",
        "sdk/include/dxsdk",
        "sdk/lib/pseh/include",
        "sdk/include/crt",  # passed as -isystem
    }
)
# `build-<arch>/sdk/...` — the generator passes these as `${BUILD_DIR}/sdk/...`, per arch.
_BUILD_SDK_RE = re.compile(r"^build-[^/]+/sdk/")


# Macros whose truth value is FIXED for every build in this tree, so a branch guarded on
# them is not "conditional", it is dead (or live) code. `__REACTOS__` is defined by the root
# CMakeLists for every TU — and by gen_driver_types.sh's own bindgen invocation — which makes
# the wine-derived `#ifndef __REACTOS__` idiom (a `case DLL_WINE_PREATTACH:` that ReactOS
# never compiles) unambiguously dead. Reading those arms produced a FALSE §9.112 alarm, and a
# checker that cries wolf gets ignored.
# One rule, one home: `dead_regions` owns it, because `translate` needs the same verdict to
# keep a dead body out of the PAID worklist and two copies of "is this branch compiled" is
# exactly the drift that would let the two disagree about the same `#ifndef`.
_cond_truth = cond_truth


def scan_includes(text: str) -> list[Include]:
    """Every ``#include`` in a file, in order.

    Over-approximating is the safe direction for an allowlist: an extra header makes the
    island bigger, a missing one makes a ``switch`` silently collapse (§9.112). So BOTH arms
    of an unknown conditional are kept. Three refinements keep it honest:

    * a branch guarded on a fixed condition (``#if 0``, ``#ifndef __REACTOS__``, and the
      ``#else`` of ``#ifdef __REACTOS__``) is genuinely dead and skipped — while its live
      sibling arm is still walked, which a naive "skip to #endif" would lose;
    * each include records whether it sits inside an UNKNOWN conditional, DISCOUNTING the
      file's own include guard (otherwise every include in every header looks conditional).
      Only an unconditional unresolved include means our search path is wrong;
    * ``#include SOME_MACRO`` is reported by the caller, never skipped silently.
    """
    out: list[Include] = []
    # frame = (dead, known) per open conditional
    stack: list[tuple[bool, bool]] = []
    guard_depth: int | None = None
    lines = text.splitlines()
    for n, line in enumerate(lines, 1):
        d = _DIRECTIVE_RE.match(line)
        if d:
            directive = d.group(1)
            if directive in ("if", "ifdef", "ifndef"):
                truth = _cond_truth(line)
                stack.append((truth is False, truth is not None))
                if guard_depth is None and len(stack) == 1 and _looks_like_include_guard(lines, n):
                    guard_depth = 1
                continue
            if directive in ("else", "elif") and stack:
                dead, known = stack[-1]
                if directive == "else" and known:
                    stack[-1] = (not dead, True)  # the sibling arm of a FIXED condition
                else:
                    stack[-1] = (False, False)  # unknown -> keep walking (over-approximate)
                continue
            if directive == "endif":
                if guard_depth is not None and len(stack) == guard_depth:
                    guard_depth = None
                if stack:
                    stack.pop()
                continue
        if any(dead for dead, _ in stack):
            continue
        m = _INCLUDE_RE.match(line)
        if m is None:
            continue
        spelling = m.group(1) or m.group(2)
        unknown = sum(1 for _, known in stack if not known) - (1 if guard_depth is not None else 0)
        out.append(
            Include(spelling, spelling[1:-1], spelling.startswith("<"), n, conditional=unknown > 0)
        )
    return out


def _looks_like_include_guard(lines: list[str], directive_line: int) -> bool:
    """``#ifndef _FOO_H`` immediately followed by ``#define _FOO_H`` — the guard idiom."""
    m = re.match(
        r"^\s*#\s*(?:ifndef\s+(\w+)|if\s+!\s*defined\s*\(?\s*(\w+))", lines[directive_line - 1]
    )
    if m is None:
        return False
    sym = m.group(1) or m.group(2)
    for line in lines[directive_line : directive_line + 3]:
        if re.match(rf"^\s*#\s*define\s+{re.escape(sym)}\b", line):
            return True
    return False


_IFDEF_RE = re.compile(r"^\s*#\s*if(n?)def\s+([A-Za-z_]\w*)")
_IF_DEFINED_RE = re.compile(r"^\s*#\s*if\s+(!?)\s*defined\s*\(?\s*([A-Za-z_]\w*)\s*\)?\s*$")


def _guard_macro(line: str) -> tuple[str, bool] | None:
    """(macro, wants_defined) for a single-macro `#ifdef`/`#ifndef`/`#if defined(X)`."""
    m = _IFDEF_RE.match(line)
    if m is not None:
        return m.group(2), m.group(1) != "n"
    m = _IF_DEFINED_RE.match(line)
    if m is not None:
        return m.group(2), m.group(1) != "!"
    return None


_GUARD_OPEN_RE = re.compile(r"^\s*#\s*if(n?)def\s+([A-Za-z_]\w*)\s*$", re.M)

# `#if <expr>` — the whole expression, for the integer-gate evaluator below.
_IF_EXPR_RE = re.compile(r"^\s*#\s*if\s+(.+?)\s*$")


def _pinned_int_macros(module) -> dict[str, int]:
    """`pinned_int_macros` over a parsed module's own `-D` list.

    The evaluator itself lives in `dead_regions` because the SAME question -- is this
    `#if` decidably false for this target? -- decides two things: whether a switch label
    is real (here) and whether a function body is on the bill (`never_compiled_lines`).
    Two copies drifted once already.
    """
    return pinned_int_macros(module.definitions)


_eval_int_guard = eval_int_guard


def _audit_dead_bodies(module, repo_root: Path, findings: list) -> None:
    """Warn when most of a module's CODE sits in a decidably-FALSE `#ifdef`.

    This is a PRE-PAYMENT check and it exists because skipping it cost real money.
    wldap32 looks like an 8,625-line LDAP client; in this tree every one of its entry
    points compiles to `return WLDAP32_LDAP_NOT_SUPPORTED;` because 181 `#ifdef HAVE_LDAP`
    blocks across 19 of its 20 TUs are disabled — ReactOS does not vendor OpenLDAP. It is
    a STUB DLL. Translating it bought 162 functions of the path the compiler throws away,
    and the emitted Rust was worse than useless: unconditional, so it changed behaviour,
    and calling OpenLDAP symbols that do not exist here, so it could never link.

    The dead-region seal already tolerates an omission inside such a block, but tolerating
    is not the same as telling you the module is a stub before you pay.

    TWO classes of gate, and BOTH are needed. wine's `#undef` settles `#ifdef HAVE_LDAP`;
    the module's own `-DNAME=<int>` settles `#if _MSVCR_VER >= 100`. msvcrt is the case
    that proved the second one is not optional: concurrency.c is 3,839 lines wholly inside
    `#if _MSVCR_VER >= 100`, handler4.c 775 more inside
    `defined(__x86_64__) && _MSVCR_VER>=140`, and msvcrt pins `_MSVCR_VER=0`. Both compile
    to a 396-byte object with ZERO defined symbols.

    AND the finding has to be PER-TU, not just module-wide. Those two files are 10.6% of
    msvcrt's compiled lines — under the 15% module threshold, so the module-wide warning
    stayed silent on 4,614 lines of guaranteed-dead code. You choose a FILE to translate,
    so a wholly-dead file is the finding that actually saves the money.
    """
    undefs = _wine_config_undefs(repo_root)
    pinned = _pinned_int_macros(module)
    if not undefs and not pinned:
        return
    total_lines = 0
    dead_lines = 0
    culprits: dict[str, int] = {}
    for tu in module.compiled_tus:
        try:
            text = tu.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        lines = text.splitlines()
        total_lines += len(lines)
        tu_lines = len(lines)
        tu_dead = 0
        depth_dead = 0
        stack: list[bool | None] = []
        for line in lines:
            m = _GUARD_OPEN_RE.match(line)
            if m is not None:
                macro = m.group(2)
                dead = macro in undefs and m.group(1) != "n"
                stack.append(dead)
                if dead:
                    depth_dead += 1
                    culprits[macro] = culprits.get(macro, 0) + 1
                continue
            m = _IF_EXPR_RE.match(line)
            if m is not None:
                expr = m.group(1)
                verdict = _eval_int_guard(expr, pinned) if pinned else None
                # `None` (undecidable) must stay None, not False: the `#else` inverter
                # below only flips a verdict it actually knows.
                stack.append(None if verdict is None else not verdict)
                if verdict is False:
                    depth_dead += 1
                    for name in set(re.findall(r"[A-Za-z_]\w*", expr)) & set(pinned):
                        culprits[name] = culprits.get(name, 0) + 1
                continue
            if re.match(r"^\s*#\s*else\s*$", line) and stack:
                if stack[-1] is not None:
                    if stack[-1]:
                        depth_dead -= 1
                    stack[-1] = not stack[-1]
                    if stack[-1]:
                        depth_dead += 1
                continue
            if re.match(r"^\s*#\s*elif\b", line) and stack:
                # An `#elif` arm is live exactly when the arms before it were false, which
                # is a question this evaluator does not answer. Drop to UNKNOWN rather than
                # inheriting the previous arm's verdict — inheriting it would report the
                # `#elif` body as dead on the strength of a test that no longer applies.
                if stack[-1]:
                    depth_dead -= 1
                stack[-1] = None
                continue
            if re.match(r"^\s*#\s*endif\b", line) and stack:
                if stack.pop():
                    depth_dead -= 1
                continue
            if depth_dead > 0:
                dead_lines += 1
                tu_dead += 1
        # A file that is ~entirely dead is its own finding, at ERROR: the module-wide
        # percentage below can be under threshold while a specific file you are about to
        # pay for is 100% dead. msvcrt is exactly that shape.
        if tu_lines and 100 * tu_dead / tu_lines >= 90:
            findings.append(
                Finding(
                    "error",
                    "DEAD_TU",
                    f"{_rel(tu, repo_root)} is {100 * tu_dead / tu_lines:.0f}% dead: "
                    f"{tu_dead} of its {tu_lines} lines sit inside a decidably-FALSE "
                    "`#if`, so this TU compiles to (almost) nothing — verify with "
                    "`nm` on its .obj. It is in the source list, so it LOOKS live and "
                    "the DEAD_SOURCE check cannot see it. DO NOT PAY TO TRANSLATE IT.",
                    subject=_rel(tu, repo_root),
                )
            )
    if not total_lines or not culprits:
        return
    pct = 100 * dead_lines / total_lines
    if pct < 15:
        return
    top = ", ".join(
        f"{m} ({n} blocks)" for m, n in sorted(culprits.items(), key=lambda kv: -kv[1])[:3]
    )
    findings.append(
        Finding(
            "warn",
            "MOSTLY_DEAD_BODIES",
            f"{pct:.0f}% of this module's compiled lines are inside a decidably-FALSE "
            f"#if ({top}) — wine's config.h #undefs it, or the module pins it to a value "
            "with -D, so the compiler throws that code away and the real module is far "
            "smaller than it looks (often a pure stub). "
            "PAYING TO TRANSLATE IT BUYS THE PATH THAT IS NOT BUILT, and the output is "
            "unconditional Rust that changes behaviour and calls symbols this tree does not "
            "have. Read what the module actually compiles to BEFORE translating.",
        )
    )


_CONFIG_UNDEF_RE = re.compile(r"^\s*/\*\s*#\s*undef\s+([A-Za-z_]\w*)\s*\*/\s*$", re.M)


@functools.lru_cache(maxsize=4)
def _wine_config_undefs(repo_root: Path) -> frozenset[str]:
    """Macros wine's config.h explicitly leaves UNDEFINED for this tree.

    autoconf writes the off cases as `/* #undef HAVE_LDAP_H */`, which is an assertion
    about THIS build, not a maybe — every wine-sourced module's `#ifdef HAVE_*` prologue
    guard is decidable from it.
    """
    cfg = repo_root / "sdk" / "include" / "wine" / "config.h"
    try:
        text = cfg.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return frozenset()
    return frozenset(_CONFIG_UNDEF_RE.findall(text))


def scan_preamble(
    text: str,
    known_undefined: frozenset[str] = frozenset(),
    known_defined: frozenset[str] = frozenset(),
) -> tuple[list[str], bool]:
    """The TU's include PROLOGUE, verbatim, plus "had UNDECIDED conditional directives".

    Mirroring the real prologue (including its ``#define WIN32_NO_STATUS`` /
    ``#define NTOS_MODE_USER`` ordering) is what makes a wrapper guaranteed-legal:
    if the ``.c`` compiles, that exact sequence is bindgen-able (§9.108).

    Guarded includes are the exception, and it is not cosmetic: this used to drop the
    `#ifdef`/`#endif` while KEEPING the include between them, promoting a conditional
    include to an unconditional one. wldap32 wraps OpenLDAP under `#ifdef HAVE_LDAP_H`,
    which wine's config.h `#undef`s for this tree — the mirrored wrapper therefore asked
    for a header that does not exist here and bindgen died on it.

    *known_undefined* is the decidable set (in practice wine's config.h `#undef` list).
    A guard on a macro in it is resolved and the dead arm dropped; a guard we cannot
    decide keeps BOTH arms as before and sets the "conditional" flag so the caller warns.
    *known_defined* is the other half: macros every compile of this TU defines. fltmc's
    `#ifdef __REACTOS__ ... #else #include <Windows.h> #endif` kept the MSVC arm, and the
    island asked a case-sensitive host for a `Windows.h` that no ReactOS compile opens.
    """
    lines = text.splitlines()
    end = 0
    in_comment = False
    for i, line in enumerate(lines):
        s = line.strip()
        if in_comment:
            if "*/" in s:
                in_comment = False
            continue
        if s.startswith("/*") and "*/" not in s:
            in_comment = True
            continue
        if not s or s.startswith(("//", "/*")):
            continue
        if s.startswith("#"):
            if _INCLUDE_RE.match(line):
                end = i + 1
            continue
        break  # first real code line ends the prologue
    keep: list[str] = []
    conditional = False
    # stack of "this arm is dead" flags; None = undecided (keep both arms, as before)
    arms: list[bool | None] = []
    for line in lines[:end]:
        d = _DIRECTIVE_RE.match(line)
        if d is None:
            continue
        kind = d.group(1)
        if kind in ("ifdef", "ifndef", "if"):
            guard = _guard_macro(line)
            if guard is not None and guard[0] in known_undefined:
                # dead when the arm wanted it DEFINED
                arms.append(guard[1])
            elif guard is not None and guard[0] in known_defined:
                arms.append(not guard[1])
            else:
                conditional = True
                arms.append(None)
            continue
        if kind in ("elif",):
            conditional = True
            if arms:
                arms[-1] = None
            continue
        if kind == "else":
            if arms and arms[-1] is not None:
                arms[-1] = not arms[-1]
            continue
        if kind == "endif":
            if arms:
                arms.pop()
            continue
        if any(a for a in arms):
            continue  # inside a decidably-dead arm
        if _PACKING_RE.search(line):
            # A packing include opens a region that the TU closes AFTER its prologue
            # (partlist.c: `#include <pshpack1.h>`, two typedefs, `#include <poppack.h>`), so
            # the prologue holds only the push. Carried into the module's merged wrapper it
            # packed every header after it: setuplib's island laid out USETUP_DATA,
            # FILE_SYSTEM and 60-odd more `#[repr(C, packed)]`, which C never does.
            continue
        if kind in ("include", "define", "undef"):
            keep.append(line.strip())
    return keep, conditional


_PACKING_RE = re.compile(r"#\s*include\s*[<\"](?:pshpack\d*|poppack)\.h[>\"]|#\s*pragma\s+pack\b")


def resolve_include(inc: Include, includer: Path, env: IncludeEnv) -> Path | None:
    """Resolve one include. Quoted form searches the includer's directory FIRST."""
    cands: list[Path] = []
    # A Windows-style `..\\diblib\\DibLib_interface.h` (win32ss/gdi/dib/dib_new.c) is the same
    # path to the compilers this tree is built with; on a POSIX host it is one odd filename.
    name = inc.name.replace("\\", "/")
    if not inc.angled:
        cands.append(includer.parent / name)
    cands.extend(d / name for d, _ in env.search)
    for c in cands:
        if c.is_file():
            return c.resolve()
    return None


def walk_includes(tus: list[Path], env: IncludeEnv) -> Closure:
    """BFS the include closure of *tus*. Unresolved includes are RECORDED, never dropped."""
    closure = Closure()
    unresolved: dict[tuple[str, str, int], Unresolved] = {}
    for tu in tus:
        tu_key = _rel(tu, env.repo_root)
        reached: set[Path] = set()
        queue: list[tuple[Path, int]] = [(tu, 0)]
        seen_files = {tu.resolve()}
        while queue:
            cur, depth = queue.pop(0)
            try:
                text = _read(cur)
            except OSError as exc:  # pragma: no cover - unreadable file
                key = (cur.name, tu_key, 0)
                unresolved[key] = Unresolved(cur.name, tu_key, 0, f"unreadable: {exc}")
                continue
            for inc in scan_includes(text):
                target = resolve_include(inc, cur, env)
                if target is None:
                    reason = (
                        "toolchain/CRT header (not in-tree)"
                        if inc.name in CRT_HEADERS
                        else "NOT FOUND on the include path"
                    )
                    if inc.conditional:
                        reason += " [inside #if — probably a dead platform arm]"
                    key = (inc.name, _rel(cur, env.repo_root), inc.line)
                    unresolved[key] = Unresolved(
                        inc.name,
                        _rel(cur, env.repo_root),
                        inc.line,
                        reason,
                        conditional=inc.conditional,
                    )
                    continue
                ref = closure.headers.get(target)
                if ref is None:
                    ref = HeaderRef(
                        path=target,
                        name=inc.name,
                        kind=env.classify(target),
                        depth=depth + 1,
                    )
                    closure.headers[target] = ref
                ref.depth = min(ref.depth, depth + 1)
                includer = _rel(cur, env.repo_root)
                if includer not in ref.includers:
                    ref.includers.append(includer)
                ref.tus.add(tu_key)
                reached.add(target)
                # The toolchain's own headers (libc++'s <vector>, <map>) are recorded but never
                # descended into: they feed no allowlist (see the kind filter in derive), and
                # their internals select platform arms by `#if` the lexical walker cannot
                # evaluate -- drwtsn32's <map> reached libc++'s android.h and pthread.h and
                # failed the derive on headers no ReactOS compile ever opens.
                if target not in seen_files and ref.kind != "toolchain":
                    seen_files.add(target)
                    queue.append((target, depth + 1))
            # macro-spelled includes (`#include SOME_MACRO`) cannot be walked lexically
            for line_no, line in enumerate(text.splitlines(), 1):
                if _INCLUDE_RE.match(line) or not line.lstrip().startswith("#"):
                    continue
                mm = _INCLUDE_MACRO_RE.match(line)
                if mm:
                    key = (mm.group(1), _rel(cur, env.repo_root), line_no)
                    unresolved[key] = Unresolved(
                        mm.group(1),
                        _rel(cur, env.repo_root),
                        line_no,
                        "macro-spelled #include (needs a preprocessor)",
                        conditional=True,
                    )
        closure.per_tu[tu_key] = reached
    closure.unresolved = sorted(unresolved.values(), key=lambda u: (u.includer, u.line, u.name))
    return closure


# ---------------------------------------------------------------------------
# The §9.112 hazard: constants a `switch` dispatches on
# ---------------------------------------------------------------------------


def find_switch_constants(tus: list[Path], repo_root: Path) -> list[SwitchConst]:
    """Identifiers appearing in ``case`` labels, via tree-sitter (no regex guessing).

    A hard dependency on purpose: a regex fallback would quietly miss arms, and a
    missed arm is the exact silent-catch-all bug this check exists to prevent.
    """
    try:
        from tree_sitter import Parser

        from .scanner import C_LANG
    except Exception as exc:  # pragma: no cover - env without tree-sitter
        raise ProfileDeriveError(
            "tree-sitter (tree_sitter + tree_sitter_c) is required for the switch-constant "
            f"audit and could not be imported: {exc}. Install the rustconv requirements; "
            "there is deliberately no regex fallback (it would miss arms silently)."
        ) from exc

    parser = Parser(C_LANG)
    found: list[SwitchConst] = []
    for tu in tus:
        src = tu.read_bytes()
        tree = parser.parse(src)
        key = _rel(tu, repo_root)
        stack = [tree.root_node]
        while stack:
            node = stack.pop()
            stack.extend(node.children)
            if node.type != "case_statement":
                continue
            value = node.child_by_field_name("value")
            if value is None:
                continue  # `default:`
            if _in_dead_preproc_branch(node, src):
                continue  # e.g. wine's `#ifndef __REACTOS__ case DLL_WINE_PREATTACH:`
            inner = [value]
            while inner:
                v = inner.pop()
                inner.extend(v.children)
                if v.type == "identifier":
                    # `case MAKEWPARAM(IDOK, BN_CLICKED):` -- the callee is a function-like
                    # macro, not a constant an arm could bind; its ARGUMENTS are the arms.
                    parent = v.parent
                    if (
                        parent is not None
                        and parent.type == "call_expression"
                        and parent.child_by_field_name("function") == v
                    ):
                        continue
                    text = src[v.start_byte : v.end_byte].decode("utf-8", "replace")
                    found.append(SwitchConst(text, key, v.start_point[0] + 1))
        found.extend(_macro_switch_constants(src, key, parser))
    # stable, de-duplicated by (ident, tu, line)
    return sorted(set(found), key=lambda s: (s.tu, s.line, s.ident))


def _macro_switch_constants(src: bytes, key: str, parser) -> list[SwitchConst]:
    """`case` labels that only exist AFTER a TU-local function-like macro expands.

    The plain AST walk above sees the *unexpanded* source, so a TU whose arms are written
    ``ERR_CASE(ERROR_ACCESS_DENIED)`` has no ``case_statement`` node at all and audits as
    clean — silently, which is the one failure mode this whole check exists to prevent.
    msvcrt's errno.c hides all 39 of its arms that way.

    Resolution is still tree-sitter, never a regex over ``case``: each parameter is replaced
    by a unique sentinel, the body is parsed as a real switch, and a parameter counts only if
    its sentinel lands in a ``case`` label *value*. So ``ERR_MAPS(oserr, crterr)`` contributes
    ``oserr`` and not ``crterr`` — the latter is the assigned value, not an arm.
    """
    # Local, like the C_LANG import in the caller: scanner.py pulls in tree_sitter_c at
    # module scope, and profile_derive keeps that a soft dependency so a missing parser
    # surfaces as ProfileDeriveError rather than a bare ImportError at import time.
    from .scanner import FN_MACRO_RE, macro_body, split_macro_args

    src_text = src.decode("utf-8", "replace")
    # name -> parameter indices that reach a `case` label
    case_params: dict[str, list[int]] = {}
    # `#define ERR_CASE(oserr)` is itself a textual `ERR_CASE(` match, and its argument is
    # the PARAMETER NAME. Skipping the definition's own span is what keeps `oserr` from
    # being reported as a 43rd switch constant that no arm ever names.
    def_spans: dict[str, list[tuple[int, int]]] = {}
    for m in FN_MACRO_RE.finditer(src_text):
        name, params_src = m.group(1), m.group(2)
        def_spans.setdefault(name, []).append((m.start(), m.end()))
        body = macro_body(src_text, m.start(3))
        if not re.search(r"\bcase\b", body):
            continue
        params = [p.strip() for p in params_src.split(",") if p.strip()]
        if not params:
            continue
        sentinels = {f"__rcv_case_param_{i}__": i for i in range(len(params))}
        probe = body
        for i, p in enumerate(params):
            probe = re.sub(rf"\b{re.escape(p)}\b", f"__rcv_case_param_{i}__", probe)
        # A replacement list is a fragment; give it a switch to live in so `case` parses.
        probe_bytes = f"void __rcv_probe(void){{switch(0){{{probe}\n}}}}".encode()
        tree = parser.parse(probe_bytes)
        hits: set[int] = set()
        stack = [tree.root_node]
        while stack:
            node = stack.pop()
            stack.extend(node.children)
            if node.type != "case_statement":
                continue
            value = node.child_by_field_name("value")
            if value is None:
                continue
            inner = [value]
            while inner:
                v = inner.pop()
                inner.extend(v.children)
                if v.type == "identifier":
                    text = probe_bytes[v.start_byte : v.end_byte].decode("utf-8", "replace")
                    if text in sentinels:
                        hits.add(sentinels[text])
        if hits:
            case_params[name] = sorted(hits)
    if not case_params:
        return []

    found: list[SwitchConst] = []
    for name, idxs in case_params.items():
        for call in re.finditer(rf"\b{re.escape(name)}[ \t]*\(", src_text):
            if any(lo <= call.start() < hi for lo, hi in def_spans.get(name, ())):
                continue  # the `#define` line itself
            args = split_macro_args(src_text, call.end())
            if args is None:
                continue  # unbalanced (truncated file / inside a string) — not an arm
            line = src_text.count("\n", 0, call.start()) + 1
            for i in idxs:
                if i >= len(args):
                    continue
                # The argument may be `(x)` or a cast; take every identifier in it, the
                # same rule the AST walk uses for a literal `case` value.
                for ident in re.findall(r"[A-Za-z_]\w*", args[i]):
                    found.append(SwitchConst(ident, key, line))
    return found


#: Names that are CALLED like functions and are never a hand-written FFI burden: the CRT
#: (a Rust crate gets these from its own substrate, not from this island) and the fixed
#: NT/Win32 macro vocabulary that expands inline. Both contaminate a raw `called - defined`
#: count badly enough to move a module a whole band — tcpsvcs drops from 66 to ~44.
_NOT_A_PROTOTYPE = frozenset(
    [
        "strlen",
        "wcslen",
        "strcpy",
        "wcscpy",
        "strcat",
        "wcscat",
        "strcmp",
        "wcscmp",
        "stricmp",
        "wcsicmp",
        "strncmp",
        "wcsncmp",
        "strchr",
        "wcschr",
        "strrchr",
        "wcsrchr",
        "strstr",
        "wcsstr",
        "strtok",
        "wcstok",
        "sprintf",
        "swprintf",
        "snprintf",
        "_snwprintf",
        "vsprintf",
        "vswprintf",
        "sscanf",
        "swscanf",
        "atoi",
        "atol",
        "wtoi",
        "wcstol",
        "wcstoul",
        "strtoul",
        "strtol",
        "malloc",
        "calloc",
        "realloc",
        "free",
        "memcpy",
        "memmove",
        "memset",
        "memcmp",
        "qsort",
        "bsearch",
        "abs",
        "labs",
        "rand",
        "srand",
        "time",
        "localtime",
        "gmtime",
        "mktime",
        "toupper",
        "tolower",
        "towupper",
        "towlower",
        "isdigit",
        "iswdigit",
        "isalpha",
        "iswalpha",
        "isspace",
        "iswspace",
        "isalnum",
        "iswalnum",
        "printf",
        "fprintf",
        "fputs",
        "fopen",
        "fclose",
        "fread",
        "fwrite",
        "exit",
        "abort",
        "assert",
        "va_start",
        "va_end",
        "va_arg",
        "offsetof",
    ]
)

#: Function-like macro spellings the include closure declares as macros rather than functions.
#: Derived per module (see `_external_prototype_survey`); this is only the fallback vocabulary
#: for the ones every module uses and no closure header spells as a prototype.
_ALWAYS_MACRO = frozenset(
    [
        "ARRAYSIZE",
        "_countof",
        "RTL_NUMBER_OF",
        "FIELD_OFFSET",
        "CONTAINING_RECORD",
        "MAKELANGID",
        "MAKEWORD",
        "MAKELONG",
        "LOWORD",
        "HIWORD",
        "LOBYTE",
        "HIBYTE",
        "MAKEINTRESOURCE",
        "ZeroMemory",
        "CopyMemory",
        "MoveMemory",
        "FillMemory",
        "RtlZeroMemory",
        "RtlCopyMemory",
        "NT_SUCCESS",
        "NT_ERROR",
        "NT_WARNING",
        "SUCCEEDED",
        "FAILED",
        "HRESULT_FROM_WIN32",
        "TRACE",
        "WARN",
        "ERR",
        "FIXME",
        "DPRINT",
        "DPRINT1",
        "ASSERT",
        "InitializeListHead",
        "IsListEmpty",
        "InsertHeadList",
        "InsertTailList",
        "RemoveEntryList",
        "RemoveHeadList",
        "RemoveTailList",
        "FD_SET",
        "FD_ZERO",
        "FD_ISSET",
        "FD_CLR",
        "min",
        "max",
    ]
)


@dataclass(frozen=True)
class PrototypeSurvey:
    """How much FFI surface `BLOCKLIST_FUNCS=1` would make somebody hand-write.

    IT REPORTS. IT DOES NOT RECOMMEND. `called - defined` over the twelve modules whose flag
    was set by hand very nearly separates them:

        intl 111 | desk 359 | sysdm 268 | syssetup 301 | umpnpmgr 101 | aclui 163 | samlib 73
        tcpsvcs 66 | mswsock 39 | wlanapi 20 | snmpapi 18 | wsock32 0

    — the first seven chose 0, the last five kept 1, and samlib 73 against tcpsvcs 66 is a
    7-unit margin on twelve samples. Refining it to the prototypes the include closure really
    DECLARES (dropping the CRT and the macro vocabulary) tightens the number a lot:

        intl 69 | desk 198 | sysdm 130 | syssetup 205 | aclui 97 | samlib 60     <- chose 0
        tcpsvcs 47 | wlanapi 16 | snmpapi 7 | wsock32 0                          <- kept 1

    60 against 47 is a real gap, and it is still not a rule. Ten samples, and the pair that
    defines the boundary is exactly the pair whose reasoning was never about the count:
    samlib's own comment argues that a hand-written declaration of an RPC-shaped SAM
    entrypoint wrong by one pointer level LINKS and then corrupts at run time. That is a RISK
    argument. A threshold fitted to ten points would be picking against the only written-down
    rationale in the set.

    So the evidence is printed — both counts and every name — and the flag stays a human
    decision, settled the MACRO_SIGNED way: island it both ways, one cargo check each. Same
    discipline as the substrate protocol's "ambiguous layouts stay red".

    THE FIRST ATTEMPT AT THIS MEASURED THE EXACT COMPLEMENT OF WHAT IT WANTED, and the shape
    of that mistake is worth keeping. It asked `header_symbols` which names the closure
    "declares" — but that function answers a different question (which CONSTANTS a header
    owns) and returns only macros and enum members, never a declaration. Every name it
    returned was therefore counted as a prototype. The numbers looked plausible (intl 26,
    desk 122, samlib 2) and were meaningless; the tell was samlib landing on 2, tied with
    wlanapi's 2, which would have recommended `1` for the module with the documented reason
    to choose 0. A helper whose name nearly fits is worth reading before it is trusted.
    """

    #: External functions the module calls that the include closure really declares — the
    #: prototypes bindgen would emit, i.e. what `BLOCKLIST_FUNCS=1` makes you hand-write.
    prototypes: tuple[str, ...]
    #: Called, not defined here, and declared by no header in the closure: reached through a
    #: header the closure did not walk, or an import library. samlib's SAM entrypoints are
    #: these, which is exactly why the `prototypes` count understates its risk.
    undeclared: tuple[str, ...]

    @property
    def summary(self) -> str:
        return (
            f"{len(self.prototypes)} external prototype(s) the closure declares, "
            f"{len(self.undeclared)} called but declared nowhere in it. The count does NOT "
            "settle BLOCKLIST_FUNCS (samlib chose 0 on 60; tcpsvcs kept 1 on 47) — "
            "decide it the MACRO_SIGNED way, island both ways, one cargo check each. Note "
            "the ordering constraint: BLOCKLIST_FUNCS_FROM_CRATE greps OUT_CRATE for "
            "#[no_mangle] names, so on a FIRST derive against an empty scaffold it blocklists "
            "nothing and 1 is the only safe unattended answer"
        )


def _external_prototype_survey(tus: list[Path], closure: Closure) -> PrototypeSurvey:
    """Measure the hand-writing burden `BLOCKLIST_FUNCS=1` imposes on this module."""
    try:
        from tree_sitter import Parser

        from .scanner import C_LANG
    except Exception as exc:  # pragma: no cover - env without tree-sitter
        raise ProfileDeriveError(
            f"tree-sitter is required for the BLOCKLIST_FUNCS survey: {exc}"
        ) from exc

    parser = Parser(C_LANG)
    called: set[str] = set()
    defined: set[str] = set()
    for tu in tus:
        src = tu.read_bytes()
        stack = [parser.parse(src).root_node]
        while stack:
            node = stack.pop()
            if node.type == "call_expression":
                fn = node.child_by_field_name("function")
                if fn is not None and fn.type == "identifier":
                    called.add(src[fn.start_byte : fn.end_byte].decode("utf-8", "replace"))
            elif node.type == "function_definition":
                decl = node.child_by_field_name("declarator")
                while decl is not None and decl.type != "identifier":
                    decl = decl.child_by_field_name("declarator")
                if decl is not None:
                    defined.add(src[decl.start_byte : decl.end_byte].decode("utf-8", "replace"))
            stack.extend(node.children)

    external = called - defined - _NOT_A_PROTOTYPE - _ALWAYS_MACRO
    # A name the closure spells as a MACRO or an ENUM member is not a prototype bindgen would
    # emit, whatever it looks like at the call site. Asking the headers is what keeps this from
    # being a hand-maintained exclusion list that rots — `header_symbols` returns ONLY those
    # two families (kinds: literal / expression / function-like / enum), never a declaration,
    # so its whole key set is the exclusion.
    macro_or_enum: set[str] = set()
    declared: set[str] = set()
    for header in closure.headers:
        try:
            macro_or_enum |= set(header_symbols(header, frozenset(external)))
            declared |= header_prototypes(header, frozenset(external))
        except OSError:
            continue
    prototypes = declared - macro_or_enum
    return PrototypeSurvey(
        tuple(sorted(prototypes)),
        tuple(sorted(external - prototypes - macro_or_enum)),
    )


def _in_dead_preproc_branch(node, src: bytes) -> bool:
    """True if *node* sits in a preprocessor branch this tree never compiles.

    Same fixed-truth rule as :func:`_cond_truth`, applied to the tree-sitter ancestor chain:
    ``#ifndef __REACTOS__`` (and the ``#else`` of ``#ifdef __REACTOS__``) is dead code, so its
    ``case`` labels must not be audited — that is where the false `DLL_WINE_PREATTACH` alarm
    came from.
    """
    child = node
    parent = node.parent
    while parent is not None:
        if parent.type in ("preproc_ifdef", "preproc_if"):
            head = src[parent.start_byte : parent.start_byte + 200].decode("utf-8", "replace")
            truth = _cond_truth(head.splitlines()[0] if head else "")
            if truth is not None:
                in_else = child.type in ("preproc_else", "preproc_elif")
                if truth is bool(in_else):
                    return True
        child, parent = parent, parent.parent
    return False


_DEFINE_RE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)(\(|[ \t]+)?([^\n]*)", re.M)
_ENUM_RE = re.compile(r"\benum\b[^\{;]*\{")


def enum_prefix(text: str, pos: int) -> str | None:
    """The prefix bindgen will glue onto the enum member at *pos*, or None if it cannot tell.

    bindgen never emits an enum member under its BARE name. For `typedef enum _FOO { A, B } FOO;`
    it emits `_FOO_A`; for an anonymous `typedef enum { A } FOO;` it emits `FOO_A`. Either way a
    `match` arm written with the bare name is an unresolved path — which in Rust is a fresh
    binding that matches EVERYTHING (§9.112), the same silent catch-all a missing `#define`
    produces. The tag wins over the typedef because that is bindgen's own preference.
    """
    open_brace = text.rfind("{", 0, pos)
    if open_brace < 0:
        return None
    head = text[max(0, open_brace - 200) : open_brace]
    m = re.search(r"\benum\b(?:\s+(?:class\s+)?([A-Za-z_]\w*))?\s*$", head)
    if m is None:
        return None
    if m.group(1):
        return m.group(1)
    close = text.find("}", pos)
    if close < 0:
        return None
    m2 = re.match(r"\s*([A-Za-z_]\w*)", text[close + 1 :])
    return m2.group(1) if m2 else None


def header_enum_prefix(path: Path, ident: str) -> str | None:
    """The bindgen prefix for enum member *ident* as declared in *path*.

    Looked up on demand rather than carried by :func:`header_symbols`, because it is only ever
    needed for the handful of constants that turn out to BE enum members.
    """
    text = _read(path)
    for m in re.finditer(rf"^[ \t]*{re.escape(ident)}\b[ \t]*(?:=[^,}}\n]*)?\s*[,}}]", text, re.M):
        pfx = enum_prefix(text, m.start())
        if pfx:
            return pfx
    return None


def _macro_kind(body: str) -> str:
    """What bindgen can do with a ``#define`` body.

    * ``literal``    — integer literal (possibly cast/parenthesised): emitted as a const
    * ``expression`` — references other identifiers: bindgen MAY fail to fold it
    """
    core = re.sub(r"\(\s*(?:[A-Za-z_]\w*\s*)+\s*\)", "", body)  # drop casts
    return "literal" if re.fullmatch(r"[-+~()\s0-9a-fA-FxXLlUu|&<>*/+]*", core) else "expression"


_C_COMMENT_RE = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)


def _strip_c_comments(text: str) -> str:
    """Blank C comments, keeping newlines so line-anchored patterns still line up."""
    return _C_COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


# A C++ namespace-scope integral constant: `static const int gViewsCommandID = FCIDM_X;`. It is
# a legal case label in C++ and owned by the header just as a `#define` would be, but neither
# the macro nor the enum scan sees it, so it was reported UNOWNED and blocked the module.
_CXX_CONST_RE = (
    r"^[ \t]*(?:static[ \t]+)?(?:constexpr|const)[ \t]+(?:(?:unsigned|signed)[ \t]+)?"
    r"[A-Za-z_][\w:]*[ \t]+({name})[ \t]*=[ \t]*([^;\n]+);"
)


def _cxx_consts(text: str, alt: str) -> dict[str, tuple[str, str]]:
    return {
        m.group(1): ("cxx-const", m.group(2).strip())
        for m in re.finditer(_CXX_CONST_RE.format(name=alt), text, re.M)
    }


def header_symbols(path: Path, wanted: frozenset[str] | None = None) -> dict[str, tuple[str, str]]:
    """``{name: (macro_kind, definition)}`` for the constants a header OWNS.

    ``macro_kind`` ∈ {literal, expression, function-like, enum}. Pass *wanted* to search
    only for specific identifiers — a single alternation regex over the whole text, which
    is what makes auditing a 138-header closure cheap enough to run every time.

    The alternation is anchored with ``\\b`` and ordered longest-first, because regex
    alternation is first-match-wins: without it, ``TDI_ADDRESS_TYPE_IP`` swallows the
    ``#define TDI_ADDRESS_TYPE_IPX`` line, `IPX` is reported UNOWNED, and the tool raises a
    FALSE §9.112 alarm on a constant that is defined two lines from its sibling. (Found on
    the real ``afd``/``tdihelpers`` closure, not in a unit test.)

    Comments are blanked first: setuplib's fsutil.h spells an enum member
    ``/**/ChangeSystemPartition/**/ // FIXME``, which no member pattern sees through, and the
    member was reported UNOWNED.
    """
    text = _strip_c_comments(_read(path))
    out: dict[str, tuple[str, str]] = {}
    if wanted is not None:
        if not wanted:
            return out
        alt = "|".join(re.escape(w) for w in sorted(wanted, key=lambda w: (-len(w), w)))
        for m in re.finditer(rf"^[ \t]*#[ \t]*define[ \t]+({alt})\b(\()?([^\n]*)", text, re.M):
            name, paren, body = m.group(1), m.group(2), m.group(3).strip().rstrip("\\").strip()
            out.setdefault(name, ("function-like", body) if paren else (_macro_kind(body), body))
        # enum members: one per line, `NAME[= ...]` followed by `,` or the closing `}`. The
        # separator match must cross a NEWLINE: the LAST member of an enum has no trailing
        # comma and its `}` sits on the next line (`DomainDisplayServer` in ntsam.h), which a
        # same-line-only pattern reports as UNOWNED — another false §9.112 alarm.
        for m in re.finditer(rf"^[ \t]*({alt})\b[ \t]*(=[^,}}\n]*)?\s*[,}}]", text, re.M):
            if "enum" in text:
                out.setdefault(m.group(1), ("enum", m.group(0).strip()))
        for name, kind in _cxx_consts(text, alt).items():
            out.setdefault(name, kind)
        return out
    for m in _DEFINE_RE.finditer(text):
        name, sep, body = m.group(1), m.group(2), m.group(3).strip().rstrip("\\").strip()
        out.setdefault(name, ("function-like", body) if sep == "(" else (_macro_kind(body), body))
    for m in _ENUM_RE.finditer(text):
        end = text.find("}", m.end())  # enum bodies carry no nested braces
        if end < 0:
            continue
        for member in text[m.end() : end].split(","):
            mm = re.match(r"\s*([A-Za-z_]\w*)", member)
            if mm:
                out.setdefault(mm.group(1), ("enum", member.strip()))
    for name, kind in _cxx_consts(text, r"[A-Za-z_]\w*").items():
        out.setdefault(name, kind)
    return out


def header_prototypes(path: Path, wanted: frozenset[str]) -> set[str]:
    """Which of *wanted* this header DECLARES as a function — `RET Name(args);`.

    `header_symbols` deliberately answers a different question (which constants a header
    OWNS) and returns only macros and enum members. Using it as a stand-in for "declared"
    silently measured the exact complement of what was wanted: every name it returned was
    counted as a prototype, so the BLOCKLIST_FUNCS survey was reporting the macro-shaped
    call targets and calling them the FFI burden.

    Deliberately lexical and narrow, in the same spirit: a candidate is `Name(` whose line
    does not begin a preprocessor directive (that would be a function-like macro) and whose
    balanced argument list is followed by `;`. A DEFINITION has `{` there instead, and a call
    site has neither. It will miss a declaration wrapped in a macro (`STDAPI_(x) Name(...)`
    still matches; `DECLARE_HANDLE`-style ones do not), which understates rather than
    invents — the survey reports both halves precisely so the reader is not left inferring
    from one number.
    """
    if not wanted:
        return set()
    text = _read(path)
    alt = "|".join(re.escape(w) for w in sorted(wanted, key=lambda w: (-len(w), w)))
    found: set[str] = set()
    for m in re.finditer(rf"\b({alt})\s*\(", text):
        line_start = text.rfind("\n", 0, m.start()) + 1
        if text[line_start : m.start()].lstrip().startswith("#"):
            continue  # `#define Name(a, b) ...` — a macro, not a declaration
        depth, index = 0, m.end() - 1
        while index < len(text):
            if text[index] == "(":
                depth += 1
            elif text[index] == ")":
                depth -= 1
                if depth == 0:
                    break
            index += 1
        tail = text[index + 1 : index + 200].lstrip()
        # Skip a trailing attribute/calling-convention macro before the terminator.
        tail = re.sub(r"^(?:__attribute__\s*\(\([^)]*\)\)|\w+)\s*", "", tail, count=1)
        if tail.startswith(";"):
            found.add(m.group(1))
    return found


def resolve_sys_crate(repo_root: Path, module_name: str, usermode: bool) -> str:
    """The -sys crate the TRANSLATED CODE will actually glob-import.

    `ntdll-sys`/`ntkrnl-sys` is only the default. A module with its own bindgen island —
    msvcrt, win32k, shell32 — imports THAT, and checking its constants against ntdll-sys
    says "not in the -sys crate" about every single one. Those findings are marked
    BLOCKING (§9.112), so getting the crate wrong does not merely add noise: it makes the
    gate that exists to catch silent catch-all `match` arms unable to be satisfied. msvcrt
    reported six of them with all six constants already sitting in `msvcrt-sys`.

    Resolution is by EXISTENCE on disk, in the same order rustconv's own `--target` proves
    an output crate: `<module>-sys` if there is one, else the mode default.

    `module_name` may be a CMake TARGET rather than the directory, and a target routinely
    carries a role suffix the crate does not: msvcrt's `.c` are split across `msvcrt_shared`
    and `msvcrt_static`, both of which import `msvcrt-sys`. So strip trailing `_<role>`
    segments one at a time and try each stem. The first version of this special-cased
    `_static` alone, which left `--target msvcrt_shared` resolving to `ntdll-sys` and
    re-emitting the six BLOCKING findings this function exists to prevent -- with all six
    constants already present in `msvcrt-sys` and correct. A partial fix here reads exactly
    like no fix at all, because the symptom is identical.
    """
    stem = Path(module_name).name
    stems = [stem]
    while "_" in stems[-1]:
        stems.append(stems[-1].rsplit("_", 1)[0])
    for cand in (f"{st}-sys" for st in stems):
        if (repo_root / "sdk/lib/rust" / cand / "src").is_dir():
            return cand
    return "ntdll-sys" if usermode else "ntkrnl-sys"


def sys_crate_consts(
    repo_root: Path, usermode: bool, arch: str = "i386", crate: str | None = None
) -> dict[str, str]:
    """``{const name: providing file stem}`` for the sibling -sys crate's exports.

    ``ntdll-sys`` for usermode, ``ntkrnl-sys`` for kernel — the same choice
    ``gen_driver_types.sh`` makes for its blocklist source. EVERY bindgen island in the
    crate is scanned, not just the primary one, and the provider is recorded: a const that
    lives in a SECONDARY island (``hal_bindgen_<arch>.rs``) is reachable, but only through
    that island's module path (``ntkrnl_sys::hal_ddk``) — not through the ``ddk`` glob the
    generated island re-exports. Reporting the difference is what separates "already
    covered" from "covered only if you add the right import".
    """
    crate = crate or ("ntdll-sys" if usermode else "ntkrnl-sys")
    base = repo_root / "sdk/lib/rust" / crate / "src"
    primary = {f"bindgen_{arch}", "bindgen_common"}
    out: dict[str, str] = {}
    if not base.is_dir():
        return out
    other_arches = tuple(f"_{a}" for a in ARCHES if a != arch)
    files = sorted(base.glob("*bindgen*.rs"), key=lambda p: (p.stem not in primary, p.stem))
    for f in files:
        if f.stem.endswith(other_arches):
            continue  # another arch's island
        text = _read(f)
        for name in re.findall(r"pub const (\w+)", text):
            out.setdefault(name, f.stem)
        # bindgen spells a C enumerator `<enum type>_<member>` (`_DEVICE_USAGE_NOTIFICATION_
        # TYPE_DeviceUsageTypePaging`), so the bare member the C switches on never appears
        # as a `pub const` of its own. It IS carried -- `port resolve` aliases the bare
        # name to the prefixed one -- and reporting it as missing made every PnP driver's
        # DeviceUsageType*/DeviceText* cases a blocking error.
        for _ty, member in re.findall(r"pub const (\w+?)_(\w+):\s*\1\s*=", text):
            out.setdefault(member, f.stem)
    return out


def prelude_provided_names(prelude: Path | None, text: str | None = None) -> set[str]:
    """Names a crate prelude defines (`pub const X`) or re-exports by name
    (`pub use a::b::X;`, `pub use a::{X, Y as Z};` -- the alias is what callers see)."""
    if text is None:
        if prelude is None or not prelude.is_file():
            return set()
        text = _read(prelude)
    names = set(re.findall(r"^\s*pub const (\w+)", text, re.M))
    for body in re.findall(r"^\s*pub use [\w:]+::\{([^}]*)\}\s*;", text, re.M):
        for item in body.split(","):
            words = item.split()
            if words:
                names.add(words[-1])
    names.update(re.findall(r"^\s*pub use [\w:]+::(\w+)\s*;", text, re.M))
    names.update(re.findall(r"^\s*pub use [\w:]+ as (\w+)\s*;", text, re.M))
    return names


def own_constants(
    consts: list[SwitchConst],
    closure: Closure,
    tus: list[Path],
    sys_consts: dict[str, str],
) -> list[ConstOwner]:
    """Attribute each switch constant to the header (or TU) that DEFINES it.

    Search order is (depth, name) over the closure, so the nearest header wins — the same
    "first match on the include path" intuition a human applies when reading precomp.h.
    """
    wanted = frozenset(c.ident for c in consts)
    tu_local: dict[str, tuple[str, str]] = {}
    for tu in tus:
        tu_local.update(header_symbols(tu, wanted))
    ordered = sorted(closure.headers.items(), key=lambda kv: (kv[1].depth, kv[1].name))
    per_header: dict[Path, dict[str, tuple[str, str]]] = {}
    remaining = set(wanted)
    for path, _ref in ordered:
        if not remaining:
            break
        syms = header_symbols(path, frozenset(remaining))
        if syms:
            per_header[path] = syms
            remaining -= set(syms)
    owners: list[ConstOwner] = []
    for ident in sorted(wanted):
        src = sys_consts.get(ident)
        in_sys = src is not None
        hit: tuple[Path, tuple[str, str]] | None = None
        for path, syms in per_header.items():
            if ident in syms:
                hit = (path, syms[ident])
                break
        if hit is not None:
            # bindgen reads headers as C, so a C++ const never reaches the island: the header
            # owns it, but the translation has to carry it (see SWITCH_CONST_CXX).
            kind = "cxx-const" if hit[1][0] == "cxx-const" else "header"
            owners.append(
                ConstOwner(ident, hit[0], kind, hit[1][1], hit[1][0], in_sys=in_sys, sys_source=src)
            )
        elif ident in tu_local:
            kind, body = tu_local[ident]
            owners.append(
                ConstOwner(ident, None, "tu-local", body, kind, in_sys=in_sys, sys_source=src)
            )
        elif in_sys:
            owners.append(
                ConstOwner(ident, None, "sys-crate", None, "n/a", in_sys=True, sys_source=src)
            )
        else:
            owners.append(ConstOwner(ident, None, "UNOWNED", None, "n/a"))
    return owners


# ---------------------------------------------------------------------------
# Genuine header conflicts -> TYPES_STEM split (§9.108)
# ---------------------------------------------------------------------------

_TYPEDEF_STRUCT_RE = re.compile(r"typedef\s+(?:struct|union|enum)\s+(\w+)?\s*\{")
_STRUCT_DEF_RE = re.compile(r"^\s*(?:typedef\s+)?(?:struct|union)\s+(\w+)\s*\{", re.M)
_TYPEDEF_NAME_RE = re.compile(r"\}\s*([A-Za-z_]\w*)\s*(?:,\s*\*?\w+\s*)*;")
# `} DUMMYUNIONNAME;` is the Windows ANONYMOUS-aggregate macro closing an inline union FIELD,
# not a type definition. Counting it as one made every header carrying an anonymous union
# "conflict" with every other, and proposed a bogus 3-island TYPES_STEM split for wldap32.
_ANON_AGGREGATE_NAMES = frozenset(
    f"DUMMY{kind}NAME{suffix}"
    for kind in ("UNION", "STRUCT")
    for suffix in ("", *(str(n) for n in range(1, 10)))
)


def header_type_defs(path: Path) -> set[str]:
    """Names a header DEFINES a body for (struct/union/enum + their typedef names)."""
    text = _read(path)
    names: set[str] = set()
    for m in _TYPEDEF_STRUCT_RE.finditer(text):
        if m.group(1):
            names.add(m.group(1))
    names.update(m.group(1) for m in _STRUCT_DEF_RE.finditer(text) if m.group(1))
    names.update(m.group(1) for m in _TYPEDEF_NAME_RE.finditer(text))
    return names - _ANON_AGGREGATE_NAMES


def find_conflicts(closure: Closure, headers: list[Path]) -> list[tuple[str, Path, Path]]:
    """Type names defined by two headers that NO SINGLE TU includes together.

    Co-reachability is the false-positive filter and the whole §9.108 insight: if one
    ``.c`` includes both headers and compiles, the pair is compatible by construction
    (guarded re-definitions, ``#ifndef X_DEFINED``, …) and is not a conflict.
    """
    defs = {p: header_type_defs(p) for p in headers}
    owner: dict[str, list[Path]] = {}
    for p, names in defs.items():
        for n in names:
            owner.setdefault(n, []).append(p)
    conflicts: list[tuple[str, Path, Path]] = []
    for name, paths in sorted(owner.items()):
        if len(paths) < 2:
            continue
        for i, a in enumerate(paths):
            for b in paths[i + 1 :]:
                if any(a in s and b in s for s in closure.per_tu.values()):
                    continue  # co-reachable in a real TU -> proven compatible
                conflicts.append((name, a, b))
    return conflicts


def group_tus(
    closure: Closure, conflicts: list[tuple[str, Path, Path]]
) -> list[tuple[str, list[str]]]:
    """Greedily group TUs into conflict-free islands; one group = one TYPES_STEM."""
    bad_pairs = {(a, b) for _, a, b in conflicts} | {(b, a) for _, a, b in conflicts}
    groups: list[list[str]] = []
    group_headers: list[set[Path]] = []
    for tu in sorted(closure.per_tu):
        hs = closure.per_tu[tu]
        placed = False
        for gi, gh in enumerate(group_headers):
            if any((a, b) in bad_pairs for a in gh for b in hs):
                continue
            gh.update(hs)
            groups[gi].append(tu)
            placed = True
            break
        if not placed:
            groups.append([tu])
            group_headers.append(set(hs))
    stems = ["types"] + [f"types_{i}" for i in range(1, len(groups))]
    return list(zip(stems, groups, strict=True))


# ---------------------------------------------------------------------------
# compile_commands.json — the strongest available oracle when a build dir exists
# ---------------------------------------------------------------------------


@dataclass
class CompileDb:
    """What the REAL compile command says, for the module and for the whole tree."""

    path: Path
    canon_root: str  # the canonical build prefix baked into the db (e.g. /tmp/vibeos)
    module_defines: list[str]  # module-specific -D (tree-wide baseline subtracted)
    module_files: set[Path]
    include_dirs: list[Path]


def _canon_prefix(sample: str, repo_root: Path) -> str | None:
    """The canonical build prefix baked into a db path (``/tmp/vibeos``), or ``None``.

    Found by walking the path's suffixes until one exists under *repo_root* — module
    independent, so it also works for a module whose own sources are all commented out.
    """
    parts = Path(sample).parts
    # i >= 2: a 1-component prefix would be "/" and remapping that would rewrite every path.
    for i in range(2, len(parts)):
        if (repo_root / Path(*parts[i:])).exists():
            return str(Path(*parts[:i]))
    return None


def load_compile_db(repo_root: Path, module: CMakeModule, arch: str) -> CompileDb | None:
    """Read ``build-<arch>/compile_commands.json``, or ``None`` when there is none.

    Two things only the real command knows:

    * the module's OWN ``-D`` set. It is isolated by INTERSECTING the define sets of
      every entry in the db — that intersection is exactly the repo-wide
      ``add_definitions()`` baseline (which ``gen_driver_types.sh`` supplies itself), so
      subtracting it leaves precisely the module-specific macros (``NDIS50_MINIPORT``,
      ``_WIN32_WINNT=0x600``, …). Self-calibrating, no denylist to rot.
    * the module's extra ``-I`` set.

    Build dirs bake the CANONICAL path (``/tmp/vibeos`` — the per-build bind mount), so
    that prefix is remapped onto *repo_root* before any path is used.
    """
    db_path = repo_root / f"build-{arch}" / "compile_commands.json"
    if not db_path.is_file():
        return None
    data = json.loads(_read(db_path))
    if not data:
        return None
    canon: str | None = None
    for e in data[:64]:  # a few samples: an entry may name a file this tree no longer has
        canon = _canon_prefix(str(e.get("file", "")), repo_root)
        if canon is not None:
            break
    if canon is None:
        return None
    # Entries belong to this module if the OBJECT they produce lands in its
    # `CMakeFiles/<target>.dir/`. Matching the source path instead is wrong twice over:
    # `afd` compiles ../tdihelpers/tdi.c, whose entry mentions "afd" nowhere (so a
    # directory-substring match misses the module's real flags), AND that same tdi.c is
    # compiled by `netio` too — with `-D_WIN32_WINNT=0x600` against afd's 0x502. Picking the
    # wrong one silently changes every version-gated struct layout in the island.
    want = {p.resolve() for p in module.compiled_tus}
    obj_marker = f"CMakeFiles/{module.name}.dir/"
    baseline: set[str] | None = None
    mine: set[str] = set()
    files: set[Path] = set()
    incs: list[Path] = []
    for e in data:
        f = str(e.get("file", ""))
        # C/C++ only. The .rc entries are llvm-rc/windres invocations carrying ~150
        # -DLANGUAGE_* + -DRC_INVOKED, which say nothing about the C type surface.
        if not f.lower().endswith((".c", ".cpp", ".cc", ".cxx")):
            continue
        cmd = str(e.get("command", ""))
        defines = {t for t in cmd.split() if t.startswith("-D")}
        baseline = defines if baseline is None else (baseline & defines)
        local = Path(f.replace(canon, str(repo_root)))
        objs = re.findall(r"-o\s+(\S+)", cmd)
        if objs:
            if not any(obj_marker in o for o in objs):
                continue
        elif local.resolve() not in want and not _is_within(local, module.module_dir):
            continue  # no -o to attribute by: fall back to the source location
        files.add(local)
        mine |= defines
        # `-isystem` is how CMake spells an INTERFACE library's include dirs, so reading only
        # `-I` lost every one of them: browseui's `atl_classes` (sdk/lib/atl) went missing and
        # derive could not find atlstr.h. Both the joined and the separated spelling occur.
        toks = cmd.split()
        for i, t in enumerate(toks):
            if t.startswith("-I"):
                raw = t[2:] or (toks[i + 1] if i + 1 < len(toks) else "")
            elif t.startswith("-isystem"):
                raw = t[len("-isystem") :] or (toks[i + 1] if i + 1 < len(toks) else "")
            else:
                continue
            if not raw or raw.startswith("-"):
                continue
            p = Path(raw.replace(canon, str(repo_root)))
            if p not in incs:
                incs.append(p)
    if not files:
        return None
    return CompileDb(
        path=db_path,
        canon_root=canon,
        module_defines=sorted(mine - (baseline or set())),
        module_files=files,
        include_dirs=incs,
    )


# ---------------------------------------------------------------------------
# Derivation
# ---------------------------------------------------------------------------


_VERSION_MACROS = ("WINVER", "_WIN32_WINNT", "NTDDI_VERSION", "_WIN32_WINDOWS", "_WIN32_IE")
# NTDDI for a given _WIN32_WINNT. 0x502 maps to WS03SP1 (not 0x05020000) deliberately: the
# amd64 peb_teb.h ACTIVATION_CONTEXT_STACK layout needs SP1 — see the pcix/partmgr profiles.
_NTDDI_FOR = {
    "0x502": "0x05020400",
    "0x600": "0x06000000",
    "0x601": "0x06010000",
    "0x602": "0x06020000",
    "0x603": "0x06030000",
}


def _default_ver_defs(module: CMakeModule, db_defines: list[str] | None) -> list[str]:
    """VER_DEFS mirroring the module's real compile, plus the bindgen-side extras.

    Sources in increasing authority: the repo-wide baseline (root CMakeLists) → the
    module's own ``add_definitions()`` → the module-specific ``-D`` isolated from
    ``compile_commands.json``. ``remove_definitions`` needs no special case: a module that
    removes then re-adds a macro (``snmpapi``: ``_WIN32_WINNT`` 0x502 → 0x600) is handled
    by last-write-wins. ``gen_driver_types.sh`` supplies the arch defines,
    ``__REACTOS__``, ``DBG`` and ``_USE_DUMMY_PSEH`` itself, so they stay out of here.
    """
    ver = {
        "WINVER": "0x502",
        "_WIN32_WINNT": "0x502",
        "_WIN32_WINDOWS": "0x502",
        "_WIN32_IE": "0x603",
    }
    extra: list[str] = []
    set_by_module: set[str] = set()
    for source in (module.definitions, db_defines or []):
        for d in source:
            name, _, val = d[2:].partition("=")
            if name in _VERSION_MACROS:
                if val:
                    ver[name] = val
                    set_by_module.add(name)
            elif d not in extra:
                extra.append(d)
    if "_WIN32_WINNT" in set_by_module and "WINVER" not in set_by_module:
        ver["WINVER"] = ver["_WIN32_WINNT"]  # WINVER tracks _WIN32_WINNT unless set apart
    ver.setdefault("NTDDI_VERSION", _NTDDI_FOR.get(ver["_WIN32_WINNT"], "0x05020400"))
    defs = [f"-D{name}={ver[name]}" for name in _VERSION_MACROS if name in ver]
    defs += extra
    if module.unicode:
        defs += ["-DUNICODE", "-D_UNICODE"]
    defs.append("-D_CRT_NO_POSIX_ERROR_CODES")  # bindgen-side: errno.h vs winerror.h
    return defs


def derive_source(
    source: Path,
    repo_root: Path,
    *,
    arch: str,
    preferred_target: str | None = None,
    usermode: bool | None = None,
) -> DerivedProfile:
    """Audit the CMake source owner, including retained sources after a Rust flip.

    A nearby DLL target need not own the TU (msvcrt_static does), and auditing
    only the remaining C after a partial flip silently excludes the requested file.
    """
    source = source.resolve()
    for directory in source.parents:
        cmake = directory / "CMakeLists.txt"
        if not cmake.is_file():
            continue
        owners = []
        for name in sorted(_target_names(cmake)):
            for flipped in (False, True):
                try:
                    module = parse_cmakelists(
                        cmake, repo_root, arch=arch, target=name, flipped_sources=flipped
                    )
                except ProfileDeriveError:
                    continue
                if source in {p.resolve() for p in module.compiled_tus}:
                    owners.append((name, flipped))
                    break
        preferred = [item for item in owners if item[0] == preferred_target]
        if preferred:
            owners = preferred
        if len(owners) > 1:
            raise ProfileDeriveError(f"{source}: ambiguous CMake owners: {owners}")
        if owners:
            name, flipped = owners[0]
            # The cargo-linked kernel is ~240 flipped TUs whose island is the whole of
            # ntkrnl-sys, not a module allowlist. Auditing all of them for one file costs
            # minutes and fails on TUs that no longer compile at all (mm/marea.c includes the
            # deleted cache/ headers), so a kernel file is audited as itself.
            scoped = (source,) if name in _cargo_kernel_targets(cmake) else ()
            return derive(
                directory,
                repo_root,
                arch=arch,
                target=name,
                usermode=usermode,
                flipped_sources=flipped,
                sources=scoped,
            )
        if directory == repo_root.resolve():
            break
    raise ProfileDeriveError(
        f"{source}: no proven CMake source owner, including retained flipped sources"
    )


def derive(
    module_dir: Path,
    repo_root: Path,
    arch: str = "i386",
    target: str | None = None,
    strict_unresolved: bool = True,
    use_compile_db: bool = True,
    extra_inc_dirs: tuple[Path, ...] = (),
    usermode: bool | None = None,
    flipped_sources: bool = False,
    sources: tuple[Path, ...] = (),
) -> DerivedProfile:
    """Derive a full profile for the module at *module_dir*.

    Raises :class:`ProfileDeriveError` when an UNCONDITIONAL include cannot be resolved
    (unless *strict_unresolved* is false) — a partial allowlist that looks complete is the
    §9.112 bug, so it is never emitted silently. *extra_inc_dirs* is the escape hatch for a
    module that includes from an unusual directory (``iphlpapi`` reaches into
    ``dll/win32/dhcpcsvc/include``) when there is no compile db to learn it from.
    """
    module_dir = module_dir.resolve()
    module = parse_cmakelists(
        module_dir / "CMakeLists.txt",
        repo_root,
        arch=arch,
        target=target,
        flipped_sources=flipped_sources,
    )
    module.usermode_override = usermode
    full_tus = set(module.compiled_tus)
    if sources:
        selected = {(p if p.is_absolute() else repo_root / p).resolve() for p in sources}
        unknown = selected - {p.resolve() for p in full_tus}
        if unknown:
            raise ProfileDeriveError(
                "Selected sources are not proven members of this CMake target: "
                + ", ".join(str(p) for p in sorted(unknown))
            )
        missing = [p for p in selected if not p.is_file()]
        if missing:
            raise ProfileDeriveError(f"Selected source is missing: {missing}")
        module.entries = [e for e in module.entries if e.kind != "tu" or e.path in selected]
        module.findings.append(
            Finding(
                "warn",
                "PARTIAL_TU_SCOPE",
                f"Explicit subset: {len(selected)}/{len(full_tus)} translation units; "
                "this profile does not certify the other source files",
            )
        )
    findings = list(module.findings)
    tus = [p for p in module.compiled_tus if p.is_file()]
    if not tus:
        # "No .c" has TWO causes and they need opposite responses. Before blaming a
        # Rust flip, check whether a SIBLING target in the same CMakeLists compiles the
        # C: a module whose sources live in a static lib the DLL links (msvcrt ->
        # msvcrt_static) looks identical from the MODULE target, and telling that user
        # to go read git history sends them the wrong way entirely. This also describes
        # every whole-archive cohab module, where the .c stay compiled forever and the
        # Rust wins at link — so "its .c are commented out" is not even true of them.
        siblings = _targets_compiling_c(module.cmake, repo_root, arch, exclude=module.name)
        if siblings:
            listed = ", ".join(f"{n} ({c} .c)" for n, c in siblings)
            raise ProfileDeriveError(
                f"{module.cmake}: target {module.name!r} compiles no in-tree .c, but these "
                f"targets in the same file do: {listed}.\n  The sources are probably in a "
                f"static library this target links. Re-run with:\n\n"
                f"    --target {siblings[0][0]}\n"
            )
        raise ProfileDeriveError(
            f"{module.cmake}: target {module.name!r} compiles no in-tree .c — nothing to derive "
            "an island for. (An already-Rust-flipped module looks exactly like this: its .c are "
            "commented out of the source list. Re-run with --flipped-sources to audit against "
            "the list the flip commented out, or use git to read the pre-flip CMakeLists.)"
        )

    # THE SAME CONDITION, one step less obvious, and it is the one that goes quiet. A cohab flip
    # comments out the real .c but LEAVES the `rust_*.c` scaffolding compiled, so `tus` is not
    # empty — it is one shim with no switch statements, no enum members and no dead bodies in it.
    # Every audit then reports nothing and every empty section reads as a pass. sysdm printed
    # "(no case labels with identifiers)" that way while carrying the exact defect the case-label
    # audit exists to catch: `LVN_BEGINLABELEDIT` is `__MINGW_NAME_AW(...)`, a function-like
    # macro, so bindgen emits no constant and the match arm is a catch-all binding
    # (SWITCH_CONST_NOT_BINDGENABLE, severity ERROR). Run against the live list it would have
    # named that before a single paid call. A vacuous clean bill is worse than no report:
    # AGENTS.md, "fail loud, never silently degrade".
    if all(is_shim_tu(tu) for tu in tus):
        shims = ", ".join(sorted(tu.name for tu in tus))
        raise ProfileDeriveError(
            f"{module.cmake}: target {module.name!r} compiles ONLY cohab scaffolding ({shims}) — "
            "this module is already flipped, and every audit below would report an empty "
            "section that reads as a pass.\n  To re-audit the C the Rust replaced:\n\n"
            "    --flipped-sources\n"
        )

    # The compile db is loaded FIRST so its real `-I` list can seed the search path: it is the
    # only place that knows a module reaching outside its own directory (afd includes
    # ../tdihelpers/include). Without a db, that same case fails loudly and names the header —
    # then --extra-inc is the explicit fix, chosen by a human rather than guessed here.
    db = load_compile_db(repo_root, module, arch) if use_compile_db else None
    env = build_include_env(
        repo_root,
        module,
        arch=arch,
        extra_dirs=tuple(extra_inc_dirs),
        compile_dirs=tuple(db.include_dirs if db else ()),
    )
    if env.build_dir is None:
        findings.append(
            Finding(
                "warn",
                "NO_BUILD_DIR",
                f"build-{arch}/ is absent: GENERATED headers (MIDL *_c.h/*_s.h, xdk, mc) cannot "
                "be resolved; the closure and the allowlist are incomplete by that much",
            )
        )
    closure = walk_includes(tus, env)

    # An UNCONDITIONAL unresolved include means our search path is wrong, so the closure —
    # and therefore the allowlist — is genuinely incomplete: hard-fail. A CONDITIONAL one is
    # nearly always a dead platform arm the real preprocessor never takes (rpcmac.h,
    # intrin_mips.h), and a CRT/toolchain header lives outside the tree by design: those are
    # reported as findings, never silently dropped.
    hard_unresolved = [
        u for u in closure.unresolved if u.name not in CRT_HEADERS and not u.conditional
    ]
    for u in closure.unresolved:
        findings.append(
            Finding(
                "error" if u in hard_unresolved else ("warn" if not u.conditional else "info"),
                "UNRESOLVED_INCLUDE",
                f"{u.includer}:{u.line} #include {u.name} — {u.reason}",
            )
        )
    if hard_unresolved and strict_unresolved:
        listing = "\n".join(
            f"    {u.includer}:{u.line}  {u.name}  ({u.reason})" for u in hard_unresolved[:20]
        )
        raise ProfileDeriveError(
            f"{len(hard_unresolved)} unconditional include(s) could not be resolved for "
            f"{module.name}; the include closure — and therefore the allowlist — would be "
            f"incomplete:\n{listing}\n"
            "  Fix the search path (build_include_env / --extra-inc) or pass "
            "--allow-unresolved to proceed with these recorded as findings."
        )

    # ---- switch-constant audit (§9.112) ----
    switch_consts = find_switch_constants(tus, repo_root)
    sys_crate = getattr(module, "sys_crate", None) or resolve_sys_crate(
        repo_root, module.name, module.is_usermode
    )
    sys_consts = sys_crate_consts(repo_root, module.is_usermode, arch, sys_crate)
    # A name the output crate's prelude already provides is resolved for every module of
    # the crate, which is exactly the fix SWITCH_CONST_IN_BASE_HEADER asks for -- so a
    # re-derive after transcribing must stop reporting it.
    for name in prelude_provided_names(
        repo_root / "sdk/lib/rust" / f"{module.crate_stem}-rust" / "src" / "prelude.rs"
    ):
        sys_consts[name] = "prelude"  # overrides a secondary-island hit: the prelude fixed it
    owners = own_constants(switch_consts, closure, tus, sys_consts)

    # ---- BLOCKLIST_FUNCS evidence (never a decision) ----
    survey = _external_prototype_survey(tus, closure) if module.is_usermode else None
    if survey is not None:
        findings.append(Finding("info", "BLOCKLIST_FUNCS_EVIDENCE", survey.summary))

    # ---- allowlist decisions ----
    # Tiering mirrors how a human reads precomp.h: the module's OWN headers, its generated
    # MIDL headers, and the DOMAIN headers those name directly. It deliberately does NOT
    # follow an SDK header's own fan-out — `<winbase.h>` alone reaches the whole Win32
    # platform (synchapi/processthreadsapi/ioapiset/…), which the -sys crate already owns.
    # Over-inclusion here is cheap (the type blocklist keeps one identity for anything the
    # -sys crate has); UNDER-inclusion of a constant is the silent §9.112 bug, so const
    # owners are force-added below regardless of tier.
    allow_reasons: dict[str, str] = {}
    # which TUs actually reach each allowlisted header — the per-island allowlist filter
    allow_tus: dict[str, set[str]] = {}
    excluded: dict[str, str] = {}
    module_regex = _hdr_regex_dir(module.module_dir.name)
    allow_reasons[module_regex] = f"module-local headers under {module.module_dir.name}/"
    allow_tus[module_regex] = {_rel(p, repo_root) for p in tus}
    module_code = {_rel(p, repo_root) for p in tus}
    module_code |= {
        _rel(p, repo_root) for p, r in closure.headers.items() if r.kind in ("module", "generated")
    }
    for ref in sorted(closure.headers.values(), key=lambda r: r.name):
        rx = _hdr_regex(ref.name)
        base = Path(ref.name).name
        if ref.kind == "module":
            continue  # covered by the directory regex
        if ref.kind in ("crt", "toolchain"):
            excluded[ref.name] = "CRT/toolchain header"
            continue
        if base in BASE_PLATFORM_HEADERS or ref.name.startswith(("ndk/", "wine/")):
            excluded[ref.name] = "base platform / debug channel — supplied by the -sys crate"
            continue
        if ref.kind == "generated":
            allow_reasons[rx] = (
                f"the module's OWN generated header (MIDL/spec2def), via {ref.includers[0]}"
            )
            allow_tus.setdefault(rx, set()).update(ref.tus)
            continue
        named_by_module = sorted(set(ref.includers) & module_code)
        if named_by_module:
            allow_reasons[rx] = (
                f"domain header named directly by the module's own code ({named_by_module[0]}), "
                f"depth {ref.depth}"
            )
            allow_tus.setdefault(rx, set()).update(ref.tus)
        else:
            excluded[ref.name] = (
                f"depth {ref.depth}, reached only through SDK headers ({ref.includers[0]}) — "
                "promote if the island needs it"
            )

    # Force-add const owners: this is the §9.112 prophylactic, and it OVERRIDES the tier
    # heuristic — an omitted const is a silent catch-all, not a compile error.
    for o in owners:
        if o.owner_kind != "header" or o.owner is None:
            continue
        ref = closure.headers[o.owner]
        rx = _hdr_regex(ref.name)
        if ref.kind == "module":
            o.allowlisted = True
            continue
        if o.in_sys:
            # already a const in ntdll-sys/ntkrnl-sys: it resolves through the crate glob,
            # so its (often huge) owning header does NOT need to be allowlisted.
            o.allowlisted = True
            if o.sys_source not in (f"bindgen_{arch}", "bindgen_common", "prelude"):
                findings.append(
                    Finding(
                        "warn",
                        "SWITCH_CONST_SECONDARY_ISLAND",
                        f"`{o.ident}` ({ref.name}) exists in the -sys crate only in "
                        f"{o.sys_source}.rs, NOT in the primary bindgen the generated island "
                        "re-exports — the translation must import it from that island's module "
                        "path explicitly (e.g. ntkrnl_sys::hal_ddk). Without that import it is "
                        "still an unresolved name, i.e. still a silent catch-all (§9.112).",
                    )
                )
            continue
        if Path(ref.name).name in BASE_PLATFORM_HEADERS or ref.kind == "crt":
            # Allowlisting a BASE header would re-emit the whole platform locally. The two
            # sane fixes are hand-transcribing the constant into the prelude, or teaching the
            # shared -sys bindgen about it. Say so instead of proposing a monster island.
            findings.append(
                Finding(
                    "error",
                    "SWITCH_CONST_IN_BASE_HEADER",
                    f"`{o.ident}` is owned by the BASE header {ref.name} and is NOT in the -sys "
                    f"crate ({sys_crate}), so the "
                    "island will not carry it and the match arm becomes a silent catch-all "
                    "(§9.112). Fix by transcribing it into the crate prelude, or by adding its "
                    f"owning enum/#define to the -sys bindgen — NOT by allowlisting {ref.name}, "
                    "which would re-emit the whole platform.",
                )
            )
            continue
        if rx not in allow_reasons:
            allow_reasons[rx] = (
                f"OWNS switch constant {o.ident} (§9.112: an omitted const becomes a "
                f"catch-all match arm) — depth {ref.depth}"
            )
            excluded.pop(ref.name, None)
            allow_tus.setdefault(rx, set()).update(ref.tus)
            findings.append(
                Finding(
                    "warn",
                    "SWITCH_CONST_HEADER_ADDED",
                    f"{ref.name} allowlisted because it owns `{o.ident}` "
                    f"(used in a case label) — depth {ref.depth}, would have been excluded",
                )
            )
        o.allowlisted = True
        if o.macro_kind == "function-like":
            # A warning, not a blocking error: the case label is `NAME(args)`, and a
            # call-shaped pattern cannot bind everything the way an unresolved bare name
            # does -- rustc rejects it ("expected tuple struct or tuple variant, found
            # function"), so the failure is loud by construction. pciidex's
            # `case PIO_MODE(0):` blocked the whole module on a hazard it cannot have.
            findings.append(
                Finding(
                    "warn",
                    "SWITCH_CONST_NOT_BINDGENABLE",
                    f"`{o.ident}` ({ref.name}) is a FUNCTION-LIKE macro: bindgen emits no "
                    "constant for it — fold the case label to its value or port the macro "
                    "(`macro_rules!` is legal in pattern position); rustc rejects the "
                    "call-shaped pattern until you do",
                )
            )
        elif o.macro_kind == "expression":
            findings.append(
                Finding(
                    "info",
                    "SWITCH_CONST_EXPRESSION",
                    f"`{o.ident}` ({ref.name}) = {o.definition!r} is a macro EXPRESSION; "
                    "bindgen may not fold it — verify it lands in the island",
                )
            )
        elif o.macro_kind == "enum":
            # Allowlisting the header is NOT enough here, which is the whole trap: the constant
            # IS in the island, under a name the translation never uses. The bare name stays
            # unresolved, and an unresolved name in a match arm binds everything (§9.112).
            o.enum_prefix = header_enum_prefix(ref.path, o.ident)
            if o.enum_prefix:
                findings.append(
                    Finding(
                        "warn",
                        "SWITCH_CONST_ENUM_PREFIXED",
                        f"`{o.ident}` ({ref.name}) is an ENUM MEMBER: bindgen emits it as "
                        f"`{o.enum_prefix}_{o.ident}`, never the bare name. Alias it in the "
                        f"prelude — pub use crate::types::{o.enum_prefix}_{o.ident} as "
                        f"{o.ident}; — and do NOT re-declare it with a hand-typed value.",
                    )
                )
            else:
                findings.append(
                    Finding(
                        "warn",
                        "SWITCH_CONST_ENUM_PREFIXED",
                        f"`{o.ident}` ({ref.name}) is an ENUM MEMBER, so bindgen emits it "
                        "PREFIXED with its enum's tag or typedef, never the bare name. The tag "
                        "could not be determined here — grep the island for the suffix and "
                        "alias the result in the prelude.",
                    )
                )
    for o in owners:
        if o.owner_kind == "UNOWNED":
            findings.append(
                Finding(
                    "error",
                    "SWITCH_CONST_UNOWNED",
                    f"`{o.ident}` is used in a case label but NO header in the include closure "
                    "and no -sys crate defines it — the match arm WILL silently match "
                    "everything (§9.112). Find its owner and add it to the wrapper.",
                )
            )
        elif o.owner_kind == "cxx-const" and not o.in_sys:
            findings.append(
                Finding(
                    "warn",
                    "SWITCH_CONST_CXX",
                    f"`{o.ident}` is a C++ const in {o.owner.name if o.owner else '?'} "
                    f"(= {o.definition}) — bindgen reads headers as C and never emits it, so "
                    "the translation must carry it as a Rust const with that value",
                )
            )
        elif o.owner_kind == "tu-local":
            findings.append(
                Finding(
                    "info",
                    "SWITCH_CONST_TU_LOCAL",
                    f"`{o.ident}` is defined in the .c itself ({o.macro_kind}) — the translation "
                    "must carry it as a Rust const, no island change needed",
                )
            )

    # ---- wrapper: mirror each TU's real prologue ----
    known_undefined = _wine_config_undefs(repo_root)
    wrapper: list[str] = []
    for tu in tus:
        # Defined by EVERY compile of this TU: the global ReactOS definition, the language,
        # and the module's character set (set_module_type ... UNICODE).
        known_defined = frozenset(
            {"__REACTOS__"}
            | ({"__cplusplus"} if str(tu).lower().endswith(CPP_SUFFIXES) else set())
            | ({"UNICODE", "_UNICODE"} if module.unicode else set())
        )
        lines, conditional = scan_preamble(_read(tu), known_undefined, known_defined)
        if conditional:
            findings.append(
                Finding(
                    "info",
                    "CONDITIONAL_PROLOGUE",
                    f"{_rel(tu, repo_root)} has #if/#ifdef in its include prologue — the mirrored "
                    "wrapper keeps both arms; review",
                )
            )
        for line in lines:
            if line not in wrapper:
                wrapper.append(line)
    if module.pch_header is not None and any(t not in module.pch_skip for t in tus):
        pch_line = f'#include "{module.pch_header.name}"'
        if pch_line not in wrapper:
            wrapper.insert(0, pch_line)

    # ---- islands / conflicts ----
    conflicts = find_conflicts(closure, sorted(closure.headers))
    if any(str(t).lower().endswith(CPP_SUFFIXES) for t in tus):
        # A C++ module takes ATL from atl-rust (atl_contract), not from its island: bindgen's
        # rendering of ATL's own classes (CAtlStringMgr, CStringT) does not even compile
        # (`derive(Copy)` on a class with a non-Copy member, E0204).
        for rx in [rx for rx in allow_reasons if re.search(r"atl\w*\\\.h", rx)]:
            del allow_reasons[rx]
    allow_files = sorted(allow_reasons)
    tu_by_key = {_rel(p, repo_root): p for p in tus}

    def _wrapper_for(keys: list[str]) -> list[str]:
        """Union of just THESE TUs' prologues, in first-seen order."""
        lines: list[str] = []
        for key in keys:
            src = tu_by_key.get(key)
            if src is None:
                continue
            for line in scan_preamble(_read(src), known_undefined)[0]:
                if line not in lines:
                    lines.append(line)
        # Only if a TU in THIS island actually compiles with the PCH. wldap32 splits
        # exactly along that line: ber.c is the PCH-skipped TU and it is also the one
        # whose headers conflict with precomp.h's.
        srcs = [tu_by_key[k] for k in keys if k in tu_by_key]
        if module.pch_header is not None and any(s not in module.pch_skip for s in srcs):
            pch_line = f'#include "{module.pch_header.name}"'
            if pch_line not in lines:
                lines.insert(0, pch_line)
        return lines

    if conflicts:
        groups = group_tus(closure, conflicts)
        islands = [
            Island(
                stem=stem,
                tus=group,
                allow_files=[
                    rx
                    for rx in allow_files
                    if allow_tus.get(rx, set()) & set(group) or rx == module_regex
                ],
                reason="TU group with a mutually-legal include set",
                wrapper_inc=_wrapper_for(group),
                label=module.crate_stem if i == 0 else f"{module.crate_stem}-{stem}",
            )
            for i, (stem, group) in enumerate(groups)
        ]
        for name, a, b in conflicts[:10]:
            findings.append(
                Finding(
                    "warn",
                    "HEADER_CONFLICT",
                    f"{name} is defined by BOTH {a.name} and {b.name}, and no single TU includes "
                    "both — split into TYPES_STEM islands (§9.108), do not hand-blocklist",
                )
            )
        if len(islands) > 1:
            findings.append(
                Finding(
                    "warn",
                    "TYPES_STEM_SPLIT",
                    f"{len(islands)} islands proposed: "
                    + "; ".join(f"{i.stem}=[{', '.join(i.tus)}]" for i in islands),
                )
            )
    else:
        islands = [
            Island(
                "types",
                sorted(closure.per_tu),
                allow_files,
                "one mutually-legal include set",
                wrapper_inc=wrapper,
                label=module.crate_stem,
            )
        ]

    # ---- extra -I ----
    extra_inc = [f'-I"${{SRC}}/{module.module_dir.relative_to(repo_root)}"']
    if (module.module_dir / "include").is_dir():
        extra_inc.append(f'-I"${{SRC}}/{module.module_dir.relative_to(repo_root)}/include"')
    if env.build_dir is not None:
        extra_inc.append(f'-I"${{SRC}}/build-{arch}/{module.module_dir.relative_to(repo_root)}"')
    for d in module.include_dirs:
        p = (d if d.is_absolute() else module.module_dir / d).resolve()
        if p.is_dir() and _is_within(p, repo_root):
            extra_inc.append(f'-I"${{SRC}}/{p.relative_to(repo_root)}"')
    late_inc: list[str] = []
    for d in extra_inc_dirs:
        p = (d if d.is_absolute() else repo_root / d).resolve()
        entry = (
            f'-I"${{SRC}}/{p.relative_to(repo_root)}"' if _is_within(p, repo_root) else f'-I"{p}"'
        )
        if entry not in extra_inc:
            extra_inc.append(entry)

    db_defines: list[str] | None = None
    db_path: Path | None = None
    if db is not None:
        db_path = db.path
        db_defines = db.module_defines
        # Compare only IN-TREE TUs: the db's build-dir entries are the generated sources
        # (MIDL/spec2def products), which the source list carries as ${CMAKE_CURRENT_BINARY_DIR}.
        db_tus = {p.resolve() for p in db.module_files if _is_within(p, module.module_dir)}
        want = {p.resolve() for p in full_tus}
        missing = want - db_tus
        stale = db_tus - want
        if missing:
            findings.append(
                Finding(
                    "warn",
                    "COMPILE_DB_DIVERGENCE",
                    "in the CMakeLists source list but NOT in compile_commands.json (stale db, or "
                    "the file is excluded): "
                    + ", ".join(sorted(_rel(p, repo_root) for p in missing)),
                )
            )
        if stale:
            findings.append(
                Finding(
                    "warn",
                    "COMPILE_DB_DIVERGENCE",
                    "compiled per compile_commands.json but NOT in the parsed source list — the "
                    "CMake parse missed a source: "
                    + ", ".join(sorted(_rel(p, repo_root) for p in stale)),
                )
            )
        for p in db.include_dirs:
            if not _is_within(p, repo_root) or not p.is_dir():
                continue
            # Emit unless the GENERATOR already has it (_GENERATOR_BASE_INC), not unless
            # WE searched it — see that constant for why the distinction matters.
            rel = p.relative_to(repo_root).as_posix()
            # The build-dir SDK includes are the generator's `${BUILD_DIR}/sdk/...`, which
            # is PER-ARCH. Emitting the observed `build-i386/sdk/...` instead would bake one
            # arch's generated headers into a 3-arch generator — the cross-arch header leak
            # AGENTS.md forbids, and it would read as correct because i386 is what we derive
            # from. (A build-dir path under the MODULE's own dir is different: those are
            # arch-independent MIDL wire types, and _is_within already keeps them.)
            if _BUILD_SDK_RE.match(rel):
                continue
            if not _is_within(p, module.module_dir) and rel not in _GENERATOR_BASE_INC:
                entry = f'-I"${{SRC}}/{p.relative_to(repo_root)}"'
                if entry not in extra_inc:
                    extra_inc.append(entry)

        # Re-bucket by the REAL command's ordering, whatever the entry's source. An -I the
        # compiler ranks BELOW the standard SDK dirs must stay below them here: it is an
        # alternate surface for headers the SDK also owns, and hoisting it changes which
        # definition a bare `#include <x.h>` resolves to. `sdk/include/wine` is the live case --
        # it comes from the module's own include_directories(), so a db-only rule misses it,
        # yet CMake still emits it last. samlib: 23 redefinition errors early, 0 late.
        order = {
            p.relative_to(repo_root).as_posix(): i
            for i, p in enumerate(db.include_dirs)
            if _is_within(p, repo_root)
        }
        std_at = order.get("sdk/include")
        if std_at is not None:

            def _rank(entry: str) -> int | None:
                inner = entry[3:].rstrip('"')
                return order.get(inner.removeprefix("${SRC}/")) if "${SRC}/" in inner else None

            demoted = [e for e in extra_inc if (r := _rank(e)) is not None and r > std_at]
            if demoted:
                extra_inc[:] = [e for e in extra_inc if e not in demoted]
                late_inc.extend(e for e in demoted if e not in late_inc)
    else:
        findings.append(
            Finding(
                "info",
                "NO_COMPILE_DB",
                f"no build-{arch}/compile_commands.json — VER_DEFS come from the CMakeLists parse "
                "plus the repo-wide defaults, not from the real compile command",
            )
        )

    ver_defs = _default_ver_defs(module, db_defines)
    if (
        module.is_usermode
        and any("ndk/" in w for w in wrapper)
        and ("-DNTOS_MODE_USER" not in ver_defs)
    ):
        ver_defs.append("-DNTOS_MODE_USER")

    return DerivedProfile(
        module=module,
        env=env,
        closure=closure,
        switch_consts=switch_consts,
        const_owners=owners,
        islands=islands,
        wrapper_inc=wrapper,
        allow_files=allow_files,
        extra_inc=extra_inc,
        late_inc=late_inc,
        ver_defs=ver_defs,
        allow_reasons=allow_reasons,
        excluded=excluded,
        findings=findings,
        compile_db_used=db_path,
        prototype_survey=survey,
    )


def _hdr_regex_dir(dirname: str) -> str:
    """``wkssvc`` -> ``.*wkssvc[/\\].*\\.h`` — the module-local-directory ALLOW_FILES form."""
    return r".*" + re.escape(dirname) + r"[/\\].*\.h"


# ---------------------------------------------------------------------------
# Emission — gen_driver_types.sh vocabulary
# ---------------------------------------------------------------------------


_OPAQUE_RE = re.compile(r"pub struct (\w+) \{\s*pub _address: u8,\s*\}")


TCHAR_FAMILY = (
    "TCHAR",
    "PTCHAR",
    "TBYTE",
    "PTBYTE",
    "LPTSTR",
    "LPCTSTR",
    "PTSTR",
    "PCTSTR",
    "LPTCH",
    "PTCH",
)


def opaque_types_to_unblock(dp: DerivedProfile, arch: str = "amd64") -> list[str]:
    """Struct tags the kernel -sys crate leaves opaque that this module needs WHOLE.

    ntkrnl-sys emits `_SCSI_REQUEST_BLOCK { _address: u8 }`, and the island blocklists every
    -sys type, so `Srb->SrbStatus` resolved to the opaque one: atapi's first check had 22
    `no field SrbStatus` and 22 `no field Cdb`. A tag is unblocked (with its plain and `P`
    typedefs) when an allowlisted closure header defines it and the module's own code names it.
    """
    root = dp.env.repo_root
    sys_src = root / "sdk/lib/rust/ntkrnl-sys/src" / f"bindgen_{arch}.rs"
    if not sys_src.is_file():
        return []
    opaque = set(_OPAQUE_RE.findall(_read(sys_src)))
    allow = [re.compile(a) for a in dp.allow_files]
    defined: set[str] = set()
    for path, ref in dp.closure.headers.items():
        posix = path.as_posix()
        if ref.kind != "module" and not any(a.fullmatch(posix) or a.search(posix) for a in allow):
            continue
        for tag in re.findall(r"\bstruct\s+(\w+)\s*\{", _read(path)):
            if tag in opaque:
                defined.add(tag)
    own = "\n".join(_read(p) for p in dp.module.compiled_tus)
    own += "\n".join(_read(p) for p, r in dp.closure.headers.items() if r.kind == "module")
    out: list[str] = []
    for tag in sorted(defined):
        plain = tag[1:] if tag.startswith("_") else tag
        names = [tag, plain, f"P{plain}"]
        if any(re.search(rf"\b{re.escape(n)}\b", own) for n in names):
            out.extend(dict.fromkeys(names))
    return out


CPP_SUFFIXES = (".cpp", ".cc", ".cxx")


def emit_profile(dp: DerivedProfile) -> str:
    """The ready-to-paste ``case`` arm for ``gen_driver_types.sh``.

    Emitted (never patched in): the integrator owns that file.
    """
    m = dp.module
    rel = m.module_dir.relative_to(dp.env.repo_root)
    multi = len(dp.islands) > 1
    wrapper = "\\n".join(dp.wrapper_inc)
    out: list[str] = []
    # Multi-island profiles use the netapi32 shape: ONE case arm matching every island label,
    # branching WRAPPER_INC/ALLOW_FILES on $DRIVER, with OUT_CRATE_DIR pinned so all islands
    # land in the same crate. TYPES_STEM comes from the env at invocation.
    labels = "|".join(i.label for i in dp.islands) if multi else m.crate_stem
    out.append(f"  {labels})")
    out.append(
        f"    # AUTO-DERIVED by rustconv.profile_derive on {date.today().isoformat()} from "
        f"{rel}/CMakeLists.txt"
    )
    out.append(
        f"    # ({len(m.compiled_tus)} compiled TU(s), {len(dp.closure.headers)} headers in the "
        f"include closure, {len({c.ident for c in dp.switch_consts})} switch constant(s))."
    )
    out.append(
        "    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c "
        "compiles,"
    )
    out.append("    # that exact sequence is bindgen-able). Review the WHY notes in the report.")
    out.append(f"    USERMODE={1 if m.is_usermode else 0}")
    if not m.is_usermode:
        # Real Rust unions, not `__BindgenUnionField<T>`: a translation of C reaches an
        # anonymous union member as a plain field (`Regs.Control`), and crate-repair routes
        # that to `__bindgen_anon_N.Control` -- which only typechecks against a real union.
        # Kernel only, and only because no derived kernel arm predates it: flipping it for a
        # usermode arm someone already compiles against would rewrite that crate's island.
        out.append("    REAL_UNIONS=1")
    if m.is_usermode:
        # Emitted as 1 always, and the measurement goes NEXT TO IT rather than into it. A
        # count that nearly separates twelve samples is a count that will pick wrong on the
        # thirteenth (see PrototypeSurvey), and BLOCKLIST_FUNCS_FROM_CRATE has an ordering
        # constraint that makes 1 the only safe unattended answer anyway.
        survey = dp.prototype_survey
        if survey is not None:
            out.append(
                f"    # BLOCKLIST_FUNCS, MEASURED: {len(survey.prototypes)} external "
                f"prototype(s) declared by the include closure, {len(survey.undeclared)} "
                "called but declared nowhere in it."
            )
            out.append(
                "    # The count does NOT settle the flag — island it both ways and run one "
                "cargo check each. See the report."
            )
        if any(str(tu).lower().endswith(CPP_SUFFIXES) for tu in m.compiled_tus):
            # A C++ cell keeps the SDK's prototypes (fltmc's FilterLoad is in no other crate):
            # the module's OWN functions are `_Z`-mangled without `extern "C"` and the -x c++
            # post-pass strips them, so nothing it defines can clash. FROM_CRATE drops any
            # `extern "C"` one it also defines.
            out.append("    BLOCKLIST_FUNCS=0")
            out.append("    BLOCKLIST_FUNCS_FROM_CRATE=1")
        else:
            out.append("    BLOCKLIST_FUNCS=1")
    out.append(f'    DRIVER_DIR="${{SRC}}/{rel}"')
    if any(str(tu).lower().endswith(CPP_SUFFIXES) for tu in m.compiled_tus):
        # A C++ TU's include closure declares classes (`class CCanvasWindow : public
        # CWindowImpl<...>`), which a C parse rejects as an unknown type name: the island
        # must be parsed the way the TU is compiled.
        out.append("    LANG_ARGS=( -x c++ -std=gnu++17 )")
    if not multi:
        out.append(f"    WRAPPER_INC=$'{wrapper}'")
        out.extend(_allow_files_lines(dp.allow_files))
    else:
        out.append('    case "$DRIVER" in')
        for isl in dp.islands:
            out.append(f"      # island {isl.stem} — {len(isl.tus)} TU(s):")
            line = "      #"
            for tu in isl.tus:
                leaf = Path(tu).name
                if len(line) + len(leaf) > 92:
                    out.append(line)
                    line = "      #"
                line += f" {leaf}"
            out.append(line)
            out.append(f"      {isl.label})")
            out.append("        WRAPPER_INC=$'" + "\\n".join(isl.wrapper_inc) + "'")
            out.extend("    " + line for line in _allow_files_lines(isl.allow_files))
            out.append("        ;;")
        out.append("    esac")
    unblock = opaque_types_to_unblock(dp) if not m.is_usermode else []
    if m.is_usermode and m.unicode:
        # ntdll-sys is built WITHOUT UNICODE, so its glob says `TCHAR = c_char`, and a module
        # header's `TCHAR szPath[MAX_PATH]` field inherits it: half the bytes the C has
        # (desk's _ICON_PATH, then the control panels' buffers). Unblocked, the island emits
        # the family from its own -DUNICODE compile; the prelude covers the locals.
        out.append(
            "    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's."
        )
        out.append('    UNBLOCK="' + " ".join([*unblock, *TCHAR_FAMILY]) + '"')
        unblock = []
    if unblock:
        out.append(
            "    # OPAQUE in ntkrnl-sys ({ _address: u8 }) but defined by an allowlisted header "
            "and named by"
        )
        out.append("    # this module's C, so the island emits the real layout (fields reachable).")
        out.append('    UNBLOCK="' + " ".join(unblock) + '"')
    out.append("    EXTRA_INC=( " + " ".join(dp.extra_inc) + " )")
    if dp.late_inc:
        out.append(
            "    # LOWER priority than the standard SDK dirs -- the real compile lists these"
        )
        out.append(
            "    # after them, and hoisting one ahead swaps which header an include resolves to."
        )
        out.append("    LATE_INC=( " + " ".join(dp.late_inc) + " )")
    out.append("    VER_DEFS=( " + " ".join(dp.ver_defs) + " )")
    if multi:
        out.append(
            f'    OUT_CRATE_DIR="${{OUT_CRATE_DIR:-${{SRC}}/sdk/lib/rust/{dp.crate_name}/src}}"'
        )
    out.append("    ;;")
    if multi:
        out.append("")
        out.append("  # MULTI-ISLAND: these TUs do NOT share one legal include set (§9.108), so")
        out.append("  # generate one pass per group — all into the same crate:")
        for isl in dp.islands:
            stem_env = "" if isl.stem == "types" else f"TYPES_STEM={isl.stem} "
            out.append(f"  #   {stem_env}gen_driver_types.sh {isl.label}")
        out.append("  # Then dispatch them from the crate: one `mod <stem>;` per island, and each")
        out.append("  # translated module imports from the island its own C include set belongs to")
        out.append("  # (names legitimately overlap between islands — never glob both into scope).")
    return "\n".join(out) + "\n"


def _allow_files_lines(allow_files: list[str]) -> list[str]:
    """``ALLOW_FILES=( --allowlist-file '…' … )`` in the shape the script's profiles use."""
    lines = ["    ALLOW_FILES=( " + " ".join(f"--allowlist-file '{a}'" for a in allow_files[:2])]
    lines += [f"                  --allowlist-file '{a}'" for a in allow_files[2:]]
    lines[-1] = lines[-1] + " )"
    return lines


def emit_report(dp: DerivedProfile) -> str:
    """The evidence: what was derived, and WHY each header is in or out."""
    m = dp.module
    root = dp.env.repo_root
    L: list[str] = []
    L.append(
        f"=== profile_derive: {m.name} ({m.module_type}, "
        f"{'USERMODE' if m.is_usermode else 'KERNEL'}) ==="
    )
    L.append(f"  cmake        : {_rel(m.cmake, root)}")
    L.append(f"  arch         : {dp.env.arch}")
    src_of_truth = (
        "compile_commands.json + CMakeLists" if dp.compile_db_used else "CMakeLists parse only"
    )
    L.append(f"  include src  : {src_of_truth}")
    L.append(f"  islands      : {len(dp.islands)} ({', '.join(i.stem for i in dp.islands)})")
    L.append("")
    L.append("--- compiled translation units (the translation surface) ---")
    for e in m.entries:
        if e.kind == "tu" and e.path is not None:
            note = f"   [only under {e.condition}]" if e.condition else ""
            L.append(f"  TU        {_rel(e.path, root)}{note}")
    for g in m.generated_sources:
        L.append(f"  GENERATED {g}   (stays C — MIDL/spec2def product)")
    if m.pch_header:
        L.append(f"  PCH       {_rel(m.pch_header, root)}")
    for d in m.dead_sources:
        L.append(
            f"  DEAD      {_rel(d, root)}   (on disk, NOT compiled — dead source or already "
            "Rust-flipped)"
        )
    L.append("")
    L.append("--- include closure (why each header is in) ---")
    deep = 0
    for ref in sorted(dp.closure.headers.values(), key=lambda r: (r.depth, r.name)):
        if ref.depth > 2 and ref.kind not in ("module", "generated"):
            deep += 1
            continue
        L.append(
            f"  d{ref.depth} {ref.kind:9s} {ref.name:28s} via {ref.includers[0]}"
            + (f" (+{len(ref.includers) - 1} more)" if len(ref.includers) > 1 else "")
        )
    if deep:
        L.append(f"  … +{deep} deeper SDK header(s) (depth >2) — see --emit json for the full list")
    if dp.closure.unresolved:
        L.append("")
        L.append("--- UNRESOLVED includes (never silently dropped) ---")
        for u in dp.closure.unresolved:
            L.append(f"  {u.includer}:{u.line}  {u.name}  — {u.reason}")
    L.append("")
    L.append("--- ALLOW_FILES decisions ---")
    for rx in dp.allow_files:
        L.append(f"  + {rx}")
        L.append(f"      {dp.allow_reasons[rx]}")
    if dp.excluded:
        buckets: dict[str, list[str]] = {}
        for name, why in sorted(dp.excluded.items()):
            buckets.setdefault(why.split("(")[0].split("—")[0].strip(), []).append(name)
        L.append("  excluded (grouped by reason; --emit json lists every one):")
        for why, names in sorted(buckets.items()):
            L.append(
                f"  - {why} [{len(names)}]: "
                + ", ".join(names[:8])
                + (" …" if len(names) > 8 else "")
            )
    L.append("")
    L.append("--- switch-constant audit (field manual §9.112) ---")
    if not dp.switch_consts:
        L.append("  (no case labels with identifiers)")
    used_at: dict[str, list[SwitchConst]] = {}
    for c in dp.switch_consts:
        used_at.setdefault(c.ident, []).append(c)
    for o in dp.const_owners:
        where = used_at.get(o.ident, [])
        site = f"{where[0].tu}:{where[0].line}" if where else "?"
        owner = o.owner.name if o.owner else o.owner_kind
        carried = o.owner_kind in ("sys-crate", "tu-local", "cxx-const")
        mark = "✓" if (o.allowlisted or carried) else "✗"
        how = "-sys glob" if o.in_sys else ("island" if o.allowlisted else o.owner_kind)
        L.append(f"  {mark} {o.ident:38s} {owner:24s} [{o.macro_kind}] via {how:10s} {site}")
    L.append("")
    if dp.prototype_survey is not None:
        s = dp.prototype_survey
        L.append("--- BLOCKLIST_FUNCS evidence (measured; the flag stays yours) ---")
        L.append(f"  external prototypes the closure declares  : {len(s.prototypes)}")
        L.append(f"  called but declared nowhere in the closure: {len(s.undeclared)}")
        L.append(f"  {s.summary}")
        if s.prototypes:
            line = "   "
            for name in s.prototypes:
                if len(line) + len(name) > 96:
                    L.append(line)
                    line = "   "
                line += f" {name}"
            L.append(line)
        L.append("")
    L.append("--- findings ---")
    if not dp.findings:
        L.append("  (none)")
    for f in sorted(dp.findings, key=lambda f: ("error", "warn", "info").index(f.severity)):
        L.append(str(f))
    L.append("")
    L.append("--- derived profile (paste into gen_driver_types.sh) ---")
    L.append(emit_profile(dp))
    return "\n".join(L)


def to_json(dp: DerivedProfile) -> dict:
    root = dp.env.repo_root
    return {
        "module": dp.module.name,
        "cmake": _rel(dp.module.cmake, root),
        "arch": dp.env.arch,
        "module_type": dp.module.module_type,
        "usermode": dp.module.is_usermode,
        "crate": dp.crate_name,
        "compiled_tus": [_rel(p, root) for p in dp.module.compiled_tus],
        "generated_sources": dp.module.generated_sources,
        "dead_sources": [_rel(p, root) for p in dp.module.dead_sources],
        "closure": [
            {
                "name": r.name,
                "path": _rel(p, root),
                "kind": r.kind,
                "depth": r.depth,
                "includers": r.includers,
            }
            for p, r in sorted(dp.closure.headers.items(), key=lambda kv: kv[1].name)
        ],
        "unresolved": [
            {"name": u.name, "includer": u.includer, "line": u.line, "reason": u.reason}
            for u in dp.closure.unresolved
        ],
        "allow_files": {rx: dp.allow_reasons[rx] for rx in dp.allow_files},
        "excluded": dp.excluded,
        "wrapper_inc": dp.wrapper_inc,
        "extra_inc": dp.extra_inc,
        "late_inc": dp.late_inc,
        "ver_defs": dp.ver_defs,
        "islands": [{"stem": i.stem, "tus": i.tus, "reason": i.reason} for i in dp.islands],
        "switch_constants": [
            {
                "ident": o.ident,
                "owner": _rel(o.owner, root) if o.owner else None,
                "owner_kind": o.owner_kind,
                "macro_kind": o.macro_kind,
                "allowlisted": o.allowlisted,
            }
            for o in dp.const_owners
        ],
        "findings": [
            {"severity": f.severity, "code": f.code, "detail": f.detail} for f in dp.findings
        ],
        "compile_db": _rel(dp.compile_db_used, root) if dp.compile_db_used else None,
        "profile": emit_profile(dp),
    }


# ---------------------------------------------------------------------------
# Crate scaffold (shape copied from sdk/lib/rust/wkssvc-rust)
# ---------------------------------------------------------------------------


_RUST_KEYWORDS = frozenset(
    {
        "as",
        "async",
        "await",
        "break",
        "const",
        "continue",
        "crate",
        "dyn",
        "else",
        "enum",
        "extern",
        "false",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "ref",
        "return",
        "self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
        "abstract",
        "become",
        "box",
        "do",
        "final",
        "gen",
        "macro",
        "override",
        "priv",
        "try",
        "typeof",
        "unsized",
        "virtual",
        "yield",
    }
)


def _rust_mod_ident(stem: str) -> str:
    if stem[:1].isdigit():  # ndis's 30stubs.c: no raw identifier makes this one legal
        return f"_{stem}"
    return f"r#{stem}" if stem in _RUST_KEYWORDS else stem


def _rust_mod_decl(stem: str) -> str:
    """`pub mod x;` for a TU, with a #[path] when the file name is not an identifier."""
    ident = _rust_mod_ident(stem)
    if ident == f"_{stem}":
        return f'#[path = "{stem}.rs"] pub mod {ident};'
    return f"pub mod {ident};"


#: Appended to a KERNEL crate's scaffold prelude: what every port of a WDM driver needs and
#: the -sys `ddk` glob does not carry (see the comments inside). Written once for
#: pciidex/atapi/legide on 2026-10-06; every later kernel port starts from it.
KERNEL_PRELUDE_EXTRAS = r"""
// ================= PnP / resource constants the -sys glob does not carry ===================
// The C switches on these. ntkrnl-sys has them only in `driver_ddk` (hand-written, every
// arch) and, on x86, `hal_ddk` -- neither is reached by the `ddk` glob the island
// re-exports, so a bare name in a match arm would bind everything (§9.112). Values: wdm.h.
pub use ntkrnl_sys::driver_ddk::{
    IRP_MN_CANCEL_REMOVE_DEVICE, IRP_MN_CANCEL_STOP_DEVICE, IRP_MN_DEVICE_USAGE_NOTIFICATION,
    IRP_MN_QUERY_CAPABILITIES, IRP_MN_QUERY_DEVICE_RELATIONS, IRP_MN_QUERY_DEVICE_TEXT,
    IRP_MN_QUERY_ID, IRP_MN_QUERY_INTERFACE, IRP_MN_QUERY_PNP_DEVICE_STATE, IRP_MN_QUERY_POWER,
    IRP_MN_QUERY_REMOVE_DEVICE, IRP_MN_QUERY_RESOURCES, IRP_MN_QUERY_RESOURCE_REQUIREMENTS,
    IRP_MN_QUERY_STOP_DEVICE, IRP_MN_REMOVE_DEVICE, IRP_MN_SET_POWER, IRP_MN_START_DEVICE,
    IRP_MN_STOP_DEVICE, IRP_MN_SURPRISE_REMOVAL,
};
// CM_PARTIAL_RESOURCE_DESCRIPTOR.Type is a UCHAR.
pub const CmResourceTypePort: u8 = 1;
pub const CmResourceTypeInterrupt: u8 = 2;
pub const CmResourceTypeMemory: u8 = 3;

// ================= kernel header helpers ======================================================
// wdm.h/ntddk.h MACROS and MSVC intrinsics the C calls as functions. ntkrnl-sys has portable
// wrappers for them outside the `ddk` glob (ke, inline::tier1, inline::ke, interlocked), so
// they are re-exported here BY NAME -- the C's `_`-prefixed intrinsic spellings included.
// ntkrnl_sys::ke's spin-lock helpers take ITS `PKSPIN_LOCK` (`*mut usize`); a driver's lock
// fields are the island's (`*mut c_ulonglong` on 64-bit) -- the same width, a distinct Rust
// type, so a bare re-export made every call site E0308 (16 in atapi). Thin wrappers over the
// island's type instead.
#[inline]
pub unsafe fn KeAcquireSpinLock(SpinLock: PKSPIN_LOCK, OldIrql: *mut KIRQL) {
    ntkrnl_sys::ke::KeAcquireSpinLock(SpinLock.cast(), OldIrql)
}
#[inline]
pub unsafe fn KeReleaseSpinLock(SpinLock: PKSPIN_LOCK, NewIrql: KIRQL) {
    ntkrnl_sys::ke::KeReleaseSpinLock(SpinLock.cast(), NewIrql)
}
#[inline]
pub unsafe fn KeAcquireSpinLockAtDpcLevel(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::ke::KeAcquireSpinLockAtDpcLevel(SpinLock.cast())
}
#[inline]
pub unsafe fn KeReleaseSpinLockFromDpcLevel(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel(SpinLock.cast())
}
#[inline]
pub unsafe fn KeInitializeSpinLock(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::inline::ke::KeInitializeSpinLock(SpinLock.cast())
}
pub use ntkrnl_sys::inline::tier1::{
    IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation,
    IoMarkIrpPending, IoSetCancelRoutine, IoSetCompletionRoutine, IoSetNextIrpStackLocation,
    IoSkipCurrentIrpStackLocation,
};
// KeRaiseIrql/KeLowerIrql are wdm.h MACROS over KfRaiseIrql/KfLowerIrql: x86 has wrappers in
// inline::tier1; amd64 writes CR8 inline (inline::mm); arm64 calls the HAL exports.
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::inline::tier1::{KeLowerIrql, KeRaiseIrql};
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = ntkrnl_sys::inline::mm::KfRaiseIrql(NewIrql);
}
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) {
    ntkrnl_sys::inline::mm::KfLowerIrql(NewIrql);
}
#[cfg(target_arch = "aarch64")]
extern "C" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    fn KfLowerIrql(NewIrql: KIRQL);
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = KfRaiseIrql(NewIrql);
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) {
    KfLowerIrql(NewIrql);
}
// ntbasedef.h: `__assume(e)` is an optimiser promise; `RTL_BITS_OF(t)` is sizeof(t) * 8.
#[inline(always)]
pub unsafe fn __assume(cond: bool) {
    core::hint::assert_unchecked(cond)
}
#[macro_export] macro_rules! RTL_BITS_OF { ($t:ty $(,)?) => { (core::mem::size_of::<$t>() * 8) }; }
pub use crate::RTL_BITS_OF;
pub use ntkrnl_sys::inline::common::InterlockedCompareExchange;
pub use ntkrnl_sys::inline::common::InterlockedCompareExchange as _InterlockedCompareExchange;
pub use ntkrnl_sys::inline::ke::{BitScanForward, BitScanReverse};
pub use ntkrnl_sys::inline::ke::{
    BitScanForward as _BitScanForward, BitScanReverse as _BitScanReverse,
};
pub use ntkrnl_sys::interlocked::{
    InterlockedAnd, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedExchangeAdd,
    InterlockedExchangePointer, InterlockedIncrement, InterlockedOr, InterlockedXor,
};
pub use ntkrnl_sys::interlocked::{
    InterlockedAnd as _InterlockedAnd, InterlockedDecrement as _InterlockedDecrement,
    InterlockedExchangeAdd as _InterlockedExchangeAdd,
    InterlockedIncrement as _InterlockedIncrement,
    InterlockedOr as _InterlockedOr, InterlockedXor as _InterlockedXor,
};

/// wdm.h: `IoAcquireRemoveLockEx(RemoveLock, Tag, __FILE__, __LINE__, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoAcquireRemoveLock(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) -> NTSTATUS {
    IoAcquireRemoveLockEx(
        RemoveLock,
        Tag,
        concat!(file!(), "\0").as_ptr() as PCSTR,
        line!(),
        core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}
/// wdm.h: `IoReleaseRemoveLockEx(RemoveLock, Tag, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoReleaseRemoveLock(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    IoReleaseRemoveLockEx(RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG)
}
/// wdm.h: `IoReleaseRemoveLockAndWaitEx(RemoveLock, Tag, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoReleaseRemoveLockAndWait(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    IoReleaseRemoveLockAndWaitEx(RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG)
}

// PAGED_CODE() asserts IRQL < DISPATCH_LEVEL in checked builds and is empty otherwise.
#[macro_export] macro_rules! PAGED_CODE { () => {{}}; }
// NT_VERIFY(exp) is `((exp) ? TRUE : FALSE)` in a free build: the expression is evaluated.
#[macro_export] macro_rules! NT_VERIFY { ($e:expr $(,)?) => { ($e) }; }
// ntbasedef.h: UNREACHABLE is `__builtin_unreachable()`, so reaching it is UB in C too.
#[macro_export] macro_rules! UNREACHABLE {
    () => { core::hint::unreachable_unchecked() }; // nolint: assert-to-ub (__builtin_unreachable)
}
pub use crate::{NT_VERIFY, PAGED_CODE, UNREACHABLE};

// HAL port/register I/O (hal.dll exports on i386/arm64, inline on amd64).
pub use ntkrnl_sys::portio::*;
// C's TRUE/FALSE are ints; in kernel code they land in BOOLEAN (UCHAR) far more than BOOL.
pub const TRUE: BOOLEAN = 1;
pub const FALSE: BOOLEAN = 0;

// C's min()/max() macros (`((a) < (b) ? (a) : (b))`): both operands already share a type in
// a translation that compiles, and the C evaluates the winner twice, which no caller here
// relies on.
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
"""


def scaffold_crate(
    dp: DerivedProfile,
    out_dir: Path,
    force: bool = False,
    kept: list[Path] | None = None,
) -> list[Path]:
    """Write the crate skeleton: Cargo.toml, lib.rs, prelude.rs, types.rs (+ island stubs).

    Refuses to overwrite an existing file unless *force*: the per-arch island files are
    ``gen_driver_types.sh`` output, and clobbering a real island with a placeholder is
    exactly the silent-degradation this pipeline must never do.

    ``src/types.rs`` and ``src/types_<arch>.rs`` are the exception, because BOTH tools write
    them and neither is wrong to: whichever ran first owns them. Our copy is a placeholder
    that a real island strictly dominates, so an existing one is kept and reported via
    *kept* rather than aborting the scaffold. That keeps the two steps order-insensitive —
    running gen_driver_types.sh first used to abort the whole scaffold over a dispatch stub
    whose content is identical either way.
    """
    m = dp.module
    crate = dp.crate_name
    lib = crate.replace("-", "_")
    src = out_dir / "src"
    written: list[Path] = []
    # PLAN, then commit. Nothing below touches the disk until every target has been checked,
    # because a scaffold that aborts halfway leaves files it refuses to overwrite on the retry
    # — the tool wedging itself against its own partial output, with a message that reads like
    # the user put them there.
    planned: list[tuple[Path, str]] = []
    soft: set[Path] = set()

    def write(path: Path, text: str) -> None:
        planned.append((path, text))

    def write_unless_generated(path: Path, text: str) -> None:
        """For the files gen_driver_types.sh also owns — keep theirs, never clobber."""
        soft.add(path)
        planned.append((path, text))

    def commit() -> None:
        conflicts = [p for p, _ in planned if p.exists() and p not in soft]
        if conflicts and not force:
            listing = "\n  ".join(str(p) for p in conflicts)
            raise ProfileDeriveError(
                f"refusing to overwrite {len(conflicts)} existing file(s) "
                f"(pass force=True):\n  {listing}"
            )
        src.mkdir(parents=True, exist_ok=True)
        (src / m.crate_stem).mkdir(exist_ok=True)
        for path, text in planned:
            if path in soft and path.exists() and not force:
                if kept is not None:
                    kept.append(path)
                continue
            path.write_text(text)
            written.append(path)

    deps = (
        'panic-common = { path = "../panic-common", features = ["user"] }\n'
        'ntdll-sys = { path = "../ntdll-sys" }\n'
        if m.is_usermode
        else 'panic-common = { path = "../panic-common", features = ["kernel"] }\n'
        'ntkrnl-sys = { path = "../ntkrnl-sys" }\n'
        # Kernel crates too: DPRINT1 is win32macros' (DbgPrint is cdecl in ntoskrnl exactly
        # as in ntdll), and the rest are header macros kernel C uses as much as user C.
    ) + 'win32macros-rust = { path = "../win32macros-rust" }\n'
    is_cpp = any(str(tu).lower().endswith(CPP_SUFFIXES) for tu in m.compiled_tus)
    atl_block = ""
    if is_cpp:
        # The ATL types a C++ worker is told are in scope (atl_contract.py). Named, not a
        # glob: atl-rust's primitive aliases (WCHAR, BOOL) would collide with the island's.
        com = any(
            re.search(r"\bCComHeapPtr\b", tu.read_text(errors="replace"))
            for tu in m.compiled_tus
            if tu.is_file()
        )
        # CComHeapPtr is the COM task allocator (ole32): only a module that uses it imports it.
        deps += (
            'atlclasses-rust = { path = "../atlclasses-rust", features = ["com"] }\n'
            if com
            else 'atlclasses-rust = { path = "../atlclasses-rust" }\n'
        )
        atl_block = (
            "\n// ================= ATL =========================================================="
            "==============\n"
            "// ATL's CStringW/CComPtr/containers, ported once in atlclasses-rust.\n"
            "pub use atl_rust::{atl_format, CAtlList, CComPtr, CHeapPtr, CSimpleArray, "
            "CSimpleMap, CStringW, POSITION};\n"
            + ("pub use atl_rust::CComHeapPtr;\n" if com else "")
        )
    win32_reexport = ""
    # The MACROS only, by name: a glob would also pull win32macros' type aliases
    # (HANDLE, WCHAR) into the prelude next to the island's, and every use of either
    # becomes E0659 ambiguous.
    macro_src = dp.env.repo_root / "sdk" / "lib" / "rust" / "win32macros-rust" / "src"
    names = sorted(
        {
            n
            for f in (macro_src.glob("*.rs") if macro_src.is_dir() else [])
            for n in re.findall(r"macro_rules!\s+([A-Za-z_]\w*)", f.read_text())
        }
    )
    if names:
        win32_reexport = (
            "// The Win32 header macros, ported once in win32macros-rust, not per module.\n"
            f"pub use win32macros_rust::{{{', '.join(names)}}};\n"
        )
    write(
        out_dir / "Cargo.toml",
        f"""# {crate} — see src/lib.rs for the module map.
[package]
name = "{crate}"
version = "0.1.0"
edition = "2021"
autobins = false

[lib]
name = "{lib}"
path = "src/lib.rs"
crate-type = ["staticlib"]

[dependencies]
{deps}""",
    )

    mods = []
    for tu in m.compiled_tus:
        # `enum.c` (atapi) is a Rust keyword: a raw identifier still names enum.rs.
        mods.append(f"    {_rust_mod_decl(tu.stem)}")
    write(
        src / "lib.rs",
        f"""//! {m.crate_stem} — Rust translation of {m.module_dir.relative_to(dp.env.repo_root)}.
//! staticlib → lib{lib}.a, cohab-linked into the cmake-built {m.name}
//! ({"usermode" if m.is_usermode else "kernel-mode"} module type `{m.module_type}`).
//!
//! Module map mirrors the CMakeLists source list exactly — the {len(m.compiled_tus)} TU(s)
//! the build actually compiles. Generated sources ({", ".join(m.generated_sources) or "none"})
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh {m.crate_stem})
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod {m.crate_stem} {{
{chr(10).join(mods)}
}}

// staticlib needs a panic handler + eh personality (no_std cohab); std supplies them under test.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {{
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {{}}
""",
    )

    # A usermode island re-exports ntdll_sys::nt publicly; a KERNEL island's
    # `use ntkrnl_sys::ddk::*;` is private, so the prelude must glob the kernel types itself.
    # It said "the island re-exports" for both, and atapi's first resolve reported IRP,
    # LIST_ENTRY and IO_STACK_LOCATION as "defined nowhere".
    kernel_glob = (
        "// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;"
        if m.is_usermode
        else "// A kernel island's `use ntkrnl_sys::ddk::*;` is private: glob it here.\n"
        "pub use ntkrnl_sys::ddk::*;"
    )
    uses_wine = any("wine/debug.h" in w for w in dp.wrapper_inc)
    debug_block = (
        """// The wine/debug.h channel (TRACE/ERR/WARN/FIXME). Kept as no-op MACROS, never
// out-of-line fns: an out-of-line no-op is an optimization barrier that uncloaks latent
// ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, FIXME, TRACE, WARN};"""
        if uses_wine
        else """// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};"""
    )
    write(
        src / "prelude.rs",
        f"""//! {crate} prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void}};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{{addr_of, addr_of_mut}};

// ================= debug ======================================================================
{debug_block}
{atl_block}
// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {{
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {{
    s >= 0
}}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS {{ ($s:expr) => {{ (($s) as i32) >= 0 }}; }}
#[macro_export] macro_rules! CONTAINING_RECORD {{
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{{{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {{
            core::mem::size_of::<T>()
        }}
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe {{ core::ptr::addr_of!((*slot.as_ptr()).$($field).+) }});
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }}}};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {{
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    }};
}}
pub use crate::{{CONTAINING_RECORD, NT_SUCCESS}};
{win32_reexport}
{kernel_glob}
"""
        + ("" if m.is_usermode else KERNEL_PRELUDE_EXTRAS),
    )

    write_unless_generated(
        src / "types.rs",
        """//! Module-private struct layouts (exact, bindgen-generated per arch).
//! Generated by tools/rustconv/gen_driver_types.sh. Do not edit by hand.
#[cfg(target_arch = "x86")]      #[path = "types_i386.rs"]  mod types_arch;
#[cfg(target_arch = "x86_64")]   #[path = "types_amd64.rs"] mod types_arch;
#[cfg(target_arch = "aarch64")]  #[path = "types_arm64.rs"] mod types_arch;
pub use types_arch::*;
""",
    )
    for arch in ARCHES:
        # a REAL island must never be clobbered with a placeholder
        write_unless_generated(
            src / f"types_{arch}.rs",
            f"""// PLACEHOLDER — the real bindgen island has not been generated yet.
// An empty island is a CORRECTNESS bug, not a missing convenience: constants absent from an
// island become silent catch-all match arms (field manual §9.112). So this fails the build
// loudly instead of compiling as an empty module.
compile_error!(
    "types_{arch}.rs is a placeholder: run tools/rustconv/gen_driver_types.sh {m.crate_stem} \\
     (add the derived profile first) to generate the real island."
);
""",
        )
    commit()
    return written


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def _default_repo_root() -> Path:
    from .config import REACTOS_ROOT

    return Path(REACTOS_ROOT)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        prog="python -m rustconv.profile_derive",
        description="Derive a gen_driver_types.sh island profile from a module's build inputs.",
    )
    ap.add_argument(
        "module", help="module directory (holding CMakeLists.txt), abs or repo-relative"
    )
    ap.add_argument("--repo-root", type=Path, default=None)
    ap.add_argument("--arch", default="i386", choices=ARCHES)
    ap.add_argument("--target", default=None, help="CMake target, when the file has several")
    ap.add_argument(
        "--source",
        type=Path,
        action="append",
        default=[],
        help="explicit TU subset, repo-relative; repeatable, membership checked",
    )
    ap.add_argument("--emit", default="report", choices=("report", "profile", "json"))
    ap.add_argument("--allow-unresolved", action="store_true", help="demote unresolved to findings")
    ap.add_argument("--no-compile-db", action="store_true", help="ignore compile_commands.json")
    ap.add_argument(
        "--extra-inc",
        type=Path,
        action="append",
        default=[],
        metavar="DIR",
        help="extra include dir (repeatable) for a module that includes from an unusual path",
    )
    ap.add_argument(
        "--flipped-sources",
        dest="flipped_sources",
        action="store_true",
        help="Re-enable the source list a cohab flip commented out, and audit against THAT. "
        "A flip does not delete the list, so the pre-flip translation surface is still in the "
        "CMakeLists — which is how a flipped module gets re-audited (case labels, dead bodies, "
        "enum members) without going to git. Without it, a flipped module refuses rather than "
        "reporting empty sections that read as passes.",
    )
    ap.add_argument("--scaffold-dir", type=Path, default=None, help="also write the crate skeleton")
    ap.add_argument("--force", action="store_true", help="scaffold may overwrite existing files")
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument(
        "--usermode",
        dest="usermode",
        action="store_true",
        default=None,
        help="target is user-mode (USERMODE=1). Needed for a target with no "
        "set_module_type(), e.g. a static library shared between DLLs (msvcrt_static).",
    )
    mode.add_argument(
        "--kernel", dest="usermode", action="store_false", help="target is kernel-mode (USERMODE=0)"
    )
    args = ap.parse_args(argv)

    repo_root = (args.repo_root or _default_repo_root()).resolve()
    module_dir = Path(args.module)
    if not module_dir.is_absolute():
        module_dir = repo_root / module_dir
    try:
        dp = derive(
            module_dir,
            repo_root,
            arch=args.arch,
            target=args.target,
            strict_unresolved=not args.allow_unresolved,
            use_compile_db=not args.no_compile_db,
            extra_inc_dirs=tuple(args.extra_inc),
            usermode=args.usermode,
            flipped_sources=args.flipped_sources,
            sources=tuple(args.source),
        )
    except ProfileDeriveError as exc:
        print(f"profile_derive: FAILED\n{exc}", file=sys.stderr)
        return 2

    if args.emit == "profile":
        print(emit_profile(dp), end="")
    elif args.emit == "json":
        print(json.dumps(to_json(dp), indent=2))
    else:
        print(emit_report(dp))

    if args.scaffold_dir is not None:
        kept: list[Path] = []
        written = scaffold_crate(dp, args.scaffold_dir, force=args.force, kept=kept)
        print(f"\nscaffold: {len(written)} file(s) under {args.scaffold_dir}", file=sys.stderr)
        for p in written:
            print(f"  {p}", file=sys.stderr)
        if kept:
            print(
                f"kept {len(kept)} existing gen_driver_types.sh output(s) — NOT overwritten:",
                file=sys.stderr,
            )
            for p in kept:
                print(f"  {p}", file=sys.stderr)

    return 1 if dp.errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
