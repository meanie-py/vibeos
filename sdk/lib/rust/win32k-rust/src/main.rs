#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, ExInitializeResourceLite, KeAddSystemServiceTable, KeBugCheck, KeInitializeEvent, KeSetEvent, KeSetKernelStackSwapEnable, MmPageEntireDriver, ObCloseHandle, ObReferenceObjectByHandle, PsEstablishWin32Callouts, RtlAllocateHeap, PsGetProcessExitStatus, PsGetProcessId, PsGetProcessWin32Process, PsGetThreadWin32Thread, PsSetProcessWin32Process, PsSetThreadWin32Thread, RtlInitUnicodeString, ZwClose, ZwCreateEvent};
use ntkrnl_sys::inline::common::PopEntryList;
use ntkrnl_sys::inline::ex::ExInitializeFastMutex;
// NtCurrentTeb is a FORCEINLINE per-arch TIB read (no kernel symbol) — use the intrinsic.
use ntkrnl_sys::inline::ke::NtCurrentTeb;
use ntkrnl_sys::ob::{ObReferenceObject, ObDereferenceObject};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
#[allow(unused_imports)]
use core::ffi::c_void;

// MsqCleanupThreadMsgs / MsqDestroyMessageQueue / co_MsqInsertMouseMessage — msgqueue.rs
// FASTCALL exports (i386 fastcall, cdecl else).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn MsqCleanupThreadMsgs(pti: PTHREADINFO);
    fn MsqDestroyMessageQueue(pti: PTHREADINFO);
    fn co_MsqInsertMouseMessage(msg: *mut MSG, flags: u32, extra: usize, hook: u32);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn MsqCleanupThreadMsgs(pti: PTHREADINFO);
    fn MsqDestroyMessageQueue(pti: PTHREADINFO);
    fn co_MsqInsertMouseMessage(msg: *mut MSG, flags: u32, extra: usize, hook: u32);
}
extern "C" {
    static Win32kSSDT: [usize; 0];
    static Win32kSSPT: [u8; 0];
    static Win32kNumberOfSysCalls: ULONG;
}
// FreeFontSupport is FASTCALL (text.h: VOID FASTCALL FreeFontSupport(VOID)) —
// the freetype.rs export is `@FreeFontSupport@0`, so declare it fastcall.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn FreeFontSupport();
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn FreeFontSupport();
}

// UserHeapAlloc — usrheap.h static __inline (no kernel symbol):
// `RtlAllocateHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, Bytes)`.
#[inline(always)]
unsafe fn UserHeapAlloc(size: usize) -> PVOID {
    RtlAllocateHeap(GlobalUserHeap as HANDLE, HEAP_NO_SERIALIZE, size as SIZE_T)
}

// minwindef.h macros (preprocessor in C, local fns here).
#[inline(always)]
fn MAKELONG(a: u32, b: u32) -> LONG { ((a & 0xffff) | ((b & 0xffff) << 16)) as LONG }
#[inline(always)]
fn MAKELPARAM(low: u32, high: u32) -> LPARAM { MAKELONG(low, high) as LPARAM }

// win32k-internal (cdecl `_Name`) exports referenced by main.
extern "C" {
    fn UserDestroyObjectsForOwner(HandleTable: *mut core::ffi::c_void, Owner: *mut core::ffi::c_void) -> u8;
    fn GdiProcessDestroy(Process: PEPROCESS) -> NTSTATUS;
    fn W32kGetDefaultKeyLayout() -> PKL;
    fn IntSetThreadDesktop(hDesktop: HDESK, FreeOnFailure: u32) -> BOOL;
    fn RtlGetExpWinVer(BaseAddress: *mut core::ffi::c_void) -> u32;
    fn UserDisplayNotifyShutdown(ppiCurrent: PPROCESSINFO);
    fn UserCreateHeap(SectionObject: *mut *mut core::ffi::c_void, SystemBase: *mut PVOID, HeapSize: usize) -> *mut core::ffi::c_void;
}

// bindgen flattened the embedded W32PROCESS/W32THREAD base of PROCESSINFO/THREADINFO
// to opaque padding; the base fields (peProcess, W32PF_flags, W32Pid, InputIdleEvent,
// RefCount / pEThread) sit at offset 0. Reinterpret to reach them.
#[inline(always)]
unsafe fn w32proc(ppi: PPROCESSINFO) -> *mut _W32PROCESS { ppi as *mut _W32PROCESS }
#[inline(always)]
unsafe fn w32thr(pti: PTHREADINFO) -> *mut _W32THREAD { pti as *mut _W32THREAD }

// gpsi->ptCursor lives in the opaque PERUSERSERVERINFO tail of tagSERVERINFO.
#[inline(always)]
unsafe fn gpsi_pusi() -> *mut _PERUSERSERVERINFO {
    core::ptr::addr_of_mut!((*gpsi).__bindgen_padding_0) as *mut _PERUSERSERVERINFO
}

// IntReference{Process,Thread}Info are InterlockedIncrement macros on the base RefCount;
// IntDereferenceThreadInfo decrements + frees. Reimplement faithfully.
#[inline(always)]
unsafe fn IntReferenceProcessInfo(ppi: PPROCESSINFO) {
    *core::ptr::addr_of_mut!((*w32proc(ppi)).RefCount) += 1;
}
// IntDereferenceProcessInfo (win32.h macro): decrement RefCount; free at 0.
#[inline(always)]
unsafe fn IntDereferenceProcessInfo(ppi: PPROCESSINFO) {
    let refcount = core::ptr::addr_of_mut!((*w32proc(ppi)).RefCount);
    *refcount = (*refcount).wrapping_sub(1);
    if *refcount == 0 {
        UserDeleteW32Process(ppi);
    }
}
#[inline(always)]
unsafe fn IntReferenceThreadInfo(pti: PTHREADINFO) {
    *core::ptr::addr_of_mut!((*w32thr(pti)).RefCount) += 1;
}
#[inline(always)]
unsafe fn IntDereferenceThreadInfo(pti: PTHREADINFO) {
    let refcount = core::ptr::addr_of_mut!((*w32thr(pti)).RefCount);
    if (*refcount).wrapping_sub(1) == 0 {
        // matches C: if RefCount hits 0, the thread info is freed by UserDeleteW32Thread
        *refcount = 0;
        UserDeleteW32Thread(pti);
    } else {
        *refcount -= 1;
    }
}

