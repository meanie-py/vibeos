use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut tblCheckInBuffer: [DWORD; 22] = [
    0 as ::core::ffi::c_int as DWORD,
    ::core::mem::size_of::<DDCLOSEHANDLE>() as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    ::core::mem::size_of::<HANDLE>() as DWORD,
    ::core::mem::size_of::<DDGETFIELDNUMIN>() as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    ::core::mem::size_of::<DDFLIPOVERLAY>() as DWORD,
    ::core::mem::size_of::<DDFLIPVIDEOPORT>() as DWORD,
    ::core::mem::size_of::<DDGETAUTOFLIPIN>() as DWORD,
    ::core::mem::size_of::<DDGETAUTOFLIPIN>() as DWORD,
    ::core::mem::size_of::<DDREGISTERCALLBACK>() as DWORD,
    ::core::mem::size_of::<DDREGISTERCALLBACK>() as DWORD,
    ::core::mem::size_of::<DDGETPOLARITYIN>() as DWORD,
    ::core::mem::size_of::<DDOPENVPCAPTUREDEVICEIN>() as DWORD,
    ::core::mem::size_of::<DDADDVPCAPTUREBUFF>() as DWORD,
    ::core::mem::size_of::<HANDLE>() as DWORD,
];

#[no_mangle]
pub static mut tblCheckOutBuffer: [DWORD; 22] = [
    ::core::mem::size_of::<DDGETVERSIONNUMBER>() as DWORD,
    ::core::mem::size_of::<DWORD>() as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    ::core::mem::size_of::<DDGETKERNELCAPSOUT>() as DWORD,
    ::core::mem::size_of::<DDGETFIELDNUMOUT>() as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    0 as ::core::ffi::c_int as DWORD,
    ::core::mem::size_of::<DWORD>() as DWORD,
    ::core::mem::size_of::<DWORD>() as DWORD,
    ::core::mem::size_of::<DDGETAUTOFLIPOUT>() as DWORD,
    ::core::mem::size_of::<DDGETAUTOFLIPOUT>() as DWORD,
    ::core::mem::size_of::<DWORD>() as DWORD,
    ::core::mem::size_of::<DWORD>() as DWORD,
    ::core::mem::size_of::<DDGETPOLARITYOUT>() as DWORD,
    ::core::mem::size_of::<DDOPENVPCAPTUREDEVICEOUT>() as DWORD,
    ::core::mem::size_of::<DWORD>() as DWORD,
    ::core::mem::size_of::<DWORD>() as DWORD,
];

