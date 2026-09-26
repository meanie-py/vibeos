//! Rust translation of `win32ss/user/ntuser/accelerator.c` — accelerator tables.
//! [ntuser wave 4]

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentThreadWin32Thread, RtlMultiByteToUnicodeN, ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// SafeProbeForRead/Write are NTAPI (stdcall i386, `_Safe...@12`).
extern "system" {
    fn SafeProbeForRead(addr: *mut c_void, len: SIZE_T, align: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(addr: *mut c_void, len: SIZE_T, align: ULONG) -> NTSTATUS;
}
// KPROCESSOR_MODE is i8 (0 = KernelMode, 1 = UserMode) — see ntkrnl_sys::probe.
extern "system" {
    fn KeGetPreviousMode() -> i8;
}
// FASTCALL win32k exports.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn MENU_FindItem(pmenu: *mut PMENU, ppos: *mut u32, flag: u32) -> PVOID;
    fn co_IntSendMessage(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: isize) -> isize;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn MENU_FindItem(pmenu: *mut PMENU, ppos: *mut u32, flag: u32) -> PVOID;
    fn co_IntSendMessage(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: isize) -> isize;
}

// UserHMGetHandle(obj) = obj->head.h ; head.h is at object offset 0.
#[inline]
unsafe fn hm_get_handle<T>(obj: *mut T) -> HANDLE {
    *(obj as *const HANDLE)
}
#[inline]
unsafe fn push_entry_list(head: *mut SINGLE_LIST_ENTRY, entry: *mut SINGLE_LIST_ENTRY) {
    (*entry).Next = (*head).Next;
    (*head).Next = entry;
}
#[inline]
unsafe fn pop_entry_list(head: *mut SINGLE_LIST_ENTRY) -> *mut SINGLE_LIST_ENTRY {
    let first = (*head).Next;
    if !first.is_null() { (*head).Next = (*first).Next; }
    first
}
// UserRefObjectCo/UserDerefObjectCo (object.h static __inline).
#[inline]
unsafe fn user_ref_object_co(obj: PVOID, entry: *mut USER_REFERENCE_ENTRY) {
    let w32: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*entry).obj = obj;
    UserReferenceObject(obj);
    push_entry_list(core::ptr::addr_of_mut!((*w32).ReferencesList), core::ptr::addr_of_mut!((*entry).Entry));
}
#[inline]
unsafe fn user_deref_object_co(obj: PVOID) {
    let w32: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let _ = pop_entry_list(core::ptr::addr_of_mut!((*w32).ReferencesList));
    UserDereferenceObject(obj);
}
// UserGetMenuObject is FORCEINLINE (userfuncs.h) — no symbol.
#[inline]
unsafe fn user_get_menu_object(h_menu: HMENU) -> PMENU {
    let p = UserGetObject(gHandleTable, h_menu as PVOID, TYPE_MENU) as PMENU;
    if p.is_null() {
        EngSetLastError(ERROR_INVALID_MENU_HANDLE);
    }
    p
}

// ---- file-scope preamble (translated) ----
pub const FVIRT_TBL_END: u32 = 0x80;