// HandleToUlong / EngGetTickCount32 / InitializeListHead / RtlZeroMemory / IS_IMM_MODE macros.
#[inline(always)]
fn HandleToUlong(h: HANDLE) -> u32 { h as usize as u32 }
#[inline(always)]
unsafe fn EngGetTickCount32() -> ULONG { EngGetTickCount() as ULONG }
#[inline(always)]
unsafe fn InitializeListHead(head: *mut LIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}
#[inline(always)]
unsafe fn RtlZeroMemory(dst: *mut core::ffi::c_void, n: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, n);
}
#[inline(always)]
fn IS_IMM_MODE() -> bool { false }

// EngGetTickCount is DEFINED extern "stdcall" (engmisc.rs, i386) — a flat
// `extern "C"` decl is a calling-convention mismatch. With 0 args it doesn't
// ud2 (stack stays balanced) so scan_ud2 misses it, but LLVM still treats the
// ABI mismatch as UB: the statement `pcti->timeLastRead = EngGetTickCount32()`
// became provable-UB → the whole InitThreadCallback success path was DCE'd.
extern "system" { fn EngGetTickCount() -> ULONGLONG; }

// tags.h pool tags (C multichar 'ipsU' etc. → LE DWORD = reversed bytes).
const USERTAG_PROCESSINFO: u32 = u32::from_le_bytes(*b"Uspi");
const USERTAG_EVENT: u32 = u32::from_le_bytes(*b"Usev");
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl");
const TAG_STRING: u32 = u32::from_be_bytes(*b" RTS"); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')

// winuser/ketypes constants not in the win32k bindgen glob.
const EVENT_ALL_ACCESS: u32 = 0x1F0003;
const STARTF_SHELLPRIVATE: u32 = 0x00000400;
const IO_NO_INCREMENT: i32 = 0;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut hModuleWin: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut gpsi: PSERVERINFO = core::ptr::null_mut();

#[no_mangle]
pub static mut ppiScrnSaver: PPROCESSINFO = core::ptr::null_mut();

#[no_mangle]
pub static mut gppiList: PPROCESSINFO = core::ptr::null_mut();

unsafe extern "system" fn DbgPreServiceHook(ulSyscallId: ULONG, pulArguments: PULONG_PTR) {
    GdiDbgPreServiceHook(ulSyscallId, pulArguments);
    UserDbgPreServiceHook(ulSyscallId, pulArguments);
}

unsafe extern "system" fn DbgPostServiceHook(ulSyscallId: ULONG, mut ulResult: ULONG_PTR) -> ULONG_PTR {
    ulResult = GdiDbgPostServiceHook(ulSyscallId, ulResult);
    ulResult = UserDbgPostServiceHook(ulSyscallId, ulResult);
    ulResult
}

#[no_mangle]
pub unsafe extern "system" fn AllocW32Process(
    Process: PEPROCESS,
    W32Process: *mut PPROCESSINFO,
) -> NTSTATUS {
    let mut ppiCurrent: PPROCESSINFO;

    ppiCurrent = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;
    if !ppiCurrent.is_null() {
        return STATUS_SUCCESS;
    }

    ppiCurrent = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<PROCESSINFO>() as _,
        USERTAG_PROCESSINFO,
    ) as PPROCESSINFO;
    if ppiCurrent.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlZeroMemory(ppiCurrent as *mut _, core::mem::size_of::<PROCESSINFO>());

    PsSetProcessWin32Process(Process as _, ppiCurrent as *mut _, core::ptr::null_mut());
    IntReferenceProcessInfo(ppiCurrent);

    // captures(none) output pointer — volatile store so it survives DCE (see AllocW32Thread).
    core::ptr::write_volatile(W32Process, ppiCurrent);
    STATUS_SUCCESS
}

#[no_mangle]
unsafe extern "C" fn UserDeleteW32Process(ppiCurrent: PPROCESSINFO) {
    if !(*w32proc(ppiCurrent)).InputIdleEvent.is_null() {
        ExFreePoolWithTag((*w32proc(ppiCurrent)).InputIdleEvent as *mut _, USERTAG_EVENT);
    }

    if !(*ppiCurrent).rpdeskStartup.is_null() {
        ObDereferenceObject((*ppiCurrent).rpdeskStartup as *mut _);
    }

    ExFreePoolWithTag(ppiCurrent as *mut _, USERTAG_PROCESSINFO);
}

unsafe fn UserProcessCreate(Process: PEPROCESS) -> NTSTATUS {
    let ppiCurrent: PPROCESSINFO = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;

    InitializeListHead(core::ptr::addr_of_mut!((*ppiCurrent).DriverObjListHead));
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*ppiCurrent).DriverObjListLock) as _);

    {
        let Event: PKEVENT;

        Event = ExAllocatePoolWithTag(NonPagedPool,
                                      core::mem::size_of::<_KEVENT>() as _,
                                      USERTAG_EVENT) as PKEVENT;
        if !Event.is_null() {
            KeInitializeEvent(Event as _,
                              _EVENT_TYPE_SynchronizationEvent,
                              0u8);
        } else {
            KeBugCheck(0); // nolint: bugcheck-code (faithful: upstream win32k main.c says KeBugCheck(0))
        }

        (*w32proc(ppiCurrent)).InputIdleEvent = Event;
        KeInitializeEvent((*w32proc(ppiCurrent)).InputIdleEvent as _, _EVENT_TYPE_NotificationEvent, 0u8);
    }

    (*w32proc(ppiCurrent)).peProcess = Process;
    (*w32proc(ppiCurrent)).W32Pid = HandleToUlong(PsGetProcessId(Process as _));

    (*w32proc(ppiCurrent)).W32PF_flags |= W32PF_PROCESSCONNECTED;
    if !(*(*Process).Peb).ProcessParameters.is_null() &&
        ((*(*(*Process).Peb).ProcessParameters).WindowFlags & STARTF_SCREENSAVER) != 0 {
        ppiScrnSaver = ppiCurrent;
        (*w32proc(ppiCurrent)).W32PF_flags |= W32PF_SCREENSAVER;
    }

    (*w32proc(ppiCurrent)).W32PF_flags |= W32PF_ALLOWFOREGROUNDACTIVATE;

    STATUS_SUCCESS
}

