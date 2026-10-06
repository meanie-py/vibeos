#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NOT_IMPLEMENTED};
use ntkrnl_sys::constants::{
    STATUS_UNSUCCESSFUL, STATUS_ACCESS_DENIED, STATUS_INFO_LENGTH_MISMATCH,
};
use ntkrnl_sys::ddk::{ObReferenceObjectByHandle, PsGetCurrentThread, PsGetProcessWin32Process};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::RtlCopyMemory;
#[allow(unused_imports)]
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope type, constant, or table declarations to translate.

// Debug-channel / stub macros — no kernel symbol, no-op.
#[allow(unused_imports)]
use {ERR as _ERR, TRACE as _TRACE};
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// HWND_BROADCAST = (HWND)0xffff (winuser.h).
const HWND_BROADCAST: HWND = 0xffff as HWND;

// SafeProbeForRead/SafeProbeForWrite — NTAPI (`_Name@12`, stdcall i386 /
// cdecl elsewhere). Same shape as winsta.rs/console.rs.
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// SetLastNtError(`@SetLastNtError@4`) and IntGetDesktopObjectHandle
// (`@IntGetDesktopObjectHandle@4`) are FASTCALL on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn IntGetDesktopObjectHandle(DesktopObject: PDESKTOP) -> HDESK;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn IntGetDesktopObjectHandle(DesktopObject: PDESKTOP) -> HDESK;
}

