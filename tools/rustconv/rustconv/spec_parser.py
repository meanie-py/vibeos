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

A trailing target is TWO different things and only the DOT tells them apart, exactly as
spec2def decides it (`fIsExternal = !!ScanToken(strTarget, '.')`, spec2def.c:691):

    kernel32.FileTimeNow    dotted   -> a real forward; the .def points at another DLL and
                                        THIS module needs no symbol at all
    WLDAP32_ber_alloc_t     bare     -> a LOCAL redirection; the .def maps the export onto
                                        that symbol, which must exist in THIS module

Treating the bare form as a forward is a silent hole: wldap32 shipped a translation missing
`WLDAP32_ber_alloc_t`, `WLDAP32_ldap_first_entry` and five more, and the export audit called
them "forwarded, no implementation needed". ntoskrnl's `@ fastcall Ex...(ptr) Exf...` rows
are the same shape. Hence `impl_symbol` and `symbol` below — an auditor wants the symbol
the LINKER will look for, which is not always the export name.
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
RE_VERSION_FLAG = re.compile(r"^-version=(?P<ranges>\S+)$")

# sdk/cmake/config.cmake:87 — every module is spec2def'd at this version unless it says
# otherwise, and spec2def emits an entry only when the version falls inside its range
# (spec2def.c:1155). So `-version=0x600+` rows are NOT in our .def at all: auditing them
# reports 23 phantom MISSING exports on ws2_32 alone (the whole `Wah*` helper family).
DEFAULT_EXPORT_VERSION = 0x502


def _version_included(ranges: str, version: int) -> bool:
    """spec2def.c:1115-1164 — comma-separated `V`, `V+`, or `V-W`, all hex, `0x` optional."""
    for part in ranges.split(","):
        part = part.strip()
        if not part:
            continue
        if part.endswith("+"):
            start, end = int(part[:-1], 16), 0xFFF
        elif "-" in part:
            lo, hi = part.split("-", 1)
            start, end = int(lo, 16), int(hi, 16)
        else:
            start = end = int(part, 16)
        if start <= version <= end:
            return True
    return False


@dataclass
class SpecEntry:
    name: str
    convention: str  # "stdcall", "fastcall", "cdecl", "varargs", "extern", "stub"
    params: list[str]  # ["ptr", "long", "str", ...]
    arch_filter: list[str] | None  # None = all arches, or ["i386"], ["x86_64", "arm64"], etc.
    forward_target: str | None  # DOTTED target only: forwarded to another DLL's symbol
    is_variable: bool  # True for "@ extern" entries (data exports)
    # --- generalisation for the win32 DLL specs (defaulted: old callers unaffected) ---
    ordinal: int | None = None  # explicit ordinal, or None for "@"
    flags: list[str] = field(default_factory=list)  # raw flags, e.g. ["-stub", "-private"]
    is_stub: bool = False  # "@ stub X" or "-stub": spec2def SYNTHESISES the impl
    is_private: bool = False  # "-private": no importlib entry, but still needs an impl
    impl_symbol: str | None = None  # BARE target: the local symbol this export maps onto
    in_version: bool = True  # False when `-version=` excludes it at DEFAULT_EXPORT_VERSION

    @property
    def is_forwarded(self) -> bool:
        return self.forward_target is not None

    @property
    def symbol(self) -> str:
        """The symbol the linker must find in THIS module.

        Usually the export name, but a bare trailing target renames it — that is what
        an auditor has to look for, not `name`.
        """
        return self.impl_symbol or self.name

    @property
    def needs_implementation(self) -> bool:
        """True iff a real symbol must exist in THIS module's object code.

        False for data exports (`extern`), stubs (spec2def emits the body), entries this
        build's `-version=` excludes, DLL-forwarded entries (the .def maps them straight
        at another DLL), and `-import` entries.  A bare local target is NOT a forward —
        see the module docstring.

        On `-import`: spec2def IGNORES the flag (spec2def.c:1209, "The export is imported
        from an import library. Ignored."), so the .def gets a plain export and the LINKER
        resolves it out of an import library the module already links — the jump thunk in
        e.g. libole32.a. No symbol is needed in this module's own object code, so requiring
        a Rust fn for one is a false positive. The 2026-09-20 upstream merge rewrote
        oleaut32's Get/Set/CreateErrorInfo from `ole32.GetErrorInfo` forwarders into
        `-import` and this audit reported all three as missing.
        """
        if self.is_variable or self.is_stub or self.is_forwarded or not self.in_version:
            return False
        if "-import" in self.flags:
            return False
        return self.name != "@"  # bare "@" = autoname/noname placeholder


# spec2def.c:1611-1627 — an `-arch=` list matches when it names the target arch OR its
# WIDTH alias, and `win32`/`win64` appear in specs as often as the arch names do.
ARCH_SPEC_TOKENS: dict[str, frozenset[str]] = {
    "i386": frozenset({"i386", "win32"}),
    "amd64": frozenset({"x86_64", "win64"}),
    "x86_64": frozenset({"x86_64", "win64"}),
    "arm": frozenset({"arm", "win32"}),
    "arm64": frozenset({"arm64", "win64"}),
}


def parse_spec(path: Path = SPEC_FILE, arch: str | None = None) -> dict[str, SpecEntry]:
    """Parse a .spec file and return dict of name → SpecEntry.

    Defaults to ntoskrnl.spec (the historical behaviour).  Pass any other
    `.spec` path to parse an arbitrary DLL's export list.

    `arch` drops the rows that arch does not build, which is two fixes in one. The
    obvious half: `@ fastcall -arch=arm HalSweepIcache()` is not an i386 export and
    demanding a symbol for it is a false MISSING (hal had three). The subtler half:
    the return is keyed by NAME, so a per-arch pair — hal.spec declares
    `ExAcquireFastMutex` twice, once for arm/arm64 and once for i386 — silently
    collapses to whichever row is written last. Filtering first means the surviving
    entry is the one this arch actually compiles, instead of the one lower in the file.
    """
    entries: dict[str, SpecEntry] = {}
    want = ARCH_SPEC_TOKENS.get(arch) if arch else None

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
        # spec2def.c:691 — only a DOT makes it an inter-DLL forward. A bare target is a
        # local rename, and the symbol still has to exist here.
        target = m.group("forward")
        forward = target if target and "." in target else None
        impl_symbol = target if target and "." not in target else None

        flags = m.group("flags").split() if m.group("flags") else []

        arch_filter: list[str] | None = None
        in_version = True
        for f in flags:
            am = RE_ARCH_FLAG.match(f)
            if am:
                arch_filter = am.group("arches").split(",")
            vm = RE_VERSION_FLAG.match(f)
            if vm:
                in_version = _version_included(vm.group("ranges"), DEFAULT_EXPORT_VERSION)

        if want is not None and arch_filter and not (want & set(arch_filter)):
            continue

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
            is_stub=convention == "stub"
            or any(f == "-stub" or f.startswith("-stub=") for f in flags),
            is_private="-private" in flags,
            impl_symbol=impl_symbol,
            in_version=in_version,
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
