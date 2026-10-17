"""Unit tests for the extern-declaration drift scanner's SUBSTRATE oracle.

The definition-based half of this scanner is exercised by the workspace sweep
itself. What is pinned here is the bindgen-island oracle, because its two
interesting properties are both things the sweep cannot demonstrate: that it
finds a re-declared import nothing else in the pipeline can check, and that it
declines to judge a crate by an island that crate does not link.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.scan_extern_decl_drift import scan


def _tree(root: Path, *, dep: bool, decl: str, call: str, sub: str = "") -> list[Path]:
    """A `foo-rust` crate and a `bar-sys` island, optionally wired together."""
    sysdir = root / "bar-sys" / "src"
    sysdir.mkdir(parents=True)
    (root / "bar-sys" / "Cargo.toml").write_text('[package]\nname = "bar-sys"\n')
    (sysdir / "bindgen_i386.rs").write_text(
        "pub type SIZE_T = usize;\n"
        "pub type ULONG = u32;\n"
        'extern "C" {\n'
        "    pub fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: SIZE_T) -> *mut u8;\n"
        "}\n"
    )
    crate = root / "foo-rust"
    (crate / "src" / sub).mkdir(parents=True)
    (crate / "Cargo.toml").write_text(
        '[package]\nname = "foo-rust"\n\n[dependencies]\n'
        + ('bar-sys = { path = "../bar-sys" }\n' if dep else "")
    )
    (crate / "src" / sub / "lib.rs").write_text(
        "pub type SIZE_T = usize;\npub type ULONG = u32;\n"
        f'extern "C" {{\n    {decl}\n}}\n'
        f"pub unsafe fn go() {{ {call} }}\n"
    )
    return [crate / "src"]


_BAD = "fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: u32) -> *mut u8;"
_GOOD = "fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: SIZE_T) -> *mut u8;"
_CALL = "ExAllocatePoolWithTag(0, 8);"


def test_width_drift_against_the_substrate_is_reported(tmp_path):
    # The scsiport bug: a re-declared ntoskrnl import narrowing SIZE_T to u32.
    # rustc compares the CALL against this declaration and never the declaration
    # against the header, so nothing else in the pipeline can see it.
    crates = _tree(tmp_path, dep=True, decl=_BAD, call=_CALL)
    findings = scan(tmp_path, crates)
    assert len(findings) == 1, findings
    assert "ExAllocatePoolWithTag" in findings[0][2]
    assert "WIDTH" in findings[0][2]


def test_a_matching_declaration_is_silent(tmp_path):
    crates = _tree(tmp_path, dep=True, decl=_GOOD, call=_CALL)
    assert scan(tmp_path, crates) == []


def test_an_island_the_crate_does_not_link_is_not_an_oracle(tmp_path):
    """The RegEnumValueW lesson, and the reason the oracle is dependency-scoped.

    Kernel mode and user mode each have a function of that name and they are
    NOT the same function -- win32k's takes seven parameters, the Win32 one
    eight. A global by-name oracle reported win32k's CORRECT declaration as an
    arity bug. A crate can only call what it links.
    """
    crates = _tree(tmp_path, dep=False, decl=_BAD, call=_CALL)
    assert scan(tmp_path, crates) == []


def test_an_uncalled_declaration_is_not_reported(tmp_path):
    # Only a decl that GOVERNS a call governs an ABI; an address-taken or dead
    # declaration deliberately differs (SSDT slot counts, kernel-invoked thunks).
    crates = _tree(tmp_path, dep=True, decl=_BAD, call="")
    assert scan(tmp_path, crates) == []


def test_pointer_width_is_known_inside_an_arch_gated_module(tmp_path):
    """`ULONG_PTR` and `ULONG` are the same four bytes under `ke/i386/`.

    Reporting them as drift there was three findings against ntoskrnl's i386
    trap handlers, every one of them correct code.
    """
    crates = _tree(tmp_path, dep=True, decl=_BAD, call=_CALL, sub="i386")
    assert scan(tmp_path, crates) == []


def _static_tree(root: Path, c: str, rust: str) -> list[Path]:
    """A crate whose cohab C hands it data by asm label, as fastfat's does."""
    (root / "drv").mkdir()
    (root / "drv" / "rust_cohab.c").write_text(c)
    crate = root / "foo-rust"
    (crate / "src").mkdir(parents=True)
    (crate / "Cargo.toml").write_text('[package]\nname = "foo-rust"\n')
    (crate / "src" / "lib.rs").write_text("pub type ULONG = u32;\npub type BOOLEAN = u8;\n" + rust)
    return [crate / "src"]


