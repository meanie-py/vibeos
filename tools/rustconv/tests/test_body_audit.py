"""body_audit: each rule is pinned against the shape that motivated it."""

from pathlib import Path

from rustconv.body_audit import audit


def _run(tmp_path: Path, c: str, rust: str, *, glue: str = "", macros=None):
    cf = tmp_path / "mod.c"
    cf.write_text(c)
    rf = tmp_path / "mod.rs"
    rf.write_text(rust)
    glue_files = []
    if glue:
        g = tmp_path / "rust_glue.c"
        g.write_text(glue)
        glue_files = [g]
    return {
        f.function: f for f in audit([cf], [rf], tmp_path, glue_c=glue_files, macros=macros or {})
    }


def test_the_fastfat_shape_a_dropped_bitmap_update_is_high(tmp_path):
    """FatCreateNewDirent lost `RtlSetBits`, so new dirents stayed FREE (2026-09-26)."""
    c = """
ULONG FatCreateNewDirent(PDCB Dcb, ULONG N)
{
    ULONG Off = RtlFindClearBits(&Dcb->Bitmap, N, 0);
    RtlSetBits(&Dcb->Bitmap, Off, N);
    return Off;
}
"""
    rust = """
pub unsafe extern "system" fn FatCreateNewDirent(Dcb: PDCB, N: u32) -> u32 {
    // RtlSetBits(&Dcb->Bitmap, Off, N);   <- quoted C in a comment is NOT a call
    let off = RtlFindClearBits(addr_of_mut!((*Dcb).Bitmap), N, 0);
    off
}
"""
    got = _run(tmp_path, c, rust)
    assert got["FatCreateNewDirent"].severity == "high"
    assert got["FatCreateNewDirent"].dropped == ["RtlSetBits"]


def test_lowerings_dead_arms_and_tracing_are_not_findings(tmp_path):
    c = """
VOID Clean(PVOID p, ULONG n)
{
    DebugTrace(0, Dbg, "x", 0);
    RtlZeroMemory(p, n);
#ifndef __REACTOS__
    WineOnlyHelper(p);
#endif
    _SEH2_TRY {
        Work(p);
    } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
        HandlerOnly(p);
    } _SEH2_END;
}
"""
    rust = """
pub unsafe fn Clean(p: PVOID, n: u32) {
    core::ptr::write_bytes(p as *mut u8, 0, n as usize);
    Work(p);
}
"""
    assert _run(tmp_path, c, rust) == {}


def test_a_finally_block_still_runs_so_its_calls_are_owed(tmp_path):
    """Under DUMMY_PSEH the except handler is dead, but `_SEH2_FINALLY` executes."""
    c = """
VOID Pin(PBCB b)
{
    _SEH2_TRY {
        Work(b);
    } _SEH2_FINALLY {
        FatUnpinBcb(b);
    } _SEH2_END;
}
"""
    rust = "pub unsafe fn Pin(b: PBCB) { Work(b); }\n"
    assert _run(tmp_path, c, rust)["Pin"].dropped == ["FatUnpinBcb"]


def test_a_wrapper_is_judged_with_the_helper_it_delegates_to(tmp_path):
    """cdrom's DeviceEvtFileClose is a 3-line call into a renamed body (here, live C glue)."""
    c = """
VOID DeviceEvtFileClose(WDFFILEOBJECT f)
{
    DeviceCleanupProtectedLocks(f);
    DeviceUnlockExclusive(f);
}
"""
    rust = "pub unsafe fn DeviceEvtFileClose(f: WDFFILEOBJECT) { CdromDeviceEvtFileClose(f); }\n"
    glue = """
VOID CdromDeviceEvtFileClose(WDFFILEOBJECT f)
{
    DeviceCleanupProtectedLocks(f);
    DeviceUnlockExclusive(f);
}
"""
    assert _run(tmp_path, c, rust, glue=glue) == {}
    rust_helper = rust + (
        "unsafe fn CdromDeviceEvtFileClose(f: WDFFILEOBJECT) {\n"
        "    DeviceCleanupProtectedLocks(f);\n}\n"
    )
    assert _run(tmp_path, c, rust_helper)["DeviceEvtFileClose"].dropped == ["DeviceUnlockExclusive"]


