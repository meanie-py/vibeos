"""Exercise generated harnesses with real host compilers and wrong candidates."""

import ctypes
import json
from pathlib import Path

import pytest

from rustconv.difftest_contract import ContractError, grade, load_contract, selected_rust

C = """
#include <stdint.h>
uint32_t fold(uint32_t x) { return x ^ 85; }
void copy(const uint8_t *src, uint8_t *dst, uint32_t n) {
    for (uint32_t i = 0; i < n; ++i) dst[i] = src[i] ^ 85;
}
"""
RUST = """
pub unsafe extern "C" fn fold(x: u32) -> u32 { x ^ 85 }
pub unsafe extern "C" fn copy(src: *const u8, dst: *mut u8, n: u32) {
    for i in 0..n as usize { *dst.add(i) = *src.add(i) ^ 85; }
}
"""


def contract(tmp_path):
    (tmp_path / "ref.c").write_text(C)
    (tmp_path / "candidate.rs").write_text(RUST)
    data = {
        "version": 1,
        "scope": "stateless-host",
        "validity": "Pure unsigned arithmetic; separate buffers, n initialized bytes each.",
        "c_source": "ref.c",
        "rust_source": "candidate.rs",
        "seed": 12,
        "random_cases": 3,
        "functions": [
            {
                "c": "fold",
                "rust": "candidate::fold",
                "return": "u32",
                "args": [{"name": "x", "type": "u32", "values": [0, 1, 2**32 - 1]}],
            },
            {
                "c": "copy",
                "rust": "candidate::copy",
                "return": "void",
                "args": [
                    {"name": "src", "type": "const_bytes", "length": "n"},
                    {"name": "dst", "type": "bytes", "length": "n"},
                    {"name": "n", "type": "u32", "values": [0, 1, 2, 7]},
                ],
            },
        ],
        "mutants": [
            {"name": "scalar", "find": "{ x ^ 85 }", "replace": "{ x ^ 84 }"},
            {
                "name": "drop_write",
                "find": "*dst.add(i) = *src.add(i) ^ 85;",
                "replace": "let _ = (*src.add(i), dst);",
            },
            {"name": "overrun", "find": "*dst.add(i) =", "replace": "*dst.add(i + 1) ="},
        ],
    }
    path = tmp_path / "contract.json"
    path.write_text(json.dumps(data))
    return path, data


def save(path, data):
    path.write_text(json.dumps(data))


def test_generated_harness_runs_and_kills_mutants(tmp_path):
    path, _ = contract(tmp_path)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "pass", report
    assert report["cases"] == 29
    assert report["mismatches"] == 0
    assert all(m["detected"] for m in report["mutants"])
    assert str(tmp_path / "ref.c") in report["inputs"]
    assert str(tmp_path / "candidate.rs") in report["inputs"]
    assert (tmp_path / "run/reference.d").exists()
    with pytest.raises(FileExistsError):
        grade(path, tmp_path / "run")


def test_wrong_candidate_is_not_accepted_and_saves_input(tmp_path):
    path, data = contract(tmp_path)
    source = tmp_path / "candidate.rs"
    source.write_text(RUST.replace("{ x ^ 85 }", "{ x ^ 86 }"))
    data["mutants"][0]["find"] = "{ x ^ 86 }"
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "mismatch", report
    example = json.loads((tmp_path / "run/counterexample.json").read_text())
    assert example["reference"]["return"] != example["candidate"]["return"]


def test_surviving_mutant_refuses_green(tmp_path):
    path, data = contract(tmp_path)
    data["mutants"][0]["replace"] = "{ x ^ (80 + 5) }"
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "survived" in report["error"]


def test_noncompiling_mutant_is_not_a_kill(tmp_path):
    path, data = contract(tmp_path)
    data["mutants"][0]["replace"] = "{ undefined_name }"
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "did not compile" in report["error"]


@pytest.mark.parametrize("side", ["C", "Rust"])
def test_compiler_checks_prototype(tmp_path, side):
    path, _data = contract(tmp_path)
    if side == "C":
        (tmp_path / "ref.c").write_text(
            C.replace("uint32_t fold(uint32_t", "uint32_t fold(uint64_t")
        )
    else:
        (tmp_path / "candidate.rs").write_text(RUST.replace("fold(x: u32)", "fold(x: u64)"))
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "failed" in report["error"]


def test_invalid_reference_write_is_not_rust_mismatch(tmp_path):
    path, _ = contract(tmp_path)
    (tmp_path / "ref.c").write_text(C.replace("dst[i] =", "dst[i + 1] ="))
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "C reference" in report["error"]


def test_crash_is_isolated_and_refuses_green(tmp_path):
    path, _ = contract(tmp_path)
    (tmp_path / "candidate.rs").write_text(RUST.replace("{ x ^ 85 }", "{ std::process::abort() }"))
    report = grade(path, tmp_path / "run")
    # The C reference is clean, so an abort inside the Rust call is the finding itself.
    assert report["status"] == "mismatch"
    assert report["candidate_fault"]["signal"] == 6
    assert report["candidate_fault"]["phase"] == "calling"


@pytest.mark.parametrize("change", ["typo", "negative_length", "unsupported", "no_mutants"])
def test_contract_refuses_guesses(tmp_path, change):
    path, data = contract(tmp_path)
    if change == "typo":
        data["functions"][1]["args"][0]["lenght"] = "n"
    elif change == "negative_length":
        data["functions"][1]["args"][0]["length"] = -1
    elif change == "unsupported":
        data["functions"][0]["args"][0]["type"] = "struct_handle"
    else:
        data["mutants"] = []
    save(path, data)
    with pytest.raises(ContractError):
        load_contract(path)


def test_selected_functions_preserve_bytes_and_refuse_cfg():
    assert selected_rust(RUST, ["fold"]).strip() == RUST.strip().splitlines()[0]
    with pytest.raises(ContractError, match="conditional"):
        selected_rust("#[cfg(unix)]\npub fn foo() {}", ["foo"])
    with pytest.raises(ContractError, match="missing"):
        selected_rust(RUST, ["missing"])


def test_real_shipping_pilots(tmp_path):
    root = Path(__file__).resolve().parents[1]
    for pilot in ("tolower", "swab"):
        report = grade(root / f"data/difftest/{pilot}.json", tmp_path / pilot)
        assert report["status"] == "pass", report
        assert all(m["detected"] for m in report["mutants"])


def test_undefined_c_overflow_invalidates_reference(tmp_path):
    path, data = contract(tmp_path)
    (tmp_path / "ref.c").write_text(
        "#include <stdint.h>\nuint32_t fold(uint32_t x) { volatile int32_t a = 2147483647; "
        "volatile int32_t b = a + 1; return (uint32_t)b + x; }\n"
    )
    data["functions"] = data["functions"][:1]
    data["mutants"] = data["mutants"][:1]
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "reference: execution failed" in report["error"]
    assert "signed integer overflow" in (tmp_path / "run/reference.stderr").read_text()


