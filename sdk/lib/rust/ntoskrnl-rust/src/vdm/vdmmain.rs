use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;

use ntkrnl_sys::ddk::{
    ACCESS_MASK, HANDLE,
    KEY_VALUE_BASIC_INFORMATION,
    LARGE_INTEGER,
    _OBJECT_ATTRIBUTES,
    _UNICODE_STRING,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use core::arch::asm;

extern "C" {
    static mut KeI386VirtualIntExtensions: ULONG;
}

const CR4_VME: usize = 0x00000001;
const KF_V86_VIS: u32 = 0x00000001;

#[cfg(target_arch = "x86")]
unsafe fn __readeflags() -> u32 {
    let flags: u32;
    asm!("pushfd", "pop {}", out(reg) flags, options(nomem, nostack));
    flags
}

#[cfg(target_arch = "x86")]
unsafe fn __writeeflags(flags: u32) {
    asm!("push {}", "popfd", in(reg) flags, options(nomem, nostack));
}

#[cfg(target_arch = "x86")]
unsafe fn _disable() {
    asm!("cli", options(nomem, nostack));
}

#[cfg(target_arch = "x86")]
unsafe fn __readcr4() -> usize {
    let cr4: usize;
    asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack));
    cr4
}

#[cfg(target_arch = "x86")]
unsafe fn __writecr4(cr4: usize) {
    asm!("mov cr4, {}", in(reg) cr4, options(nomem, nostack));
}

extern "system" {
    fn RtlInitUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const u16);
    fn ZwOpenKey(KeyHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: *mut _OBJECT_ATTRIBUTES) -> NTSTATUS;
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn ZwQueryValueKey(
        KeyHandle: HANDLE,
        ValueName: *mut _UNICODE_STRING,
        KeyValueInformationClass: ULONG,
        KeyValueInformation: PVOID,
        Length: ULONG,
        ResultLength: *mut ULONG,
    ) -> NTSTATUS;
    fn ZwOpenSection(
        SectionHandle: *mut HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
    ) -> NTSTATUS;
    fn ZwMapViewOfSection(
        SectionHandle: HANDLE,
        ProcessHandle: HANDLE,
        BaseAddress: *mut PVOID,
        ZeroBits: ULONG_PTR,
        CommitSize: SIZE_T,
        SectionOffset: *mut LARGE_INTEGER,
        ViewSize: *mut SIZE_T,
        InheritDisposition: ULONG,
        AllocationType: ULONG,
        Win32Protect: ULONG,
    ) -> NTSTATUS;
    fn ZwUnmapViewOfSection(ProcessHandle: HANDLE, BaseAddress: PVOID) -> NTSTATUS;
    
}
#[allow(unused_imports)]
use crate::ke::krnlinit::KiProcessorBlock;
extern "C" {
    fn KeGetPcr() -> *mut _KPCR;
}

#[repr(C)]
struct _KPCR {
    _padding: [u8; 0x120],
    Prcb: *mut _KPRCB,
}

#[repr(C)]
struct _KPRCB {
    _padding: [u8; 0x60],
    FeatureBits: ULONG,
}

const KEY_READ: ACCESS_MASK = 0x20019;
const SECTION_ALL_ACCESS: ACCESS_MASK = 0xF001F;
const ViewUnmap: ULONG = 0x02;
const KeyValueBasicInformation: ULONG = 0;
const VdmStartExecution: ULONG = 0;
const VdmQueueInterrupt: ULONG = 1;
const VdmDelayInterrupt: ULONG = 2;
const VdmInitialize: ULONG = 3;

extern "system" {
    fn VdmpStartExecution() -> NTSTATUS;
}

