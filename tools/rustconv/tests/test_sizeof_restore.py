"""sizeof_restore: put back a dropped sizeof, never inside a macro's type slot."""

from rustconv.sizeof_restore import restore

C = """
DWORD f(void)
{
    DWORD size = sizeof(METARECORD);
    size = FIELD_OFFSET(METARECORD, rdParm[2]);
    return size;
}
"""

RUST = """pub unsafe extern "C" fn f() -> DWORD {
    let mut size: DWORD = METARECORD as DWORD;
    size = FIELD_OFFSET!(METARECORD, rdParm[2]) as DWORD;
    return size;
}
"""


def test_a_bare_type_in_value_position_gets_its_sizeof_back(tmp_path):
    c = tmp_path / "metadc.c"
    c.write_text(C)
    out, fixed = restore(RUST, c, {"METARECORD"})
    assert "core::mem::size_of::<METARECORD>() as DWORD;" in out
    # winegdi's metadc.c: the macro's TYPE argument must stay a type.
    assert "FIELD_OFFSET!(METARECORD, rdParm[2])" in out
    assert fixed == ["f: sizeof(METARECORD) x1"]