def test_every_function_needs_a_sensitivity_control(tmp_path):
    path, data = contract(tmp_path)
    data["mutants"] = data["mutants"][:1]
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "no sensitivity control detected" in report["error"]


def test_header_changed_during_build_refuses_grade(tmp_path, monkeypatch):
    from rustconv import difftest_contract as module

    path, _ = contract(tmp_path)
    header = tmp_path / "constant.h"
    header.write_text("#define CONST 85\n")
    (tmp_path / "ref.c").write_text('#include "constant.h"\n' + C.replace("^ 85", "^ CONST"))
    original = module._command

    def changing_command(argv, directory, label, timeout=120):
        if label == "build-c":
            header.write_text("#define CONST 84\n")
        return original(argv, directory, label, timeout)

    monkeypatch.setattr(module, "_command", changing_command)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "invalid"
    assert "input changed during compilation" in report["error"]


def test_cli_returns_nonzero_for_invalid_grade(tmp_path, monkeypatch):
    import argparse

    from rustconv.__main__ import cmd_difftest_contract

    path, data = contract(tmp_path)
    data["functions"][0]["return"] = "unsupported"
    save(path, data)
    with pytest.raises(SystemExit) as exc:
        cmd_difftest_contract(
            argparse.Namespace(contract=str(path), out=str(tmp_path / "run"), json=False)
        )
    assert exc.value.code == 1


def test_libc_symbol_cannot_interpose_on_original_body(tmp_path):
    path, data = contract(tmp_path)
    # Deliberately NOT libc abs: the oracle must execute the file we compiled.
    (tmp_path / "ref.c").write_text("int abs(int x) { return x + 7; }\n")
    (tmp_path / "candidate.rs").write_text(
        '#[no_mangle] pub unsafe extern "C" fn abs(x: i32) -> i32 { x + 7 }\n'
    )
    data["functions"] = [
        {
            "c": "abs",
            "rust": "candidate::abs",
            "return": "i32",
            "args": [{"name": "x", "type": "i32", "values": [-1, 0, 1]}],
        }
    ]
    data["mutants"] = [{"name": "wrong", "find": "x + 7", "replace": "x + 8"}]
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "pass", report
    rows = [
        json.loads(line) for line in (tmp_path / "run/reference.stdout").read_text().splitlines()
    ]
    assert next(row for row in rows if "return" in row)["return"] == 6


def test_flexible_record_generator_checks_layout_and_kills_mutants(tmp_path):
    (tmp_path / "record.c").write_text(
        "#include <stddef.h>\n#include <stdint.h>\n"
        "typedef struct { uint8_t Revision; uint8_t SubAuthorityCount; "
        "uint8_t Authority[6]; uint32_t SubAuthority[1]; } SIDISH;\n"
        "uint32_t record_length(SIDISH *sid) { "
        "return (uint32_t)offsetof(SIDISH, SubAuthority[sid->SubAuthorityCount]); }\n"
    )
    (tmp_path / "record.rs").write_text(
        'pub unsafe extern "C" fn record_length(sid: *const u8) -> u32 { '
        "8 + 4 * (*sid.add(1) as u32) }\n"
    )
    data = {
        "version": 1,
        "scope": "stateless-host",
        "validity": (
            "SID-like records have counts 0..=15 and one physical tail element at count zero."
        ),
        "c_source": "record.c",
        "rust_source": "record.rs",
        "random_cases": 7,
        "seed": 481,
        "functions": [
            {
                "c": "record_length",
                "rust": "candidate::record_length",
                "return": "u32",
                "args": [
                    {
                        "name": "sid",
                        "type": "const_bytes",
                        "c_type": "SIDISH *",
                        "rust_type": "*const u8",
                        "record": {
                            "layout": "SIDISH",
                            "base_size": 12,
                            "fields": [
                                {
                                    "name": "Revision",
                                    "offset": 0,
                                    "type": "u8",
                                    "count": 1,
                                    "values": [1],
                                },
                                {
                                    "name": "SubAuthorityCount",
                                    "offset": 1,
                                    "type": "u8",
                                    "count": 1,
                                    "values": [0, 1, 2, 7, 15],
                                },
                                {
                                    "name": "Authority",
                                    "offset": 2,
                                    "type": "u8",
                                    "count": 6,
                                    "values": [0, 1, 127, 255],
                                },
                                {
                                    "name": "SubAuthority",
                                    "offset": 8,
                                    "type": "u32",
                                    "count_from": "SubAuthorityCount",
                                    "min_count": 1,
                                    "values": [0, 1, 2147483648, 4294967295],
                                },
                            ],
                        },
                    }
                ],
            }
        ],
        "mutants": [
            {"name": "wrong_count_offset", "find": "*sid.add(1)", "replace": "*sid.add(0)"},
            {"name": "wrong_header_size", "find": "8 + 4 *", "replace": "7 + 4 *"},
        ],
    }
    path = tmp_path / "record-contract.json"
    save(path, data)
    report = grade(path, tmp_path / "record-run")
    assert report["status"] == "pass", report
    assert report["schema"] == 16
    assert all(mutant["detected"] for mutant in report["mutants"])
    cases = json.loads((tmp_path / "record-run/cases.json").read_text())
    assert any(case["records"]["sid"]["SubAuthorityCount"] == 0 for case in cases)
    assert all(len(bytes.fromhex(case["allocations"]["sid"])) >= 12 for case in cases)
    c_wrapper = (tmp_path / "record-run/reference.c").read_text()
    assert "_Static_assert(sizeof(SIDISH) == 12" in c_wrapper
    assert "offsetof(SIDISH, SubAuthority) == 8" in c_wrapper
    c_source = tmp_path / "record.c"
    c_source.write_text(
        c_source.read_text().replace("uint8_t Authority[6]", "uint16_t Authority[3]")
    )
    wrong_layout = grade(path, tmp_path / "wrong-layout-run")
    assert wrong_layout["status"] == "invalid"
    assert "build-c failed" in wrong_layout["error"]


