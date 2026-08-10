"""Tests for scan_import_cc — the cross-module import calling-convention lint.

Both halves matter, but the must-NOT-fire half is what this lint lives or dies on. Its first
whole-tree run reported 22 sites and SIX of them were the scanner's own blind spots, every one
of which reads exactly like a real bug when you skim it:

  * a name exported by two unrelated DLLs with different conventions (user32's stdcall
    UserRealizePalette vs win32k's FASTCALL one),
  * a target whose name merely shares a stem with the crate (rosapps' `ps` utility vs ps-rust,
    which is ntoskrnl's Ps subsystem),
  * a `#[cfg(target_arch = ...)]` BLOCK EXPRESSION picking the convention per arch, which is
    the CORRECT idiom and read as an i386 declaration,
  * a `#[no_mangle]` definition without `pub`, which is still a global symbol to the linker,
  * and a symbol the consuming module defines out of line in its own C.

Baseline a batch like that and the real findings ride along inside it. So each of those is
pinned below by the shape that fooled it, not just by "no findings".
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_import_cc import c_defines, crate_link_set, read_spec, scan_crate

KERNEL32 = """\
@ stdcall SetErrorMode(long)
@ stdcall -norelay GetProcessHeap()
@ cdecl -arch=i386 memcpy(ptr ptr long)
@ stdcall -arch=arm64 AcquireSRWLockExclusive(ptr) ntdll_vista.RtlAcquireSRWLockExclusive
@ cdecl -arch=!i386 _chgsignf(float)
@ stdcall -arch=win32,arm64 BothOfThose(long)
@ cdecl -arch=win64 SixtyFourOnly(long)
@ stdcall -private BasepPrivateThing(long)
@ stub BaseGenerateAppCompatData
@ extern SomeDataExport
;@ stdcall CommentedOut(long)
"""

NTOSKRNL = "@ fastcall -arch=i386 ExAcquireFastMutexUnsafe(ptr)\n"


def _spec(tmp_path: Path, name: str, text: str) -> Path:
    p = tmp_path / f"{name}.spec"
    p.write_text(text)
    return p


def _crate(tmp_path: Path, src: str) -> str:
    d = tmp_path / "crate" / "src"
    d.mkdir(parents=True, exist_ok=True)
    (d / "lib.rs").write_text(src)
    return str(d)


def _scan(tmp_path: Path, src: str, oracle_text=KERNEL32, module_dirs=None) -> list[str]:
    spec = _spec(tmp_path, "kernel32", oracle_text)
    return scan_crate(_crate(tmp_path, src), {"kernel32": read_spec(str(spec))}, module_dirs or [])


# --- the .spec parser -------------------------------------------------------------------


def test_flags_follow_the_convention_token(tmp_path):
    """spec2def reads the convention FIRST, then options. Parsing it the other way round
    does not fail loudly — every flagged export silently leaves the oracle, which is most of
    kernel32."""
    exports = read_spec(str(_spec(tmp_path, "kernel32", KERNEL32)))
    assert exports["GetProcessHeap"] == {"stdcall"}  # -norelay AFTER the convention
    assert exports["SetErrorMode"] == {"stdcall"}
    assert exports["memcpy"] == {"C"}  # cdecl and varargs both mean C


def test_spec_entries_that_are_not_oracles(tmp_path):
    exports = read_spec(str(_spec(tmp_path, "kernel32", KERNEL32)))
    assert "AcquireSRWLockExclusive" not in exports  # -arch=arm64: not in the i386 import lib
    # A leading `!` NEGATES. msvcrt.spec alone has ~60 of these, and a substring test admits
    # every one — i.e. it imports exactly the exports that do not exist on i386.
    assert "_chgsignf" not in exports
    assert "SixtyFourOnly" not in exports  # win64 is not spec2def's alias for win32
    assert exports["BothOfThose"] == {"stdcall"}  # a list including win32 IS i386
    assert "BasepPrivateThing" not in exports  # -private: not in the import lib at all
    assert "BaseGenerateAppCompatData" not in exports  # @ stub carries no convention
    assert "SomeDataExport" not in exports  # @ extern is data
    assert "CommentedOut" not in exports  # `;` comment


# --- MUST CATCH -------------------------------------------------------------------------


def test_winapi_declared_cdecl(tmp_path):
    """The wave-18c bounce, reduced: WINAPI is stdcall on i386, so the import lib has
    `_SetErrorMode@4` and an `extern "C"` decl asks for `_SetErrorMode`."""
    out = _scan(tmp_path, 'extern "C" { fn SetErrorMode(m: u32) -> u32; }')
    assert len(out) == 1
    assert "SetErrorMode" in out[0] and "IMPORT_CC" in out[0]


def test_cdecl_declared_stdcall(tmp_path):
    """The other direction is equally a link failure: `extern "system"` on msvcrt's cdecl
    memcpy emits `_memcpy@12`."""
    out = _scan(tmp_path, 'extern "system" { fn memcpy(d: *mut u8, s: *const u8, n: usize); }')
    assert len(out) == 1 and "memcpy" in out[0]


def test_fastcall_export_declared_stdcall(tmp_path):
    """fastfat's real bug: a FASTCALL kernel export (`@Name@4`) declared "system" (`_Name@4`).
    It was worked around with an asm-aliased stdcall trampoline rather than fixed."""
    spec = _spec(tmp_path, "ntoskrnl", NTOSKRNL)
    out = scan_crate(
        _crate(tmp_path, 'extern "system" { fn ExAcquireFastMutexUnsafe(m: *mut u8); }'),
        {"ntoskrnl": read_spec(str(spec))},
        [],
    )
    assert len(out) == 1 and "@Name@N" in out[0]


def test_island_findings_are_tagged_separately(tmp_path):
    """A generated island transcribes the C header, so a clash with the .spec is upstream
    evidence to judge, not a decl to rewrite — the tag is what stops the next person
    'fixing' generated code."""
    d = tmp_path / "crate" / "src"
    d.mkdir(parents=True)
    (d / "bindgen_i386.rs").write_text('extern "C" { fn SetErrorMode(m: u32) -> u32; }')
    out = scan_crate(str(d), {"kernel32": read_spec(str(_spec(tmp_path, "k", KERNEL32)))}, [])
    assert len(out) == 1 and "[IMPORT_CC_ISLAND]" in out[0]


# --- MUST NOT FIRE ----------------------------------------------------------------------


def test_correct_declaration_is_silent(tmp_path):
    assert _scan(tmp_path, 'extern "system" { fn SetErrorMode(m: u32) -> u32; }') == []


def test_cfg_block_expression_picks_the_convention_per_arch(tmp_path):
    """ntkrnl-sys/guards.rs is RIGHT: fastcall on x86, cdecl elsewhere. The attribute is on
    the block, not on the extern inside it."""
    assert (
        _scan(
            tmp_path,
            """
