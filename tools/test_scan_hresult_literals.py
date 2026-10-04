"""scan_hresult_literals: a bare HRESULT the C function never returns is reported."""

import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.scan_hresult_literals import hresult_names, scan_files  # noqa: E402

C_SRC = """
static HRESULT WINAPI Stream_Read(IStream *iface, void *pv,
                                  ULONG cb /* [in] */, ULONG *read)
{
    if (!pv) return STG_E_INVALIDPOINTER;
    if (!cb) return STG_E_READFAULT;
    return S_OK;
}

static NTSTATUS Driver_Query(PVOID buffer)
{
    return STATUS_BUFFER_OVERFLOW;
}
"""

RUST_SRC = """
unsafe extern "system" fn Stream_Read(iface: *mut IStream, pv: *mut c_void, cb: u32) -> HRESULT {
    if pv.is_null() {
        return 0x80030009u32 as HRESULT;
    }
    if cb == 0 {
        return 0x80030004u32 as HRESULT;
    }
    const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
    let _ = 0x80030004u32 as HRESULT; // nolint: hresult-literal (deliberate)
    0
}

extern "system" {
    fn Stream_Read(iface: *mut IStream) -> HRESULT;
}

unsafe fn Driver_Query(buffer: *mut c_void) -> i32 {
    0x80000005u32 as i32
}
"""


def test_reports_only_the_wrong_literal(tmp_path):
    cdir = tmp_path / "c"
    cdir.mkdir()
    (cdir / "stream.c").write_text(C_SRC)
    rs = tmp_path / "stream.rs"
    rs.write_text(RUST_SRC)

    findings = scan_files([rs], [cdir], hresult_names())

    # 0x80030009 is STG_E_INVALIDPOINTER, which the C returns: clean. 0x80030004 is
    # STG_E_TOOMANYOPENFILES where the C says STG_E_READFAULT: the storage32 bug.
    # The `const`, the nolint line, the extern declaration and the NTSTATUS function
    # (no HRESULT names in its C body) are all out of scope.
    assert [(f[1], f[2]) for f in findings] == [(7, "Stream_Read")]
    assert "STG_E_TOOMANYOPENFILES" in findings[0][3]
    assert "STG_E_READFAULT" in findings[0][4]


def test_cli_lint_is_clean_on_the_tree():
    r = subprocess.run(
        [
            sys.executable,
            str(REPO / "tools" / "scan_hresult_literals.py"),
            "--lint",
            "--crate",
            "ole32-rust",
        ],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, r.stdout + r.stderr
