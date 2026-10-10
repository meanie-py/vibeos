#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeUserModeCallback, ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::rtl::RtlCopyMemory;

// PagedPool POOL_TYPE variant.
const PagedPool: POOL_TYPE = 1;

// USER32_CALLBACK_DDE{POST,GET} are _USER32_CALLBACK enum variants (13/14).
const USER32_CALLBACK_DDEPOST: u32 = 13;
const USER32_CALLBACK_DDEGET: u32 = 14;

// WM_DDE_* message ids (sdk/include/psdk/dde.h).
const WM_DDE_INITIATE:  UINT = 0x3E0;
const WM_DDE_TERMINATE: UINT = 0x3E1;
const WM_DDE_UNADVISE:  UINT = 0x3E3;
const WM_DDE_ACK:       UINT = 0x3E4;
const WM_DDE_DATA:      UINT = 0x3E5;
const WM_DDE_REQUEST:   UINT = 0x3E6;
const WM_DDE_POKE:      UINT = 0x3E7;
const WM_DDE_EXECUTE:   UINT = 0x3E8;

// USERTAG pool tags. C multi-char literal is big-endian; LE bytes = reversed.
//   USERTAG_DDE  = 'EdsU' -> "UsdE"
//   USERTAG_DDE1 = '1dsU' -> "Usd1"
//   USERTAG_DDE5 = '5dsU' -> "Usd5"
const USERTAG_DDE:  u32 = u32::from_le_bytes(*b"UsdE");
const USERTAG_DDE1: u32 = u32::from_le_bytes(*b"Usd1");
const USERTAG_DDE5: u32 = u32::from_le_bytes(*b"Usd5");

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0). FORCEINLINE macro,
// no kernel symbol — reimplement inline (matches clipboard.rs/timer.rs).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}

