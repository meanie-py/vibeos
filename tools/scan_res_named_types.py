#!/usr/bin/env python3
"""scan_res_named_types.py — a committed .res whose resource TYPE is a string naming a
predefined RT_* type.

The class: llvm-rc has no MESSAGETABLE keyword. Given windmc's `1 MESSAGETABLE "x.bin"` it
takes the word as a user-defined type NAME and emits a string-typed resource, where rc.exe
and GNU windres emit integer type 11 (RT_MESSAGETABLE). Every lookup by the real type then
misses: RtlFindMessage returned STATUS_RESOURCE_TYPE_NOT_FOUND for every message table in
the OS, FormatMessage(FROM_SYSTEM) failed everywhere, and notepad's error box read just "5".

The cmake path is fixed at the source (sdk/cmake/mc_rc_numeric_type.cmake rewrites the
generated .rc, and fails the build if a MESSAGETABLE statement survives). The COMMITTED
blobs are the other door: the cargo-linked kernel and ntdll ship .res files generated out
of band (tools/gen_*_res.py) and nothing re-runs those when a .mc changes, so a stale blob
keeps the bug with every gate green. This reads the .res bytes themselves.

Output: one line per offending (file, type). Exit 1 on findings.
"""

from __future__ import annotations

import struct
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

# winuser.h RT_* names as a resource script spells them. A STRING type equal to one of these
# is a keyword the resource compiler did not recognise.
PREDEFINED = {
    "CURSOR": 1,
    "BITMAP": 2,
    "ICON": 3,
    "MENU": 4,
    "DIALOG": 5,
    "STRING": 6,
    "STRINGTABLE": 6,
    "FONTDIR": 7,
    "FONT": 8,
    "ACCELERATORS": 9,
    "ACCELERATOR": 9,
    "RCDATA": 10,
    "MESSAGETABLE": 11,
    "GROUP_CURSOR": 12,
    "GROUP_ICON": 14,
    "VERSIONINFO": 16,
    "VERSION": 16,
    "DLGINCLUDE": 17,
    "PLUGPLAY": 19,
    "VXD": 20,
    "ANICURSOR": 21,
    "ANIICON": 22,
    "HTML": 23,
    "MANIFEST": 24,
}


def _rsrc_section(buf: bytes) -> bytes:
    """The raw bytes of the COFF object's resource section (`.rsrc` / `.rsrc$01`).

    The committed blobs are `windres -O coff` OUTPUT -- COFF objects the linker merges --
    not the `.res` record format rc.exe writes."""
    nsect, _ts, _symptr, _nsym, optsz = struct.unpack_from("<HIIIH", buf, 2)
    base = 20 + optsz
    for i in range(nsect):
        h = base + 40 * i
        name = buf[h : h + 8].rstrip(b"\0").decode("ascii", "replace")
        size, ptr = struct.unpack_from("<II", buf, h + 16)
        if name in (".rsrc", ".rsrc$01"):
            return buf[ptr : ptr + size]
    raise ValueError("no .rsrc section")


def resource_types(buf: bytes) -> list[str | int]:
    """Every top-level resource TYPE: the root IMAGE_RESOURCE_DIRECTORY's entries. A named
    entry's Name has the high bit set and points (section-relative) at a counted UTF-16
    string; an ID entry's Name is the integer type."""
    rsrc = _rsrc_section(buf)
    named, ids = struct.unpack_from("<HH", rsrc, 12)
    types: list[str | int] = []
    for i in range(named + ids):
        (name,) = struct.unpack_from("<I", rsrc, 16 + 8 * i)
        if name & 0x80000000:
            off = name & 0x7FFFFFFF
            (length,) = struct.unpack_from("<H", rsrc, off)
            types.append(rsrc[off + 2 : off + 2 + 2 * length].decode("utf-16-le"))
        else:
            types.append(name & 0xFFFF)
    return types


def scan_bytes(buf: bytes) -> list[str]:
    """The predefined names spelled as STRING types in one resource object."""
    return sorted(
        {t for t in resource_types(buf) if isinstance(t, str) and t.upper() in PREDEFINED}
    )


def committed_res() -> list[Path]:
    out = subprocess.run(
        ["git", "-C", str(REPO), "ls-files", "*.res"], capture_output=True, text=True, check=True
    ).stdout
    return [REPO / line for line in out.splitlines() if line]


def main() -> int:
    findings = []
    for path in committed_res():
        rel = path.relative_to(REPO)
        try:
            bad = scan_bytes(path.read_bytes())
        except (OSError, ValueError, struct.error) as e:
            findings.append(f"{rel}: [UNREADABLE] {e}")
            continue
        for name in bad:
            findings.append(
                f'{rel}: [NAMED_RT_TYPE] resource type is the STRING "{name}", not RT id '
                f"{PREDEFINED[name.upper()]} -- regenerate it (tools/gen_*_res.py)"
            )
    for line in findings:
        print(line)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