pub unsafe fn raise(n: u8) -> u8 {
    #[cfg(not(target_arch = "x86"))]
    {
        extern "C" { fn SetErrorMode(m: u32) -> u32; }
        SetErrorMode(n as u32) as u8
    }
    #[cfg(target_arch = "x86")]
    {
        extern "system" { fn SetErrorMode(m: u32) -> u32; }
        SetErrorMode(n as u32) as u8
    }
}
""",
        )
        == []
    )


def test_no_mangle_definition_without_pub_is_still_a_definition(tmp_path):
    """usbhub-rust defines InterlockedIncrement itself and deliberately leaves `pub` off so it
    stays out of the glob. #[no_mangle] makes it global to the LINKER regardless, so it is not
    an import of ntoskrnl's fastcall export of the same name."""
    assert (
        _scan(
            tmp_path,
            """
extern "system" { fn SetErrorMode(m: u32) -> u32; }
#[no_mangle]
unsafe extern "C" fn SetErrorMode(_m: u32) -> u32 { 0 }
""",
        )
        == []
    )


def test_symbol_defined_by_the_modules_own_c(tmp_path):
    """fastfat gives ExInterlockedPopEntrySList an out-of-line body because the DDK only has a
    FORCEINLINE, so its Rust calls the LOCAL stdcall one, not ntoskrnl's fastcall export."""
    mod = tmp_path / "mod"
    mod.mkdir()
    (mod / "shim.c").write_text(
        "DWORD SetErrorMode(UINT mode)\n{\n    return real_set_error_mode(mode);\n}\n"
    )
    assert (
        _scan(tmp_path, 'extern "C" { fn SetErrorMode(m: u32) -> u32; }', module_dirs=[str(mod)])
        == []
    )


def test_a_prototype_is_not_a_definition(tmp_path):
    """The discriminator above must not be fooled by a declaration or a call — otherwise any
    module that merely USES the symbol suppresses its own finding."""
    mod = tmp_path / "mod"
    mod.mkdir()
    (mod / "use.c").write_text(
        "DWORD SetErrorMode(UINT mode);\nvoid f(void) { SetErrorMode(1); }\n"
    )
    assert c_defines([str(mod)], "SetErrorMode") is None


def test_stem_match_needs_a_spec_to_corroborate_it(tmp_path):
    """Rule 2 exists for unflipped crates like wldap32-rust. Without the spec requirement it
    also matches rosapps' `ps` utility for ps-rust — ntoskrnl's Ps subsystem — and judges
    kernel code against kernel32."""
    by_target = {"ps": {"kernel32"}, "wldap32": {"kernel32"}}
    specs = {"wldap32": ["/somewhere/wldap32.spec"]}
    assert crate_link_set("ps-rust", {}, by_target, {}, specs) is None
    libs, _dirs = crate_link_set("wldap32-rust", {}, by_target, {}, specs)
    assert libs == {"kernel32"}


def test_a_named_cmakelists_wins_even_when_it_imports_nothing(tmp_path):
    """ntoskrnl and freeldr link no import libs. "Imports nothing" is an ANSWER — falling
    through to the stem rule there is how kernel-rust got judged against ntdll."""
    libs, _dirs = crate_link_set(
        "kernel-rust",
        {"kernel-rust": {"/repo/ntoskrnl/CMakeLists.txt"}},
        {"kernel": {"ntdll"}},
        {},
        {},
    )
    assert libs == set()


def test_nolint_suppresses(tmp_path):
    assert (
        _scan(
            tmp_path,
            'extern "C" { fn SetErrorMode(m: u32) -> u32; } // nolint: import-cc (verified)',
        )
        == []
    )


def test_unknown_symbol_is_not_evidence(tmp_path):
    assert _scan(tmp_path, 'extern "C" { fn NotInAnySpec(m: u32) -> u32; }') == []
