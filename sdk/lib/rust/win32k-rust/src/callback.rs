use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeUserModeCallback, PsGetCurrentProcessWin32Process, PsGetCurrentThreadWin32Thread, RtlCopyUnicodeString};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::inline::ke::NtCurrentTeb;

extern "C" {
    // message.c: `UINT lParamMemorySize(...)` — cdecl (`_lParamMemorySize`).
    fn lParamMemorySize(Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> UINT;
    // cursoricon.c: `VOID IntLoadSystenIcons(HICON, DWORD)` — cdecl (`_IntLoadSystenIcons`).
    fn IntLoadSystenIcons(hcur: HICON, id: DWORD);
}
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// UserEnterCo()/UserLeaveCo() are ntuser.h macros for UserEnterExclusive()/
// UserLeave() — both FASTCALL (`@UserEnterExclusive@0`/`@UserLeave@0` on i386).
// (Matches dde.rs.)
#[cfg(target_arch = "x86")]
extern "fastcall" {
    #[link_name = "UserEnterExclusive"]
    fn UserEnterCo();
    #[link_name = "UserLeave"]
    fn UserLeaveCo();
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    #[link_name = "UserEnterExclusive"]
    fn UserEnterCo();
    #[link_name = "UserLeave"]
    fn UserLeaveCo();
}

// ERR() is a debug-channel macro (debug.h) — expands to __noop in non-debug
// builds, so there is no exported symbol. Swallow all args (matches cliprgn.rs/
// ntuser.rs which stub ERR as a no-op).

// USERTAG_CALLBACK = 'ccsU' (tags.h). C multichar is big-endian: bytes in memory
// little-endian are 'U','s','c','c'.
const USERTAG_CALLBACK: u32 = u32::from_le_bytes(*b"Uscc");

// ---- FORCEINLINE / macro reimplementations (no exported symbol) ----

// LIST_ENTRY inline ops on win32k_sys PLIST_ENTRY (matches event.rs/gdipool.rs).
#[inline(always)]
unsafe fn InsertTailList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let b = (*h).Blink;
    (*e).Flink = h;
    (*e).Blink = b;
    (*b).Flink = e;
    (*h).Blink = e;
}
#[inline(always)]
unsafe fn RemoveEntryList(e: PLIST_ENTRY) {
    let f = (*e).Flink;
    let b = (*e).Blink;
    (*b).Flink = f;
    (*f).Blink = b;
}
#[inline(always)]
unsafe fn RemoveHeadList(h: PLIST_ENTRY) -> PLIST_ENTRY {
    let e = (*h).Flink;
    RemoveEntryList(e);
    e
}
#[inline(always)]
unsafe fn IsListEmpty(h: PLIST_ENTRY) -> u8 {
    ((*h).Flink == h) as u8
}

// RtlCopyMemory / RtlZeroMemory (ntddk macros over memcpy/memset — no symbol).
#[inline(always)]
unsafe fn RtlCopyMemory(dst: PVOID, src: PVOID, len: SIZE_T) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len as usize);
}
#[inline(always)]
unsafe fn RtlZeroMemory(dst: PVOID, len: SIZE_T) {
    core::ptr::write_bytes(dst as *mut u8, 0, len as usize);
}

// GetWin32ClientInfo() = (PCLIENTINFO)NtCurrentTeb()->Win32ClientInfo (ntuser.h
// macro). Win32ClientInfo is the first byte of an inline field on the TEB, so its
// address is the PCLIENTINFO (matches misc.rs).
#[inline(always)]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    let teb = NtCurrentTeb() as *mut ntkrnl_sys::ddk::_TEB;
    core::ptr::addr_of_mut!((*teb).Win32ClientInfo) as PCLIENTINFO
}

// DesktopHeapAddressToUser (desktop.h static __inline — no exported symbol).
#[inline(always)]
unsafe fn DesktopHeapAddressToUser(lpMem: PVOID) -> PVOID {
    let W32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    let mut Mapping: PW32HEAP_USER_MAPPING = (*W32Process).HeapMappings.Next;
    while !Mapping.is_null() {
        if (lpMem as ULONG_PTR) >= ((*Mapping).KernelMapping as ULONG_PTR)
            && (lpMem as ULONG_PTR)
                < ((*Mapping).KernelMapping as ULONG_PTR) + (*Mapping).Limit
        {
            return (((lpMem as ULONG_PTR) - ((*Mapping).KernelMapping as ULONG_PTR))
                + ((*Mapping).UserMapping as ULONG_PTR)) as PVOID;
        }
        Mapping = (*Mapping).Next;
    }
    core::ptr::null_mut()
}

// gpsi->{hIconWindows,hIconSmWindows,oembmi} live in the anonymous
// PERUSERSERVERINFO member that bindgen flattened into tagSERVERINFO's
// __bindgen_padding_0 tail; cast it (matches misc.rs/focus.rs/nonclient.rs).
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _INT_CALLBACK_HEADER {
    pub ListEntry: _LIST_ENTRY,
}
pub type INT_CALLBACK_HEADER = _INT_CALLBACK_HEADER;
pub type PINT_CALLBACK_HEADER = *mut _INT_CALLBACK_HEADER;

