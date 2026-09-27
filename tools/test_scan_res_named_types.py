"""scan_res_named_types: a resource TYPE spelled as a string that names an RT_* type."""

from __future__ import annotations

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import scan_res_named_types as srn


def _coff_rsrc(types: list[str | int]) -> bytes:
    """A minimal `windres -O coff` shaped object: one .rsrc section holding a root
    IMAGE_RESOURCE_DIRECTORY with the given top-level types (named entries first, as the
    format requires), string names stored after the entry table."""
    named = [t for t in types if isinstance(t, str)]
    ids = [t for t in types if isinstance(t, int)]
    table_end = 16 + 8 * len(types)
    entries, strings = b"", b""
    for name in named:
        off = table_end + len(strings)
        entries += struct.pack("<II", 0x80000000 | off, 0)
        strings += struct.pack("<H", len(name)) + name.encode("utf-16-le")
    for ident in ids:
        entries += struct.pack("<II", ident, 0)
    rsrc = struct.pack("<IIHHHH", 0, 0, 0, 0, len(named), len(ids)) + entries + strings
    header = struct.pack("<HHIIIHH", 0x14C, 1, 0, 0, 0, 0, 0)
    sect_ptr = 20 + 40
    section = b".rsrc$01" + struct.pack("<IIIIIIHHI", 0, 0, len(rsrc), sect_ptr, 0, 0, 0, 0, 0)
    return header + section + rsrc


def test_string_messagetable_is_reported():
    """The shipped shape: llvm-rc turned `1 MESSAGETABLE "x.bin"` into a NAMED type."""
    assert srn.scan_bytes(_coff_rsrc(["MESSAGETABLE", 16])) == ["MESSAGETABLE"]


def test_integer_messagetable_is_clean():
    """What rc.exe/GNU windres emit, and what the rewrite now feeds llvm-rc: type 11."""
    assert srn.scan_bytes(_coff_rsrc([2, 11, 16])) == []


def test_a_genuinely_custom_string_type_is_not_a_finding():
    """User-defined string types are legal (e.g. "WAVE"); only RT_* spellings are the bug."""
    assert srn.scan_bytes(_coff_rsrc(["WAVE", 16])) == []


def test_types_are_read_in_directory_order():
    assert srn.resource_types(_coff_rsrc(["MESSAGETABLE", 2, 16])) == ["MESSAGETABLE", 2, 16]
