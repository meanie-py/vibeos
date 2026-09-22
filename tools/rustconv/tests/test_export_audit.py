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

from rustconv.asm_defs import (
    AsmDefsError,
    asm_provided_names,
    module_asm_sources,
    syscall_table_names,
)
from rustconv.config import REACTOS_ROOT
from rustconv.export_audit import (
    audit_exports,
    audit_exports_detailed,
    module_rust_crates,
    scan_rust_fns,
)
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


def test_a_bare_target_is_a_local_rename_not_a_forward(tmp_path):
    """Only a DOT makes a target a forward (spec2def.c:691). A bare one renames the symbol
    the linker looks for, and calling it "forwarded" hid 7 genuinely-missing wldap32
    exports behind a green audit."""
    spec = parse_spec(
        _write(
            tmp_path,
            "x.spec",
            "310 varargs ber_printf(ptr str) WLDAP32_ber_printf\n"
            "@ stdcall CoFileTimeNow(ptr) kernel32.FileTimeNow\n",
        )
    )
    e = spec["ber_printf"]
    assert not e.is_forwarded
    assert e.forward_target is None
    assert e.impl_symbol == "WLDAP32_ber_printf"
    assert e.symbol == "WLDAP32_ber_printf"  # what the linker wants
    assert e.needs_implementation
    assert spec["CoFileTimeNow"].symbol == "CoFileTimeNow"  # forwards keep their own name


def test_version_excluded_entries_need_no_implementation(tmp_path):
    """We spec2def at DLL_EXPORT_VERSION 0x502 (config.cmake:87), so a `0x600+` row is not
    in the .def at all. Auditing them invented 23 phantom MISSING exports on ws2_32 --
    the entire `Wah*` family -- in a module that is fully flipped and links."""
    spec = parse_spec(
        _write(
            tmp_path,
            "x.spec",
            "@ stdcall -version=0x600+ WahCloseApcHelper(long)\n"
            "@ stdcall -version=0x501-0x502 OldThing(long)\n"
            "@ stdcall -version=0x400,0x600+ EitherEnd(long)\n"
            "@ stdcall Always(long)\n",
        )
    )
    assert not spec["WahCloseApcHelper"].in_version
    assert not spec["WahCloseApcHelper"].needs_implementation
    assert spec["OldThing"].needs_implementation  # 0x502 is the range's top end
    assert not spec["EitherEnd"].needs_implementation  # 0x502 falls in neither part
    assert spec["Always"].needs_implementation


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


# ---------------------------------------------------------------------------
# GENERATED: the build's third supplier of symbols is assembly
# ---------------------------------------------------------------------------


def _syscall_tree(tmp_path: Path, stub: str) -> tuple[Path, Path]:
    """A miniature of ntoskrnl/ntdll.S + ntoskrnl/include/sysfuncs.h."""
    (tmp_path / "ntoskrnl" / "include").mkdir(parents=True)
    (tmp_path / "ntoskrnl" / "include" / "sysfuncs.h").write_text(
        "    SVC_(Close, 1)\n    SVC_(OpenFile, 6)\n"
    )
    asm = tmp_path / "ntoskrnl" / "stubs.S"
    asm.write_text(
        f"#include <syscalls.inc>\n#define SVC_(name, argcount) {stub} name, argcount\n#include <sysfuncs.h>\n"
    )
    return asm, tmp_path


def test_stub_u_generates_both_the_nt_and_the_zw_half(tmp_path):
    asm, root = _syscall_tree(tmp_path, "STUB_U")
    assert set(syscall_table_names(asm, root)) == {"NtClose", "ZwClose", "NtOpenFile", "ZwOpenFile"}


def test_stub_k_generates_only_the_zw_half(tmp_path):
    # ntoskrnl/ex/zw.S: the kernel's Nt* are real code, only Zw* is a stub.
    asm, root = _syscall_tree(tmp_path, "STUB_K")
    assert set(syscall_table_names(asm, root)) == {"ZwClose", "ZwOpenFile"}


def test_a_file_with_no_svc_define_expands_nothing(tmp_path):
    (tmp_path / "plain.S").write_text("#include <ksamd64.inc>\nPUBLIC _KiFastSystemCall@0\n")
    assert syscall_table_names(tmp_path / "plain.S", tmp_path) == set()


def test_an_unreadable_syscall_table_fails_loud(tmp_path):
    # Silently expanding to nothing would answer "593 exports are MISSING" with confidence.
    asm = tmp_path / "stubs.S"
    asm.write_text("#define SVC_(name, argcount) STUB_U name, argcount\n#include <nowhere.h>\n")
    with pytest.raises(AsmDefsError, match="SVC_ syscall table"):
        syscall_table_names(asm, tmp_path)


def test_generated_syscall_stubs_are_not_missing(tmp_path):
    asm, root = _syscall_tree(tmp_path, "STUB_U")
    spec = _write(tmp_path, "x.spec", "@ stdcall NtClose(long)\n@ stdcall NtDropped(long)\n")
    rs = _write(tmp_path, "lib.rs", "// nothing here\n")
    findings = audit_exports_detailed(spec, [rs], repo_root=root, asm_srcs=[asm])
    assert [(f.kind, f.name) for f in findings] == [
        ("GENERATED", "NtClose"),
        ("MISSING", "NtDropped"),
    ]


def test_a_literal_asm_label_also_counts(tmp_path):
    # amd64 labels are undecorated on both sides, so this is the plain case.
    asm = tmp_path / "chkstk_ms.s"
    asm.write_text(".globl __chkstk\n__chkstk:\n    ret\n")
    spec = _write(tmp_path, "x.spec", "@ cdecl __chkstk()\n")
    rs = _write(tmp_path, "lib.rs", "// nothing here\n")
    findings = audit_exports_detailed(spec, [rs], repo_root=tmp_path, asm_srcs=[asm], arch="amd64")
    assert [(f.kind, f.name) for f in findings] == [("GENERATED", "__chkstk")]


