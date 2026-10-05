"""A translated C enum becomes bindgen's shape: an integer alias plus one const each."""

from rustconv.crate_integrate import lower_c_enums

SRC = """#[repr(C)]
#[derive(Clone, Copy)]
pub enum _ADAPTER_TYPE {
    EisaAdapter = 0,
    ScsiAdapter, // implicit
    AdapterTypeMax = 5,
}
pub type ADAPTER_TYPE = _ADAPTER_TYPE;
fn f(t: ADAPTER_TYPE) -> bool { t == ADAPTER_TYPE::ScsiAdapter || t as u32 >= AdapterTypeMax as u32 }
pub enum Data { A(u32), B }
"""


def test_fieldless_enum_is_lowered_and_paths_rewritten():
    out, lowered = lower_c_enums(SRC)
    assert lowered == ["_ADAPTER_TYPE"]
    assert "pub type _ADAPTER_TYPE = ::core::ffi::c_uint;" in out
    assert "pub const EisaAdapter: _ADAPTER_TYPE = 0;" in out
    assert "pub const ScsiAdapter: _ADAPTER_TYPE = EisaAdapter + 1;" in out
    assert "pub const AdapterTypeMax: _ADAPTER_TYPE = 5;" in out
    assert "#[repr(C)]" not in out.split("pub enum Data")[0]
    assert "t == ScsiAdapter" in out
    assert "pub enum Data { A(u32), B }" in out  # a real Rust enum is left alone
    assert lower_c_enums(out) == (out, [])


def test_a_negative_enumerator_makes_it_signed():
    out, _ = lower_c_enums("pub enum E {\n    Neg = -1,\n    Zero,\n}\n")
    assert "pub type E = ::core::ffi::c_int;" in out
