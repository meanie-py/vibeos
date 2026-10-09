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


# ── comment placement (2026-10-08) ─────────────────────────────────────────────
# Each case is a shape that shipped wrong with the constant's name sitting beside it.
_CAT = {
    "RmAuditSetCommand": 1,
    "RmCreateLogonSession": 2,
    "LPC_CONNECTION_REQUEST": 10,
    "LPC_PORT_CLOSED": 5,
    "LPC_CLIENT_DIED": 6,
    "DBG_CONTROL_C": 0x40010005,
    "ERROR_INVALID_PARAMETER": 87,
}


def _scan(tmp_path, body):
    f = tmp_path / "x.rs"
    f.write_text("fn f() {\n" + body + "\n}\n")
    return sorted((name, lv) for _ln, lv, name, _want, _src in S.scan_file(f, _CAT))


def test_match_arm_names_an_enum_member(tmp_path):
    # se-rust's SRM dispatch: RM_API_NUMBER starts at 1, the arms said 0/1.
    body = (
        "match n {\n0 => a(), // RmAuditSetCommand\n1 => b(), // RmCreateLogonSession\n_ => c(),\n}"
    )
    assert _scan(tmp_path, body) == [("RmAuditSetCommand", 0), ("RmCreateLogonSession", 1)]


def test_comment_alone_under_a_block_opener(tmp_path):
    body = (
        "if t == 5 {\n// LPC_CONNECTION_REQUEST\nx();\n}\n"
        "if t == 4 || t == 3 {\n// LPC_PORT_CLOSED || LPC_CLIENT_DIED\nbreak;\n}"
    )
    assert _scan(tmp_path, body) == [
        ("LPC_CLIENT_DIED", 3), ("LPC_CONNECTION_REQUEST", 5), ("LPC_PORT_CLOSED", 4)
    ]  # fmt: skip


def test_comment_alone_over_a_lone_literal(tmp_path):
    # kernel32's console handler raised exception code 0 for DBG_CONTROL_C.
    body = "let c = if n == 0 {\n// DBG_CONTROL_C\n0\n} else { 1 };"
    assert _scan(tmp_path, body) == [("DBG_CONTROL_C", 0)]


def test_opener_zero_is_a_boolean_test(tmp_path):
    # rtl-heap: the comment names what the block does, not the `== 0`.
    body = "if free(p) == 0u8 {\n/* ERROR_INVALID_PARAMETER */\nset_error();\n}"
    assert _scan(tmp_path, body) == []