def test_record_pointer_relocation_is_normalized_and_checked(tmp_path):
    (tmp_path / "pointer.c").write_text(
        "#include <stdint.h>\n"
        "typedef struct { const uint8_t *Data; uint32_t Length; } BYTE_VIEW;\n"
        "uint32_t view_sum(const BYTE_VIEW *view, const uint8_t *payload, uint32_t n) {\n"
        "  if (view->Data != payload || view->Length != n) return UINT32_MAX;\n"
        "  if (!n) return 0;\n"
        "  return (uint32_t)view->Data[0] * 257u + view->Data[1] + view->Length;\n"
        "}\n"
    )
    (tmp_path / "pointer.rs").write_text(
        'pub unsafe extern "C" fn view_sum(view: *const u8, payload: *const u8, n: u32) -> u32 {\n'
        "  let data = core::ptr::read_unaligned(view.cast::<*const u8>());\n"
        "  let length = core::ptr::read_unaligned("
        "view.add(core::mem::size_of::<*const u8>()).cast::<u32>());\n"
        "  if data != payload || length != n { u32::MAX } else if n == 0 { 0 } else {\n"
        "    *data as u32 * 257 + *data.add(1) as u32 + length\n"
        "  }\n"
        "}\n"
    )
    pointer_size = ctypes.sizeof(ctypes.c_void_p)
    pointer_alignment = ctypes.alignment(ctypes.c_void_p)
    record_size = (
        (pointer_size + 4 + pointer_alignment - 1) // pointer_alignment * pointer_alignment
    )
    data = {
        "version": 1,
        "scope": "stateless-host",
        "validity": "The view points to a two-byte initialized payload in this call.",
        "c_source": "pointer.c",
        "rust_source": "pointer.rs",
        "random_cases": 2,
        "seed": 729,
        "functions": [
            {
                "c": "view_sum",
                "rust": "candidate::view_sum",
                "return": "u32",
                "args": [
                    {
                        "name": "view",
                        "type": "const_bytes",
                        "c_type": "const BYTE_VIEW *",
                        "rust_type": "*const u8",
                        "record": {
                            "layout": "BYTE_VIEW",
                            "base_size": record_size,
                            "fields": [
                                {
                                    "name": "Data",
                                    "offset": 0,
                                    "points_to": "payload",
                                },
                                {
                                    "name": "Length",
                                    "offset": pointer_size,
                                    "type": "u32",
                                    "count": 1,
                                    "values": [0, 2],
                                },
                            ],
                        },
                    },
                    {"name": "payload", "type": "const_bytes", "length": "n"},
                    {"name": "n", "type": "u32", "values": [0, 2]},
                ],
            }
        ],
        "mutants": [
            {
                "name": "wrong_embedded_data_offset",
                "find": "*data.add(1)",
                "replace": "*data.add(0)",
            }
        ],
    }
    path = tmp_path / "pointer-contract.json"
    save(path, data)
    report = grade(path, tmp_path / "pointer-run")
    assert report["status"] == "pass", report
    assert report["schema"] == 16
    assert report["mutants"][0]["detected"]
    cases = json.loads((tmp_path / "pointer-run/cases.json").read_text())
    assert cases[0]["pointer_fields"] == [
        {
            "record_arg": "view",
            "source_binding": "view",
            "field": "Data",
            "offset": 0,
            "target_arg": "payload",
            "target_pointee": None,
            "source_pointee": None,
            "nullable": False,
            "one_past": True,
        }
    ]
    assert any(case["bindings"]["payload"]["length"] == 0 for case in cases)
    rows = [
        json.loads(line)
        for line in (tmp_path / "pointer-run/reference.stdout").read_text().splitlines()
        if line.startswith("{") and '"return"' in line
    ]
    assert rows and all(
        row["pointer_fields"]["view.Data"] == {"allocation": "payload", "offset": 0} for row in rows
    )
    wrapper = (tmp_path / "pointer-run/reference.c").read_text()
    assert f"sizeof(BYTE_VIEW) == {record_size}" in wrapper
    assert f"sizeof(((BYTE_VIEW *)0)->Data) == {pointer_size}" in wrapper
    invalid = json.loads(json.dumps(data))
    invalid["functions"][0]["args"][0]["record"]["fields"][0]["points_to"] = "missing"
    invalid_path = tmp_path / "pointer-contract-invalid.json"
    save(invalid_path, invalid)
    with pytest.raises(ContractError, match="points_to must name"):
        load_contract(invalid_path)


def test_nested_records_generate_values_relocate_links_and_check_layouts(tmp_path):
    (tmp_path / "nested.c").write_text(
        "#include <stdint.h>\n"
        "typedef struct { const uint8_t *Data; uint32_t Length; } INNER_VIEW;\n"
        "typedef struct { uint8_t Tag; INNER_VIEW View; } OUTER_VIEW;\n"
        "uint32_t nested_score(const OUTER_VIEW *root, const uint8_t *payload, uint32_t n) {\n"
        "  if (root->View.Data != payload || root->View.Length != n) return UINT32_MAX;\n"
        "  if (!n) return root->Tag;\n"
        "  return root->Tag + (uint32_t)root->View.Data[0] * 257u + "
        "root->View.Data[1] + root->View.Length;\n"
        "}\n"
    )
    pointer_size = ctypes.sizeof(ctypes.c_void_p)
    pointer_alignment = ctypes.alignment(ctypes.c_void_p)
    nested_size = (
        (pointer_size + 4 + pointer_alignment - 1) // pointer_alignment * pointer_alignment
    )
    nested_offset = (1 + pointer_alignment - 1) // pointer_alignment * pointer_alignment
    root_size = (
        (nested_offset + nested_size + pointer_alignment - 1)
        // pointer_alignment
        * pointer_alignment
    )
    (tmp_path / "nested.rs").write_text(
        'pub unsafe extern "C" fn nested_score(\n'
        "    root: *const u8, payload: *const u8, n: u32\n"
        ") -> u32 {\n"
        "  let tag = *root;\n"
        f"  let inner = root.add({nested_offset});\n"
        "  let data = core::ptr::read_unaligned(inner.cast::<*const u8>());\n"
        "  let length = core::ptr::read_unaligned("
        "inner.add(core::mem::size_of::<*const u8>()).cast::<u32>());\n"
        "  if data != payload || length != n { u32::MAX } else if n == 0 { tag as u32 } else {\n"
        "    tag as u32 + *data as u32 * 257 + *data.add(1) as u32 + length\n"
        "  }\n"
        "}\n"
    )
    data = {
        "version": 1,
        "scope": "stateless-host",
        "validity": "The inline view links to a two-byte payload when its length is nonzero.",
        "c_source": "nested.c",
        "rust_source": "nested.rs",
        "random_cases": 2,
        "seed": 887,
        "functions": [
            {
                "c": "nested_score",
                "rust": "candidate::nested_score",
                "return": "u32",
                "args": [
                    {
                        "name": "root",
                        "type": "const_bytes",
                        "c_type": "const OUTER_VIEW *",
                        "rust_type": "*const u8",
                        "record": {
                            "layout": "OUTER_VIEW",
                            "base_size": root_size,
                            "fields": [
                                {
                                    "name": "Tag",
                                    "offset": 0,
                                    "type": "u8",
                                    "count": 1,
                                    "values": [0, 0x5A],
                                },
                                {
                                    "name": "View",
                                    "offset": nested_offset,
                                    "record": {
                                        "layout": "INNER_VIEW",
                                        "base_size": nested_size,
                                        "fields": [
                                            {
                                                "name": "Data",
                                                "offset": 0,
                                                "points_to": "payload",
                                            },
                                            {
                                                "name": "Length",
                                                "offset": pointer_size,
                                                "type": "u32",
                                                "count": 1,
                                                "values": [0, 2],
                                            },
                                        ],
                                    },
                                },
                            ],
                        },
                    },
                    {"name": "payload", "type": "const_bytes", "length": "n"},
                    {"name": "n", "type": "u32", "values": [0, 2]},
                ],
            }
        ],
        "mutants": [
            {"name": "wrong_nested_tag", "find": "let tag = *root;", "replace": "let tag = 0;"}
        ],
    }
    path = tmp_path / "nested-contract.json"
    save(path, data)
    report = grade(path, tmp_path / "nested-run")
    assert report["status"] == "pass", report
    assert report["schema"] == 16
    assert all(mutant["detected"] for mutant in report["mutants"])
    cases = json.loads((tmp_path / "nested-run/cases.json").read_text())
    assert cases[0]["records"]["root"]["View"]["Length"] == 0
    assert cases[0]["pointer_fields"][0]["field"] == "View.Data"
    assert cases[0]["pointer_fields"][0]["offset"] == nested_offset
    wrapper = (tmp_path / "nested-run/reference.c").read_text()
    assert f"sizeof(OUTER_VIEW) == {root_size}" in wrapper
    assert f"offsetof(OUTER_VIEW, View) == {nested_offset}" in wrapper
    assert f"sizeof(((OUTER_VIEW *)0)->View) == {nested_size}" in wrapper
    assert f"offsetof(INNER_VIEW, Length) == {pointer_size}" in wrapper
    invalid = json.loads(json.dumps(data))
    child = invalid["functions"][0]["args"][0]["record"]["fields"][1]["record"]
    child["fields"].append(
        {
            "name": "Tail",
            "offset": nested_size,
            "type": "u8",
            "count_from": "Length",
            "values": [0, 1],
        }
    )
    invalid_path = tmp_path / "nested-contract-invalid.json"
    save(invalid_path, invalid)
    with pytest.raises(ContractError, match="nested records cannot have flexible tails"):
        load_contract(invalid_path)


