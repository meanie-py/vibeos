"""Shipped contracts over code no hand-written *-difftest crate grades.

cryptlib's MD4/MD5/SHA1 (records from arbitrary sampled context states), rtl's CRC32
(a selected static table) and rtl's run-encode pair (a UNICODE_STRING whose Buffer is
a Length-sized byte pointee, plus a scripted NtQuerySystemTime fixture) and rtl's LZNT1
(encoder-produced example streams, with mutants reaching the lznt1.rs support module).
"""

from pathlib import Path

import pytest

from rustconv.difftest_contract import ContractError, grade, load_contract, selected_rust

DATA = Path(__file__).resolve().parents[1] / "data/difftest"


@pytest.mark.parametrize("name", ["md4", "md5", "sha1", "crc32", "encode", "compress"])
def test_shipped_target_passes_with_every_mutant_detected(tmp_path, name):
    report = grade(DATA / f"{name}.json", tmp_path / name)
    assert report["status"] == "pass", report
    assert report["mismatches"] == 0
    assert all(m["detected"] for m in report["mutants"])


def test_byte_pointee_fault_proves_the_allocation_ends_at_length(tmp_path):
    report = grade(DATA / "encode.json", tmp_path / "encode")
    overread = next(m for m in report["mutants"] if m["name"] == "encode_overread")
    assert overread["fault"]["phase"] == "calling"


def test_byte_pointee_length_must_name_a_root_scalar(tmp_path):
    import json

    data = json.loads((DATA / "encode.json").read_text())
    for fn in data["functions"]:
        for arg in fn["args"]:
            if "record" in arg:
                arg["record"]["pointees"][0]["bytes"]["length"] = "Buffer"
    base = DATA.resolve()
    for key in ("c_source", "c_prelude", "rust_source"):
        data[key] = str((base / data[key]).resolve())
    data["c_include_dirs"] = [str((base / p).resolve()) for p in data["c_include_dirs"]]
    data["rust_support"] = {k: str((base / v).resolve()) for k, v in data["rust_support"].items()}
    path = tmp_path / "bad.json"
    path.write_text(json.dumps(data))
    with pytest.raises(ContractError, match="bytes length must name a root"):
        load_contract(path)


def test_selection_copies_non_function_items_and_guarded_twins():
    source = (
        "#![cfg_attr(not(test), no_std)]\n"
        "static TABLE: [u32; 2] = [1, 2];\n"
        "macro_rules! twice { ($x:expr) => { $x * 2 } }\n"
        "#[cfg(not(test))]\nfn pick() -> u32 { 1 }\n"
        "#[cfg(test)]\nfn pick() -> u32 { 2 }\n"
        "fn body() -> u32 { twice!(TABLE[1]) + pick() }\n"
    )
    out = selected_rust(source, ["TABLE", "twice", "pick", "body"], preserve_cfg=True)
    assert "static TABLE" in out and "macro_rules! twice" in out
    assert out.count("fn pick") == 2
    assert "no_std" not in out
    with pytest.raises(ContractError, match="duplicate"):
        selected_rust("fn a() {}\nfn a() {}\n", ["a"], preserve_cfg=True)
