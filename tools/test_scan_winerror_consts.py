"""scan_winerror_consts: a const named after a winerror.h code, with another value."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_winerror_consts import header_values, scan_text

HEADER = """
#define TYPE_E_AMBIGUOUSNAME   _HRESULT_TYPEDEF_(0x8002802C)
#define ERROR_FILE_NOT_FOUND   2L
#define SEC_E_NO_CONTEXT       _HRESULT_TYPEDEF_(0x80090361L)
"""

SRC = """
pub const TYPE_E_AMBIGUOUSNAME: HRESULT = 0x8002_8CA6u32 as HRESULT;
pub const ERROR_FILE_NOT_FOUND: DWORD = 2;
pub const SEC_E_NO_CONTEXT: SECURITY_STATUS = 0x8009_0361u32 as i32;
const ERROR_FILE_NOT_FOUND: u32 = 3; // nolint: winerror-const (test)
// const TYPE_E_AMBIGUOUSNAME: HRESULT = 1;
const MY_OWN_CODE: u32 = 7;
const SEC_E_NO_CONTEXT: i32 = -2146893215;
"""


def test_only_wrong_values_of_header_names():
    codes = header_values(HEADER)
    assert codes["ERROR_FILE_NOT_FOUND"] == {2}
    # line 8 is negative decimal: the regex never matches a sign, so it is not judged
    assert [(n, name) for n, name, _ in scan_text(SRC, codes)] == [(2, "TYPE_E_AMBIGUOUSNAME")]


def test_tree_is_clean():
    import subprocess

    r = subprocess.run(
        [
            sys.executable,
            str(Path(__file__).resolve().parent / "scan_winerror_consts.py"),
            "--lint",
        ],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, r.stdout