def test_cfg_split_twins_are_judged_as_a_union(tmp_path):
    """hal's HalpQueryPciRegistryInfo: full body + a `minihal` NULL face in ONE file."""
    c = """
PVOID HalpQueryPciRegistryInfo(VOID)
{
    ZwOpenKey(0);
    ZwClose(0);
    return 0;
}
"""
    rust = """
#[cfg(not(feature = "minihal"))]
pub unsafe fn HalpQueryPciRegistryInfo() -> PVOID { ZwOpenKey(0); ZwClose(0); null_mut() }
#[cfg(feature = "minihal")]
pub unsafe fn HalpQueryPciRegistryInfo() -> PVOID { null_mut() }
"""
    assert _run(tmp_path, c, rust) == {}


def test_macros_are_judged_by_the_calls_they_expand_to(tmp_path):
    c = """
VOID Q(PVCB v)
{
    SetFlag(v->Flags, 1);
    FatAcquireCloseMutex();
    KeWaitForMutexObject(v);
}
"""
    macros = {
        "SetFlag": "((F) |= (SF))",
        "FatAcquireCloseMutex": "ExAcquireFastMutexUnsafe(&FatCloseQueueMutex)",
        "KeWaitForMutexObject": "KeWaitForSingleObject()",
    }
    rust = "pub unsafe fn Q(v: PVCB) { (*v).Flags |= 1; KeWaitForSingleObject(v); }\n"
    got = _run(tmp_path, c, rust, macros=macros)
    assert got["Q"].dropped == ["FatAcquireCloseMutex(->ExAcquireFastMutexUnsafe)"]


def test_an_empty_body_is_a_silent_stub_and_a_bugcheck_is_a_loud_one(tmp_path):
    body = "\n".join(f"    Step{i}(x);" for i in range(40))
    c = f"VOID Big(PVOID x)\n{{\n{body}\n}}\nVOID Loud(PVOID x)\n{{\n{body}\n}}\n"
    rust = (
        "pub unsafe fn Big(x: PVOID) { let _ = x; }\n"
        "pub unsafe fn Loud(x: PVOID) { FatBugCheck(0, 0, 0); }\n"
    )
    got = _run(tmp_path, c, rust)
    assert got["Big"].severity == "stub-silent"
    assert got["Loud"].severity == "stub-loud"


def test_com_vtable_macros_and_aw_names_resolve(tmp_path):
    c = """
HRESULT Bind(IShellFolder *psf)
{
    IShellFolder_Release(psf);
    return SearchPath(0);
}
"""
    rust = (
        "pub unsafe fn Bind(psf: *mut IShellFolder) -> HRESULT {\n"
        "    ((*(*psf).lpVtbl).Release)(psf);\n    SearchPathW(null())\n}\n"
    )
    assert _run(tmp_path, c, rust) == {}


def test_version_guards_fold_so_win8_blocks_are_not_owed(tmp_path):
    """fastfat's FatMountVolume: IoVolumeDeviceToGuid sits under NTDDI_VERSION > NTDDI_WIN8."""
    c = """
VOID Mount(PVCB v)
{
#if (NTDDI_VERSION > NTDDI_WIN8)
    IoVolumeDeviceToGuid(v);
#endif
    Work(v);
}
"""
    rust = "pub unsafe fn Mount(v: PVCB) { Work(v); }\n"
    pins = {"NTDDI_VERSION": 0x05020100, "NTDDI_WIN8": 0x06020000}
    cf = tmp_path / "m.c"
    cf.write_text(c)
    rf = tmp_path / "m.rs"
    rf.write_text(rust)
    assert audit([cf], [rf], tmp_path, pins=pins) == []
    # Without the pins the guard is undecidable, so the call reads as owed.
    assert audit([cf], [rf], tmp_path)[0].dropped == ["IoVolumeDeviceToGuid"]


