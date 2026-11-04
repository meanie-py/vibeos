"""c2rust_anchor refuses a STOCK c2rust (it panics on the address of a __stdcall function)."""

from __future__ import annotations

import stat
from pathlib import Path

import pytest

from rustconv import c2rust_anchor


def _fake(tmp_path: Path, name: str, body: str) -> str:
    p = tmp_path / name
    p.write_text("#!/bin/sh\n" + body)
    p.chmod(p.stat().st_mode | stat.S_IXUSR)
    return str(p)


# argv: transpile --output-dir <dir> <src> -- ...
_PANICS = (
    "echo \"thread 'main' panicked at convert_type.rs:518:25: ctype parameter must be a "
    'function instead of Attributed" >&2\nexit 101\n'
)
_TRANSPILES = 'mkdir -p "$3/src" && echo "pub fn get() {}" > "$3/src/repro.rs"\n'


def test_a_binary_that_panics_on_the_reproducer_is_refused(tmp_path: Path) -> None:
    binary = _fake(tmp_path, "c2rust", _PANICS)
    reason = c2rust_anchor.unpatched_reason(binary)
    assert reason and "STOCK" in reason and "tools/c2rust/install.sh" in reason
    assert "instead of Attributed" in reason
    with pytest.raises(c2rust_anchor.C2RustError):
        c2rust_anchor.require_patched(binary)


def test_a_binary_that_transpiles_the_reproducer_is_accepted(tmp_path: Path) -> None:
    binary = _fake(tmp_path, "c2rust", _TRANSPILES)
    assert c2rust_anchor.unpatched_reason(binary) is None
    c2rust_anchor.require_patched(binary)


def test_reinstalling_in_place_is_probed_again(tmp_path: Path) -> None:
    binary = _fake(tmp_path, "c2rust", _PANICS)
    assert c2rust_anchor.unpatched_reason(binary)
    _fake(tmp_path, "c2rust", _TRANSPILES + "# a different size, so a different key\n")
    assert c2rust_anchor.unpatched_reason(binary) is None


def test_the_reproducer_ships_in_the_tree() -> None:
    text = c2rust_anchor._STDCALL_REPRO.read_text()
    assert "__attribute__((stdcall))" in text and "return f;" in text
