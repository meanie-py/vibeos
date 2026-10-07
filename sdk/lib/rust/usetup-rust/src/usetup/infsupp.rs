use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut SpInfExports: SPINF_EXPORTS = unsafe {
    _SPINF_EXPORTS {
        SpInfCloseInfFile: Some(SetupCloseInfFile),
        SpInfFindFirstLine: Some(
            SetupFindFirstLineW,
        ),
        SpInfFindNextLine: Some(
            SetupFindNextLine,
        ),
        SpInfGetFieldCount: Some(SetupGetFieldCount),
        SpInfGetBinaryField: Some(
            SetupGetBinaryField,
        ),
        SpInfGetIntField: Some(
            SetupGetIntField,
        ),
        SpInfGetMultiSzField: Some(
            SetupGetMultiSzFieldW,
        ),
        SpInfGetStringField: Some(
            SetupGetStringFieldW,
        ),
        SpInfGetField: Some(pSetupGetField),
        SpInfOpenInfFile: Some(
            SetupOpenInfFileExW,
        ),
    }
};

#[no_mangle]
pub unsafe extern "system" fn SetupCloseInfFile(InfHandle: HINF) {
    if InfHandle == INVALID_HANDLE_VALUE {
        return;
    }
    InfCloseFile(InfHandle);
}

#[no_mangle]
pub unsafe extern "system" fn SetupFindFirstLineW(
    InfHandle: HINF,
    Section: PCWSTR,
    Key: PCWSTR,
    Context: PINFCONTEXT,
) -> BOOL {
    let mut pContext: PINFCONTEXT = core::ptr::null_mut();
    let mut ret: BOOL;

    if InfHandle == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }

    ret = InfFindFirstLine(InfHandle, Section, Key, &mut pContext) as BOOL;
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    memcpy(
        Context as *mut core::ffi::c_void,
        pContext as *const core::ffi::c_void,
        core::mem::size_of::<INFCONTEXT>(),
    );
    InfFreeContext(pContext);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupFindNextLine(
    ContextIn: PINFCONTEXT,
    ContextOut: PINFCONTEXT,
) -> BOOL {
    return (InfFindNextLine(ContextIn, ContextOut) != 0) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetFieldCount(Context: PINFCONTEXT) -> u32 {
    return InfGetFieldCount(Context) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupGetField(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
) -> PCWSTR {
    let mut Data: PWSTR = core::ptr::null_mut();
    if InfGetDataField(Context, FieldIndex, &mut Data) == 0 {
        return core::ptr::null();
    }
    return Data;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetBinaryField(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    ReturnBuffer: PUCHAR,
    ReturnBufferSize: ULONG,
    RequiredSize: PULONG,
) -> BOOL {
    return (InfGetBinaryField(
        Context,
        FieldIndex,
        ReturnBuffer,
        ReturnBufferSize,
        RequiredSize,
    ) != 0) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetIntField(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    IntegerValue: *mut INT,
) -> BOOL {
    return (InfGetIntField(Context, FieldIndex, IntegerValue) != 0) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetMultiSzFieldW(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    ReturnBuffer: PWSTR,
    ReturnBufferSize: ULONG,
    RequiredSize: PULONG,
) -> BOOL {
    return (InfGetMultiSzField(
        Context,
        FieldIndex,
        ReturnBuffer,
        ReturnBufferSize,
        RequiredSize,
    ) != 0) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetStringFieldW(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    ReturnBuffer: PWSTR,
    ReturnBufferSize: ULONG,
    RequiredSize: PULONG,
) -> BOOL {
    return (InfGetStringField(
        Context,
        FieldIndex,
        ReturnBuffer,
        ReturnBufferSize,
        RequiredSize,
    ) != 0) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenInfFileExW(
    FileName: PCWSTR,
    InfClass: PCWSTR,
    InfStyle: u32,
    LocaleId: LCID,
    ErrorLine: PUINT,
) -> *mut core::ffi::c_void {
    let mut hInf: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut FileNameU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ErrorLineUL: u32 = 0;
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut FileNameU, FileName);
    Status = InfOpenFile(
        &mut hInf,
        &mut FileNameU,
        LANGIDFROMLCID!(LocaleId),
        &mut ErrorLineUL,
    );
    *ErrorLine = ErrorLineUL as u32;
    if ((NT_SUCCESS!(Status)) == 0) {
        return INVALID_HANDLE_VALUE;
    }

    return hInf;
}

#[no_mangle]
pub unsafe extern "system" fn INF_OpenBufferedFileA(
    FileBuffer: PSTR,
    FileSize: ULONG,
    InfClass: PCSTR,
    InfStyle: DWORD,
    LocaleId: LCID,
    ErrorLine: PUINT,
) -> HINF {
    let mut hInf: HINF = core::ptr::null_mut();
    let mut ErrorLineUL: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    Status = InfOpenBufferedFile(
        &mut hInf,
        (FileBuffer).cast(),
        FileSize,
        LANGIDFROMLCID!(LocaleId),
        &mut ErrorLineUL,
    );
    *ErrorLine = ErrorLineUL as UINT;
    if ((NT_SUCCESS!(Status)) == 0) {
        return INVALID_HANDLE_VALUE;
    }

    return hInf;
}
