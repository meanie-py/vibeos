"""Alias, pointer-identity and protected-page grading must fail in the right direction."""

import json
import signal
import subprocess
from pathlib import Path

import pytest

from rustconv.difftest_contract import (
    Allocation,
    ContractError,
    _execute,
    generate_cases,
    grade,
    load_contract,
    normalized_pointer,
    selected_rust,
)

ROOT = Path(__file__).resolve().parents[1]


def search_contract(tmp_path):
    c = tmp_path / "source.c"
    rust = tmp_path / "source.rs"
    c.write_text(
        "#include <stdint.h>\n"
        "const uint8_t *search(const uint8_t *s, uint32_t n) {\n"
        "  for (uint32_t i = 0; i < n; ++i) if (s[i] == 255) return s+i;\n"
        "  return 0;\n}\n"
    )
    rust.write_text(
        'pub unsafe extern "C" fn search(s: *const u8, n: u32) -> *const u8 {\n'
        " for i in 0..n { if *s.add(i as usize) == 255 { return s.add(i as usize); } }\n"
        " core::ptr::null()\n}\n"
    )
    data = {
        "version": 1,
        "scope": "stateless-host",
        "memory": "guard-pages",
        "validity": "Read-only bounded search of initialized bytes.",
        "c_source": str(c),
        "rust_source": str(rust),
        "random_cases": 0,
        "functions": [
            {
                "c": "search",
                "rust": "candidate::search",
                "return": {
                    "type": "const_bytes",
                    "within": ["s"],
                    "nullable": True,
                    "one_past": False,
                },
                "args": [
                    {"name": "s", "type": "const_bytes", "length": "n"},
                    {"name": "n", "type": "u32", "values": [0, 1, 2, 7]},
                ],
            }
        ],
        "mutants": [
            {
                "name": "overread",
                "find": "for i in 0..n",
                "replace": "for i in 0..=n",
                "expect": "memory_fault",
            }
        ],
    }
    path = tmp_path / "contract.json"
    path.write_text(json.dumps(data))
    return path, data


def test_shared_arena_pilot_grades_aliases_and_return_pointer(tmp_path):
    report = grade(ROOT / "data/difftest/memmove.json", tmp_path / "run")
    assert report["status"] == "pass", report
    assert report["cases"] == 1428
    function = report["per_function"][0]
    assert function["placements"] == ["left", "right"]
    assert set(function["layouts"]) == {
        "same",
        "backward_overlap",
        "forward_overlap",
        "wider_overlap",
        "adjacent",
        "disjoint",
    }
    assert all(m["detected"] for m in report["mutants"])
    cases = json.loads((tmp_path / "run/cases.json").read_text())
    backward = next(
        c for c in cases if c["layout"] == "backward_overlap" and c["args"]["count"] == 7
    )
    assert backward["bindings"]["src"]["allocation"] == backward["bindings"]["dest"]["allocation"]
    assert len(backward["allocations"]["arena"]) == 16  # eight bytes encoded as hex


def test_shipping_search_pointer_and_both_read_boundaries(tmp_path):
    report = grade(ROOT / "data/difftest/memchr.json", tmp_path / "run")
    assert report["status"] == "pass", report
    mutants = {m["name"]: m for m in report["mutants"]}
    for name, placement in (("read_past_end", "right"), ("read_before_start", "left")):
        control = mutants[name]
        assert control["detected"]
        assert control["fault"]["signal"] == signal.SIGSEGV
        assert control["first_input"]["placement"] == placement
        assert control["completed_cases"] < report["cases"]
    assert mutants["wrong_return"]["fault"] is None
    assert mutants["wrong_return"]["mismatches"] > 0
    # Pointer identity is encoded without process addresses, including actual positive hits.
    rows = [
        json.loads(line) for line in (tmp_path / "run/candidate.stdout").read_text().splitlines()
    ]
    results = [r["return"] for r in rows if "return" in r]
    assert {"null": True} in results
    assert {"allocation": "s", "offset": 0} in results
    assert any(r.get("offset", 0) > 0 for r in results)


@pytest.mark.parametrize("side", ["reference", "candidate"])
def test_unmutated_read_overrun_is_never_a_pass(tmp_path, side):
    path, data = search_contract(tmp_path)
    if side == "reference":
        c = Path(data["c_source"])
        c.write_text(c.read_text().replace("i < n", "i <= n"))
    else:
        rust = Path(data["rust_source"])
        rust.write_text(rust.read_text().replace("0..n", "0..=n"))
    report = grade(path, tmp_path / "run")
    # A faulting reference cannot be graded; a faulting candidate against a clean
    # reference is a mismatch. Neither is ever a pass.
    if side == "reference":
        assert report["status"] == "invalid", report
        assert "reference: execution failed" in report["error"]
    else:
        assert report["status"] == "mismatch", report
        assert report["candidate_fault"]["phase"] == "calling"
    fault = json.loads((tmp_path / f"run/{side}.fault.json").read_text())
    assert fault["phase"] == "calling"
    assert fault["signal"] == signal.SIGSEGV


def test_memory_control_requires_a_fault_not_merely_different_output(tmp_path):
    path, data = search_contract(tmp_path)
    data["mutants"][0].update({"find": "== 255", "replace": "== 254"})
    path.write_text(json.dumps(data))
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "survived" in report["error"]


