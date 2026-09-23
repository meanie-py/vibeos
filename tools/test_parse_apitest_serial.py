"""Tests for the apitest sweep parser — chiefly the ways it could go FALSE GREEN."""

from __future__ import annotations

import importlib.util
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def _load():
    path = REPO / "tools" / "conformance" / "parse_apitest_serial.py"
    spec = importlib.util.spec_from_file_location("parse_apitest_serial", path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


PA = _load()


def test_a_subtest_that_printed_its_summary_and_THEN_crashed_is_not_OK():
    """The false green this parser shipped with for about ten minutes.

    afd_apitest's `send` prints a clean wine summary and then takes the kernel
    down on teardown. Attributing the BUGCHECK only to rows that already looked
    bad scored the guilty subtest OK on the very boot it crashed.
    """
    log = "\n".join(
        [
            "[APITEST-SWEEP] suite afd_apitest.exe subtests=2",
            "[APITEST-SWEEP] launch afd_apitest.exe send",
            "[APITEST:afd_apitest.exe send] send: 11 tests executed "
            "(0 marked as todo, 0 failures), 0 skipped.",
            "*** BUGCHECK: 0x0000001E (0xffffffffc0000005, 0x0, 0x0, 0x0)",
        ]
    )
    results, meta = PA.parse(log)
    assert meta["bugcheck"] == "0x0000001E"
    (row,) = results
    assert row["name"] == "afd_apitest.exe send"
    assert row["status"] == "CRASH", row
    assert row["status"] in PA.BAD


def test_a_skipped_subtest_never_absorbs_a_later_crash():
    log = "\n".join(
        [
            "[APITEST-SWEEP] SKIP advapi32_apitest.exe RtlEncryptMemory",
            "*** BUGCHECK: 0x0000007E (0x0, 0x0, 0x0, 0x0)",
        ]
    )
    results, _ = PA.parse(log)
    assert [r["status"] for r in results] == ["SKIP"]


def test_identity_is_suite_plus_subtest_not_the_suite():
    """A suite bundles up to 19 subtests; collapsing them loses the attribution."""
    log = "\n".join(
        [
            "[APITEST-SWEEP] launch advapi32_apitest.exe RegEnumKey",
            "[APITEST:advapi32_apitest.exe RegEnumKey] RegEnumKey: 25 tests executed "
            "(0 marked as todo, 0 failures), 0 skipped.",
            "[APITEST-SWEEP] exit advapi32_apitest.exe RegEnumKey code=0",
            "[APITEST-SWEEP] launch advapi32_apitest.exe RegEnumValueW",
            "[APITEST:advapi32_apitest.exe RegEnumValueW] RegEnumValueW: 47 tests executed "
            "(0 marked as todo, 0 failures), 0 skipped.",
            "[APITEST-SWEEP] exit advapi32_apitest.exe RegEnumValueW code=0",
        ]
    )
    results, _ = PA.parse(log)
    assert [r["name"] for r in results] == [
        "advapi32_apitest.exe RegEnumKey",
        "advapi32_apitest.exe RegEnumValueW",
    ]
    assert [r["pass"] for r in results] == [25, 47]


def test_wine_counts_executed_so_pass_is_executed_minus_failures():
    log = "\n".join(
        [
            "[APITEST-SWEEP] launch acpi_apitest.exe Bus_PDO_EvalMethod",
            "[APITEST:acpi_apitest.exe Bus_PDO_EvalMethod] Bus_PDO_EvalMethod.c:857: "
            "Test failed: Line 641: Status = 0xc0000004, expected 0xc000009a",
            "[APITEST:acpi_apitest.exe Bus_PDO_EvalMethod] Bus_PDO_EvalMethod: "
            "335 tests executed (0 marked as todo, 9 failures), 0 skipped.",
            "[APITEST-SWEEP] exit acpi_apitest.exe Bus_PDO_EvalMethod code=0",
        ]
    )
    (row,) = PA.parse(log)[0]
    assert (row["pass"], row["fail"]) == (326, 9)
    assert row["asserts"] == {"Bus_PDO_EvalMethod.c:857"}
    assert row["status"] == "OK"


def test_exit_without_a_summary_line_stays_NO_END_and_gates():
    """The child died mid-run: it must not read as a clean zero-assertion pass."""
    log = "\n".join(
        [
            "[APITEST-SWEEP] launch advapi32_apitest.exe RegOpenKeyExW",
            "[APITEST:advapi32_apitest.exe RegOpenKeyExW] RegOpenKeyExW.c:37: Test failed: boom",
            "[APITEST-SWEEP] exit advapi32_apitest.exe RegOpenKeyExW code=1",
        ]
    )
    (row,) = PA.parse(log)[0]
    assert row["status"] == "NO-END"
    assert row["status"] in PA.BAD


def test_nosubtests_carries_the_reason_not_just_the_code():
    log = "[APITEST-SWEEP] NOSUBTESTS advpack_apitest.exe (exit 255) first-line=(no output)"
    _, meta = PA.parse(log)
    assert meta["nosubtests"]["advpack_apitest.exe"] == (255, "first-line=(no output)")
