"""scan_ptr_typedef_indirection: the usetup FileCopyCallback shape is caught, PFOO* is not."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_ptr_typedef_indirection import c_definitions, findings_for  # noqa: E402

C = """
UINT CALLBACK
FileCopyCallback(PVOID Context, UINT Notification, UINT_PTR Param1, UINT_PTR Param2)
{
    PFILEPATHS_W FilePathInfo;
    PCM_INT13_DRIVE_PARAMETER *Int13Drives;
    FilePathInfo = (PFILEPATHS_W)Param1;
    return 0;
}
"""


def _rust(decl: str) -> str:
    return (
        "use crate::prelude::*;\n\n"
        "pub unsafe extern \"system\" fn FileCopyCallback(Context: PVOID, Notification: UINT,\n"
        "        Param1: UINT_PTR, Param2: UINT_PTR) -> UINT {\n"
        f"    {decl}\n"
        "    let mut Int13Drives: *mut PCM_INT13_DRIVE_PARAMETER = core::ptr::null_mut();\n"
        "    0\n}\n"
    )


def _run(rust: str):
    c_body = c_definitions(C)["FileCopyCallback"]
    at = rust.index("pub unsafe")
    return findings_for(c_body, rust, rust[at:], at)


def test_one_level_too_deep_is_reported_at_its_line():
    hits = _run(_rust("let mut FilePathInfo: *mut PFILEPATHS_W = Param1 as *mut PFILEPATHS_W;"))
    assert [line for line, _ in hits] == [5]
    assert "FilePathInfo" in hits[0][1]


def test_a_genuine_pointer_to_pointer_typedef_is_not():
    assert _run(_rust("let mut FilePathInfo: PFILEPATHS_W = Param1 as PFILEPATHS_W;")) == []


def test_nolint_on_the_declaration_suppresses_it():
    rust = _rust(
        "let mut FilePathInfo: *mut PFILEPATHS_W = core::ptr::null_mut(); "
        "// nolint: ptr-typedef-indirection (reviewed)"
    )
    assert _run(rust) == []