def test_fixed_arrays_of_nested_records_cross_values_relocate_each_link(tmp_path):
    (tmp_path / "nested_array.c").write_text(
        "#include <stdint.h>\n"
        "typedef struct { uint32_t Code; const uint8_t *Link; } ITEM;\n"
        "typedef struct { uint32_t Count; ITEM Single[1]; ITEM Entries[2]; } BATCH;\n"
        "uint32_t nested_array_score(const BATCH *root, const uint8_t *payload) {\n"
        "  if (root->Single[0].Link != payload || root->Entries[0].Link != payload || "
        "root->Entries[1].Link != payload) {\n"
        "    return UINT32_MAX;\n"
        "  }\n"
        "  return root->Count + root->Single[0].Code + root->Entries[0].Code + "
        "root->Entries[1].Code + root->Single[0].Link[0] + root->Entries[1].Link[1];\n"
        "}\n"
    )
    pointer_size = ctypes.sizeof(ctypes.c_void_p)
    pointer_alignment = ctypes.alignment(ctypes.c_void_p)
    item_alignment = max(ctypes.alignment(ctypes.c_uint32), pointer_alignment)
    link_offset = (4 + pointer_alignment - 1) // pointer_alignment * pointer_alignment
    item_size = (link_offset + pointer_size + item_alignment - 1) // item_alignment * item_alignment
    single_offset = (4 + item_alignment - 1) // item_alignment * item_alignment
    entries_offset = single_offset + item_size
    batch_size = entries_offset + 2 * item_size
    (tmp_path / "nested_array.rs").write_text(
        'pub unsafe extern "C" fn nested_array_score(\n'
        "    root: *const u8, payload: *const u8\n"
        ") -> u32 {\n"
        "  let count = core::ptr::read_unaligned(root.cast::<u32>());\n"
        f"  let single_code = core::ptr::read_unaligned(root.add({single_offset}).cast::<u32>());\n"
        f"  let single_link = core::ptr::read_unaligned("
        f"root.add({single_offset + link_offset}).cast::<*const u8>());\n"
        f"  let code0 = core::ptr::read_unaligned(root.add({entries_offset}).cast::<u32>());\n"
        f"  let link0 = core::ptr::read_unaligned(root.add({entries_offset + link_offset})"
        ".cast::<*const u8>());\n"
        f"  let code1 = core::ptr::read_unaligned(root.add({entries_offset + item_size})"
        ".cast::<u32>());\n"
        f"  let link1 = core::ptr::read_unaligned("
        f"root.add({entries_offset + item_size + link_offset}).cast::<*const u8>());\n"
        "  if single_link != payload || link0 != payload || link1 != payload { u32::MAX } else {\n"
        "    count + single_code + code0 + code1 + *single_link as u32 + *link1.add(1) as u32\n"
        "  }\n"
        "}\n"
    )
    data = {
        "version": 1,
        "scope": "stateless-host",
        "validity": "Both inline entries point to a two-byte payload allocation.",
        "c_source": "nested_array.c",
        "rust_source": "nested_array.rs",
        "memory": "canary",
        "random_cases": 1,
        "seed": 1381,
        "functions": [
            {
                "c": "nested_array_score",
                "rust": "candidate::nested_array_score",
                "return": "u32",
                "args": [
                    {
                        "name": "root",
                        "type": "const_bytes",
                        "c_type": "const BATCH *",
                        "rust_type": "*const u8",
                        "record": {
                            "layout": "BATCH",
                            "base_size": batch_size,
                            "fields": [
                                {
                                    "name": "Count",
                                    "offset": 0,
                                    "type": "u32",
                                    "count": 1,
                                    "values": [0, 5],
                                },
                                {
                                    "name": "Single",
                                    "offset": single_offset,
                                    "count": 1,
                                    "record": {
                                        "layout": "ITEM",
                                        "base_size": item_size,
                                        "fields": [
                                            {
                                                "name": "Code",
                                                "offset": 0,
                                                "type": "u32",
                                                "count": 1,
                                                "values": [1, 9],
                                            },
                                            {
                                                "name": "Link",
                                                "offset": link_offset,
                                                "points_to": "payload",
                                            },
                                        ],
                                    },
                                },
                                {
                                    "name": "Entries",
                                    "offset": entries_offset,
                                    "count": 2,
                                    "record": {
                                        "layout": "ITEM",
                                        "base_size": item_size,
                                        "fields": [
                                            {
                                                "name": "Code",
                                                "offset": 0,
                                                "type": "u32",
                                                "count": 1,
                                                "values": [1, 9],
                                            },
                                            {
                                                "name": "Link",
                                                "offset": link_offset,
                                                "points_to": "payload",
                                            },
                                        ],
                                    },
                                },
                            ],
                        },
                    },
                    {"name": "payload", "type": "const_bytes", "length": 2},
                ],
            }
        ],
        "mutants": [
            {
                "name": "wrong_second_entry_code",
                "find": (
                    f"let code1 = core::ptr::read_unaligned(root.add({entries_offset + item_size})"
                    ".cast::<u32>());"
                ),
                "replace": "let code1 = 0u32;",
            }
        ],
    }
    path = tmp_path / "nested-array-contract.json"
    save(path, data)
    report = grade(path, tmp_path / "nested-array-run")
    assert report["status"] == "pass", report
    assert report["schema"] == 16
    assert report["cases"] == 81
    assert report["mutants"][0]["detected"]
    cases = json.loads((tmp_path / "nested-array-run/cases.json").read_text())
    assert cases[0]["records"]["root"]["Entries"] == [
        {"Code": 1, "Link": {"points_to": "payload", "nullable": False, "one_past": False}},
        {"Code": 1, "Link": {"points_to": "payload", "nullable": False, "one_past": False}},
    ]
    assert cases[0]["records"]["root"]["Single"] == [
        {"Code": 1, "Link": {"points_to": "payload", "nullable": False, "one_past": False}}
    ]
    assert [field["field"] for field in cases[0]["pointer_fields"]] == [
        "Single[0].Link",
        "Entries[0].Link",
        "Entries[1].Link",
    ]
    assert [field["offset"] for field in cases[0]["pointer_fields"]] == [
        single_offset + link_offset,
        entries_offset + link_offset,
        entries_offset + item_size + link_offset,
    ]
    wrapper = (tmp_path / "nested-array-run/reference.c").read_text()
    assert "sizeof(((BATCH *)0)->Entries) / sizeof(((BATCH *)0)->Entries[0]) == 2" in wrapper
    assert "_Generic(&((BATCH *)0)->Single, ITEM (*)[1]: 1" in wrapper
    assert "parent->Entries[0] = *nested;" in wrapper
    assert f"offsetof(BATCH, Entries) == {entries_offset}" in wrapper

    invalid = json.loads(json.dumps(data))
    invalid["functions"][0]["args"][0]["record"]["fields"][1]["count"] = 65
    invalid_path = tmp_path / "nested-array-contract-invalid.json"
    save(invalid_path, invalid)
    with pytest.raises(ContractError, match="nested record count"):
        load_contract(invalid_path)


