// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_19319FD29DD2: [WCHAR; 79] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0049, 0x0063, 0x006F, 0x006E, 0x004F, 0x0076, 0x0065, 0x0072, 0x006C, 0x0061, 0x0079, 0x0049, 0x0064, 0x0065, 0x006E, 0x0074, 0x0069, 0x0066, 0x0069, 0x0065, 0x0072, 0x0073, 0x0000];

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::SHGetPathFromIDListW;
use crate::shellcom::*;
use crate::types::*;
use crate::win32::*;

extern "system" {
}

// ---- file-scope preamble (translated) ----
// C: `WCHAR swShell32Name[MAX_PATH];` in folders.cpp — read by brfolder.cpp and
// iconcache.cpp, so this must keep its exported name.
#[no_mangle]
pub static mut swShell32Name: [u16; MAX_PATH] = [0u16; MAX_PATH];
pub static mut NumIconOverlayHandlers: u32 = 0;
pub static mut Handlers: *mut *mut IShellIconOverlayIdentifier = core::ptr::null_mut();

pub type LPCITEMIDLIST = *const core::ffi::c_void;
unsafe extern "C" fn InitIconOverlays() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwIndex: u32 = 0;
    let mut dwResult: u32 = 0;
    let mut dwSize: u32 = 0;
    let mut clsid: CLSID = core::mem::zeroed();

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        CPP_COM_WSTR_19319FD29DD2.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) != ERROR_SUCCESS
    {
        return;
    }

    if RegQueryInfoKeyW(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwResult),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS
    {
        RegCloseKey(hKey);
        return;
    }

    Handlers = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((dwResult as usize) * core::mem::size_of::<*mut IShellIconOverlayIdentifier>()) as _,
    ) as *mut *mut IShellIconOverlayIdentifier;
    if Handlers.is_null() {
        RegCloseKey(hKey);
        return;
    }

    dwIndex = 0;
    CoInitialize(core::ptr::null_mut());

    let mut szName: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut szValue: [u16; 100] = [0u16; 100];

    loop {
        dwSize = (MAX_PATH) as u32;
        dwResult = (RegEnumKeyExW(
            hKey,
            dwIndex,
            szName.as_mut_ptr(),
            core::ptr::addr_of_mut!(dwSize),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;

        if dwResult == ERROR_NO_MORE_ITEMS {
            break;
        }

        if dwResult == (ERROR_SUCCESS) as u32 {
            dwSize = 100;
            if RegGetValueW(
                hKey,
                szName.as_ptr(),
                core::ptr::null_mut(),
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                (szValue.as_mut_ptr()).cast(),
                core::ptr::addr_of_mut!(dwSize),
            ) == ERROR_SUCCESS
            {
                let mut overlay_ptr: *mut IShellIconOverlayIdentifier = core::ptr::null_mut();
                CLSIDFromString(szValue.as_ptr(), core::ptr::addr_of_mut!(clsid));
                dwResult = (CoCreateInstance(
                    core::ptr::addr_of!(clsid),
                    core::ptr::null_mut(),
                    CLSCTX_INPROC_SERVER,
                    core::ptr::addr_of!(IID_IShellIconOverlayIdentifier), core::ptr::addr_of_mut!(overlay_ptr).cast(),
                )) as u32;
                if dwResult == (S_OK) as u32 {
                    *Handlers.add(NumIconOverlayHandlers as usize) = overlay_ptr;
                    NumIconOverlayHandlers += 1;
                }
            }
        }

        dwIndex = dwIndex.wrapping_add(1);

        if 1 == 0 {
            break;
        }
    }

    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn GetIconOverlay(pidl: LPCITEMIDLIST, wTemp: *mut u16, pIndex: *mut i32) -> i32 {
    let mut Index: u32;
    let mut hResult: i32;
    let mut Priority: i32 = 0;
    let mut HighestPriority: i32;
    let mut IconIndex: u32;
    let mut Flags: u32 = 0;
    let mut szPath: [u16; MAX_PATH] = [0u16; MAX_PATH];

    if SHGetPathFromIDListW((pidl).cast(), szPath.as_mut_ptr()) == 0 {
        return FALSE;
    }

    if Handlers.is_null() {
        InitIconOverlays();
    }

    HighestPriority = 101;
    IconIndex = NumIconOverlayHandlers;
    Index = 0;
    while Index < NumIconOverlayHandlers {
        hResult = (**Handlers.add(Index as usize)).IsMemberOf(szPath.as_ptr(), SFGAO_FILESYSTEM);
        if hResult == S_OK {
            hResult = (**Handlers.add(Index as usize)).GetPriority(&mut Priority);
            if hResult == S_OK {
                if Priority < HighestPriority {
                    HighestPriority = Priority;
                    IconIndex = Index;
                }
            }
        }
        Index += 1;
    }

    if IconIndex == NumIconOverlayHandlers {
        return FALSE;
    }

    hResult = (**Handlers.add(IconIndex as usize)).GetOverlayInfo(
                              wTemp,
                              (MAX_PATH) as INT,
                              pIndex,
                              &mut Flags);

    if hResult == S_OK {
        return TRUE;
    } else {
        return FALSE;
    }
}
