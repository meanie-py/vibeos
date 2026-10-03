"""scan_extern_static_width: a Rust extern static read wider than its C definition."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import scan_extern_static_width as sesw

SHIM = (
    "#define FF_SYM(s) s\n"
    'const ULONG __c_MAX_LFN_CHARACTERS __asm__(FF_SYM("MAX_LFN_CHARACTERS")) = 260;\n'
    "const LONGLONG __c_FCB_LOOKUP_AS_HINT "
    '__asm__(FF_SYM("FCB_LOOKUP_ALLOCATIONSIZE_HINT")) = -1;\n'
    'const ULONG __c_FAT_NTC_DCB __asm__(FF_SYM("FAT_NTC_DCB")) = 0x503;\n'
    'static PVOID __c_Slot __asm__("_Slot") = 0;\n'
)


def test_c_definitions_read_the_asm_name_and_width():
    c = sesw.c_definitions(SHIM)
    assert c == {
        "MAX_LFN_CHARACTERS": 4,
        "FCB_LOOKUP_ALLOCATIONSIZE_HINT": 8,
        "FAT_NTC_DCB": 4,
        "Slot": 8,  # i386 leading underscore stripped
    }


def test_fastfat_max_lfn_overread_fires():
    # fastfat's FatSetRenameInfo: an 8-byte read of a 4-byte ULONG bugchecked every arm64 rename.
    src = 'extern "system" {\n    static MAX_LFN_CHARACTERS: usize;\n}\n'
    assert sesw.scan_rust(src, sesw.c_definitions(SHIM)) == [(2, "MAX_LFN_CHARACTERS", 8, 4)]


def test_equal_and_narrower_reads_pass():
    src = (
        "    static FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64;\n"
        "    static FAT_NTC_DCB: u16;\n"  # narrower: the low bytes on little-endian
        "    pub static mut Slot: *mut c_void;\n"
    )
    assert sesw.scan_rust(src, sesw.c_definitions(SHIM)) == []


def test_unknown_types_and_definitions_are_skipped_not_guessed():
    src = (
        "    static MAX_LFN_CHARACTERS: SomeAlias;\n"
        "    static NOT_IN_C: usize;\n"
        "    // static MAX_LFN_CHARACTERS: usize;\n"
        "static OWN: usize = 4;\n"
    )
    assert sesw.scan_rust(src, sesw.c_definitions(SHIM)) == []


def test_real_tree_is_clean():
    rc = subprocess.run([sys.executable, str(Path(sesw.__file__))], capture_output=True, text=True)
    assert rc.returncode == 0, rc.stdout
