//! Rust translation of `win32ss/user/ntuser/prop.c` — window property lists.
//! [ntuser wave 2: easy leaves]

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{RtlAllocateHeap, RtlFreeHeap};
use ntkrnl_sys::base::{IsListEmpty, InsertTailList, RemoveEntryList, NT_SUCCESS, STATUS_SUCCESS};
use ntkrnl_sys::constants::{STATUS_INVALID_HANDLE, STATUS_INVALID_PARAMETER};
use core::ffi::c_void;

// _MmCopyToCaller (win32k copy.c, cdecl) — probe+SEH copy to a user buffer.
// `MmCopyToCaller(x,y,z)` is the macro form. On i386 the C name `_MmCopyToCaller`
// decorates to `__MmCopyToCaller`, which extern "C" `_MmCopyToCaller` produces;
// amd64/arm64 match the bare name.
extern "C" {
    fn _MmCopyToCaller(target: *mut c_void, source: *mut c_void, bytes: UINT) -> NTSTATUS;
}

// UserHeapAlloc/UserHeapFree are FORCEINLINE in usrheap.h (no symbol) — inline
// them: RtlAllocateHeap/RtlFreeHeap on GlobalUserHeap, HEAP_NO_SERIALIZE
// (the global user lock serializes). The C ASSERT(UserIsEnteredExclusive())
// is a retail no-op.
#[inline]
unsafe fn user_heap_alloc(bytes: SIZE_T) -> *mut c_void {
    RtlAllocateHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, bytes)
}
#[inline]
unsafe fn user_heap_free(mem: *mut c_void) -> BOOL {
    RtlFreeHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, mem) as BOOL
}