def test_hidden_pointee_records_generate_and_compare_multihop_mutations(tmp_path):
    (tmp_path / "pointee_graph.c").write_text(
        "#include <stdint.h>\n"
        "typedef struct NODE { uint32_t Value; struct NODE *Next; } NODE;\n"
        "typedef struct { NODE *Head; const NODE *Observer; } ROOT;\n"
        "uint32_t chain_score(const ROOT *root) {\n"
        "  if (!root->Head || !root->Head->Next || !root->Observer) return UINT32_MAX;\n"
        "  root->Head->Next->Value += 1;\n"
        "  return root->Head->Value * 100u + root->Head->Next->Value + "
        "root->Observer->Value;\n"
        "}\n"
    )
    pointer_size = ctypes.sizeof(ctypes.c_void_p)
    pointer_alignment = ctypes.alignment(ctypes.c_void_p)
    next_offset = (4 + pointer_alignment - 1) // pointer_alignment * pointer_alignment
    node_size = (
        (next_offset + pointer_size + pointer_alignment - 1)
        // pointer_alignment
        * pointer_alignment
    )
    (tmp_path / "pointee_graph.rs").write_text(
        'pub unsafe extern "C" fn chain_score(root: *const u8) -> u32 {\n'
        "  let first = core::ptr::read_unaligned(root.cast::<*mut u8>());\n"
        "  if first.is_null() { return u32::MAX; }\n"
        "  let first_value = core::ptr::read_unaligned(first.cast::<u32>());\n"
        f"  let observer = core::ptr::read_unaligned(root.add({pointer_size})"
        ".cast::<*const u8>());\n"
        "  if observer.is_null() { return u32::MAX; }\n"
        "  let observer_value = core::ptr::read_unaligned(observer.cast::<u32>());\n"
        f"  let next = core::ptr::read_unaligned(first.add({next_offset}).cast::<*mut u8>());\n"
        "  if next.is_null() { return u32::MAX; }\n"
        "  let next_value = core::ptr::read_unaligned(next.cast::<u32>());\n"
        "  core::ptr::write_unaligned(next.cast::<u32>(), next_value + 1);\n"
        "  first_value * 100 + next_value + 1 + observer_value\n"
        "}\n"
    )

    def node(values, next_ref=None):
        fields = [{"name": "Value", "offset": 0, "type": "u32", "count": 1, "values": values}]
        if next_ref is not None:
            fields.append(
                {"name": "Next", "offset": next_offset, "points_to": {"pointee": next_ref}}
            )
        return {"layout": "NODE", "base_size": node_size, "fields": fields}

    data = {
        "version": 1,
        "scope": "stateless-host",
        "validity": (
            "The root links to two separately allocated owned nodes; "
            "the second value is incremented."
        ),
        "c_source": "pointee_graph.c",
        "rust_source": "pointee_graph.rs",
        "memory": "canary",
        "random_cases": 1,
        "seed": 1729,
        "functions": [
            {
                "c": "chain_score",
                "rust": "candidate::chain_score",
                "return": "u32",
                "args": [
                    {
                        "name": "root",
                        "type": "const_bytes",
                        "c_type": "const ROOT *",
                        "rust_type": "*const u8",
                        "record": {
                            "layout": "ROOT",
                            "base_size": pointer_size * 2,
                            "fields": [
                                {
                                    "name": "Head",
                                    "offset": 0,
                                    "points_to": {"pointee": "first"},
                                },
                                {
                                    "name": "Observer",
                                    "offset": pointer_size,
                                    "points_to": {"pointee": "observer"},
                                },
                            ],
                            "pointees": [
                                {
                                    "name": "first",
                                    "record": node([2, 5], "second"),
                                    "mutable": True,
                                },
                                {"name": "second", "record": node([4, 11]), "mutable": True},
                                {"name": "observer", "record": node([8, 12])},
                            ],
                        },
                    }
                ],
            }
        ],
        "mutants": [
            {
                "name": "skip_hidden_pointee_write",
                "find": "core::ptr::write_unaligned(next.cast::<u32>(), next_value + 1);",
                "replace": "core::ptr::write_unaligned(next.cast::<u32>(), next_value);",
            }
        ],
    }
    path = tmp_path / "pointee-graph-contract.json"
    save(path, data)
    report = grade(path, tmp_path / "pointee-graph-run")
    assert report["status"] == "pass", report
    assert report["schema"] == 16
    assert report["cases"] == 9
    assert report["mutants"][0]["detected"]
    cases = json.loads((tmp_path / "pointee-graph-run/cases.json").read_text())
    case = cases[0]
    assert case["pointees"]["root"]["first"]["Value"] == 2
    assert case["pointees"]["root"]["second"]["Value"] == 4
    assert case["pointees"]["root"]["observer"]["Value"] == 8
    assert [field["target_pointee"] for field in case["pointer_fields"]] == [
        "first",
        "observer",
        "second",
    ]
    assert [field["source_binding"] for field in case["pointer_fields"]] == [
        "root",
        "root",
        "@root/pointee/first",
    ]
    wrapper = (tmp_path / "pointee-graph-run/reference.c").read_text()
    assert "sizeof(NODE) ==" in wrapper
    assert "((ROOT *)0)->Head = target;" in wrapper
    assert "const NODE * target" in wrapper
    assert "((ROOT *)0)->Observer = target;" in wrapper
    assert "((NODE *)0)->Next = target;" in wrapper

    invalid = json.loads(json.dumps(data))
    invalid["functions"][0]["args"][0]["record"]["pointees"][0]["record"]["fields"][1]["points_to"][
        "pointee"
    ] = "missing"
    invalid_path = tmp_path / "pointee-graph-contract-invalid.json"
    save(invalid_path, invalid)
    with pytest.raises(ContractError, match="unknown pointee"):
        load_contract(invalid_path)

    unreachable = json.loads(json.dumps(data))
    unreachable["functions"][0]["args"][0]["record"]["fields"][0]["points_to"]["pointee"] = "second"
    unreachable_path = tmp_path / "pointee-graph-unreachable.json"
    save(unreachable_path, unreachable)
    with pytest.raises(ContractError, match="not reachable from the root"):
        load_contract(unreachable_path)


