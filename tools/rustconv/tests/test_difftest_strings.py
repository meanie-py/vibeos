"""Bounded grammar inputs and NUL-terminated C-string grading."""

import json
import signal
from pathlib import Path

import pytest

from rustconv.difftest_contract import (
    ContractError,
    generate_cases,
    grade,
    load_contract,
    validate_string_generator,
)

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "data/difftest/atoi64.json"
ENDPTR_CONTRACT = ROOT / "data/difftest/strtoi64.json"
WIDE_CONTRACT = ROOT / "data/difftest/wtoi64.json"
MUTABLE_CONTRACT = ROOT / "data/difftest/strlwr.json"
MUTABLE_WIDE_CONTRACT = ROOT / "data/difftest/wcslwr.json"
OWNED_CONTRACT = ROOT / "data/difftest/strdup.json"


def test_string_grammar_rejects_unbounded_or_oversized_inputs():
    with pytest.raises(ContractError, match="maximum plus terminator"):
        validate_string_generator(
            {
                "max_bytes": 2,
                "examples": [""],
                "grammar": {"literal": "ab"},
            },
            "input",
        )
    with pytest.raises(ContractError, match="ASCII"):
        validate_string_generator(
            {
                "max_bytes": 4,
                "examples": ["é"],
                "grammar": {"literal": "a"},
            },
            "input",
        )
    with pytest.raises(ContractError, match="exactly one operator"):
        validate_string_generator(
            {
                "max_bytes": 4,
                "examples": [""],
                "grammar": {"literal": "a", "choice": [{"literal": "b"}]},
            },
            "input",
        )


def test_atoi64_cases_are_bounded_reproducible_and_nul_terminated():
    data = load_contract(CONTRACT)
    cases = generate_cases(data)
    assert cases == generate_cases(data)
    assert len(cases) == 831
    embedded_nul = next(case for case in cases if case.get("strings", {}).get("nptr") == "12\0x99")
    payload = bytes.fromhex(embedded_nul["allocations"]["nptr"])
    assert payload == b"12\0x99\0"
    assert embedded_nul["bindings"]["nptr"]["length"] == len(payload)
    assert {"left", "right"} <= {case["placement"] for case in cases}
    assert all(
        len(bytes.fromhex(case["allocations"]["nptr"])) <= 24
        for case in cases
        if "nptr" in case["allocations"]
    )
    null_cases = [case for case in cases if case["nulls"]["nptr"]]
    assert len(null_cases) == 277
    assert all(case["placement"] == "canary" and not case["allocations"] for case in null_cases)


def test_shipping_atoi64_cstr_and_parser_pass_with_sensitivity_controls(tmp_path):
    report = grade(CONTRACT, tmp_path / "atoi64")
    assert report["status"] == "pass", report
    assert report["schema"] == 16
    assert report["cases"] == 831
    assert report["mismatches"] == 0
    mutants = {mutant["name"]: mutant for mutant in report["mutants"]}
    assert set(mutants) == {"skip_parser_input", "wrong_return", "read_past_terminator"}
    assert all(mutant["detected"] for mutant in mutants.values())
    boundary = mutants["read_past_terminator"]
    assert boundary["expect"] == "memory_fault"
    assert boundary["fault"]["phase"] == "calling"
    assert boundary["fault"]["signal"] in (signal.SIGSEGV, signal.SIGBUS)
    assert boundary["fault"]["case"] == 1
    assert json.loads((tmp_path / "atoi64/report.json").read_text())["status"] == "pass"


def test_strtoi64_nullable_endptr_and_radix_grammar(tmp_path):
    report = grade(ENDPTR_CONTRACT, tmp_path / "strtoi64")
    assert report["status"] == "pass", report
    assert report["cases"] == 1016
    assert report["mismatches"] == 0
    mutants = {mutant["name"]: mutant for mutant in report["mutants"]}
    assert set(mutants) == {"wrong_base", "wrong_end_pointer"}
    assert all(mutant["detected"] for mutant in mutants.values())
    cases = json.loads((tmp_path / "strtoi64/cases.json").read_text())
    assert {case["placement"] for case in cases} == {"left", "right"}
    assert len([case for case in cases if case["nulls"]["endptr"]]) == 508
    assert all(case["outptrs"] == ["endptr"] for case in cases)


