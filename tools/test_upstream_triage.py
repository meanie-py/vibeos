"""upstream_triage.py: which unmerged upstream commits are BUG ORACLES.

Why these tests exist: the classifier's first draft called 31 of 65 commits oracle hits,
and 18 of those were driven entirely by `modules/rostests/winetests/**` sources. Those are
not translations — this build does not compile the winetests by CONFIGURATION — so every
Wine-sync commit looked like a bug we had inherited. "Absent from the build graph" is only
evidence of translation for code the build is supposed to compile, and that distinction is
invisible in the output: both buckets just say a number.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("upstream_triage", HERE / "upstream_triage.py")
assert _spec and _spec.loader
T = importlib.util.module_from_spec(_spec)
sys.modules["upstream_triage"] = T
_spec.loader.exec_module(T)


class FakeRegistry:
    def __init__(self, vendored: set[str] | None = None) -> None:
        self._v = vendored or set()

    def is_vendored(self, rel: str) -> bool:
        return rel in self._v


def cls(rel: str, live: set[str], on_disk: bool = True, vendored: set[str] | None = None) -> str:
    return T.classify_file(rel, live, FakeRegistry(vendored), on_disk)


def test_compiled_source_is_a_plain_merge() -> None:
    assert cls("dll/win32/usp10/usp10.c", {"dll/win32/usp10/usp10.c"}) == T.MERGE


def test_uncompiled_source_still_on_disk_is_an_oracle_hit() -> None:
    """The whole point: we translated it, so an upstream fix there is our bug too."""
    assert cls("ntoskrnl/ke/i386/thrdini.c", set()) == T.ORACLE


def test_a_source_we_do_not_have_is_not_an_oracle_hit() -> None:
    """Upstream code this port never carried is new code, not a translation of ours."""
    assert cls("drivers/brand/new/thing.c", set(), on_disk=False) == T.OTHER


def test_winetests_are_not_oracle_hits() -> None:
    """The regression that motivated the bucket: 18 false ORACLEs came from here."""
    assert cls("modules/rostests/winetests/usp10/usp10.c", set()) == T.TESTS
    assert cls("modules/rosapps/applications/foo/foo.c", set()) == T.TESTS


def test_headers_are_their_own_verdict() -> None:
    """A header is never a compile edge, so it must not be read as `translated` — and it
    can be the most invasive change there is (CMHIVE gaining a UNICODE_STRING member
    moves every field after it, and our Rust binds that struct)."""
    assert cls("sdk/lib/cmlib/cmlib.h", set()) == T.HEADER
    assert cls("ntoskrnl/include/internal/cm.h", {"x"}) == T.HEADER


def test_vendored_beats_everything() -> None:
    v = {"sdk/lib/3rdparty/zlib/inflate.c"}
    assert cls("sdk/lib/3rdparty/zlib/inflate.c", set(), vendored=v) == T.VENDORED


def test_non_source_files_are_other() -> None:
    assert cls("sdk/cmake/gcc.cmake", set()) == T.OTHER
    assert cls("boot/bootdata/hivesys.inf", set()) == T.OTHER
