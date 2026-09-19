"""The pre-payment guard against re-buying a TU the output crate already ships.

Written after $2.06 of DeepSeek went on re-translating msvcrt's file.c, which had been
ported, wired and bug-fixed weeks earlier — the lane doc still listed it as owed, and
nothing in the pipeline looked at the crate before spending.
"""

from pathlib import Path

import rustconv.__main__ as main


def _crate(tmp_path: Path, **modules: str) -> Path:
    crate = tmp_path / "thing-rust"
    (crate / "src").mkdir(parents=True)
    for name, body in modules.items():
        (crate / "src" / f"{name}.rs").write_text(body)
    return crate


def test_blocks_when_the_majority_of_the_tu_is_already_no_mangle(tmp_path):
    crate = _crate(
        tmp_path,
        thing=(
            '#[no_mangle]\npub unsafe extern "C" fn alpha() {}\n'
            '#[no_mangle]\npub unsafe extern "C" fn beta() {}\n'
        ),
    )
    hit = main._already_translated(Path("thing.c"), crate, {"alpha", "beta", "gamma"})
    assert hit is not None
    where, hits, total = hit
    assert (hits, total) == (2, 3)
    assert where == [crate / "src" / "thing.rs"]


def test_matches_by_symbol_so_a_renamed_module_cannot_hide(tmp_path):
    """msvcrt's wcs.c ships as wcs_rest.rs — the obvious `<stem>.rs` lookup misses it,
    which is exactly the shape that would let the same mistake happen twice."""
    crate = _crate(
        tmp_path,
        wcs_rest=(
            '#[no_mangle]\npub unsafe extern "C" fn alpha() {}\n'
            '#[no_mangle]\npub unsafe extern "C" fn beta() {}\n'
        ),
    )
    assert main._already_translated(Path("wcs.c"), crate, {"alpha", "beta"}) is not None


def test_export_name_alias_counts(tmp_path):
    """Regression: the index scans MASKED source so a commented-out item cannot match,
    and masking blanks string CONTENTS — which is exactly where export_name keeps the
    symbol. Read masked, this resolved to a name of four spaces and matched nothing."""
    crate = _crate(
        tmp_path,
        m='#[export_name = "alpha"]\npub unsafe extern "C" fn rust_alpha() {}\n',
    )
    assert main._already_translated(Path("m.c"), crate, {"alpha"}) is not None


def test_a_commented_out_export_is_not_a_port(tmp_path):
    """The other half of the same trade-off: scanning masked text is what buys this."""
    crate = _crate(
        tmp_path,
        m='// #[no_mangle] pub unsafe extern "C" fn alpha() {}\n',
    )
    assert main._already_translated(Path("m.c"), crate, {"alpha"}) is None


def test_a_few_shared_helpers_do_not_block(tmp_path):
    """msvcrt's main.c shares 1 of its 4 functions with the crate and is NOT ported."""
    crate = _crate(tmp_path, other='#[no_mangle]\npub unsafe extern "C" fn alpha() {}\n')
    assert main._already_translated(Path("m.c"), crate, {"alpha", "b", "c", "d"}) is None


def test_internal_fns_are_not_a_port(tmp_path):
    """Only #[no_mangle]/#[export_name] displaces the C; plain fns are scaffolding."""
    crate = _crate(tmp_path, m="pub unsafe fn alpha() {}\npub unsafe fn beta() {}\n")
    assert main._already_translated(Path("m.c"), crate, {"alpha", "beta"}) is None


def test_no_crate_on_disk_is_not_a_block(tmp_path):
    assert main._already_translated(Path("m.c"), tmp_path / "absent-rust", {"alpha"}) is None