@pytest.mark.parametrize(
    ("field_change", "message"),
    [
        (lambda record: record.update(base_size=2), "does not contain"),
        (lambda record: record["fields"][2].update(offset=1), "overlap"),
        (lambda record: record["fields"][2].update(count_from="Missing"), "earlier scalar"),
    ],
)
def test_record_contract_rejects_ambiguous_layouts(tmp_path, field_change, message):
    path, data = contract(tmp_path)
    data["functions"] = [
        {
            "c": "copy",
            "rust": "candidate::copy",
            "return": "void",
            "args": [
                {
                    "name": "src",
                    "type": "const_bytes",
                    "c_type": "SAMPLE *",
                    "record": {
                        "layout": "SAMPLE",
                        "base_size": 3,
                        "fields": [
                            {"name": "tag", "offset": 0, "type": "u8", "count": 1, "values": [1]},
                            {"name": "size", "offset": 1, "type": "u8", "count": 1, "values": [1]},
                            {
                                "name": "bytes",
                                "offset": 2,
                                "type": "u8",
                                "count_from": "size",
                                "min_count": 1,
                                "values": [0, 255],
                            },
                        ],
                    },
                }
            ],
        }
    ]
    data["mutants"] = [{"name": "copy", "find": "^", "replace": "// altered\n"}]
    field_change(data["functions"][0]["args"][0]["record"])
    save(path, data)
    with pytest.raises(ContractError, match=message):
        load_contract(path)


def test_libc_owned_byte_result_is_copied_and_freed(tmp_path):
    (tmp_path / "owned.c").write_text(
        "#include <stdint.h>\n#include <stdlib.h>\n"
        "void *copy_owned(const uint8_t *src, uint32_t length) {\n"
        "  uint8_t *out = malloc(length ? length : 1);\n"
        "  if (!out) return 0;\n"
        "  for (uint32_t i = 0; i < length; ++i) out[i] = src[i] ^ 0x5a;\n"
        "  return out;\n}\n"
    )
    (tmp_path / "owned.rs").write_text(
        'unsafe extern "C" { fn malloc(size: usize) -> *mut core::ffi::c_void; }\n'
        'pub unsafe extern "C" fn copy_owned(src: *const u8, length: u32) -> *mut u8 {\n'
        "  let out = malloc(length.max(1) as usize) as *mut u8;\n"
        "  if !out.is_null() {\n"
        "    for i in 0..length as usize { *out.add(i) = *src.add(i) ^ 0x5a; }\n"
        "  }\n"
        "  out\n}\n"
    )
    data = {
        "version": 1,
        "scope": "stateless-host",
        "validity": "The function returns a libc-malloc-owned buffer of exactly length bytes.",
        "c_source": "owned.c",
        "rust_source": "owned.rs",
        "random_cases": 5,
        "seed": 71,
        "functions": [
            {
                "c": "copy_owned",
                "rust": "candidate::copy_owned",
                "return": {
                    "type": "owned_bytes",
                    "allocator": "libc",
                    "nullable": False,
                    "length": "length",
                },
                "args": [
                    {"name": "src", "type": "const_bytes", "length": "length"},
                    {
                        "name": "length",
                        "type": "u32",
                        "values": [0, 1, 2, 7, 32],
                        "random_range": [0, 32],
                    },
                ],
            }
        ],
        "mutants": [
            {
                "name": "wrong_owned_byte",
                "find": "*out.add(i) = *src.add(i) ^ 0x5a;",
                "replace": "*out.add(i) = *src.add(i) ^ 0x5b;",
            }
        ],
    }
    path = tmp_path / "owned-contract.json"
    save(path, data)
    report = grade(path, tmp_path / "owned-run")
    assert report["status"] == "pass", report
    assert report["schema"] == 16
    assert report["mismatches"] == 0
    assert report["mutants"][0]["detected"]
    observations = [
        json.loads(line)
        for line in (tmp_path / "owned-run/reference.stdout").read_text().splitlines()
        if '"return"' in line
    ]
    assert any(row["return"]["bytes"] == "" for row in observations)


def test_owned_bytes_rejects_unknown_allocator_and_non_scalar_length(tmp_path):
    path, data = contract(tmp_path)
    data["functions"] = [
        {
            "c": "fold",
            "rust": "candidate::fold",
            "return": {
                "type": "owned_bytes",
                "allocator": "unknown",
                "nullable": False,
                "length": "x",
            },
            "args": [{"name": "x", "type": "u32", "values": [0, 1]}],
        }
    ]
    data["mutants"] = [{"name": "one", "find": "^", "replace": "// mutant\n"}]
    save(path, data)
    with pytest.raises(ContractError, match="allocator must be libc or custom"):
        load_contract(path)

    data["functions"][0]["return"]["allocator"] = "libc"
    data["functions"][0]["return"]["length"] = "missing"
    save(path, data)
    with pytest.raises(ContractError, match="scalar argument"):
        load_contract(path)