def test_a_static_wider_than_its_rust_definition_is_reported(tmp_path):
    crates = _static_tree(
        tmp_path,
        "",
        '#[no_mangle]\npub static mut Flag: BOOLEAN = 0;\nextern "C" { static Flag: ULONG; }\n',
    )
    findings = scan(tmp_path, crates)
    assert len(findings) == 1, findings
    assert "Flag: static WIDTH decl `ULONG` (4)" in findings[0][2]


def test_per_arch_definitions_that_disagree_are_not_an_oracle(tmp_path):
    crates = _static_tree(
        tmp_path,
        "",
        "#[no_mangle]\npub static mut Probe: ULONG = 0;\n"
        "#[no_mangle]\npub static mut Probe: usize = 0;\n"
        'extern "C" { static Probe: usize; }\n',
    )
    assert scan(tmp_path, crates) == []


_C_UCHAR_FN = "UCHAR FatIndexBitSize(PBIOS_PARAMETER_BLOCK B){ return __c_FatIndexBitSize(B); }\n"


def test_a_return_wider_than_the_c_definition_is_reported(tmp_path):
    # The 14-cluster FAT16 volume: a UCHAR return read as u32 took 24 bits nobody
    # wrote, and they became part of a divisor.
    crates = _static_tree(
        tmp_path,
        _C_UCHAR_FN,
        'extern "C" { fn FatIndexBitSize(b: *const u8) -> u32; }\n'
        "pub unsafe fn go() { FatIndexBitSize(core::ptr::null()); }\n",
    )
    findings = scan(tmp_path, crates)
    assert len(findings) == 1, findings
    assert "FatIndexBitSize: RETURN WIDTH" in findings[0][2]


def test_a_return_declared_at_the_c_width_is_silent(tmp_path):
    crates = _static_tree(
        tmp_path,
        _C_UCHAR_FN,
        'extern "C" { fn FatIndexBitSize(b: *const u8) -> u8; }\n'
        "pub unsafe fn go() { FatIndexBitSize(core::ptr::null()); }\n",
    )
    assert scan(tmp_path, crates) == []


def test_a_return_wider_than_the_rust_definition_is_reported(tmp_path):
    # rustc does not extend a narrow return either: `x as u8` comes back unmasked.
    crates = _static_tree(
        tmp_path,
        "",
        '#[no_mangle]\npub extern "C" fn Hit() -> u8 { 1 }\n'
        'extern "C" { fn Hit() -> ULONG; }\n'
        "pub unsafe fn go() { Hit(); }\n",
    )
    findings = scan(tmp_path, crates)
    assert len(findings) == 1, findings
    assert "Hit: RETURN WIDTH decl `-> ULONG` (4) wider than def `-> u8`" in findings[0][2]


def _crt_tree(root: Path, *, kernel: bool, ret: str) -> list[Path]:
    """ucrt-rust defines towupper; a user (or kernel) crate declares and calls it."""
    crt = root / "ucrt-rust"
    (crt / "src").mkdir(parents=True)
    (crt / "Cargo.toml").write_text('[package]\nname = "ucrt-rust"\n')
    (crt / "src" / "lib.rs").write_text(
        '#[no_mangle]\npub unsafe extern "C" fn towupper(c: u16) -> u16 { c }\n'
    )
    user = root / "find-rust"
    (user / "src").mkdir(parents=True)
    (user / "Cargo.toml").write_text(
        '[package]\nname = "find-rust"\n\n[dependencies]\n'
        + ('ntkrnl-sys = { path = "../ntkrnl-sys" }\n' if kernel else "")
    )
    (user / "src" / "lib.rs").write_text(
        "pub type c_int = i32;\n"
        f'extern "C" {{ fn towupper(c: c_int) -> {ret}; }}\n'
        "pub unsafe fn go() { towupper(97); }\n"
    )
    return [crt / "src", user / "src"]


def test_a_crt_import_declared_wider_than_the_crt_definition_is_reported(tmp_path):
    # 23 crates declared towupper `-> c_int` over ucrt-rust's `-> u16` (a wint_t),
    # and compared two such returns: the in-crate pairing has no definition to read.
    findings = scan(tmp_path, _crt_tree(tmp_path, kernel=False, ret="c_int"))
    assert len(findings) == 1, findings
    assert (
        "towupper: RETURN WIDTH decl `-> c_int` (4) wider than CRT def `-> u16`" in (findings[0][2])
    )


def test_the_crt_authority_judges_neither_kernel_crates_nor_correct_declarations(tmp_path):
    assert scan(tmp_path / "k", _crt_tree(tmp_path / "k", kernel=True, ret="c_int")) == []
    assert scan(tmp_path / "u", _crt_tree(tmp_path / "u", kernel=False, ret="u16")) == []