def test_aborting_mutant_is_not_a_guard_page_detection(tmp_path):
    path, data = search_contract(tmp_path)
    data["mutants"][0].update(
        {"find": "for i in 0..n", "replace": "std::process::abort(); for i in 0..n"}
    )
    path.write_text(json.dumps(data))
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "execution failed" in report["error"]


@pytest.mark.parametrize(
    "stdout", ["", '{"event":"call","case":0}\n{"event":"returned","case":0}\n']
)
def test_setup_or_observation_fault_does_not_count(tmp_path, monkeypatch, stdout):
    from rustconv import difftest_contract as module

    (tmp_path / "cases.json").write_text('[{"placement":"right"}]')
    monkeypatch.setattr(
        module,
        "_command",
        lambda *a, **k: subprocess.CompletedProcess([], -signal.SIGSEGV, stdout, ""),
    )
    with pytest.raises(ContractError, match="execution failed"):
        _execute(tmp_path, tmp_path / "unused.so", "bad", 1, allow_memory_fault=True)


@pytest.mark.parametrize(
    "change", ["missing_binding", "overflowing_view", "bad_extent", "unknown_return_base"]
)
def test_invalid_layout_or_return_contract_is_refused(tmp_path, change):
    data = json.loads((ROOT / "data/difftest/memmove.json").read_text())
    base = ROOT / "data/difftest"
    for key in ("c_source", "rust_source"):
        data[key] = str((base / data[key]).resolve())
    for helper in data["rust_helpers"]:
        helper["source"] = str((base / helper["source"]).resolve())
    if change == "missing_binding":
        del data["functions"][0]["layouts"][0]["bindings"]["src"]
    elif change == "overflowing_view":
        data["functions"][0]["layouts"][0]["bindings"]["src"]["offset"] = 60000
    elif change == "bad_extent":
        data["functions"][0]["layouts"][0]["allocations"]["arena"] = {"arg": "unknown", "add": 1}
    else:
        data["functions"][0]["return"]["within"] = ["unknown"]
    path = tmp_path / "bad.json"
    path.write_text(json.dumps(data))
    with pytest.raises(ContractError):
        generate_cases(load_contract(path))


def test_pointer_normalization_never_dereferences_and_checks_endpoint():
    memory = {"a": Allocation(b"abc", "canary")}
    try:
        case = {"bindings": {"p": {"allocation": "a", "offset": 0, "length": 3}}}
        rule = {"within": ["p"], "nullable": False, "one_past": False}
        assert normalized_pointer(1, rule, case, memory) == (
            {"invalid": True},
            ["returned pointer outside declared views"],
        )
        assert normalized_pointer(None, rule, case, memory)[1] == ["unexpected null return"]
        assert normalized_pointer(memory["a"].address + 3, rule, case, memory)[1]
        rule["one_past"] = True
        assert normalized_pointer(memory["a"].address + 3, rule, case, memory) == (
            {"allocation": "a", "offset": 3},
            [],
        )
    finally:
        memory["a"].close()


def test_nested_selection_preserves_conditional_ancestors():
    source = '#[cfg(feature="x")] mod nested { #![cfg(unix)] pub fn f() {} }'
    selected = selected_rust(source, ["nested::f"], preserve_cfg=True)
    assert '#[cfg(feature="x")]' in selected
    assert "#[cfg(unix)]" in selected
    with pytest.raises(ContractError, match="conditional"):
        selected_rust(source, ["nested::f"])


def test_compiler_must_prove_selected_conditional_function_exists(tmp_path):
    path, data = search_contract(tmp_path)
    rust = Path(data["rust_source"])
    rust.write_text('#[cfg(feature="disabled")]\n' + rust.read_text())
    data["rust_items"] = ["search"]
    path.write_text(json.dumps(data))
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "-rust failed" in report["error"]


def test_fault_control_must_be_explicit(tmp_path):
    path, data = search_contract(tmp_path)
    data["mutants"][0]["expect"] = "difference"
    path.write_text(json.dumps(data))
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "mutant_overread: execution failed" in report["error"]


def test_timeout_retains_partial_protocol_and_cannot_count_as_memory_fault(tmp_path, monkeypatch):
    from rustconv import difftest_contract as module

    def timeout(*args, **kwargs):
        raise subprocess.TimeoutExpired(
            args[0], 1, output=b'{"event":"call","case":0}\n', stderr=b"stuck"
        )

    monkeypatch.setattr(module.subprocess, "run", timeout)
    with pytest.raises(ContractError, match="execution failed"):
        _execute(tmp_path, tmp_path / "unused.so", "timed", 1, allow_memory_fault=True)
    assert (tmp_path / "timed.stdout").read_text() == '{"event":"call","case":0}\n'
    assert json.loads((tmp_path / "timed.result.json").read_text())["timed_out"]
    assert json.loads((tmp_path / "timed.fault.json").read_text())["signal"] is None


def test_relocating_relative_module_paths_is_refused():
    with pytest.raises(ContractError, match="relative module path"):
        selected_rust("mod m { pub fn f() { super::helper(); } }", ["m::f"], preserve_cfg=True)
    # Text in comments/literals is not a path expression.
    assert '"super::helper"' in selected_rust('pub fn f() { let _ = "super::helper"; }', ["f"])