// CODE_SEG("INIT")
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "system" fn Ki386VdmEnablePentiumExtentions(enable: BOOLEAN) {
    let eflags: u32;
    let cr4: usize;

    eflags = __readeflags();
    _disable();

    cr4 = __readcr4();
    if enable != 0 {
        __writecr4(cr4 | CR4_VME);
    } else {
        __writecr4(cr4 & !CR4_VME);
    }

    __writeeflags(eflags);
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "system" fn Ki386VdmEnablePentiumExtentions(_enable: BOOLEAN) {
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KeI386VdmInitialize() {
    let mut status: NTSTATUS;
    let mut object_attributes: _OBJECT_ATTRIBUTES;
    let mut reg_handle: HANDLE = core::ptr::null_mut();
    let mut name: _UNICODE_STRING = core::mem::zeroed();
    let mut key_value_info: [u8; core::mem::size_of::<KEY_VALUE_BASIC_INFORMATION>() + 30] = [0; core::mem::size_of::<KEY_VALUE_BASIC_INFORMATION>() + 30];
    let mut return_length: ULONG = 0;

    static WOW_KEY_PATH: [u16; 55] = [
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
        0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
        0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
        0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
        0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
        0x005C, 0x0057, 0x006F, 0x0077, 0x0000,
    ];

    RtlInitUnicodeString(&mut name, WOW_KEY_PATH.as_ptr());
    object_attributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &mut name as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    status = ZwOpenKey(&mut reg_handle, KEY_READ, &mut object_attributes);
    if status < 0 {
        return;
    }

    static DISABLE_VME: [u16; 11] = [
        0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0056, 0x006D, 0x0065, 0x0000,
    ];

    RtlInitUnicodeString(&mut name, DISABLE_VME.as_ptr());
    status = ZwQueryValueKey(
        reg_handle,
        &mut name,
        KeyValueBasicInformation,
        key_value_info.as_mut_ptr() as PVOID,
        key_value_info.len() as ULONG,
        &mut return_length,
    );
    if status < 0 {
        let pcr = KeGetPcr();
        if ((*(*pcr).Prcb).FeatureBits & KF_V86_VIS) != 0 {
            Ki386VdmEnablePentiumExtentions(1u8);
            KeI386VirtualIntExtensions = 1;
        }
    }

    ZwClose(reg_handle);
}

#[no_mangle]
pub unsafe extern "system" fn VdmpInitialize(control_data: PVOID) -> NTSTATUS {
    let control_data = opaque_ptr(control_data);
    let mut object_attributes: _OBJECT_ATTRIBUTES;
    let mut phys_mem_name: _UNICODE_STRING = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut phys_mem_handle: HANDLE = core::ptr::null_mut();
    let mut base_address: PVOID;
    let null_address: PVOID = core::ptr::null_mut();
    let mut offset: LARGE_INTEGER = core::mem::zeroed();
    let mut view_size: ULONG;

    static PHYS_MEM_PATH: [u16; 23] = [
        0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0050, 0x0068,
        0x0079, 0x0073, 0x0069, 0x0063, 0x0061, 0x006C, 0x004D, 0x0065, 0x006D, 0x006F,
        0x0072, 0x0079, 0x0000,
    ];

    RtlInitUnicodeString(&mut phys_mem_name, PHYS_MEM_PATH.as_ptr());
    object_attributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &mut phys_mem_name as *mut _UNICODE_STRING,
        Attributes: OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    status = ZwOpenSection(&mut phys_mem_handle, SECTION_ALL_ACCESS, &mut object_attributes);
    if status < 0 {
        return status;
    }

    unsafe { (*(&mut offset as *mut LARGE_INTEGER as *mut i64)) = 0; }
    view_size = PAGE_SIZE as ULONG;
    base_address = core::ptr::null_mut();
    status = ZwMapViewOfSection(
        phys_mem_handle,
        NtCurrentProcess(),
        &mut base_address,
        0,
        view_size as SIZE_T,
        &mut offset,
        &mut view_size as *mut ULONG as *mut SIZE_T,
        ViewUnmap,
        0,
        PAGE_READWRITE,
    );
    if status < 0 {
        ZwClose(phys_mem_handle);
        return status;
    }

    // VDM: copy physical memory to virtual address 0 (DOS conventional memory area).
    // null_address is intentionally 0 — this is the VDM DOS memory base.
    // Use write_volatile byte-by-byte to avoid the null pointer UB lint.
    for i in 0..(view_size as usize) {
        core::ptr::write_volatile(
            (null_address as usize + i) as *mut u8,
            *((base_address as *const u8).add(i)),
        );
    }

    ZwClose(phys_mem_handle);

    status = ZwUnmapViewOfSection(NtCurrentProcess(), base_address);

    if status < 0 {
        return status;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtVdmControl(
    control_code: ULONG,
    control_data: PVOID,
) -> NTSTATUS {
    let control_data = opaque_ptr(control_data);
    let status: NTSTATUS;

    match control_code {
        VdmStartExecution => {
            status = VdmpStartExecution();
        }
        VdmInitialize => {
            status = VdmpInitialize(control_data);
        }
        _ => {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/vdm/vdmmain.c
//  * PURPOSE:         VDM Support Services
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// Ki386VdmEnablePentiumExtentions(IN BOOLEAN Enable)
// {
//     ULONG EFlags, Cr4;
//
//     /* Save interrupt state and disable them */
//     EFlags = __readeflags();
//     _disable();
//
//     /* Enable or disable VME as required */
//     Cr4 = __readcr4();
//     __writecr4(Enable ? Cr4 | CR4_VME : Cr4 & ~CR4_VME);
//
//     /* Restore interrupt state */
//     __writeeflags(EFlags);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KeI386VdmInitialize(VOID)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE RegHandle;
//     UNICODE_STRING Name;
//     UCHAR KeyValueInfo[sizeof(KEY_VALUE_BASIC_INFORMATION) + 30];
//     ULONG ReturnLength;
//
//     /* Make sure that there is a WOW key */
//     RtlInitUnicodeString(&Name,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\"
//                          L"Control\\Wow");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&RegHandle, KEY_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status)) return;
//
//     /* Check if VME is enabled */
//     RtlInitUnicodeString(&Name, L"DisableVme");
//     Status = ZwQueryValueKey(RegHandle,
//                              &Name,
//                              KeyValueBasicInformation,
//                              &KeyValueInfo,
//                              sizeof(KeyValueInfo),
//                              &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Not present, so check if the CPU supports VME */
//         if (KeGetPcr()->Prcb->FeatureBits & KF_V86_VIS)
//         {
//             /* Enable them. FIXME: Use IPI */
//             Ki386VdmEnablePentiumExtentions(TRUE);
//             KeI386VirtualIntExtensions = TRUE;
//         }
//     }
//
//     /* Close the key */
//     ZwClose(RegHandle);
// }
//
// NTSTATUS
// NTAPI
// VdmpInitialize(PVOID ControlData)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING PhysMemName = RTL_CONSTANT_STRING(L"\\Device\\PhysicalMemory");
//     NTSTATUS Status;
//     HANDLE PhysMemHandle;
//     PVOID BaseAddress;
//     volatile PVOID NullAddress = NULL;
//     LARGE_INTEGER Offset;
//     ULONG ViewSize;
//
//     /* Open the physical memory section */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &PhysMemName,
//                                OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenSection(&PhysMemHandle,
//                            SECTION_ALL_ACCESS,
//                            &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Couldn't open \\Device\\PhysicalMemory\n");
//         return Status;
//     }
//
//     /* Map the BIOS and device registers into the address space */
//     Offset.QuadPart = 0;
//     ViewSize = PAGE_SIZE;
//     BaseAddress = 0;
//     Status = ZwMapViewOfSection(PhysMemHandle,
//                                 NtCurrentProcess(),
//                                 &BaseAddress,
//                                 0,
//                                 ViewSize,
//                                 &Offset,
//                                 &ViewSize,
//                                 ViewUnmap,
//                                 0,
//                                 PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Couldn't map physical memory (%x)\n", Status);
//         ZwClose(PhysMemHandle);
//         return Status;
//     }
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /* Copy the first physical page into the first virtual page */
//         RtlMoveMemory(NullAddress, BaseAddress, ViewSize);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Fail */
//         DPRINT1("Couldn't copy first page (%x)\n", Status);
//         ZwClose(PhysMemHandle);
//         ZwUnmapViewOfSection(NtCurrentProcess(), BaseAddress);
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Close physical memory section handle */
//     ZwClose(PhysMemHandle);
//
//     /* Unmap the section */
//     Status = ZwUnmapViewOfSection(NtCurrentProcess(), BaseAddress);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Couldn't unmap the section (%x)\n", Status);
//         return Status;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtVdmControl(IN ULONG ControlCode,
//              IN PVOID ControlData)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check which control code this is */
//     switch (ControlCode)
//     {
//         /* VDM Execution start */
//         case VdmStartExecution:
//
//             /* Call the sub-function */
//             Status = VdmpStartExecution();
//             break;
//
//         case VdmInitialize:
//
//             /* Call the init sub-function */
//             Status = VdmpInitialize(ControlData);
//             break;
//
//         default:
//
//             /* Unsupported */
//             DPRINT1("Unknown VDM call: %lx\n", ControlCode);
//             Status = STATUS_INVALID_PARAMETER;
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
