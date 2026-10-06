//! Translated from `ntoskrnl/ps/i386/psldt.c` — LDT support for x86.

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::constants::STATUS_INFO_LENGTH_MISMATCH;
use ntkrnl_sys::ex::ExFreePoolWithTag;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NO_LDT: NTSTATUS = 0xC0000117_u32 as NTSTATUS;
const TAG_KERNEL: ULONG = 0x2020654B;

extern "system" {
    fn Ke386GetGdtEntryThread(
        Thread: PKTHREAD,
        Offset: ULONG,
        Descriptor: PKGDTENTRY,
    ) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PspDeleteLdt(Process: PEPROCESS) {
    // FIXME — LdtInformation must be null as long as we don't implement VDMs.
    ntkrnl_sys::ASSERT!((*Process).LdtInformation.is_null());
}

#[no_mangle]
pub unsafe extern "system" fn PspDeleteVdmObjects(Process: PEPROCESS) {
    if (*Process).VdmObjects.is_null() {
        return;
    }

    // FIXME: need to do more than just freeing the main VdmObjects member.
    ExFreePoolWithTag((*Process).VdmObjects, TAG_KERNEL);
    (*Process).VdmObjects = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn PspQueryDescriptorThread(
    Thread: PETHREAD,
    ThreadInformation: PVOID,
    ThreadInformationLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    if ThreadInformationLength as usize != core::mem::size_of::<DESCRIPTOR_TABLE_ENTRY>() {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    // ThreadInformation is a pool copy by the time NtQueryInformationThread calls
    // this (its OutputBounce); ReturnLength is still the caller's pointer.
    let DescriptorEntry: DESCRIPTOR_TABLE_ENTRY =
        core::ptr::read_unaligned(ThreadInformation as *const DESCRIPTOR_TABLE_ENTRY);

    // GDT selector? (LDT bit is 0x4)
    if (DescriptorEntry.Selector & 0x4) == 0 {
        let mut Descriptor: LDT_ENTRY = core::mem::zeroed();
        let Status = Ke386GetGdtEntryThread(
            core::ptr::addr_of_mut!((*Thread).Tcb),
            DescriptorEntry.Selector & 0xFFFFFFF8,
            core::ptr::addr_of_mut!(Descriptor) as PKGDTENTRY,
        );
        if Status < 0 {
            return Status;
        }

        let UserOut = ThreadInformation as *mut DESCRIPTOR_TABLE_ENTRY;
        core::ptr::write_unaligned(
            core::ptr::addr_of_mut!((*UserOut).Descriptor),
            Descriptor,
        );
        if !ReturnLength.is_null() {
            // C: inside the caller's _SEH2_TRY, whose handler returns the code.
            let length = core::mem::size_of::<LDT_ENTRY>() as ULONG;
            if let Err(s) = ntkrnl_sys::probe::safe_copy_raw(
                ReturnLength as PVOID,
                &length as *const ULONG as *const core::ffi::c_void,
                core::mem::size_of::<ULONG>(),
            ) {
                return s;
            }
        }

        STATUS_SUCCESS
    } else {
        // LDT selectors only meaningful for VDM, which we don't implement.
        ntkrnl_sys::ASSERT!((*(*Thread).ThreadsProcess).LdtInformation.is_null());
        STATUS_NO_LDT
    }
}
