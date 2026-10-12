"""The Windows version macros a module is compiled at: `WINVER`, `_WIN32_WINNT`,
`NTDDI_VERSION`, and the `NTDDI_*` / `_WIN32_WINNT_*` names sdkddkver.h defines.

`#if (NTDDI_VERSION >= NTDDI_VISTA)` has a macro on both sides. Without these values every
such conditional is undecidable, so the scan showed the model both arms and body_audit
read Vista/Win8 blocks as live. The arbiter library is the case that cost: 22 functions
came back from the model built on the Vista arm of a pre-Vista build, and each was fixed
by hand. One table, used by both, so the scan and the audit decide the same way.

The tree pins `-D_WIN32_WINNT=0x502` (top-level CMakeLists), which sdkddkver.h turns into
NTDDI_WS03SP1; a module's own `-D_WIN32_WINNT=` / `-DNTDDI_VERSION=` overrides it.
"""

from __future__ import annotations

import re
from pathlib import Path

_VERSION_ALIAS_RE = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+((?:NTDDI|_WIN32_WINNT|_WIN32_IE)_\w+)[ \t]+"
    r"((?:NTDDI|_WIN32_WINNT|_WIN32_IE)_\w+)[ \t]*(?://.*)?$",
    re.M,
)
_VERSION_DEFINE_RE = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+((?:NTDDI|_WIN32_WINNT|_WIN32_IE)_\w+)[ \t]+(0[xX][0-9a-fA-F]+)",
    re.M,
)
# `-D_WIN32_WINNT=0x600` on a compile line, or `_WIN32_WINNT=0x600` inside
# target_compile_definitions (spelled without the -D).
_WINNT_RE = re.compile(r"(?:-D|\b)_WIN32_WINNT=(0[xX][0-9a-fA-F]+)")
_NTDDI_RE = re.compile(r"(?:-D|\b)NTDDI_VERSION=(0[xX][0-9a-fA-F]+)")


def repo_root_of(path: Path) -> Path | None:
    """The checkout holding *path*: the nearest ancestor with sdk/include/psdk/sdkddkver.h."""
    for d in (path if path.is_dir() else path.parent, *path.parents):
        if (d / "sdk" / "include" / "psdk" / "sdkddkver.h").is_file():
            return d
    return None


def version_defines(module_dir: Path, repo: Path) -> dict[str, int]:
    pins: dict[str, int] = {}
    ddk = repo / "sdk" / "include" / "psdk" / "sdkddkver.h"
    if ddk.is_file():
        text = ddk.read_text(errors="replace")
        for name, value in _VERSION_DEFINE_RE.findall(text):
            pins.setdefault(name, int(value, 16))
        # `#define NTDDI_VISTA NTDDI_WIN6`: an alias guards fastfat's FatOpenVolume Vista block.
        for name, target in _VERSION_ALIAS_RE.findall(text):
            if target in pins:
                pins.setdefault(name, pins[target])
    try:
        cml = (module_dir / "CMakeLists.txt").read_text(errors="replace")
    except OSError:
        cml = ""
    cml = re.sub(r"#[^\n]*", "", cml)  # a commented-out override is not one
    winnt = _WINNT_RE.search(cml)
    ntddi = _NTDDI_RE.search(cml)
    pins["_WIN32_WINNT"] = int(winnt.group(1), 16) if winnt else 0x502
    pins["WINVER"] = pins["_WIN32_WINNT"]
    if ntddi:
        pins["NTDDI_VERSION"] = int(ntddi.group(1), 16)
    elif pins["_WIN32_WINNT"] == 0x502:
        pins["NTDDI_VERSION"] = pins.get("NTDDI_WS03SP1", 0x05020100)  # sdkddkver.h:153
    else:
        pins["NTDDI_VERSION"] = pins["_WIN32_WINNT"] << 16
    return pins
