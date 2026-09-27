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
