"""Membership reuse must preserve orphan findings, not suppress them."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import rust_module_tree as module_tree
import scan_orphan_modules as scanner


def test_normalized_membership_is_reused_only_within_scan(tmp_path, monkeypatch):
    crate = tmp_path / "crate"
    src = crate / "src"
    src.mkdir(parents=True)
    (crate / "Cargo.toml").write_text('[package]\nname = "fixture"\nversion = "0.1.0"\n')
    files = {
        "lib.rs": "mod live;\n",
        "live.rs": "//! ORPHAN-BY-DESIGN: stale assertion\n",
        "orphan.rs": "fn unused() {}\n",
        "parked.rs": "//! ORPHAN-BY-DESIGN: deliberately parked\n",
    }
    for name, text in files.items():
        (src / name).write_text(text)
    membership = [str(src / "lib.rs"), str(src / "live.rs")]
    calls = []

    def live_files(root):
        calls.append(root)
        return membership

    monkeypatch.setattr(scanner, "crate_live_files", live_files)
    findings, declared, rootless, stale = scanner.scan([str(crate)])
    assert findings == [(str(src / "orphan.rs"), None)]
    assert declared == [(str(src / "parked.rs"), "deliberately parked")]
    assert rootless == []
    assert stale == [str(src / "live.rs")]
    assert len(calls) == 1

    # A fresh invocation must rebuild its normalized set, not reuse stale state.
    membership.append(str(src / "orphan.rs"))
    assert scanner.scan([str(crate)])[0] == []
    assert len(calls) == 2


def test_real_module_tree_and_symlink_membership(tmp_path):
    crate = tmp_path / "crate"
    src = crate / "src"
    src.mkdir(parents=True)
    (crate / "Cargo.toml").write_text('[package]\nname = "fixture"\nversion = "0.1.0"\n')
    (src / "lib.rs").write_text("mod live;\nmod missing;\n")
    (src / "live.rs").write_text("fn live() {}\n")
    (src / "alias.rs").symlink_to(src / "live.rs")
    (src / "orphan.rs").write_text("fn orphan() {}\n")
    findings, declared, rootless, stale = scanner.scan([str(crate)])
    assert findings == [(str(src / "orphan.rs"), None)]
    assert (declared, rootless, stale) == ([], [], [])
    phantoms = scanner.scan_phantoms([str(crate)])
    assert len(phantoms) == 1
    assert phantoms[0][1] == "missing"


def test_leaf_fast_path_skips_parsing_but_not_reachability(tmp_path, monkeypatch):
    crate = tmp_path / "crate"
    src = crate / "src"
    src.mkdir(parents=True)
    (crate / "Cargo.toml").write_text('[package]\nname = "fixture"\nversion = "0.1.0"\n')
    (src / "lib.rs").write_text('mod leaf;\ninclude!("included.rs");\n')
    leaf = "pub fn model() -> usize { 42 }\n" * 100
    (src / "leaf.rs").write_text(leaf)
    (src / "included.rs").write_text("mod missing;\n")
    original = module_tree.strip_comments

    def parse(text):
        assert text != leaf, "plain leaf must not enter the expensive parser"
        return original(text)

    monkeypatch.setattr(module_tree, "strip_comments", parse)
    live = module_tree.crate_live_files(str(crate), _cache={})
    assert live == {str(src / name) for name in ("lib.rs", "leaf.rs", "included.rs")}
    assert module_tree.decl_sites(leaf) == []
    assert module_tree.decl_sites("/* mod fake; */ mod real;") == [("real", [])]
    assert module_tree.decl_sites("mod/* separator */real;") == [("real", [])]
    assert not module_tree.crate_is_opaque(str(crate), _cache={})


def test_dynamic_include_still_marks_crate_opaque(tmp_path):
    src = tmp_path / "src"
    src.mkdir()
    (tmp_path / "Cargo.toml").write_text('[package]\nname = "fixture"\nversion = "0.1.0"\n')
    (src / "lib.rs").write_text('include!(env!("GENERATED"));\n')
    assert module_tree.crate_is_opaque(str(tmp_path), _cache={})
