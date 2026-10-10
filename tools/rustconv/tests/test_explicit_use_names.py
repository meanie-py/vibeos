"""`port check`'s catch-all detector: a name the crate imports BY NAME is in scope.

ndis's prelude imports wdm.h's IRP_MN_* through `pub use ntkrnl_sys::driver_ddk::{..}` (the
`ddk` glob does not carry them), and every PnP switch arm read as a catch-all binding. A glob
must still NOT count: a constant a glob does not reach is exactly the catch-all (§9.112).
"""

from rustconv.prelude_resolve import _explicit_use_names


def test_braced_list_and_single_imports_count():
    text = """
pub use ntkrnl_sys::driver_ddk::{
    IRP_MN_START_DEVICE, IRP_MN_REMOVE_DEVICE,
    IRP_MN_STOP_DEVICE as STOP,
};
use ntkrnl_sys::ddk::STATUS_PENDING;
use ntkrnl_sys::ddk::Foo as BAR_CONST;
"""
    names = _explicit_use_names(text)
    assert {"IRP_MN_START_DEVICE", "IRP_MN_REMOVE_DEVICE", "STOP", "STATUS_PENDING"} <= names
    assert "BAR_CONST" in names
    assert "IRP_MN_STOP_DEVICE" not in names  # renamed: only the alias is in scope


def test_a_glob_is_not_a_by_name_import():
    assert _explicit_use_names("pub use ntkrnl_sys::ddk::*;") == set()
