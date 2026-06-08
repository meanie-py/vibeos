//! Rust translation of `drivers/wmi/wmilib.c` — the ReactOS WMI library
//! (`wmilib.sys`), an export-only kernel DLL (cmake `set_module_type ...
//! ENTRYPOINT 0`) exporting three Windows-Management-Instrumentation helpers.
//!
//! Faithful port of the C: all three are unimplemented stubs. The C bodies do
//! `DPRINT1("... unimplemented\n")` (a NDEBUG-compiled-out debug print, dropped
//! here) and return `STATUS_SUCCESS`; `WmiSystemControl` additionally writes
//! `*IrpDisposition = IrpNotCompleted` ("Irp is not completed") when the
//! pointer is non-NULL. No state, no DDK calls → self-contained (no ntkrnl-sys).
//!
//! ABI: wmilib.h declares the Wmi* fns `NTAPI` → `#[no_mangle] extern "system"`
//! (stdcall `_Name@N` on i386, cdecl on amd64/arm64). The 3 exports are driven
//! by wmilib.spec (spec2def .def); cmake keeps the MODULE link.
//!
//! crate-type = staticlib → `libwmilib_rust.a` (cohabitation; wmilib.c kept on
//! disk per the never-delete rule).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

use core::ffi::c_void;

type NTSTATUS = i32;
type ULONG = u32;
type CCHAR = i8;
type PVOID = *mut c_void;

const STATUS_SUCCESS: NTSTATUS = 0;

// enum _SYSCTL_IRP_DISPOSITION { IrpProcessed=0, IrpNotCompleted=1, ... }
// (sdk/include/ddk/wmilib.h). PSYSCTL_IRP_DISPOSITION points at a C int.
const IRP_NOT_COMPLETED: i32 = 1;

#[no_mangle]
pub unsafe extern "system" fn WmiCompleteRequest(
    _DeviceObject: PVOID, // PDEVICE_OBJECT
    _Irp: PVOID,          // PIRP
    _Status: NTSTATUS,
    _BufferUsed: ULONG,
    _PriorityBoost: CCHAR,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmiFireEvent(
    _DeviceObject: PVOID,   // PDEVICE_OBJECT
    _Guid: *const c_void,   // LPCGUID
    _InstanceIndex: ULONG,
    _EventDataSize: ULONG,
    _EventData: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmiSystemControl(
    _WmiLibInfo: PVOID,         // PWMILIB_CONTEXT
    _DeviceObject: PVOID,       // PDEVICE_OBJECT
    _Irp: PVOID,                // PIRP
    IrpDisposition: *mut i32,   // PSYSCTL_IRP_DISPOSITION (OUT)
) -> NTSTATUS {
    // Return info that the Irp is not completed.
    if !IrpDisposition.is_null() {
        core::ptr::write_volatile(IrpDisposition, IRP_NOT_COMPLETED);
    }
    STATUS_SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
