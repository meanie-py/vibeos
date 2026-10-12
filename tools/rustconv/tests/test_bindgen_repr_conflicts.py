"""bindgen_repr_conflicts: the packed+aligned types rustc rejects (E0587/E0588)."""

import importlib.util
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "bindgen_repr_conflicts", Path(__file__).resolve().parents[1] / "bindgen_repr_conflicts.py"
)
brc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(brc)

# dbghelp.h's shape on amd64: a pack(4) struct holding CONTEXT (align 16), and a pack(4)
# struct holding it through a typedef and a (raw, pre-REAL_UNIONS) union wrapper.
ISLAND = """
#[repr(C)]
#[repr(align(16))]
pub struct _CONTEXT {
    pub P1Home: DWORD64,
}
pub type CONTEXT = _CONTEXT;
#[repr(C, packed(4))]
pub struct _MINIDUMP_THREAD_CALLBACK {
    pub ThreadId: ULONG,
    pub Context: CONTEXT,
}
pub type MINIDUMP_THREAD_CALLBACK = _MINIDUMP_THREAD_CALLBACK;
#[repr(C)]
pub struct _MINIDUMP_CALLBACK_INPUT__bindgen_ty_1 {
    pub Thread: __BindgenUnionField<MINIDUMP_THREAD_CALLBACK>,
}
#[repr(C, packed(4))]
pub struct _MINIDUMP_CALLBACK_INPUT {
    pub ProcessId: ULONG,
    pub __bindgen_anon_1: _MINIDUMP_CALLBACK_INPUT__bindgen_ty_1,
}
#[repr(C, packed(4))]
pub struct _HARMLESS {
    pub p: *mut CONTEXT,
    pub a: [ULONG; 4],
}
"""


def test_packed_types_holding_an_aligned_type_by_value_are_found_transitively():
    assert brc.conflicts(ISLAND) == ["_MINIDUMP_CALLBACK_INPUT", "_MINIDUMP_THREAD_CALLBACK"]


def test_a_packed_struct_that_only_points_at_an_aligned_type_is_fine():
    assert "_HARMLESS" not in brc.conflicts(ISLAND)


def test_packed_and_aligned_on_one_struct_is_a_conflict():
    text = "#[repr(C, packed(4))]\n#[repr(align(4))]\npub struct X {\n    pub a: u8,\n}\n"
    assert brc.conflicts(text) == ["X"]