unsafe fn UserProcessDestroy(Process: PEPROCESS) -> NTSTATUS {
    let ppiCurrent: PPROCESSINFO = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;

    if ppiScrnSaver == ppiCurrent {
        ppiScrnSaver = core::ptr::null_mut();
    }

    crate::pfnclient::forget(ppiCurrent);

    IntFreeImeHotKeys();

    if !gpwlCache.is_null() {
        ExFreePoolWithTag(gpwlCache as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
        gpwlCache = core::ptr::null_mut();
    }

    /* Destroy user objects */
    UserDestroyObjectsForOwner(gHandleTable as *mut core::ffi::c_void, ppiCurrent as *mut core::ffi::c_void);

    /* Remove it from the list of GUI apps */
    co_IntGraphicsCheck((0u8) as _);

    /*
     * Deregister logon application automatically
     */
    if gpidLogon == (*(*w32proc(ppiCurrent)).peProcess).UniqueProcessId {
        gpidLogon = core::ptr::null_mut();
    }

    /* Close the current window station */
    UserSetProcessWindowStation(core::ptr::null_mut());

    if gppiInputProvider == ppiCurrent {
        gppiInputProvider = core::ptr::null_mut();
    }

    if !(*ppiCurrent).hdeskStartup.is_null() {
        ZwClose((*ppiCurrent).hdeskStartup as HANDLE);
        (*ppiCurrent).hdeskStartup = core::ptr::null_mut();
    }

    /* Clean up the process icon cache */
    IntCleanupCurIconCache(ppiCurrent);

    STATUS_SUCCESS
}

unsafe fn InitProcessCallback(Process: PEPROCESS) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ppiCurrent: PPROCESSINFO;
    let mut KernelMapping: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut UserMapping: *mut core::ffi::c_void = core::ptr::null_mut();

    /* We might be called with an already allocated win32 process */
    ppiCurrent = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;
    if !ppiCurrent.is_null() {
        /* There is no more to do for us (this is a success code!) */
        return STATUS_ALREADY_WIN32;
    }

    /* Allocate a new Win32 process info */
    Status = AllocW32Process(Process, &mut ppiCurrent);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'block: {
        /* Map the global user heap into the process */
        Status = MapGlobalUserHeap(Process, &mut KernelMapping, &mut UserMapping);
        if !NT_SUCCESS(Status) {
            break 'block;
        }

        /* Initialize USER process info */
        Status = UserProcessCreate(Process);
        if !NT_SUCCESS(Status) {
            break 'block;
        }

        /* Initialize GDI process info */
        Status = GdiProcessCreate(Process);
        if !NT_SUCCESS(Status) {
            break 'block;
        }

        /* Add the process to the global list */
        (*ppiCurrent).ppiNext = gppiList;
        gppiList = ppiCurrent;

        /* If we belong to a job that restricts the UI, join it now. A process is normally
           assigned to its job while still suspended, before it ever calls into win32k, so
           the kernel could not hand us over back then (upstream b272427baa3). */
        Status = crate::job::IntJobConnectProcess(ppiCurrent);
        if !NT_SUCCESS(Status) {
            ERR!("IntJobConnectProcess failed, Status 0x%08lx\n", Status);
            break 'block;
        }

        return STATUS_SUCCESS;
    }

    ExitProcessCallback(Process);
    Status
}

const STATUS_ALREADY_WIN32: NTSTATUS = 0x4000001B;

unsafe fn ExitProcessCallback(Process: PEPROCESS) -> NTSTATUS {
    let ppiCurrent: PPROCESSINFO = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;

    (*w32proc(ppiCurrent)).W32PF_flags |= W32PF_TERMINATED;

    /* Leave the job we were restricted by, if any */
    crate::job::IntJobDisconnectProcess(ppiCurrent);

    /* Remove it from the list */
    let mut pppi: *mut PPROCESSINFO = core::ptr::addr_of_mut!(gppiList);
    while !(*pppi).is_null() && *pppi != ppiCurrent {
        pppi = core::ptr::addr_of_mut!((**pppi).ppiNext);
    }
    *pppi = (*ppiCurrent).ppiNext;

    /* Cleanup GDI info */
    GdiProcessDestroy(Process);

    /* Cleanup USER info */
    UserProcessDestroy(Process);

    /* The process is dying */
    PsSetProcessWin32Process(Process as _, core::ptr::null_mut(), ppiCurrent as *mut _);
    (*w32proc(ppiCurrent)).peProcess = core::ptr::null_mut();

    /* Finally, dereference */
    IntDereferenceProcessInfo(ppiCurrent);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn Win32kProcessCallback(
    Process: PEPROCESS,
    Initialize: BOOLEAN,
) -> NTSTATUS {
    let Status: NTSTATUS;

    UserEnterExclusive();

    if Initialize != 0 {
        Status = InitProcessCallback(Process);
    } else {
        Status = ExitProcessCallback(Process);
    }

    UserLeave();

    Status
}

unsafe fn AllocW32Thread(Thread: PETHREAD, W32Thread: *mut PTHREADINFO) -> NTSTATUS {
    let mut ptiCurrent: PTHREADINFO;

    ptiCurrent = PsGetThreadWin32Thread(Thread as _) as PTHREADINFO;

    ptiCurrent = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<THREADINFO>() as _,
        USERTAG_THREADINFO,
    ) as PTHREADINFO;
    if ptiCurrent.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlZeroMemory(ptiCurrent as _, core::mem::size_of::<THREADINFO>());

    PsSetThreadWin32Thread(Thread as _, ptiCurrent as _, core::ptr::null_mut());
    ObReferenceObject(Thread as _);
    IntReferenceThreadInfo(ptiCurrent);

    // captures(none) output pointer (lint_kernel.py-flagged): volatile store so
    // LLVM can't DCE the write and leave the caller's local NULL.
    core::ptr::write_volatile(W32Thread, ptiCurrent);
    STATUS_SUCCESS
}

