"""Unit tests for the host free-space preflight / ENOSPC classification.

The evaluator is small but every branch is one an agent reads under time
pressure during an incident, so each is pinned — especially the two that are
easy to get backwards: DISK_FULL must beat the generic build-error matchers,
and a probe taken AFTER space was reclaimed must not print a reassuring number
that contradicts the ENOSPC just observed.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tests" / "qemu"))

from cli import build, hostdisk


def test_human_is_short_and_carries_magnitude():
    assert hostdisk.human(0) == "0B"
    assert hostdisk.human(5 * 1024**3) == "5.0G"
    assert hostdisk.human(192 * 1024) == "192.0K"


def test_free_bytes_walks_up_to_an_existing_ancestor(tmp_path):
    # A build dir that does not exist yet still sits on a real filesystem.
    missing = tmp_path / "build-arm64" / "nested" / "deeper"
    assert hostdisk.free_bytes(missing) is not None


def test_probe_is_ok_when_threshold_is_tiny(tmp_path):
    rep = hostdisk.probe([tmp_path], threshold=1)
    assert rep["ok"] is True
    assert rep["lines"] == []


def test_probe_reports_the_number_when_short(tmp_path):
    rep = hostdisk.probe([tmp_path], threshold=10**15)
    assert rep["ok"] is False
    assert len(rep["lines"]) >= 1
    # The whole value of this module is that the number is in the message.
    assert "DISK_FULL:" in rep["lines"][0]
    assert "free on" in rep["lines"][0]
    assert "need" in rep["lines"][0]


def test_probe_flags_critical_below_the_harness_write_cliff(tmp_path):
    # Threshold above free space, but free space also below CRITICAL -> critical.
    huge = 10**15
    rep = hostdisk.probe([tmp_path], threshold=huge)
    free = hostdisk.free_bytes(tmp_path)
    assert rep["critical"] is (free < hostdisk.CRITICAL_FREE_BYTES)


def test_probe_dedupes_by_filesystem(tmp_path):
    # Several paths on one filesystem must yield ONE report, not N.
    a = tmp_path / "a"
    b = tmp_path / "b"
    a.mkdir()
    b.mkdir()
    rep = hostdisk.probe([tmp_path, a, b], threshold=1)
    assert len(rep["free"]) == 1


def test_looks_like_enospc_matches_the_shapes_the_toolchain_emits():
    for blob in (
        "cc: error: write error: No space left on device",
        "OSError: [Errno 28] No space left on device: '/tmp/x/summary.json'",
        "ld.lld: error: failed to write output: ENOSPC",
        "Disk quota exceeded",
    ):
        assert hostdisk.looks_like_enospc(blob), blob


def test_looks_like_enospc_does_not_fire_on_ordinary_errors():
    for blob in (
        "error[E0425]: cannot find value `foo` in this scope",
        "ld.lld: error: undefined symbol: SpiParseDeviceInfo",
        "CMake Error at CMakeLists.txt:12",
        "",
    ):
        assert not hostdisk.looks_like_enospc(blob), blob


def test_disk_full_beats_the_generic_build_matchers():
    # An ENOSPC arrives wearing compiler/linker "error:" text. If the generic
    # matchers win, a host problem is filed as a defect in the diff -- the exact
    # misdirection that cost four lanes a day.
    cls, detail = build._classify_build_failure(
        "", "cc: error: write error: No space left on device"
    )
    assert cls == "DISK_FULL"
    assert detail  # never an empty explanation

    cls, _ = build._classify_build_failure(
        "", "ld.lld: error: undefined symbol: foo\nNo space left on device"
    )
    assert cls == "DISK_FULL"


def test_ordinary_failures_still_classify_normally():
    assert build._classify_build_failure("", "error[E0425]: nope")[0] == "CARGO_CHECK_FAIL"
    assert (
        build._classify_build_failure("", "ld.lld: error: undefined symbol: foo")[0] == "LINK_FAIL"
    )
    assert build._classify_build_failure("", "CMake Error at x")[0] == "CMAKE_CONFIGURE_FAIL"


def test_enospc_detail_never_contradicts_the_error_it_explains():
    # Space is normally fine by the time we probe (the failure freed it, or a
    # different mount filled). The detail must SAY that rather than print a
    # reassuring "plenty free" that reads as "the ENOSPC was imaginary".
    lines = hostdisk.enospc_detail()
    assert lines
    joined = " ".join(lines)
    assert "DISK_FULL:" in joined or "headroom" in joined


def test_repo_root_is_this_worktree():
    # Fleet-isolation law: derived from __file__, never a sibling worktree.
    assert (hostdisk.REPO_ROOT / "tests" / "qemu" / "cli" / "hostdisk.py").is_file()


def test_gate_paths_covers_tmp_and_every_build_dir():
    paths = [str(p) for p in hostdisk.gate_paths(Path("/repo"), ("i386", "amd64"))]
    assert "/tmp" in paths  # where artifact dirs + every command's stdout live
    assert "/repo/build-i386" in paths
    assert "/repo/build-amd64" in paths


def test_threshold_resolves_at_call_time_not_import_time(monkeypatch, tmp_path):
    # Regression: `threshold: int = MIN_FREE_BYTES` as a DEFAULT ARG binds the
    # constant once at import, so raising the module value had no effect and the
    # preflight failure path was unreachable (and therefore untested).
    monkeypatch.setattr(hostdisk, "MIN_FREE_BYTES", 10**15)
    rep = hostdisk.probe([tmp_path])
    assert rep["ok"] is False, "probe ignored the overridden module threshold"


def test_preflight_fails_loudly_with_the_number(monkeypatch, tmp_path):
    from cli.artifacts import Run

    monkeypatch.setattr(hostdisk, "MIN_FREE_BYTES", 10**15)
    run = Run("hostdisk-selftest", "amd64")
    assert hostdisk.preflight(run, tmp_path, ("i386",)) is False
    assert run.classification == "DISK_FULL"
    stage = next(s for s in run.stages if s.name == "host-disk")
    assert stage.status == "fail"
    assert "DISK_FULL:" in stage.detail and "free on" in stage.detail


def test_preflight_passes_and_records_headroom_when_healthy(tmp_path):
    from cli.artifacts import Run

    run = Run("hostdisk-selftest-ok", "amd64")
    assert hostdisk.preflight(run, tmp_path, ("i386",)) is True
    stage = next(s for s in run.stages if s.name == "host-disk")
    assert stage.status == "ok" and "free" in stage.detail