def test_i386_asm_decoration_is_stripped_before_matching(tmp_path):
    # The asm carries the decoration the linker sees — `_KiFastSystemCall@0` for stdcall,
    # `__alldiv` for cdecl; the .spec carries the bare name and spec2def adds `_`/`@N`.
    asm = tmp_path / "dispatch.S"
    asm.write_text("PUBLIC _KiFastSystemCall@0\nPUBLIC __alldiv\n")
    spec = _write(tmp_path, "x.spec", "@ stdcall KiFastSystemCall()\n@ cdecl _alldiv()\n")
    rs = _write(tmp_path, "lib.rs", "// nothing here\n")
    findings = audit_exports_detailed(spec, [rs], repo_root=tmp_path, asm_srcs=[asm], arch="i386")
    assert [(f.kind, f.name) for f in findings] == [
        ("GENERATED", "KiFastSystemCall"),
        ("GENERATED", "_alldiv"),
    ]


def test_ntdll_nt_and_zw_exports_are_all_generated():
    # The real thing: 593 of ntdll's 649 MISSING rows were its syscall stubs.
    crates = module_rust_crates(REACTOS_ROOT / "dll/ntdll", REACTOS_ROOT / "sdk/lib/rust")
    asm = module_asm_sources(REACTOS_ROOT / "dll/ntdll", REACTOS_ROOT, crates)
    names = asm_provided_names(asm, REACTOS_ROOT)
    assert names["NtClose"].startswith("a generated syscall stub")
    assert names["ZwClose"].startswith("a generated syscall stub")
    assert len([n for n in names if n.startswith(("Nt", "Zw"))]) > 500


# ---------------------------------------------------------------------------
# Reading the build's own declaration of what it links
# ---------------------------------------------------------------------------


def test_add_cargo_kernel_dll_is_a_cargo_link_too():
    # hal.dll uses the second spelling; matching only add_cargo_module found it zero crates.
    crates = module_rust_crates(REACTOS_ROOT / "hal/halx86", REACTOS_ROOT / "sdk/lib/rust")
    assert "hal-rust" in {c.name for c in crates}


def test_crates_declared_in_an_included_cmake_are_found():
    # freeldr declares BOTH its cargo links behind include(rosload.cmake) / include(uefi.cmake).
    crates = module_rust_crates(
        REACTOS_ROOT / "boot/freeldr/freeldr", REACTOS_ROOT / "sdk/lib/rust"
    )
    names = {c.name for c in crates}
    assert {"rosload-cargo", "uefildr-cargo"} <= names


# ---------------------------------------------------------------------------
# .spec rows this arch does not build
# ---------------------------------------------------------------------------


def test_arch_gated_rows_are_dropped(tmp_path):
    spec_file = _write(
        tmp_path,
        "x.spec",
        "@ fastcall -arch=arm HalSweepIcache()\n@ stdcall -arch=i386,x86_64 HalGetBus()\n",
    )
    assert set(parse_spec(spec_file, arch="i386")) == {"HalGetBus"}
    assert set(parse_spec(spec_file, arch="arm")) == {"HalSweepIcache"}
    assert set(parse_spec(spec_file)) == {"HalSweepIcache", "HalGetBus"}


def test_the_win32_win64_width_aliases_are_honoured(tmp_path):
    # spec2def matches -arch= against the arch name OR its width alias.
    spec_file = _write(
        tmp_path, "x.spec", "@ stdcall -arch=win64 Wide()\n@ stdcall -arch=win32 Narrow()\n"
    )
    assert set(parse_spec(spec_file, arch="arm64")) == {"Wide"}
    assert set(parse_spec(spec_file, arch="amd64")) == {"Wide"}
    assert set(parse_spec(spec_file, arch="i386")) == {"Narrow"}


def test_per_arch_twins_do_not_collapse_onto_each_other(tmp_path):
    # hal.spec declares ExAcquireFastMutex twice; keyed by name, the last row won
    # regardless of which arch was being audited.
    spec_file = _write(
        tmp_path,
        "x.spec",
        "@ fastcall -arch=arm,arm64 ExAcquireFastMutex(ptr)\n"
        "@ fastcall -arch=i386 ExAcquireFastMutex(ptr) ntoskrnl.ExiAcquireFastMutex\n",
    )
    assert parse_spec(spec_file, arch="arm64")["ExAcquireFastMutex"].forward_target is None
    assert parse_spec(spec_file, arch="i386")["ExAcquireFastMutex"].forward_target is not None


def test_import_flagged_entries_need_no_local_symbol(tmp_path):
    """`-import` means the LINKER resolves the export out of an import library.

    spec2def ignores the flag outright (spec2def.c:1209, "The export is imported from an
    import library. Ignored."), so the .def carries a plain export and the jump thunk in
    e.g. libole32.a satisfies it. Nothing has to exist in this module's own object code.

    This rotted in the 2026-09-20 upstream merge: oleaut32's Get/Set/CreateErrorInfo went
    from `ole32.GetErrorInfo` forwarders to `-import`, and the audit — which correctly
    skips forwarders — reported all three as MISSING against a tree that links fine.
    """
    spec_file = _write(
        tmp_path,
        "x.spec",
        "200 stdcall -import GetErrorInfo(long ptr)\n201 stdcall LocalOne(long)\n",
    )
    parsed = parse_spec(spec_file)
    assert parsed["GetErrorInfo"].needs_implementation is False
    # ...and an ordinary neighbour on the same spec still does.
    assert parsed["LocalOne"].needs_implementation is True
