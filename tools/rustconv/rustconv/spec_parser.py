"""Parse a ReactOS/Wine `.spec` export list for export metadata.

Originally ntoskrnl-only; now handles ANY `.spec` in the tree (the win32 DLLs
use the ordinal-prefixed Wine dialect, not just `@`).  `parse_spec()` still
defaults to ntoskrnl's spec, so existing callers are unaffected.

Grammar accepted (matches sdk/tools/spec2def/spec2def.c):

    <ordinal|@> <convention> [-flag ...] <name>[(<params>)] [<forward target>]

    @ stdcall ExUuidCreate(ptr)                         # ntoskrnl dialect
    @ fastcall ExAcquireRundownProtection(ptr) ExfAcquireRundownProtection
    @ stdcall -arch=x86_64 ExAcquireFastMutex(ptr)
    @ extern CcFastMdlReadWait                          # DATA export
    @ varargs DbgPrint(str)
    2 stdcall SysAllocString(wstr)                      # ordinal (oleaut32)
    @ stdcall -stub CoCancelCall(long long)             # spec2def SYNTHESISES it
    @ stub CoQueryReleaseObject                         # ditto
    @ stdcall CoFileTimeNow(ptr) kernel32.FileTimeNow   # FORWARDED to another dll
    @ stdcall -private DllRegisterServer()
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

from .config import REACTOS_ROOT, SPEC_FILE

# <ordinal|@>  <convention>  [-flags...]  <name>  [(params)]  [forward.target]
RE_SPEC_LINE = re.compile(
    r"^(?P<ordinal>@|\d+)\s+"
    r"(?P<convention>\w+)"  # stdcall, cdecl, fastcall, thiscall, varargs, extern, stub
    r"(?P<flags>(?:\s+-[\w=.,+|?$-]+)*)"  # -arch=..., -stub, -private, -version=..., -ret64
    r"\s+(?P<name>[A-Za-z_?@][\w?@$]*)"  # export name (or bare @ = autoname/noname)
    r"(?:\s*\((?P<params>[^)]*)\))?"  # optional param list
    r"(?:\s+(?P<forward>[\w.$?@]+))?"  # optional forward target (dll.Symbol or Symbol)
    r"\s*$"
)

RE_ARCH_FLAG = re.compile(r"^-arch=(?P<arches>[\w,]+)$")


@dataclass
class SpecEntry:
    name: str
    convention: str  # "stdcall", "fastcall", "cdecl", "varargs", "extern", "stub"
    params: list[str]  # ["ptr", "long", "str", ...]
    arch_filter: list[str] | None  # None = all arches, or ["i386"], ["x86_64", "arm64"], etc.
    forward_target: str | None  # forwarded to another symbol name
    is_variable: bool  # True for "@ extern" entries (data exports)
    # --- generalisation for the win32 DLL specs (defaulted: old callers unaffected) ---
    ordinal: int | None = None  # explicit ordinal, or None for "@"
    flags: list[str] = field(default_factory=list)  # raw flags, e.g. ["-stub", "-private"]
    is_stub: bool = False  # "@ stub X" or "-stub": spec2def SYNTHESISES the impl
    is_private: bool = False  # "-private": no importlib entry, but still needs an impl

    @property
    def is_forwarded(self) -> bool:
        return self.forward_target is not None

    @property
    def needs_implementation(self) -> bool:
        """True iff a real symbol must exist in THIS module's object code.

        False for data exports (`extern`), stubs (spec2def emits the body), and
        forwarded entries (the .def maps them straight at another DLL).
        """
        if self.is_variable or self.is_stub or self.is_forwarded:
            return False
        return self.name != "@"  # bare "@" = autoname/noname placeholder


def parse_spec(path: Path = SPEC_FILE) -> dict[str, SpecEntry]:
    """Parse a .spec file and return dict of name → SpecEntry.

    Defaults to ntoskrnl.spec (the historical behaviour).  Pass any other
    `.spec` path to parse an arbitrary DLL's export list.
    """
    entries: dict[str, SpecEntry] = {}

    for raw in path.read_text(errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or line.startswith(";"):
            continue
        # Trailing comment AFTER an entry — ntoskrnl.spec has two, and the old
        # regex silently DROPPED both of them (`@ fastcall -arch=i386
        # KiEoiHelper(ptr) #ReactOS-Specific`).  A spec line we fail to parse is
        # an export we never audit, so never drop one quietly.
        line = re.split(r"\s+[#;]", line, maxsplit=1)[0].strip()

        m = RE_SPEC_LINE.match(line)
        if not m:
            continue

        convention = m.group("convention")
        name = m.group("name")
        params_str = m.group("params")
        forward = m.group("forward")

        flags = m.group("flags").split() if m.group("flags") else []

        arch_filter: list[str] | None = None
        for f in flags:
            am = RE_ARCH_FLAG.match(f)
            if am:
                arch_filter = am.group("arches").split(",")

        params = []
        if params_str:
            params = [p.strip() for p in params_str.split() if p.strip()]

        ord_str = m.group("ordinal")
        ordinal = int(ord_str) if ord_str != "@" else None

        entries[name] = SpecEntry(
            name=name,
            convention=convention,
            params=params,
            arch_filter=arch_filter,
            forward_target=forward,
            is_variable=convention == "extern",
            ordinal=ordinal,
            flags=flags,
            is_stub=convention == "stub" or "-stub" in flags,
            is_private="-private" in flags,
        )

    return entries


class SpecNotFound(Exception):
    """No unambiguous .spec could be located for a source file."""


def find_spec_for_source(c_path: Path, root: Path = REACTOS_ROOT) -> Path:
    """Locate the `.spec` that owns a given C/Rust source file.

    `dll/win32/oleaut32/olefont.c` → `dll/win32/oleaut32/oleaut32.spec`

    Walks up from the source file until it finds a directory containing a
    `.spec`.  Nearly every DLL dir has exactly one — but NOT all (msvfw32,
    uxtheme and winmm/midimap each ship two), so when there are several we take
    `<dirname>.spec`, which is what each of those three CMakeLists actually
    feeds to spec2def.  If that tie-break does not resolve it, FAIL LOUDLY
    rather than guess.
    """
    c_path = Path(c_path)
    start = c_path if c_path.is_dir() else c_path.parent
    root = Path(root).resolve()

    d = start.resolve()
    while True:
        specs = sorted(d.glob("*.spec"))
        if len(specs) == 1:
            return specs[0]
        if len(specs) > 1:
            preferred = d / f"{d.name}.spec"
            if preferred in specs:
                return preferred
            raise SpecNotFound(
                f"{d}: {len(specs)} .spec files and none named {d.name}.spec — "
                f"cannot pick one: {[s.name for s in specs]}"
            )
        if d == root or d == d.parent:
            raise SpecNotFound(f"no .spec found for {c_path} (searched up to {root})")
        d = d.parent


def get_exports_for_file(
    spec: dict[str, SpecEntry], function_names: list[str]
) -> dict[str, SpecEntry]:
    """Filter spec entries to only those matching the given function names."""
    return {name: spec[name] for name in function_names if name in spec}


def rust_calling_convention(entry: SpecEntry) -> str:
    """Map spec convention to Rust extern string."""
    if entry.convention == "stdcall":
        return 'extern "system"'
    elif entry.convention == "fastcall":
        # fastcall is arch-specific; on x86 it's fastcall, on x64/arm64 it's "C"
        return 'extern "fastcall"'  # caller should use cfg for i386 vs others
    elif entry.convention == "cdecl" or entry.convention == "varargs":
        return 'extern "C"'
    else:
        return 'extern "C"'


def print_spec_stats(spec: dict[str, SpecEntry]) -> None:
    """Print summary statistics."""
    funcs = sum(1 for e in spec.values() if not e.is_variable)
    vars_ = sum(1 for e in spec.values() if e.is_variable)
    forwarded = sum(1 for e in spec.values() if e.forward_target)
    arch_specific = sum(1 for e in spec.values() if e.arch_filter)

    conventions: dict[str, int] = {}
    for e in spec.values():
        conventions[e.convention] = conventions.get(e.convention, 0) + 1

    print(
        f"Spec: {funcs} functions, {vars_} variables, "
        f"{forwarded} forwarded, {arch_specific} arch-specific"
    )
    for conv, count in sorted(conventions.items()):
        print(f"  {conv}: {count}")