// InitCsrApiPort (`_InitCsrApiPort`, no @N) is CDECL on all arches.
extern "C" {
    fn InitCsrApiPort(CsrPortHandle: HANDLE) -> NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NtUserBitBltSysBmp(
    hdc: HDC,
    nXDest: INT,
    nYDest: INT,
    nWidth: INT,
    nHeight: INT,
    nXSrc: INT,
    nYSrc: INT,
    dwRop: DWORD,
) -> BOOL {
    let mut Ret: BOOL = 0;
    UserEnterExclusive();

    Ret = NtGdiBitBlt(
        hdc,
        nXDest,
        nYDest,
        nWidth,
        nHeight,
        hSystemBM,
        nXSrc,
        nYSrc,
        dwRop,
        CLR_INVALID,
        0,
    );

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDragObject(
    hwnd1: HWND,
    hwnd2: HWND,
    u1: UINT,
    dw1: DWORD,
    hc1: HCURSOR,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDrawAnimatedRects(
    hwnd: HWND,
    idAni: INT,
    lprcFrom: *mut RECT,
    lprcTo: *mut RECT,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEvent(Unknown0: DWORD) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetAltTabInfo(
    hwnd: HWND,
    iItem: INT,
    pati: PALTTABINFO,
    pszItemText: LPWSTR,
    cchItemText: UINT,
    Ansi: BOOL,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserInitializeClientPfnArrays(
    pfnClientA: PPFNCLIENT,
    pfnClientW: PPFNCLIENT,
    pfnClientWorker: PPFNCLIENTWORKER,
    hmodUser: HINSTANCE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if ClientPfnInit != 0 {
        return Status;
    }

    UserEnterExclusive();

    Status = SafeProbeForRead(
        pfnClientA as *mut core::ffi::c_void,
        core::mem::size_of::<PFNCLIENT>() as _,
        1,
    );
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(
            pfnClientW as *mut core::ffi::c_void,
            core::mem::size_of::<PFNCLIENT>() as _,
            1,
        );
    }
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(
            pfnClientWorker as *mut core::ffi::c_void,
            core::mem::size_of::<PFNCLIENTWORKER>() as _,
            1,
        );
    }
    // The probes above VALIDATE only; these four captures are the access, and they
    // write the client callback tables the whole of user32's callback path dispatches
    // through. A fault lands in Status, checked just below, and the tables are only
    // declared initialised (ClientPfnInit) once every one of them has landed.
    if NT_SUCCESS(Status) {
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!((*gpsi).apfnClientA) as *mut core::ffi::c_void,
            pfnClientA as *const core::ffi::c_void,
            core::mem::size_of::<PFNCLIENT>(),
        ) || !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!((*gpsi).apfnClientW) as *mut core::ffi::c_void,
            pfnClientW as *const core::ffi::c_void,
            core::mem::size_of::<PFNCLIENT>(),
        ) || !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!((*gpsi).apfnClientWorker) as *mut core::ffi::c_void,
            pfnClientWorker as *const core::ffi::c_void,
            core::mem::size_of::<PFNCLIENTWORKER>(),
        ) || !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!((*gpsi).aStoCidPfn) as *mut core::ffi::c_void,
            pfnClientW as *const core::ffi::c_void,
            core::mem::size_of_val(&(*gpsi).aStoCidPfn),
        ) {
            Status = STATUS_ACCESS_VIOLATION;
        } else {
            hModClient = hmodUser;
            ClientPfnInit = 1;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    UserLeave();
    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtUserInitTask(
    Unknown0: DWORD,
    Unknown1: DWORD,
    Unknown2: DWORD,
    Unknown3: DWORD,
    Unknown4: DWORD,
    Unknown5: DWORD,
    Unknown6: DWORD,
    Unknown7: DWORD,
    Unknown8: DWORD,
    Unknown9: DWORD,
    Unknown10: DWORD,
    Unknown11: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserMNDragLeave() -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserMNDragOver(
    Unknown0: DWORD,
    Unknown1: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserModifyUserStartupInfoFlags(
    Unknown0: DWORD,
    Unknown1: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserQueryUserCounters(
    Unknown0: DWORD,
    Unknown1: DWORD,
    Unknown2: DWORD,
    Unknown3: DWORD,
    Unknown4: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRegisterTasklist(Unknown0: DWORD) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetConsoleReserveKeys(
    Unknown0: DWORD,
    Unknown1: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetDbgTag(
    Unknown0: DWORD,
    Unknown1: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetDbgTagCount(Unknown0: DWORD) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetRipFlags(Unknown0: DWORD) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDbgWin32HeapFail(
    Unknown0: DWORD,
    Unknown1: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDbgWin32HeapStat(
    Unknown0: DWORD,
    Unknown1: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetSysColors(
    cElements: core::ffi::c_int,
    lpaElements: *const INT,
    lpaRgbValues: *const COLORREF,
    Flags: FLONG,
) -> BOOL {
    let mut Ret: DWORD = 1;

    if cElements == 0 {
        return 1;
    }

    if (cElements as ULONG) >= 0x40000000 {
        EngSetLastError(ERROR_NOACCESS);
        return 0;
    }

    UserEnterExclusive();

    // The C probes both arrays inside a _SEH2_TRY whose handler calls
    // SetLastNtError and sets Ret = FALSE. Both probes were dropped, so
    // IntSetSysColors read cElements entries off two raw user pointers —
    // and cElements is the caller's, bounded above only by 0x40000000.
    let Status = SafeProbeForRead(
        lpaElements as PVOID,
        cElements as SIZE_T * core::mem::size_of::<INT>() as SIZE_T,
        1,
    );
    let Status = if NT_SUCCESS(Status) {
        SafeProbeForRead(
            lpaRgbValues as PVOID,
            cElements as SIZE_T * core::mem::size_of::<COLORREF>() as SIZE_T,
            1,
        )
    } else {
        Status
    };

    if NT_SUCCESS(Status) {
        // The probes above only range-check; they are validate-only by design under
        // USE_DUMMY_PSEH. The actual loads happen one level down, inside
        // IntSetSysColors, which is ALSO reachable with kernel pointers -- so the
        // capture belongs here, not there.
        //
        // Chunked deliberately: cElements is the caller's, bounded only by the
        // 0x40000000 test above, so a whole-array capture would be a user-triggered
        // 8 GB kernel allocation. IntSetSysColors applies each element
        // independently, so feeding it CHUNK at a time is equivalent.
        const SYSCOLOR_CHUNK: usize = 256;
        let mut Elements: [INT; SYSCOLOR_CHUNK] = [0; SYSCOLOR_CHUNK];
        let mut RgbValues: [COLORREF; SYSCOLOR_CHUNK] = [0; SYSCOLOR_CHUNK];
        let total = cElements as usize;
        let mut done = 0usize;
        while done < total {
            let n = core::cmp::min(SYSCOLOR_CHUNK, total - done);
            if !crate::copy::capture_from_user(
                Elements.as_mut_ptr() as *mut core::ffi::c_void,
                lpaElements.add(done) as *const core::ffi::c_void,
                n * core::mem::size_of::<INT>(),
            ) || !crate::copy::capture_from_user(
                RgbValues.as_mut_ptr() as *mut core::ffi::c_void,
                lpaRgbValues.add(done) as *const core::ffi::c_void,
                n * core::mem::size_of::<COLORREF>(),
            ) {
                SetLastNtError(STATUS_ACCESS_VIOLATION);
                Ret = 0;
                break;
            }
            IntSetSysColors(n as _, Elements.as_ptr(), RgbValues.as_ptr());
            done += n;
        }
    } else {
        SetLastNtError(Status);
        Ret = 0;
    }

    if Ret != 0 {
        UserSendNotifyMessage(HWND_BROADCAST, WM_SYSCOLORCHANGE, 0, 0);

        UserRedrawDesktop();
    }

    UserLeave();
    Ret as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUpdateInstance(
    Unknown0: DWORD,
    Unknown1: DWORD,
    Unknown2: DWORD,
) -> DWORD {
    0
}
// NtUserUserHandleGrantAccess: implemented in job.rs (upstream b272427baa3).
#[no_mangle]
pub unsafe extern "system" fn NtUserWaitForMsgAndEvent(Unknown0: DWORD) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserWin32PoolAllocationStats(
    Unknown0: DWORD,
    Unknown1: DWORD,
    Unknown2: DWORD,
    Unknown3: DWORD,
    Unknown4: DWORD,
    Unknown5: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserYieldTask() -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserHardErrorControl(
    dwUnknown1: DWORD,
    dwUnknown2: DWORD,
    dwUnknown3: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserNotifyProcessCreate(
    NewProcessId: HANDLE,
    ParentThreadId: HANDLE,
    dwUnknown: ULONG,
    CreateFlags: ULONG,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserProcessConnect(
    ProcessHandle: HANDLE,
    pUserConnect: PUSERCONNECT,
    Size: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let W32Process: PPROCESSINFO;

    if pUserConnect.is_null()
        || Size as usize != core::mem::size_of_val(&*pUserConnect)
    {
        return STATUS_UNSUCCESSFUL;
    }

    Status = ObReferenceObjectByHandle(
        ProcessHandle,
        PROCESS_VM_OPERATION,
        *PsProcessType as _,
        1i32 as i8,
        core::ptr::addr_of_mut!(Process) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    UserEnterShared();

    W32Process = PsGetProcessWin32Process(Process as _) as _;

    if W32Process.is_null() {
        UserLeave();
        ObDereferenceObject(Process as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    Status = SafeProbeForWrite(
        pUserConnect as PVOID,
        core::mem::size_of::<USERCONNECT>() as _,
        core::mem::size_of::<PVOID>() as _,
    );
    if NT_SUCCESS(Status) {
        // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped probe AND every one
        // of these stores in one _SEH2_TRY. Three of them also read ulSharedDelta
        // back OUT of the user struct to compute the next one. Assemble the whole
        // SHAREDINFO in kernel memory and publish it in one fault-tolerant copy.
        let mut siClient: SHAREDINFO = core::mem::zeroed();

        siClient.ulSharedDelta = ((*W32Process).HeapMappings.KernelMapping as ULONG_PTR)
            .wrapping_sub((*W32Process).HeapMappings.UserMapping as ULONG_PTR);

        siClient.psi = (gpsi as ULONG_PTR).wrapping_sub(siClient.ulSharedDelta) as PVOID as _;
        siClient.aheList =
            (gHandleTable as ULONG_PTR).wrapping_sub(siClient.ulSharedDelta) as PVOID as _;
        siClient.pDispInfo = core::ptr::null_mut();

        siClient.DefWindowMsgs.maxMsgs = 0;
        siClient.DefWindowMsgs.abMsgs = core::ptr::null_mut();
        siClient.DefWindowSpecMsgs.maxMsgs = 0;
        siClient.DefWindowSpecMsgs.abMsgs = core::ptr::null_mut();

        let count = siClient.awmControl.len();
        let mut i: UINT = 0;
        while (i as usize) < count {
            siClient.awmControl[i as usize].maxMsgs = 0;
            siClient.awmControl[i as usize].abMsgs = core::ptr::null_mut();
            i += 1;
        }

        if !crate::copy::publish_to_user(
            core::ptr::addr_of_mut!((*pUserConnect).siClient) as *mut core::ffi::c_void,
            core::ptr::addr_of!(siClient) as *const core::ffi::c_void,
            core::mem::size_of::<SHAREDINFO>(),
        ) {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    UserLeave();

    ObDereferenceObject(Process as PVOID);

    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtUserQueryInformationThread(
    ThreadHandle: HANDLE,
    ThreadInformationClass: USERTHREADINFOCLASS,
    ThreadInformation: PVOID,
    ThreadInformationLength: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Thread: PETHREAD = core::ptr::null_mut();

    if PsGetCurrentProcess() as PVOID != gpepCSRSS as PVOID {
        return STATUS_ACCESS_DENIED;
    }

    UserEnterExclusive();

    'quit: {
        Status = ObReferenceObjectByHandle(
            ThreadHandle,
            THREAD_QUERY_INFORMATION,
            *PsThreadType as _,
            1i32 as _,
            core::ptr::addr_of_mut!(Thread) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        match ThreadInformationClass {
            _ => {
                Status = STATUS_NOT_IMPLEMENTED;
            }
        }

        ObDereferenceObject(Thread as PVOID);
    }

    UserLeave();
    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRealInternalGetMessage(
    lpMsg: LPMSG,
    hWnd: HWND,
    wMsgFilterMin: UINT,
    wMsgFilterMax: UINT,
    wRemoveMsg: UINT,
    bGMSG: BOOL,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRealWaitMessageEx(
    dwWakeMask: DWORD,
    uTimeout: UINT,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserResolveDesktopForWOW(Unknown0: DWORD) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetInformationProcess(
    dwUnknown1: DWORD,
    dwUnknown2: DWORD,
    dwUnknown3: DWORD,
    dwUnknown4: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetInformationThread(
    ThreadHandle: HANDLE,
    ThreadInformationClass: USERTHREADINFOCLASS,
    ThreadInformation: PVOID,
    ThreadInformationLength: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Thread: PETHREAD = core::ptr::null_mut();

    /* Allow only CSRSS to perform this operation */
    if PsGetCurrentProcess() as PVOID != gpepCSRSS as PVOID {
        return STATUS_ACCESS_DENIED;
    }

    UserEnterExclusive();

    'quit: {
        /* Get the Thread */
        Status = ObReferenceObjectByHandle(
            ThreadHandle,
            THREAD_SET_INFORMATION,
            *PsThreadType as _,
            1i32 as i8,
            core::ptr::addr_of_mut!(Thread) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        match ThreadInformationClass {
            _ if ThreadInformationClass == UserThreadInitiateShutdown => {
                let mut CapturedFlags: ULONG = 0;

                if ThreadInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                    Status = STATUS_INFO_LENGTH_MISMATCH;
                } else {
                    // The C probes ThreadInformation for WRITE inside a _SEH2_TRY
                    // before reading it, and wraps the write-back in a second one.
                    // Both were dropped, so the caller's raw pointer was dereferenced
                    // twice unchecked. One probe for write covers both accesses, which
                    // is exactly what the C's ProbeForWrite was doing.
                    //
                    // Severity, stated honestly: the CSRSS check at the top of this
                    // function means an arbitrary process cannot reach here. This
                    // hardens the kernel against a compromised csrss; it does not
                    // close an unprivileged path the way the NtGdi* ones do.
                    Status = SafeProbeForWrite(
                        ThreadInformation,
                        core::mem::size_of::<ULONG>() as SIZE_T,
                        core::mem::align_of::<ULONG>() as ULONG,
                    );
                    if NT_SUCCESS(Status) {
                        // Probe is validate-only; the C had probe, load and store in
                        // one _SEH2_TRY.
                        if !crate::copy::capture_from_user(
                            core::ptr::addr_of_mut!(CapturedFlags) as *mut core::ffi::c_void,
                            ThreadInformation as *const core::ffi::c_void,
                            core::mem::size_of::<ULONG>(),
                        ) {
                            Status = STATUS_ACCESS_VIOLATION;
                        } else {
                            Status =
                                UserInitiateShutdown(Thread, core::ptr::addr_of_mut!(CapturedFlags));

                            if !crate::copy::publish_to_user(
                                ThreadInformation as *mut core::ffi::c_void,
                                core::ptr::addr_of!(CapturedFlags) as *const core::ffi::c_void,
                                core::mem::size_of::<ULONG>(),
                            ) {
                                Status = STATUS_ACCESS_VIOLATION;
                            }
                        }
                    }
                }
            }

            _ if ThreadInformationClass == UserThreadEndShutdown => {
                if ThreadInformationLength != core::mem::size_of::<NTSTATUS>() as ULONG {
                    Status = STATUS_INFO_LENGTH_MISMATCH;
                } else {
                    // C: ProbeForRead(ThreadInformation, sizeof(ShutdownStatus),
                    // __alignof(ShutdownStatus)) inside _SEH2_TRY — dropped.
                    Status = SafeProbeForRead(
                        ThreadInformation,
                        core::mem::size_of::<NTSTATUS>() as SIZE_T,
                        core::mem::align_of::<NTSTATUS>() as ULONG,
                    );
                    if NT_SUCCESS(Status) {
                        // Probe is validate-only; the C had probe AND load in one
                        // _SEH2_TRY.
                        let mut ShutdownStatus: NTSTATUS = 0;
                        if crate::copy::capture_from_user(
                            core::ptr::addr_of_mut!(ShutdownStatus) as *mut core::ffi::c_void,
                            ThreadInformation as *const core::ffi::c_void,
                            core::mem::size_of::<NTSTATUS>(),
                        ) {
                            Status = UserEndShutdown(Thread, ShutdownStatus);
                        } else {
                            Status = STATUS_ACCESS_VIOLATION;
                        }
                    }
                }
            }

            _ if ThreadInformationClass == UserThreadCsrApiPort => {
                if ThreadInformationLength != core::mem::size_of::<HANDLE>() as ULONG {
                    Status = STATUS_INFO_LENGTH_MISMATCH;
                } else {
                    // C: ProbeForRead(ThreadInformation, sizeof(CsrPortHandle),
                    // __alignof(CsrPortHandle)) inside _SEH2_TRY — dropped.
                    Status = SafeProbeForRead(
                        ThreadInformation,
                        core::mem::size_of::<HANDLE>() as SIZE_T,
                        core::mem::align_of::<HANDLE>() as ULONG,
                    );
                    if NT_SUCCESS(Status) {
                        // Probe is validate-only; the C had probe AND load in one
                        // _SEH2_TRY.
                        let mut CsrPortHandle: HANDLE = core::ptr::null_mut();
                        if crate::copy::capture_from_user(
                            core::ptr::addr_of_mut!(CsrPortHandle) as *mut core::ffi::c_void,
                            ThreadInformation as *const core::ffi::c_void,
                            core::mem::size_of::<HANDLE>(),
                        ) {
                            Status = InitCsrApiPort(CsrPortHandle);
                        } else {
                            Status = STATUS_ACCESS_VIOLATION;
                        }
                    }
                }
            }

            _ if ThreadInformationClass == UserThreadUseActiveDesktop => {
                if Thread as PVOID != PsGetCurrentThread() as PVOID {
                    Status = STATUS_NOT_IMPLEMENTED;
                } else {
                    let hdesk: HDESK = IntGetDesktopObjectHandle(gpdeskInputDesktop);
                    IntSetThreadDesktop(hdesk, 0);
                }
            }

            _ if ThreadInformationClass == UserThreadRestoreDesktop => {
                if Thread as PVOID != PsGetCurrentThread() as PVOID {
                    Status = STATUS_NOT_IMPLEMENTED;
                } else {
                    IntSetThreadDesktop(core::ptr::null_mut(), 0);
                }
            }

            _ => {
                Status = STATUS_NOT_IMPLEMENTED;
            }
        }

        ObDereferenceObject(Thread as PVOID);
    }

    UserLeave();
    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSoundSentry() -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserTestForInteractiveUser(
    dwUnknown1: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRemoteConnect(
    dwUnknown1: DWORD,
    dwUnknown2: DWORD,
    dwUnknown3: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRemoteRedrawRectangle(
    dwUnknown1: DWORD,
    dwUnknown2: DWORD,
    dwUnknown3: DWORD,
    dwUnknown4: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRemoteRedrawScreen() -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRemoteStopScreenUpdates() -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCtxDisplayIOCtl(
    dwUnknown1: DWORD,
    dwUnknown2: DWORD,
    dwUnknown3: DWORD,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserLockWindowUpdate(hWnd: HWND) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserQuerySendMessage(Unknown0: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtUserAddClipboardFormatListener(hwnd: HWND) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtUserRemoveClipboardFormatListener(
    hwnd: HWND,
) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtUserGetUpdatedClipboardFormats(
    lpuiFormats: PUINT,
    cFormats: UINT,
    pcFormatsOut: PUINT,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiMakeObjectXferable(
    hHandle: HANDLE,
    dwProcessId: DWORD,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiMakeObjectUnXferable(hHandle: HANDLE) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtDxEngGetRedirectionBitmap(Unknown0: DWORD) -> DWORD {
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Native User stubs
//  * FILE:             win32ss/user/ntuser/ntstubs.c
//  * PROGRAMER:        Casper S. Hornstrup (chorns@users.sourceforge.net)
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
// DBG_DEFAULT_CHANNEL(UserMisc);
//
// //
// // Works like BitBlt, https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt
// //
// BOOL
// __stdcall
// NtUserBitBltSysBmp(
//    HDC hdc,
//    INT nXDest,
//    INT nYDest,
//    INT nWidth,
//    INT nHeight,
//    INT nXSrc,
//    INT nYSrc,
//    DWORD dwRop )
// {
//    BOOL Ret = FALSE;
//    UserEnterExclusive();
//
//    Ret = NtGdiBitBlt( hdc,
//                    nXDest,
//                    nYDest,
//                    nWidth,
//                   nHeight,
//                 hSystemBM,
//                     nXSrc,
//                     nYSrc,
//                     dwRop,
//               CLR_INVALID,
//                         0);
//
//    UserLeave();
//    return Ret;
// }
//
// DWORD
// __stdcall
// NtUserDragObject(
//    HWND    hwnd1,
//    HWND    hwnd2,
//    UINT    u1,
//    DWORD   dw1,
//    HCURSOR hc1
// )
// {
//    STUB
//
//    return 0;
// }
//
// BOOL
// __stdcall
// NtUserDrawAnimatedRects(
//    HWND hwnd,
//    INT idAni,
//    RECT *lprcFrom,
//    RECT *lprcTo)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserEvent(
//    DWORD Unknown0)
// {
//    STUB
//
//    return 0;
// }
//
// BOOL
// __stdcall
// NtUserGetAltTabInfo(
//    HWND hwnd,
//    INT  iItem,
//    PALTTABINFO pati,
//    LPWSTR pszItemText,
//    UINT   cchItemText,
//    BOOL   Ansi)
// {
//    STUB
//
//    return 0;
// }
//
// NTSTATUS
// __stdcall
// NtUserInitializeClientPfnArrays(
//   PPFNCLIENT pfnClientA,
//   PPFNCLIENT pfnClientW,
//   PPFNCLIENTWORKER pfnClientWorker,
//   HINSTANCE hmodUser)
// {
//    NTSTATUS Status = STATUS_SUCCESS;
//    TRACE("Enter NtUserInitializeClientPfnArrays User32 0x%p\n", hmodUser);
//
//    if (ClientPfnInit) return Status;
//
//    UserEnterExclusive();
//
//    Status = SafeProbeForRead(pfnClientA, sizeof(PFNCLIENT), 1);
//    if (NT_SUCCESS(Status))
//        Status = SafeProbeForRead(pfnClientW, sizeof(PFNCLIENT), 1);
//    if (NT_SUCCESS(Status))
//        Status = SafeProbeForRead(pfnClientWorker, sizeof(PFNCLIENTWORKER), 1);
//    if (NT_SUCCESS(Status))
//    {
//        RtlCopyMemory(&gpsi->apfnClientA, pfnClientA, sizeof(PFNCLIENT));
//        RtlCopyMemory(&gpsi->apfnClientW, pfnClientW, sizeof(PFNCLIENT));
//        RtlCopyMemory(&gpsi->apfnClientWorker, pfnClientWorker, sizeof(PFNCLIENTWORKER));
//
//        //// FIXME: HAX! Temporary until server side is finished.
//        //// Copy the client side procs for now.
//        RtlCopyMemory(&gpsi->aStoCidPfn, pfnClientW, sizeof(gpsi->aStoCidPfn));
//
//        hModClient = hmodUser;
//        ClientPfnInit = TRUE;
//    }
//
//    if (!NT_SUCCESS(Status))
//    {
//       ERR("Failed reading Client Pfns from user space.\n");
//       SetLastNtError(Status);
//    }
//
//    UserLeave();
//    return Status;
// }
//
// DWORD
// __stdcall
// NtUserInitTask(
//    DWORD Unknown0,
//    DWORD Unknown1,
//    DWORD Unknown2,
//    DWORD Unknown3,
//    DWORD Unknown4,
//    DWORD Unknown5,
//    DWORD Unknown6,
//    DWORD Unknown7,
//    DWORD Unknown8,
//    DWORD Unknown9,
//    DWORD Unknown10,
//    DWORD Unknown11)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserMNDragLeave(VOID)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserMNDragOver(
//    DWORD Unknown0,
//    DWORD Unknown1)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserModifyUserStartupInfoFlags(
//    DWORD Unknown0,
//    DWORD Unknown1)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserQueryUserCounters(
//    DWORD Unknown0,
//    DWORD Unknown1,
//    DWORD Unknown2,
//    DWORD Unknown3,
//    DWORD Unknown4)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserRegisterTasklist(
//    DWORD Unknown0)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserSetConsoleReserveKeys(
//    DWORD Unknown0,
//    DWORD Unknown1)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserSetDbgTag(
//    DWORD Unknown0,
//    DWORD Unknown1)
// {
//    STUB;
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserSetDbgTagCount(
//     DWORD Unknown0)
// {
//     STUB;
//
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserSetRipFlags(
//    DWORD Unknown0)
// {
//    STUB;
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserDbgWin32HeapFail(
//     DWORD Unknown0,
//     DWORD Unknown1)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserDbgWin32HeapStat(
//     DWORD Unknown0,
//     DWORD Unknown1)
// {
//    STUB
//
//    return 0;
// }
//
// BOOL
// __stdcall
// NtUserSetSysColors(
//    int cElements,
//    IN CONST INT *lpaElements,
//    IN CONST COLORREF *lpaRgbValues,
//    FLONG Flags)
// {
//    DWORD Ret = TRUE;
//
//    if (cElements == 0)
//       return TRUE;
//
//    /* We need this check to prevent overflow later */
//    if ((ULONG)cElements >= 0x40000000)
//    {
//       EngSetLastError(ERROR_NOACCESS);
//       return FALSE;
//    }
//
//    UserEnterExclusive();
//
//    _SEH2_TRY
//    {
//       ProbeForRead(lpaElements, cElements * sizeof(INT), 1);
//       ProbeForRead(lpaRgbValues, cElements * sizeof(COLORREF), 1);
//
//       IntSetSysColors(cElements as _, lpaElements, lpaRgbValues);
//    }
//    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//    {
//       SetLastNtError(_SEH2_GetExceptionCode());
//       Ret = FALSE;
//    }
//    _SEH2_END;
//
//    if (Ret)
//    {
//       UserSendNotifyMessage(HWND_BROADCAST, WM_SYSCOLORCHANGE, 0, 0);
//
//       UserRedrawDesktop();
//    }
//
//    UserLeave();
//    return Ret;
// }
//
// DWORD
// __stdcall
// NtUserUpdateInstance(
//    DWORD Unknown0,
//    DWORD Unknown1,
//    DWORD Unknown2)
// {
//    STUB
//
//    return 0;
// }
//
// BOOL
// __stdcall
// NtUserUserHandleGrantAccess(
//    IN HANDLE hUserHandle,
//    IN HANDLE hJob,
//    IN BOOL bGrant)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserWaitForMsgAndEvent(
//    DWORD Unknown0)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserWin32PoolAllocationStats(
//    DWORD Unknown0,
//    DWORD Unknown1,
//    DWORD Unknown2,
//    DWORD Unknown3,
//    DWORD Unknown4,
//    DWORD Unknown5)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserYieldTask(VOID)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserGetRawInputBuffer(
//     PRAWINPUT pData,
//     PUINT pcbSize,
//     UINT cbSizeHeader)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserGetRawInputData(
//     HRAWINPUT hRawInput,
//     UINT uiCommand,
//     LPVOID pData,
//     PUINT pcbSize,
//     UINT cbSizeHeader)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserGetRawInputDeviceInfo(
//     HANDLE hDevice,
//     UINT uiCommand,
//     LPVOID pData,
//     PUINT pcbSize
// )
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserGetRawInputDeviceList(
//     PRAWINPUTDEVICELIST pRawInputDeviceList,
//     PUINT puiNumDevices,
//     UINT cbSize)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserGetRegisteredRawInputDevices(
//     PRAWINPUTDEVICE pRawInputDevices,
//     PUINT puiNumDevices,
//     UINT cbSize)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserHardErrorControl(
//     DWORD dwUnknown1,
//     DWORD dwUnknown2,
//     DWORD dwUnknown3)
// {
//     STUB;
//     return 0;
// }
//
// BOOL
// NTAPI
// NtUserNotifyProcessCreate(
//     HANDLE NewProcessId,
//     HANDLE ParentThreadId,
//     ULONG  dwUnknown,
//     ULONG  CreateFlags)
// {
//     // STUB;
//     TRACE("NtUserNotifyProcessCreate is UNIMPLEMENTED\n");
//     return FALSE;
// }
//
// NTSTATUS
// __stdcall
// NtUserProcessConnect(
//     IN  HANDLE ProcessHandle,
//     OUT PUSERCONNECT pUserConnect,
//     IN  ULONG Size)
// {
//     NTSTATUS Status;
//     PEPROCESS Process = NULL;
//     PPROCESSINFO W32Process;
//
//     TRACE("NtUserProcessConnect\n");
//
//     if (pUserConnect == NULL ||
//         Size != sizeof(*pUserConnect))
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* Get the process object the user handle was referencing */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_VM_OPERATION,
//                                        *PsProcessType,
//                                        UserMode,
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     UserEnterShared();
//
//     /* Get Win32 process information */
//     W32Process = PsGetProcessWin32Process(Process as _) as _;
//
//     if (!W32Process)
//     {
//         ERR("NtUserProcessConnect: W32Process is NULL for process %p\n", Process);
//         UserLeave();
//         ObDereferenceObject(Process);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     Status = SafeProbeForWrite(pUserConnect, sizeof(*pUserConnect), sizeof(PVOID));
//     if (NT_SUCCESS(Status))
//     {
//                 UINT i;
//
//                 // FIXME: Check that pUserConnect->ulVersion == USER_VERSION;
//                 // FIXME: Check the value of pUserConnect->dwDispatchCount.
//
//
//
//                 // FIXME: Instead of assuming that the mapping of the heap desktop
//                 // also holds there, we **MUST** create and map instead the shared
//                 // section! Its client base must be stored in W32Process->pClientBase.
//                 // What is currently done (ReactOS-specific only), is that within the
//                 // IntUserHeapCommitRoutine()/MapGlobalUserHeap() routines we assume
//                 // it's going to be also called early, so that we manually add a very
//                 // first memory mapping that corresponds to the "global user heap",
//                 // and that we use instead of a actual win32 "shared USER section"
//                 // (see slide 29 of https://paper.bobylive.com/Meeting_Papers/BlackHat/USA-2011/BH_US_11_Mandt_win32k_Slides.pdf )
//
//                 pUserConnect->siClient.ulSharedDelta =
//                     (ULONG_PTR)W32Process->HeapMappings.KernelMapping -
//                     (ULONG_PTR)W32Process->HeapMappings.UserMapping;
//
//         #define SERVER_TO_CLIENT(ptr) \
//             ((PVOID)((ULONG_PTR)ptr - pUserConnect->siClient.ulSharedDelta))
//
//                 ASSERT(gpsi);
//                 ASSERT(gHandleTable);
//
//                 pUserConnect->siClient.psi       = SERVER_TO_CLIENT(gpsi);
//                 pUserConnect->siClient.aheList   = SERVER_TO_CLIENT(gHandleTable);
//                 pUserConnect->siClient.pDispInfo = NULL;
//
//                 // NOTE: kernel server should also have a SHAREDINFO gSharedInfo;
//                 // FIXME: These USER window-proc data should be used somehow!
//
//                 pUserConnect->siClient.DefWindowMsgs.maxMsgs     = 0;
//                 pUserConnect->siClient.DefWindowMsgs.abMsgs      = NULL;
//                 pUserConnect->siClient.DefWindowSpecMsgs.maxMsgs = 0;
//                 pUserConnect->siClient.DefWindowSpecMsgs.abMsgs  = NULL;
//
//                 for (i = 0; i < ARRAYSIZE(pUserConnect->siClient.awmControl); ++i)
//                 {
//                     pUserConnect->siClient.awmControl[i].maxMsgs = 0;
//                     pUserConnect->siClient.awmControl[i].abMsgs  = NULL;
//                 }
//         #undef SERVER_TO_CLIENT
//     }
//
//     if (!NT_SUCCESS(Status))
//         SetLastNtError(Status);
//
//     UserLeave();
//
//     /* Dereference the process object */
//     ObDereferenceObject(Process);
//
//     return Status;
// }
//
// NTSTATUS
// __stdcall
// NtUserQueryInformationThread(IN HANDLE ThreadHandle,
//                              IN USERTHREADINFOCLASS ThreadInformationClass,
//                              OUT PVOID ThreadInformation,
//                              IN ULONG ThreadInformationLength)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PETHREAD Thread;
//
//     /* Allow only CSRSS to perform this operation */
//     if (PsGetCurrentProcess() != gpepCSRSS)
//         return STATUS_ACCESS_DENIED;
//
//     UserEnterExclusive();
//
//     /* Get the Thread */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_QUERY_INFORMATION,
//                                        *PsThreadType,
//                                        UserMode,
//                                        (PVOID)&Thread,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) goto Quit;
//
//     switch (ThreadInformationClass)
//     {
//         default:
//         {
//             STUB;
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//         }
//     }
//
//     ObDereferenceObject(Thread);
//
// Quit:
//     UserLeave();
//     return Status;
// }
//
// BOOL
// __stdcall
// NtUserRealInternalGetMessage(
//     LPMSG lpMsg,
//     HWND hWnd,
//     UINT wMsgFilterMin,
//     UINT wMsgFilterMax,
//     UINT wRemoveMsg,
//     BOOL bGMSG)
// {
//     STUB;
//     return 0;
// }
//
// BOOL
// __stdcall
// NtUserRealWaitMessageEx(
//     DWORD dwWakeMask,
//     UINT uTimeout)
// {
//     STUB;
//     return 0;
// }
//
// BOOL
// __stdcall
// NtUserRegisterRawInputDevices(
//     IN PCRAWINPUTDEVICE pRawInputDevices,
//     IN UINT uiNumDevices,
//     IN UINT cbSize)
// {
//     STUB;
//     return 0;
// }
//
// DWORD __stdcall
// NtUserResolveDesktopForWOW(DWORD Unknown0)
// {
//     STUB
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserSetInformationProcess(
//     DWORD dwUnknown1,
//     DWORD dwUnknown2,
//     DWORD dwUnknown3,
//     DWORD dwUnknown4)
// {
//     STUB;
//     return 0;
// }
//
// HDESK FASTCALL
// IntGetDesktopObjectHandle(PDESKTOP DesktopObject);
//
// NTSTATUS
// __stdcall
// NtUserSetInformationThread(IN HANDLE ThreadHandle,
//                            IN USERTHREADINFOCLASS ThreadInformationClass,
//                            IN PVOID ThreadInformation,
//                            IN ULONG ThreadInformationLength)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PETHREAD Thread;
//
//     /* Allow only CSRSS to perform this operation */
//     if (PsGetCurrentProcess() != gpepCSRSS)
//         return STATUS_ACCESS_DENIED;
//
//     UserEnterExclusive();
//
//     /* Get the Thread */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_SET_INFORMATION,
//                                        *PsThreadType,
//                                        UserMode,
//                                        (PVOID)&Thread,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) goto Quit;
//
//     switch (ThreadInformationClass)
//     {
//         case UserThreadInitiateShutdown:
//         {
//             ULONG CapturedFlags = 0;
//
//             TRACE("Shutdown initiated\n");
//
//             if (ThreadInformationLength != sizeof(CapturedFlags))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Capture the caller value */
//             Status = STATUS_SUCCESS;
//             _SEH2_TRY
//             {
//                 ProbeForWrite(ThreadInformation, sizeof(CapturedFlags), __alignof(CapturedFlags));
//                 CapturedFlags = *(PULONG)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             Status = UserInitiateShutdown(Thread, &CapturedFlags);
//
//             /* Return the modified value to the caller */
//             _SEH2_TRY
//             {
//                 *(PULONG)ThreadInformation = CapturedFlags;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             break;
//         }
//
//         case UserThreadEndShutdown:
//         {
//             NTSTATUS ShutdownStatus;
//
//             TRACE("Shutdown ended\n");
//
//             if (ThreadInformationLength != sizeof(ShutdownStatus))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Capture the caller value */
//             Status = STATUS_SUCCESS;
//             _SEH2_TRY
//             {
//                 ProbeForRead(ThreadInformation, sizeof(ShutdownStatus), __alignof(ShutdownStatus));
//                 ShutdownStatus = *(NTSTATUS*)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             Status = UserEndShutdown(Thread, ShutdownStatus);
//             break;
//         }
//
//         case UserThreadCsrApiPort:
//         {
//             HANDLE CsrPortHandle;
//
//
//             TRACE("Set CSR API Port for Win32k\n");
//             if (ThreadInformationLength != sizeof(CsrPortHandle))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Capture the caller value */
//             Status = STATUS_SUCCESS;
//             _SEH2_TRY
//             {
//                 ProbeForRead(ThreadInformation, sizeof(CsrPortHandle), __alignof(CsrPortHandle));
//                 CsrPortHandle = *(PHANDLE)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             ERR("[WIN32K] UserThreadCsrApiPort: captured CsrPortHandle=%p (ThreadInformation=%p, len=%lu)\n",
//                     CsrPortHandle, ThreadInformation, ThreadInformationLength);
//             Status = InitCsrApiPort(CsrPortHandle);
//             break;
//         }
//
//         case UserThreadUseActiveDesktop:
//         {
//             HDESK hdesk;
//
//             if (Thread != PsGetCurrentThread())
//             {
//                 Status = STATUS_NOT_IMPLEMENTED;
//                 break;
//             }
//
//             hdesk = IntGetDesktopObjectHandle(gpdeskInputDesktop);
//             IntSetThreadDesktop(hdesk, FALSE);
//
//             break;
//         }
//         case UserThreadRestoreDesktop:
//         {
//             if (Thread != PsGetCurrentThread())
//             {
//                 Status = STATUS_NOT_IMPLEMENTED;
//                 break;
//             }
//
//             IntSetThreadDesktop(NULL, FALSE);
//             break;
//         }
//         default:
//         {
//             STUB;
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//         }
//     }
//
//     ObDereferenceObject(Thread);
//
// Quit:
//     UserLeave();
//     return Status;
// }
//
// BOOL
// __stdcall
// NtUserSoundSentry(VOID)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserTestForInteractiveUser(
//     DWORD dwUnknown1)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserRemoteConnect(
//     DWORD dwUnknown1,
//     DWORD dwUnknown2,
//     DWORD dwUnknown3)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserRemoteRedrawRectangle(
//     DWORD dwUnknown1,
//     DWORD dwUnknown2,
//     DWORD dwUnknown3,
//     DWORD dwUnknown4)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserRemoteRedrawScreen(VOID)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserRemoteStopScreenUpdates(VOID)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserCtxDisplayIOCtl(
//     DWORD dwUnknown1,
//     DWORD dwUnknown2,
//     DWORD dwUnknown3)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL __stdcall
// NtUserLockWindowUpdate(HWND hWnd)
// {
//     STUB;
//     return FALSE;
// }
//
// DWORD __stdcall
// NtUserQuerySendMessage(DWORD Unknown0)
// {
//     STUB;
//
//     return 0;
// }
//
// BOOL __stdcall NtUserAddClipboardFormatListener(
//     HWND hwnd
// )
// {
//     STUB;
//     return FALSE;
// }
//
// BOOL __stdcall NtUserRemoveClipboardFormatListener(
//     HWND hwnd
// )
// {
//     STUB;
//     return FALSE;
// }
//
// BOOL __stdcall NtUserGetUpdatedClipboardFormats(
//     PUINT lpuiFormats,
//     UINT cFormats,
//     PUINT pcFormatsOut
// )
// {
//     STUB;
//     return FALSE;
// }
//
// // Yes, I know, these do not belong here, just tell me where to put them
// BOOL
// __stdcall
// NtGdiMakeObjectXferable(
//      HANDLE hHandle,
//      DWORD dwProcessId)
// {
//     STUB;
//     return 0;
// }
//
// BOOL
// __stdcall
// NtGdiMakeObjectUnXferable(
//      HANDLE hHandle)
// {
//     STUB;
//     return 0;
// }
//
// DWORD
// __stdcall
// NtDxEngGetRedirectionBitmap(
//     DWORD Unknown0)
// {
//     STUB;
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