def test_an_abnormal_termination_only_block_is_dead_under_dummy_pseh(tmp_path):
    """pseh2.h's DUMMY arm defines _SEH2_AbnormalTermination() as (0); `|| !NT_SUCCESS`
    keeps the block live."""
    c = """
VOID Open(PFCB f, NTSTATUS s)
{
    _SEH2_TRY { Work(f); } _SEH2_FINALLY {
        if (_SEH2_AbnormalTermination()) { FatDeleteFcb(f); }
        if (_SEH2_AbnormalTermination() || !NT_SUCCESS(s)) { FatDeleteCcb(f); }
    } _SEH2_END;
}
"""
    rust = "pub unsafe fn Open(f: PFCB, s: NTSTATUS) { Work(f); }\n"
    assert _run(tmp_path, c, rust)["Open"].dropped == ["FatDeleteCcb"]


def test_macros_defined_in_the_c_file_and_dbg_wrappers_resolve(tmp_path):
    """allocsup.c defines FatReserveClusters itself; dbgbitmap.h renames RtlSetBits."""
    c = """
VOID Alloc(PVCB v, ULONG i, ULONG n)
{
    FatReserveClusters(v, i, n);
}
"""
    macros = {"FatReserveClusters": "{ NT_ASSERT(Check(n)); RtlSetBitsDbg(&v->Map, i - 2, n); }"}
    rust = "pub unsafe fn Alloc(v: PVCB, i: u32, n: u32) { RtlSetBits(map(v), i - 2, n); }\n"
    assert _run(tmp_path, c, rust, macros=macros) == {}


def test_a_crate_on_real_seh_owes_its_handlers_and_unwinds(tmp_path):
    """fastfat moved to the ntkrnl_sys::seh trampolines: its except handlers now run."""
    c = """
VOID Open(PFCB f)
{
    _SEH2_TRY { Work(f); } _SEH2_FINALLY {
        if (_SEH2_AbnormalTermination()) { FatDeleteFcb(f); }
    } _SEH2_END;
}
"""
    rust = "pub unsafe fn Open(f: PFCB) { Work(f); }\n"
    cf = tmp_path / "o.c"
    cf.write_text(c)
    rf = tmp_path / "o.rs"
    rf.write_text(rust)
    assert audit([cf], [rf], tmp_path) == []
    rf.write_text("use ntkrnl_sys::seh::try_finally;\n" + rust)
    assert audit([cf], [rf], tmp_path)[0].dropped == ["FatDeleteFcb"]


def test_a_dead_c_file_superseded_by_live_c_is_not_a_translation(tmp_path):
    """gdi32: Wine's enhmfdrv/bitblt.c sits unbuilt beside the live emfdrv.c stub it lost to."""
    c = (
        "BOOL EMFDRV_PatBlt(PHYSDEV d)\n{\n    EMFDRV_WriteRecord(d);\n"
        "    EMFDRV_UpdateBBox(d);\n}\n"
    )
    glue = "BOOL EMFDRV_PatBlt(WINEDC *dc)\n{\n    return TRUE;\n}\n"
    assert _run(tmp_path, c, "pub fn unrelated() {}\n", glue=glue) == {}


def test_a_commented_out_feature_switch_pins_its_guard_off(tmp_path):
    """wlansvc: `//#define GET_IF_ENTRY2_IMPLEMENTED 1` leaves the UNIMPLEMENTED arm live."""
    from rustconv.body_audit import disabled_switches

    c = """//#define GET_IF_ENTRY2_IMPLEMENTED 1
//#define ALSO_ON 1
#define ALSO_ON 1
DWORD Enum(PVOID p)
{
#if GET_IF_ENTRY2_IMPLEMENTED
    GetIfEntry2(p);
#else
    UNIMPLEMENTED;
#endif
    return 0;
}
"""
    (tmp_path / "rpc.c").write_text(c)
    pins = disabled_switches(tmp_path, "")
    assert pins == {"GET_IF_ENTRY2_IMPLEMENTED": 0}
    rf = tmp_path / "rpc.rs"
    rf.write_text("pub unsafe fn Enum(p: PVOID) -> u32 { 0 }\n")
    assert audit([tmp_path / "rpc.c"], [rf], tmp_path, pins=pins) == []
    assert audit([tmp_path / "rpc.c"], [rf], tmp_path)[0].dropped == ["GetIfEntry2"]