def test_wtoi64_uses_utf16_code_units_and_guarded_terminators(tmp_path):
    report = grade(WIDE_CONTRACT, tmp_path / "wtoi64")
    assert report["status"] == "pass", report
    assert report["cases"] == 280
    assert report["mismatches"] == 0
    cases = json.loads((tmp_path / "wtoi64/cases.json").read_text())
    supplementary = next(case for case in cases if case.get("strings", {}).get("str") == "12🌱99")
    assert (
        bytes.fromhex(supplementary["allocations"]["str"]) == "12🌱99".encode("utf-16-le") + b"\0\0"
    )
    assert supplementary["bindings"]["str"]["length"] == 14
    mutants = {mutant["name"]: mutant for mutant in report["mutants"]}
    assert set(mutants) == {"skip_wide_input", "read_past_wide_terminator"}
    assert all(mutant["detected"] for mutant in mutants.values())
    fault = mutants["read_past_wide_terminator"]["fault"]
    assert fault["phase"] == "calling"
    assert fault["signal"] in (signal.SIGSEGV, signal.SIGBUS)


def test_strlwr_mutable_cstr_preserves_tail_and_return_pointer(tmp_path):
    report = grade(MUTABLE_CONTRACT, tmp_path / "strlwr")
    assert report["status"] == "pass", report
    assert report["cases"] == 272
    assert report["mismatches"] == 0
    cases = json.loads((tmp_path / "strlwr/cases.json").read_text())
    embedded = next(case for case in cases if case["strings"]["_String"] == "Ab\0CD")
    assert bytes.fromhex(embedded["allocations"]["_String"]) == b"Ab\0CD\0"
    assert embedded["bindings"]["_String"]["readonly"] is False
    assert {case["placement"] for case in cases} == {"left", "right"}
    mutants = {mutant["name"]: mutant for mutant in report["mutants"]}
    assert set(mutants) == {"skip_ascii_folding", "read_past_terminator"}
    assert all(mutant["detected"] for mutant in mutants.values())
    fault = mutants["read_past_terminator"]["fault"]
    assert fault["phase"] == "calling"
    assert fault["signal"] in (signal.SIGSEGV, signal.SIGBUS)


def test_wcslwr_mutable_utf16_null_and_supplementary_inputs(tmp_path):
    report = grade(MUTABLE_WIDE_CONTRACT, tmp_path / "wcslwr")
    assert report["status"] == "pass", report
    assert report["cases"] == 408
    assert report["mismatches"] == 0
    cases = json.loads((tmp_path / "wcslwr/cases.json").read_text())
    supplementary = next(
        case for case in cases if case.get("strings", {}).get("_String") == "Äà é 🌱"
    )
    payload = "Äà é 🌱".encode("utf-16-le") + b"\0\0"
    assert bytes.fromhex(supplementary["allocations"]["_String"]) == payload
    assert supplementary["bindings"]["_String"]["length"] == len(payload)
    null_cases = [case for case in cases if case["nulls"]["_String"]]
    assert len(null_cases) == 136
    assert all(not case["allocations"] and case["placement"] == "canary" for case in null_cases)
    candidate_rows = [
        json.loads(line)
        for line in (tmp_path / "wcslwr/candidate.stdout").read_text().splitlines()
        if '"return"' in line
    ]
    assert candidate_rows[0]["return"] == {"allocation": "_String", "offset": 0}
    assert candidate_rows[2]["return"] == {"null": True}
    mutants = {mutant["name"]: mutant for mutant in report["mutants"]}
    assert set(mutants) == {"skip_ascii_folding", "read_past_terminator"}
    assert all(mutant["detected"] for mutant in mutants.values())
    fault = mutants["read_past_terminator"]["fault"]
    assert fault["phase"] == "calling"
    assert fault["signal"] in (signal.SIGSEGV, signal.SIGBUS)


def test_strdup_copies_owned_cstrings_and_releases_them(tmp_path):
    report = grade(OWNED_CONTRACT, tmp_path / "strdup")
    assert report["status"] == "pass", report
    assert report["cases"] == 405
    assert report["mismatches"] == 0
    cases = json.loads((tmp_path / "strdup/cases.json").read_text())
    null_input_cases = [case for case in cases if case["nulls"]["str_"]]
    assert len(null_input_cases) == 135
    assert all(not case["allocations"] for case in null_input_cases)
    candidate_rows = [
        json.loads(line)
        for line in (tmp_path / "strdup/candidate.stdout").read_text().splitlines()
        if '"return"' in line
    ]
    assert candidate_rows[0]["return"] == {"bytes": "00"}
    assert candidate_rows[2]["return"] == {"null": True}
    embedded = next(case for case in cases if case.get("strings", {}).get("str_") == "a\0tail")
    assert bytes.fromhex(embedded["allocations"]["str_"]) == b"a\0tail\0"
    mutants = {mutant["name"]: mutant for mutant in report["mutants"]}
    assert set(mutants) == {"allocation_always_fails", "copy_wrong_bytes"}
    assert all(mutant["detected"] for mutant in mutants.values())
