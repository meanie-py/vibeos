use win32k_sys::win32k::*;
// ntdll Rtl* heap/string helpers from the substrate (RtlNtStatusToDosError too).
use crate::internal::{
    RtlAllocateHeap, RtlCreateUnicodeStringFromAsciiz, RtlFreeHeap, RtlFreeUnicodeString,
};
// LOWORD/HIWORD are macros, ValidateHwnd lives in crate::internal, NT_SUCCESS in winbase.
use crate::internal::{HIWORD, LOWORD, ValidateHwnd};
use crate::winbase::NT_SUCCESS;
#[allow(unused_imports)]
use core::ffi::c_void;

// STATUS_INVALID_HANDLE — ntstatus.h.
const STATUS_INVALID_HANDLE: NTSTATUS = 0xC0000008u32 as NTSTATUS;

extern "C" {
    // user32-internal: maps a kernel shared-section pointer into the client view
    // (SharedPtrToUser is exported cdecl from misc/misc.rs → `_SharedPtrToUser`).
    fn SharedPtrToUser(ptr: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}
// ntdll Rtl* export is stdcall on i386 (`_RtlNtStatusToDosError@4`) → extern "system".
extern "system" {
    fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG;
}
extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn GlobalGetAtomNameA(nAtom: ATOM, lpBuffer: LPSTR, nSize: i32) -> u32;
    fn SetLastError(dwErrCode: u32);
    fn GlobalGetAtomNameW(nAtom: ATOM, lpBuffer: LPWSTR, nSize: i32) -> u32;
    fn GlobalFindAtomW(lpString: LPCWSTR) -> ATOM;
    fn GlobalAddAtomW(lpString: LPCWSTR) -> ATOM;
}

