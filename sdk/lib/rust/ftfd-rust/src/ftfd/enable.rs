use crate::prelude::*;
use ntkrnl_sys::ddk::LPWSTR;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gadrvfn: [DRVFN; 12] = unsafe {
    [
        _DRVFN {
            iFunc: INDEX_DrvEnablePDEV as ULONG,
            pfn: ::core::mem::transmute::<
                Option<
                    unsafe extern "system" fn(
                        *mut DEVMODEW,
                        LPWSTR,
                        ULONG,
                        *mut HSURF,
                        ULONG,
                        *mut ULONG,
                        ULONG,
                        *mut DEVINFO,
                        HDEV,
                        LPWSTR,
                        HANDLE,
                    ) -> DHPDEV,
                >,
                PFN,
            >(Some(
                FtfdEnablePDEV,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvCompletePDEV as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(DHPDEV, HDEV) -> ()>, PFN>(
                Some(FtfdCompletePDEV),
            ),
        },
        _DRVFN {
            iFunc: INDEX_DrvDisablePDEV as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(DHPDEV) -> ()>, PFN>(Some(
                FtfdDisablePDEV,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvLoadFontFile as ULONG,
            pfn: ::core::mem::transmute::<
                Option<
                    unsafe extern "system" fn(
                        ULONG,
                        *mut ULONG_PTR,
                        *mut PVOID,
                        *mut ULONG,
                        *mut DESIGNVECTOR,
                        ULONG,
                        ULONG,
                    ) -> ULONG_PTR,
                >,
                PFN,
            >(Some(
                FtfdLoadFontFile,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvUnloadFontFile as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(ULONG_PTR) -> BOOL>, PFN>(
                Some(FtfdUnloadFontFile),
            ),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryFontFile as ULONG,
            pfn: ::core::mem::transmute::<
                Option<unsafe extern "system" fn(ULONG_PTR, ULONG, ULONG, *mut ULONG) -> LONG>,
                PFN,
            >(Some(
                FtfdQueryFontFile,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryFontCaps as ULONG,
            pfn: ::core::mem::transmute::<
                Option<unsafe extern "system" fn(ULONG, *mut ULONG) -> LONG>,
                PFN,
            >(Some(
                FtfdQueryFontCaps,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryFontTree as ULONG,
            pfn: ::core::mem::transmute::<
                Option<
                    unsafe extern "system" fn(DHPDEV, ULONG_PTR, ULONG, ULONG, *mut ULONG_PTR) -> PVOID,
                >,
                PFN,
            >(Some(
                FtfdQueryFontTree,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryFont as ULONG,
            pfn: ::core::mem::transmute::<
                Option<
                    unsafe extern "system" fn(DHPDEV, ULONG_PTR, ULONG, *mut ULONG_PTR) -> PIFIMETRICS,
                >,
                PFN,
            >(Some(
                FtfdQueryFont,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvFree as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(PVOID, ULONG_PTR) -> ()>, PFN>(
                Some(FtfdFree),
            ),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryGlyphAttrs as ULONG,
            pfn: ::core::mem::transmute::<
                Option<unsafe extern "system" fn(*mut FONTOBJ, ULONG) -> PFD_GLYPHATTR>,
                PFN,
            >(Some(
                FtfdQueryGlyphAttrs,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryFontData as ULONG,
            pfn: ::core::mem::transmute::<
                Option<
                    unsafe extern "system" fn(
                        DHPDEV,
                        *mut FONTOBJ,
                        ULONG,
                        HGLYPH,
                        *mut GLYPHDATA,
                        PVOID,
                        ULONG,
                    ) -> LONG,
                >,
                PFN,
            >(Some(
                FtfdQueryFontData,
            )),
        },
    ]
};

#[no_mangle]
pub static mut gftlibrary: FT_Library =
    ::core::ptr::null::<FT_LibraryRec_>() as *mut FT_LibraryRec_;

// INDEX_Drv* constants come from winddi.h and are expected to be provided by the caller.

#[no_mangle]
pub unsafe extern "system" fn FtfdEnableDriver(
    iEngineVersion: ULONG,
    cj: ULONG,
    pded: PDRVENABLEDATA,
) -> BOOL {
    let mut fterror: FT_Error;

    FT_Message(b"FtfdEnableDriver()\n\0".as_ptr() as *const i8);

    /* Check parameter */
    if (cj as usize) < core::mem::size_of::<DRVENABLEDATA>() {
        return (FALSE) as BOOL;
    }

    /* Initialize freetype library */
    fterror = FT_Init_FreeType(core::ptr::addr_of_mut!(gftlibrary));
    if fterror != 0 {
        FT_Message(
            b"an error occurred during library initialization: %ld.\n\0".as_ptr() as *const i8,
            fterror,
        );
        return (FALSE) as BOOL;
    }

    /* Fill DRVENABLEDATA */
    (*pded).c = (core::mem::size_of::<[DRVFN; 12]>() / core::mem::size_of::<DRVFN>()) as ULONG;
    (*pded).pdrvfn = core::ptr::addr_of_mut!(gadrvfn) as *mut DRVFN;
    (*pded).iDriverVersion = DDI_DRIVER_VERSION_NT5 as ULONG;

    /* Success */
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FtfdEnablePDEV(
    mut pdm: *mut DEVMODEW,
    mut pwszLogAddress: LPWSTR,
    mut cPat: ULONG,
    mut phsurfPatterns: *mut HSURF,
    mut cjCaps: ULONG,
    mut pdevcaps: *mut ULONG,
    mut cjDevInfo: ULONG,
    mut pdi: *mut DEVINFO,
    mut hdev: HDEV,
    mut pwszDeviceName: LPWSTR,
    mut hDriver: HANDLE,
) -> DHPDEV {
    FT_Message(
        b"FtfdEnablePDEV(hdev=%p)\n\0".as_ptr() as *const i8,
        hdev,
    );
    __debugbreak();

    /* Return a dummy DHPDEV */
    return 1 as usize as PVOID as DHPDEV;
}
#[no_mangle]
pub unsafe extern "system" fn FtfdCompletePDEV(dhpdev: DHPDEV, hdev: HDEV) {
    FT_Message(b"FtfdCompletePDEV()\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn FtfdDisablePDEV(dhpdev: DHPDEV) {
    FT_Message(b"FtfdDisablePDEV()\n\0".as_ptr() as *const i8);
    /* Nothing to do */
}