def test_version_aliases_pin_and_an_inlined_completion_routine_is_kept(tmp_path):
    """fastfat: FatOpenVolume's `NTDDI_VISTA` guard; FatSingleAsync's hand-expanded
    IoSetCompletionRoutine."""
    from rustconv.body_audit import version_pins

    inc = tmp_path / "sdk" / "include" / "psdk"
    inc.mkdir(parents=True)
    (inc / "sdkddkver.h").write_text(
        "#define NTDDI_WIN6 0x06000000\n#define NTDDI_VISTA NTDDI_WIN6\n"
        "#define NTDDI_WS03SP1 0x05020100\n"
    )
    mod = tmp_path / "m"
    mod.mkdir()
    (mod / "CMakeLists.txt").write_text("")
    pins = version_pins(mod, tmp_path)
    assert pins["NTDDI_VISTA"] == 0x06000000
    c = """
VOID Open(PIRP Irp)
{
#if (NTDDI_VERSION >= NTDDI_VISTA)
    FsRtlAreVolumeStartupApplicationsComplete();
#endif
    IoSetCompletionRoutine(Irp, Done, 0, TRUE, TRUE, TRUE);
}
"""
    cf = mod / "o.c"
    cf.write_text(c)
    rf = mod / "o.rs"
    rf.write_text("pub unsafe fn Open(Irp: PIRP) { (*sp).CompletionRoutine = Some(Done); }\n")
    assert audit([cf], [rf], tmp_path, pins=pins) == []


def test_a_nested_macro_called_by_name_keeps_what_it_expands_to(tmp_path):
    """fastfat's FatLocateDirent expands FatReadDirent inline and calls the cohab FUNCTION
    FatUnpinBcb, which stands in for the FatUnpinBcb -> CcUnpinData macro."""
    c = "VOID Locate(PBCB *b)\n{\n    FatReadDirent(b);\n}\n"
    macros = {
        "FatReadDirent": "{ FatUnpinBcb(b); FatReadDirectoryFile(b); }",
        "FatUnpinBcb": "{ CcUnpinData(b); }",
    }
    rust = "pub unsafe fn Locate(b: *mut PBCB) { FatUnpinBcb(b); FatReadDirectoryFile(b); }\n"
    assert _run(tmp_path, c, rust, macros=macros) == {}
    rust_lost = "pub unsafe fn Locate(b: *mut PBCB) { FatReadDirectoryFile(b); }\n"
    got = _run(tmp_path, c, rust_lost, macros=macros)["Locate"]
    assert got.dropped == ["FatReadDirent(->CcUnpinData)"]


def test_reviewed_ledger_hides_only_the_callees_it_names(tmp_path):
    """A reviewed lead stays hidden until a drop the reviewer never saw appears."""
    from rustconv.body_audit import Finding, apply_reviewed, load_reviewed, render

    led = tmp_path / "reviewed.toml"
    led.write_text(
        '[[reviewed]]\nmodule = "m"\nfunction = "F"\ncallees = ["RtlSetBits"]\n'
        'verdict = "false"\nnote = "kept by a helper"\n'
        '[[reviewed]]\nmodule = "m"\nfunction = "O"\ncallees = ["IoStartPacket"]\n'
        'verdict = "owed"\nnote = "lane x"\n'
    )
    ledger = load_reviewed(led)
    f_ok = Finding("F", "a.c", "a.rs", "high", 10, 10, ["RtlSetBits"])
    f_new = Finding("F", "a.c", "a.rs", "high", 10, 10, ["RtlSetBits", "RtlClearBits"])
    f_owed = Finding("O", "a.c", "a.rs", "high", 10, 10, ["IoStartPacket"])
    apply_reviewed("m", [f_ok, f_owed], ledger)
    apply_reviewed("m", [f_new], ledger)
    assert f_ok.reviewed == ("false", "kept by a helper")
    assert f_new.reviewed is None
    assert f_owed.reviewed == ("owed", "lane x")
    out = render("m", [f_ok, f_owed], 0)
    assert out.splitlines()[0] == "body-audit m: clean; 1 owed; 1 reviewed"
    assert "  [high, owed] O" in out and "] F " not in out


