from rustconv import port
from rustconv.profile_derive import unique_unit_stems


def test_shared_stem_qualified_by_directory():
    # crypt32: store.c and reactos/store.c both became store.rs
    units = ["dll/win32/crypt32/store.c", "dll/win32/crypt32/reactos/store.c", "dll/win32/crypt32/cert.c"]
    assert unique_unit_stems(units) == {
        "dll/win32/crypt32/store.c": "store",
        "dll/win32/crypt32/reactos/store.c": "reactos_store",
        "dll/win32/crypt32/cert.c": "cert",
    }


def test_port_output_paths_do_not_collide():
    state = {"stem": "crypt32", "tus": ["d/store.c", "d/reactos/store.c"]}
    assert port._unit_mod(state, "d/store.c") == "store"
    assert port._unit_mod(state, "d/reactos/store.c") == "reactos_store"
    assert port._unit_mod({"stem": "x", "tus": ["d/a.c"]}, "d/a.c") == "a"


def test_c_offsetof_call_as_offset_of(tmp_path, monkeypatch):
    # crypt32 decode.c's item tables kept `offsetof(..)`, the type as `size_of::<T>()`
    src = tmp_path / "decode.rs"
    src.write_text(
        "x = offsetof(core::mem::size_of::<CRL_INFO>(), SignatureAlgorithm . pszObjId);\n"
        "y = offsetof(FOO, a);\nz = core::mem::offset_of!(BAR, b);\n"
    )
    monkeypatch.setattr(port, "_stem_sources", lambda state: [src])
    assert port._c_offsetof_calls({}) == 2
    assert src.read_text() == (
        "x = core::mem::offset_of!(CRL_INFO, SignatureAlgorithm.pszObjId);\n"
        "y = core::mem::offset_of!(FOO, a);\nz = core::mem::offset_of!(BAR, b);\n"
    )


def test_curried_named_channel(tmp_path, monkeypatch):
    # crypt32 chain.c: `TRACE_(chain)("..", ..)` kept in the C's curried spelling
    src = tmp_path / "chain.rs"
    src.write_text('TRACE_(chain)(\n    "x %d\\n", a);\nWARN_(chain, "y");\n')
    monkeypatch.setattr(port, "_stem_sources", lambda state: [src])
    assert port._named_channel_debug({}) == 2
    assert src.read_text() == (
        'WINE_TRACE_!(__wine_dbch_chain,\n    "x %d\\n", a);\n'
        'WINE_WARN_!(__wine_dbch_chain, "y");\n'
    )
