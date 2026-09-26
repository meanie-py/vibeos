"""Oracles for the sharded Tier 2 sweep.

The failure this guards is FALSE COMPLETENESS: a sweep that stopped early must
never read as one that finished. That is not hypothetical -- the single-boot
harness produced order-dependent PREFIXES of the export table for weeks, and
nothing in the output said so.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from ucrt_tier2_sweep import parse_serial


def test_parses_export_rows_and_done_fields() -> None:
    rows, done = parse_serial(
        "[UCRTDIFF] EXPORT wctob MATCH n=40\n"
        "[UCRTDIFF] EXPORT atoi DIFF rust=3 dll=0 of 23 first=atoi:2147483648\n"
        "[UCRTDIFF] done cases=100 dll_pass=100 dll_fail=0 rust_pass=97 rust_fail=3 "
        "shard=0/8 exports=28 skipped_shard=192 partial=0 "
        "zombie_faults=0 dropped_writes=0\n"
    )
    assert rows["wctob"] == "MATCH n=40"
    assert rows["atoi"].startswith("DIFF rust=3")
    assert done is not None
    assert done["partial"] == "0"
    assert done["shard"] == "0/8"


def test_truncated_run_has_no_done_line() -> None:
    """A run cut off mid-sweep must be distinguishable from a clean one."""
    rows, done = parse_serial("[UCRTDIFF] EXPORT wctob MATCH n=40\n")
    assert rows and done is None


def test_deadline_truncation_is_reported_not_hidden() -> None:
    """partial>0 is the harness saying its own deadline cut the run short."""
    _rows, done = parse_serial(
        "[UCRTDIFF] done cases=9 dll_pass=9 dll_fail=0 rust_pass=9 rust_fail=0 "
        "shard=0/8 exports=4 skipped_shard=192 partial=24 "
        "zombie_faults=0 dropped_writes=0\n"
    )
    assert done is not None and done["partial"] == "24"


def test_zombie_write_accounting_is_visible() -> None:
    """dropped_writes>0 means an abandoned thread tried to corrupt a verdict and
    was fenced. It must reach the log: silently dropping it would hide the very
    bug the fence was added for."""
    _rows, done = parse_serial(
        "[UCRTDIFF] done cases=9 dll_pass=9 dll_fail=0 rust_pass=9 rust_fail=0 "
        "shard=3/8 exports=27 skipped_shard=193 partial=0 "
        "zombie_faults=17 dropped_writes=4021\n"
    )
    assert done is not None
    assert done["zombie_faults"] == "17"
    assert done["dropped_writes"] == "4021"


def test_generator_gates_every_export() -> None:
    """Every emitted export must sit behind `if (export_begin(...))`. A bare call
    would run regardless of shard and regardless of the deadline, which is exactly
    the single-boot behaviour this replaced."""
    inc = Path(__file__).resolve().parent / "payloads" / "ucrtdiff" / "generated.inc"
    if not inc.exists():  # built artifact; absent on a fresh checkout
        return
    text = inc.read_text()
    gated = text.count("if (export_begin(")
    begins = text.count("export_begin(")
    assert gated == begins, f"{begins - gated} ungated export_begin call(s)"
    assert text.count("run_guarded(") == gated
