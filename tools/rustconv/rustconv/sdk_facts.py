"""sdk_facts.py — resolve SDK constant NAMES to VALUES using the real target compiler.

WHY THIS EXISTS. Closing the substrate of a translated TU means declaring a few hundred
constants the model used but the crate does not define. Transcribing them by hand is both
slow and unsafe: a wrong value compiles fine and misbehaves at runtime, and a constant that
stays missing does not even fail loudly — in a ``match`` arm an unresolved name becomes a
BINDING that swallows every case (the catch-all class that shipped a BUGCHECK, field manual
[§9.112]). So the value has to come from an authority, not from a reading of the header.

WHY THE COMPILER AND NOT A HEADER PARSE. The names are a mix of object-like macros
(``FOF_NOCONFIRMATION``), macros defined in terms of other macros
(``SEE_MASK_CLASSALL``), and C/C++ ENUM constants (``SHCONTF_FOLDERS``, ``ASSOCSTR_COMMAND``)
— and from the outside you cannot tell which is which. A ``-dM`` dump sees only the first
two; a regex over headers gets the third wrong whenever an enumerator's value is implicit or
an expression. Asking clang to evaluate ``(long long)(NAME)`` treats all three kinds
identically, for the actual target, with the TU's actual include closure — which is also the
only way arch-dependent values come out right.

HOW. A probe TU ``#include``s the real source (so the include closure and any file-local
enum are exactly the ones the translated code saw), then instantiates an undefined template
``rcv_show<(long long)(NAME)>`` once per line. clang evaluates the argument and prints it
back inside the diagnostic, so the VALUE arrives as text, attributed to a line, and a name
that is not an integer constant (a GUID, a string, a type) fails on its own line and is
reported rather than guessed at.

An earlier version emitted the values into an object file and read the bytes back. Do not
go back to it — it was tried twice and withdrawn both times. Reading ``.data`` picks up
the included TU's own statics first, and giving
the array a private section only moved the problem: at batch size the i386 object emitted
the section as ALL ZEROS while amd64 emitted it correctly, so 81 constants silently resolved
to 0 and were reported as "arch-dependent" rather than as the tooling failure they were.
Both failure modes produce plausible integers instead of an error, which is precisely what
this module must never do. A sentinel-guarded retry confirmed the section really is emitted
empty rather than being misread, so there is nothing to salvage. The diagnostic text has no
such failure mode: either clang prints a value or it does not compile.

The cost of that safety is real: a constant spelled as a POINTER cast (``IDC_ARROW`` is
``MAKEINTRESOURCE(32512)``) is a valid C initializer but not a C++ core constant expression,
so it lands in ``unresolved`` instead. That is the correct outcome — such a name is not an
integer on the Rust side either, and wants a deliberate spelling rather than a transcribed
number.

Values are resolved independently per architecture and only accepted on agreement; a name
whose value differs across targets is reported as arch-dependent instead of silently taking
whichever target ran first. That is the same rule ``crate_repair`` applies to width-dependent
casts, and for the same reason — an i386-derived value baked into a shared module is wrong
on the other two targets in a way nothing downstream will catch.
"""

from __future__ import annotations

import re
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

from .macro_authority import (
    CANONICAL_BUILD_ROOT,
    MacroAuthorityError,
    _compile_entry,
    _sibling_target_entry,
)


def _probe_entry(repo_root: Path, arch: str, tu: Path) -> list[str] | None:
    """The TU's own compile command, else a same-module sibling's with the TU substituted.

    The second half is the same fallback the c2rust anchor uses. A module that was never
    compiled in this tree (upstream's new pciidex/atapi, unbuilt since the merge) or one a
    cohab flip took out of the build has no entry of its own, and the probe used to fail for
    EVERY TU -- which left its hundreds of header constants for the model to guess.
    """
    return _compile_entry(repo_root, arch, tu) or _sibling_target_entry(repo_root, arch, tu)


_ARCHES = ("i386", "amd64", "arm64")
# clang prints the evaluated argument back in the diagnostic, attributed to its line.
_SHOW_RE = re.compile(
    r"probe\.cpp:(\d+):\d+: error: implicit instantiation of undefined template "
    r"'rcv_show<(-?\d+)>'"
)
# ...and the C TYPE, the same way: clang keeps typedef sugar in a template argument, so
# STATUS_PARTITION_FAILURE comes back as `NTSTATUS`, not `long`. A value typed by its
# magnitude alone (`i32`) does not compare against the NTSTATUS it is used with.
_TYPE_RE = re.compile(
    r"probe\.cpp:(\d+):\d+: error: implicit instantiation of undefined template "
    r"'rcv_type<([^']+)>'"
)


@dataclass(frozen=True)
class Fact:
    """One resolved constant: the value every target agreed on."""

    name: str
    value: int
    #: the C type clang reports, typedef name kept: `NTSTATUS|long` (sugar|canonical) or
    #: `int` when there is no sugar. None if not probed.
    c_type: str | None = None