def test_custom_owned_byte_result_uses_the_paired_deallocator(tmp_path):
    (tmp_path / "custom.c").write_text(
        "#include <stdint.h>\n#include <stdlib.h>\n"
        "static int active;\n"
        "static uint32_t active_tag;\n"
        "static void *pool_alloc(size_t n) {\n"
        "  if (active) return 0;\n"
        "  void *p = malloc(n ? n : 1); if (p) active = 1; return p;\n}\n"
        "void pool_release(uint32_t tag, uint8_t marker, void *p) {\n"
        "  if (p && tag == active_tag && marker == 0x5a) { active = 0; free(p); }\n}\n"
        "void *copy_owned(const uint8_t *src, uint32_t length, uint32_t tag) {\n"
        "  uint8_t *out = pool_alloc(length); if (!out) return 0;\n"
        "  active_tag = tag;\n"
        "  for (uint32_t i = 0; i < length; ++i) out[i] = src[i] ^ 0x6d;\n"
        "  return out;\n}\n"
    )
    (tmp_path / "custom.rs").write_text(
        "use core::ffi::c_void;\n"
        'unsafe extern "C" { fn malloc(size: usize) -> *mut c_void; fn free(p: *mut c_void); }\n'
        "static mut ACTIVE: bool = false;\n"
        "static mut ACTIVE_TAG: u32 = 0;\n"
        "unsafe fn pool_alloc(n: usize) -> *mut u8 {\n"
        "  if ACTIVE { return core::ptr::null_mut(); }\n"
        "  let p = malloc(n.max(1)) as *mut u8; if !p.is_null() { ACTIVE = true; } p\n}\n"
        'pub unsafe extern "C" fn pool_release(tag: u32, marker: u8, p: *mut u8) {\n'
        "  if !p.is_null() && marker == 0x5a && tag == ACTIVE_TAG {\n"
        "    ACTIVE = false; ACTIVE_TAG = 0; free(p as *mut c_void);\n  }\n}\n"
        'pub unsafe extern "C" fn copy_owned(src: *const u8, length: u32, tag: u32) -> *mut u8 {\n'
        "  let out = pool_alloc(length as usize); if out.is_null() { return out; }\n"
        "  ACTIVE_TAG = tag;\n"
        "  for i in 0..length as usize { *out.add(i) = *src.add(i) ^ 0x6d; }\n"
        "  out\n}\n"
    )
    data = {
        "version": 1,
        "scope": "stateless-host",
        "validity": (
            "Each result belongs to its library's pool; "
            "the tag and marker are required to release it."
        ),
        "c_source": "custom.c",
        "rust_source": "custom.rs",
        "random_cases": 3,
        "seed": 7,
        "functions": [
            {
                "c": "copy_owned",
                "rust": "candidate::copy_owned",
                "return": {
                    "type": "owned_bytes",
                    "allocator": "custom",
                    "nullable": False,
                    "length": "length",
                    "c_free": "pool_release",
                    "rust_free": "candidate::pool_release",
                    "free_args": [
                        {"arg": "tag"},
                        {"type": "u8", "value": 90},
                    ],
                },
                "args": [
                    {"name": "src", "type": "const_bytes", "length": "length"},
                    {"name": "length", "type": "u32", "values": [0, 1, 3, 8]},
                    {"name": "tag", "type": "u32", "values": [1, 7]},
                ],
            }
        ],
        "mutants": [
            {
                "name": "wrong_custom_owned_byte",
                "find": "*out.add(i) = *src.add(i) ^ 0x6d;",
                "replace": "*out.add(i) = *src.add(i) ^ 0x6c;",
            }
        ],
    }
    path = tmp_path / "custom-contract.json"
    save(path, data)
    report = grade(path, tmp_path / "custom-run")
    assert report["status"] == "pass", report
    assert report["mutants"][0]["detected"]
    assert "diff_free_0" in (tmp_path / "custom-run/reference.c").read_text()
    assert "diff_free_0" in (tmp_path / "custom-run/candidate.rs").read_text()
    assert (
        "uint32_t a2, uint8_t free_arg_1, void * ptr"
        in (tmp_path / "custom-run/reference.c").read_text()
    )
    c_source = tmp_path / "custom.c"
    c_source.write_text(
        c_source.read_text().replace("uint8_t marker, void *p", "uint8_t marker, uint8_t *p")
    )
    wrong_free_type = grade(path, tmp_path / "wrong-free-type-run")
    assert wrong_free_type["status"] == "invalid"
    assert "build-c failed" in wrong_free_type["error"]


def test_stateful_sequence_isolated_and_checks_external_fixture_calls(tmp_path):
    (tmp_path / "stateful.c").write_text(
        "#include <stdint.h>\n"
        "static uint32_t state;\n"
        "void reset_state(void) { state = 0; }\n"
        "uint32_t remember_external(uint32_t key) { state = fixture_lookup(key); return state; }\n"
        "uint32_t read_state(void) { return state; }\n"
    )
    rust_source = (
        'unsafe extern "C" { fn fixture_lookup(key: u32) -> u32; }\n'
        "static mut STATE: u32 = 0;\n"
        'pub unsafe extern "C" fn reset_state() { STATE = 0; }\n'
        'pub unsafe extern "C" fn remember_external(key: u32) -> u32 { '
        "STATE = fixture_lookup(key); STATE }\n"
        'pub unsafe extern "C" fn read_state() -> u32 { STATE }\n'
    )
    (tmp_path / "stateful.rs").write_text(rust_source)
    data = {
        "version": 1,
        "scope": "stateful-host",
        "validity": "Each sequence starts with fresh process state and makes one scripted lookup.",
        "c_source": "stateful.c",
        "rust_source": "stateful.rs",
        "random_cases": 0,
        "seed": 31,
        "functions": [
            {"c": "reset_state", "rust": "candidate::reset_state", "return": "void", "args": []},
            {
                "c": "remember_external",
                "rust": "candidate::remember_external",
                "return": "u32",
                "args": [{"name": "key", "type": "u32", "values": [1, 7, 9]}],
            },
            {"c": "read_state", "rust": "candidate::read_state", "return": "u32", "args": []},
        ],
        "fixtures": [
            {
                "name": "lookup",
                "c": "fixture_lookup",
                "rust_decl": "fixture_lookup",
                "return": "u32",
                "args": [{"name": "key", "type": "u32"}],
            }
        ],
        "sequences": [
            {
                "name": "lookup_roundtrip",
                "steps": [
                    {"function": "reset_state"},
                    {"function": "remember_external"},
                    {"function": "read_state"},
                ],
                "fixtures": {
                    "lookup": [
                        {
                            "args": [{"step": 1, "arg": "key"}],
                            "return": 137,
                        }
                    ]
                },
            },
            {"name": "fresh_process", "steps": [{"function": "read_state"}]},
        ],
        "mutants": [
            {
                "name": "wrong_fixture_argument",
                "find": "STATE = fixture_lookup(key);",
                "replace": "STATE = fixture_lookup(key ^ 1);",
            },
            {
                "name": "missing_fixture_call",
                "find": "STATE = fixture_lookup(key);",
                "replace": "STATE = 0;",
            },
            {
                "name": "extra_fixture_call",
                "find": "STATE = fixture_lookup(key);",
                "replace": "STATE = fixture_lookup(key) + fixture_lookup(key);",
            },
            {
                "name": "wrong_state_read",
                "find": 'pub unsafe extern "C" fn read_state() -> u32 { STATE }',
                "replace": 'pub unsafe extern "C" fn read_state() -> u32 { STATE + 1 }',
            },
        ],
    }
    path = tmp_path / "stateful-contract.json"
    save(path, data)
    report = grade(path, tmp_path / "stateful-run")
    assert report["status"] == "pass", report
    assert report["scope"] == "stateful-host"
    assert report["schema"] == 16
    assert report["cases"] == 4
    assert all(mutant["detected"] for mutant in report["mutants"])
    assert all(function["detected_mutants"] for function in report["per_function"])
    cases = json.loads((tmp_path / "stateful-run/cases.json").read_text())
    assert cases[0]["fixtures"]["lookup"][0]["args"] == [1]
    observed = [
        json.loads(line)
        for line in (tmp_path / "stateful-run/reference.stdout").read_text().splitlines()
    ]
    assert observed[0]["steps"][1]["fixture_calls"] == [
        {"fixture": "lookup", "args": [1], "return": 137}
    ]
    assert observed[-1]["steps"][0]["return"] == 0
    assert (tmp_path / "stateful-run/reference.cases/case-0000.command.json").exists()
    (tmp_path / "stateful.rs").write_text(
        rust_source.replace("fn fixture_lookup(key: u32)", "fn fixture_lookup(key: u64)")
    )
    wrong_fixture_type = grade(path, tmp_path / "wrong-fixture-type-run")
    assert wrong_fixture_type["status"] == "invalid"
    assert "build-rust failed" in wrong_fixture_type["error"]


