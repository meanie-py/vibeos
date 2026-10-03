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


def cls(
    rel: str,
    live: set[str],
    on_disk: bool = True,
    vendored: set[str] | None = None,
    disabled: frozenset[str] = frozenset(),
) -> str:
    return T.classify_file(rel, live, FakeRegistry(vendored), on_disk, disabled)


def test_compiled_source_is_a_plain_merge() -> None:
    assert cls("dll/win32/usp10/usp10.c", {"dll/win32/usp10/usp10.c"}) == T.MERGE


def test_uncompiled_source_a_cmake_comment_disables_is_an_oracle_hit() -> None:
    """The whole point: we translated it, so an upstream fix there is our bug too."""
    f = "ntoskrnl/ke/i386/thrdini.c"
    assert cls(f, set(), disabled=frozenset({f})) == T.ORACLE


def test_uncompiled_source_nothing_disables_is_unbuilt_not_oracle() -> None:
    """Dead C with no disabling comment was never built by this port, so there is no
    Rust to read it against. Counting these as oracle hits made the already-merged
    range read as 465."""
    assert cls("drivers/bus/pcix/pci/ecam.c", set()) == T.UNBUILT


def _exists(paths: set[str]):
    return lambda rel: rel in paths


def test_comment_resolution_handles_the_generator_expression_spelling() -> None:
    """ntos.cmake disables pnpmgr.c inside a generator expression. The cohab registry
    missed exactly this, and pnpmgr.c is the translated file whose upstream fix
    (a7f658e3221) went dead on arrival and cost a day."""
    line = "    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpmgr.c>"
    f = "ntoskrnl/io/pnpmgr/pnpmgr.c"
    assert T.resolve_comment_sources("ntoskrnl/ntos.cmake", line, _exists({f})) == {f}


def test_comment_resolution_relative_and_current_dir_spellings() -> None:
    exists = _exists({"hal/halx86/generic/beep.c", "drivers/hid/hidclass/pdo.c"})
    assert T.resolve_comment_sources(
        "hal/halx86/generic.cmake",
        "    # generic/beep.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/beep.rs)",
        exists,
    ) == {"hal/halx86/generic/beep.c"}
    assert T.resolve_comment_sources(
        "drivers/hid/hidclass/CMakeLists.txt",
        "    #${CMAKE_CURRENT_SOURCE_DIR}/pdo.c  # replaced by Rust",
        exists,
    ) == {"drivers/hid/hidclass/pdo.c"}


def test_comment_resolution_ignores_names_that_do_not_exist() -> None:
    """Prose mentions a .rs-adjacent name or a file long gone; only real paths count."""
    line = "# mouhid.c moved to sdk/lib/rust/mouhid-rust (see old/gone.c)"
    assert (
        T.resolve_comment_sources("drivers/hid/mouhid/CMakeLists.txt", line, _exists(set()))
        == set()
    )


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


def test_module_level_flip_catches_prose_comments() -> None:
    """usbport's CMakeLists describes its flip as `(debug/device/.../roothub/...` -- no
    resolvable file name -- and the exact-name rule filed a real usbport fix (roothub.c,
    da13f3ca820) as UNBUILT. The nearest CMakeLists mentioning Rust must be enough."""
    idx = T.DisabledIndex(
        exact=set(),
        cmake_dirs={"drivers/usb/usbport", "drivers/bus/newthing", "drivers"},
        rust_dirs={"drivers/usb/usbport", "drivers"},
    )
    assert "drivers/usb/usbport/roothub.c" in idx
    # the NEAREST CMakeLists decides: a module with its own non-Rust CMakeLists is not
    # flipped just because a parent directory's CMakeLists mentions Rust somewhere
    assert "drivers/bus/newthing/x.c" not in idx
    assert cls("drivers/usb/usbport/roothub.c", set(), disabled=idx) == T.ORACLE
    assert cls("drivers/bus/newthing/x.c", set(), disabled=idx) == T.UNBUILT


def test_exact_disable_wins_even_without_a_rust_comment_nearby() -> None:
    """pnpmgr.c has no CMakeLists of its own; ntos.cmake disables it by path."""
    f = "ntoskrnl/io/pnpmgr/pnpmgr.c"
    idx = T.DisabledIndex(exact={f}, cmake_dirs={"ntoskrnl"}, rust_dirs=set())
    assert f in idx
