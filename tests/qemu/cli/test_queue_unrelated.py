"""A red stage the batch cannot reach must not convict the batch.

The fixture in `REAL_*` is not invented: it is the verbatim salient line and file list from
the 2026-09-13 rejection of `warnings-burndown@9a47bb69dedc`, which was told it had caused a
`towlower` throughput regression in msvcrt while its diff contained no msvcrt file at all.
"""

from pathlib import Path

import pytest

from tests.qemu.cli import queue
from tests.qemu.cli.queue import (
    GateVerdict,
    Submission,
    _failed_substages,
    _harness_closure,
    _touches,
    unrelated_red,
)

REPO = Path(__file__).resolve().parents[3]

REAL_SALIENT = (
    "✓ drift:consts   ✓ drift:protos:file   ✓ drift:protos:process   ✓ drift:protos:time   "
    "✓ drift:layout:seh-i386   ✓ cnt-difftest   ✗ msvcrt-difftest(exit 1)   "
    "✓ msvcrt-scanf-difftest   ✓ msvcrt-seh-difftest   ✓ portcls-difftest   ✓ rtl-difftest   "
    "✓ sysaudio-difftest"
)
REAL_BATCH = {
    "sdk/lib/rust/cdfs-rust/src/deviosup.rs",
    "sdk/lib/rust/disk-rust/src/disk.rs",
    "sdk/lib/rust/disk-rust/src/diskwmi.rs",
    "sdk/lib/rust/fastfat-rust/src/verfysup.rs",
    "sdk/lib/rust/freeldr-rust/src/frldr.rs",
    "sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/kiinit.rs",
    "sdk/lib/rust/ntoskrnl-rust/src/ke/i386/traphdlr.rs",
    "tools/lint-baseline/scan_const_contradictions.txt",
}


def _verdict(stages, salient=REAL_SALIENT):
    return GateVerdict(
        ok=False,
        classification="DIFFTEST_FAIL",
        artifact="/tmp/x",
        salient=salient,
        stages=stages,
    )


@pytest.fixture
def paths(monkeypatch):
    """Drive `unrelated_red`'s file set without needing a real integration worktree."""
    box = {"value": REAL_BATCH}
    monkeypatch.setattr(queue, "_batch_paths", lambda wt, tip: box["value"])
    return box


def test_the_real_rejection_would_now_be_excused(paths):
    assert unrelated_red(REPO, Path("/nonexistent"), "tip", _verdict({"difftest": "fail"}))


def test_a_batch_that_touches_the_harness_closure_is_still_judged(paths):
    for touching in (
        "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/locale.rs",  # a path-dep of the harness
        "sdk/lib/rust/msvcrt-difftest/tests/perf.rs",  # the harness itself
        "dll/win32/msvcrt/string.c",  # C the harness's build.rs compiles
    ):
        paths["value"] = REAL_BATCH | {touching}
        assert not unrelated_red(
            REPO, Path("/nonexistent"), "tip", _verdict({"difftest": "fail"})
        ), touching


def test_unknown_reach_means_guilty(paths):
    # A sub-stage of the difftest DRIVER is not a crate, so its reach is underivable.
    assert not unrelated_red(
        REPO,
        Path("/nonexistent"),
        "tip",
        _verdict({"difftest": "fail"}, "✗ drift:consts(exit 1)"),
    )
    # An unparseable salient says only "red", which is not enough to say red ABOUT WHAT.
    assert not unrelated_red(
        REPO,
        Path("/nonexistent"),
        "tip",
        _verdict({"difftest": "fail"}, "it broke"),
    )
    # git declining to answer must not be read as "disjoint".
    paths["value"] = None
    assert not unrelated_red(REPO, Path("/nonexistent"), "tip", _verdict({"difftest": "fail"}))


def test_only_difftest_is_in_scope(paths):
    # A build or lint red is global by nature: it has no per-crate sub-result to map, and
    # excusing one would let a real break sit queued behind a derivation that cannot see it.
    for stages in (
        {"difftest": "fail", "build:i386": "fail"},
        {"build:i386": "fail"},
        {"lint:amd64": "fail"},
    ):
        assert not unrelated_red(REPO, Path("/nonexistent"), "tip", _verdict(stages)), stages


def test_closure_is_a_real_over_approximation():
    closure = _harness_closure(REPO, "msvcrt-difftest")
    assert closure is not None
    assert "sdk/lib/rust/msvcrt-difftest" in closure
    assert "sdk/lib/rust/msvcrt-crt-rust" in closure  # transitive path-dep
    assert any(c.startswith("dll/win32/msvcrt") for c in closure)  # C from build.rs
    assert _harness_closure(REPO, "no-such-difftest") is None


def test_prefix_match_respects_path_boundaries():
    # `startswith` alone would let a sibling crate masquerade as a member of the closure.
    assert not _touches({"sdk/lib/rust/msvcrt-sys-extra/src/lib.rs"}, {"sdk/lib/rust/msvcrt-sys"})
    assert _touches({"sdk/lib/rust/msvcrt-sys/src/lib.rs"}, {"sdk/lib/rust/msvcrt-sys"})
    assert _touches({"sdk/lib/rust/msvcrt-sys"}, {"sdk/lib/rust/msvcrt-sys"})


def test_failed_substage_parsing():
    assert _failed_substages("✓ a   ✗ b(exit 1)   ✗ c(exit 101)") == ["b", "c"]
    assert _failed_substages("✓ a   ✓ b") is None
    assert _failed_substages("") is None


def _sub(seq, tmp):
    return Submission(seq=seq, lane="l@w", tip="t", worktree=str(tmp), submitted_at=0)


def test_the_excuse_is_bounded(tmp_path, monkeypatch):
    monkeypatch.setattr(queue, "queue_dir", lambda repo: tmp_path)
    sub = _sub("0001-l", tmp_path)
    assert queue._unrelated_excuses(tmp_path, sub) == 0
    for expected in (1, 2):
        assert queue._unrelated_excuses(tmp_path, sub, bump=True) == expected
    assert queue._unrelated_excuses(tmp_path, sub) >= queue._MAX_UNRELATED_EXCUSES


def test_an_unpersistable_counter_refuses_the_excuse(tmp_path, monkeypatch):
    # If the bound cannot be recorded it is not a bound, so the excuse must not be granted.
    monkeypatch.setattr(queue, "queue_dir", lambda repo: tmp_path / "nope")
    monkeypatch.setattr(
        queue.Path, "mkdir", lambda *a, **k: (_ for _ in ()).throw(OSError("read-only"))
    )
    sub = _sub("0002-l", tmp_path)
    assert queue._unrelated_excuses(tmp_path, sub, bump=True) > queue._MAX_UNRELATED_EXCUSES