def test_reviewed_ledger_rejects_an_entry_without_a_note(tmp_path):
    import pytest

    from rustconv.body_audit import load_reviewed

    led = tmp_path / "reviewed.toml"
    led.write_text(
        '[[reviewed]]\nmodule = "m"\nfunction = "F"\ncallees = ["X"]\nverdict = "false"\n'
    )
    with pytest.raises(ValueError):
        load_reviewed(led)


def test_the_committed_ledger_loads():
    from rustconv.body_audit import load_reviewed

    assert load_reviewed()


def test_provenance_frames_the_definition_not_a_call_site():
    """cdrom: `if (BootEnvironmentIsWinPE()) {` precedes the definition in the file."""
    from rustconv.body_audit import _old_body

    c = (
        "BOOLEAN BootEnvironmentIsWinPE(VOID);\n"
        "NTSTATUS DriverEntry(PVOID d)\n{\n    if (BootEnvironmentIsWinPE()) {\n"
        "        SetFlag(d);\n    }\n}\n"
        "BOOLEAN\nBootEnvironmentIsWinPE(\n    VOID\n    )\n/* doc */\n{\n"
        "    WdfRegistryOpenKey(0);\n}\n"
    )
    body = _old_body(c, "BootEnvironmentIsWinPE")
    assert body is not None and "WdfRegistryOpenKey" in body and "SetFlag" not in body


def test_rules_from_the_2026_10_04_triage(tmp_path):
    """Each shape was a false lead in the high-lead burn-down."""
    c = """
#define NDEBUG
VOID A(PDEVICE_OBJECT d)
{
    UNICODE_STRING s = RTL_CONSTANT_STRING(L"x");
    IoReleaseRemoveLock(&d->Lock, 0);
    ClassAcquireRemoveLock(d, 0);
    _InterlockedExchangeAdd(&d->N, 1);
#ifndef NDEBUG
    DumpEverything(d);
#endif
#if defined(_USE_DUMMY_PSEH) || defined(NEVER_SEEN)
    LiveArm(d);
#else
    DeadArm(d);
#endif
#if 0 // retired
    Retired(d);
#endif
}
"""
    rust = """
#[link_name = "ClassAcquireRemoveLock"]
fn class_acquire_remove_lock(d: PVOID, t: PVOID) -> i32;
pub unsafe extern "system" fn A(d: PDEVICE_OBJECT) {
    IoReleaseRemoveLockEx(addr_of_mut!((*d).Lock), null_mut(), 0);
    class_acquire_remove_lock(d, null_mut());
    (*d).N.fetch_add(1, SeqCst);
    LiveArm(d);
}
"""
    assert _run(tmp_path, c, rust) == {}


def test_a_four_char_tag_closes_its_quote():
    """freeldr heap.c's `'dnE#'` left the masker inside a literal for 460 lines."""
    from rustconv.lex_mask import mask_code

    assert "Next" in mask_code("ULONG t = 'dnE#';\nNext(t);\n")


def test_printf_family_and_renamed_crt_spellings_are_kept(tmp_path):
    c = """
VOID F(LPWSTR b, LPCWSTR a, LPCWSTR x)
{
    swprintf(b, L"%s", a);
    if (wcsicmp(a, x) == 0 && IsEqualIID(&IID_A, &IID_B))
        abs(1);
}
"""
    rust = """
pub unsafe extern "system" fn F(b: LPWSTR, a: LPCWSTR, x: LPCWSTR) {
    StringCchPrintfW(b, 256, fmt.as_ptr(), a);
    if _wcsicmp(a, x) == 0 && guid_eq(&IID_A, &IID_B) { let _ = 1i32.abs(); }
}
"""
    assert _run(tmp_path, c, rust) == {}
