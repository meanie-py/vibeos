"""Export-coverage audit: the .spec is ground truth for what the DLL must define.

The bug this exists to catch (real, from the ole32 run): `StgCreateDocfile` came
out of the pipeline as a module-private Rust-ABI `unsafe fn` — so `_StgCreateDocfile@16`
was simply ABSENT from the linked DLL, and nothing noticed until the link failed.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.config import REACTOS_ROOT
from rustconv.export_audit import audit_exports, audit_exports_detailed, scan_rust_fns
from rustconv.spec_parser import SpecNotFound, find_spec_for_source, parse_spec

OLE32_SPEC = REACTOS_ROOT / "dll/win32/ole32/ole32.spec"
OLEAUT32_SPEC = REACTOS_ROOT / "dll/win32/oleaut32/oleaut32.spec"
OLE32_RUST = REACTOS_ROOT / "sdk/lib/rust/ole32-rust/src"
OLEAUT32_RUST = REACTOS_ROOT / "sdk/lib/rust/oleaut32-rust/src"


# ---------------------------------------------------------------------------
# spec_parser: the win32 (ordinal) dialect, flags, forwards
# ---------------------------------------------------------------------------


def _write(tmp_path: Path, name: str, text: str) -> Path:
    p = tmp_path / name
    p.write_text(text)
    return p


def test_parses_ordinal_dialect(tmp_path):
    spec = parse_spec(
        _write(
            tmp_path,
            "x.spec",
            "2 stdcall SysAllocString(wstr)\n"
            "@ stdcall -private DllRegisterServer()\n"
            "@ cdecl VarFmt(str)\n",
        )
    )
    assert spec["SysAllocString"].ordinal == 2
    assert spec["SysAllocString"].convention == "stdcall"
    assert spec["DllRegisterServer"].ordinal is None
    assert spec["DllRegisterServer"].is_private
    assert spec["DllRegisterServer"].needs_implementation
    assert spec["VarFmt"].convention == "cdecl"


def test_stub_extern_and_forward_need_no_implementation(tmp_path):
    spec = parse_spec(
        _write(
            tmp_path,
            "x.spec",
            "@ stub CoQueryReleaseObject\n"
            "@ stdcall -stub CoCancelCall(long long)\n"
            "@ extern CcFastMdlReadWait\n"
            "@ stdcall CoFileTimeNow(ptr) kernel32.FileTimeNow\n"
            "@ stdcall CoRealThing(ptr)\n",
        )
    )
    assert not spec["CoQueryReleaseObject"].needs_implementation  # spec2def emits the body
    assert spec["CoCancelCall"].is_stub
    assert not spec["CoCancelCall"].needs_implementation
    assert spec["CcFastMdlReadWait"].is_variable  # data export
    assert not spec["CcFastMdlReadWait"].needs_implementation
    assert spec["CoFileTimeNow"].is_forwarded  # .def forwards it at kernel32
    assert not spec["CoFileTimeNow"].needs_implementation
    assert spec["CoRealThing"].needs_implementation


def test_trailing_comment_does_not_drop_the_entry(tmp_path):
    # ntoskrnl.spec really has these two; the old regex silently dropped both.
    spec = parse_spec(
        _write(
            tmp_path,
            "x.spec",
            "@ fastcall -arch=i386 KiEoiHelper(ptr) #ReactOS-Specific\n",
        )
    )
    assert spec["KiEoiHelper"].convention == "fastcall"
    assert spec["KiEoiHelper"].arch_filter == ["i386"]


def test_ntoskrnl_default_still_works():
    spec = parse_spec()  # no arg == ntoskrnl.spec, the historical behaviour
    assert "ExAllocatePoolWithTag" in spec
    assert spec["ExAllocatePoolWithTag"].convention == "stdcall"


# ---------------------------------------------------------------------------
# find_spec_for_source
# ---------------------------------------------------------------------------


def test_find_spec_for_source_real_tree():
    got = find_spec_for_source(REACTOS_ROOT / "dll/win32/oleaut32/olefont.c")
    assert got == OLEAUT32_SPEC


def test_find_spec_for_source_walks_up_from_a_subdir():
    got = find_spec_for_source(REACTOS_ROOT / "dll/win32/ole32/ole32.spec")
    assert got == OLE32_SPEC


def test_find_spec_multi_spec_dir_prefers_dirname_spec():
    # uxtheme/, msvfw32/ and winmm/midimap/ each ship TWO .spec files; the
    # <dirname>.spec tie-break is what all three CMakeLists actually feed to
    # spec2def.
    got = find_spec_for_source(REACTOS_ROOT / "dll/win32/uxtheme/draw.c")
    assert got.name == "uxtheme.spec"


def test_find_spec_ambiguous_fails_loudly(tmp_path):
    (tmp_path / "a.spec").write_text("")
    (tmp_path / "b.spec").write_text("")
    with pytest.raises(SpecNotFound):
        find_spec_for_source(tmp_path / "x.c", root=tmp_path)


def test_find_spec_absent_fails_loudly(tmp_path):
    with pytest.raises(SpecNotFound):
        find_spec_for_source(tmp_path / "x.c", root=tmp_path)


# ---------------------------------------------------------------------------
# audit_exports — the three failure shapes
# ---------------------------------------------------------------------------

SPEC_3 = (
    "@ stdcall StgCreateDocfile(wstr long long ptr)\n"
    "@ stdcall CoTaskMemFree(ptr)\n"
    "@ cdecl DebugPrint(str)\n"
)


def test_clean_when_everything_is_exported(tmp_path):
    spec = _write(tmp_path, "x.spec", SPEC_3)
    rs = _write(
        tmp_path,
        "lib.rs",
        '#[no_mangle]\npub unsafe extern "system" fn StgCreateDocfile(a: u32) -> i32 { 0 }\n'
        '#[no_mangle]\npub unsafe extern "system" fn CoTaskMemFree(p: *mut u8) { }\n'
        '#[no_mangle]\npub unsafe extern "C" fn DebugPrint(s: *const u8) { }\n',
    )
    assert audit_exports(spec, [rs]) == []


def test_the_stgcreatedocfile_shape_is_caught(tmp_path):
    # EXACTLY the real ole32 defect: it exists, but private + Rust ABI, so the
    # symbol _StgCreateDocfile@16 never makes it into the DLL.
    spec = _write(tmp_path, "x.spec", "@ stdcall StgCreateDocfile(wstr long long ptr)\n")
    rs = _write(tmp_path, "lib.rs", "unsafe fn StgCreateDocfile(a: u32) -> i32 { 0 }\n")
    findings = audit_exports_detailed(spec, [rs])
    assert len(findings) == 1
    assert findings[0].kind == "NOT_EXPORTED"
    assert findings[0].name == "StgCreateDocfile"
    assert "no_mangle" in str(findings[0]).lower()
    assert 'extern "system"' in findings[0].expected


def test_missing_entirely(tmp_path):
    spec = _write(tmp_path, "x.spec", "@ stdcall CoGetMalloc(long ptr)\n")
    rs = _write(tmp_path, "lib.rs", "// nothing here\n")
    findings = audit_exports_detailed(spec, [rs])
    assert [(f.kind, f.name) for f in findings] == [("MISSING", "CoGetMalloc")]


def test_stdcall_emitted_as_extern_c_is_an_abi_mismatch(tmp_path):
    # links fine, then silently corrupts the i386 stack on every call
    spec = _write(tmp_path, "x.spec", "@ stdcall CoTaskMemFree(ptr)\n")
    rs = _write(
        tmp_path,
        "lib.rs",
        '#[no_mangle]\npub unsafe extern "C" fn CoTaskMemFree(p: *mut u8) { }\n',
    )
    findings = audit_exports_detailed(spec, [rs])
    assert [(f.kind, f.name) for f in findings] == [("ABI_MISMATCH", "CoTaskMemFree")]
    assert findings[0].expected == 'extern "system"'
    assert findings[0].found == 'extern "C"'


def test_stdcall_spelled_stdcall_is_accepted(tmp_path):
    spec = _write(tmp_path, "x.spec", "@ stdcall CoTaskMemFree(ptr)\n")
    rs = _write(
        tmp_path,
        "lib.rs",
        '#[no_mangle]\npub unsafe extern "stdcall" fn CoTaskMemFree(p: *mut u8) { }\n',
    )
    assert audit_exports(spec, [rs]) == []


def test_extern_block_declaration_is_not_an_implementation(tmp_path):
    # a bodiless decl inside `extern "C" { ... }` must not mask a MISSING export
    spec = _write(tmp_path, "x.spec", "@ stdcall StgCreateDocfile(wstr long long ptr)\n")
    rs = _write(
        tmp_path,
        "lib.rs",
        'extern "system" {\n    fn StgCreateDocfile(a: u32) -> i32;\n}\n',
    )
    findings = audit_exports_detailed(spec, [rs])
    assert [(f.kind, f.name) for f in findings] == [("MISSING", "StgCreateDocfile")]


def test_inline_attribute_style_is_seen(tmp_path):
    # oleaut32's vartype.rs writes `#[no_mangle] pub unsafe extern "system" fn X(`
    # on ONE line — a line-anchored scanner reports it as MISSING (it did).
    spec = _write(tmp_path, "x.spec", "339 stdcall VarI8FromStr(wstr long long ptr)\n")
    rs = _write(
        tmp_path,
        "lib.rs",
        '#[no_mangle] pub unsafe extern "system" fn VarI8FromStr(\n'
        "    s: *mut u16,\n"
        ") -> i32 { 0 }\n",
    )
    assert audit_exports(spec, [rs]) == []


def test_macro_rules_generated_exports_are_resolved(tmp_path):
    # ole32's 24 H*_User{Size,Marshal,Unmarshal,Free} exports are generated by a
    # macro_rules! (from the C `IMPL_WIREM_HANDLE(HWND)` macro).
    spec = _write(
        tmp_path,
        "x.spec",
        "@ stdcall HWND_UserSize(ptr long ptr)\n@ stdcall HWND_UserFree(ptr ptr)\n",
    )
    rs = _write(
        tmp_path,
        "lib.rs",
        "macro_rules! IMPL_WIREM_HANDLE {\n"
        "    ($ty:ty, $size:ident, $free:ident) => {\n"
        "        #[no_mangle]\n"
        '        pub unsafe extern "system" fn $size(f: *mut u32) -> u32 { 0 }\n'
        "        #[no_mangle]\n"
        '        pub unsafe extern "system" fn $free(f: *mut u32) { }\n'
        "    };\n"
        "}\n"
        "IMPL_WIREM_HANDLE!(HWND, HWND_UserSize, HWND_UserFree);\n",
    )
    assert audit_exports(spec, [rs]) == []


def test_export_name_attribute_is_honoured(tmp_path):
    spec = _write(tmp_path, "x.spec", "@ stdcall CoFoo(ptr)\n")
    rs = _write(
        tmp_path,
        "lib.rs",
        '#[export_name = "CoFoo"]\npub unsafe extern "system" fn co_foo_impl(p: *mut u8) { }\n',
    )
    assert audit_exports(spec, [rs]) == []


# ---------------------------------------------------------------------------
# real data
# ---------------------------------------------------------------------------


@pytest.mark.skipif(not OLE32_SPEC.exists(), reason="ole32 not in tree")
def test_ole32_is_fully_covered_by_rust():
    # ole32 is 100% Rust (its C SOURCE list is empty but for guid.c + the
    # generated RPC proxies) and it builds and boots — so the audit must be clean.
    assert audit_exports(OLE32_SPEC, [OLE32_RUST]) == []


@pytest.mark.skipif(not OLEAUT32_SPEC.exists(), reason="oleaut32 not in tree")
def test_oleaut32_exports_are_all_present_in_rust():
    """oleaut32 is now 100% Rust — every spec export must have a #[no_mangle] Rust
    definition with the spec's calling convention.

    This replaces a test that asserted the missing set was *exactly* the exports of
    the six then-untranslated C files (connpt/olefont/olepicture/olepropframe/
    recinfo/varformat). That was a good cross-check of the auditor at the time, but
    it encoded a snapshot of the migration, so it went red the moment those six were
    translated. The invariant below cannot rot: it stays true for the rest of the
    DLL's life, and it is exactly what the auditor exists to guarantee — the
    `StgCreateDocfile` defect (an export that came back a private Rust-ABI fn, so
    `_StgCreateDocfile@16` was absent from the DLL) is precisely what it catches.
    """
    assert audit_exports(OLEAUT32_SPEC, [OLEAUT32_RUST]) == []


@pytest.mark.skipif(not OLE32_RUST.exists(), reason="ole32-rust not in tree")
def test_scanner_finds_the_real_ole32_exports():
    fns = scan_rust_fns(sorted(OLE32_RUST.rglob("*.rs")))
    for name in ("CoCreateInstance", "StgCreateDocfile", "HWND_UserSize", "CoTaskMemAlloc"):
        assert fns[name].no_mangle, name
        assert fns[name].abi == "system", name
