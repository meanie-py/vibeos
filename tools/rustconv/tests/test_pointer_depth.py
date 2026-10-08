from rustconv import pointer_depth as pd

ISLAND = "pub type PFOO = *mut _FOO;\npub type PBAR = *mut _BAR;\n"
TYPES = pd.pointer_typedefs([ISLAND, ISLAND])
TARGETS = pd.pointer_targets([ISLAND, ISLAND])

C = """\
VOID
Get(PFOO Foo, PBAR *Out)
{
    PFOO Local;
    Local = Foo;
    Alloc(sizeof(*Foo));
}
"""

RUST = """\
unsafe extern "C" fn Get(mut Foo: *mut PFOO, mut Out: *mut PBAR) {
    let mut Local: *mut PFOO = Foo as *mut PFOO;
    (*(*Foo)).Field = 1;
    Alloc(core::mem::size_of::<PFOO>());
}
"""


def _decls(c=C):
    return {"Get": pd.c_declarations("Get", [("PFOO", "Foo"), ("PBAR *", "Out")], c, TYPES)}


def test_a_pointer_typedef_read_as_its_struct_loses_one_level():
    fixed, fixes = pd.repair(RUST, _decls(), TYPES, TARGETS)
    assert "mut Foo: PFOO" in fixed and "(*Foo).Field" in fixed
    assert "Local: PFOO = Foo as PFOO;" in fixed
    assert "mut Out: *mut PBAR" in fixed  # the C's out-parameter keeps its star
    assert fixes and "Foo" in fixes[0]


def test_size_of_a_pointer_typedef_is_the_pointee():
    # atapi: C `sizeof(*DevExt)` -> size_of::<PATAPORT_DEVICE_EXTENSION>() (8 bytes) as the
    # PDO extension size handed to IoCreateDevice.
    fixed, fixes = pd.repair(RUST, _decls(), TYPES, TARGETS)
    assert "size_of::<_FOO>()" in fixed and "size_of::<PFOO>" not in fixed
    assert any("size_of::<PFOO>" in f for f in fixes)
    # ...unless the C itself measures the pointer.
    meant = C.replace("sizeof(*Foo)", "sizeof(PFOO)")
    fixed, _ = pd.repair(RUST, _decls(meant), TYPES, TARGETS)
    assert "size_of::<PFOO>()" in fixed


def test_size_of_a_raw_pointer_is_the_pointee_unless_the_c_measures_a_pointer():
    # acpi.c `sizeof(*TimingMode)` -> size_of::<*mut _IDE_ACPI_TIMING_MODE_BLOCK>() sized an
    # ACPI output buffer 8 bytes; atapi's smart.c length checks had the same misreading.
    c = "VOID\nGet(PFOO Foo)\n{\n    Alloc(sizeof(*Foo));\n}\n"
    rust = (
        'unsafe extern "C" fn Get(Foo: PFOO) {\n    Alloc(core::mem::size_of::<*mut _FOO>());\n}\n'
    )
    decls = {"Get": pd.c_declarations("Get", [("PFOO", "Foo")], c, TYPES)}
    fixed, fixes = pd.repair(rust, decls, TYPES, TARGETS)
    assert "size_of::<_FOO>()" in fixed and any("*mut _FOO" in f for f in fixes)
    # A spelled-out pointer sizeof, or sizeof of a pointer VARIABLE, means a pointer size.
    for meant in ("Alloc(sizeof(_FOO *));", "Alloc(sizeof(Foo));"):
        c2 = c.replace("Alloc(sizeof(*Foo));", meant)
        decls = {"Get": pd.c_declarations("Get", [("PFOO", "Foo")], c2, TYPES)}
        assert pd.repair(rust, decls, TYPES, TARGETS)[0] == rust