// IntGetProp is FASTCALL, cross-file — dual-cfg + #[no_mangle].
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetProp(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> PPROPERTY {
    int_get_prop(window, atom, system_prop)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetProp(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> PPROPERTY {
    int_get_prop(window, atom, system_prop)
}

unsafe fn int_get_prop(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> PPROPERTY {
    let system_flag: u16 = if system_prop != 0 { PROPERTY_FLAG_SYSTEM as u16 } else { 0 };

    let mut list_entry: PLIST_ENTRY = (*window).PropListHead.Flink;
    let mut i: u32 = 0;
    while i < (*window).PropListItems {
        let property = (list_entry as *mut u8)
            .offset(-(core::mem::offset_of!(PROPERTY, PropListEntry) as isize)) as PPROPERTY;
        list_entry = (*list_entry).Flink;

        if (*property).Atom == atom && ((*property).fs & PROPERTY_FLAG_SYSTEM as u16) == system_flag {
            return property;
        }
        i += 1;
    }
    core::ptr::null_mut()
}

// UserGetProp / UserRemoveProp / UserSetProp / UserRemoveWindowProps — FASTCALL.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetProp(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> HANDLE {
    user_get_prop(window, atom, system_prop)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetProp(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> HANDLE {
    user_get_prop(window, atom, system_prop)
}

unsafe fn user_get_prop(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> HANDLE {
    let prop = IntGetProp(window, atom, system_prop);
    if !prop.is_null() {
        (*prop).Data
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserRemoveProp(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> HANDLE {
    user_remove_prop(window, atom, system_prop)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserRemoveProp(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> HANDLE {
    user_remove_prop(window, atom, system_prop)
}

unsafe fn user_remove_prop(window: PWND, atom: ATOM, system_prop: BOOLEAN) -> HANDLE {
    let prop = IntGetProp(window, atom, system_prop);
    if prop.is_null() {
        return core::ptr::null_mut();
    }

    let data = (*prop).Data;
    RemoveEntryList(core::ptr::addr_of_mut!((*prop).PropListEntry) as *mut _);
    user_heap_free(prop as *mut c_void);
    (*window).PropListItems -= 1;
    data
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserSetProp(window: PWND, atom: ATOM, data: HANDLE, system_prop: BOOLEAN) -> BOOL {
    user_set_prop(window, atom, data, system_prop)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserSetProp(window: PWND, atom: ATOM, data: HANDLE, system_prop: BOOLEAN) -> BOOL {
    user_set_prop(window, atom, data, system_prop)
}

unsafe fn user_set_prop(window: PWND, atom: ATOM, data: HANDLE, system_prop: BOOLEAN) -> BOOL {
    let mut prop: PPROPERTY = IntGetProp(window, atom, system_prop);
    if prop.is_null() {
        prop = user_heap_alloc(core::mem::size_of::<PROPERTY>() as SIZE_T) as PPROPERTY;
        if prop.is_null() {
            return 0;
        }
        (*prop).Atom = atom;
        (*prop).fs = if system_prop != 0 { PROPERTY_FLAG_SYSTEM as u16 } else { 0 };
        InsertTailList(
            core::ptr::addr_of_mut!((*window).PropListHead) as *mut _,
            core::ptr::addr_of_mut!((*prop).PropListEntry) as *mut _,
        );
        (*window).PropListItems += 1;
    }

    (*prop).Data = data;
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserRemoveWindowProps(window: PWND) {
    user_remove_window_props(window)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserRemoveWindowProps(window: PWND) {
    user_remove_window_props(window)
}

unsafe fn user_remove_window_props(window: PWND) {
    while !IsListEmpty(core::ptr::addr_of!((*window).PropListHead) as *const _) {
        let list_entry: PLIST_ENTRY = (*window).PropListHead.Flink;
        let property = (list_entry as *mut u8)
            .offset(-(core::mem::offset_of!(PROPERTY, PropListEntry) as isize)) as PPROPERTY;
        RemoveEntryList(core::ptr::addr_of_mut!((*property).PropListEntry) as *mut _);
        user_heap_free(property as *mut c_void);
        (*window).PropListItems -= 1;
    }
}

// NtUserRemoveProp / NtUserSetProp — NTAPI syscalls (stdcall i386).
#[no_mangle]
pub unsafe extern "system" fn NtUserRemoveProp(h_wnd: HWND, atom: ATOM) -> HANDLE {
    nt_user_remove_prop(h_wnd, atom)
}

unsafe fn nt_user_remove_prop(h_wnd: HWND, atom: ATOM) -> HANDLE {
    let mut data: HANDLE = core::ptr::null_mut();

    UserEnterExclusive();
    let window = UserGetWindowObject(h_wnd);
    if !window.is_null() {
        data = UserRemoveProp(window, atom, 0);
    }
    UserLeave();

    data
}

#[no_mangle]
pub unsafe extern "system" fn NtUserSetProp(h_wnd: HWND, atom: ATOM, data: HANDLE) -> BOOL {
    nt_user_set_prop(h_wnd, atom, data)
}

unsafe fn nt_user_set_prop(h_wnd: HWND, atom: ATOM, data: HANDLE) -> BOOL {
    UserEnterExclusive();
    let window = UserGetWindowObject(h_wnd);
    let ret: BOOL = if window.is_null() { 0 } else { UserSetProp(window, atom, data, 0) };
    UserLeave();

    ret
}

// NtUserBuildPropList was SAL-annotated (_Out_writes_bytes_to_opt_) → skipped
// by rustconv's scanner (the SAL/scanner-miss class); hand-translated.
// APIENTRY (stdcall i386).
#[no_mangle]
pub unsafe extern "system" fn NtUserBuildPropList(h_wnd: HWND, buffer: LPVOID, buffer_size: DWORD, count: *mut DWORD) -> NTSTATUS {
    nt_user_build_prop_list(h_wnd, buffer, buffer_size, count)
}

unsafe fn nt_user_build_prop_list(h_wnd: HWND, buffer: LPVOID, mut buffer_size: DWORD, count: *mut DWORD) -> NTSTATUS {
    const ITEM: usize = core::mem::size_of::<PROPLISTITEM>();
    let mut status: NTSTATUS;
    let mut cnt: DWORD = 0;

    UserEnterShared();

    'exit: {
        let window = UserGetWindowObject(h_wnd);
        if window.is_null() {
            status = STATUS_INVALID_HANDLE;
            break 'exit;
        }

        if !buffer.is_null() {
            if buffer_size == 0 || (buffer_size as usize % ITEM != 0) {
                status = STATUS_INVALID_PARAMETER;
                break 'exit;
            }

            /* Copy list */
            let mut li = buffer as *mut PROPLISTITEM;
            let mut list_entry: PLIST_ENTRY = (*window).PropListHead.Flink;
            while (buffer_size as usize >= ITEM)
                && list_entry != core::ptr::addr_of_mut!((*window).PropListHead) as PLIST_ENTRY
            {
                let property = (list_entry as *mut u8)
                    .offset(-(core::mem::offset_of!(PROPERTY, PropListEntry) as isize)) as PPROPERTY;
                list_entry = (*list_entry).Flink;
                if (*property).fs & PROPERTY_FLAG_SYSTEM as u16 == 0 {
                    let mut listitem: PROPLISTITEM = core::mem::zeroed();
                    listitem.Atom = (*property).Atom;
                    listitem.Data = (*property).Data;

                    status = _MmCopyToCaller(
                        li as *mut c_void,
                        core::ptr::addr_of_mut!(listitem) as *mut c_void,
                        ITEM as UINT,
                    );
                    if !NT_SUCCESS(status) {
                        break 'exit;
                    }

                    buffer_size -= ITEM as DWORD;
                    cnt += 1;
                    li = li.add(1);
                }
            }
        } else {
            /* FIXME: This counts user and system props */
            cnt = (*window).PropListItems * ITEM as DWORD;
        }

        if !count.is_null() {
            status = _MmCopyToCaller(
                count as *mut c_void,
                core::ptr::addr_of_mut!(cnt) as *mut c_void,
                core::mem::size_of::<DWORD>() as UINT,
            );
            if !NT_SUCCESS(status) {
                break 'exit;
            }
        }

        status = STATUS_SUCCESS;
    }

    UserLeave();
    status
}
