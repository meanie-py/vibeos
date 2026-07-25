"""Static tables are replaced from the C, not diffed against it.

Every case here is a real shape mbcs.c produced across two runs of the same file. The
values came back right every time and the LENGTH came back wrong every time, in three
different ways — which is why this pass overwrites rather than validates.
"""

from __future__ import annotations

from rustconv.table_fidelity import c_arrays, repair_tables

C = """
static const unsigned char tbl[] = {
  0x40,0x49,0x68,0x94,
  0x90,0x93 };

static const unsigned char other[16] = { 1, 2 };
"""


def test_c_arrays_reads_unsized_tables_only():
    """A SIZED C declaration already states its length; it is not this pass's business."""
    found = c_arrays(C)
    assert found == {"tbl": ["0x40", "0x49", "0x68", "0x94", "0x90", "0x93"]}


def test_faithful_table_is_left_alone():
    rust = "pub static tbl: [u8; 6] = [\n    0x40,0x49,0x68,0x94,0x90,0x93,\n];"
    out, findings = repair_tables(rust, C)
    assert out == rust and findings == []


def test_padded_table_is_truncated_to_the_c():
    """mbcs.c run 1: 158 real elements plus 32 invented trailing zeros."""
    rust = "pub static tbl: [u8; 8] = [0x40,0x49,0x68,0x94,0x90,0x93,0x00,0x00];"
    out, findings = repair_tables(rust, C)
    assert "0x00" not in out
    assert "[u8; 6]" in out
    assert "the C's 6" in str(findings[0])


def test_declared_length_disagreeing_with_initialisers_is_corrected():
    """mbcs.c run 2: correct values under `[u8; 187]`. This shape cannot compile."""
    rust = "pub static tbl: [u8; 187] = [0x40,0x49,0x68,0x94,0x90,0x93];"
    out, findings = repair_tables(rust, C)
    assert "[u8; 6]" in out
    assert "declared 187 over 6 initialisers" in str(findings[0])


def test_wrong_values_are_replaced_with_the_cs_own_data():
    rust = "pub static tbl: [u8; 6] = [0x40,0x49,0x68,0x94,0x90,0xFF];"
    out, findings = repair_tables(rust, C)
    assert "0xFF" not in out and "0x93" in out
    assert "values differ from the C" in str(findings[0])


def test_hex_and_decimal_spellings_compare_equal():
    """`0x40` and `64` are the same byte; a spelling change is not a finding."""
    rust = "pub static tbl: [u8; 6] = [64,73,104,148,144,147];"
    out, findings = repair_tables(rust, C)
    assert out == rust and findings == []


def test_static_mut_and_bare_static_are_both_handled():
    rust = "static mut tbl: [u8; 99] = [0x40,0x49,0x68,0x94,0x90,0x93];"
    out, findings = repair_tables(rust, C)
    assert "[u8; 6]" in out and findings


def test_table_with_no_c_counterpart_keeps_its_values():
    """We can correct a length we can count; we must not invent data we cannot source."""
    rust = "pub static mystery: [u8; 9] = [1,2,3];"
    out, findings = repair_tables(rust, C)
    assert "[u8; 3]" in out
    assert "1,2,3" in out
    assert "VALUES NOT VERIFIED" in str(findings[0])


def test_consistent_unsourceable_table_is_untouched():
    rust = "pub static mystery: [u8; 3] = [1,2,3];"
    out, findings = repair_tables(rust, C)
    assert out == rust and findings == []


def test_multiple_tables_are_each_handled():
    c = C + "\nstatic const unsigned char two[] = { 7, 8 };\n"
    rust = "pub static tbl: [u8; 2] = [0x40,0x49];\npub static two: [u8; 5] = [7,8];"
    out, findings = repair_tables(rust, c)
    assert "[u8; 6]" in out and "[u8; 2] = [\n    7,8,\n]" in out
    assert len(findings) == 2


# --------------------------------------------------------------------------- #
# shapes that are NOT initialiser lists — both mis-"repaired" on the first run
# --------------------------------------------------------------------------- #


def test_repeat_expression_is_not_an_initialiser_list():
    """`[0; 257]` has one comma-free body. Counting commas would shrink it to 1 byte."""
    rust = "pub static mut MSVCRT_mbctype: [u8; 257] = [0; 257];"
    out, findings = repair_tables(rust, C)
    assert out == rust and findings == []


def test_array_of_structs_is_left_alone():
    """7 structs, 42 commas. Neither the count nor the data is linear."""
    rust = (
        "pub static g_cpextrainfo: [cp_extra_info_t; 7] = [\n"
        "    cp_extra_info_t { cp: 932, TrailBytes: [0x40, 0x7e, 0] },\n"
        "    cp_extra_info_t { cp: 0, TrailBytes: [1, 255, 0] },\n"
        "];"
    )
    out, findings = repair_tables(rust, C)
    assert out == rust and findings == []


def test_non_literal_elements_are_left_alone():
    """A table of named constants is not data we can prove against the C."""
    rust = "pub static tbl: [u8; 2] = [SOME_CONST, OTHER_CONST];"
    out, findings = repair_tables(rust, C)
    assert out == rust and findings == []
