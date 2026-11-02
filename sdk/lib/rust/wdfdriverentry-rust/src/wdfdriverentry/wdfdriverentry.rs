use crate::prelude::*;
use ntkrnl_sys::ddk::{PDRIVER_OBJECT, PDRIVER_UNLOAD, PUNICODE_STRING, RtlCopyUnicodeString};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_D50216DB97F4: [u16; 12] = [
    0x004Bu16, 0x006D, 0x0064, 0x0066, 0x004C, 0x0069, 0x0062, 0x0072, 0x0061, 0x0072,
    0x0079, 0x0000,
];

#[no_mangle]
pub static mut WdfFunctions_01017: *const WDFFUNC = ::core::ptr::null::<WDFFUNC>();

#[no_mangle]
pub static mut WdfDriverGlobals: PWDF_DRIVER_GLOBALS =
    ::core::ptr::null::<_WDF_DRIVER_GLOBALS>() as *mut _WDF_DRIVER_GLOBALS;

#[no_mangle]
pub static mut BindInfo: WDF_BIND_INFO = unsafe {
    _WDF_BIND_INFO {
        Size: ::core::mem::size_of::<WDF_BIND_INFO>() as ULONG,
        Component: RUSTCONV_C2R_WSTR_D50216DB97F4.as_ptr() as PWCHAR,
        Version: _WDF_VERSION {
            Major: __WDF_MAJOR_VERSION as WDF_MAJOR_VERSION,
            Minor: __WDF_MINOR_VERSION as WDF_MINOR_VERSION,
            Build: __WDF_BUILD_NUMBER as WDF_BUILD_NUMBER,
        },
        FuncCount: WdfFunctionTableNumEntries as ::core::ffi::c_int as ULONG,
        FuncTable: &raw const WdfFunctions_01017 as *mut *const WDFFUNC as *mut WDFFUNC,
        Module: ::core::ptr::null::<_LIBRARY_MODULE>() as *mut _LIBRARY_MODULE,
    }
};

#[no_mangle]
pub static mut pOriginalUnload: PDRIVER_UNLOAD = None;

#[no_mangle]
pub static mut gRegistryPath: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
};

pub const WDFENTRY_TAG: u32 = 0x4546_4457; // 'EFDW'

unsafe extern "C" fn FxDriverUnloadCommon() {
    WdfVersionUnbind(
        core::ptr::addr_of_mut!(gRegistryPath),
        core::ptr::addr_of_mut!(BindInfo),
        WdfDriverGlobals as PWDF_COMPONENT_GLOBALS,
    );
}
#[no_mangle]
pub unsafe extern "system" fn FxDriverUnload(DriverObject: PDRIVER_OBJECT) {
    if pOriginalUnload.is_some() {
        (pOriginalUnload.unwrap_unchecked())(DriverObject);
    }
    FxDriverUnloadCommon();
}
#[no_mangle]
pub unsafe extern "system" fn FxDriverEntry(
    mut DriverObject: PDRIVER_OBJECT,
    mut RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;

    if DriverObject.is_null() {
        return DriverEntry(DriverObject, RegistryPath);
    }

    // Copy registry path
    gRegistryPath.MaximumLength = ((*RegistryPath).Length as usize)
        .wrapping_add(core::mem::size_of_val(&UNICODE_NULL))
        as USHORT;
    gRegistryPath.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        gRegistryPath.MaximumLength as SIZE_T,
        WDFENTRY_TAG as ULONG,
    ) as PWCH;

    if gRegistryPath.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyUnicodeString(core::ptr::addr_of_mut!(gRegistryPath), RegistryPath);

    // Bind wdf driver to framework
    status = WdfVersionBind(
        DriverObject,
        RegistryPath,
        core::ptr::addr_of_mut!(BindInfo),
        core::ptr::addr_of_mut!(WdfDriverGlobals) as *mut PWDF_COMPONENT_GLOBALS,
    );

    if !NT_SUCCESS!(status) {
        return status;
    }

    // Call original entry point
    status = DriverEntry(DriverObject, RegistryPath);
    if !NT_SUCCESS!(status) {
        FxDriverUnloadCommon();
        return status;
    }

    if (*WdfDriverGlobals).DisplaceDriverUnload != 0 {
        pOriginalUnload = (*DriverObject).DriverUnload;
        (*DriverObject).DriverUnload =
            Some(FxDriverUnload);
    }
    return STATUS_SUCCESS;
}
