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

import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def _fake_build(tmp_path: Path, names: list[str]) -> Path:
    d = tmp_path / "build-amd64" / "modules" / "rostests" / "apitests"
    d.mkdir(parents=True)
    for n in names:
        (d / n).write_bytes(b"MZ")
    return tmp_path / "build-amd64"


def _run(tmp_path, build, quarantine: str, extra: list[str]) -> int:
    out = tmp_path / "gen"
    out.mkdir(exist_ok=True)
    (out / "quarantine-amd64.txt").write_text(quarantine, encoding="utf-8")
    result = subprocess.run(
        [
            str(REPO / "tools/codegen"),
            "--root",
            str(tmp_path),
            "apitest-suites",
            "--arch",
            "amd64",
            "--build-dir",
            str(build),
            "--out-dir",
            str(out),
            "--quarantine-dir",
            str(out),
            *extra,
        ],
        capture_output=True,
        text=True,
        check=False,
        timeout=180,
    )
    print(result.stdout, end="")
    print(result.stderr, end="", file=sys.stderr)
    return result.returncode


def test_only_restricts_the_suite_list(tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe", "c_apitest.exe"])
    rc = _run(tmp_path, build, "", ["--only", "b_apitest.exe"])
    assert rc == 0
    listed = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert listed == ["b_apitest.exe"]


def test_only_is_repeatable(tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe", "c_apitest.exe"])
    rc = _run(tmp_path, build, "", ["--only", "a_apitest.exe", "--only", "c_apitest.exe"])
    assert rc == 0
    listed = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert listed == ["a_apitest.exe", "c_apitest.exe"]


def test_only_naming_nothing_is_an_ERROR_not_an_empty_run(tmp_path) -> None:
    """An empty sweep list boots, reports `done`, and proves nothing."""
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    assert _run(tmp_path, build, "", ["--only", "typo_apitest.exe"]) == 1


def test_only_does_not_make_the_staleness_warning_cry_wolf(tmp_path, capsys) -> None:
    """The regression that motivated this file.

    The quarantine staleness check must compare against what the BUILD produced, not
    the --only subset -- otherwise every healthy quarantine entry for a filtered-out
    suite is reported as missing, and the warning that catches real typos becomes
    noise nobody reads.
    """
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    rc = _run(tmp_path, build, "a_apitest.exe\n", ["--only", "b_apitest.exe"])
    assert rc == 0
    err = capsys.readouterr().err
    assert "did not produce" not in err, err


def test_only_says_so_in_the_generated_header(tmp_path) -> None:
    """A restricted list must not read like the full sweep.

    A baseline banked from a filtered run would silently carry one suite's coverage,
    and the ratchet floor is a set of assertion identities -- a floor built from the
    wrong set is not detectably wrong later.
    """
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    _run(tmp_path, build, "", ["--only", "b_apitest.exe"])
    text = (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text()
    assert "RESTRICTED" in text
    assert "do not bank a baseline" in text.lower()


def test_without_only_nothing_changes(tmp_path) -> None:
    """The flag must be a no-op when absent; this is the whole safety argument."""
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe", "c_apitest.exe"])
    rc = _run(tmp_path, build, "b_apitest.exe\n", [])
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


def test_ignore_quarantine_arms_a_quarantined_suite(tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    rc = _run(tmp_path, build, "b_apitest.exe\n", ["--ignore-quarantine"])
    assert rc == 0
    listed = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert listed == ["a_apitest.exe", "b_apitest.exe"]


def test_ignore_quarantine_empties_the_subtest_skips(tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    rc = _run(tmp_path, build, "a_apitest.exe SomeSubtest\n", ["--ignore-quarantine"])
    assert rc == 0
    skips = [
        ln
        for ln in (tmp_path / "gen" / "apitest-skip-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert skips == []


def test_quarantine_still_applies_without_the_flag(tmp_path) -> None:
    """The flag must be the ONLY thing that disarms the quarantine."""
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    rc = _run(tmp_path, build, "b_apitest.exe\n", [])
    assert rc == 0
    listed = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert listed == ["a_apitest.exe"]


def test_ignore_quarantine_marks_the_list_as_unbankable(tmp_path) -> None:
    """Same guard --only carries: a reproduction run must not seed a baseline."""
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    rc = _run(tmp_path, build, "a_apitest.exe Sub\n", ["--ignore-quarantine"])
    assert rc == 0
    text = (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text()
    assert "QUARANTINE IGNORED" in text
    assert "do not bank a baseline" in text


def test_ignore_quarantine_is_reported_in_the_summary(tmp_path, capsys) -> None:
    """It reached only the --only branch of the summary, so on its own it was silent."""
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    rc = _run(tmp_path, build, "a_apitest.exe Sub\n", ["--ignore-quarantine"])
    assert rc == 0
    assert "QUARANTINE IGNORED" in capsys.readouterr().out


def test_stale_quarantine_still_warns_when_ignored(tmp_path, capsys) -> None:
    """Disabling the SUBTRACTION must not disable the typo check that guards it."""
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    rc = _run(tmp_path, build, "typo_apitest.exe\n", ["--ignore-quarantine"])
    assert rc == 0
    assert "typo_apitest.exe" in capsys.readouterr().err


def test_only_suite_colon_subtest_emits_the_two_field_line(tmp_path) -> None:
    """`--only user32_apitest.exe:UserHandleGrantAccess` runs that one subtest: the sweep
    reads `<suite> <subtest>` as a single-subtest request, so a test behind an earlier
    stalling subtest can be reached at all."""
    build = _fake_build(tmp_path, ["a_apitest.exe", "b_apitest.exe"])
    rc = _run(tmp_path, build, "", ["--only", "b_apitest.exe:Grant"])
    assert rc == 0
    body = [
        ln
        for ln in (tmp_path / "gen" / "apitest-suites-amd64.txt").read_text().splitlines()
        if ln and not ln.startswith("#")
    ]
    assert body == ["b_apitest.exe Grant"]


def test_only_colon_subtest_still_rejects_a_suite_the_build_lacks(tmp_path) -> None:
    build = _fake_build(tmp_path, ["a_apitest.exe"])
    assert _run(tmp_path, build, "", ["--only", "typo_apitest.exe:Grant"]) == 1


def _fixture(root):
    import json

    files = json.loads((REPO / "tools/vibeos-codegen/tests/payload/files.json").read_text())
    for name, text in files.items():
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)


def _native(root, *args):
    return subprocess.run(
        [str(REPO / "tools/codegen"), "--root", str(root), *map(str, args)],
        capture_output=True,
        text=True,
        check=False,
        timeout=180,
    )


def test_frozen_python_payload_outputs_and_diagnostics(tmp_path):
    import json

    _fixture(tmp_path)
    oracles = json.loads((REPO / "tools/vibeos-codegen/tests/payload/oracles.json").read_text())
    out = tmp_path / "payload"
    for case, extra in [
        ("api_all", []),
        ("api_only", ["--only", "b_apitest.exe:Z", "--only", "b_apitest.exe:A"]),
        ("api_ignore", ["--ignore-quarantine", "--only", "a_apitest.exe"]),
        ("api_missing", ["--only", "nope.exe"]),
        ("wine_default", ["--register", "ole32"]),
        (
            "wine_opts",
            [
                "--register",
                "",
                "--only",
                "foo:alpha",
                "--only",
                "bar_winetest",
                "--skip",
                "bar:zulu",
                "--repeat",
                "foo:alpha=3",
                "--timeout",
                "foo=51",
                "--global-flag",
                "foo_winetest.exe=0x20",
            ],
        ),
        ("wine_bad", ["--repeat", "foo:no=3"]),
    ]:
        api = case.startswith("api_")
        command, prefix = ("apitest-suites", "apitest") if api else ("winetest-suites", "winetest")
        selection = [] if api else ["--suites", "foo,bar,dwrite"]
        result = _native(tmp_path, command, "--arch", "amd64", "--out-dir", out, *selection, *extra)
        expected = oracles[case]
        assert result.returncode == expected["rc"], (case, result.stderr)
        assert result.stdout == expected["stdout"], case
        assert result.stderr == expected["stderr"], case
        if result.returncode == 0:
            assert (out / f"{prefix}-suites-amd64.txt").read_text() == expected["suites"], case
            assert (out / f"{prefix}-skip-amd64.txt").read_text() == expected["skips"], case
            checked = _native(
                tmp_path,
                command,
                "--arch",
                "amd64",
                "--out-dir",
                out,
                *selection,
                *extra,
                "--check",
            )
            assert checked.returncode == 0, checked.stderr


def test_native_discovery_is_sorted_unquarantined_and_read_only(tmp_path):
    import json

    _fixture(tmp_path)
    result = _native(
        tmp_path, "apitest-suites", "--build-dir", tmp_path / "build-amd64", "--discover-json"
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == ["a_apitest.exe", "b_apitest.exe", "rtl_unittest.exe"]
    assert not list(tmp_path.glob("tools/payloads/apitest/apitest-*.txt"))
    assert (
        _native(
            tmp_path, "apitest-suites", "--arch", "amd64", "--discover-json", "--ignore-quarantine"
        ).returncode
        != 0
    )


def test_suite_generator_refuses_foreign_build_and_source_links(tmp_path):
    _fixture(tmp_path)
    foreign = tmp_path.parent / (tmp_path.name + "-foreign.exe")
    foreign.write_text("MZ")
    target = tmp_path / "build-amd64/modules/rostests/apitests/a_apitest.exe"
    target.unlink()
    target.symlink_to(foreign)
    result = _native(tmp_path, "apitest-suites", "--arch", "amd64", "--discover-json")
    assert result.returncode != 0 and "outside" in result.stderr
    source = tmp_path / "modules/rostests/winetests/foo/testlist.c"
    source.unlink()
    source.symlink_to(foreign)
    result = _native(tmp_path, "winetest-suites", "--arch", "amd64", "--suites", "foo")
    assert result.returncode != 0 and "outside" in result.stderr


def test_bad_winetest_options_leave_both_existing_lists_untouched(tmp_path):
    _fixture(tmp_path)
    out = tmp_path / "payload"
    out.mkdir()
    paths = [out / f"winetest-{kind}-amd64.txt" for kind in ["suites", "skip"]]
    for path in paths:
        path.write_text("keep")
    for option, value in [
        ("--skip", "foo:nope"),
        ("--repeat", "foo:alpha=0"),
        ("--timeout", "foo=0"),
        ("--global-flag", "missing=1"),
        ("--suites", "../foo"),
    ]:
        result = _native(
            tmp_path,
            "winetest-suites",
            "--arch",
            "amd64",
            "--suites",
            "foo",
            "--out-dir",
            out,
            option,
            value,
        )
        assert result.returncode != 0, (option, result.stdout)
        assert all(path.read_text() == "keep" for path in paths)


def test_legacy_discovery_bridge_returns_native_names():
    import importlib.util
    import tempfile

    spec = importlib.util.spec_from_file_location(
        "native_gen_suites", REPO / "tools/payloads/apitest/gen_suites.py"
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    # Keep the selected artifact inside this checkout, isolated from its build dirs.
    (REPO / ".cache").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="suite-discovery-", dir=REPO / ".cache") as work:
        build = _fake_build(Path(work), ["b_apitest.exe", "a_unittest.exe"])
        assert module.discover(build) == ["a_unittest.exe", "b_apitest.exe"]
