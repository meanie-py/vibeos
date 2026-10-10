"""Pins apply_rustc_suggestions: a fix is applied only where every arch agrees."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import apply_rustc_suggestions as a

MUT = (10, 14, "")


def test_an_edit_needs_every_arch_that_compiled_the_file():
    # amd64 says the `mut` is unused; i386 compiled the same file and did not (a cfg arm).
    sugg = {"amd64": {"x.rs": {MUT}}, "i386": {"x.rs": set()}}
    compiled = {"amd64": {"x.rs"}, "i386": {"x.rs"}}
    assert a.agreed(sugg, compiled) == {}


def test_a_file_one_arch_compiles_needs_only_that_arch():
    sugg = {"i386": {"types_i386.rs": {MUT}}}
    compiled = {"i386": {"types_i386.rs"}, "amd64": {"other.rs"}}
    assert a.agreed(sugg, compiled) == {"types_i386.rs": [MUT]}


def test_overlapping_edits_keep_the_first():
    sugg = {"amd64": {"x.rs": {(10, 20, ""), (15, 25, "")}}}
    assert a.agreed(sugg, {"amd64": {"x.rs"}}) == {"x.rs": [(10, 20, "")]}


def _diag(expansion):
    span = {
        "suggestion_applicability": "MachineApplicable",
        "suggested_replacement": "",
        "file_name": "x.rs",
        "byte_start": 1,
        "byte_end": 5,
        "expansion": None,
    }
    return {
        "spans": [{"expansion": expansion}],
        "children": [{"spans": [span], "children": []}],
    }


def test_a_diagnostic_inside_a_macro_expansion_is_never_fixed():
    assert a._suggestions(_diag(None)) == [("x.rs", (1, 5, ""))]
    assert a._suggestions(_diag({"macro_decl_name": "m!"})) == []


def test_a_float_from_bits_rewrite_is_refused():
    d = _diag(None)
    d["children"][0]["spans"][0]["suggested_replacement"] = "f64::from_bits(bits)"
    assert a._suggestions(d) == []


def test_a_deletion_at_line_end_leaves_no_trailing_space():
    src = b"    (x == 0) as i32 )\n"
    end = src.index(b")\n")
    assert a.apply_edits(src, [(end, end + 1, "")]) == b"    (x == 0) as i32\n"
