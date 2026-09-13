"""The flexible-array panic reader: length discrimination and corpus honesty."""

from __future__ import annotations

import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scan_flex_array_panic as s


class _Proc:
    def __init__(self, stdout: str) -> None:
        self.stdout, self.stderr, self.returncode = stdout, "", 0


def _run(monkeypatch, listing: str):
    monkeypatch.setattr(s.subprocess, "run", lambda *a, **k: _Proc(listing))
    return s.scan_object(Path("fake.a"))


FLEX = """
00001000 <_Victim@8>:
    1000: 6a 01                        \tpushl\t$0x1
    1002: 57                           \tpushl\t%edi
    1003: e8 00 00 00 00               \tcalll\t0x1008
\t\t\t00001004:  IMAGE_REL_I386_REL32\t__ZN4core9panicking18panic_bounds_checkE
"""

REAL = FLEX.replace("$0x1\n", "$0x2\n").replace("_Victim@8", "_Innocent@8")


def test_a_length_one_check_is_reported(monkeypatch):
    assert _run(monkeypatch, FLEX) == [("_Victim@8", 1)]


def test_a_real_fixed_array_is_left_alone(monkeypatch):
    """SubKeyLists[Type] on [T; 2] is a correct check, not a flexible array."""
    assert _run(monkeypatch, REAL) == []


def test_an_unrelated_push_between_does_not_carry_the_length(monkeypatch):
    listing = FLEX.replace(
        "    1002: 57                           \tpushl\t%edi\n",
        "    1002: 50                           \tpushl\t%eax\n"
        "    1003: 90                           \tnop\n",
    )
    assert _run(monkeypatch, listing) == []


def test_the_length_does_not_leak_across_symbols(monkeypatch):
    listing = (
        "00001000 <_Pusher@4>:\n    1000: 6a 01                        \tpushl\t$0x1\n"
        "00002000 <_Other@4>:\n"
        "    2000: e8 00 00 00 00               \tcalll\t0x2005\n"
        "\t\t\t00002001:  IMAGE_REL_I386_REL32\t__ZN4core9panicking18panic_bounds_checkE\n"
    )
    assert _run(monkeypatch, listing) == []


def test_non_i386_refuses_rather_than_grading_nothing(capsys):
    assert s.main(["--arch", "arm64"]) == 2
    assert "i386-only" in capsys.readouterr().err


def test_the_corpus_comes_from_the_workspace_and_is_not_empty():
    corpus = s.declared_corpus()
    assert len(corpus) > 100
    assert "cmlib-rust" in corpus


def test_a_crate_target_dir_is_not_mistaken_for_the_artifact(tmp_path):
    real = tmp_path / "sdk" / "lib" / "cmlib" / "libcmlib_rust.a"
    inter = tmp_path / "sdk" / "lib" / "cmlib" / "cmlib-rust-target" / "libcmlib_rust.a"
    for p in (real, inter):
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(b"")
    assert s.find_artifact(tmp_path, "cmlib-rust") == real


def test_the_crate_target_dir_is_PRUNED_not_filtered_afterwards(tmp_path, monkeypatch):
    """Selecting the right artifact is not the same as never entering the wrong tree.

    The old spelling was a recursive `rglob(want)` rooted at the build dir, with an
    `if "-target/" not in str(p)` filter after it — which passes the test above while
    still walking every file under the one
    directory a concurrent build is rewriting — and pathlib's recursive glob lets a
    FileNotFoundError out of the generator, so the scanner dies with a traceback that
    names no source defect. Assert the descent, not just the answer.
    """
    real = tmp_path / "sdk" / "lib" / "cmlib" / "libcmlib_rust.a"
    target = tmp_path / "sdk" / "lib" / "cmlib" / "cmlib-rust-target" / "deep" / "deeper"
    real.parent.mkdir(parents=True, exist_ok=True)
    real.write_bytes(b"")
    target.mkdir(parents=True)
    (target / "libcmlib_rust.a").write_bytes(b"")

    seen: list[str] = []
    real_walk = os.walk

    def spy(top, *a, **kw):
        for dirpath, dirnames, filenames in real_walk(top, *a, **kw):
            seen.append(dirpath)
            yield dirpath, dirnames, filenames

    monkeypatch.setattr(s.os, "walk", spy)
    assert s.find_artifact(tmp_path, "cmlib-rust") == real
    # The spy must have run: an rglob implementation calls os.walk zero times, and
    # "no -target in an empty list" would then be true for the wrong reason.
    assert seen, "os.walk was never called — this assertion would pass vacuously"
    assert not any("-target" in p for p in seen), f"descended into a pruned tree: {seen}"