static mut iTheId: INT = -2;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCbAllocateMemory(Size: ULONG) -> PVOID {
    IntCbAllocateMemory_impl(Size)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCbAllocateMemory(Size: ULONG) -> PVOID {
    IntCbAllocateMemory_impl(Size)
}

unsafe fn IntCbAllocateMemory_impl(Size: ULONG) -> PVOID {
    let Mem: PINT_CALLBACK_HEADER = ExAllocatePoolWithTag(
        PagedPool,
        (Size as usize + core::mem::size_of::<INT_CALLBACK_HEADER>()) as SIZE_T,
        USERTAG_CALLBACK,
    ) as PINT_CALLBACK_HEADER;

    if Mem.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(
        Mem as PVOID,
        (Size as usize + core::mem::size_of::<INT_CALLBACK_HEADER>()) as SIZE_T,
    );

    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    InsertTailList(
        core::ptr::addr_of_mut!((*W32Thread).W32CallbackListHead),
        core::ptr::addr_of_mut!((*Mem).ListEntry),
    );

    Mem.add(1) as PVOID
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCbFreeMemory(Data: PVOID) {
    IntCbFreeMemory_impl(Data)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCbFreeMemory(Data: PVOID) {
    IntCbFreeMemory_impl(Data)
}

unsafe fn IntCbFreeMemory_impl(Data: PVOID) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if (*W32Thread).TIF_flags & TIF_INCLEANUP != 0 {
        return;
    }

    let Mem = (Data as PINT_CALLBACK_HEADER).offset(-1);

    RemoveEntryList(core::ptr::addr_of_mut!((*Mem).ListEntry));

    ExFreePoolWithTag(Mem as PVOID, USERTAG_CALLBACK);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCleanupThreadCallbacks(W32Thread: PTHREADINFO) {
    IntCleanupThreadCallbacksImpl(W32Thread)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCleanupThreadCallbacks(W32Thread: PTHREADINFO) {
    IntCleanupThreadCallbacksImpl(W32Thread)
}

unsafe fn IntCleanupThreadCallbacksImpl(W32Thread: PTHREADINFO) {
    while IsListEmpty(core::ptr::addr_of_mut!((*W32Thread).W32CallbackListHead)) == 0 {
        let CurrentEntry: PLIST_ENTRY =
            RemoveHeadList(core::ptr::addr_of_mut!((*W32Thread).W32CallbackListHead));

        let Mem = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_INT_CALLBACK_HEADER, ListEntry) as isize))
            as *mut _INT_CALLBACK_HEADER;

        ExFreePoolWithTag(Mem as *mut core::ffi::c_void, USERTAG_CALLBACK);
    }
}

unsafe fn IntSetTebWndCallback(hWnd: *mut HWND, pWnd: *mut PWND, pActCtx: *mut *mut core::ffi::c_void) {
    let hWndS = *hWnd;
    let Window = UserGetWindowObject(*hWnd);
    let ClientInfo = GetWin32ClientInfo();

    *hWnd = (*ClientInfo).CallbackWnd.hWnd;
    *pWnd = (*ClientInfo).CallbackWnd.pWnd;
    *pActCtx = (*ClientInfo).CallbackWnd.pActCtx;

    if !Window.is_null() {
        (*ClientInfo).CallbackWnd.hWnd = hWndS;
        (*ClientInfo).CallbackWnd.pWnd = DesktopHeapAddressToUser(Window as *mut core::ffi::c_void) as PWND;
        (*ClientInfo).CallbackWnd.pActCtx = (*Window).pActCtx;
    } else {
        (*ClientInfo).CallbackWnd.hWnd = hWndS;
        (*ClientInfo).CallbackWnd.pWnd = Window;
        (*ClientInfo).CallbackWnd.pActCtx = 0 as *mut core::ffi::c_void;
    }
}

unsafe fn IntRestoreTebWndCallback(hWnd: HWND, pWnd: PWND, pActCtx: *mut core::ffi::c_void) {
    let ClientInfo: PCLIENTINFO = GetWin32ClientInfo();

    (*ClientInfo).CallbackWnd.hWnd = hWnd;
    (*ClientInfo).CallbackWnd.pWnd = pWnd;
    (*ClientInfo).CallbackWnd.pActCtx = pActCtx;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn co_IntClientLoadLibrary(
    pstrLibName: PUNICODE_STRING,
    pstrInitFunc: PUNICODE_STRING,
    Unload: BOOL,
    ApiHook: BOOL,
) -> BOOL {
    let mut ResultPointer: PVOID = core::ptr::null_mut();
    let mut ResultLength: ULONG = 0;
    let mut ArgumentLength: ULONG;
    let pArguments: PCLIENT_LOAD_LIBRARY_ARGUMENTS;
    let mut Status: NTSTATUS;
    let mut bResult: BOOL = 0;
    let mut pLibNameBuffer: ULONG_PTR = 0;
    let mut pInitFuncBuffer: ULONG_PTR = 0;

    /* Calculate the size of the argument */
    ArgumentLength = core::mem::size_of::<CLIENT_LOAD_LIBRARY_ARGUMENTS>() as ULONG;
    if !pstrLibName.is_null() {
        pLibNameBuffer = ArgumentLength as ULONG_PTR;
        ArgumentLength += (*pstrLibName).Length as ULONG + core::mem::size_of::<u16>() as ULONG;
    }
    if !pstrInitFunc.is_null() {
        pInitFuncBuffer = ArgumentLength as ULONG_PTR;
        ArgumentLength += (*pstrInitFunc).Length as ULONG + core::mem::size_of::<u16>() as ULONG;
    }

    /* Allocate the argument */
    pArguments = IntCbAllocateMemory(ArgumentLength) as PCLIENT_LOAD_LIBRARY_ARGUMENTS;
    if pArguments.is_null() {
        return 0;
    }

    /* Fill the argument */
    (*pArguments).Unload = Unload;
    (*pArguments).ApiHook = ApiHook;
    if !pstrLibName.is_null() {
        /* Copy the string to the callback memory */
        pLibNameBuffer += pArguments as ULONG_PTR;
        (*pArguments).strLibraryName.Buffer = pLibNameBuffer as *mut u16;
        (*pArguments).strLibraryName.MaximumLength =
            (*pstrLibName).Length + core::mem::size_of::<u16>() as u16;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*pArguments).strLibraryName) as _,
            pstrLibName as _,
        );

        /* Fix argument pointer to be relative to the argument */
        pLibNameBuffer -= pArguments as ULONG_PTR;
        (*pArguments).strLibraryName.Buffer = pLibNameBuffer as *mut u16;
    }

    if !pstrInitFunc.is_null() {
        /* Copy the strings to the callback memory */
        pInitFuncBuffer += pArguments as ULONG_PTR;
        (*pArguments).strInitFuncName.Buffer = pInitFuncBuffer as *mut u16;
        (*pArguments).strInitFuncName.MaximumLength =
            (*pstrInitFunc).Length + core::mem::size_of::<u16>() as u16;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*pArguments).strInitFuncName) as _,
            pstrInitFunc as _,
        );

        /* Fix argument pointers to be relative to the argument */
        pInitFuncBuffer -= pArguments as ULONG_PTR;
        (*pArguments).strInitFuncName.Buffer = pInitFuncBuffer as *mut u16;
    }

    /* Do the callback */
    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_CLIENTLOADLIBRARY,
        pArguments as PVOID,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    /* Free the argument */
    IntCbFreeMemory(pArguments as PVOID);

    if !NT_SUCCESS(Status) {
        return 0;
    }

    Status = SafeProbeForRead(ResultPointer, core::mem::size_of::<HMODULE>() as ULONG, 1);
    if !NT_SUCCESS(Status) {
        bResult = 0;
    } else {
        /* Probe and copy the usermode result data. The probe is validate-only
         * (USE_DUMMY_PSEH) and the C had probe AND load in one _SEH2_TRY, so the
         * load itself has to be the fault-tolerant copy — this buffer is written
         * by user mode and can be unmapped by the time we read it. */
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(bResult) as *mut core::ffi::c_void,
            ResultPointer as *const core::ffi::c_void,
            core::mem::size_of::<BOOL>(),
        ) {
            bResult = 0;
        }
    }

    bResult
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn co_IntClientLoadLibrary(
    pstrLibName: PUNICODE_STRING,
    pstrInitFunc: PUNICODE_STRING,
    Unload: BOOL,
    ApiHook: BOOL,
) -> BOOL {
    let mut ResultPointer: PVOID = core::ptr::null_mut();
    let mut ResultLength: ULONG = 0;
    let mut ArgumentLength: ULONG;
    let pArguments: PCLIENT_LOAD_LIBRARY_ARGUMENTS;
    let mut Status: NTSTATUS;
    let mut bResult: BOOL = 0;
    let mut pLibNameBuffer: ULONG_PTR = 0;
    let mut pInitFuncBuffer: ULONG_PTR = 0;

    /* Calculate the size of the argument */
    ArgumentLength = core::mem::size_of::<CLIENT_LOAD_LIBRARY_ARGUMENTS>() as ULONG;
    if !pstrLibName.is_null() {
        pLibNameBuffer = ArgumentLength as ULONG_PTR;
        ArgumentLength += (*pstrLibName).Length as ULONG + core::mem::size_of::<u16>() as ULONG;
    }
    if !pstrInitFunc.is_null() {
        pInitFuncBuffer = ArgumentLength as ULONG_PTR;
        ArgumentLength += (*pstrInitFunc).Length as ULONG + core::mem::size_of::<u16>() as ULONG;
    }

    /* Allocate the argument */
    pArguments = IntCbAllocateMemory(ArgumentLength) as PCLIENT_LOAD_LIBRARY_ARGUMENTS;
    if pArguments.is_null() {
        return 0;
    }

    /* Fill the argument */
    (*pArguments).Unload = Unload;
    (*pArguments).ApiHook = ApiHook;
    if !pstrLibName.is_null() {
        /* Copy the string to the callback memory */
        pLibNameBuffer += pArguments as ULONG_PTR;
        (*pArguments).strLibraryName.Buffer = pLibNameBuffer as *mut u16;
        (*pArguments).strLibraryName.MaximumLength =
            (*pstrLibName).Length + core::mem::size_of::<u16>() as u16;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*pArguments).strLibraryName) as _,
            pstrLibName as _,
        );

        /* Fix argument pointer to be relative to the argument */
        pLibNameBuffer -= pArguments as ULONG_PTR;
        (*pArguments).strLibraryName.Buffer = pLibNameBuffer as *mut u16;
    }

    if !pstrInitFunc.is_null() {
        /* Copy the strings to the callback memory */
        pInitFuncBuffer += pArguments as ULONG_PTR;
        (*pArguments).strInitFuncName.Buffer = pInitFuncBuffer as *mut u16;
        (*pArguments).strInitFuncName.MaximumLength =
            (*pstrInitFunc).Length + core::mem::size_of::<u16>() as u16;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*pArguments).strInitFuncName) as _,
            pstrInitFunc as _,
        );

        /* Fix argument pointers to be relative to the argument */
        pInitFuncBuffer -= pArguments as ULONG_PTR;
        (*pArguments).strInitFuncName.Buffer = pInitFuncBuffer as *mut u16;
    }

    /* Do the callback */
    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_CLIENTLOADLIBRARY,
        pArguments as PVOID,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    /* Free the argument */
    IntCbFreeMemory(pArguments as PVOID);

    if !NT_SUCCESS(Status) {
        return 0;
    }

    Status = SafeProbeForRead(ResultPointer, core::mem::size_of::<HMODULE>() as _, 1);
    if !NT_SUCCESS(Status) {
        bResult = 0;
    } else {
        /* Probe and copy the usermode result data. The probe is validate-only
         * (USE_DUMMY_PSEH) and the C had probe AND load in one _SEH2_TRY, so the
         * load itself has to be the fault-tolerant copy — this buffer is written
         * by user mode and can be unmapped by the time we read it. */
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(bResult) as *mut core::ffi::c_void,
            ResultPointer as *const core::ffi::c_void,
            core::mem::size_of::<BOOL>(),
        ) {
            bResult = 0;
        }
    }

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn co_IntCallSentMessageCallback(
    CompletionCallback: SENDASYNCPROC,
    mut hWnd: HWND,
    Msg: UINT,
    CompletionCallbackContext: ULONG_PTR,
    Result: LRESULT,
) {
    let mut Arguments: SENDASYNCPROC_CALLBACK_ARGUMENTS = core::mem::zeroed();
    let mut ResultPointer: PVOID = core::ptr::null_mut();
    let mut pActCtx: PVOID = core::ptr::null_mut();
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut ResultLength: ULONG = 0;
    let Status: NTSTATUS;

    Arguments.Callback = CompletionCallback;
    Arguments.Wnd = hWnd;
    Arguments.Msg = Msg;
    Arguments.Context = CompletionCallbackContext;
    Arguments.Result = Result;

    IntSetTebWndCallback(&mut hWnd, &mut pWnd, &mut pActCtx);

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_SENDASYNCPROC,
        core::ptr::addr_of_mut!(Arguments) as PVOID,
        core::mem::size_of::<SENDASYNCPROC_CALLBACK_ARGUMENTS>() as ULONG,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    IntRestoreTebWndCallback(hWnd, pWnd, pActCtx);

    if !NT_SUCCESS(Status) {
        return;
    }
}
#[no_mangle]
pub unsafe extern "system" fn co_IntCallWindowProc(
    Proc: WNDPROC,
    IsAnsiProc: BOOLEAN,
    mut Wnd: HWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lParamBufferSize: INT,
) -> LRESULT {
    let mut StackArguments: WINDOWPROC_CALLBACK_ARGUMENTS = core::mem::zeroed();
    let Arguments: PWINDOWPROC_CALLBACK_ARGUMENTS;
    let Status: NTSTATUS;
    let mut ResultPointer: PVOID;
    let mut pActCtx: PVOID = core::ptr::null_mut();
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut ResultLength: ULONG;
    let ArgumentLength: ULONG;
    let Result: LRESULT;

    if lParamBufferSize != -1 {
        ArgumentLength = core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>() as ULONG
            + lParamBufferSize as ULONG;
        Arguments = IntCbAllocateMemory(ArgumentLength) as PWINDOWPROC_CALLBACK_ARGUMENTS;
        if Arguments.is_null() {
            return -1;
        }
        core::ptr::copy(
            lParam as PVOID,
            (Arguments as *mut core::ffi::c_char)
                .add(core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>()) as PVOID,
            lParamBufferSize as usize,
        );
    } else {
        Arguments = core::ptr::addr_of_mut!(StackArguments);
        ArgumentLength = core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>() as ULONG;
    }
    (*Arguments).Proc = Proc;
    (*Arguments).IsAnsiProc = IsAnsiProc as _;
    (*Arguments).Wnd = Wnd;
    (*Arguments).Msg = Message;
    (*Arguments).wParam = wParam;
    (*Arguments).lParam = lParam;
    (*Arguments).lParamBufferSize = lParamBufferSize;
    ResultPointer = core::ptr::null_mut();
    ResultLength = ArgumentLength;

    IntSetTebWndCallback(&mut Wnd, &mut pWnd, &mut pActCtx);

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_WINDOWPROC,
        Arguments as PVOID,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );
    if !NT_SUCCESS(Status) {
        UserEnterCo();
        return 0;
    }

    core::ptr::copy(
        ResultPointer,
        Arguments as PVOID,
        ArgumentLength as usize,
    );

    UserEnterCo();

    IntRestoreTebWndCallback(Wnd, pWnd, pActCtx);

    if !NT_SUCCESS(Status) {
        if lParamBufferSize != -1 {
            IntCbFreeMemory(Arguments as PVOID);
        }
        return -1;
    }

    Result = (*Arguments).Result;

    if lParamBufferSize != -1 {
        let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        let InSendMessage: BOOL = (((*(*pti).pcti).CTI_flags & CTI_INSENDMESSAGE) != 0) as BOOL;

        match Message {
            WM_CREATE | WM_GETMINMAXINFO | WM_GETTEXT | WM_NCCALCSIZE | WM_NCCREATE
            | WM_STYLECHANGING | WM_WINDOWPOSCHANGING | WM_SIZING | WM_MOVING
            | WM_MEASUREITEM | WM_NEXTMENU => {
                if InSendMessage != 0 {
                    core::ptr::copy(
                        (Arguments as *mut core::ffi::c_char)
                            .add(core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>())
                            as PVOID,
                        lParam as PVOID,
                        lParamBufferSize as usize,
                    );
                } else {
                    core::ptr::copy(
                        (Arguments as *mut core::ffi::c_char)
                            .add(core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>())
                            as PVOID,
                        lParam as PVOID,
                        lParamBufferSize as usize,
                    );
                }
            }
            _ => {}
        }
        IntCbFreeMemory(Arguments as PVOID);
    }
    Result
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn co_IntLoadSysMenuTemplate() -> HMENU {
    let mut Result: LRESULT = 0;
    let mut Status: NTSTATUS;
    let mut ResultPointer: PVOID;
    let mut ResultLength: ULONG;

    ResultPointer = core::ptr::null_mut();
    ResultLength = core::mem::size_of::<LRESULT>() as ULONG;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_LOADSYSMENUTEMPLATE,
        core::ptr::addr_of_mut!(ResultPointer) as PVOID,
        0,
        core::ptr::addr_of_mut!(ResultPointer),
        core::ptr::addr_of_mut!(ResultLength),
    );

    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(ResultPointer, core::mem::size_of::<LRESULT>() as ULONG, 1);
        if !NT_SUCCESS(Status) {
            Result = 0;
        } else if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(Result) as *mut core::ffi::c_void,
            ResultPointer as *const core::ffi::c_void,
            core::mem::size_of::<LRESULT>(),
        ) {
            // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
            Result = 0;
        }
    }

    UserEnterCo();

    Result as HMENU
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn co_IntLoadSysMenuTemplate() -> HMENU {
    let mut Result: LRESULT = 0;
    let mut Status: NTSTATUS;
    let mut ResultPointer: PVOID;
    let mut ResultLength: ULONG;

    ResultPointer = core::ptr::null_mut();
    ResultLength = core::mem::size_of::<LRESULT>() as ULONG;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_LOADSYSMENUTEMPLATE,
        core::ptr::addr_of_mut!(ResultPointer) as PVOID,
        0,
        core::ptr::addr_of_mut!(ResultPointer),
        core::ptr::addr_of_mut!(ResultLength),
    );

    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(ResultPointer, core::mem::size_of::<LRESULT>() as _, 1);
        if !NT_SUCCESS(Status) {
            Result = 0;
        } else if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(Result) as *mut core::ffi::c_void,
            ResultPointer as *const core::ffi::c_void,
            core::mem::size_of::<LRESULT>(),
        ) {
            // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
            Result = 0;
        }
    }

    UserEnterCo();

    Result as HMENU
}
#[no_mangle]
pub unsafe extern "system" fn co_IntLoadDefaultCursors() -> BOOL {
    let Status: NTSTATUS;
    let mut ResultPointer: PVOID;
    let mut ResultLength: ULONG;
    let mut DefaultCursor: BOOL = 1;

    ResultPointer = core::ptr::null_mut();
    ResultLength = core::mem::size_of::<HCURSOR>() as ULONG;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_LOADDEFAULTCURSORS,
        core::ptr::addr_of_mut!(DefaultCursor) as PVOID,
        core::mem::size_of::<BOOL>() as ULONG,
        core::ptr::addr_of_mut!(ResultPointer),
        core::ptr::addr_of_mut!(ResultLength),
    );

    UserEnterCo();

    if !NT_SUCCESS(Status) {
        return 0;
    }

    crate::desktop::gDesktopCursor = *(ResultPointer as *mut HCURSOR);

    1
}
#[no_mangle]
pub unsafe extern "system" fn co_IntCallHookProc(
    HookId: INT,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
    Proc: HOOKPROC,
    Mod: INT,
    offPfn: ULONG_PTR,
    Ansi: BOOLEAN,
    ModuleName: PUNICODE_STRING,
) -> LRESULT {
    let mut ArgumentLength: ULONG;
    let mut Argument: PVOID = core::ptr::null_mut();
    let mut Result: LRESULT = 0;
    let Status: NTSTATUS;
    let mut ResultPointer: PVOID;
    let mut ResultLength: ULONG;
    let Common: PHOOKPROC_CALLBACK_ARGUMENTS;
    let mut CbtCreateWnd: *mut CBT_CREATEWNDW = core::ptr::null_mut();
    let mut Extra: PCHAR = core::ptr::null_mut();
    let mut CbtCreatewndExtra: PHOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS = core::ptr::null_mut();
    let pti: PTHREADINFO;
    let pWnd: PWND;
    let mut pMsg: PMSG = core::ptr::null_mut();
    let mut Hit: BOOL = 0;
    let mut lParamSize: UINT = 0;
    let mut pCWP: *mut CWPSTRUCT = core::ptr::null_mut();
    let mut pCWPR: *mut CWPRETSTRUCT = core::ptr::null_mut();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if (*pti).TIF_flags & TIF_INCLEANUP != 0 {
        return 0;
    }

    ArgumentLength = core::mem::size_of::<HOOKPROC_CALLBACK_ARGUMENTS>() as ULONG;

    'fault: {
        match HookId {
            x if x == WH_CBT as INT => {
                match Code {
                    c if c == HCBT_CREATEWND as INT => {
                        pWnd = UserGetWindowObject(wParam as HWND);
                        if pWnd.is_null() {
                            break 'fault;
                        }
                        CbtCreateWnd = lParam as *mut CBT_CREATEWNDW;
                        ArgumentLength +=
                            core::mem::size_of::<HOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS>() as ULONG;
                    }
                    c if c == HCBT_MOVESIZE as INT => {
                        ArgumentLength += core::mem::size_of::<RECTL>() as ULONG;
                    }
                    c if c == HCBT_ACTIVATE as INT => {
                        ArgumentLength += core::mem::size_of::<CBTACTIVATESTRUCT>() as ULONG;
                    }
                    c if c == HCBT_CLICKSKIPPED as INT => {
                        ArgumentLength += core::mem::size_of::<MOUSEHOOKSTRUCT>() as ULONG;
                    }
                    c if c == HCBT_KEYSKIPPED as INT
                        || c == HCBT_MINMAX as INT
                        || c == HCBT_SETFOCUS as INT
                        || c == HCBT_SYSCOMMAND as INT
                        || c == HCBT_DESTROYWND as INT
                        || c == HCBT_QS as INT =>
                    {
                    }
                    _ => {
                        break 'fault;
                    }
                }
            }
            x if x == WH_KEYBOARD_LL as INT => {
                ArgumentLength += core::mem::size_of::<KBDLLHOOKSTRUCT>() as ULONG;
            }
            x if x == WH_MOUSE_LL as INT => {
                ArgumentLength += core::mem::size_of::<MSLLHOOKSTRUCT>() as ULONG;
            }
            x if x == WH_MOUSE as INT => {
                ArgumentLength += core::mem::size_of::<MOUSEHOOKSTRUCT>() as ULONG;
            }
            x if x == WH_CALLWNDPROC as INT => {
                pCWP = lParam as *mut CWPSTRUCT;
                ArgumentLength = core::mem::size_of::<CWP_Struct>() as ULONG;
                if (*pCWP).message == WM_CREATE || (*pCWP).message == WM_NCCREATE {
                    lParamSize = core::mem::size_of::<CREATESTRUCTW>() as UINT;
                } else {
                    lParamSize = lParamMemorySize((*pCWP).message, (*pCWP).wParam, (*pCWP).lParam);
                }
                ArgumentLength += lParamSize;
            }
            x if x == WH_CALLWNDPROCRET as INT => {
                pCWPR = lParam as *mut CWPRETSTRUCT;
                ArgumentLength = core::mem::size_of::<CWPR_Struct>() as ULONG;
                if (*pCWPR).message == WM_CREATE || (*pCWPR).message == WM_NCCREATE {
                    lParamSize = core::mem::size_of::<CREATESTRUCTW>() as UINT;
                } else {
                    lParamSize =
                        lParamMemorySize((*pCWPR).message, (*pCWPR).wParam, (*pCWPR).lParam);
                }
                ArgumentLength += lParamSize;
            }
            x if x == WH_MSGFILTER as INT
                || x == WH_SYSMSGFILTER as INT
                || x == WH_GETMESSAGE as INT =>
            {
                ArgumentLength += core::mem::size_of::<MSG>() as ULONG;
            }
            x if x == WH_FOREGROUNDIDLE as INT
                || x == WH_KEYBOARD as INT
                || x == WH_SHELL as INT => {}
            _ => {
                break 'fault;
            }
        }

        Argument = IntCbAllocateMemory(ArgumentLength);
        if Argument.is_null() {
            break 'fault;
        }
        Common = Argument as PHOOKPROC_CALLBACK_ARGUMENTS;
        (*Common).HookId = HookId;
        (*Common).Code = Code;
        (*Common).wParam = wParam;
        (*Common).lParam = lParam;
        (*Common).Proc = Proc;
        (*Common).Mod = Mod as _;
        (*Common).offPfn = offPfn;
        (*Common).Ansi = Ansi;
        (*Common).lParamSize = lParamSize;
        if !(*ModuleName).Buffer.is_null() && (*ModuleName).Length != 0 {
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*Common).ModuleName) as PVOID,
                (*ModuleName).Buffer as PVOID,
                (*ModuleName).Length as SIZE_T,
            );
        }
        Extra = (Common as PCHAR)
            .add(core::mem::size_of::<HOOKPROC_CALLBACK_ARGUMENTS>());

        match HookId {
            x if x == WH_CBT as INT => match Code {
                c if c == HCBT_CREATEWND as INT => {
                    CbtCreatewndExtra = Extra as PHOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS;
                    RtlCopyMemory(
                        core::ptr::addr_of_mut!((*CbtCreatewndExtra).Cs) as PVOID,
                        (*CbtCreateWnd).lpcs as PVOID,
                        core::mem::size_of::<CREATESTRUCTW>() as SIZE_T,
                    );
                    (*CbtCreatewndExtra).WndInsertAfter = (*CbtCreateWnd).hwndInsertAfter;
                    (*CbtCreatewndExtra).Cs.lpszClass = (*(*CbtCreateWnd).lpcs).lpszClass;
                    (*CbtCreatewndExtra).Cs.lpszName = (*(*CbtCreateWnd).lpcs).lpszName;
                    (*Common).lParam =
                        (Extra as isize - Common as PCHAR as isize) as LPARAM;
                }
                c if c == HCBT_CLICKSKIPPED as INT => {
                    RtlCopyMemory(
                        Extra as PVOID,
                        lParam as PVOID,
                        core::mem::size_of::<MOUSEHOOKSTRUCT>() as SIZE_T,
                    );
                    (*Common).lParam =
                        (Extra as isize - Common as PCHAR as isize) as LPARAM;
                }
                c if c == HCBT_MOVESIZE as INT => {
                    RtlCopyMemory(
                        Extra as PVOID,
                        lParam as PVOID,
                        core::mem::size_of::<RECTL>() as SIZE_T,
                    );
                    (*Common).lParam =
                        (Extra as isize - Common as PCHAR as isize) as LPARAM;
                }
                c if c == HCBT_ACTIVATE as INT => {
                    RtlCopyMemory(
                        Extra as PVOID,
                        lParam as PVOID,
                        core::mem::size_of::<CBTACTIVATESTRUCT>() as SIZE_T,
                    );
                    (*Common).lParam =
                        (Extra as isize - Common as PCHAR as isize) as LPARAM;
                }
                _ => {}
            },
            x if x == WH_KEYBOARD_LL as INT => {
                RtlCopyMemory(
                    Extra as PVOID,
                    lParam as PVOID,
                    core::mem::size_of::<KBDLLHOOKSTRUCT>() as SIZE_T,
                );
                (*Common).lParam = (Extra as isize - Common as PCHAR as isize) as LPARAM;
            }
            x if x == WH_MOUSE_LL as INT => {
                RtlCopyMemory(
                    Extra as PVOID,
                    lParam as PVOID,
                    core::mem::size_of::<MSLLHOOKSTRUCT>() as SIZE_T,
                );
                (*Common).lParam = (Extra as isize - Common as PCHAR as isize) as LPARAM;
            }
            x if x == WH_MOUSE as INT => {
                RtlCopyMemory(
                    Extra as PVOID,
                    lParam as PVOID,
                    core::mem::size_of::<MOUSEHOOKSTRUCT>() as SIZE_T,
                );
                (*Common).lParam = (Extra as isize - Common as PCHAR as isize) as LPARAM;
            }
            x if x == WH_CALLWNDPROC as INT => {
                let pcwps: PCWP_Struct = Common as PCWP_Struct;
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*pcwps).cwps) as PVOID,
                    pCWP as PVOID,
                    core::mem::size_of::<CWPSTRUCT>() as SIZE_T,
                );
                if lParamSize != 0 {
                    RtlCopyMemory(
                        core::ptr::addr_of_mut!((*pcwps).Extra) as PVOID,
                        (*pCWP).lParam as PVOID,
                        lParamSize as SIZE_T,
                    );
                }
            }
            x if x == WH_CALLWNDPROCRET as INT => {
                let pcwprs: PCWPR_Struct = Common as PCWPR_Struct;
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*pcwprs).cwprs) as PVOID,
                    pCWPR as PVOID,
                    core::mem::size_of::<CWPRETSTRUCT>() as SIZE_T,
                );
                if lParamSize != 0 {
                    RtlCopyMemory(
                        core::ptr::addr_of_mut!((*pcwprs).Extra) as PVOID,
                        (*pCWPR).lParam as PVOID,
                        lParamSize as SIZE_T,
                    );
                }
            }
            x if x == WH_MSGFILTER as INT
                || x == WH_SYSMSGFILTER as INT
                || x == WH_GETMESSAGE as INT =>
            {
                pMsg = lParam as PMSG;
                RtlCopyMemory(
                    Extra as PVOID,
                    pMsg as PVOID,
                    core::mem::size_of::<MSG>() as SIZE_T,
                );
                (*Common).lParam = (Extra as isize - Common as PCHAR as isize) as LPARAM;
            }
            _ => {}
        }

        ResultPointer = core::ptr::null_mut();
        ResultLength = ArgumentLength;

        UserLeaveCo();

        Status = KeUserModeCallback(
            USER32_CALLBACK_HOOKPROC,
            Argument,
            ArgumentLength,
            &mut ResultPointer,
            &mut ResultLength,
        );

        UserEnterCo();

        if !NT_SUCCESS(Status) {
            if iTheId != HookId {
                iTheId = HookId;
            }
            break 'fault;
        }

        if !ResultPointer.is_null() {
            core::ptr::copy(ResultPointer, Argument, ArgumentLength as usize);
            Result = (*Common).Result;
        }

        match HookId {
            x if x == WH_CBT as INT => {
                match Code {
                    c if c == HCBT_CREATEWND as INT => {
                        if !CbtCreatewndExtra.is_null() {
                            (*CbtCreateWnd).hwndInsertAfter = (*CbtCreatewndExtra).WndInsertAfter;
                            (*(*CbtCreateWnd).lpcs).x = (*CbtCreatewndExtra).Cs.x;
                            (*(*CbtCreateWnd).lpcs).y = (*CbtCreatewndExtra).Cs.y;
                            (*(*CbtCreateWnd).lpcs).cx = (*CbtCreatewndExtra).Cs.cx;
                            (*(*CbtCreateWnd).lpcs).cy = (*CbtCreatewndExtra).Cs.cy;
                        }
                    }
                    c if c == HCBT_MOVESIZE as INT => {
                        if !Extra.is_null() && lParam != 0 {
                            RtlCopyMemory(
                                lParam as PVOID,
                                Extra as PVOID,
                                core::mem::size_of::<RECTL>() as SIZE_T,
                            );
                        }
                    }
                    _ => {}
                }
                if !pMsg.is_null() {
                    RtlCopyMemory(
                        pMsg as PVOID,
                        Extra as PVOID,
                        core::mem::size_of::<MSG>() as SIZE_T,
                    );
                }
            }
            x if x == WH_GETMESSAGE as INT => {
                if !pMsg.is_null() {
                    RtlCopyMemory(
                        pMsg as PVOID,
                        Extra as PVOID,
                        core::mem::size_of::<MSG>() as SIZE_T,
                    );
                }
            }
            _ => {}
        }
    }

    if Hit != 0 {}
    if !Argument.is_null() {
        IntCbFreeMemory(Argument);
    }

    Result
}
#[no_mangle]
pub unsafe extern "system" fn co_IntCallEventProc(
    hook: HWINEVENTHOOK,
    event: DWORD,
    hWnd: HWND,
    idObject: LONG,
    idChild: LONG,
    dwEventThread: DWORD,
    dwmsEventTime: DWORD,
    Proc: WINEVENTPROC,
    Mod: INT,
    offPfn: ULONG_PTR,
) -> LRESULT {
    let Result: LRESULT = 0;
    let Status: NTSTATUS;
    let Common: PEVENTPROC_CALLBACK_ARGUMENTS;
    let ArgumentLength: ULONG;
    let mut ResultLength: ULONG;
    let Argument: PVOID;
    let mut ResultPointer: PVOID;

    ArgumentLength = core::mem::size_of::<EVENTPROC_CALLBACK_ARGUMENTS>() as ULONG;

    Argument = IntCbAllocateMemory(ArgumentLength);
    if Argument.is_null() {
        ERR!("EventProc callback failed: out of memory\n");
        return 0;
    }
    Common = Argument as PEVENTPROC_CALLBACK_ARGUMENTS;
    (*Common).hook = hook;
    (*Common).event = event;
    (*Common).hwnd = hWnd;
    (*Common).idObject = idObject;
    (*Common).idChild = idChild;
    (*Common).dwEventThread = dwEventThread;
    (*Common).dwmsEventTime = dwmsEventTime;
    (*Common).Proc = Proc;
    (*Common).Mod = Mod as _;
    (*Common).offPfn = offPfn;

    ResultPointer = core::ptr::null_mut();
    ResultLength = core::mem::size_of::<LRESULT>() as ULONG;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_EVENTPROC,
        Argument,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    IntCbFreeMemory(Argument);

    if !NT_SUCCESS(Status) {
        return 0;
    }

    Result
}
#[no_mangle]
pub unsafe extern "system" fn co_IntCallLoadMenu(
    hModule: HINSTANCE,
    pMenuName: PUNICODE_STRING,
) -> HMENU {
    let mut Result: LRESULT = 0;
    let Status: NTSTATUS;
    let Common: PLOADMENU_CALLBACK_ARGUMENTS;
    let mut ArgumentLength: ULONG;
    let mut ResultLength: ULONG;
    let Argument: PVOID;
    let mut ResultPointer: PVOID;

    ArgumentLength = core::mem::size_of::<LOADMENU_CALLBACK_ARGUMENTS>() as ULONG;

    ArgumentLength += (*pMenuName).Length as ULONG + core::mem::size_of::<u16>() as ULONG;

    Argument = IntCbAllocateMemory(ArgumentLength);
    if Argument.is_null() {
        ERR!("LoadMenu callback failed: out of memory\n");
        return 0 as HMENU;
    }
    Common = Argument as PLOADMENU_CALLBACK_ARGUMENTS;

    (*Common).hModule = hModule;
    if (*pMenuName).Length != 0 {
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*Common).MenuName) as PVOID,
            (*pMenuName).Buffer as PVOID,
            (*pMenuName).Length as SIZE_T,
        );
    } else {
        (*Common).InterSource = (*pMenuName).Buffer;
    }

    ResultPointer = core::ptr::null_mut();
    ResultLength = core::mem::size_of::<LRESULT>() as ULONG;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_LOADMENU,
        Argument,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    if NT_SUCCESS(Status) {
        Result = *(ResultPointer as *mut LRESULT);
    } else {
        Result = 0;
    }

    IntCbFreeMemory(Argument);

    Result as HMENU
}
#[no_mangle]
pub unsafe extern "system" fn co_IntClientThreadSetup() -> NTSTATUS {
    let Status: NTSTATUS;
    let ArgumentLength: ULONG;
    let mut ResultLength: ULONG;
    let Argument: PVOID;
    let mut ResultPointer: PVOID;

    ArgumentLength = 0;
    ResultLength = 0;
    Argument = core::ptr::null_mut();
    ResultPointer = core::ptr::null_mut();

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_CLIENTTHREADSTARTUP as ULONG,
        Argument,
        ArgumentLength,
        core::ptr::addr_of_mut!(ResultPointer),
        core::ptr::addr_of_mut!(ResultLength),
    );

    UserEnterCo();

    Status
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntCopyImage(
    hnd: HANDLE,
    type_: UINT,
    desiredx: INT,
    desiredy: INT,
    flags: UINT,
) -> HANDLE {
    co_IntCopyImage_impl(hnd, type_, desiredx, desiredy, flags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntCopyImage(
    hnd: HANDLE,
    type_: UINT,
    desiredx: INT,
    desiredy: INT,
    flags: UINT,
) -> HANDLE {
    co_IntCopyImage_impl(hnd, type_, desiredx, desiredy, flags)
}

unsafe fn co_IntCopyImage_impl(
    hnd: HANDLE,
    type_: UINT,
    desiredx: INT,
    desiredy: INT,
    flags: UINT,
) -> HANDLE {
    let Handle: HANDLE;
    let Status: NTSTATUS;
    let mut ArgumentLength: ULONG;
    let mut ResultLength: ULONG;
    let Argument: PVOID;
    let mut ResultPointer: PVOID;
    let Common: PCOPYIMAGE_CALLBACK_ARGUMENTS;

    ArgumentLength = 0;
    ResultLength = 0;
    ResultPointer = core::ptr::null_mut();

    ArgumentLength = core::mem::size_of::<COPYIMAGE_CALLBACK_ARGUMENTS>() as ULONG;

    Argument = IntCbAllocateMemory(ArgumentLength);
    if Argument.is_null() {
        return core::ptr::null_mut();
    }
    Common = Argument as PCOPYIMAGE_CALLBACK_ARGUMENTS;

    (*Common).hImage = hnd;
    (*Common).uType = type_;
    (*Common).cxDesired = desiredx;
    (*Common).cyDesired = desiredy;
    (*Common).fuFlags = flags;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_COPYIMAGE,
        Argument,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    if NT_SUCCESS(Status) {
        Handle = *(ResultPointer as *mut HANDLE);
    } else {
        Handle = core::ptr::null_mut();
    }

    IntCbFreeMemory(Argument);

    Handle
}
#[no_mangle]
pub unsafe extern "system" fn co_IntGetCharsetInfo(Locale: LCID, pCs: PCHARSETINFO) -> BOOL {
    let mut Status: NTSTATUS;
    let ArgumentLength: ULONG;
    let mut ResultLength: ULONG;
    let Argument: PVOID;
    let mut ResultPointer: PVOID;
    let Common: PGET_CHARSET_INFO;

    ArgumentLength = core::mem::size_of::<GET_CHARSET_INFO>() as ULONG;

    Argument = IntCbAllocateMemory(ArgumentLength);
    if Argument.is_null() {
        return 0;
    }
    Common = Argument as PGET_CHARSET_INFO;

    (*Common).Locale = Locale;

    ResultPointer = core::ptr::null_mut();
    ResultLength = ArgumentLength;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_GETCHARSETINFO,
        Argument,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    if NT_SUCCESS(Status) {
        core::ptr::copy(ResultPointer, Argument, ArgumentLength as usize);
    }

    UserEnterCo();

    RtlCopyMemory(
        pCs as PVOID,
        core::ptr::addr_of_mut!((*Common).Cs) as PVOID,
        core::mem::size_of::<CHARSETINFO>() as SIZE_T,
    );

    IntCbFreeMemory(Argument);

    if !NT_SUCCESS(Status) {
        return 0;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetWndIcons() -> BOOL {
    co_IntSetWndIcons_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetWndIcons() -> BOOL {
    co_IntSetWndIcons_impl()
}

unsafe fn co_IntSetWndIcons_impl() -> BOOL {
    let Status: NTSTATUS;
    let ArgumentLength: ULONG;
    let mut ResultLength: ULONG;
    let Argument: PVOID;
    let mut ResultPointer: PVOID;
    let Common: PSETWNDICONS_CALLBACK_ARGUMENTS;

    ResultPointer = core::ptr::null_mut();
    ArgumentLength = core::mem::size_of::<SETWNDICONS_CALLBACK_ARGUMENTS>() as ULONG;
    ResultLength = ArgumentLength;

    Argument = IntCbAllocateMemory(ArgumentLength);
    if Argument.is_null() {
        return 0;
    }
    Common = Argument as PSETWNDICONS_CALLBACK_ARGUMENTS;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_SETWNDICONS,
        Argument,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    if !NT_SUCCESS(Status) {
        IntCbFreeMemory(Argument);
        return 0;
    }

    core::ptr::copy(ResultPointer, Common as PVOID, ArgumentLength as usize);
    (*gpsi_per_user()).hIconSmWindows = (*Common).hIconSmWindows;
    (*gpsi_per_user()).hIconWindows = (*Common).hIconWindows;

    IntLoadSystenIcons((*Common).hIconSample, OIC_SAMPLE as DWORD);
    IntLoadSystenIcons((*Common).hIconHand, OIC_HAND as DWORD);
    IntLoadSystenIcons((*Common).hIconQuestion, OIC_QUES as DWORD);
    IntLoadSystenIcons((*Common).hIconBang, OIC_BANG as DWORD);
    IntLoadSystenIcons((*Common).hIconNote, OIC_NOTE as DWORD);
    IntLoadSystenIcons((*gpsi_per_user()).hIconWindows, OIC_WINLOGO as DWORD);
    IntLoadSystenIcons((*gpsi_per_user()).hIconSmWindows, OIC_INTERNAL_WINSMALL as DWORD);

    IntCbFreeMemory(Argument);

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntDeliverUserAPC() {
    co_IntDeliverUserAPC_impl();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntDeliverUserAPC() {
    co_IntDeliverUserAPC_impl();
}

unsafe fn co_IntDeliverUserAPC_impl() {
    let mut ResultLength: ULONG = 0;
    let mut ResultPointer: PVOID = core::ptr::null_mut();
    let Status: NTSTATUS;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_DELIVERUSERAPC,
        core::ptr::null_mut(),
        0,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    if !NT_SUCCESS(Status) {
        ERR!("Delivering User APC callback failed!\n");
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetupOBM() {
    co_IntSetupOBM_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetupOBM() {
    co_IntSetupOBM_impl()
}

unsafe fn co_IntSetupOBM_impl() {
    let Status: NTSTATUS;
    let ArgumentLength: ULONG;
    let mut ResultLength: ULONG;
    let Argument: PVOID;
    let mut ResultPointer: PVOID;
    let Common: PSETOBM_CALLBACK_ARGUMENTS;

    ResultPointer = core::ptr::null_mut();
    ArgumentLength = core::mem::size_of::<SETOBM_CALLBACK_ARGUMENTS>() as ULONG;
    ResultLength = ArgumentLength;

    Argument = IntCbAllocateMemory(ArgumentLength);
    if Argument.is_null() {
        return;
    }
    Common = Argument as PSETOBM_CALLBACK_ARGUMENTS;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_SETOBM,
        Argument,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    if !NT_SUCCESS(Status) {
        IntCbFreeMemory(Argument);
        return;
    }

    core::ptr::copy(ResultPointer, Common as PVOID, ArgumentLength as usize);
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*gpsi_per_user()).oembmi) as PVOID,
        core::ptr::addr_of!((*Common).oembmi) as *const _ as PVOID,
        core::mem::size_of_val(&(*gpsi_per_user()).oembmi) as SIZE_T,
    );

    IntCbFreeMemory(Argument);
}

#[no_mangle]
pub unsafe extern "system" fn co_UserCBClientPrinterThunk(
    pkt: PVOID,
    InSize: INT,
    pvOutData: PVOID,
    mut OutSize: INT,
) -> LRESULT {
    let Status: NTSTATUS;
    let mut ResultPointer: PVOID = core::ptr::null_mut();

    Status = KeUserModeCallback(
        USER32_CALLBACK_UMPD as ULONG,
        pkt,
        InSize as ULONG,
        &mut ResultPointer,
        &mut OutSize as *mut INT as PULONG,
    );

    if !NT_SUCCESS(Status) {
        return 1;
    }

    if OutSize != 0 {
        core::ptr::copy(ResultPointer, pvOutData, OutSize as usize);
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn co_IntImmProcessKey(
    hWnd: HWND,
    hKL: HKL,
    vKey: UINT,
    lParam: LPARAM,
    dwHotKeyID: DWORD,
) -> DWORD {
    let mut ret: DWORD = 0;
    let Status: NTSTATUS;
    let mut ResultLength: ULONG = core::mem::size_of::<DWORD>() as ULONG;
    let mut ResultPointer: PVOID = core::ptr::null_mut();
    let mut Common: IMMPROCESSKEY_CALLBACK_ARGUMENTS = IMMPROCESSKEY_CALLBACK_ARGUMENTS {
        hWnd,
        hKL,
        vKey,
        lParam,
        dwHotKeyID,
    };

    UserLeaveCo();
    Status = KeUserModeCallback(
        USER32_CALLBACK_IMMPROCESSKEY,
        core::ptr::addr_of_mut!(Common) as PVOID,
        core::mem::size_of::<IMMPROCESSKEY_CALLBACK_ARGUMENTS>() as ULONG,
        &mut ResultPointer,
        &mut ResultLength,
    );
    UserEnterCo();

    if NT_SUCCESS(Status) {
        ret = *(ResultPointer as LPDWORD);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn co_ClientImmLoadLayout(
    hKL: HKL,
    pImeInfoEx: PIMEINFOEX,
) -> i32 {
    let mut ret: i32 = 0;
    let mut Status: i32;
    let mut Common: IMMLOADLAYOUT_CALLBACK_ARGUMENTS = core::mem::zeroed();
    Common.hKL = hKL;
    let mut ResultLength: u32 = core::mem::size_of::<IMMLOADLAYOUT_CALLBACK_OUTPUT>() as u32;
    let mut ResultPointer: PIMMLOADLAYOUT_CALLBACK_OUTPUT = core::ptr::null_mut();

    RtlZeroMemory(pImeInfoEx as *mut _, core::mem::size_of::<IMEINFOEX>() as SIZE_T);

    UserLeaveCo();
    Status = KeUserModeCallback(
        USER32_CALLBACK_IMMLOADLAYOUT,
        core::ptr::addr_of_mut!(Common) as *mut _,
        core::mem::size_of::<IMMLOADLAYOUT_CALLBACK_ARGUMENTS>() as u32,
        core::ptr::addr_of_mut!(ResultPointer) as *mut *mut _,
        &mut ResultLength,
    );
    UserEnterCo();

    if !NT_SUCCESS(Status)
        || ResultPointer.is_null()
        || ResultLength != core::mem::size_of::<IMMLOADLAYOUT_CALLBACK_OUTPUT>() as u32
    {
        ERR!(
            "0x%lX, %p, %lu\n",
            Status,
            ResultPointer,
            ResultLength,
        );
        return 0;
    }

    Status = SafeProbeForRead(ResultPointer as *mut _, ResultLength as _, 1);
    if !NT_SUCCESS(Status) {
        ret = 0;
    } else {
        // Probe is validate-only; the C had probe AND both loads in one _SEH2_TRY.
        // ResultPointer is a user-mode buffer, so BOTH the flag and the struct it
        // guards have to come across through the fault-tolerant copy.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(ret) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*ResultPointer).ret) as *const core::ffi::c_void,
            core::mem::size_of::<i32>(),
        ) {
            ret = 0;
        }
        if ret != 0
            && !crate::copy::capture_from_user(
                pImeInfoEx as *mut core::ffi::c_void,
                core::ptr::addr_of!((*ResultPointer).iiex) as *const core::ffi::c_void,
                core::mem::size_of::<IMEINFOEX>(),
            )
        {
            ret = 0;
        }
    }

    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Callback to usermode support
//  * FILE:             win32ss/user/ntuser/callback.c
//  * PROGRAMER:        Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *                   Thomas Weidenmueller (w3seek@users.sourceforge.net)
//  * NOTES:            Please use the Callback Memory Management functions for
//  *                   callbacks to make sure, the memory is freed on thread
//  *                   termination!
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
// DBG_DEFAULT_CHANNEL(UserCallback);
//
//
// /* __stdcall MEMORY MANAGEMENT ************************************************/
//
// typedef struct _INT_CALLBACK_HEADER
// {
//    /* List entry in the THREADINFO structure */
//    LIST_ENTRY ListEntry;
// }
// INT_CALLBACK_HEADER, *PINT_CALLBACK_HEADER;
//
// PVOID FASTCALL
// IntCbAllocateMemory(ULONG Size)
// {
//    PINT_CALLBACK_HEADER Mem;
//    PTHREADINFO W32Thread;
//
//    if(!(Mem = ExAllocatePoolWithTag(PagedPool, Size + sizeof(INT_CALLBACK_HEADER),
//                                     USERTAG_CALLBACK)))
//    {
//       return NULL;
//    }
//
//    RtlZeroMemory(Mem, Size + sizeof(INT_CALLBACK_HEADER));
//    W32Thread = PsGetCurrentThreadWin32Thread();
//    ASSERT(W32Thread);
//
//    /* Insert the callback memory into the thread's callback list */
//
//    InsertTailList(&W32Thread->W32CallbackListHead, &Mem->ListEntry);
//
//    return (Mem + 1);
// }
//
// VOID FASTCALL
// IntCbFreeMemory(PVOID Data)
// {
//    PINT_CALLBACK_HEADER Mem;
//    PTHREADINFO W32Thread;
//
//    W32Thread = PsGetCurrentThreadWin32Thread();
//    ASSERT(W32Thread);
//
//    if (W32Thread->TIF_flags & TIF_INCLEANUP)
//    {
//       ERR("CbFM Thread is already in cleanup\n");
//       return;
//    }
//
//    ASSERT(Data);
//
//    Mem = ((PINT_CALLBACK_HEADER)Data - 1);
//
//    /* Remove the memory block from the thread's callback list */
//    RemoveEntryList(&Mem->ListEntry);
//
//    /* Free memory */
//    ExFreePoolWithTag(Mem, USERTAG_CALLBACK);
// }
//
// VOID FASTCALL
// IntCleanupThreadCallbacks(PTHREADINFO W32Thread)
// {
//    PLIST_ENTRY CurrentEntry;
//    PINT_CALLBACK_HEADER Mem;
//
//    while (!IsListEmpty(&W32Thread->W32CallbackListHead))
//    {
//       CurrentEntry = RemoveHeadList(&W32Thread->W32CallbackListHead);
//       Mem = CONTAINING_RECORD(CurrentEntry, INT_CALLBACK_HEADER,
//                               ListEntry);
//
//       /* Free memory */
//       ExFreePoolWithTag(Mem, USERTAG_CALLBACK);
//    }
// }
//
// //
// // Pass the Current Window handle and pointer to the Client Callback.
// // This will help user space programs speed up read access with the window object.
// //
// static VOID
// IntSetTebWndCallback (HWND * hWnd, PWND * pWnd, PVOID * pActCtx)
// {
//   HWND hWndS = *hWnd;
//   PWND Window = UserGetWindowObject(*hWnd);
//   PCLIENTINFO ClientInfo = GetWin32ClientInfo();
//
//   *hWnd = ClientInfo->CallbackWnd.hWnd;
//   *pWnd = ClientInfo->CallbackWnd.pWnd;
//   *pActCtx = ClientInfo->CallbackWnd.pActCtx;
//
//   if (Window)
//   {
//      ClientInfo->CallbackWnd.hWnd = hWndS;
//      ClientInfo->CallbackWnd.pWnd = DesktopHeapAddressToUser(Window);
//      ClientInfo->CallbackWnd.pActCtx = Window->pActCtx;
//   }
//   else //// What if Dispatching WM_SYS/TIMER with NULL window? Fix AbiWord Crash when sizing.
//   {
//      ClientInfo->CallbackWnd.hWnd = hWndS;
//      ClientInfo->CallbackWnd.pWnd = Window;
//      ClientInfo->CallbackWnd.pActCtx = 0;
//   }
// }
//
// static VOID
// IntRestoreTebWndCallback (HWND hWnd, PWND pWnd, PVOID pActCtx)
// {
//   PCLIENTINFO ClientInfo = GetWin32ClientInfo();
//
//   ClientInfo->CallbackWnd.hWnd = hWnd;
//   ClientInfo->CallbackWnd.pWnd = pWnd;
//   ClientInfo->CallbackWnd.pActCtx = pActCtx;
// }
//
// /* FUNCTIONS *****************************************************************/
//
// /* Calls ClientLoadLibrary in user32 */
// BOOL
// NTAPI
// co_IntClientLoadLibrary(PUNICODE_STRING pstrLibName,
//                         PUNICODE_STRING pstrInitFunc,
//                         BOOL Unload,
//                         BOOL ApiHook)
// {
//    PVOID ResultPointer;
//    ULONG ResultLength;
//    ULONG ArgumentLength;
//    PCLIENT_LOAD_LIBRARY_ARGUMENTS pArguments;
//    NTSTATUS Status;
//    BOOL bResult;
//    ULONG_PTR pLibNameBuffer = 0, pInitFuncBuffer = 0;
//
//    /* Do not allow the desktop thread to do callback to user mode */
//    ASSERT(PsGetCurrentThreadWin32Thread() != gptiDesktopThread);
//
//    TRACE("co_IntClientLoadLibrary: %S, %S, %d, %d\n", pstrLibName->Buffer, pstrLibName->Buffer, Unload, ApiHook);
//
//    /* Calculate the size of the argument */
//    ArgumentLength = sizeof(CLIENT_LOAD_LIBRARY_ARGUMENTS);
//    if(pstrLibName)
//    {
//        pLibNameBuffer = ArgumentLength;
//        ArgumentLength += pstrLibName->Length + sizeof(WCHAR);
//    }
//    if(pstrInitFunc)
//    {
//        pInitFuncBuffer = ArgumentLength;
//        ArgumentLength += pstrInitFunc->Length + sizeof(WCHAR);
//    }
//
//    /* Allocate the argument */
//    pArguments = IntCbAllocateMemory(ArgumentLength);
//    if(pArguments == NULL)
//    {
//        return FALSE;
//    }
//
//    /* Fill the argument */
//    pArguments->Unload = Unload;
//    pArguments->ApiHook = ApiHook;
//    if(pstrLibName)
//    {
//        /* Copy the string to the callback memory */
//        pLibNameBuffer += (ULONG_PTR)pArguments;
//        pArguments->strLibraryName.Buffer = (PWCHAR)pLibNameBuffer;
//        pArguments->strLibraryName.MaximumLength = pstrLibName->Length + sizeof(WCHAR);
//        RtlCopyUnicodeString(&pArguments->strLibraryName, pstrLibName);
//
//        /* Fix argument pointer to be relative to the argument */
//        pLibNameBuffer -= (ULONG_PTR)pArguments;
//        pArguments->strLibraryName.Buffer = (PWCHAR)(pLibNameBuffer);
//    }
//
//    if(pstrInitFunc)
//    {
//        /* Copy the strings to the callback memory */
//        pInitFuncBuffer += (ULONG_PTR)pArguments;
//        pArguments->strInitFuncName.Buffer = (PWCHAR)pInitFuncBuffer;
//        pArguments->strInitFuncName.MaximumLength = pstrInitFunc->Length + sizeof(WCHAR);
//        RtlCopyUnicodeString(&pArguments->strInitFuncName, pstrInitFunc);
//
//        /* Fix argument pointers to be relative to the argument */
//        pInitFuncBuffer -= (ULONG_PTR)pArguments;
//        pArguments->strInitFuncName.Buffer = (PWCHAR)(pInitFuncBuffer);
//    }
//
//    /* Do the callback */
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_CLIENTLOADLIBRARY,
//                                pArguments,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    /* Free the argument */
//    IntCbFreeMemory(pArguments);
//
//    if(!NT_SUCCESS(Status))
//    {
//        return FALSE;
//    }
//
//    Status = SafeProbeForRead(ResultPointer, sizeof(HMODULE), 1);
//    if (!NT_SUCCESS(Status))
//    {
//        bResult = FALSE;
//    }
//    else
//    {
//        /* Probe and copy the usermode result data */
//
//        bResult = *(BOOL*)ResultPointer;
//    }
//
//    return bResult;
// }
//
// VOID __stdcall
// co_IntCallSentMessageCallback(SENDASYNCPROC CompletionCallback,
//                               HWND hWnd,
//                               UINT Msg,
//                               ULONG_PTR CompletionCallbackContext,
//                               LRESULT Result)
// {
//    SENDASYNCPROC_CALLBACK_ARGUMENTS Arguments;
//    PVOID ResultPointer, pActCtx;
//    PWND pWnd;
//    ULONG ResultLength;
//    NTSTATUS Status;
//
//    /* Do not allow the desktop thread to do callback to user mode */
//    ASSERT(PsGetCurrentThreadWin32Thread() != gptiDesktopThread);
//
//    Arguments.Callback = CompletionCallback;
//    Arguments.Wnd = hWnd;
//    Arguments.Msg = Msg;
//    Arguments.Context = CompletionCallbackContext;
//    Arguments.Result = Result;
//
//    IntSetTebWndCallback (&hWnd, &pWnd, &pActCtx);
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_SENDASYNCPROC,
//                                &Arguments,
//                                sizeof(SENDASYNCPROC_CALLBACK_ARGUMENTS),
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    IntRestoreTebWndCallback (hWnd, pWnd, pActCtx);
//
//    if (!NT_SUCCESS(Status))
//    {
//       ERR("KeUserModeCallback failed with %lx\n", Status);
//       return;
//    }
//    return;
// }
//
// LRESULT __stdcall
// co_IntCallWindowProc(WNDPROC Proc,
//                      BOOLEAN IsAnsiProc,
//                      HWND Wnd,
//                      UINT Message,
//                      WPARAM wParam,
//                      LPARAM lParam,
//                      INT lParamBufferSize)
// {
//    WINDOWPROC_CALLBACK_ARGUMENTS StackArguments = { 0 };
//    PWINDOWPROC_CALLBACK_ARGUMENTS Arguments;
//    NTSTATUS Status;
//    PVOID ResultPointer, pActCtx;
//    PWND pWnd;
//    ULONG ResultLength;
//    ULONG ArgumentLength;
//    LRESULT Result;
//
//    TRACE("co_IntCallWindowProc(Proc %p, IsAnsiProc: %s, Wnd %p, Message %u, wParam %Iu, lParam %Id, lParamBufferSize %d)\n",
//        Proc, IsAnsiProc ? "TRUE" : "FALSE", Wnd, Message, wParam, lParam, lParamBufferSize);
//
//    /* Do not allow the desktop thread to do callback to user mode */
//    ASSERT(PsGetCurrentThreadWin32Thread() != gptiDesktopThread);
//
//    if (lParamBufferSize != -1)
//    {
//       ArgumentLength = sizeof(WINDOWPROC_CALLBACK_ARGUMENTS) + lParamBufferSize;
//       Arguments = IntCbAllocateMemory(ArgumentLength);
//       if (NULL == Arguments)
//       {
//          ERR("Unable to allocate buffer for window proc callback\n");
//          return -1;
//       }
//       RtlMoveMemory((PVOID) ((char *) Arguments + sizeof(WINDOWPROC_CALLBACK_ARGUMENTS)),
//                     (PVOID) lParam, lParamBufferSize);
//    }
//    else
//    {
//       Arguments = &StackArguments;
//       ArgumentLength = sizeof(WINDOWPROC_CALLBACK_ARGUMENTS);
//    }
//    Arguments->Proc = Proc;
//    Arguments->IsAnsiProc = IsAnsiProc;
//    Arguments->Wnd = Wnd;
//    Arguments->Msg = Message;
//    Arguments->wParam = wParam;
//    Arguments->lParam = lParam;
//    Arguments->lParamBufferSize = lParamBufferSize;
//    ResultPointer = NULL;
//    ResultLength = ArgumentLength;
//
//    IntSetTebWndCallback (&Wnd, &pWnd, &pActCtx);
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_WINDOWPROC,
//                                Arguments,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//    if (!NT_SUCCESS(Status))
//    {
//       ERR("Error Callback to User space Status %lx Message %d\n",Status,Message);
//       UserEnterCo();
//       return 0;
//    }
//
//    _SEH2_TRY
//    {
//       /* Simulate old behaviour: copy into our local buffer */
//       RtlMoveMemory(Arguments, ResultPointer, ArgumentLength);
//    }
//    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//    {
//       ERR("Failed to copy result from user mode, Message %u lParam size %d!\n", Message, lParamBufferSize);
//       Status = _SEH2_GetExceptionCode();
//    }
//    _SEH2_END;
//
//    UserEnterCo();
//
//    IntRestoreTebWndCallback (Wnd, pWnd, pActCtx);
//
//    if (!NT_SUCCESS(Status))
//    {
//      ERR("Call to user mode failed! 0x%08lx\n",Status);
//       if (lParamBufferSize != -1)
//       {
//          IntCbFreeMemory(Arguments);
//       }
//       return -1;
//    }
//    Result = Arguments->Result;
//
//    if (lParamBufferSize != -1)
//    {
//       PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//       // Is this message being processed from inside kernel space?
//       BOOL InSendMessage = (pti->pcti->CTI_flags & CTI_INSENDMESSAGE);
//
//       TRACE("Copy lParam Message %u lParam %d!\n", Message, lParam);
//       switch (Message)
//       {
//           default:
//             TRACE("Don't copy lParam, Message %u Size %d lParam %d!\n", Message, lParamBufferSize, lParam);
//             break;
//           // Write back to user/kernel space. Also see g_MsgMemory.
//           case WM_CREATE:
//           case WM_GETMINMAXINFO:
//           case WM_GETTEXT:
//           case WM_NCCALCSIZE:
//           case WM_NCCREATE:
//           case WM_STYLECHANGING:
//           case WM_WINDOWPOSCHANGING:
//           case WM_SIZING:
//           case WM_MOVING:
//           case WM_MEASUREITEM:
//           case WM_NEXTMENU:
//             TRACE("Copy lParam, Message %u Size %d lParam %d!\n", Message, lParamBufferSize, lParam);
//             if (InSendMessage)
//                // Copy into kernel space.
//                RtlMoveMemory((PVOID) lParam,
//                              (PVOID) ((char *) Arguments + sizeof(WINDOWPROC_CALLBACK_ARGUMENTS)),
//                               lParamBufferSize);
//             else
//             {
//              _SEH2_TRY
//              { // Copy into user space.
//                RtlMoveMemory((PVOID) lParam,
//                              (PVOID) ((char *) Arguments + sizeof(WINDOWPROC_CALLBACK_ARGUMENTS)),
//                               lParamBufferSize);
//              }
//              _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//              {
//                 ERR("Failed to copy lParam to user space, Message %u!\n", Message);
//              }
//              _SEH2_END;
//             }
//             break;
//       }
//       IntCbFreeMemory(Arguments);
//    }
//
//    return Result;
// }
//
// HMENU __stdcall
// co_IntLoadSysMenuTemplate(VOID)
// {
//    LRESULT Result = 0;
//    NTSTATUS Status;
//    PVOID ResultPointer;
//    ULONG ResultLength;
//
//    /* Do not allow the desktop thread to do callback to user mode */
//    ASSERT(PsGetCurrentThreadWin32Thread() != gptiDesktopThread);
//
//    ResultPointer = NULL;
//    ResultLength = sizeof(LRESULT);
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_LOADSYSMENUTEMPLATE,
//                                &ResultPointer,
//                                0,
//                                &ResultPointer,
//                                &ResultLength);
//    if (NT_SUCCESS(Status))
//    {
//       /* Simulate old behaviour: copy into our local buffer */
//       Status = SafeProbeForRead(ResultPointer, sizeof(LRESULT), 1);
//       if (!NT_SUCCESS(Status))
//       {
//           Result = 0;
//       }
//       else
//       {
//           Result = *(LRESULT*)ResultPointer;
//       }
//    }
//
//    UserEnterCo();
//
//    return (HMENU)Result;
// }
//
// extern HCURSOR gDesktopCursor;
//
// BOOL __stdcall
// co_IntLoadDefaultCursors(VOID)
// {
//    NTSTATUS Status;
//    PVOID ResultPointer;
//    ULONG ResultLength;
//    BOOL DefaultCursor = TRUE;
//
//    /* Do not allow the desktop thread to do callback to user mode */
//    ASSERT(PsGetCurrentThreadWin32Thread() != gptiDesktopThread);
//
//    ResultPointer = NULL;
//    ResultLength = sizeof(HCURSOR);
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_LOADDEFAULTCURSORS,
//                                &DefaultCursor,
//                                sizeof(BOOL),
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    if (!NT_SUCCESS(Status))
//    {
//       return FALSE;
//    }
//
//    /* HACK: The desktop class doen't have a proper cursor yet, so set it here */
//     gDesktopCursor = *((HCURSOR*)ResultPointer);
//
//    return TRUE;
// }
//
// static INT iTheId = -2; // Set it out of range.
//
// LRESULT __stdcall
// co_IntCallHookProc(INT HookId,
//                    INT Code,
//                    WPARAM wParam,
//                    LPARAM lParam,
//                    HOOKPROC Proc,
//                    INT Mod,
//                    ULONG_PTR offPfn,
//                    BOOLEAN Ansi,
//                    PUNICODE_STRING ModuleName)
// {
//    ULONG ArgumentLength;
//    PVOID Argument = NULL;
//    LRESULT Result = 0;
//    NTSTATUS Status;
//    PVOID ResultPointer;
//    ULONG ResultLength;
//    PHOOKPROC_CALLBACK_ARGUMENTS Common;
//    CBT_CREATEWNDW *CbtCreateWnd = NULL;
//    PCHAR Extra;
//    PHOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS CbtCreatewndExtra = NULL;
//    PTHREADINFO pti;
//    PWND pWnd;
//    PMSG pMsg = NULL;
//    BOOL Hit = FALSE;
//    UINT lParamSize = 0;
//    CWPSTRUCT* pCWP = NULL;
//    CWPRETSTRUCT* pCWPR = NULL;
//
//    ASSERT(Proc);
//    /* Do not allow the desktop thread to do callback to user mode */
//    ASSERT(PsGetCurrentThreadWin32Thread() != gptiDesktopThread);
//
//    pti = PsGetCurrentThreadWin32Thread();
//    if (pti->TIF_flags & TIF_INCLEANUP)
//    {
//       ERR("Thread is in cleanup and trying to call hook %d\n", Code);
//       return 0;
//    }
//
//    ArgumentLength = sizeof(HOOKPROC_CALLBACK_ARGUMENTS);
//
//    switch(HookId)
//    {
//       case WH_CBT:
//          TRACE("WH_CBT: Code %d\n", Code);
//          switch(Code)
//          {
//             case HCBT_CREATEWND:
//                pWnd = UserGetWindowObject((HWND) wParam);
//                if (!pWnd)
//                {
//                   ERR("WH_CBT HCBT_CREATEWND wParam bad hWnd!\n");
//                   goto Fault_Exit;
//                }
//                TRACE("HCBT_CREATEWND AnsiCreator %s, AnsiHook %s\n", pWnd->state & WNDS_ANSICREATOR ? "True" : "False", Ansi ? "True" : "False");
//               // Due to KsStudio.exe, just pass the callers original pointers
//               // except class which point to kernel space if not an atom.
//               // Found by, Olaf Siejka
//                CbtCreateWnd = (CBT_CREATEWNDW *) lParam;
//                ArgumentLength += sizeof(HOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS);
//                break;
//
//             case HCBT_MOVESIZE:
//                ArgumentLength += sizeof(RECTL);
//                break;
//             case HCBT_ACTIVATE:
//                ArgumentLength += sizeof(CBTACTIVATESTRUCT);
//                break;
//             case HCBT_CLICKSKIPPED:
//                ArgumentLength += sizeof(MOUSEHOOKSTRUCT);
//                break;
// /*   ATM pass on */
//             case HCBT_KEYSKIPPED:
//             case HCBT_MINMAX:
//             case HCBT_SETFOCUS:
//             case HCBT_SYSCOMMAND:
// /*   These types pass through. */
//             case HCBT_DESTROYWND:
//             case HCBT_QS:
//                break;
//             default:
//                ERR("Trying to call unsupported CBT hook %d\n", Code);
//                goto Fault_Exit;
//          }
//          break;
//      case WH_KEYBOARD_LL:
//          ArgumentLength += sizeof(KBDLLHOOKSTRUCT);
//          break;
//      case WH_MOUSE_LL:
//          ArgumentLength += sizeof(MSLLHOOKSTRUCT);
//          break;
//      case WH_MOUSE:
//          ArgumentLength += sizeof(MOUSEHOOKSTRUCT);
//          break;
//      case WH_CALLWNDPROC:
//      {
//          pCWP = (CWPSTRUCT*) lParam;
//          ArgumentLength = sizeof(CWP_Struct);
//          if ( pCWP->message == WM_CREATE || pCWP->message == WM_NCCREATE )
//          {
//              lParamSize = sizeof(CREATESTRUCTW);
//          }
//          else
//              lParamSize = lParamMemorySize(pCWP->message, pCWP->wParam, pCWP->lParam);
//          ArgumentLength += lParamSize;
//          break;
//       }
//       case WH_CALLWNDPROCRET:
//       {
//          pCWPR = (CWPRETSTRUCT*) lParam;
//          ArgumentLength = sizeof(CWPR_Struct);
//          if ( pCWPR->message == WM_CREATE || pCWPR->message == WM_NCCREATE )
//          {
//              lParamSize = sizeof(CREATESTRUCTW);
//          }
//          else
//              lParamSize = lParamMemorySize(pCWPR->message, pCWPR->wParam, pCWPR->lParam);
//          ArgumentLength += lParamSize;
//          break;
//       }
//       case WH_MSGFILTER:
//       case WH_SYSMSGFILTER:
//       case WH_GETMESSAGE:
//          ArgumentLength += sizeof(MSG);
//          break;
//       case WH_FOREGROUNDIDLE:
//       case WH_KEYBOARD:
//       case WH_SHELL:
//          break;
//       default:
//          ERR("Trying to call unsupported window hook %d\n", HookId);
//          goto Fault_Exit;
//    }
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       ERR("HookProc callback %d failed: out of memory %d\n",HookId,ArgumentLength);
//       goto Fault_Exit;
//    }
//    Common = (PHOOKPROC_CALLBACK_ARGUMENTS) Argument;
//    Common->HookId = HookId;
//    Common->Code = Code;
//    Common->wParam = wParam;
//    Common->lParam = lParam;
//    Common->Proc = Proc;
//    Common->Mod = Mod;
//    Common->offPfn = offPfn;
//    Common->Ansi = Ansi;
//    Common->lParamSize = lParamSize;
//    if (ModuleName->Buffer && ModuleName->Length)
//    {
//       RtlCopyMemory(&Common->ModuleName, ModuleName->Buffer, ModuleName->Length);
//       // If ModuleName->Buffer NULL while in destroy,
//       //    this will make User32:Hook.c complain about not loading the library module.
//       // Fix symptom for CORE-10549.
//    }
//    Extra = (PCHAR) Common + sizeof(HOOKPROC_CALLBACK_ARGUMENTS);
//
//    switch(HookId)
//    {
//       case WH_CBT:
//          switch(Code)
//          { // Need to remember this is not the first time through! Call Next Hook?
//             case HCBT_CREATEWND:
//                CbtCreatewndExtra = (PHOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS) Extra;
//                RtlCopyMemory( &CbtCreatewndExtra->Cs, CbtCreateWnd->lpcs, sizeof(CREATESTRUCTW) );
//                CbtCreatewndExtra->WndInsertAfter = CbtCreateWnd->hwndInsertAfter;
//                CbtCreatewndExtra->Cs.lpszClass   = CbtCreateWnd->lpcs->lpszClass;
//                CbtCreatewndExtra->Cs.lpszName    = CbtCreateWnd->lpcs->lpszName;
//                Common->lParam = (LPARAM) (Extra - (PCHAR) Common);
//                //ERR("HCBT_CREATEWND: hWnd %p Csw %p Name %p Class %p\n", Common->wParam, CbtCreateWnd->lpcs, CbtCreateWnd->lpcs->lpszName, CbtCreateWnd->lpcs->lpszClass);
//                break;
//             case HCBT_CLICKSKIPPED:
//                RtlCopyMemory(Extra, (PVOID) lParam, sizeof(MOUSEHOOKSTRUCT));
//                Common->lParam = (LPARAM) (Extra - (PCHAR) Common);
//                break;
//             case HCBT_MOVESIZE:
//                RtlCopyMemory(Extra, (PVOID) lParam, sizeof(RECTL));
//                Common->lParam = (LPARAM) (Extra - (PCHAR) Common);
//                break;
//             case HCBT_ACTIVATE:
//                RtlCopyMemory(Extra, (PVOID) lParam, sizeof(CBTACTIVATESTRUCT));
//                Common->lParam = (LPARAM) (Extra - (PCHAR) Common);
//                break;
//          }
//          break;
//       case WH_KEYBOARD_LL:
//          RtlCopyMemory(Extra, (PVOID) lParam, sizeof(KBDLLHOOKSTRUCT));
//          Common->lParam = (LPARAM) (Extra - (PCHAR) Common);
//          break;
//       case WH_MOUSE_LL:
//          RtlCopyMemory(Extra, (PVOID) lParam, sizeof(MSLLHOOKSTRUCT));
//          Common->lParam = (LPARAM) (Extra - (PCHAR) Common);
//          break;
//       case WH_MOUSE:
//          RtlCopyMemory(Extra, (PVOID) lParam, sizeof(MOUSEHOOKSTRUCT));
//          Common->lParam = (LPARAM) (Extra - (PCHAR) Common);
//          break;
//       case WH_CALLWNDPROC:
//       {
//          PCWP_Struct pcwps = (PCWP_Struct)Common;
//          RtlCopyMemory( &pcwps->cwps, pCWP, sizeof(CWPSTRUCT));
//          /* For CALLWNDPROC and CALLWNDPROCRET, we must be wary of the fact that
//           * lParam could be a pointer to a buffer. This buffer must be exported
//           * to user space too */
//          if ( lParamSize )
//          {
//              RtlCopyMemory( &pcwps->Extra, (PVOID)pCWP->lParam, lParamSize );
//          }
//       }
//          break;
//       case WH_CALLWNDPROCRET:
//       {
//          PCWPR_Struct pcwprs = (PCWPR_Struct)Common;
//          RtlCopyMemory( &pcwprs->cwprs, pCWPR, sizeof(CWPRETSTRUCT));
//          if ( lParamSize )
//          {
//              RtlCopyMemory( &pcwprs->Extra, (PVOID)pCWPR->lParam, lParamSize );
//          }
//       }
//          break;
//       case WH_MSGFILTER:
//       case WH_SYSMSGFILTER:
//       case WH_GETMESSAGE:
//          pMsg = (PMSG)lParam;
//          RtlCopyMemory(Extra, (PVOID) pMsg, sizeof(MSG));
//          Common->lParam = (LPARAM) (Extra - (PCHAR) Common);
//          break;
//       case WH_FOREGROUNDIDLE:
//       case WH_KEYBOARD:
//       case WH_SHELL:
//          break;
//    }
//
//    ResultPointer = NULL;
//    ResultLength = ArgumentLength;
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_HOOKPROC,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    if (!NT_SUCCESS(Status))
//    {
//       if ( iTheId != HookId ) // Hook ID can change.
//       {
//           ERR("Failure to make Callback %d! Status 0x%x ArgumentLength %d\n",HookId,Status,ArgumentLength);
//           iTheId = HookId;
//       }
//       goto Fault_Exit;
//    }
//
//    if (ResultPointer)
//    {
//       _SEH2_TRY
//       {
//          /* Simulate old behaviour: copy into our local buffer */
//          RtlMoveMemory(Argument, ResultPointer, ArgumentLength);
//          Result = Common->Result;
//       }
//       _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//       {
//          Result = 0;
//          Hit = TRUE;
//       }
//       _SEH2_END;
//    }
//    else
//    {
//       ERR("ERROR: Hook %d Code %d ResultPointer 0x%p ResultLength %u\n",HookId,Code,ResultPointer,ResultLength);
//    }
//
//    /* Support write backs... SEH is in UserCallNextHookEx. */
//    switch (HookId)
//    {
//       case WH_CBT:
//       {
//          switch (Code)
//          {
//             case HCBT_CREATEWND:
//                if (CbtCreatewndExtra)
//                {/*
//                   The parameters could have been changed, include the coordinates
//                   and dimensions of the window. We copy it back.
//                  */
//                   CbtCreateWnd->hwndInsertAfter = CbtCreatewndExtra->WndInsertAfter;
//                   CbtCreateWnd->lpcs->x  = CbtCreatewndExtra->Cs.x;
//                   CbtCreateWnd->lpcs->y  = CbtCreatewndExtra->Cs.y;
//                   CbtCreateWnd->lpcs->cx = CbtCreatewndExtra->Cs.cx;
//                   CbtCreateWnd->lpcs->cy = CbtCreatewndExtra->Cs.cy;
//                }
//             break;
//             case HCBT_MOVESIZE:
//                if (Extra && lParam)
//                {
//                   RtlCopyMemory((PVOID) lParam, Extra, sizeof(RECTL));
//                }
//             break;
//          }
//       }
//       // "The GetMsgProc hook procedure can examine or modify the message."
//       case WH_GETMESSAGE:
//          if (pMsg)
//          {
//             RtlCopyMemory((PVOID) pMsg, Extra, sizeof(MSG));
//          }
//          break;
//    }
//
// Fault_Exit:
//    if (Hit)
//    {
//       ERR("Exception CallHookProc HookId %d Code %d\n",HookId,Code);
//    }
//    if (Argument) IntCbFreeMemory(Argument);
//
//    return Result;
// }
//
// //
// // Events are notifications w/o results.
// //
// LRESULT
// __stdcall
// co_IntCallEventProc(HWINEVENTHOOK hook,
//                            DWORD event,
//                              HWND hWnd,
//                          LONG idObject,
//                           LONG idChild,
//                    DWORD dwEventThread,
//                    DWORD dwmsEventTime,
//                      WINEVENTPROC Proc,
//                                INT Mod,
//                      ULONG_PTR offPfn)
// {
//    LRESULT Result = 0;
//    NTSTATUS Status;
//    PEVENTPROC_CALLBACK_ARGUMENTS Common;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//
//    ArgumentLength = sizeof(EVENTPROC_CALLBACK_ARGUMENTS);
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       ERR("EventProc callback failed: out of memory\n");
//       return 0;
//    }
//    Common = (PEVENTPROC_CALLBACK_ARGUMENTS) Argument;
//    Common->hook = hook;
//    Common->event = event;
//    Common->hwnd = hWnd;
//    Common->idObject = idObject;
//    Common->idChild = idChild;
//    Common->dwEventThread = dwEventThread;
//    Common->dwmsEventTime = dwmsEventTime;
//    Common->Proc = Proc;
//    Common->Mod = Mod;
//    Common->offPfn = offPfn;
//
//    ResultPointer = NULL;
//    ResultLength = sizeof(LRESULT);
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_EVENTPROC,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    IntCbFreeMemory(Argument);
//
//    if (!NT_SUCCESS(Status))
//    {
//       return 0;
//    }
//
//    return Result;
// }
//
// //
// // Callback Load Menu and results.
// //
// HMENU
// __stdcall
// co_IntCallLoadMenu( HINSTANCE hModule,
//                     PUNICODE_STRING pMenuName )
// {
//    LRESULT Result = 0;
//    NTSTATUS Status;
//    PLOADMENU_CALLBACK_ARGUMENTS Common;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//
//    ArgumentLength = sizeof(LOADMENU_CALLBACK_ARGUMENTS);
//
//    ArgumentLength += pMenuName->Length + sizeof(WCHAR);
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       ERR("LoadMenu callback failed: out of memory\n");
//       return 0;
//    }
//    Common = (PLOADMENU_CALLBACK_ARGUMENTS) Argument;
//
//    Common->hModule = hModule;
//    if (pMenuName->Length)
//       RtlCopyMemory(&Common->MenuName, pMenuName->Buffer, pMenuName->Length);
//    else
//       Common->InterSource = pMenuName->Buffer;
//
//    ResultPointer = NULL;
//    ResultLength = sizeof(LRESULT);
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_LOADMENU,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    if (NT_SUCCESS(Status))
//    {
//       Result = *(LRESULT*)ResultPointer;
//    }
//    else
//    {
//       Result = 0;
//    }
//
//    IntCbFreeMemory(Argument);
//
//    return (HMENU)Result;
// }
// /* --- pretranslation split: the remaining function groups live in callback.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
// /* callback.split2.c — #include fragment of callback.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original callback.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */
//
//
// NTSTATUS
// __stdcall
// co_IntClientThreadSetup(VOID)
// {
//    NTSTATUS Status;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//
//    /* Do not allow the desktop thread to do callback to user mode */
//    ASSERT(PsGetCurrentThreadWin32Thread() != gptiDesktopThread);
//
//    ArgumentLength = ResultLength = 0;
//    Argument = ResultPointer = NULL;
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_CLIENTTHREADSTARTUP,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    return Status;
// }
//
// HANDLE FASTCALL
// co_IntCopyImage(HANDLE hnd, UINT type, INT desiredx, INT desiredy, UINT flags)
// {
//    HANDLE Handle;
//    NTSTATUS Status;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//    PCOPYIMAGE_CALLBACK_ARGUMENTS Common;
//
//    ArgumentLength = ResultLength = 0;
//    Argument = ResultPointer = NULL;
//
//    ArgumentLength = sizeof(COPYIMAGE_CALLBACK_ARGUMENTS);
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       ERR("CopyImage callback failed: out of memory\n");
//       return 0;
//    }
//    Common = (PCOPYIMAGE_CALLBACK_ARGUMENTS) Argument;
//
//    Common->hImage = hnd;
//    Common->uType = type;
//    Common->cxDesired = desiredx;
//    Common->cyDesired = desiredy;
//    Common->fuFlags = flags;
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_COPYIMAGE,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//
//    UserEnterCo();
//
//    if (NT_SUCCESS(Status))
//    {
//       Handle = *(HANDLE*)ResultPointer;
//    }
//    else
//    {
//       ERR("CopyImage callback failed!\n");
//       Handle = NULL;
//    }
//
//    IntCbFreeMemory(Argument);
//
//    return Handle;
// }
//
// BOOL
// __stdcall
// co_IntGetCharsetInfo(LCID Locale, PCHARSETINFO pCs)
// {
//    NTSTATUS Status;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//    PGET_CHARSET_INFO Common;
//
//    ArgumentLength = sizeof(GET_CHARSET_INFO);
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       ERR("GetCharsetInfo callback failed: out of memory\n");
//       return 0;
//    }
//    Common = (PGET_CHARSET_INFO) Argument;
//
//    Common->Locale = Locale;
//
//    ResultPointer = NULL;
//    ResultLength = ArgumentLength;
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_GETCHARSETINFO,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//    if (NT_SUCCESS(Status))
//    {
//       _SEH2_TRY
//       {
//          /* Need to copy into our local buffer */
//          RtlMoveMemory(Argument, ResultPointer, ArgumentLength);
//       }
//       _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//       {
//          ERR("Failed to copy result from user mode!\n");
//          Status = _SEH2_GetExceptionCode();
//       }
//       _SEH2_END;
//    }
//
//    UserEnterCo();
//
//    RtlCopyMemory(pCs, &Common->Cs, sizeof(CHARSETINFO));
//
//    IntCbFreeMemory(Argument);
//
//    if (!NT_SUCCESS(Status))
//    {
//       ERR("GetCharsetInfo Failed!!\n");
//       return FALSE;
//    }
//
//    return TRUE;
// }
//
// BOOL FASTCALL
// co_IntSetWndIcons(VOID)
// {
//    NTSTATUS Status;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//    PSETWNDICONS_CALLBACK_ARGUMENTS Common;
//
//    ResultPointer = NULL;
//    ResultLength = ArgumentLength = sizeof(SETWNDICONS_CALLBACK_ARGUMENTS);
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       ERR("Set Window Icons callback failed: out of memory\n");
//       return FALSE;
//    }
//    Common = (PSETWNDICONS_CALLBACK_ARGUMENTS) Argument;
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_SETWNDICONS,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//
//    UserEnterCo();
//
//    if (!NT_SUCCESS(Status))
//    {
//       ERR("Set Window Icons callback failed!\n");
//       IntCbFreeMemory(Argument);
//       return FALSE;
//    }
//
//    RtlMoveMemory(Common, ResultPointer, ArgumentLength);
//    gpsi->hIconSmWindows = Common->hIconSmWindows;
//    gpsi->hIconWindows   = Common->hIconWindows;
//
//    IntLoadSystenIcons(Common->hIconSample,   OIC_SAMPLE);
//    IntLoadSystenIcons(Common->hIconHand,     OIC_HAND);
//    IntLoadSystenIcons(Common->hIconQuestion, OIC_QUES);
//    IntLoadSystenIcons(Common->hIconBang,     OIC_BANG);
//    IntLoadSystenIcons(Common->hIconNote,     OIC_NOTE);
//    IntLoadSystenIcons(gpsi->hIconWindows,    OIC_WINLOGO);
//    IntLoadSystenIcons(gpsi->hIconSmWindows,  OIC_INTERNAL_WINSMALL);
//
//    ERR("hIconSmWindows %p hIconWindows %p \n",gpsi->hIconSmWindows,gpsi->hIconWindows);
//
//    IntCbFreeMemory(Argument);
//
//    return TRUE;
// }
//
// VOID FASTCALL
// co_IntDeliverUserAPC(VOID)
// {
//    ULONG ResultLength;
//    PVOID ResultPointer;
//    NTSTATUS Status;
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_DELIVERUSERAPC,
//                                0,
//                                0,
//                                &ResultPointer,
//                                &ResultLength);
//
//
//    UserEnterCo();
//
//    if (!NT_SUCCESS(Status))
//    {
//       ERR("Delivering User APC callback failed!\n");
//    }
// }
//
// VOID FASTCALL
// co_IntSetupOBM(VOID)
// {
//    NTSTATUS Status;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//    PSETOBM_CALLBACK_ARGUMENTS Common;
//
//    ResultPointer = NULL;
//    ResultLength = ArgumentLength = sizeof(SETOBM_CALLBACK_ARGUMENTS);
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       ERR("Set Window Icons callback failed: out of memory\n");
//       return;
//    }
//    Common = (PSETOBM_CALLBACK_ARGUMENTS) Argument;
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_SETOBM,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//
//    UserEnterCo();
//
//    if (!NT_SUCCESS(Status))
//    {
//       ERR("Set Window Icons callback failed!\n");
//       IntCbFreeMemory(Argument);
//       return;
//    }
//
//    RtlMoveMemory(Common, ResultPointer, ArgumentLength);
//    RtlCopyMemory(gpsi->oembmi, Common->oembmi, sizeof(gpsi->oembmi));
//
//    IntCbFreeMemory(Argument);
// }
//
// //
// //  Called from Kernel GDI sides, no UserLeave/EnterCo required.
// //
// LRESULT
// __stdcall
// co_UserCBClientPrinterThunk( PVOID pkt, INT InSize, PVOID pvOutData, INT OutSize )
// {
//    NTSTATUS Status;
//    PVOID ResultPointer;
//
//    Status = KeUserModeCallback( USER32_CALLBACK_UMPD,
//                                 pkt,
//                                 InSize,
//                                &ResultPointer,
//                                (PULONG)&OutSize );
//
//
//    if (!NT_SUCCESS(Status))
//    {
//       ERR("User UMPD callback failed!\n");
//       return 1;
//    }
//
//    if (OutSize) RtlMoveMemory( pvOutData, ResultPointer, OutSize );
//
//    return 0;
// }
//
// // Win: ClientImmProcessKey
// DWORD
// __stdcall
// co_IntImmProcessKey(HWND hWnd, HKL hKL, UINT vKey, LPARAM lParam, DWORD dwHotKeyID)
// {
//     DWORD ret = 0;
//     NTSTATUS Status;
//     ULONG ResultLength = sizeof(DWORD);
//     PVOID ResultPointer = NULL;
//     IMMPROCESSKEY_CALLBACK_ARGUMENTS Common = { hWnd, hKL, vKey, lParam, dwHotKeyID };
//
//     UserLeaveCo();
//     Status = KeUserModeCallback(USER32_CALLBACK_IMMPROCESSKEY,
//                                 &Common,
//                                 sizeof(Common),
//                                 &ResultPointer,
//                                 &ResultLength);
//     UserEnterCo();
//
//     if (NT_SUCCESS(Status))
//         ret = *(LPDWORD)ResultPointer;
//
//     return ret;
// }
//
// /* Win: ClientImmLoadLayout */
// BOOL
// __stdcall
// co_ClientImmLoadLayout(
//      HKL hKL,
//      PIMEINFOEX pImeInfoEx)
// {
//     BOOL ret;
//     NTSTATUS Status;
//     IMMLOADLAYOUT_CALLBACK_ARGUMENTS Common = { hKL };
//     ULONG ResultLength = sizeof(IMMLOADLAYOUT_CALLBACK_OUTPUT);
//     PIMMLOADLAYOUT_CALLBACK_OUTPUT ResultPointer = NULL;
//
//     RtlZeroMemory(pImeInfoEx, sizeof(IMEINFOEX));
//
//     UserLeaveCo();
//     Status = KeUserModeCallback(USER32_CALLBACK_IMMLOADLAYOUT,
//                                 &Common,
//                                 sizeof(Common),
//                                 (PVOID*)&ResultPointer,
//                                 &ResultLength);
//     UserEnterCo();
//
//     if (!NT_SUCCESS(Status) || !ResultPointer ||
//         ResultLength != sizeof(IMMLOADLAYOUT_CALLBACK_OUTPUT))
//     {
//         ERR("0x%lX, %p, %lu\n", Status, ResultPointer, ResultLength);
//         return FALSE;
//     }
//
//     Status = SafeProbeForRead(ResultPointer, ResultLength, 1);
//     if (!NT_SUCCESS(Status))
//     {
//         ret = FALSE;
//     }
//     else
//     {
//         ret = ResultPointer->ret;
//         if (ret)
//             RtlCopyMemory(pImeInfoEx, &ResultPointer->iiex, sizeof(IMEINFOEX));
//     }
//
//     return ret;
// }
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