@dataclass
class Resolution:
    facts: list[Fact]
    #: name -> why it could not be resolved (not an integer constant, or arch-dependent)
    unresolved: dict[str, str]


# A plain-C TU cannot be probed through C++ (its own flags -- `-std=gnu99`, `-x c` -- refuse
# a .cpp, and C-isms in its bodies do not compile as C++), and C has no templates. Two
# diagnostics carry the same facts there: a DUPLICATED case label prints its value, and an
# incompatible struct initializer prints the expression's type, typedef sugar first
# ("'NTSTATUS' (aka 'long')").
# An enumerator names itself in the message: "duplicate case value: 'X' and 'X' both equal '44'".
_C_SHOW_RE = re.compile(
    r"probe\.c:(\d+):\d+: error: duplicate case value(?: '(-?\d+)'|: .*? both equal '(-?\d+)')"
)
_C_TYPE_RE = re.compile(
    r"probe\.c:(\d+):\d+: error: initializing 'struct rcv_t' with an expression of "
    r"incompatible type '([^']+)'(?: \(aka '([^']+)'\))?"
)


def _probe_source_c(tu: Path, names: list[str]) -> str:
    """Two lines per name: its duplicated case label, then its typed initializer."""
    lines = [
        f'#include "{tu.as_posix()}"',
        "",
        "struct rcv_t { int rcv_x; };",
        "static void rcv_probe(void) {",
    ]
    for i, n in enumerate(names):
        # `0 +`: a bare enumerator label makes clang name it instead of printing its value.
        case = f"case 0 + (long long)({n}):"
        lines.append(f"    switch ((long long)0) {{ {case} {case} ; }}")
        lines.append(f"    {{ struct rcv_t t{i} = ({n}); }}")
    lines += ["}", ""]
    return "\n".join(lines)


_C_FIRST_NAME_LINE = 5  # 1:#include 2:blank 3:struct 4:fn-open 5:first name


def _probe_source(tu: Path, names: list[str]) -> str:
    """One undefined-template instantiation per name, one name per line."""
    lines = [
        f'#include "{tu.as_posix()}"',
        "",
        "template <long long V> struct rcv_show;",
        "template <typename T> struct rcv_type;",
        "static void rcv_probe(void) {",
    ]
    lines += [f"    rcv_show<(long long)({n})> v{i};" for i, n in enumerate(names)]
    lines += [f"    rcv_type<decltype({n})> t{i};" for i, n in enumerate(names)]
    lines += ["}", ""]
    return "\n".join(lines)


_FIRST_NAME_LINE = 6  # 1:#include 2:blank 3-4:templates 5:fn-open 6:first name


def _syntax_argv(argv: list[str], probe: Path) -> list[str]:
    """The TU's own compile command, retargeted at the probe, diagnostics only."""
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
        if arg.endswith((".c", ".cpp", ".cxx", ".cc")):
            continue
        out.append(arg)
    # -ferror-limit=0: every name must report, and the default cap is 20.
    # -w: the probe re-includes a whole TU, whose warnings are not this tool's business.
    return [*out, "-w", "-ferror-limit=0", "-fsyntax-only", str(probe)]


def _resolve_one_arch(
    repo_root: Path, arch: str, tu: Path, names: list[str], types: dict[str, str] | None = None
) -> tuple[dict[str, int], dict[str, str]]:
    argv = _probe_entry(repo_root, arch, tu)
    if argv is None:
        raise MacroAuthorityError(
            f"no compile_commands.json entry for {tu.name} on {arch}\n"
            f"fix: ./vibeos-cli build --arch {arch}"
        )
    is_c = tu.suffix == ".c"
    with tempfile.TemporaryDirectory() as td:
        probe = Path(td) / ("probe.c" if is_c else "probe.cpp")
        probe.write_text(_probe_source_c(tu, names) if is_c else _probe_source(tu, names))
        proc = subprocess.run(
            _syntax_argv(argv, probe),
            capture_output=True,
            text=True,
            cwd=CANONICAL_BUILD_ROOT if Path(CANONICAL_BUILD_ROOT).is_dir() else None,
        )
    values: dict[str, int] = {}
    if is_c:
        for match in _C_SHOW_RE.finditer(proc.stderr):
            index, odd = divmod(int(match.group(1)) - _C_FIRST_NAME_LINE, 2)
            if not odd and 0 <= index < len(names):
                values[names[index]] = int(match.group(2) or match.group(3))
        if types is not None:
            for match in _C_TYPE_RE.finditer(proc.stderr):
                index, odd = divmod(int(match.group(1)) - _C_FIRST_NAME_LINE, 2)
                if odd and 0 <= index < len(names):
                    sugar, canon = match.group(2).strip(), (match.group(3) or "").strip()
                    types.setdefault(names[index], f"{sugar}|{canon}" if canon else sugar)
        rejected = {n: "not an integer constant expression" for n in names if n not in values}
        return values, rejected
    for match in _SHOW_RE.finditer(proc.stderr):
        index = int(match.group(1)) - _FIRST_NAME_LINE
        if 0 <= index < len(names):
            values[names[index]] = int(match.group(2))
    if types is not None:
        for match in _TYPE_RE.finditer(proc.stderr):
            index = int(match.group(1)) - _FIRST_NAME_LINE - len(names)
            if 0 <= index < len(names):
                types.setdefault(names[index], match.group(2).strip())

    rejected = {n: "not an integer constant expression" for n in names if n not in values}
    return values, rejected


