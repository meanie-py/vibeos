"""Tests for scan_i386_sse_abi — the i386 no-SSE ABI time-bomb tripwire.

The load-bearing assertion is `test_pinned_rustc_still_only_warns`: it compiles a
trivial crate with the exact doomed combo (`-C target-feature=-sse,-sse2` on the
builtin `i686-pc-windows-gnu`) and FAILS LOUDLY the moment the pinned rustc turns
rust-lang#116344's phase-out warning into a hard error. That is the toolchain bump
firing, and the failure message carries the migration so the bumper does not have to
reconstruct it from 54 cryptic build errors. It skips (does not fail) where rustc or
the i686 target is not installed, so it is meaningful on the build fleet and inert
elsewhere.

The detection tests pin the "doomed combo" definition so a crate already moved to a
softfloat json is NOT counted, and a future crate added with the builtin triple IS.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scan_i386_sse_abi as mod


def test_pinned_rustc_still_only_warns():
    """The time bomb has not gone off: the doomed flag+triple still compiles (exit 0)."""
    probe = mod.probe_rustc()
    if probe.verdict == mod.UNAVAILABLE:
        pytest.skip(f"rustc / i686 target unavailable: {probe.stderr[:120]}")
    assert probe.verdict != mod.TRIPPED, (
        "The pinned rustc now makes `-C target-feature=-sse,-sse2` on the builtin "
        "i686-pc-windows-gnu target a HARD ERROR (rust-lang#116344). All 54 i386 "
        "crates will fail to build. Migrate them to a softfloat custom-target json "
        "(rustc-abi: x86-softfloat) built with `-Z build-std`, the model the i386 "
        "*-cargo cells already use (i686-kernel.json + add_cargo_module). See the "
        "roadmap entry and `python tools/scan_i386_sse_abi.py` for the full list.\n\n"
        f"rustc stderr:\n{probe.stderr}"
    )
    # Today it is one of: still warning (expected), or the ABI changed upstream
    # (informational — re-evaluate the flag). Both are non-fatal.
    assert probe.verdict in (mod.OK_LATENT, mod.ABI_CHANGED)


def test_detection_still_finds_the_doomed_cmakelists():
    """The combo exists in-tree (regression guard against the detector silently zeroing)."""
    doomed = mod.find_doomed_cmakelists(mod.repo_root())
    assert len(doomed) >= 1
    # every hit genuinely carries both the flag and the builtin triple
    for p in doomed:
        text = p.read_text(errors="replace")
        assert mod.SSE_FLAG in text and mod.BUILTIN_TRIPLE in text


def test_detector_excludes_a_softfloat_json_crate(tmp_path):
    """A crate already on the softfloat json is SAFE and must not be counted."""
    root = tmp_path
    (root / "AGENTS.md").write_text("")
    (root / "sdk").mkdir()
    safe = root / "sdk" / "foo"
    safe.mkdir()
    (safe / "CMakeLists.txt").write_text(
        "cargo build --target i686-kernel.json -C target-feature=-sse,-sse2 i686-pc-windows-gnu\n"
    )
    assert mod.find_doomed_cmakelists(root) == []


def test_detector_flags_builtin_triple_crate(tmp_path):
    """The builtin-triple + flag + no-json combo IS flagged."""
    root = tmp_path
    (root / "AGENTS.md").write_text("")
    (root / "sdk").mkdir()
    doomed = root / "sdk" / "bar"
    doomed.mkdir()
    cml = doomed / "CMakeLists.txt"
    cml.write_text(
        'set(X "RUSTFLAGS=-C target-feature=-sse,-sse2")\n'
        "cargo build --target i686-pc-windows-gnu\n"
    )
    assert mod.find_doomed_cmakelists(root) == [cml]
