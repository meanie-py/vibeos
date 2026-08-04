"""One C++ TU needs several translate runs, and they all derive the same output path.

`--free-functions` covers a TU's free helpers; `--cpp-com --class X` covers one class. Both
name their output after the TU's stem, so the second run used to overwrite the first with no
diagnostic at all -- it printed "Assembled to ..." and looked like a success. That happened on
folders/CNetFolder.cpp: the CNetFolderEnum assembly was replaced by the free-function assembly,
and nothing said so. The paid artifacts survive in .responses/, so the cost is time, not money,
but the failure is silent and the file it leaves behind is missing symbols the caller needs.
"""

from __future__ import annotations

import pytest

from rustconv.assembler import (
    ASSEMBLY_MARKER,
    AssemblyClobberError,
    _guard_assembly_target,
)


def _write(tmp_path, name, selector, body="pub fn f() {}\n"):
    p = tmp_path / name
    head = f"{ASSEMBLY_MARKER}{selector}\n" if selector else ""
    p.write_text(head + body)
    return p


def test_refuses_a_different_slice_of_the_same_tu(tmp_path):
    p = _write(tmp_path, "CNetFolder.rs", "class:CNetFolderEnum")
    with pytest.raises(AssemblyClobberError) as exc:
        _guard_assembly_target(p, "free-functions")
    msg = str(exc.value)
    assert "class:CNetFolderEnum" in msg
    assert "free-functions" in msg
    # The message has to say what to DO, or it just relocates the confusion.
    assert "-o" in msg


def test_allows_rerunning_the_same_slice(tmp_path):
    p = _write(tmp_path, "CNetFolder.rs", "class:CNetFolderEnum")
    _guard_assembly_target(p, "class:CNetFolderEnum")


def test_allows_a_fresh_path(tmp_path):
    _guard_assembly_target(tmp_path / "does_not_exist.rs", "free-functions")


def test_refuses_a_partial_slice_over_an_unmarked_file(tmp_path):
    """An unmarked file is the DANGEROUS case, not the safe one.

    dialogs/drvdefext.rs held a fully translated CDrvDefExt class with no marker, and a
    --free-functions run replaced all 1,742 lines of it. Nothing can tell from the outside
    what an unmarked file covers, so a partial slice has to ask.
    """
    p = _write(tmp_path, "drvdefext.rs", selector="")
    with pytest.raises(AssemblyClobberError) as exc:
        _guard_assembly_target(p, "free-functions")
    assert "-o" in str(exc.value)


def test_whole_tu_may_overwrite_an_unmarked_file(tmp_path):
    """A whole-TU run is not a partial slice, so it cannot drop part of the TU."""
    p = _write(tmp_path, "legacy.rs", selector="")
    _guard_assembly_target(p, "whole-tu")


def test_no_selector_disables_the_guard(tmp_path):
    """Targets that do not pass a selector keep their old behaviour rather than half-guarding."""
    p = _write(tmp_path, "CNetFolder.rs", "class:CNetFolderEnum")
    _guard_assembly_target(p, "")