def resolve(
    repo_root: Path, tu: Path, names: list[str], arches: tuple[str, ...] = _ARCHES
) -> Resolution:
    """Resolve ``names`` against ``tu``'s real include closure on every built arch."""
    names = sorted(dict.fromkeys(names))
    per_arch: dict[str, dict[str, int]] = {}
    unresolved: dict[str, str] = {}
    types: dict[str, str] = {}
    for arch in arches:
        try:
            values, rejected = _resolve_one_arch(repo_root, arch, tu, names, types)
        except MacroAuthorityError:
            continue
        per_arch[arch] = values
        for name, why in rejected.items():
            unresolved.setdefault(name, why)
    if not per_arch:
        raise MacroAuthorityError(
            f"no architecture could be probed for {tu.name}\nfix: ./vibeos-cli build --arch i386"
        )

    facts: list[Fact] = []
    for name in names:
        seen = {a: v[name] for a, v in per_arch.items() if name in v}
        if not seen:
            unresolved.setdefault(name, "not defined in this translation unit's closure")
            continue
        distinct = set(seen.values())
        if len(distinct) > 1:
            unresolved[name] = "arch-dependent: " + ", ".join(
                f"{a}={v}" for a, v in sorted(seen.items())
            )
            continue
        facts.append(Fact(name, distinct.pop(), types.get(name)))
    return Resolution(facts, unresolved)


def rust_type(value: int) -> str:
    """The narrowest Win32-plausible Rust type that holds ``value``.

    Deliberately conservative: Win32 flag constants are unsigned, and a value that fits in
    ``u32`` is spelled ``u32`` so it composes with ``DWORD``/``UINT`` operands without a cast.
    Negative values keep a signed type — several shell constants are genuinely ``-1``.
    """
    if value < 0:
        return "i32" if value >= -(2**31) else "i64"
    if value < 2**32:
        return "u32"
    return "u64"


def render(facts: list[Fact], source_note: str) -> str:
    """Emit the resolved facts as a Rust constant block."""
    out = [f"// {source_note}"]
    for fact in facts:
        ty = rust_type(fact.value)
        literal = f"0x{fact.value:X}" if fact.value >= 10 else str(fact.value)
        if fact.value < 0:
            literal = str(fact.value)
        out.append(f"pub const {fact.name}: {ty} = {literal};")
    return "\n".join(out)


_C_FNTYPE_RE = re.compile(
    r"probe\.c:(\d+):\d+: error: initializing 'struct rcv_t' with an expression of "
    r"incompatible type '([^']+)'"
)


def function_abis(repo_root: Path, tu: Path, names: list[str]) -> dict[str, str]:
    """The Rust ABI string for each C function, from clang's own type of `&name` on i386.

    c2rust spells every extern `extern "C"`, which is wrong for an NTAPI function on i386
    (stdcall): a cdecl call to a stdcall callee leaves the stack unbalanced. On the 64-bit
    targets there is one convention, so i386 decides: `stdcall` -> "system", `cdecl` or none
    -> "C". A name clang cannot type, or another convention (fastcall), is left out --
    reported by the caller, never guessed.
    """
    names = sorted(dict.fromkeys(names))
    argv = _probe_entry(repo_root, "i386", tu)
    if argv is None:
        raise MacroAuthorityError(
            f"no compile_commands.json entry for {tu.name} on i386\n"
            "fix: ./vibeos-cli build --arch i386"
        )
    lines = [
        f'#include "{tu.as_posix()}"',
        "struct rcv_t { int rcv_x; };",
        "static void rcv_p(void) {",
    ]
    lines += [f"    {{ struct rcv_t f{i} = (&{n}); }}" for i, n in enumerate(names)]
    lines += ["}", ""]
    first = 4
    with tempfile.TemporaryDirectory() as td:
        probe = Path(td) / "probe.c"
        probe.write_text("\n".join(lines))
        proc = subprocess.run(
            _syntax_argv(argv, probe),
            capture_output=True,
            text=True,
            cwd=CANONICAL_BUILD_ROOT if Path(CANONICAL_BUILD_ROOT).is_dir() else None,
        )
    out: dict[str, str] = {}
    for m in _C_FNTYPE_RE.finditer(proc.stderr):
        index = int(m.group(1)) - first
        if not 0 <= index < len(names) or "(*)" not in m.group(2):
            continue
        ty = m.group(2)
        if "__attribute__((stdcall))" in ty:
            out[names[index]] = "system"
        elif "__attribute__((" not in ty or "__attribute__((cdecl))" in ty:
            out[names[index]] = "C"
    return out