// ---- file-scope preamble (translated) ----
pub const ATOM_BUFFER_SIZE: usize = 256;

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetProp(hWnd: HWND, Atom: ATOM, SystemProp: u8) -> *mut core::ffi::c_void {
    IntGetProp_impl(hWnd, Atom, SystemProp)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetProp(hWnd: HWND, Atom: ATOM, SystemProp: u8) -> *mut core::ffi::c_void {
    IntGetProp_impl(hWnd, Atom, SystemProp)
}

unsafe fn IntGetProp_impl(hWnd: HWND, Atom: ATOM, SystemProp: u8) -> *mut core::ffi::c_void {

    let mut ListEntry: PLIST_ENTRY;
    let temp: PLIST_ENTRY;
    let mut Property: PPROPERTY;
    let pWnd: PWND;
    let mut i: i32;
    let SystemFlag: u16 = if SystemProp != 0 { PROPERTY_FLAG_SYSTEM as u16 } else { 0 };

    pWnd = ValidateHwnd(hWnd);
    if pWnd.is_null() {
        return core::ptr::null_mut();
    }

    #[cfg(target_arch = "x86")]
    if crate::wow64::IsWow64() {
        return crate::wow64::find_prop(pWnd, Atom, SystemFlag) as *mut core::ffi::c_void;
    }

    ListEntry = SharedPtrToUser((*pWnd).PropListHead.Flink as *mut core::ffi::c_void) as PLIST_ENTRY;
    i = 0;
    while i < (*pWnd).PropListItems as i32 {
        Property = (ListEntry as *mut u8)
            .sub(core::mem::offset_of!(PROPERTY, PropListEntry)) as PPROPERTY;
        if (*Property).Atom == Atom
            && ((*Property).fs & PROPERTY_FLAG_SYSTEM as u16) == SystemFlag
        {
            return Property as *mut core::ffi::c_void;
        }
        let temp_local = (*ListEntry).Flink;
        let _ = temp_local;
        ListEntry = SharedPtrToUser((*ListEntry).Flink as *mut core::ffi::c_void) as PLIST_ENTRY;
        i += 1;
    }
    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetProp(hWnd: HWND, Atom: ATOM, SystemProp: u8) -> *mut core::ffi::c_void {
    let Prop: PPROPERTY = IntGetProp(hWnd, Atom, SystemProp) as PPROPERTY;
    if !Prop.is_null() {
        (*Prop).Data
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetProp(hWnd: HWND, Atom: ATOM, SystemProp: u8) -> *mut core::ffi::c_void {
    let Prop: PPROPERTY = IntGetProp(hWnd, Atom, SystemProp) as PPROPERTY;
    if !Prop.is_null() {
        (*Prop).Data
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "system" fn EnumPropsA(hWnd: HWND, lpEnumFunc: PROPENUMPROCA) -> i32 {

    let mut Count: u32 = 0;
    let mut ret: i32 = -1;

    let lpEnumFunc = match lpEnumFunc {
        Some(f) => f,
        None => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return ret;
        }
    };

    let mut Status = NtUserBuildPropList(hWnd, core::ptr::null_mut(), 0, &mut Count);
    if !NT_SUCCESS(Status) {
        if Status == STATUS_INVALID_HANDLE {
            SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        } else {
            SetLastError(RtlNtStatusToDosError(Status));
        }
        return ret;
    }

    if Count > 0 {
        let pli = RtlAllocateHeap(GetProcessHeap(), 0, Count as SIZE_T) as PPROPLISTITEM;
        if pli.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            return -1;
        }

        Status = NtUserBuildPropList(hWnd, pli as LPVOID, Count, &mut Count);
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(GetProcessHeap(), 0, pli as *mut core::ffi::c_void);
            if Status == STATUS_INVALID_HANDLE {
                SetLastError(ERROR_INVALID_WINDOW_HANDLE);
            } else {
                SetLastError(RtlNtStatusToDosError(Status));
            }
            return ret;
        }

        let mut i = pli;
        while Count > 0 {
            let mut str: [core::ffi::c_char; 256] = [0; 256];

            if GlobalGetAtomNameA((*i).Atom, str.as_mut_ptr(), 256) == 0 {
                Count -= 1;
                i = i.add(1);
                continue;
            }

            ret = lpEnumFunc(hWnd, str.as_ptr() as LPCSTR, (*i).Data);
            if ret == 0 {
                break;
            }

            Count -= 1;
            i = i.add(1);
        }

        RtlFreeHeap(GetProcessHeap(), 0, pli as *mut core::ffi::c_void);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumPropsExA(hWnd: HWND, lpEnumFunc: PROPENUMPROCEXA, lParam: LPARAM) -> i32 {

    let mut pli: PPROPLISTITEM;
    let mut i: PPROPLISTITEM;
    let mut Status: i32;
    let mut Count: DWORD_PTR = 0;
    let mut ret: i32 = -1;

    let lpEnumFunc = match lpEnumFunc {
        Some(f) => f,
        None => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return ret;
        }
    };

    Status = NtUserBuildPropList(hWnd, core::ptr::null_mut(), 0, &mut Count as *mut DWORD_PTR as *mut u32);
    if !NT_SUCCESS(Status) {
        if Status == STATUS_INVALID_HANDLE {
            SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        } else {
            SetLastError(RtlNtStatusToDosError(Status));
        }
        return ret;
    }

    if Count > 0 {
        pli = RtlAllocateHeap(GetProcessHeap(), 0, Count as SIZE_T) as PPROPLISTITEM;
        if pli.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            return -1;
        }

        Status = NtUserBuildPropList(hWnd, pli as LPVOID, Count as u32, &mut Count as *mut DWORD_PTR as *mut u32);
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(GetProcessHeap(), 0, pli as LPVOID);
            if Status == STATUS_INVALID_HANDLE {
                SetLastError(ERROR_INVALID_WINDOW_HANDLE);
            } else {
                SetLastError(RtlNtStatusToDosError(Status));
            }
            return ret;
        }

        i = pli;
        while Count > 0 {
            let mut str: [core::ffi::c_char; 256] = [0; 256];

            if GlobalGetAtomNameA((*i).Atom, str.as_mut_ptr() as LPSTR, 256) == 0 {
                Count -= 1;
                i = i.add(1);
                continue;
            }

            ret = lpEnumFunc(hWnd, str.as_mut_ptr() as LPSTR, (*i).Data, lParam as ULONG_PTR);
            if ret == 0 {
                break;
            }

            Count -= 1;
            i = i.add(1);
        }

        RtlFreeHeap(GetProcessHeap(), 0, pli as LPVOID);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumPropsExW(hWnd: HWND, lpEnumFunc: PROPENUMPROCEXW, lParam: LPARAM) -> i32 {

    let mut ret: i32 = -1;

    let func = match lpEnumFunc {
        Some(f) => f,
        None => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return ret;
        }
    };

    let mut Count: DWORD = 0;
    let mut Status = NtUserBuildPropList(hWnd, core::ptr::null_mut(), 0, &mut Count);
    if !NT_SUCCESS(Status) {
        if Status == STATUS_INVALID_HANDLE {
            SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        } else {
            SetLastError(RtlNtStatusToDosError(Status));
        }
        return ret;
    }

    if Count > 0 {
        let pli = RtlAllocateHeap(GetProcessHeap(), 0, Count as SIZE_T) as PPROPLISTITEM;
        if pli.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            return -1;
        }

        Status = NtUserBuildPropList(hWnd, pli as LPVOID, Count, &mut Count);
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(GetProcessHeap(), 0, pli as *mut core::ffi::c_void);
            if Status == STATUS_INVALID_HANDLE {
                SetLastError(ERROR_INVALID_WINDOW_HANDLE);
            } else {
                SetLastError(RtlNtStatusToDosError(Status));
            }
            return ret;
        }

        let mut i = pli;
        while Count > 0 {
            let mut str: [u16; 256] = [0; 256];

            if GlobalGetAtomNameW((*i).Atom, str.as_mut_ptr(), 256) != 0 {
                ret = func(hWnd, str.as_mut_ptr() as LPWSTR, (*i).Data, lParam as ULONG_PTR);
                if ret == 0 {
                    break;
                }
            }

            Count -= 1;
            i = i.add(1);
        }

        RtlFreeHeap(GetProcessHeap(), 0, pli as *mut core::ffi::c_void);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumPropsW(hWnd: HWND, lpEnumFunc: PROPENUMPROCW) -> core::ffi::c_int {
    let mut ret: core::ffi::c_int = -1;

    if lpEnumFunc.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return ret;
    }
    let lpEnumFunc = lpEnumFunc.unwrap();

    let mut Count: DWORD_PTR = 0;

    let mut Status = NtUserBuildPropList(hWnd, core::ptr::null_mut(), 0, &mut Count as *mut _ as *mut u32);
    if !NT_SUCCESS(Status) {
        if Status == STATUS_INVALID_HANDLE {
            SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        } else {
            SetLastError(RtlNtStatusToDosError(Status));
        }
        return ret;
    }

    if Count > 0 {
        let pli = RtlAllocateHeap(GetProcessHeap(), 0, Count as SIZE_T) as PPROPLISTITEM;
        if pli.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            return -1;
        }

        Status = NtUserBuildPropList(hWnd, pli as LPVOID, Count as u32, &mut Count as *mut _ as *mut u32);
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(GetProcessHeap(), 0, pli as *mut core::ffi::c_void);
            if Status == STATUS_INVALID_HANDLE {
                SetLastError(ERROR_INVALID_WINDOW_HANDLE);
            } else {
                SetLastError(RtlNtStatusToDosError(Status));
            }
            return ret;
        }

        let mut i = pli;
        while Count > 0 {
            let mut str: [u16; 256] = [0; 256];

            if GlobalGetAtomNameW((*i).Atom, str.as_mut_ptr(), 256) == 0 {
                Count -= 1;
                i = i.add(1);
                continue;
            }

            ret = lpEnumFunc(hWnd, str.as_ptr(), (*i).Data);
            if ret == 0 {
                break;
            }

            Count -= 1;
            i = i.add(1);
        }

        RtlFreeHeap(GetProcessHeap(), 0, pli as *mut core::ffi::c_void);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetPropA(hWnd: HWND, lpString: LPCSTR) -> *mut core::ffi::c_void {
    let lpWString: PWSTR;
    let mut UString: _UNICODE_STRING = core::mem::zeroed();
    let Ret: *mut core::ffi::c_void;
    if HIWORD(lpString as DWORD_PTR as DWORD) != 0 {
        RtlCreateUnicodeStringFromAsciiz(&mut UString, lpString as LPSTR);
        lpWString = UString.Buffer;
        if lpWString.is_null() {
            return core::ptr::null_mut();
        }
        Ret = GetPropW(hWnd, lpWString);
        RtlFreeUnicodeString(&mut UString);
    } else {
        Ret = GetPropW(hWnd, lpString as LPWSTR);
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetPropW(hWnd: HWND, lpString: LPCWSTR) -> *mut core::ffi::c_void {
    let Atom: ATOM;
    let mut Data: *mut core::ffi::c_void = core::ptr::null_mut();
    let Prop: PPROPERTY;
    if HIWORD(lpString as DWORD_PTR as DWORD) != 0 {
        Atom = GlobalFindAtomW(lpString);
    } else {
        Atom = LOWORD(lpString as DWORD_PTR as DWORD) as ATOM;
    }
    Prop = IntGetProp(hWnd, Atom, 0u8) as PPROPERTY;
    if !Prop.is_null() {
        Data = (*Prop).Data;
    }
    Data
}

#[no_mangle]
pub unsafe extern "system" fn RemovePropA(hWnd: HWND, lpString: LPCSTR) -> HANDLE {
    let lpWString: PWSTR;
    let mut UString: _UNICODE_STRING = core::mem::zeroed();
    let Ret: HANDLE;

    if HIWORD(lpString as usize as u32) != 0 {
        RtlCreateUnicodeStringFromAsciiz(&mut UString, lpString as LPSTR);
        lpWString = UString.Buffer;
        if lpWString.is_null() {
            return core::ptr::null_mut();
        }
        Ret = RemovePropW(hWnd, lpWString);
        RtlFreeUnicodeString(&mut UString);
    } else {
        Ret = RemovePropW(hWnd, lpString as LPCWSTR);
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RemovePropW(hWnd: HWND, lpString: LPCWSTR) -> HANDLE {
    let Atom: ATOM;
    if HIWORD(lpString as DWORD_PTR as DWORD) != 0 {
        Atom = GlobalFindAtomW(lpString);
    } else {
        Atom = LOWORD(lpString as DWORD_PTR as DWORD) as ATOM;
    }
    NtUserRemoveProp(hWnd, Atom)
}

#[no_mangle]
pub unsafe extern "system" fn SetPropA(hWnd: HWND, lpString: LPCSTR, hData: HANDLE) -> BOOL {
    let lpWString: PWSTR;
    let mut UString: _UNICODE_STRING = core::mem::zeroed();
    let Ret: BOOL;

    if (((lpString as usize) >> 16) & 0xFFFF) != 0 {
        RtlCreateUnicodeStringFromAsciiz(&mut UString, lpString as LPSTR);
        lpWString = UString.Buffer;
        if lpWString.is_null() {
            return 0;
        }
        Ret = SetPropW(hWnd, lpWString, hData);
        RtlFreeUnicodeString(&mut UString);
    } else {
        Ret = SetPropW(hWnd, lpString as LPWSTR, hData);
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SetPropW(hWnd: HWND, lpString: LPCWSTR, hData: HANDLE) -> BOOL {

    let Atom: ATOM;
    if ((lpString as DWORD_PTR >> 16) & 0xffff) != 0 {
        Atom = GlobalAddAtomW(lpString);
    } else {
        Atom = (lpString as DWORD_PTR & 0xffff) as ATOM;
    }

    NtUserSetProp(hWnd, Atom, hData)
}
