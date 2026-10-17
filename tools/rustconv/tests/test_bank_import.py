"""bank_import: a sibling worktree's banks merge in without ever replacing ours."""

from __future__ import annotations

import json
from pathlib import Path

from rustconv import bank_import


def _bank(root: Path, ns: str, files: dict[str, str], claim: str | None) -> Path:
    d = root / ns
    d.mkdir(parents=True)
    manifest = {}
    for name, text in files.items():
        (d / f"{name}.rs").write_text(text)
        manifest[name] = {"file": f"{name}.rs", "hash": f"h-{name}"}
    (d / "manifest.json").write_text(json.dumps(manifest))
    if claim is not None:
        (d / ".source").write_text(claim + "\n")
    return d


def test_missing_bodies_come_over_and_a_differing_one_is_kept_ours(tmp_path):
    unit = tmp_path / "stage" / "cppapp" / "cicbase_h.hpp"
    src, dst = tmp_path / "theirs", tmp_path / "ours"
    ns = "cppapp__cicbase_h__factseal_v1__callseal_v1"
    _bank(src, ns, {"a": "fn a() {}", "b": "fn b() {}"}, str(unit))
    _bank(dst, ns, {"a": "fn a() { /* ours */ }"}, str(unit))

    rep = bank_import.import_banks([unit], src, dst, tmp_path)

    assert rep.files == 1 and rep.entries == 1
    assert rep.conflicts == [f"{ns}/a.rs"]
    assert (dst / ns / "a.rs").read_text() == "fn a() { /* ours */ }"
    manifest = json.loads((dst / ns / "manifest.json").read_text())
    assert set(manifest) == {"a", "b"} and (dst / ns / "b.rs").read_text() == "fn b() {}"


def test_an_empty_shell_is_filled_and_a_dry_run_writes_nothing(tmp_path):
    # What a replay leaves behind on a miss: the namespace with only its claim.
    unit = tmp_path / "stage" / "cppapp" / "cicuif_h.hpp"
    src, dst = tmp_path / "theirs", tmp_path / "ours"
    ns = "cppapp__cicuif_h__factseal_v1__callseal_v1"
    _bank(src, ns, {"x": "fn x() {}"}, str(unit))
    (dst / ns).mkdir(parents=True)
    (dst / ns / ".source").write_text(str(unit) + "\n")

    dry = bank_import.import_banks([unit], src, dst, tmp_path, dry_run=True)
    assert (dry.files, dry.entries) == (1, 1)  # x.rs, and its entry merged into the manifest
    assert not (dst / ns / "manifest.json").exists()

    bank_import.import_banks([unit], src, dst, tmp_path)
    assert json.loads((dst / ns / "manifest.json").read_text())["x"]["file"] == "x.rs"


def test_a_namespace_claimed_by_another_file_and_other_modules_are_left_alone(tmp_path):
    unit = tmp_path / "a" / "cppapp" / "main_cpp.hpp"
    other = tmp_path / "b" / "cppapp" / "main_cpp.hpp"  # same `<dir>__<stem>`, other module
    src, dst = tmp_path / "theirs", tmp_path / "ours"
    _bank(src, "cppapp__main_cpp__factseal_v1", {"m": "fn m() {}"}, str(other))
    _bank(src, "cppapp__unrelated__factseal_v1", {"u": "fn u() {}"}, str(unit))

    rep = bank_import.import_banks([unit], src, dst, tmp_path)

    assert rep.foreign == ["cppapp__main_cpp__factseal_v1"]
    assert rep.files == 0 and not dst.exists()
