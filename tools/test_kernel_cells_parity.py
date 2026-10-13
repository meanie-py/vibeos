"""The UP and MP kernels (ntoskrnl-cargo, ntkrnlmp-cargo) are two link cells for one body,
and each names the Rust libraries it links by hand. They drifted once: the utf8 crate went
into ntoskrnl's link only, the build stayed green on the UP kernel, and ntkrnlmp failed with
RtlUTF8ToUnicodeN undefined. Both lists must name the same crates."""

from __future__ import annotations

import re
from pathlib import Path

RUST = Path(__file__).resolve().parent.parent / "sdk" / "lib" / "rust"
RUST_LIB = re.compile(r'\.rust_lib\(\s*"([^"]+)"')


def _crates(cell: str) -> set[str]:
    return set(RUST_LIB.findall((RUST / cell / "build.rs").read_text()))


def test_up_and_mp_kernels_link_the_same_rust_libraries() -> None:
    up, mp = _crates("ntoskrnl-cargo"), _crates("ntkrnlmp-cargo")
    assert up, "no .rust_lib(...) found in ntoskrnl-cargo/build.rs: this test reads nothing"
    assert up == mp, f"only UP: {sorted(up - mp)}; only MP: {sorted(mp - up)}"
