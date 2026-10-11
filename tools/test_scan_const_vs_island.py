"""scan_const_contradictions --vs-island: a hand copy that matches no bindgen value."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools import scan_const_contradictions as scc
from tools.scan_const_contradictions import island_disagreements, within_crate

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


def test_pointer_cast_flag_constants_are_read():
    # fastfat carried FSRTL_CACHE_TOP_LEVEL_IRP as `(-2isize) as *mut IRP` and as
    # `0x02 as *mut IRP`; neither form was parsed, so the four-way disagreement was invisible.
    cases = {
        "const FSRTL_CACHE_TOP_LEVEL_IRP: *mut IRP = (-2isize) as *mut IRP;": -2,
        "const FSRTL_CACHE_TOP_LEVEL_IRP: *mut IRP = 0x02 as *mut IRP;": 2,
        "    const FSRTL_FSP_TOP_LEVEL_IRP: PIRP = 1 as PIRP;": 1,
        "const MM_SYSLDR_NO_IMPORTS: usize = (-2isize) as usize;": -2,
        "const X: u32 = 0x10u32;": 16,
    }
    for line, want in cases.items():
        m = scc.CONST_RE.match(line)
        assert m, line
        assert scc.parse_value(m.group(2)) == want, line


def test_sixteen_bit_spellings_agree_but_wide_values_do_not():
    assert not scc.disagrees({-1, 0xFFFF})  # VARIANT_TRUE spelled both ways
    assert scc.disagrees({2, -2})  # the fastfat bug
    assert scc.disagrees({0x10000, 0})  # never forgiven by the 16-bit rule


USERENV = "sdk/lib/rust/userenv-rust/src/"


def test_within_crate_reports_userenvs_profilepath():
    """Two values in ONE crate: GUI setup's "InitializeProfiles() failed"."""
    decls = {
        "IDS_PROFILEPATH": [
            (USERENV + "prelude.rs", 1, "common"),
            (USERENV + "userenv/setup.rs", 0, "common"),
        ]
    }
    hits = within_crate(decls)
    assert {h[1] for h in hits} == {USERENV + "prelude.rs", USERENV + "userenv/setup.rs"}, hits


def test_within_crate_ignores_two_crates_meaning_two_things():
    decls = {
        "IDC_PROPERTIES": [
            ("sdk/lib/rust/shell32-rust/src/a.rs", 2, "common"),
            ("sdk/lib/rust/comctl32-rust/src/b.rs", 0x58A, "common"),
        ]
    }
    assert within_crate(decls) == []


def test_within_crate_ignores_a_per_arch_override():
    decls = {"PAGE_SHIFT": [(USERENV + "a.rs", 12, "amd64"), (USERENV + "b.rs", 16, "arm64")]}
    assert within_crate(decls) == []


def test_island_collection_sees_camel_case_enum_members():
    """winsrv's hand copy of USERTHREADINFOCLASS was invisible to the ALL_CAPS collector."""
    m = scc.ANY_CONST_RE.match("const UserThreadUseActiveDesktop: u32 = 0;")
    assert m is not None and m.group(1) == "UserThreadUseActiveDesktop"
    hits = island_disagreements(
        {"UserThreadUseActiveDesktop": [("sdk/lib/rust/winsrv-rust/src/x.rs", 0, "common")]},
        {"UserThreadUseActiveDesktop": {7}},
    )
    assert hits == [("UserThreadUseActiveDesktop", "sdk/lib/rust/winsrv-rust/src/x.rs", 0)]


def test_the_cross_crate_check_keeps_all_caps_only():
    assert scc.CONST_RE.match("const DelayedWorkQueue: u32 = 0;") is None


def test_own_island_catches_a_constant_borrowed_from_another_crate():
    """atapi's prelude carried usbxhci's MAX_SLOTS (16); its own island says 32. --vs-island
    accepts it because usbxhci's island has 16 -- only the crate's own island can say no."""
    atapi = "sdk/lib/rust/atapi-rust/src/prelude.rs"
    own = {"atapi-rust": {"MAX_SLOTS": {32}}, "usbxhci-rust": {"MAX_SLOTS": {16}}}
    hits = scc.own_island_shadows({"MAX_SLOTS": [(atapi, 16, "common")]}, own)
    assert hits == [("MAX_SLOTS", atapi, 16)]
    usb = "sdk/lib/rust/usbxhci-rust/src/controller.rs"
    assert scc.own_island_shadows({"MAX_SLOTS": [(usb, 16, "common")]}, own) == []


def test_own_island_forgives_the_same_bits_at_another_width():
    psn = "sdk/lib/rust/openglcfg-rust/src/prelude.rs"
    own = {"openglcfg-rust": {"PSN_APPLY": {-202}}}
    assert scc.own_island_shadows({"PSN_APPLY": [(psn, 0xFFFFFF36, "common")]}, own) == []