pub const FVIRT_MASK: u32 = 0x7F;
const USERTAG_ACCEL: u32 = u32::from_be_bytes(*b"casU");
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetAccelObject(hAccel: HACCEL) -> PACCELERATOR_TABLE {
    UserGetAccelObjectImpl(hAccel)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetAccelObject(hAccel: HACCEL) -> PACCELERATOR_TABLE {
    UserGetAccelObjectImpl(hAccel)
}

unsafe fn UserGetAccelObjectImpl(hAccel: HACCEL) -> PACCELERATOR_TABLE {
    let Accel: PACCELERATOR_TABLE;

    if hAccel.is_null() {
        EngSetLastError(ERROR_INVALID_ACCEL_HANDLE);
        return core::ptr::null_mut();
    }

    Accel = UserGetObject(gHandleTable, hAccel as _, TYPE_ACCELTABLE) as PACCELERATOR_TABLE;
    if Accel.is_null() {
        EngSetLastError(ERROR_INVALID_ACCEL_HANDLE);
        return core::ptr::null_mut();
    }

    Accel
}


fastcall_fn! {
unsafe fn co_IntTranslateAccelerator(
    Window: PWND,
    pMsg: *const MSG,
    pAccel: *const ACCEL,
) -> u8 {
    let mut bFound: u8 = 0;
    let mut Mask: u32 = 0;
    let mut nPos: u32;
    let hWnd: HWND;
    let mut hMenu: HMENU;
    let mut hSubMenu: HMENU;
    let mut MenuObject: PMENU;

    hWnd = hm_get_handle(Window) as HWND;

    if (UserGetKeyState(VK_CONTROL as u32) as i32 & 0x8000) != 0 {
        Mask |= FCONTROL as u32;
    }
    if (UserGetKeyState(VK_MENU as u32) as i32 & 0x8000) != 0 {
        Mask |= FALT as u32;
    }
    if (UserGetKeyState(VK_SHIFT as u32) as i32 & 0x8000) != 0 {
        Mask |= FSHIFT as u32;
    }

    if ((*pAccel).fVirt as u32 & FVIRTKEY as u32) != 0 {
        if (*pMsg).message == WM_KEYDOWN || (*pMsg).message == WM_SYSKEYDOWN {
            if (*pMsg).wParam == (*pAccel).key as WPARAM
                && Mask == ((*pAccel).fVirt as u32 & (FSHIFT as u32 | FCONTROL as u32 | FALT as u32))
            {
                bFound = 1;
            }
        }
    } else {
        if (*pMsg).message == WM_CHAR || (*pMsg).message == WM_SYSCHAR {
            if (*pMsg).wParam == (*pAccel).key as WPARAM
                && (Mask & FALT as u32) == ((*pAccel).fVirt as u32 & FALT as u32)
            {
                bFound = 1;
            }
        }
    }

    if bFound == 0 {
        return 0;
    }

    hMenu = if ((*Window).style & WS_CHILD as u32) != 0 {
        core::ptr::null_mut()
    } else {
        (*Window).IDMenu as HMENU
    };
    hSubMenu = core::ptr::null_mut();
    MenuObject = user_get_menu_object(hMenu);
    nPos = (*pAccel).cmd as u32;
    if !MenuObject.is_null() {
        if !MENU_FindItem(&mut MenuObject, &mut nPos, MF_BYPOSITION as u32).is_null() {
            hSubMenu = hm_get_handle(MenuObject) as HMENU;
        } else {
            hMenu = core::ptr::null_mut();
        }
    }
    if hMenu.is_null() {
        hMenu = (*Window).SystemMenu;
        hSubMenu = hMenu;
        MenuObject = user_get_menu_object(hMenu);
        nPos = (*pAccel).cmd as u32;
        if !MenuObject.is_null() {
            if !MENU_FindItem(&mut MenuObject, &mut nPos, MF_BYPOSITION as u32).is_null() {
                hSubMenu = hm_get_handle(MenuObject) as HMENU;
            } else {
                hMenu = core::ptr::null_mut();
            }
        }
    }

    if !hMenu.is_null() && IntGetCaptureWindow().is_null() {
        co_IntSendMessage(hWnd, WM_INITMENU, hMenu as WPARAM, 0);
        if !hSubMenu.is_null() {
            nPos = IntFindSubMenu(&mut hMenu, hSubMenu) as u32;
            let lParam = (nPos & 0xFFFF) as isize | (1isize << 16);
            co_IntSendMessage(hWnd, WM_INITMENUPOPUP, hSubMenu as WPARAM, lParam);
        }
    }

    if ((*Window).style & WS_DISABLED as u32) == 0
        && !(!hMenu.is_null()
            && (IntGetMenuState(hMenu, (*pAccel).cmd as u32, MF_BYCOMMAND as u32)
                & (MF_DISABLED as u32 | MF_GRAYED as u32))
                != 0)
        && !(!hMenu.is_null()
            && hMenu == (*Window).IDMenu as HMENU
            && ((*Window).style & WS_MINIMIZED as u32) != 0)
    {
        if !hMenu.is_null() && hMenu == (*Window).SystemMenu {
            co_IntSendMessage(hWnd, WM_SYSCOMMAND, (*pAccel).cmd as WPARAM, 0x00010000);
        } else {
            co_IntSendMessage(hWnd, WM_COMMAND, (0x10000 | (*pAccel).cmd as u32) as WPARAM, 0);
        }
    }

    1
}
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCopyAcceleratorTable(
    hAccel: HACCEL,
    Entries: LPACCEL,
    mut EntriesCount: ULONG,
) -> ULONG {
    let Accel: PACCELERATOR_TABLE;
    let mut Ret: ULONG = 0;
    let Status: NTSTATUS;

    UserEnterShared();

    'exit: {
        Accel = UserGetAccelObject(hAccel);
        if Accel.is_null() {
            break 'exit;
        }

        /* If Entries is NULL return table size */
        if Entries.is_null() {
            Ret = (*Accel).Count as ULONG;
            break 'exit;
        }

        /* Don't overrun */
        if ((*Accel).Count as ULONG) < EntriesCount {
            EntriesCount = (*Accel).Count as ULONG;
        }

        Status = SafeProbeForWrite(
            Entries as *mut _,
            (EntriesCount as usize * core::mem::size_of::<ACCEL>()) as _,
            4,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
        } else {
            Ret = 0;
            // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped probe AND every
            // store in one _SEH2_TRY. Publish field by field rather than whole
            // structs, so ACCEL's padding stays untouched exactly as the C left it.
            while Ret < EntriesCount {
                let src = (*Accel).Table.add(Ret as usize);
                let dst = Entries.add(Ret as usize);
                macro_rules! put {
                    ($field:ident) => {
                        crate::copy::publish_to_user(
                            core::ptr::addr_of_mut!((*dst).$field) as *mut core::ffi::c_void,
                            core::ptr::addr_of!((*src).$field) as *const core::ffi::c_void,
                            core::mem::size_of_val(&(*src).$field),
                        )
                    };
                }
                if !put!(fVirt) || !put!(key) || !put!(cmd) {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    Ret = 0;
                    break;
                }
                Ret += 1;
            }
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCreateAcceleratorTable(
    Entries: LPACCEL,
    EntriesCount: ULONG,
) -> HACCEL {
    let mut Accel: PACCELERATOR_TABLE;
    let mut hAccel: HACCEL = core::ptr::null_mut();
    let mut Index: ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut Ret: HACCEL = core::ptr::null_mut();
    let pti: PTHREADINFO;

    UserEnterExclusive();

    'exit: {
        // The byte size below is `EntriesCount as usize * size_of::<ACCEL>()`. On i386
        // usize is 32 bits, so a large EntriesCount WRAPS, we allocate a short buffer
        // and then copy EntriesCount entries into it -- heap corruption from user mode.
        // Upstream c65740eeb3a (#7888) added the same bound as MAXULONG_PTR/sizeof.
        if Entries.is_null()
            || EntriesCount == 0
            || EntriesCount as usize > usize::MAX / core::mem::size_of::<ACCEL>()
        {
            SetLastNtError(STATUS_INVALID_PARAMETER);
            break 'exit;
        }

        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

        Accel = UserCreateObject(
            gHandleTable,
            (*pti).rpdesk,
            pti as *mut _,
            &mut hAccel as *mut HACCEL as PHANDLE,
            TYPE_ACCELTABLE,
            core::mem::size_of::<ACCELERATOR_TABLE>() as _,
        ) as PACCELERATOR_TABLE;

        if Accel.is_null() {
            SetLastNtError(STATUS_NO_MEMORY);
            break 'exit;
        }

        (*Accel).Count = EntriesCount;
        (*Accel).Table = ExAllocatePoolWithTag(
            PagedPool,
            (EntriesCount as usize * core::mem::size_of::<ACCEL>()) as _,
            USERTAG_ACCEL,
        ) as *mut ACCEL;
        if (*Accel).Table.is_null() {
            UserDereferenceObject(Accel as *mut _);
            UserDeleteObject(hAccel as HANDLE, TYPE_ACCELTABLE);
            SetLastNtError(STATUS_NO_MEMORY);
            break 'exit;
        }

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
        // C (win32ss/user/ntuser/accelerator.c, NtUserCreateAcceleratorTable):
        //   _SEH2_TRY {
        //       ProbeForRead(Entries, EntriesCount * sizeof(ACCEL), 4);
        //       for (Index = 0; Index < EntriesCount; Index++) { ...copy... }
        //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
        //       Status = _SEH2_GetExceptionCode();
        //   } _SEH2_END;
        //   if (!NT_SUCCESS(Status)) { ...cleanup...; goto Exit; }
        //
        // Shape: the handler only ASSIGNS Status and falls through — there is no
        // return/goto inside the SEH block itself, control simply reaches the
        // `if (!NT_SUCCESS(Status))` cleanup that already existed below. So here
        // we set `Status` on the first faulting entry and break out of both the
        // probe and the copy loop with a labeled block; the untouched
        // `if !NT_SUCCESS(Status)` check right after this block is the same
        // fall-through cleanup path as the C.
        let PreviousMode: i8 = KeGetPreviousMode();
        Index = 0;
        'seh: {
            // ProbeForRead(Entries, EntriesCount * sizeof(ACCEL), 4) — literal
            // alignment 4 from the C.
            //
            // SafeProbeForRead, not ntkrnl_sys::probe::probe_user_buffer. The
            // ntkrnl-sys range check reads `MmUserProbeAddress`, which is a DATA
            // export: a plain extern reference to it is unsatisfiable from a
            // separate driver, because the import library provides only `__imp_`
            // and the binding differs per arch (see the comment in class.rs).
            // Inside ntoskrnl the symbol is local so it resolves; from win32k.sys
            // it does not, and arm64 failed to link on exactly this
            // (`undefined symbol: MmUserProbeAddress`) while amd64 happened to
            // resolve it. win32k already imports SafeProbeForRead and it does the
            // same range check, so use the one this driver can actually link.
            let st = SafeProbeForRead(
                Entries as *mut c_void,
                (EntriesCount as usize * core::mem::size_of::<ACCEL>()) as SIZE_T,
                4,
            );
            if st < 0 {
                Status = st;
                break 'seh;
            }

            while Index < EntriesCount {
                // Fault-protected capture of Entries[Index] — the per-element reads
                // in the C loop sit inside the same _SEH2_TRY as the probe, so a
                // page fault mid-loop (not just an out-of-range pointer) must be
                // caught too, not merely range-checked once up front.
                // safe_copy_raw rather than a range-checking helper: the range
                // was proven once above, and this only has to survive the page
                // going away mid-loop. It also avoids MmUserProbeAddress (above).
                let mut CapturedEntry: ACCEL = core::mem::zeroed();
                let CapturedEntry: ACCEL = match ntkrnl_sys::probe::safe_copy_raw(
                    core::ptr::addr_of_mut!(CapturedEntry) as *mut c_void,
                    Entries.add(Index as usize) as *const c_void,
                    core::mem::size_of::<ACCEL>(),
                )
                .map(|()| CapturedEntry)
                {
                    Ok(v) => v,
                    Err(st) => {
                        Status = st;
                        break 'seh;
                    }
                };

                (*(*Accel).Table.add(Index as usize)).fVirt =
                    CapturedEntry.fVirt & FVIRT_MASK as u8;
                if (*(*Accel).Table.add(Index as usize)).fVirt & FVIRTKEY as u8 != 0 {
                    (*(*Accel).Table.add(Index as usize)).key = CapturedEntry.key;
                } else {
                    // Source is now the already fault-protected local capture
                    // (kernel memory), not the raw user pointer.
                    RtlMultiByteToUnicodeN(
                        core::ptr::addr_of_mut!((*(*Accel).Table.add(Index as usize)).key),
                        core::mem::size_of::<u16>() as _,
                        core::ptr::null_mut(),
                        core::ptr::addr_of!(CapturedEntry.key) as PCSTR,
                        core::mem::size_of::<i8>() as _,
                    );
                }

                (*(*Accel).Table.add(Index as usize)).cmd = CapturedEntry.cmd;

                Index += 1;
            }
        }

        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag((*Accel).Table as *mut _, USERTAG_ACCEL);
            UserDereferenceObject(Accel as *mut _);
            UserDeleteObject(hAccel as HANDLE, TYPE_ACCELTABLE);
            SetLastNtError(Status);
            break 'exit;
        }

        UserDereferenceObject(Accel as *mut _);

        Ret = hAccel;
    }

    UserLeave();
    Ret
}

#[no_mangle]
pub unsafe extern "C" fn UserDestroyAccelTable(Object: PVOID) -> BOOLEAN {
    let Accel: PACCELERATOR_TABLE = Object as PACCELERATOR_TABLE;

    if !(*Accel).Table.is_null() {
        ExFreePoolWithTag((*Accel).Table as PVOID, USERTAG_ACCEL);
        (*Accel).Table = core::ptr::null_mut();
    }

    UserDeleteObject(hm_get_handle(Accel), TYPE_ACCELTABLE);
    1u8
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDestroyAcceleratorTable(hAccel: HACCEL) -> BOOLEAN {
    let Accel: PACCELERATOR_TABLE;
    let mut Ret: BOOLEAN = 0u8;

    UserEnterExclusive();

    Accel = UserGetAccelObject(hAccel);
    if !Accel.is_null() {
        Ret = UserDestroyAccelTable(Accel as PVOID);
    }

    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserTranslateAccelerator(
    hWnd: HWND,
    hAccel: HACCEL,
    pUnsafeMessage: LPMSG,
) -> core::ffi::c_int {
    let mut Window: PWND = core::ptr::null_mut();
    let mut Accel: PACCELERATOR_TABLE = core::ptr::null_mut();
    let mut i: u32;
    let mut Message: MSG = core::mem::zeroed();
    let mut AccelRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut WindowRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Ret: core::ffi::c_int = 0;

    UserEnterShared();

    'cleanup: {
        if hWnd.is_null() {
            break 'cleanup;
        }

        {
            let __SehStatus = SafeProbeForRead(
                pUnsafeMessage as *mut core::ffi::c_void,
                core::mem::size_of::<MSG>() as ULONG,
                4,
            );
            if !NT_SUCCESS(__SehStatus) {
                SetLastNtError(__SehStatus);
                break 'cleanup;
            }
        }
        // Validate-only probe above; this capture is the access and takes its exit.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(Message) as *mut core::ffi::c_void,
            pUnsafeMessage as *const core::ffi::c_void,
            core::mem::size_of::<MSG>(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            break 'cleanup;
        }

        if (Message.message != WM_KEYDOWN)
            && (Message.message != WM_SYSKEYDOWN)
            && (Message.message != WM_SYSCHAR)
            && (Message.message != WM_CHAR)
        {
            break 'cleanup;
        }

        Accel = UserGetAccelObject(hAccel);
        if Accel.is_null() {
            break 'cleanup;
        }

        user_ref_object_co(Accel as *mut core::ffi::c_void, core::ptr::addr_of_mut!(AccelRef));

        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'cleanup;
        }

        user_ref_object_co(Window as *mut core::ffi::c_void, core::ptr::addr_of_mut!(WindowRef));

        i = 0;
        while i < (*Accel).Count as u32 {
            if co_IntTranslateAccelerator(
                Window,
                core::ptr::addr_of!(Message),
                core::ptr::addr_of!((*(*Accel).Table.add(i as usize))),
            ) != 0
            {
                Ret = 1;
                break;
            }

            if (*(*Accel).Table.add(i as usize)).fVirt as u32 & FVIRT_TBL_END != 0 {
                break;
            }

            i += 1;
        }
    }

    if !Window.is_null() {
        user_deref_object_co(Window as *mut core::ffi::c_void);
    }
    if !Accel.is_null() {
        user_deref_object_co(Accel as *mut core::ffi::c_void);
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserTranslateAccelerator(
    hWnd: HWND,
    hAccel: HACCEL,
    pUnsafeMessage: LPMSG,
) -> core::ffi::c_int {
    let mut Window: PWND = core::ptr::null_mut();
    let mut Accel: PACCELERATOR_TABLE = core::ptr::null_mut();
    let mut i: u32;
    let mut Message: MSG = core::mem::zeroed();
    let mut AccelRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut WindowRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Ret: core::ffi::c_int = 0;

    UserEnterShared();

    'cleanup: {
        if hWnd.is_null() {
            break 'cleanup;
        }

        {
            let __SehStatus = SafeProbeForRead(
                pUnsafeMessage as *mut core::ffi::c_void,
                core::mem::size_of::<MSG>() as _,
                4,
            );
            if !NT_SUCCESS(__SehStatus) {
                SetLastNtError(__SehStatus);
                break 'cleanup;
            }
        }
        // Validate-only probe above; this capture is the access and takes its exit.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(Message) as *mut core::ffi::c_void,
            pUnsafeMessage as *const core::ffi::c_void,
            core::mem::size_of::<MSG>(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            break 'cleanup;
        }

        if (Message.message != WM_KEYDOWN)
            && (Message.message != WM_SYSKEYDOWN)
            && (Message.message != WM_SYSCHAR)
            && (Message.message != WM_CHAR)
        {
            break 'cleanup;
        }

        Accel = UserGetAccelObject(hAccel);
        if Accel.is_null() {
            break 'cleanup;
        }

        user_ref_object_co(Accel as *mut core::ffi::c_void, core::ptr::addr_of_mut!(AccelRef));

        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'cleanup;
        }

        user_ref_object_co(Window as *mut core::ffi::c_void, core::ptr::addr_of_mut!(WindowRef));

        i = 0;
        while i < (*Accel).Count as u32 {
            if co_IntTranslateAccelerator(
                Window,
                core::ptr::addr_of!(Message),
                core::ptr::addr_of!((*(*Accel).Table.add(i as usize))),
            ) != 0
            {
                Ret = 1;
                break;
            }

            if (*(*Accel).Table.add(i as usize)).fVirt as u32 & FVIRT_TBL_END != 0 {
                break;
            }

            i += 1;
        }
    }

    if !Window.is_null() {
        user_deref_object_co(Window as *mut core::ffi::c_void);
    }
    if !Accel.is_null() {
        user_deref_object_co(Accel as *mut core::ffi::c_void);
    }

    UserLeave();
    Ret
}
