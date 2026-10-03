"""Scanner tests for verify_signatures: the Rust side must count PARAMETERS, not
the words of a comment that sits inside the parameter list."""

from pathlib import Path

from tools.verify_signatures import scan_rust_crate


def _scan(tmp_path: Path, body: str) -> dict:
    (tmp_path / "lib.rs").write_text(body)
    (fn,) = scan_rust_crate(tmp_path)
    return fn


def test_line_comment_inside_params_is_not_a_param(tmp_path):
    fn = _scan(
        tmp_path,
        """#[no_mangle]
pub unsafe extern "system" fn IopCloseFile(
    Process: PEPROCESS,
    // ddk::ULONG_PTR, the alias (c_ulonglong on 64-bit), not usize: same size,
    // distinct type -- commas here once became three phantom params.
    HandleCount: ntkrnl_sys::ddk::ULONG_PTR,
) {}
""",
    )
    assert fn["params"] == ["PEPROCESS", "ntkrnl_sys::ddk::ULONG_PTR"]


def test_unbalanced_paren_in_comment_does_not_end_the_list(tmp_path):
    fn = _scan(
        tmp_path,
        """#[no_mangle]
pub extern "system" fn Foo(
    a: ULONG, /* note: see ( below */
    b: PVOID, // trailing ) paren
) -> NTSTATUS { 0 }
""",
    )
    assert fn["params"] == ["ULONG", "PVOID"]
    assert fn["ret"] == "NTSTATUS"


def test_plain_signature_unchanged(tmp_path):
    fn = _scan(
        tmp_path,
        '#[no_mangle]\npub extern "C" fn Bar(x: *mut [u8; 4], f: Option<fn(u32) -> u32>) {}\n',
    )
    assert fn["params"] == ["*mut [u8; 4]", "Option<fn(u32) -> u32>"]