#[no_mangle]
unsafe extern "C" fn UserDeleteW32Thread(pti: PTHREADINFO) {
    let ppi: PPROCESSINFO = (*pti).ppi;

    if !(*pti).MessageQueue.is_null() {
        MsqDestroyMessageQueue(pti);
    }

    MsqCleanupThreadMsgs(pti);

    ObDereferenceObject((*w32thr(pti)).pEThread as *mut core::ffi::c_void);

    ExFreePoolWithTag(pti as *mut core::ffi::c_void, USERTAG_THREADINFO);

    IntDereferenceProcessInfo(ppi);

    {
        let mut msg: MSG = core::mem::zeroed();
        msg.message = WM_MOUSEMOVE;
        msg.wParam = UserGetMouseButtonsState() as _;
        msg.lParam = MAKELPARAM((*gpsi_pusi()).ptCursor.x as u32, (*gpsi_pusi()).ptCursor.y as u32) as _;
        msg.pt = (*gpsi_pusi()).ptCursor;
        co_MsqInsertMouseMessage(core::ptr::addr_of_mut!(msg), 0, 0, (1u8) as _);
    }
}

const USERTAG_THREADINFO: u32 = u32::from_le_bytes(*b"Usti");

unsafe fn UserThreadCreate(Thread: PETHREAD) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn UserThreadDestroy(Thread: PETHREAD) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn InitThreadCallback(Thread: PETHREAD) -> NTSTATUS {
    let Process: PEPROCESS;
    let pci: PCLIENTINFO;
    let mut ptiCurrent: PTHREADINFO = core::ptr::null_mut();
    let mut i: i32;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let pTeb: PTEB;
    let ProcessParams: PRTL_USER_PROCESS_PARAMETERS;
    let pDefKL: PKL;
    let bFirstThread: BOOLEAN;

    Process = (*Thread).ThreadsProcess;

    pTeb = NtCurrentTeb() as _;

    ProcessParams = (*(*pTeb).ProcessEnvironmentBlock).ProcessParameters;


    /* Allocate a new Win32 thread info */
    Status = AllocW32Thread(Thread, &mut ptiCurrent);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Initialize the THREADINFO */
    (*w32thr(ptiCurrent)).pEThread = Thread;
    (*ptiCurrent).ppi = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;
    IntReferenceProcessInfo((*ptiCurrent).ppi);
    (*pTeb).Win32ThreadInfo = ptiCurrent as *mut core::ffi::c_void;
    (*ptiCurrent).pClientInfo = (*pTeb).Win32ClientInfo.as_mut_ptr() as PCLIENTINFO;
    (*ptiCurrent).pcti = core::ptr::addr_of_mut!((*ptiCurrent).cti);
    bFirstThread = if ((*w32proc((*ptiCurrent).ppi)).W32PF_flags & W32PF_THREADCONNECTED) == 0 { 1u8 } else { 0u8 };

    /* Mark the process as having threads */
    (*w32proc((*ptiCurrent).ppi)).W32PF_flags |= W32PF_THREADCONNECTED;

    InitializeListHead(core::ptr::addr_of_mut!((*ptiCurrent).WindowListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*ptiCurrent).W32CallbackListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*ptiCurrent).PostedMessagesListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*ptiCurrent).SentMessagesListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*ptiCurrent).PtiLink));
    i = 0;
    while i < NB_HOOKS as i32 {
        InitializeListHead(core::ptr::addr_of_mut!((*ptiCurrent).aphkStart[i as usize]));
        i += 1;
    }
    (*ptiCurrent).ptiSibling = (*(*ptiCurrent).ppi).ptiList;
    (*(*ptiCurrent).ppi).ptiList = ptiCurrent;
    (*(*ptiCurrent).ppi).cThreads += 1;

    (*ptiCurrent).hEventQueueClient = core::ptr::null_mut();
    Status = ZwCreateEvent(core::ptr::addr_of_mut!((*ptiCurrent).hEventQueueClient), EVENT_ALL_ACCESS,
                           core::ptr::null_mut(), _EVENT_TYPE_SynchronizationEvent, 0u8);
    if !NT_SUCCESS(Status) {
        Status = Status;
        ExitThreadCallback(Thread);
        return Status;
    }
    Status = ObReferenceObjectByHandle((*ptiCurrent).hEventQueueClient, 0,
                                       *ExEventObjectType as *mut _, 1i32 as i8,
                                       core::ptr::addr_of_mut!((*ptiCurrent).pEventQueueServer) as *mut *mut core::ffi::c_void, core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        ObCloseHandle((*ptiCurrent).hEventQueueClient, 1i32 as i8);
        (*ptiCurrent).hEventQueueClient = core::ptr::null_mut();
        ExitThreadCallback(Thread);
        return Status;
    }

    (*(*ptiCurrent).pcti).timeLastRead = EngGetTickCount32();

    (*ptiCurrent).MessageQueue = MsqCreateMessageQueue(ptiCurrent);
    if (*ptiCurrent).MessageQueue.is_null() {
        Status = STATUS_NO_MEMORY;
        ExitThreadCallback(Thread);
        return Status;
    }

    pDefKL = W32kGetDefaultKeyLayout();
    UserAssignmentLock(core::ptr::addr_of_mut!((*ptiCurrent).KeyboardLayout) as *mut *mut core::ffi::c_void, pDefKL as *mut core::ffi::c_void);

    (*ptiCurrent).TIF_flags &= !TIF_INCLEANUP;

    /* CSRSS threads have some special features */
    if Process == gpepCSRSS || gpepCSRSS.is_null() {
        (*ptiCurrent).TIF_flags = TIF_CSRSSTHREAD | TIF_DONTATTACHQUEUE;
    }

    /* A thread of a process in a job restricting the UI is restricted from the moment it
       appears, not from the next time the job is touched */
    if (*w32proc((*ptiCurrent).ppi)).W32PF_flags & W32PF_JOBRESTRICTED != 0 {
        (*ptiCurrent).TIF_flags |= TIF_JOBRESTRICTED as FLONG;
    }

    /* Initialize the CLIENTINFO */
    pci = (*pTeb).Win32ClientInfo.as_mut_ptr() as PCLIENTINFO;
    RtlZeroMemory(pci as *mut core::ffi::c_void, core::mem::size_of_val(&*pci));
    (*pci).ppi = (*ptiCurrent).ppi;
    (*pci).fsHooks = (*ptiCurrent).fsHooks;
    (*pci).dwTIFlags = (*ptiCurrent).TIF_flags;
    if !pDefKL.is_null() {
        (*pci).hKL = (*pDefKL).hkl;
        (*pci).CodePage = (*pDefKL).CodePage;
    }

    /* Populate dwExpWinVer */
    if !(*Process).Peb.is_null() {
        (*ptiCurrent).dwExpWinVer = RtlGetExpWinVer((*Process).SectionBaseAddress);
    } else {
        (*ptiCurrent).dwExpWinVer = WINVER_WINNT4 as u32;
    }
    (*pci).dwExpWinVer = (*ptiCurrent).dwExpWinVer;

    /* A 32-bit guest reads its CLIENTINFO from TEB32, which win32k never
     * touches otherwise — project it (no-op for native processes). */
    crate::wow64::Wow64PublishClientInfo(ptiCurrent);

    /* Need to pass the user Startup Information to the current process. */
    if !ProcessParams.is_null() {
        if (*(*ptiCurrent).ppi).usi.cb == 0 {
            if (*ProcessParams).WindowFlags != 0 {
                (*(*ptiCurrent).ppi).usi.cb = core::mem::size_of::<USERSTARTUPINFO>() as u32;
                (*(*ptiCurrent).ppi).usi.dwX = (*ProcessParams).StartingX;
                (*(*ptiCurrent).ppi).usi.dwY = (*ProcessParams).StartingY;
                (*(*ptiCurrent).ppi).usi.dwXSize = (*ProcessParams).CountX;
                (*(*ptiCurrent).ppi).usi.dwYSize = (*ProcessParams).CountY;
                (*(*ptiCurrent).ppi).usi.dwFlags = (*ProcessParams).WindowFlags;
                (*(*ptiCurrent).ppi).usi.wShowWindow = (*ProcessParams).ShowWindowFlags as u16;
            }
        }

        if bFirstThread != 0 {
            if (*ProcessParams).WindowFlags & STARTF_USEHOTKEY != 0 {
                (*(*ptiCurrent).ppi).dwHotkey = HandleToUlong((*ProcessParams).StandardInput);
            }

            if (*ProcessParams).WindowFlags & STARTF_SHELLPRIVATE != 0 {
                let hMonitor: HMONITOR = (*ProcessParams).StandardOutput as HMONITOR;
                if !hMonitor.is_null() && !UserGetMonitorObject(hMonitor).is_null() {
                    (*(*ptiCurrent).ppi).hMonitor = hMonitor;
                }
            }
        }
    }

    /*
     * Assign a default window station and desktop to the process.
     */
    if (*ptiCurrent).TIF_flags & (TIF_SYSTEMTHREAD | TIF_CSRSSTHREAD) == 0 &&
        (*(*ptiCurrent).ppi).hdeskStartup.is_null() &&
        !InputWindowStation.is_null()
    {
        let mut hWinSta: HWINSTA = core::ptr::null_mut();
        let mut hDesk: HDESK = core::ptr::null_mut();
        let mut DesktopPath: _UNICODE_STRING = core::mem::zeroed();
        let mut pdesk: PDESKTOP = core::ptr::null_mut();

        Status = STATUS_UNSUCCESSFUL;
        if !ProcessParams.is_null() && (*ProcessParams).DesktopInfo.Length > 0 {
            Status = IntSafeCopyUnicodeStringTerminateNULL(&mut DesktopPath, core::ptr::addr_of_mut!((*ProcessParams).DesktopInfo));
        }
        if !NT_SUCCESS(Status) {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(DesktopPath) as *mut _, core::ptr::null_mut());
        }

        Status = IntResolveDesktop(Process,
                                   &mut DesktopPath,
                                   if (*ProcessParams).WindowFlags & STARTF_INHERITDESKTOP != 0 { (1u8) as _ } else { (0u8) as _ },
                                   &mut hWinSta,
                                   &mut hDesk);

        if !DesktopPath.Buffer.is_null() {
            ExFreePoolWithTag(DesktopPath.Buffer as *mut core::ffi::c_void, TAG_STRING);
        }

        if !NT_SUCCESS(Status) {
            ExitThreadCallback(Thread);
            return Status;
        }

        if UserSetProcessWindowStation(hWinSta) == 0 {
            Status = STATUS_UNSUCCESSFUL;
            ExitThreadCallback(Thread);
            return Status;
        }

        /* Validate the new desktop */
        Status = IntValidateDesktopHandle(hDesk, 1i32 as i8, 0, &mut pdesk);
        if !NT_SUCCESS(Status) {
            ExitThreadCallback(Thread);
            return Status;
        }

        (*(*ptiCurrent).ppi).hdeskStartup = hDesk;
        (*(*ptiCurrent).ppi).rpdeskStartup = pdesk;
    }

    if !(*(*ptiCurrent).ppi).hdeskStartup.is_null() {
        if IntSetThreadDesktop((*(*ptiCurrent).ppi).hdeskStartup, (0u8) as _) == 0 {
            Status = STATUS_UNSUCCESSFUL;
            ExitThreadCallback(Thread);
            return Status;
        }
    }


    /* Mark the thread as fully initialized */
    (*ptiCurrent).TIF_flags |= TIF_GUITHREADINITIALIZED;

    if (*w32proc((*ptiCurrent).ppi)).W32PF_flags & (W32PF_ALLOWFOREGROUNDACTIVATE | W32PF_APPSTARTING) == 0 &&
        (!gptiForeground.is_null() && (*gptiForeground).ppi == (*ptiCurrent).ppi)
    {
        (*ptiCurrent).TIF_flags |= TIF_ALLOWFOREGROUNDACTIVATE;
    }
    (*(*ptiCurrent).pClientInfo).dwTIFlags = (*ptiCurrent).TIF_flags;

    /* Create the default input context */
    if IS_IMM_MODE() {
        UserCreateInputContext(0);
    }

    /* Last things to do only if we are not a SYSTEM or CSRSS thread */
    if (*ptiCurrent).TIF_flags & (TIF_SYSTEMTHREAD | TIF_CSRSSTHREAD) == 0 {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            // skip_client_thread_setup
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Status = co_IntClientThreadSetup();
            if !NT_SUCCESS(Status) {
                ExitThreadCallback(Thread);
                return Status;
            }
        }
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ExitThreadCallback(Thread: PETHREAD) -> NTSTATUS {
    let ppti: *mut PTHREADINFO;
    let mut psle: PSINGLE_LIST_ENTRY;
    let ppiCurrent: PPROCESSINFO;
    let Process: PEPROCESS;
    let ptiCurrent: PTHREADINFO;
    let mut pwl: PWINDOWLIST;
    let mut pwlNext: PWINDOWLIST;

    Process = (*Thread).ThreadsProcess;

    ptiCurrent = PsGetThreadWin32Thread(Thread as _) as PTHREADINFO;

    (*ptiCurrent).TIF_flags |= TIF_INCLEANUP;
    (*(*ptiCurrent).pClientInfo).dwTIFlags = (*ptiCurrent).TIF_flags;

    ppiCurrent = (*ptiCurrent).ppi;

    IsRemoveAttachThread(ptiCurrent);

    if !gpwlList.is_null() {
        pwl = gpwlList;
        while !pwl.is_null() {
            pwlNext = (*pwl).pNextList;
            if (*pwl).pti == ptiCurrent {
                IntFreeHwndList(pwl);
            }
            pwl = pwlNext;
        }
    }

    (*ptiCurrent).TIF_flags |= TIF_DONTATTACHQUEUE;
    (*(*ptiCurrent).pClientInfo).dwTIFlags = (*ptiCurrent).TIF_flags;

    UserCloseClipboard();

    (*ppiCurrent).cThreads -= 1;

    if (*ptiCurrent).TIF_flags & TIF_GUITHREADINITIALIZED != 0 {
        if (*(*ptiCurrent).ppi).cThreads == 0 {
            if (*ptiCurrent).ppi == ppiUahServer {
                UserUnregisterUserApiHook();
            }

            if !(*ptiCurrent).pDeskInfo.is_null() {
                if (*(*ptiCurrent).pDeskInfo).ppiShellProcess == ppiCurrent {
                    let ExitCode = PsGetProcessExitStatus(Process as _);

                    UserPostMessage(
                        hwndSAS,
                        WM_LOGONNOTIFY,
                        LN_SHELL_EXITED as WPARAM,
                        ExitCode as LPARAM,
                    );

                    (*(*ptiCurrent).pDeskInfo).ppiShellProcess = core::ptr::null_mut();
                }
            }
        }

        DceFreeThreadDCE(ptiCurrent);
        DestroyTimersForThread(ptiCurrent);
        KeSetEvent((*ptiCurrent).pEventQueueServer as _, IO_NO_INCREMENT, 0u8);
        UnregisterThreadHotKeys(ptiCurrent);

        if UserDestroyObjectsForOwner(gHandleTable as *mut core::ffi::c_void, ptiCurrent as *mut core::ffi::c_void) == 0 {
            return STATUS_UNSUCCESSFUL;
        }
        UserAssignmentUnlock(
            core::ptr::addr_of_mut!((*ptiCurrent).spDefaultImc) as *mut *mut core::ffi::c_void,
        );

        if !ppiCurrent.is_null()
            && (*ppiCurrent).ptiList == ptiCurrent
            && (*ptiCurrent).ptiSibling.is_null()
            && (*w32proc(ppiCurrent)).W32PF_flags & W32PF_CLASSESREGISTERED != 0
        {
            DestroyProcessClasses(ppiCurrent);
            (*w32proc(ppiCurrent)).W32PF_flags &= !W32PF_CLASSESREGISTERED;
        }

        IntBlockInput(ptiCurrent, (0u8) as _);
        IntCleanupThreadCallbacks(ptiCurrent);

        psle = PopEntryList(core::ptr::addr_of_mut!((*ptiCurrent).ReferencesList) as *mut _) as *mut _;
        while !psle.is_null() {
            let ref_entry = (psle as *mut u8)
                .offset(-(core::mem::offset_of!(USER_REFERENCE_ENTRY, Entry) as isize))
                as PUSER_REFERENCE_ENTRY;
            UserDereferenceObject((*ref_entry).obj);

            psle = PopEntryList(core::ptr::addr_of_mut!((*ptiCurrent).ReferencesList) as *mut _) as *mut _;
        }
    }

    if (*ptiCurrent).cEnterCount != 0 {
        KeSetKernelStackSwapEnable(1u8);
        (*ptiCurrent).cEnterCount = 0;
    }

    ppti = core::ptr::addr_of_mut!((*ppiCurrent).ptiList);
    let mut ppti = ppti;
    while !(*ppti).is_null() && *ppti != ptiCurrent {
        ppti = core::ptr::addr_of_mut!((**ppti).ptiSibling);
    }

    *ppti = (*ptiCurrent).ptiSibling;

    if UserAssignmentUnlock(
        core::ptr::addr_of_mut!((*ptiCurrent).KeyboardLayout) as *mut *mut core::ffi::c_void,
    ).is_null()
    {
        (*(*ptiCurrent).pClientInfo).hKL = core::ptr::null_mut();
    }

    if gptiForeground == ptiCurrent {
        gptiForeground = core::ptr::null_mut();
    }

    if (*ppiCurrent).cThreads == 0 {
        UserDisplayNotifyShutdown(ppiCurrent);
    }

    IntSetThreadDesktop(core::ptr::null_mut(), (1u8) as _);

    if !(*ptiCurrent).hEventQueueClient.is_null() {
        ObCloseHandle((*ptiCurrent).hEventQueueClient, 1i32 as i8);
        ObDereferenceObject((*ptiCurrent).pEventQueueServer as *mut core::ffi::c_void);
    }
    (*ptiCurrent).hEventQueueClient = core::ptr::null_mut();

    PsSetThreadWin32Thread(Thread as _, core::ptr::null_mut(), ptiCurrent as *mut core::ffi::c_void);

    IntDereferenceThreadInfo(ptiCurrent);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn Win32kThreadCallback(
    Thread: PETHREAD,
    Type: PSW32THREADCALLOUTTYPE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    UserEnterExclusive();

    if Type == PsW32ThreadCalloutInitialize {
        Status = InitThreadCallback(Thread);
    } else {
        Status = ExitThreadCallback(Thread);
    }

    UserLeave();

    Status
}

unsafe extern "system" fn DriverUnload(DriverObject: PDRIVER_OBJECT) {
    FreeFontSupport();
    ResetCsrApiPort();
    ResetCsrProcess();
    IntWin32PowerManagementCleanup();
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let mut CalloutData: WIN32_CALLOUTS_FPNS = core::mem::zeroed();
    let mut GlobalUserHeapBase: PVOID = core::ptr::null_mut();

    let Result = KeAddSystemServiceTable(
        Win32kSSDT.as_ptr() as *mut _,
        core::ptr::null_mut(),
        Win32kNumberOfSysCalls,
        Win32kSSPT.as_ptr() as *mut _,
        1,
    );
    if Result == 0u8 {
        return STATUS_UNSUCCESSFUL;
    }

    hModuleWin = MmPageEntireDriver(DriverEntry as *mut _);

    (*DriverObject).DriverUnload = Some(core::mem::transmute(DriverUnload as *const ()));
    CalloutData.ProcessCallout = Some(core::mem::transmute(Win32kProcessCallback as *const ()));
    CalloutData.ThreadCallout = Some(core::mem::transmute(Win32kThreadCallback as *const ()));
    CalloutData.PowerEventCallout = Some(core::mem::transmute(IntHandlePowerEvent as *const ()));
    CalloutData.PowerStateCallout = Some(core::mem::transmute(IntHandlePowerState as *const ()));
    CalloutData.BatchFlushRoutine = Some(core::mem::transmute(NtGdiFlushUserBatch as *const ()));
    CalloutData.JobCallout = Some(crate::job::Win32kJobCallout);
    CalloutData.DesktopOpenProcedure = Some(core::mem::transmute(IntDesktopObjectOpen as *const ()));
    CalloutData.DesktopOkToCloseProcedure = Some(core::mem::transmute(IntDesktopOkToClose as *const ()));
    CalloutData.DesktopCloseProcedure = Some(core::mem::transmute(IntDesktopObjectClose as *const ()));
    CalloutData.DesktopDeleteProcedure = Some(core::mem::transmute(IntDesktopObjectDelete as *const ()));
    CalloutData.WindowStationOkToCloseProcedure = Some(core::mem::transmute(IntWinStaOkToClose as *const ()));
    CalloutData.WindowStationDeleteProcedure = Some(core::mem::transmute(IntWinStaObjectDelete as *const ()));
    CalloutData.WindowStationParseProcedure = Some(core::mem::transmute(IntWinStaObjectParse as *const ()));
    PsEstablishWin32Callouts(core::ptr::addr_of_mut!(CalloutData) as *mut _);

    GlobalUserHeap = UserCreateHeap(
        &raw mut GlobalUserHeapSection,
        &mut GlobalUserHeapBase,
        1 * 1024 * 1024,
    );
    if GlobalUserHeap.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    ExInitializeResourceLite(core::ptr::addr_of_mut!(UserLock) as *mut _);
    UserEnterExclusive();
    gpsi = UserHeapAlloc(core::mem::size_of_val(&*gpsi)) as PSERVERINFO;
    UserLeave();
    if gpsi.is_null() {
        return STATUS_UNSUCCESSFUL;
    }
    RtlZeroMemory(gpsi as PVOID, core::mem::size_of_val(&*gpsi));

    Status = InitGdiHandleTable();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitPaletteImpl();
    if !NT_SUCCESS(Status) { return Status; }
    CreateStockObjects();
    CreateSysColorObjects();
    Status = InitBrushImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitPDEVImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitLDEVImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitDeviceImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitDcImpl();
    if !NT_SUCCESS(Status) { return Status; }

    UserEnterExclusive();
    Status = InitUserImpl();
    UserLeave();
    if !NT_SUCCESS(Status) { return Status; }

    Status = InitWindowStationImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitDesktopImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitInputImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitKeyboardImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = crate::rawinput::InitRawInputImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = MsqInitializeImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitTimerImpl();
    if !NT_SUCCESS(Status) { return Status; }
    Status = InitDCEImpl();
    if !NT_SUCCESS(Status) { return Status; }

    STATUS_SUCCESS
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn InitGdiHandleTable() -> NTSTATUS;
    fn InitPaletteImpl() -> NTSTATUS;
    fn InitBrushImpl() -> NTSTATUS;
    fn InitPDEVImpl() -> NTSTATUS;
    fn InitLDEVImpl() -> NTSTATUS;
    fn InitDeviceImpl() -> NTSTATUS;
    fn InitDcImpl() -> NTSTATUS;
    fn InitUserImpl() -> NTSTATUS;
    fn InitWindowStationImpl() -> NTSTATUS;
    fn InitDesktopImpl() -> NTSTATUS;
    fn InitInputImpl() -> NTSTATUS;
    fn InitKeyboardImpl() -> NTSTATUS;
    fn MsqInitializeImpl() -> NTSTATUS;
    fn InitTimerImpl() -> NTSTATUS;
    fn InitDCEImpl() -> NTSTATUS;
    fn co_IntClientThreadSetup() -> NTSTATUS;
    fn UserGetMonitorObject(hMonitor: HMONITOR) -> *mut core::ffi::c_void;
    fn UserCloseClipboard() -> BOOL;
    fn IntWin32PowerManagementCleanup() -> u32;
}
extern "C" {
    fn ResetCsrApiPort();
    fn ResetCsrProcess();
    fn MapGlobalUserHeap( Process: PEPROCESS, KernelMapping: *mut *mut core::ffi::c_void, UserMapping: *mut *mut core::ffi::c_void, ) -> NTSTATUS;
    fn GdiProcessCreate(Process: PEPROCESS) -> NTSTATUS;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn CreateStockObjects();
    fn CreateSysColorObjects();
    fn IntFreeImeHotKeys();
    fn co_IntGraphicsCheck(Create: u32) -> u32;
    fn UserSetProcessWindowStation(hWindowStation: HWINSTA) -> BOOL;
    fn IntCleanupCurIconCache(ppi: PPROCESSINFO);
    fn UserGetMouseButtonsState() -> WORD;
    fn UserAssignmentLock(ppvObj: *mut *mut core::ffi::c_void, pvNew: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn UserAssignmentUnlock(ppvObj: *mut *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn IntResolveDesktop(Process: PEPROCESS, DesktopPath: *mut _UNICODE_STRING, bInherit: u32, phWinSta: *mut HWINSTA, phDesktop: *mut HDESK) -> NTSTATUS;
    fn IntSafeCopyUnicodeStringTerminateNULL(Dest: *mut _UNICODE_STRING, Source: *mut _UNICODE_STRING) -> NTSTATUS;
    fn IntValidateDesktopHandle(hDesktop: HDESK, AccessMode: i8, DesiredAccess: u32, ppdesk: *mut PDESKTOP) -> NTSTATUS;
    fn UserCreateInputContext(dwClientImcData: usize) -> *mut core::ffi::c_void;
    fn MsqCreateMessageQueue(pti: PTHREADINFO) -> *mut _USER_MESSAGE_QUEUE;
    fn IsRemoveAttachThread(pti: PTHREADINFO) -> u32;
    fn IntFreeHwndList(pwl: PWINDOWLIST);
    fn UserUnregisterUserApiHook() -> u32;
    fn UserPostMessage(Wnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn DceFreeThreadDCE(pti: PTHREADINFO);
    fn DestroyTimersForThread(pti: PTHREADINFO) -> BOOL;
    fn UnregisterThreadHotKeys(pti: PTHREADINFO);
    fn DestroyProcessClasses(ppi: PPROCESSINFO);
    fn IntBlockInput(pti: PTHREADINFO, BlockIt: u32) -> BOOL;
    fn IntCleanupThreadCallbacks(pti: PTHREADINFO);
    fn UserDereferenceObject(obj: *mut core::ffi::c_void) -> u32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn CreateStockObjects();
    fn CreateSysColorObjects();
    fn IntFreeImeHotKeys();
    fn co_IntGraphicsCheck(Create: u32) -> u32;
    fn UserSetProcessWindowStation(hWindowStation: HWINSTA) -> BOOL;
    fn IntCleanupCurIconCache(ppi: PPROCESSINFO);
    fn UserGetMouseButtonsState() -> WORD;
    fn UserAssignmentLock(ppvObj: *mut *mut core::ffi::c_void, pvNew: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn UserAssignmentUnlock(ppvObj: *mut *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn IntResolveDesktop(Process: PEPROCESS, DesktopPath: *mut _UNICODE_STRING, bInherit: u32, phWinSta: *mut HWINSTA, phDesktop: *mut HDESK) -> NTSTATUS;
    fn IntSafeCopyUnicodeStringTerminateNULL(Dest: *mut _UNICODE_STRING, Source: *mut _UNICODE_STRING) -> NTSTATUS;
    fn IntValidateDesktopHandle(hDesktop: HDESK, AccessMode: i8, DesiredAccess: u32, ppdesk: *mut PDESKTOP) -> NTSTATUS;
    fn UserCreateInputContext(dwClientImcData: usize) -> *mut core::ffi::c_void;
    fn MsqCreateMessageQueue(pti: PTHREADINFO) -> *mut _USER_MESSAGE_QUEUE;
    fn IsRemoveAttachThread(pti: PTHREADINFO) -> u32;
    fn IntFreeHwndList(pwl: PWINDOWLIST);
    fn UserUnregisterUserApiHook() -> u32;
    fn UserPostMessage(Wnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn DceFreeThreadDCE(pti: PTHREADINFO);
    fn DestroyTimersForThread(pti: PTHREADINFO) -> BOOL;
    fn UnregisterThreadHotKeys(pti: PTHREADINFO);
    fn DestroyProcessClasses(ppi: PPROCESSINFO);
    fn IntBlockInput(pti: PTHREADINFO, BlockIt: u32) -> BOOL;
    fn IntCleanupThreadCallbacks(pti: PTHREADINFO);
    fn UserDereferenceObject(obj: *mut core::ffi::c_void) -> u32;
}
