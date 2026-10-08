"""modrun's grader: the guest only reports, so the host's verdict is the whole test."""

import importlib.util
import sys
from pathlib import Path

import pytest

_PATH = Path(__file__).resolve().parent / "modrun.py"
_SPEC = importlib.util.spec_from_file_location("modrun_under_test", _PATH)
assert _SPEC is not None and _SPEC.loader is not None
mr = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = mr
_SPEC.loader.exec_module(mr)


@pytest.fixture
def suites(tmp_path, monkeypatch):
    (tmp_path / "m.yaml").write_text(
        "module: m\nvariants: {c: 'C:\\c.exe', rust: 'C:\\r.exe'}\n"
        "normalize: [['pid=\\d+', 'pid=N']]\n"
        "cases:\n"
        "  - {id: same, stdin: [list], expect: [Disk 0]}\n"
        "  - {id: differs}\n"
        "  - {id: crash, arches: [amd64]}\n"
        "  - {id: known, xfail: 'bug 7'}\n"
    )
    monkeypatch.setattr(mr, "SUITES", tmp_path)
    return tmp_path


def _serial(*lines):
    return "\n".join(lines) + "\n"


def test_case_file_and_arch_filter(suites):
    text = mr.case_file("i386")
    assert "case m.same\nvariant c C:\\c.exe\nvariant rust C:\\r.exe\nstdin list\ntimeout 60\nend" in text
    assert "m.crash" not in text
    assert "m.crash" in mr.case_file("amd64")


def test_grading_diffs_against_the_first_variant(suites):
    serial = _serial(
        "[MODRUN:m.same:c] Disk 0 pid=12",
        "noise from another process",
        "[MODRUN] exit m.same c rc=0x00000000 ms=5",
        "[MODRUN:m.same:rust] Disk 0 pid=99   ",
        "[MODRUN] exit m.same rust rc=0x00000000 ms=5",
        "[MODRUN:m.differs:c] a",
        "[MODRUN] exit m.differs c rc=0x00000000 ms=5",
        "[MODRUN:m.differs:rust] b",
        "[MODRUN] exit m.differs rust rc=0x00000001 ms=5",
        "[MODRUN] exit m.crash c rc=0x00000000 ms=5",
        "[MODRUN] exit m.crash rust rc=0xe0525354 ms=5",
        "[MODRUN] exit m.known c rc=0x00000000 ms=5",
        "[MODRUN:m.known:rust] x",
        "[MODRUN] exit m.known rust rc=0x00000000 ms=5",
    )
    report, failures = mr.grade(serial, "amd64-uefi")
    assert "PASS m.same (2 variant(s), 1 line(s) each)" in report
    text = "\n".join(failures)
    assert "m.differs: rust: exit 0x00000001, c exited 0x00000000" in text
    assert any("+b" in r for r in report)
    assert "m.crash: rust: crashed, exit 0xe0525354" in text
    assert any(r.startswith("XFAIL m.known") for r in report)  # differs, as expected
    assert "m.known" not in text


def test_a_missing_exit_and_an_unexpected_pass_both_fail(suites):
    serial = _serial(
        "[MODRUN:m.known:c] same",
        "[MODRUN] exit m.known c rc=0x00000000 ms=5",
        "[MODRUN:m.known:rust] same",
        "[MODRUN] exit m.known rust rc=0x00000000 ms=5",
    )
    _, failures = mr.grade(serial, "i386")
    assert any(f.startswith("m.same: c: never finished") for f in failures)
    assert any("m.known: XPASS" in f for f in failures)