def test_right_guard_placement_keeps_records_aligned(tmp_path):
    # 2 + n bytes with n odd: flush against the right guard page the record would sit
    # at an odd address and UBSan would kill the genuine C before anything is graded.
    (tmp_path / "blob.c").write_text(
        "#include <stdint.h>\n"
        "typedef struct BLOB { uint16_t n; uint8_t data[1]; } BLOB;\n"
        "uint32_t sum(const BLOB *b) { uint32_t s = b->n;\n"
        "  for (uint16_t i = 0; i < b->n; i++) s += b->data[i]; return s; }\n"
    )
    (tmp_path / "blob.rs").write_text(
        'pub unsafe extern "C" fn sum(b: *const u8) -> u32 {\n'
        "    let n = (b as *const u16).read_unaligned();\n"
        "    let mut s = n as u32;\n"
        "    for i in 0..n as usize { s += *b.add(2 + i) as u32; }\n"
        "    s\n}\n"
    )
    record = {
        "layout": "BLOB",
        "base_size": 4,
        "fields": [
            {"name": "n", "offset": 0, "type": "u16", "count": 1, "values": [0, 1, 3]},
            {
                "name": "data",
                "offset": 2,
                "type": "u8",
                "count_from": "n",
                "min_count": 1,
                "values": [0, 7],
            },
        ],
    }
    data = {
        "version": 1,
        "scope": "stateless-host",
        "memory": "guard-pages",
        "seed": 1,
        "random_cases": 4,
        "validity": "synthetic odd-length record",
        "c_source": "blob.c",
        "rust_source": "blob.rs",
        "functions": [
            {
                "c": "sum",
                "rust": "candidate::sum",
                "return": "u32",
                "args": [
                    {
                        "name": "b",
                        "type": "const_bytes",
                        "c_type": "const BLOB *",
                        "rust_type": "*const u8",
                        "record": record,
                    }
                ],
            }
        ],
        "mutants": [{"name": "negated", "find": "s += *b", "replace": "s -= *b"}],
    }
    path = tmp_path / "blob.json"
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "pass", report
    cases = json.loads((tmp_path / "run/cases.json").read_text())
    assert any(
        c["placement"] == "right" and len(bytes.fromhex(c["allocations"]["b"])) % 2 for c in cases
    )
    assert all(c["alignments"] == {"b": 2} for c in cases)


def test_mutant_find_tolerates_whitespace_reflow_but_stays_unique():
    from rustconv.difftest_contract import apply_mutant

    source = "fn f(x: u32) -> u32 {\n    if x >\n        1 { x } else { 0 }\n}\n"
    mutant = {"name": "m", "find": "            if x > 1 {", "replace": "  if x > 2 {"}
    assert "if x > 2 { x }" in apply_mutant(source, mutant)
    with pytest.raises(ContractError, match="found 2"):
        apply_mutant(source + source, mutant)
    with pytest.raises(ContractError, match="found 0"):
        apply_mutant(source, {"name": "m", "find": "if y > 1", "replace": "if y > 2"})


def test_mismatch_survives_a_failing_mutant_phase(tmp_path):
    path, data = contract(tmp_path)
    source = tmp_path / "candidate.rs"
    source.write_text(RUST.replace("{ x ^ 85 }", "{ x ^ 86 }"))
    data["mutants"][0]["find"] = "text that is not in the candidate"
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "mismatch", report
    assert report["mismatches"] > 0
    assert "found 0" in report["mutant_error"]
    assert "error" not in report


def test_candidate_fault_on_a_clean_input_is_a_mismatch(tmp_path):
    path, _ = contract(tmp_path)
    source = tmp_path / "candidate.rs"
    source.write_text(RUST.replace("{ x ^ 85 }", "{ if x == 1 { *(8 as *mut u32) = 0; } x ^ 85 }"))
    report = grade(path, tmp_path / "run")
    assert report["status"] == "mismatch", report
    assert report["candidate_fault"]["phase"] == "calling"
    assert report["candidate_fault"]["signal"] == 11
    example = json.loads((tmp_path / "run/counterexample.json").read_text())
    assert example["input"]["args"]["x"] == 1
    assert "fault" in example["candidate"]
    assert "error" not in report


def test_alignment_waiver_must_be_justified(tmp_path):
    path, data = contract(tmp_path)
    data["c_allow_ub"] = ["alignment"]
    save(path, data)
    with pytest.raises(ContractError, match="justify"):
        load_contract(path)
    data["validity"] += " The C reads WORDs misaligned; x86-64 tolerates the alignment."
    save(path, data)
    assert load_contract(path)["c_allow_ub"] == ["alignment"]
    data["c_allow_ub"] = ["signed-integer-overflow"]
    save(path, data)
    with pytest.raises(ContractError, match="alignment"):
        load_contract(path)


def test_byte_examples_set_the_payload_and_its_length(tmp_path):
    path, data = contract(tmp_path)
    data["functions"][1]["args"][0]["examples"] = ["00ff10", ""]
    save(path, data)
    report = grade(path, tmp_path / "run")
    assert report["status"] == "pass", report
    cases = json.loads((tmp_path / "run/cases.json").read_text())
    seeded = [c for c in cases if c["function"] == 1 and c["allocations"]["src"] == "00ff10"]
    assert seeded and all(c["args"]["n"] == 3 for c in seeded)
    assert any(c["function"] == 1 and c["args"]["n"] == 0 for c in cases)
