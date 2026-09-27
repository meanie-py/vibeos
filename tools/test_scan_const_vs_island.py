"""scan_const_contradictions --vs-island: a hand copy that matches no bindgen value."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_const_contradictions import island_disagreements

FSRTL = "sdk/lib/rust/fsrtl-rust/src/filelock.rs"
ISLAND = {
    "_RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan": {1},
    "_RTL_GENERIC_COMPARE_RESULTS_GenericEqual": {2},
    "ATA_FLAGS_DATA_IN": {0x02, 0x04},  # two headers, both right (uniata's private one)
}


def test_the_fsrtl_swap_is_reported():
    decls = {
        "_RTL_GENERIC_COMPARE_RESULTS_GenericEqual": [(FSRTL, 1, "common")],
        "_RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan": [(FSRTL, 2, "common")],
    }
    hits = island_disagreements(decls, ISLAND)
    assert {h[0] for h in hits} == set(decls), hits


def test_matching_any_island_value_is_clean():
    decls = {"ATA_FLAGS_DATA_IN": [("sdk/lib/rust/uniata-rust/src/x.rs", 0x04, "common")]}
    assert island_disagreements(decls, ISLAND) == []


def test_a_name_no_island_knows_is_out_of_scope():
    assert island_disagreements({"MY_PRIVATE_SCALE": [("a.rs", 7, "common")]}, ISLAND) == []


def test_a_not_mask_matches_its_32_bit_spelling():
    """`!0x0` and 0xFFFFFFFF are one 32-bit value; widths must not invent a finding."""
    decls = {"EXCEPTION_CHAIN_END": [("a.rs", ~0, "common")]}
    assert island_disagreements(decls, {"EXCEPTION_CHAIN_END": {0xFFFFFFFF}}) == []
