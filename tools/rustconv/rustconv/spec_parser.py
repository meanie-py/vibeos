"""Parse ntoskrnl.spec for export metadata."""

import re
from dataclasses import dataclass
from pathlib import Path

from .config import SPEC_FILE

# Format: @ <convention> [-arch=<arch>] <name>(<params>) [<forward_target>]
# Examples:
#   @ stdcall ExUuidCreate(ptr)
#   @ fastcall ExAcquireRundownProtection(ptr) ExfAcquireRundownProtection
#   @ stdcall -arch=x86_64 ExAcquireFastMutex(ptr)
#   @ extern CcFastMdlReadWait
#   @ varargs DbgPrint(str)
RE_SPEC_LINE = re.compile(
    r"^@\s+"
    r"(\w+)"  # convention: stdcall, fastcall, cdecl, varargs, extern
    r"(?:\s+-arch=([\w,]+))?"  # optional arch filter
    r"\s+(\w+)"  # export name
    r"(?:\(([^)]*)\))?"  # optional param list
    r"(?:\s+(\w+))?"  # optional forward target
    r"\s*$"
)


@dataclass
class SpecEntry:
    name: str
    convention: str  # "stdcall", "fastcall", "cdecl", "varargs", "extern"
    params: list[str]  # ["ptr", "long", "str", ...]
    arch_filter: list[str] | None  # None = all arches, or ["i386"], ["x86_64", "arm64"], etc.
    forward_target: str | None  # forwarded to another symbol name
    is_variable: bool  # True for "@ extern" entries (data exports)


def parse_spec(path: Path = SPEC_FILE) -> dict[str, SpecEntry]:
    """Parse ntoskrnl.spec and return dict of name → SpecEntry."""
    entries: dict[str, SpecEntry] = {}

    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue

        m = RE_SPEC_LINE.match(line)
        if not m:
            continue

        convention = m.group(1)
        arch_str = m.group(2)
        name = m.group(3)
        params_str = m.group(4)
        forward = m.group(5)

        arch_filter = None
        if arch_str:
            arch_filter = arch_str.split(",")

        params = []
        if params_str:
            params = [p.strip() for p in params_str.split() if p.strip()]

        is_variable = convention == "extern"

        entries[name] = SpecEntry(
            name=name,
            convention=convention,
            params=params,
            arch_filter=arch_filter,
            forward_target=forward,
            is_variable=is_variable,
        )

    return entries


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
