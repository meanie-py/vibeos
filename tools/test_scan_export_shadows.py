"""scan_export_shadows: which spec lines are really in the kernel's export table."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_export_shadows import _version_included


def test_unversioned_line_is_exported() -> None:
    assert _version_included(["-arch=x86_64"])


def test_a_later_os_export_is_not_in_a_502_kernel() -> None:
    """ntoskrnl_vista's PoFx* are `-version=0x602+`: the static lib is the only copy."""
    assert not _version_included(["-version=0x602+"])
    assert not _version_included(["-version=0xA00+"])


def test_ranges_and_lists_follow_spec2def() -> None:
    assert _version_included(["-version=0x400-0x502"])
    assert not _version_included(["-version=0x400-0x501"])
    assert _version_included(["-version=0x600+,0x502"])
    assert _version_included(["-version=0x502+"], version=0x502)
