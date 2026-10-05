use win32k_sys::win32k::*;

#[no_mangle]
pub static mut defaultDCstate: PDC = core::ptr::null_mut();

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCreatePrimarySurface() -> BOOL {
    /* Create surface */
    PDEVOBJ_pSurface((*gpmdev).ppdevGlobal);

    // Init Primary Displays Device Capabilities.
    PDEVOBJ_vGetDeviceCaps(
        (*gpmdev).ppdevGlobal,
        core::ptr::addr_of_mut!((*GdiHandleTable).DevCaps),
    );

    TRUE as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCreatePrimarySurface() -> BOOL {
    /* Create surface */
    PDEVOBJ_pSurface((*gpmdev).ppdevGlobal);

    // Init Primary Displays Device Capabilities.
    PDEVOBJ_vGetDeviceCaps(
        (*gpmdev).ppdevGlobal,
        core::ptr::addr_of_mut!((*GdiHandleTable).DevCaps),
    );

    TRUE as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDestroyPrimarySurface() {
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDestroyPrimarySurface() {
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEnumHDev() -> PPDEVOBJ {
    // I guess we will soon have more than one primary surface.
    // This will do for now.
    (*gpmdev).ppdevGlobal
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEnumHDev() -> PPDEVOBJ {
    // I guess we will soon have more than one primary surface.
    // This will do for now.
    (*gpmdev).ppdevGlobal
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDrawEscape(
    hdc: HDC,
    iEsc: INT,
    cjIn: INT,
    pjIn: LPSTR,
) -> INT {
    0
}
