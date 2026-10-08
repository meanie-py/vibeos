"""gen_suites.py --only: restrict the sweep to named suites.

Why the flag exists: a full apitest sweep is ~3 hours and interleaves 59 suites'
output into one serial, so asking "does this wedge need the preceding 200 subtests'
residue, or is it self-contained?" costs a whole run and still cannot separate the
two. --only turns that into a ~20-minute boot of one suite.

Why these tests exist: the flag's first draft broke the quarantine STALENESS warning
(it filtered the discovered set before the comparison, so 9 healthy entries were
reported as naming suites the build did not produce) and mislabelled its own summary
as "1 built" when 63 were. Both are the same mistake -- a filter applied upstream of
the checks that describe the build -- and both are silent.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location(
    "gen_suites", HERE / "payloads" / "apitest" / "gen_suites.py"
)
assert _spec and _spec.loader
G = importlib.util.module_from_spec(_spec)
sys.modules["gen_suites"] = G
_spec.loader.exec_module(G)


def _fake_build(tmp_path: Path, names: list[str]) -> Path:
    d = tmp_path / "build-amd64" / "modules" / "rostests" / "apitests"
    d.mkdir(parents=True)
    for n in names:
        (d / n).write_bytes(b"MZ")
    return tmp_path / "build-amd64"


def _run(monkeypatch, tmp_path, build, quarantine: str, extra: list[str]) -> int:
    out = tmp_path / "gen"
    out.mkdir(exist_ok=True)
    (out / "quarantine-amd64.txt").write_text(quarantine, encoding="utf-8")
    monkeypatch.setattr(G, "HERE", out)
    return G.main(["--arch", "amd64", "--build-dir", str(build), *extra])


def test_only_restricts_the_suite_list(monkeypatch, tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe", "c_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "", ["--only", "b_apitest.exe"])
    assert rc == 0
    listed = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert listed == ["b_apitest.exe"]


def test_only_is_repeatable(monkeypatch, tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe", "c_apitest.exe"])
    rc = _run(
        monkeypatch, tmp_path, build, "", ["--only", "a_apitest.exe", "--only", "c_apitest.exe"]
    )
    assert rc == 0
    listed = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert listed == ["a_apitest.exe", "c_apitest.exe"]


def test_only_naming_nothing_is_an_ERROR_not_an_empty_run(monkeypatch, tmp_path) -> None:
    """An empty sweep list boots, reports `done`, and proves nothing."""
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    assert _run(monkeypatch, tmp_path, build, "", ["--only", "typo_apitest.exe"]) == 1


def test_only_does_not_make_the_staleness_warning_cry_wolf(monkeypatch, tmp_path, capsys) -> None:
    """The regression that motivated this file.

    The quarantine staleness check must compare against what the BUILD produced, not
    the --only subset -- otherwise every healthy quarantine entry for a filtered-out
    suite is reported as missing, and the warning that catches real typos becomes
    noise nobody reads.
    """
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "a_apitest.exe\n", ["--only", "b_apitest.exe"])
    assert rc == 0
    err = capsys.readouterr().err
    assert "did not produce" not in err, err


def test_only_says_so_in_the_generated_header(monkeypatch, tmp_path) -> None:
    """A restricted list must not read like the full sweep.

    A baseline banked from a filtered run would silently carry one suite's coverage,
    and the ratchet floor is a set of assertion identities -- a floor built from the
    wrong set is not detectably wrong later.
    """
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    _run(monkeypatch, tmp_path, build, "", ["--only", "b_apitest.exe"])
    text = (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text()
    assert "RESTRICTED" in text
    assert "do not bank a baseline" in text.lower()


def test_without_only_nothing_changes(monkeypatch, tmp_path) -> None:
    """The flag must be a no-op when absent; this is the whole safety argument."""
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe", "c_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "b_apitest.exe\n", [])
    assert rc == 0
    text = (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text()
    listed = [ln for ln in text.splitlines() if ln and not ln.startswith("#")]
    assert listed == ["a_apitest.exe", "c_apitest.exe"]
    assert "RESTRICTED" not in text


# --ignore-quarantine: reproduce a quarantined wedge.
#
# The trap this closes is circular and cost a wasted boot: the quarantine entry that
# RECORDS a wedge is also what SUPPRESSES it, so `--only msvcrt_apitest.exe` on a
# quarantined subtest booted, printed `SKIP ... _wsystem`, and completed cleanly --
# a green run that proves nothing about the bug it was launched to reproduce.


def test_ignore_quarantine_arms_a_quarantined_suite(monkeypatch, tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "b_apitest.exe\n", ["--ignore-quarantine"])
    assert rc == 0
    listed = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert listed == ["a_apitest.exe", "b_apitest.exe"]


def test_ignore_quarantine_empties_the_subtest_skips(monkeypatch, tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "a_apitest.exe SomeSubtest\n", ["--ignore-quarantine"])
    assert rc == 0
    skips = [
        ln
        for ln in (tmp_path / "gen" / "apitest-skip-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert skips == []


def test_quarantine_still_applies_without_the_flag(monkeypatch, tmp_path) -> None:
    """The flag must be the ONLY thing that disarms the quarantine."""
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "b_apitest.exe\n", [])
    assert rc == 0
    listed = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert listed == ["a_apitest.exe"]


def test_ignore_quarantine_marks_the_list_as_unbankable(monkeypatch, tmp_path) -> None:
    """Same guard --only carries: a reproduction run must not seed a baseline."""
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "a_apitest.exe Sub\n", ["--ignore-quarantine"])
    assert rc == 0
    text = (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text()
    assert "QUARANTINE IGNORED" in text
    assert "do not bank a baseline" in text


def test_ignore_quarantine_is_reported_in_the_summary(monkeypatch, tmp_path, capsys) -> None:
    """It reached only the --only branch of the summary, so on its own it was silent."""
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "a_apitest.exe Sub\n", ["--ignore-quarantine"])
    assert rc == 0
    assert "QUARANTINE IGNORED" in capsys.readouterr().out


def test_stale_quarantine_still_warns_when_ignored(monkeypatch, tmp_path, capsys) -> None:
    """Disabling the SUBTRACTION must not disable the typo check that guards it."""
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "typo_apitest.exe\n", ["--ignore-quarantine"])
    assert rc == 0
    assert "typo_apitest.exe" in capsys.readouterr().err


def test_only_suite_colon_subtest_emits_the_two_field_line(monkeypatch, tmp_path) -> None:
    """`--only user32_apitest.exe:UserHandleGrantAccess` runs that one subtest: the sweep
    reads `<suite> <subtest>` as a single-subtest request, so a test behind an earlier
    stalling subtest can be reached at all."""
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    rc = _run(monkeypatch, tmp_path, build, "", ["--only", "b_apitest.exe:Grant"])
    assert rc == 0
    body = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert body == ["b_apitest.exe Grant"]


def test_only_colon_subtest_still_rejects_a_suite_the_build_lacks(monkeypatch, tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    assert _run(monkeypatch, tmp_path, build, "", ["--only", "typo_apitest.exe:Grant"]) == 1
