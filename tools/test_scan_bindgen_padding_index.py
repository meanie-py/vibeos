"""scan_bindgen_padding_index: indexing a bindgen padding array to reach a hidden member."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import scan_bindgen_padding_index as sbp


def test_index_read_fires():
    # The user32 wnd_pti that dropped every posted message once [u64; 3] became [u8; 24].
    src = (
        "unsafe fn wnd_pti(p: PWND) -> PTHREADINFO {\n"
        "    (*p).head.__bindgen_padding_0[2] as PTHREADINFO\n"
        "}\n"
    )
    assert sbp.scan_text(src) == [2]


def test_address_of_padding_is_layout_stable():
    src = (
        "let p = core::ptr::addr_of_mut!((*psi).__bindgen_padding_0) as *mut X;\n"
        "let o = core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0);\n"
    )
    assert sbp.scan_text(src) == []


def test_comment_mentioning_an_index_is_skipped():
    assert sbp.scan_text("// was head.__bindgen_padding_0[2] before the fix\n") == []


def test_generated_islands_are_excluded():
    assert sbp.GENERATED_RE.search("sdk/lib/rust/win32k-sys/src/bindgen_amd64.rs")
    assert sbp.GENERATED_RE.search("sdk/lib/rust/ntkrnl-sys/src/hal_bindgen_i386.rs")
    assert not sbp.GENERATED_RE.search("sdk/lib/rust/user32-rust/src/windows/message.rs")


def test_tree_is_clean():
    assert sbp.main() == 0
