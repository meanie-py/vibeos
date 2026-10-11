use crate::prelude::*;
use ntkrnl_sys::ddk::{LPWSTR, va_list};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gadrvfn: [DRVFN; 13] = unsafe {
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
                BmfdEnablePDEV,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvCompletePDEV as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(DHPDEV, HDEV) -> ()>, PFN>(
                Some(BmfdCompletePDEV),
            ),
        },
        _DRVFN {
            iFunc: INDEX_DrvDisablePDEV as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(DHPDEV) -> ()>, PFN>(Some(
                BmfdDisablePDEV,
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
                BmfdLoadFontFile,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvUnloadFontFile as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(ULONG_PTR) -> BOOL>, PFN>(
                Some(BmfdUnloadFontFile),
            ),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryFontFile as ULONG,
            pfn: ::core::mem::transmute::<
                Option<unsafe extern "system" fn(ULONG_PTR, ULONG, ULONG, *mut ULONG) -> LONG>,
                PFN,
            >(Some(
                BmfdQueryFontFile,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryFontCaps as ULONG,
            pfn: ::core::mem::transmute::<
                Option<unsafe extern "system" fn(ULONG, *mut ULONG) -> LONG>,
                PFN,
            >(Some(
                BmfdQueryFontCaps,
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
                BmfdQueryFontTree,
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
                BmfdQueryFont,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvFree as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(PVOID, ULONG_PTR) -> ()>, PFN>(
                Some(BmfdFree),
            ),
        },
        _DRVFN {
            iFunc: INDEX_DrvQueryGlyphAttrs as ULONG,
            pfn: ::core::mem::transmute::<
                Option<unsafe extern "system" fn(*mut FONTOBJ, ULONG) -> PFD_GLYPHATTR>,
                PFN,
            >(Some(
                BmfdQueryGlyphAttrs,
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
                BmfdQueryFontData,
            )),
        },
        _DRVFN {
            iFunc: INDEX_DrvDestroyFont as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "system" fn(*mut FONTOBJ) -> ()>, PFN>(
                Some(BmfdDestroyFont),
            ),
        },
    ]
};

// NOTE: INDEX_* values are from winddi.h; verify against the installed DDK headers.

#[no_mangle]
pub unsafe extern "C" fn DbgPrint_v(mut Format: PCHAR, ap_list: *mut ::core::ffi::c_void) -> ULONG {
    // va_start/va_end live in the C trampoline that calls this twin; `ap_list` is its `&args`.
    let args_0: va_list = *(ap_list as *mut va_list);
    EngDebugPrint(b"Bmfd: \0".as_ptr() as PCHAR, Format, args_0);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn BmfdEnableDriver(
    iEngineVersion: ULONG,
    cj: ULONG,
    pded: PDRVENABLEDATA,
) -> BOOL {
    DbgPrint(b"BmfdEnableDriver()\n\0".as_ptr() as PCHAR);

    /* Check parameter */
    if (cj as usize) < core::mem::size_of::<DRVENABLEDATA>() {
        return (FALSE) as BOOL;
    }

    /* Fill DRVENABLEDATA */
    (*pded).c = (core::mem::size_of::<[DRVFN; 13]>()
        / core::mem::size_of::<DRVFN>()) as ULONG;
    (*pded).pdrvfn = core::ptr::addr_of_mut!(gadrvfn) as *mut DRVFN;
    (*pded).iDriverVersion = DDI_DRIVER_VERSION_NT5 as ULONG;

    /* Success */
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn BmfdEnablePDEV(
    pdm: *mut DEVMODEW,
    pwszLogAddress: LPWSTR,
    cPat: ULONG,
    phsurfPatterns: *mut HSURF,
    cjCaps: ULONG,
    pdevcaps: *mut ULONG,
    cjDevInfo: ULONG,
    pdi: *mut DEVINFO,
    hdev: HDEV,
    pwszDeviceName: LPWSTR,
    hDriver: HANDLE,
) -> DHPDEV {
    DbgPrint(
        b"BmfdEnablePDEV(hdev=%p)\n\0".as_ptr() as PCHAR,
        hdev,
    );

    /* Return a dummy DHPDEV */
    return 1 as usize as PVOID as DHPDEV;
}

#[no_mangle]
pub unsafe extern "system" fn BmfdCompletePDEV(dhpdev: DHPDEV, hdev: HDEV) {
    // DbgPrint("BmfdCompletePDEV()\n");
}

#[no_mangle]
pub unsafe extern "system" fn BmfdDisablePDEV(dhpdev: DHPDEV) {
    // DbgPrint("BmfdDisablePDEV()\n");
}
