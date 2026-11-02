use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to emit. `DbgPrint` is already imported from `ntkrnl_sys::driver_ddk`, and `UNIMPLEMENTED` is a function-like macro that the functions inline.

// The C parameter is `Unknown`, which here would be a PATTERN: the island glob-imports
// an enum constant of that name (E0005). Renamed; the ABI is positional.
#[no_mangle]
pub unsafe extern "system" fn EditAuditInfo(Unknown_: DWORD) -> DWORD {
    UNREFERENCED_PARAMETER!(Unknown_);
    UNIMPLEMENTED!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn EditOwnerInfo(Unknown_: DWORD) -> DWORD {
    UNREFERENCED_PARAMETER!(Unknown_);
    UNIMPLEMENTED!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn EditPermissionInfo(Unknown_: DWORD) -> DWORD {
    UNREFERENCED_PARAMETER!(Unknown_);
    UNIMPLEMENTED!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FMExtensionProcW(hWnd: HWND, wEvent: WORD, lParam: LONG) -> LONG {
    UNREFERENCED_PARAMETER!(hWnd);
    UNREFERENCED_PARAMETER!(wEvent);
    UNREFERENCED_PARAMETER!(lParam);
    UNIMPLEMENTED!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SedDiscretionaryAclEditor(
    hWndOwner: HWND,
    hInstance: HINSTANCE,
    lpMachineName: LPCWSTR,
    pAclDlgControl: PACL_DLG_CONTROL,
    pAclEditControl: PACL_EDIT_CONTROL,
    lpObjectName: LPCWSTR,
    pChangeCallback: PACL_CHANGE,
    pChangeCallbackContext: PVOID,
    pObjectSecurity: PSECURITY_DESCRIPTOR,
    bNoReadPermission: BOOL,
    bReadOnly: BOOL,
    lpdwChangeContextStatus: LPDWORD,
    Reserved: DWORD,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hWndOwner);
    UNREFERENCED_PARAMETER!(hInstance);
    UNREFERENCED_PARAMETER!(lpMachineName);
    UNREFERENCED_PARAMETER!(pAclDlgControl);
    UNREFERENCED_PARAMETER!(pAclEditControl);
    UNREFERENCED_PARAMETER!(lpObjectName);
    UNREFERENCED_PARAMETER!(pChangeCallback);
    UNREFERENCED_PARAMETER!(pChangeCallbackContext);
    UNREFERENCED_PARAMETER!(pObjectSecurity);
    UNREFERENCED_PARAMETER!(bNoReadPermission);
    UNREFERENCED_PARAMETER!(bReadOnly);
    UNREFERENCED_PARAMETER!(lpdwChangeContextStatus);
    UNREFERENCED_PARAMETER!(Reserved);
    UNIMPLEMENTED!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SedTakeOwnership(
    hWndOwner: HWND,
    hInstance: HINSTANCE,
    lpMachineName: LPCWSTR,
    lpObjectType: LPCWSTR,
    lpObjectName: LPCWSTR,
    dwObjectCount: DWORD,
    pChangeCallback: PACL_CHANGE,
    pChangeCallbackContext: PVOID,
    pObjectSecurity: PSECURITY_DESCRIPTOR,
    bNoReadPermission: BOOL,
    bNoOwnerChange: BOOL,
    lpdwChangeContextStatus: LPDWORD,
    pHelpInfo: PACL_HELP_CONTROL,
    Reserved: DWORD,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hWndOwner);
    UNREFERENCED_PARAMETER!(hInstance);
    UNREFERENCED_PARAMETER!(lpMachineName);
    UNREFERENCED_PARAMETER!(lpObjectType);
    UNREFERENCED_PARAMETER!(lpObjectName);
    UNREFERENCED_PARAMETER!(dwObjectCount);
    UNREFERENCED_PARAMETER!(pChangeCallback);
    UNREFERENCED_PARAMETER!(pChangeCallbackContext);
    UNREFERENCED_PARAMETER!(pObjectSecurity);
    UNREFERENCED_PARAMETER!(bNoReadPermission);
    UNREFERENCED_PARAMETER!(bNoOwnerChange);
    UNREFERENCED_PARAMETER!(lpdwChangeContextStatus);
    UNREFERENCED_PARAMETER!(pHelpInfo);
    UNREFERENCED_PARAMETER!(Reserved);
    UNIMPLEMENTED!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SedSystemAclEditor(
    hWndOwner: HWND,
    hInstance: HINSTANCE,
    lpMachineName: LPCWSTR,
    pAclDlgControl: PACL_DLG_CONTROL,
    pAclEditControl: PACL_EDIT_CONTROL,
    lpObjectName: LPCWSTR,
    pChangeCallback: PACL_CHANGE,
    pChangeCallbackContext: PVOID,
    pObjectSecurity: PSECURITY_DESCRIPTOR,
    bNoReadPermission: BOOL,
    lpdwChangeContextStatus: LPDWORD,
    Reserved: DWORD,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hWndOwner);
    UNREFERENCED_PARAMETER!(hInstance);
    UNREFERENCED_PARAMETER!(lpMachineName);
    UNREFERENCED_PARAMETER!(pAclDlgControl);
    UNREFERENCED_PARAMETER!(pAclEditControl);
    UNREFERENCED_PARAMETER!(lpObjectName);
    UNREFERENCED_PARAMETER!(pChangeCallback);
    UNREFERENCED_PARAMETER!(pChangeCallbackContext);
    UNREFERENCED_PARAMETER!(pObjectSecurity);
    UNREFERENCED_PARAMETER!(bNoReadPermission);
    UNREFERENCED_PARAMETER!(lpdwChangeContextStatus);
    UNREFERENCED_PARAMETER!(Reserved);
    UNIMPLEMENTED!();
    return 0;
}
