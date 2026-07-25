"""Controls for tools/conformance_probe.py verdict attribution.

Both bugs this file guards against were the SAME mistake in two costumes: the
probe crediting a test with a verdict that says nothing about that test. The
first shipped four arm64 tests recorded as crashers when none of them had run;
the second summarised four amd64 tests as validly re-tested when a wedge had
manufactured their verdicts. Attribution is this tool's entire job, so it gets
controls.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from conformance_probe import POISONING, split_at_wedge


def test_no_wedge_keeps_every_verdict() -> None:
    got = ["A", "B", "C"]
    verdicts = {"A": "OK", "B": "CRASH", "C": "OK"}
    trustworthy, contaminated = split_at_wedge(got, verdicts)
    assert trustworthy == ["A", "B", "C"]
    assert contaminated == []


def test_crash_does_not_poison_later_verdicts() -> None:
    """A CRASH ends the boot, so anything after it simply never ran and lands in
    `missing`. A verdict that DOES follow a crash in the same boot is real."""
    got = ["A", "B"]
    verdicts = {"A": "CRASH", "B": "OK"}
    trustworthy, contaminated = split_at_wedge(got, verdicts)
    assert contaminated == []
    assert trustworthy == ["A", "B"]


def test_wedge_voids_everything_after_it() -> None:
    """The amd64 round-1 boot, verbatim. SeTokenFiltering wedges in-kernel and
    keeps kmtest_drv's result buffer registered; the four tests after it report
    NO-END because of that, not because of anything about themselves."""
    got = [
        "SeTokenFiltering",
        "MmReservedMapping",
        "NpfsFileInfo",
        "RtlException",
        "RtlExceptionKM",
    ]
    verdicts = {
        "SeTokenFiltering": "TIMEOUT",
        "MmReservedMapping": "NO-END",
        "NpfsFileInfo": "NO-END",
        "RtlException": "NO-END",
        "RtlExceptionKM": "NO-END",
    }
    trustworthy, contaminated = split_at_wedge(got, verdicts)
    assert trustworthy == ["SeTokenFiltering"]
    assert contaminated == [
        "MmReservedMapping",
        "NpfsFileInfo",
        "RtlException",
        "RtlExceptionKM",
    ]


def test_wedge_keeps_verdicts_recorded_before_it() -> None:
    """A test that completed BEFORE the wedge ran on a healthy kernel — that
    result is evidence and must survive, or a real pass gets thrown away."""
    got = ["Clean", "Wedger", "After"]
    verdicts = {"Clean": "OK", "Wedger": "TIMEOUT", "After": "NO-END"}
    trustworthy, contaminated = split_at_wedge(got, verdicts)
    assert trustworthy == ["Clean", "Wedger"]
    assert contaminated == ["After"]


def test_only_the_first_wedge_splits() -> None:
    got = ["A", "B", "C"]
    verdicts = {"A": "TIMEOUT", "B": "TIMEOUT", "C": "NO-END"}
    trustworthy, contaminated = split_at_wedge(got, verdicts)
    assert trustworthy == ["A"]
    assert contaminated == ["B", "C"]


def test_timeout_is_the_poisoning_verdict() -> None:
    """NO-END must NOT be poisoning: it is the ordinary way a usermode test
    self-crashes without taking the kernel down, and the sweep continues clean."""
    assert "TIMEOUT" in POISONING
    assert "NO-END" not in POISONING
    assert "CRASH" not in POISONING
