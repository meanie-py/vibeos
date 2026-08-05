"""Controls for the ctype-table generator.

The generator's whole claim is that it does not TRANSCRIBE: it derives 514 values from
ctype.c's own initializers against the msvcrt-sys island's mask values, and refuses to emit
unless every one equals the byte the C compiler actually put in `ctype.c.obj`'s `.data`.
Both halves of that claim need a control, because a proof that cannot fail is not a proof —
a `prove()` that silently returned on mismatch would look exactly like a passing one.

So: `evaluate` must resolve the abbreviations and the `_MSVCR_VER`-gated arms the way the
build does, and `prove` must EXIT on a single wrong entry.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from gen_ctype_tables import ABBREV, VER_GATED, evaluate, initializer, prove

# The real ctype.h bit positions, as the island reports them.
MASKS = {
    "_UPPER": 1,
    "_LOWER": 2,
    "_DIGIT": 4,
    "_SPACE": 8,
    "_PUNCT": 16,
    "_CONTROL": 32,
    "_BLANK": 64,
    "_HEX": 128,
}


def test_the_abbreviations_resolve_to_the_island_values():
    # ctype.c's own shorthand, from its header comment. `_S_|_C_` is a real entry.
    vals = evaluate(["_S_|_C_", "_D_|_H_", "_U_|_H_", "0"], MASKS)
    assert vals == [8 | 32, 4 | 128, 1 | 128, 0]


def test_the_version_gated_arms_use_this_dlls_setting():
    # At _MSVCR_VER=0: B110 is _BLANK, D120 is 4, and S140/L140/C140 are 0. Getting any of
    # these from a different _MSVCR_VER silently reclassifies real characters.
    vals = evaluate(["0x0028 | B110", "0x0010 | D120", "0x0020 | S140", "0x0010 | L140"], MASKS)
    assert vals == [0x0028 | 64, 0x0010 | 4, 0x0020, 0x0010]


def test_every_gated_name_is_resolvable():
    # A name in VER_GATED that is neither a mask nor an integer would raise at eval time
    # for one specific table entry — i.e. only when that arm is actually used.
    for name, (value, _why) in VER_GATED.items():
        assert value in MASKS or value.lstrip("-").isdigit(), (name, value)
    for abbrev, real in ABBREV.items():
        assert real in MASKS, (abbrev, real)


def test_prove_rejects_a_single_wrong_entry():
    """THE control. One byte off in 257 must be fatal, not a warning."""
    derived = [0] * 257
    compiled = [0] * 257
    compiled[65] = 0x81  # the object says 'A' is _UPPER|_HEX; the derivation says 0
    with pytest.raises(SystemExit) as e:
        prove("MSVCRT__ctype", ["0"] * 257, derived, compiled)
    assert "do NOT ship" in str(e.value)


def test_prove_accepts_an_exact_match():
    vals = list(range(257))
    assert prove("MSVCRT__ctype", ["x"] * 257, vals, list(vals)) is None


def test_an_out_of_range_entry_is_fatal():
    with pytest.raises(SystemExit) as e:
        evaluate(["0x1FFFF"], MASKS)
    assert "does not fit a WORD" in str(e.value)


def test_initializer_demands_exactly_257_entries():
    # 256 would be the natural mistake — the table is 257 because index 0 is the EOF slot
    # that `pctype[-1]` addresses. A short parse must fail loudly, not emit a short table.
    src = "WORD MSVCRT__ctype [257] = {" + ", ".join(["0"] * 256) + "};"
    with pytest.raises(SystemExit) as e:
        initializer(src, "WORD MSVCRT__ctype [257]")
    assert "expected 257" in str(e.value)


def test_initializer_strips_the_c_row_comments():
    src = (
        "WORD MSVCRT__wctype[257] =\n{\n"
        + ",\n".join([f"/* {i:02x} */ 0" for i in range(257)])
        + "\n};"
    )
    entries = initializer(src, "WORD MSVCRT__wctype[257]")
    assert len(entries) == 257
    assert all(e == "0" for e in entries)


def test_the_committed_tables_still_match_the_c_and_the_object():
    """End-to-end: the real ctype.c, the real island, and (when built) the real object.

    This is the drift half. It is the same thing `vibeos-cli check` runs, kept here so a
    `pytest tools/` alone still notices ctype.c moving under the committed Rust.
    """
    from gen_ctype_tables import BEGIN, END, build_block, island_masks

    target = Path(__file__).resolve().parent.parent / (
        "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs"
    )
    text = target.read_text()
    have = text.split(BEGIN, 1)[1].split(END, 1)[0].strip()
    assert have == build_block(island_masks()).strip()
