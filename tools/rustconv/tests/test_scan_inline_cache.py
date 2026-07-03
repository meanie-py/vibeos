"""Content-hash cache + tree-sitter-query collector for scan_inline_flag_literals.

The scanner re-runs on a near-unchanged Rust tree every gate build; a per-blob
content cache takes the warm sweep from ~22s to <1s. These pin the cache's
generation/versioning logic (the part that silently rots) and confirm the
query-based collector stays behavior-identical to the old recursive walk.
"""

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[3]

# scan_inline_flag_literals lives in tools/ (not the rustconv package); load it by
# path. It self-inserts tools/rustconv on sys.path for its verify_constants import.
_spec = importlib.util.spec_from_file_location(
    "scan_inline_flag_literals", REPO / "tools" / "scan_inline_flag_literals.py"
)
assert _spec and _spec.loader
S = importlib.util.module_from_spec(_spec)
sys.modules["scan_inline_flag_literals"] = S
_spec.loader.exec_module(S)


@pytest.fixture
def cache_path(tmp_path, monkeypatch):
    p = tmp_path / "cache.json"
    monkeypatch.setattr(S, "_CACHE_PATH", p)
    return p


def test_save_load_roundtrip(cache_path):
    entries = {"deadbeef": [[10, 0x2, "FOO", 0x4, "let x = 0x2;"]], "cafe": []}
    S._save_cache("fp-A", entries)
    assert S._load_cache("fp-A") == entries


def test_generation_mismatch_returns_empty(cache_path):
    S._save_cache("fp-A", {"sha": [[1, 2, "N", 3, "l"]]})
    # a different generation fingerprint (header/logic change) must not reuse
    assert S._load_cache("fp-B") == {}


def test_version_bump_invalidates(cache_path, monkeypatch):
    S._save_cache("fp-A", {"sha": [[1, 2, "N", 3, "l"]]})
    monkeypatch.setattr(S, "_CACHE_VERSION", S._CACHE_VERSION + 1)
    assert S._load_cache("fp-A") == {}


def test_corrupt_cache_degrades(cache_path):
    cache_path.write_text("{ this is not json")
    assert S._load_cache("fp-A") == {}  # never raises — cache is best-effort


def test_generations_lru_capped(cache_path):
    for i in range(S._CACHE_MAX_GENS + 3):
        S._save_cache(f"fp-{i}", {f"sha-{i}": []})
    import json

    blob = json.loads(cache_path.read_text())
    assert len(blob["gens"]) == S._CACHE_MAX_GENS
    # most-recent-first, older generations evicted
    assert blob["gens"][0]["fp"] == f"fp-{S._CACHE_MAX_GENS + 2}"
    assert all(g["fp"] != "fp-0" for g in blob["gens"])


def test_multiple_generations_coexist(cache_path):
    S._save_cache("fp-A", {"a": [[1, 2, "N", 3, "l"]]})
    S._save_cache("fp-B", {"b": []})
    # both survive (distinct generations, e.g. gate no-headers vs a --headers run)
    assert S._load_cache("fp-A") == {"a": [[1, 2, "N", 3, "l"]]}
    assert S._load_cache("fp-B") == {"b": []}


def test_generation_fp_tracks_catalog():
    fp1 = S._generation_fp({"FOO": 1, "BAR": 2})
    fp2 = S._generation_fp({"FOO": 1, "BAR": 2})
    fp3 = S._generation_fp({"FOO": 1, "BAR": 3})  # value changed
    assert fp1 == fp2  # deterministic for identical catalogs
    assert fp1 != fp3  # a changed constant value rolls the generation


def test_query_collector_matches_recursive_walk():
    """The tree-sitter query must capture exactly the integer_literals + comments
    a manual pre-order DFS would — otherwise findings (and the baseline) shift."""
    src = b"""
fn f() {
    let a = 0x2; // FOO
    /* BAR */ let b = 0x40u32;
    let c = (0x4 /* BAZ */ | 0x8);
}
"""
    root = S._PARSER.parse(src).root_node

    def recursive(node, lits, comments):
        if node.type == "integer_literal":
            lits.append(node.start_byte)
        elif node.type in ("line_comment", "block_comment"):
            comments.append(node.start_byte)
        for ch in node.children:
            recursive(ch, lits, comments)

    r_lits, r_comments = [], []
    recursive(root, r_lits, r_comments)
    q_lits, q_comments = S.collect(root)
    assert sorted(r_lits) == [n.start_byte for n in q_lits]
    assert sorted(r_comments) == [n.start_byte for n in q_comments]