#[no_mangle]
pub static mut gDxApiEntryPoint: [DRVFN; 22] = unsafe {
    [
        _DRVFN {
            iFunc: (DD_DXAPI_GETVERSIONNUMBER - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(PVOID, LPDDGETVERSIONNUMBER) -> ()>,
                PFN,
            >(Some(
                DxGetVersionNumber as unsafe extern "C" fn(PVOID, LPDDGETVERSIONNUMBER) -> (),
            )),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_CLOSEHANDLE - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxCloseHandle as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_OPENDIRECTDRAW - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxOpenDirectDraw as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_OPENSURFACE - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxOpenSurface as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_OPENVIDEOPORT - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxOpenVideoPort as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_GETKERNELCAPS - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxGetKernelCaps as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_GET_VP_FIELD_NUMBER - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxGetFieldNumber as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_SET_VP_FIELD_NUMBER - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxSetFieldNumber as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_SET_VP_SKIP_FIELD - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxSetSkipPattern as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_GET_SURFACE_STATE - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxGetSurfaceState as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_SET_SURFACE_STATE - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxSetSurfaceState as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_LOCK - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxLock as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_FLIP_OVERLAY - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxFlipOverlay as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_FLIP_VP - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxFlipVideoPort as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_GET_CURRENT_VP_AUTOFLIP_SURFACE - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxGetCurrentAutoflip as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_GET_LAST_VP_AUTOFLIP_SURFACE - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxGetPreviousAutoflip as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_REGISTER_CALLBACK - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxRegisterEvent as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_UNREGISTER_CALLBACK - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxUnregisterEvent as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_GET_POLARITY - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxGetPolarity as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_OPENVPCAPTUREDEVICE - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxOpenVpCatureDevice as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_ADDVPCAPTUREBUFFER - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxAddVpCaptureBuffer as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
        _DRVFN {
            iFunc: (DD_DXAPI_FLUSHVPCAPTUREBUFFERS - DD_FIRST_DXAPI) as ULONG,
            pfn: ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, PVOID) -> ()>, PFN>(
                Some(DxFlushVpCaptureBuffs as unsafe extern "C" fn(PVOID, PVOID) -> ()),
            ),
        },
    ]
};

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(Context1: PVOID, Context2: PVOID) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DxApiGetVersion() -> ULONG {
    0x402
}
#[no_mangle]
pub unsafe extern "system" fn DxApi(
    mut dwFunctionNum: DWORD,
    mut lpvInBuffer: LPVOID,
    mut cbInBuffer: DWORD,
    mut lpvOutBuffer: LPVOID,
    mut cbOutBuffer: DWORD,
) -> DWORD {
    dwFunctionNum = dwFunctionNum.wrapping_sub(DD_FIRST_DXAPI as DWORD);

    if lpvOutBuffer.is_null()
        || dwFunctionNum > ((DD_DXAPI_FLUSHVPCAPTUREBUFFERS - DD_FIRST_DXAPI) as DWORD)
        || gDxApiEntryPoint[dwFunctionNum as usize].pfn.is_none()
        || cbInBuffer != tblCheckInBuffer[dwFunctionNum as usize]
        || cbOutBuffer != tblCheckOutBuffer[dwFunctionNum as usize]
    {
        return 0 as DWORD;
    }

    // Divergence from the C, which calls through PFN, `LONG_PTR (APIENTRY *)()`: stdcall on
    // i386 with no prototype. Every handler in the table is a cdecl `VOID (PVOID, PVOID)`, so
    // the C call disagrees with the callee about who pops the arguments, which only the
    // caller's frame-pointer epilogue hides. Call them as what they are.
    let pfn: unsafe extern "C" fn(PVOID, PVOID) =
        core::mem::transmute(gDxApiEntryPoint[dwFunctionNum as usize].pfn.unwrap_unchecked());
    pfn(lpvInBuffer, lpvOutBuffer);
    return 0 as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn DxApiInitialize(
    p1: PVOID,
    p2: PVOID,
    p3: PVOID,
    p4: PVOID,
    p5: PVOID,
    p6: PVOID,
    p7: PVOID,
    p8: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn DxAutoflipUpdate(
    p1: PVOID,
    p2: PVOID,
    p3: PVOID,
    p4: PVOID,
    p5: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn DxEnableIRQ(p1: PVOID, p2: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn DxLoseObject(p1: PVOID, p2: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn DxUpdateCapture(p1: PVOID, p2: PVOID, p3: PVOID) {
    // UNIMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn DxGetVersionNumber(lpvInBuffer: PVOID, lpvOutBuffer: LPDDGETVERSIONNUMBER) {
    (*lpvOutBuffer).ddRVal = DD_OK as DWORD;
    (*lpvOutBuffer).dwMajorVersion = 1 as DWORD;
    (*lpvOutBuffer).dwMinorVersion = 0 as DWORD;
}

#[no_mangle]
pub unsafe extern "C" fn DxCloseHandle(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxOpenDirectDraw(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxOpenSurface(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxOpenVideoPort(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxGetKernelCaps(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxGetFieldNumber(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxSetFieldNumber(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxSetSkipPattern(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxGetSurfaceState(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxSetSurfaceState(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxLock(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxFlipOverlay(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxFlipVideoPort(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxGetCurrentAutoflip(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxGetPreviousAutoflip(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxRegisterEvent(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxUnregisterEvent(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxGetPolarity(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxOpenVpCatureDevice(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxAddVpCaptureBuffer(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}

#[no_mangle]
pub unsafe extern "C" fn DxFlushVpCaptureBuffs(lpvInBuffer: PVOID, lpvOutBuffer: PVOID) {}