// head_pti(pWnd) — bindgen flattened THRDESKHEAD (no `pti` field). pti sits at
// 2*sizeof(usize) (= 8 i386, 16 amd64/arm64): after HANDLE h + DWORD cLockObj
// plus the head's tail padding on 64-bit. NOT size_of::<HANDLE>()+size_of::<DWORD>()
// (=12 on 64-bit -> garbage pti -> AV).
#[inline(always)]
unsafe fn head_pti<T>(obj: *mut T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// ppi_W32Pid(ppi) — bindgen flattened _PROCESSINFO's W32PROCESS base into
// `__bindgen_padding_0`, hiding W32Pid. W32PROCESS is the first member (C++ base
// `: _W32PROCESS`), so reinterpret the PROCESSINFO pointer as its W32PROCESS base.
#[inline(always)]
unsafe fn ppi_W32Pid(ppi: PPROCESSINFO) -> DWORD {
    (*(ppi as *const _W32PROCESS)).W32Pid
}

// DDEPOSTGET_CALLBACK_ARGUMENTS has an anonymous embedded MSG (`INT Type; MSG; int size; ...`)
// which bindgen flattened to `__bindgen_padding_0`, losing hwnd/message/wParam/lParam.
// This repr(C) mirror restores the named MSG fields (layout-identical to the C struct).
#[repr(C)]
struct DDEPOSTGET_CB {
    Type: INT,
    msg: tagMSG,
    size: core::ffi::c_int,
    pvData: PVOID,
    buffer: [BYTE; 1],
}

// DDEDATA / DDEPOKE (sdk/include/psdk/dde.h): 2-byte bitfield, then `short cfFormat`,
// then `BYTE Value[1]`. Not in bindgen — local repr(C) mirrors.
#[repr(C)]
struct DDEDATA {
    _flags: u16,
    cfFormat: i16,
    Value: [BYTE; 1],
}
#[repr(C)]
struct DDEPOKE {
    _flags: u16,
    cfFormat: i16,
    Value: [BYTE; 1],
}

// ---- file-scope preamble (translated) ----
pub static mut gqosDefault: SECURITY_QUALITY_OF_SERVICE = SECURITY_QUALITY_OF_SERVICE {
    Length: core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as u32,
    ImpersonationLevel: _SECURITY_IMPERSONATION_LEVEL_SecurityImpersonation,
    ContextTrackingMode: SECURITY_STATIC_TRACKING as SECURITY_CONTEXT_TRACKING_MODE,
    EffectiveOnly: 1u8,
};

#[repr(C)]
pub struct _DDEIMP {
    pub qos: SECURITY_QUALITY_OF_SERVICE,
    pub ClientContext: SECURITY_CLIENT_CONTEXT,
    pub cRefInit: WORD,
    pub cRefConv: WORD,
}
pub type DDEIMP = _DDEIMP;
pub type PDDEIMP = *mut _DDEIMP;

#[repr(C)]
pub struct _DDE_DATA {
    pub lParam: LPARAM,
    pub cbSize: core::ffi::c_int,
    pub pvBuffer: PVOID,
}
pub type DDE_DATA = _DDE_DATA;
pub type PDDE_DATA = *mut _DDE_DATA;

#[repr(C)]
pub struct _DDE_PROP {
    pub spwnd: PWND,
    pub spwndPartner: PWND,
    pub pddei: PDDEIMP,
}
pub type DDE_PROP = _DDE_PROP;
pub type PDDE_PROP = *mut _DDE_PROP;

// UserEnterCo/UserLeaveCo are macros for UserEnterExclusive/UserLeave (ntuser.h),
// both FASTCALL (`@Name@0` on i386) — map to the real symbols with fastcall ABI.
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

unsafe extern "system" fn IntDDEPostCallback(
    pWnd: PWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: *mut LPARAM,
    Buffer: *mut PVOID,
) -> core::ffi::c_int {
    let Status: NTSTATUS;
    let mut ResultLength: ULONG;
    let ArgumentLength: ULONG;
    let Argument: PVOID;
    let mut ResultPointer: PVOID;
    let Common: *mut DDEPOSTGET_CB;
    let mut size: core::ffi::c_int = 0;
    ResultPointer = core::ptr::null_mut();
    ArgumentLength = core::mem::size_of::<DDEPOSTGET_CALLBACK_ARGUMENTS>() as ULONG;
    ResultLength = ArgumentLength;

    Argument = IntCbAllocateMemory(ArgumentLength);
    if Argument.is_null() {
        return 0;
    }

    Common = Argument as *mut DDEPOSTGET_CB;

    (*Common).pvData = core::ptr::null_mut();
    (*Common).size = 0;
    (*Common).msg.hwnd = UserHMGetHandle(pWnd);
    (*Common).msg.message = Msg;
    (*Common).msg.wParam = wParam;
    (*Common).msg.lParam = *lParam;

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_DDEPOST,
        Argument,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    if !NT_SUCCESS(Status) || ResultPointer.is_null() {
        ERR!("DDE Post callback failed!\n");
        IntCbFreeMemory(Argument);
        return 0;
    }

    RtlCopyMemory(Common as PVOID, ResultPointer, ArgumentLength as usize);

    size = (*Common).size;
    *lParam = (*Common).msg.lParam;
    *Buffer = (*Common).pvData;

    IntCbFreeMemory(Argument);

    if size != 0 { size } else { -1 }
}

#[no_mangle]
pub unsafe extern "system" fn IntDDEGetCallback(
    pWnd: PWND,
    pMsg: PMSG,
    Buffer: *mut core::ffi::c_void,
    size: core::ffi::c_int,
) -> i32 {
    let Status: i32;
    let ArgumentLength: u32;
    let mut ResultLength: u32;
    let Argument: *mut core::ffi::c_void;
    let mut ResultPointer: *mut core::ffi::c_void;
    let Common: *mut DDEPOSTGET_CB;

    ResultPointer = core::ptr::null_mut();
    ArgumentLength = core::mem::size_of::<DDEPOSTGET_CALLBACK_ARGUMENTS>() as u32 + size as u32;
    ResultLength = ArgumentLength;

    Argument = IntCbAllocateMemory(ArgumentLength);
    if Argument.is_null() {
        return 0;
    }

    Common = Argument as *mut DDEPOSTGET_CB;

    (*Common).size = size;
    (*Common).msg.hwnd = (*pMsg).hwnd;
    (*Common).msg.message = (*pMsg).message;
    (*Common).msg.wParam = (*pMsg).wParam;
    (*Common).msg.lParam = (*pMsg).lParam;

    if size != 0 && !Buffer.is_null() {
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*Common).buffer) as *mut core::ffi::c_void,
            Buffer,
            size as usize,
        );
    }

    UserLeaveCo();

    Status = KeUserModeCallback(
        USER32_CALLBACK_DDEGET,
        Argument,
        ArgumentLength,
        &mut ResultPointer,
        &mut ResultLength,
    );

    UserEnterCo();

    if !NT_SUCCESS(Status) || ResultPointer.is_null() {
        IntCbFreeMemory(Argument);
        return 0;
    }

    core::ptr::copy(
        ResultPointer,
        Common as *mut core::ffi::c_void,
        ArgumentLength as usize,
    );

    (*pMsg).lParam = (*Common).msg.lParam;

    IntCbFreeMemory(Argument);

    1
}
#[no_mangle]
pub unsafe extern "system" fn IntDdePostMessageHook(
    pWnd: PWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: *mut LPARAM,
    ExtraInfo: *mut LONG_PTR,
) -> BOOL {
    let pWndClient: PWND;
    let pddeData: PDDE_DATA;
    let mut size: core::ffi::c_int;
    let mut Object: HGDIOBJ = core::ptr::null_mut();
    let mut userBuf: PVOID = core::ptr::null_mut();
    let mut Buffer: PVOID = core::ptr::null_mut();
    let mut lp: LPARAM = *lParam;

    if (*head_pti(pWnd)).ppi != (*gptiCurrent).ppi {
        TRACE!(b"Posting long DDE 0x%x\n\0".as_ptr() as *const core::ffi::c_char, Msg);
        if Msg == WM_DDE_INITIATE {
            return FALSE as BOOL;
        }

        pWndClient = UserGetWindowObject(wParam as HWND);
        if pWndClient.is_null() {
            if Msg == WM_DDE_TERMINATE {
                TRACE!(b"DDE Posted WM_DDE_TERMINATE\n\0".as_ptr() as *const core::ffi::c_char);
                return TRUE as BOOL;
            }
            TRACE!(b"Invalid DDE Client Window handle\n\0".as_ptr() as *const core::ffi::c_char);
            return FALSE as BOOL;
        }

        if Msg == WM_DDE_REQUEST || Msg == WM_DDE_UNADVISE {
            return TRUE as BOOL;
        }

        if Msg == WM_DDE_TERMINATE {
            return TRUE as BOOL;
        }

        if Msg == WM_DDE_EXECUTE && *lParam == 0 {
            TRACE!(b"DDE Post EXECUTE lParam 0\n\0".as_ptr() as *const core::ffi::c_char);
            return FALSE as BOOL;
        }

        size = IntDDEPostCallback(pWnd, Msg, wParam, &mut lp, &mut userBuf);
        if size == 0 {
            ERR!("DDE Post Callback return 0 0x%x\n", Msg);
            return FALSE as BOOL;
        }

        if size == -1 {
            size = 0;
        } else {
            Buffer = ExAllocatePoolWithTag(PagedPool, size as _, USERTAG_DDE);
            if Buffer.is_null() {
                ERR!("Failed to allocate %i bytes.\n", size);
                return FALSE as BOOL;
            }
            RtlCopyMemory(Buffer, userBuf, size as usize);
        }

        TRACE!(b"DDE Post size %d 0x%x\n\0".as_ptr() as *const core::ffi::c_char, size, Msg);

        match Msg {
            WM_DDE_POKE => {
                let pddePoke = Buffer as *mut DDEPOKE;
                NT_ASSERT!(!pddePoke.is_null());
                match (*pddePoke).cfFormat as core::ffi::c_uint {
                    CF_BITMAP | CF_DIB | CF_PALETTE => {
                        RtlCopyMemory(
                            core::ptr::addr_of_mut!(Object) as PVOID,
                            core::ptr::addr_of!((*pddePoke).Value) as PVOID,
                            core::mem::size_of::<HGDIOBJ>() as usize,
                        );
                    }
                    _ => {}
                }
            }
            WM_DDE_DATA => {
                let pddeData2 = Buffer as *mut DDEDATA;
                NT_ASSERT!(!pddeData2.is_null());
                match (*pddeData2).cfFormat as core::ffi::c_uint {
                    CF_BITMAP | CF_DIB | CF_PALETTE => {
                        RtlCopyMemory(
                            core::ptr::addr_of_mut!(Object) as PVOID,
                            core::ptr::addr_of!((*pddeData2).Value) as PVOID,
                            core::mem::size_of::<HGDIOBJ>() as usize,
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if !Object.is_null() {
            GreSetObjectOwner(Object, ppi_W32Pid((*head_pti(pWnd)).ppi));
        }

        pddeData = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<DDE_DATA>() as _,
            USERTAG_DDE5,
        ) as PDDE_DATA;
        if pddeData.is_null() {
            ERR!("Failed to allocate DDE_DATA\n");
            ExFreePoolWithTag(Buffer, USERTAG_DDE);
            return FALSE as BOOL;
        }

        (*pddeData).cbSize = size;
        (*pddeData).pvBuffer = Buffer;
        (*pddeData).lParam = lp;

        TRACE!(b"DDE Post lParam c=%08lx\n\0".as_ptr() as *const core::ffi::c_char, lp);
        core::ptr::write_volatile(lParam, lp);

        core::ptr::write_volatile(ExtraInfo, pddeData as LONG_PTR);
    }
    TRUE as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn IntDdeGetMessageHook(pMsg: PMSG, ExtraInfo: LONG_PTR) -> BOOL {
    let pWnd: PWND;
    let pWndClient: PWND;
    let pddeData: PDDE_DATA;
    let pddeProp: PDDE_PROP;
    let Ret: BOOL;

    pWnd = UserGetWindowObject((*pMsg).hwnd);
    if pWnd.is_null() {
        return 1;
    }

    if (*pMsg).message == WM_DDE_TERMINATE {
        pddeProp = UserGetProp(pWnd, AtomDDETrack, 1) as PDDE_PROP;
        if !pddeProp.is_null() {
            pWndClient = UserGetWindowObject((*pMsg).wParam as HWND);
            if pWndClient.is_null() {
                // ERR
            }

            UserRemoveProp(pWnd, AtomDDETrack, 1);
            ExFreePoolWithTag(pddeProp as *mut core::ffi::c_void, USERTAG_DDE1);
        }
        return 1;
    }

    pddeData = ExtraInfo as PDDE_DATA;

    if !pddeData.is_null() {
        Ret = IntDDEGetCallback(pWnd, pMsg, (*pddeData).pvBuffer, (*pddeData).cbSize);
        if Ret == 0 {
            // ERR
        }

        if !(*pddeData).pvBuffer.is_null() {
            ExFreePoolWithTag((*pddeData).pvBuffer, USERTAG_DDE);
        }

        ExFreePoolWithTag(pddeData as *mut core::ffi::c_void, USERTAG_DDE5);

        return Ret;
    }
    1
}

// IntDdeSendMessageHook — declared `BOOL FASTCALL` in userfuncs.h, called cross-TU
// (`@IntDdeSendMessageHook@16` on i386). Export with the real fastcall decoration
// via a thin trampoline (extern "fastcall" fn-item type differs from extern "C").
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDdeSendMessageHook(pWnd: PWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    IntDdeSendMessageHook_impl(pWnd, Msg, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDdeSendMessageHook(pWnd: PWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    IntDdeSendMessageHook_impl(pWnd, Msg, wParam, lParam)
}

unsafe fn IntDdeSendMessageHook_impl(pWnd: PWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    let pWndServer: PWND;
    let pddeProp: PDDE_PROP;

    if (*head_pti(pWnd)).ppi != (*gptiCurrent).ppi {
        if Msg != WM_DDE_ACK {
            if Msg == WM_DDE_INITIATE {
                return TRUE as BOOL;
            }
            return FALSE as BOOL;
        }

        pWndServer = UserGetWindowObject(wParam as HWND);
        if pWndServer.is_null() {
            return FALSE as BOOL;
        }

        pddeProp = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<DDE_PROP>() as _, USERTAG_DDE1) as PDDE_PROP;
        if pddeProp.is_null() {
            return FALSE as BOOL;
        }

        (*pddeProp).spwnd = pWndServer;
        (*pddeProp).spwndPartner = pWnd;

        UserSetProp(pWndServer, AtomDDETrack, pddeProp as HANDLE, TRUE as BOOLEAN);
    }
    TRUE as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDdeGetQualityOfService(
    hwndClient: HWND,
    hWndServer: HWND,
    pqosPrev: PSECURITY_QUALITY_OF_SERVICE,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDdeSetQualityOfService(
    hwndClient: HWND,
    pqosNew: PSECURITY_QUALITY_OF_SERVICE,
    pqosPrev: PSECURITY_QUALITY_OF_SERVICE,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserImpersonateDdeClientWindow(
    hWndClient: HWND,
    hWndServer: HWND,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDdeInitialize(
    Unknown0: DWORD,
    Unknown1: DWORD,
    Unknown2: DWORD,
    Unknown3: DWORD,
    Unknown4: DWORD,
) -> DWORD {
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Dynamic Data Exchange
//  * FILE:             win32ss/user/ntuser/dde.c
//  * PROGRAMER:
//  */
//
// #include <win32k.h>
//
// #include <dde.h>
//
// DBG_DEFAULT_CHANNEL(UserMisc);
//
// //
// //  Default information used to support client impersonation.
// //
// SECURITY_QUALITY_OF_SERVICE gqosDefault = {sizeof(SECURITY_QUALITY_OF_SERVICE),SecurityImpersonation,SECURITY_STATIC_TRACKING,TRUE};
//
// typedef struct _DDEIMP
// {
//   SECURITY_QUALITY_OF_SERVICE qos;
//   SECURITY_CLIENT_CONTEXT ClientContext;
//   WORD cRefInit;
//   WORD cRefConv;
// } DDEIMP, *PDDEIMP;
//
// typedef struct _DDE_DATA
// {
//   LPARAM lParam;
//   int cbSize;
//   PVOID pvBuffer;
// } DDE_DATA, *PDDE_DATA;
//
// typedef struct _DDE_PROP
// {
//   PWND spwnd;
//   PWND spwndPartner;
//   PDDEIMP pddei;
// } DDE_PROP, *PDDE_PROP;
//
//
// //
// //  DDE Posting message callback to user side.
// //
// int
// __stdcall
// IntDDEPostCallback(
//    IN PWND pWnd,
//    IN UINT Msg,
//    IN WPARAM wParam,
//    IN OUT LPARAM *lParam,
//    IN OUT PVOID *Buffer)
// {
//    NTSTATUS Status;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//    PDDEPOSTGET_CALLBACK_ARGUMENTS Common;
//    int size = 0;
//    ResultPointer = NULL;
//    ResultLength = ArgumentLength = sizeof(DDEPOSTGET_CALLBACK_ARGUMENTS);
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       return FALSE;
//    }
//
//    Common = (PDDEPOSTGET_CALLBACK_ARGUMENTS) Argument;
//
//    Common->pvData  = 0;
//    Common->size    = 0;
//    Common->hwnd    = UserHMGetHandle(pWnd);
//    Common->message = Msg;
//    Common->wParam  = wParam;
//    Common->lParam  = *lParam;
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_DDEPOST,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    if (!NT_SUCCESS(Status) || ResultPointer == NULL )
//    {
//       ERR!("DDE Post callback failed!\n");
//       IntCbFreeMemory(Argument);
//       return 0;
//    }
//
//    RtlCopyMemory(Common, ResultPointer, ArgumentLength);
//
//    size    = Common->size;
//    *lParam = Common->lParam;
//    *Buffer = Common->pvData;
//
//    IntCbFreeMemory(Argument);
//
//    return size ? size : -1;
// }
//
// //
// //  DDE Get/Peek message callback to user side.
// //
// BOOL
// __stdcall
// IntDDEGetCallback(
//    IN PWND pWnd,
//    IN OUT PMSG pMsg,
//    IN PVOID Buffer,
//    IN int size)
// {
//    NTSTATUS Status;
//    ULONG ArgumentLength, ResultLength;
//    PVOID Argument, ResultPointer;
//    PDDEPOSTGET_CALLBACK_ARGUMENTS Common;
//
//    ResultPointer = NULL;
//    ResultLength = ArgumentLength = sizeof(DDEPOSTGET_CALLBACK_ARGUMENTS)+size;
//
//    Argument = IntCbAllocateMemory(ArgumentLength);
//    if (NULL == Argument)
//    {
//       return FALSE;
//    }
//
//    Common = (PDDEPOSTGET_CALLBACK_ARGUMENTS) Argument;
//
//    Common->size    = size;
//    Common->hwnd    = pMsg->hwnd;
//    Common->message = pMsg->message;
//    Common->wParam  = pMsg->wParam;
//    Common->lParam  = pMsg->lParam;
//
//    if (size && Buffer) RtlCopyMemory(&Common->buffer, Buffer, size);
//
//    UserLeaveCo();
//
//    Status = KeUserModeCallback(USER32_CALLBACK_DDEGET,
//                                Argument,
//                                ArgumentLength,
//                                &ResultPointer,
//                                &ResultLength);
//
//    UserEnterCo();
//
//    if (!NT_SUCCESS(Status) || ResultPointer == NULL )
//    {
//       ERR!("DDE Get callback failed!\n");
//       IntCbFreeMemory(Argument);
//       return FALSE;
//    }
//
//    RtlMoveMemory(Common, ResultPointer, ArgumentLength);
//
//    pMsg->lParam = Common->lParam;
//
//    IntCbFreeMemory(Argument);
//
//    return TRUE;
// }
//
// //
// //  DDE Post message hook, intercept DDE messages before going on to the target Processes Thread queue.
// //
// BOOL
// __stdcall
// IntDdePostMessageHook(
//    IN PWND pWnd,
//    IN UINT Msg,
//    IN WPARAM wParam,
//    IN OUT LPARAM *lParam,
//    IN OUT LONG_PTR *ExtraInfo)
// {
//    PWND pWndClient;
//    PDDE_DATA pddeData;
//    int size;
//    HGDIOBJ Object = NULL;
//    PVOID userBuf = NULL;
//    PVOID Buffer = NULL;
//    LPARAM lp = *lParam;
//
//    if (pWnd->head.pti->ppi != gptiCurrent->ppi)
//    {
//       TRACE!("Posting long DDE 0x%x\n",Msg);
//       // Initiate is sent only across borders.
//       if (Msg == WM_DDE_INITIATE)
//       {
//          return FALSE;
//       }
//
//       pWndClient = UserGetWindowObject((HWND)wParam);
//       if (pWndClient == NULL)
//       {
//          // This is terminating so post it.
//          if ( Msg == WM_DDE_TERMINATE)
//          {
//             TRACE!("DDE Posted WM_DDE_TERMINATE\n");
//             return TRUE;
//          }
//          TRACE!("Invalid DDE Client Window handle\n");
//          return FALSE;
//       }
//
//       if ( Msg == WM_DDE_REQUEST || Msg == WM_DDE_UNADVISE )
//       {
//          // Do not bother to callback after validation.
//          return TRUE;
//       }
//
//       if ( Msg == WM_DDE_TERMINATE )
//       {
//          //// FIXME Remove Stuff if any...
//
//          // Do not bother to callback.
//          return TRUE;
//       }
//
//       if ( Msg == WM_DDE_EXECUTE && *lParam == 0)
//       {
//          // Do not bother to do a callback.
//          TRACE!("DDE Post EXECUTE lParam 0\n");
//          return FALSE;
//       }
//
//       // Callback.
//       if ((size = IntDDEPostCallback(pWnd, Msg, wParam, &lp, &userBuf)) == 0)
//       {
//          ERR!("DDE Post Callback return 0 0x%x\n", Msg);
//          return FALSE;
//       }
//
//       // No error HACK.
//       if (size == -1)
//       {
//          size = 0;
//       }
//       else
//       {
//          // Set buffer with users data size.
//          Buffer = ExAllocatePoolWithTag(PagedPool, size, USERTAG_DDE);
//          if (Buffer == NULL)
//          {
//              ERR!("Failed to allocate %i bytes.\n", size);
//              return FALSE;
//          }
//          // No SEH? Yes, the user memory is freed after the Acknowledgment or at Termination.
//          RtlCopyMemory(Buffer, userBuf, size);
//       }
//
//       TRACE!("DDE Post size %d 0x%x\n",size, Msg);
//
//       switch(Msg)
//       {
//           case WM_DDE_POKE:
//           {
//               DDEPOKE *pddePoke = Buffer;
//               NT_ASSERT!(pddePoke != NULL);
//               switch(pddePoke->cfFormat)
//               {
//                  case CF_BITMAP:
//                  case CF_DIB:
//                  case CF_PALETTE:
//                     RtlCopyMemory(&Object, pddePoke->Value, sizeof(HGDIOBJ));
//                     break;
//                  default:
//                     break;
//               }
//               break;
//           }
//           case WM_DDE_DATA:
//           {
//               DDEDATA *pddeData2 = Buffer;
//               NT_ASSERT!(pddeData2 != NULL);
//               switch(pddeData2->cfFormat)
//               {
//                  case CF_BITMAP:
//                  case CF_DIB:
//                  case CF_PALETTE:
//                     RtlCopyMemory(&Object, pddeData2->Value, sizeof(HGDIOBJ));
//                     break;
//                  default:
//                     break;
//               }
//               break;
//           }
//           default:
//               break;
//       }
//
//       if (Object)
//       {
//          // Give gdi object to the other process.
//          GreSetObjectOwner(Object, pWnd->head.pti->ppi->W32Pid);
//       }
//
//       pddeData = ExAllocatePoolWithTag(PagedPool, sizeof(DDE_DATA), USERTAG_DDE5);
//       if (pddeData == NULL)
//       {
//          ERR!("Failed to allocate DDE_DATA\n");
//          ExFreePoolWithTag(Buffer, USERTAG_DDE);
//          return FALSE;
//       }
//
//       pddeData->cbSize       = size;
//       pddeData->pvBuffer     = Buffer;
//       pddeData->lParam       = lp;
//
//       TRACE!("DDE Post lParam c=%08lx\n",lp);
//       *lParam = lp;
//
//       // Attach this data packet to the user message.
//       *ExtraInfo = (LONG_PTR)pddeData;
//    }
//    return TRUE;
// }
//
// //
// //  DDE Get/Peek message hook, take preprocessed information and recombined it for the current Process Thread.
// //
// BOOL __stdcall
// IntDdeGetMessageHook(PMSG pMsg, LONG_PTR ExtraInfo)
// {
//    PWND pWnd, pWndClient;
//    PDDE_DATA pddeData;
//    PDDE_PROP pddeProp;
//    BOOL Ret;
//
//    pWnd = UserGetWindowObject(pMsg->hwnd);
//    if (pWnd == NULL)
//    {
//       ERR!("DDE Get Window is dead. %p\n", pMsg->hwnd);
//       return TRUE;
//    }
//
//    if (pMsg->message == WM_DDE_TERMINATE)
//    {
//       pddeProp = (PDDE_PROP)UserGetProp(pWnd, AtomDDETrack, TRUE);
//       if (pddeProp)
//       {
//          pWndClient = UserGetWindowObject((HWND)pMsg->wParam);
//          if (pWndClient == NULL)
//          {
//             ERR!("DDE Get Client WM_DDE_TERMINATE\n");
//          }
//
//          UserRemoveProp(pWnd, AtomDDETrack, TRUE);
//          ExFreePoolWithTag(pddeProp, USERTAG_DDE1);
//       }
//       return TRUE;
//    }
//
//    TRACE!("DDE Get Msg 0x%x\n",pMsg->message);
//
//    pddeData = (PDDE_DATA)ExtraInfo;
//
//    if ( pddeData )
//    {
//       TRACE!("DDE Get size %d lParam c=%08lx lp c=%08lx\n",pddeData->cbSize, pMsg->lParam, pddeData->lParam);
//
//       // Callback.
//       Ret = IntDDEGetCallback( pWnd, pMsg, pddeData->pvBuffer, pddeData->cbSize);
//       if (!Ret)
//       {
//          ERR!("DDE Get CB failed\n");
//       }
//
//       if (pddeData->pvBuffer) ExFreePoolWithTag(pddeData->pvBuffer, USERTAG_DDE);
//
//       ExFreePoolWithTag(pddeData, USERTAG_DDE5);
//
//       return Ret;
//    }
//    TRACE!("DDE Get No DDE Data found!\n");
//    return TRUE;
// }
//
// //
// //  DDE Send message hook, intercept DDE messages and associate them in a partnership with property.
// //
// BOOL FASTCALL
// IntDdeSendMessageHook(PWND pWnd, UINT Msg, WPARAM wParam, LPARAM lParam)
// {
//    PWND pWndServer;
//    PDDE_PROP pddeProp;
//
//    if (pWnd->head.pti->ppi != gptiCurrent->ppi)
//    {
//       TRACE!("Sending long DDE 0x%x\n",Msg);
//
//       // Allow only Acknowledge and Initiate to be sent across borders.
//       if (Msg != WM_DDE_ACK )
//       {
//          if (Msg == WM_DDE_INITIATE) return TRUE;
//          return FALSE;
//       }
//
//       TRACE!("Sending long WM_DDE_ACK\n");
//
//       pWndServer = UserGetWindowObject((HWND)wParam);
//       if (pWndServer == NULL)
//       {
//          ERR!("Invalid DDE Server Window handle\n");
//          return FALSE;
//       }
//
//       // Setup property so this conversation can be tracked.
//       pddeProp = ExAllocatePoolWithTag(PagedPool, sizeof(DDE_PROP), USERTAG_DDE1);
//       if (pddeProp == NULL)
//       {
//          ERR!("failed to allocate DDE_PROP\n");
//          return FALSE;
//       }
//
//       pddeProp->spwnd        = pWndServer;
//       pddeProp->spwndPartner = pWnd;
//
//       UserSetProp(pWndServer, AtomDDETrack, (HANDLE)pddeProp, TRUE);
//    }
//    return TRUE;
// }
//
//
// BOOL
// __stdcall
// NtUserDdeGetQualityOfService(
//    IN HWND hwndClient,
//    IN HWND hWndServer,
//    OUT PSECURITY_QUALITY_OF_SERVICE pqosPrev)
// {
//    STUB
//
//    return 0;
// }
//
// BOOL
// __stdcall
// NtUserDdeSetQualityOfService(
//    IN  HWND hwndClient,
//    IN  PSECURITY_QUALITY_OF_SERVICE pqosNew,
//    OUT PSECURITY_QUALITY_OF_SERVICE pqosPrev)
// {
//    STUB
//
//    return 0;
// }
//
// BOOL
// __stdcall
// NtUserImpersonateDdeClientWindow(
//    HWND hWndClient,
//    HWND hWndServer)
// {
//    STUB
//
//    return 0;
// }
//
// DWORD
// __stdcall
// NtUserDdeInitialize(
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
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
