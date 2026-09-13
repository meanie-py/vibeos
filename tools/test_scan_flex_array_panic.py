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
    """(symbol, length) pairs — the context column is exercised separately below."""
    monkeypatch.setattr(s.subprocess, "run", lambda *a, **k: _Proc(listing))
    return [(sym, ln) for sym, ln, _ in s.scan_object(Path("fake.a"))]


def _run_explain(monkeypatch, listing: str):
    monkeypatch.setattr(s.subprocess, "run", lambda *a, **k: _Proc(listing))
    return s.scan_object(Path("fake.a"), explain=True)


# The REAL shape. `panic_bounds_check(index, len, &Location)` under cdecl emits
# `push loc; push len; push index`, so the length is the MIDDLE push. The original
# fixture here had only two pushes and encoded the same wrong assumption as the code it
# was testing -- which is precisely why keying on the last push (the INDEX) survived
# review: the test could not tell the two apart.
FLEX = """
00001000 <_Victim@8>:
    1000: 68 70 05 00 00               \tpushl\t$0x570                  # imm = 0x570
\t\t\t00001001:  IMAGE_REL_I386_DIR32\t.rdata
    1005: 6a 01                        \tpushl\t$0x1
    1007: 6a 01                        \tpushl\t$0x1
    1009: e8 00 00 00 00               \tcalll\t0x100e
\t\t\t0000100a:  IMAGE_REL_I386_REL32\t__ZN4core9panicking18panic_bounds_checkE
"""

# A real fixed array: len 2, so index 1 is in range and index 2 is an ordinary
# off-the-end, not a bindgen flexible array.
REAL = FLEX.replace("pushl\t$0x1\n    1007", "pushl\t$0x2\n    1007").replace(
    "_Victim@8", "_Innocent@8"
)

# The length in a REGISTER: a runtime-length slice. A [T; 1] cannot have a dynamic
# length, so this fails the grading protocol's "is the declared length a LITERAL 1"
# test by construction and must not be reported.
DYNAMIC_LEN = FLEX.replace(
    "    1005: 6a 01                        \tpushl\t$0x1\n",
    "    1005: 50                           \tpushl\t%eax\n",
).replace("_Victim@8", "_DynamicLen@8")


def test_a_length_one_check_is_reported(monkeypatch):
    assert _run(monkeypatch, FLEX) == [("_Victim@8", 1)]


def test_a_real_fixed_array_is_left_alone(monkeypatch):
    """SubKeyLists[Type] on [T; 2] is a correct check, not a flexible array."""
    assert _run(monkeypatch, REAL) == []


def test_a_runtime_length_slice_is_not_a_flexible_array(monkeypatch):
    """Measured over the i386 corpus, keying on the INDEX reported five of these --
    sending graders to rtl_strings functions that index a plain slice and declare no
    array at all."""
    assert _run(monkeypatch, DYNAMIC_LEN) == []


def test_an_unrelated_instruction_between_does_not_carry_the_length(monkeypatch):
    """An instruction that is not part of the call setup clears the pending arguments,
    so a length from some earlier, unrelated computation cannot be attributed here."""
    listing = FLEX.replace(
        "    1009: e8 00 00 00 00               \tcalll\t0x100e\n",
        "    1008: 90                           \tnop\n"
        "    1009: e8 00 00 00 00               \tcalll\t0x100e\n",
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


def test_context_is_collected_only_when_asked(monkeypatch):
    """The corpus-wide default run must not carry windows it will never print."""
    assert _run_explain(monkeypatch, FLEX)[0][2]
    monkeypatch.setattr(s.subprocess, "run", lambda *a, **k: _Proc(FLEX))
    assert s.scan_object(Path("fake.a"))[0][2] == []


def test_context_names_what_the_check_was_about(monkeypatch):
    """WHY --explain exists: the flagged symbol is where the check was INLINED TO, so
    its own source may declare no array at all. The relocation in the window is the
    only place the real array is named."""
    listing = FLEX.replace(
        "    1005: 6a 01                        \tpushl\t$0x1\n",
        "    1004: 8d 3d 00 00 00 00            \tleal\t0x0, %edi\n"
        "\t\t\t00001005:  IMAGE_REL_I386_DIR32\t_ReparseBuffer_PathBuffer\n"
        "    1005: 6a 01                        \tpushl\t$0x1\n",
    )
    ctx = _run_explain(monkeypatch, listing)[0][2]
    assert any("_ReparseBuffer_PathBuffer" in line for line in ctx)


def test_the_window_does_not_span_symbols(monkeypatch):
    """A window that leaked across a symbol boundary would name another function's
    array and send the grader to the wrong file."""
    listing = (
        "00000900 <_Neighbour@4>:\n"
        "\t\t\t00000901:  IMAGE_REL_I386_DIR32\t_NOT_THIS_ONE\n" + FLEX.lstrip("\n")
    )
    ctx = _run_explain(monkeypatch, listing)[0][2]
    assert not any("_NOT_THIS_ONE" in line for line in ctx)


def test_a_large_immediate_with_objdumps_trailing_comment_is_read(monkeypatch):
    """objdump renders the 5-byte push as `pushl $0x570  # imm = 0x570`. Anchoring the
    length regex on end-of-line skipped every large immediate -- and the &Location
    argument is always one, so five sites reported no source location at all."""
    assert _run(monkeypatch, FLEX) == [("_Victim@8", 1)]
