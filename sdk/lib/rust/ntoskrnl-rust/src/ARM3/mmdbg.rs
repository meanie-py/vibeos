use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{
    PULONGLONG,
    MmIsAddressValid,
    MmIsSessionAddress,
    KeInvalidateTlbEntry,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub static mut MiDebugMapping: PVOID = MI_DEBUG_MAPPING as PVOID;
#[no_mangle]
pub static mut MmDebugPte: PMMPTE = core::ptr::null_mut();

unsafe fn MiDbgTranslatePhysicalAddress(
    PhysicalAddress: ULONG64,
    Flags: ULONG,
) -> PVOID {
    let Pfn: PFN_NUMBER;
    let mut TempPte: MMPTE;
    let MappingBaseAddress: PVOID;

    if MmDebugPte.is_null() {
        return core::ptr::null_mut();
    }

    if (Flags & (MMDBG_COPY_CACHED |
                 MMDBG_COPY_UNCACHED |
                 MMDBG_COPY_WRITE_COMBINED)) != 0
    {
        return core::ptr::null_mut();
    }

    MappingBaseAddress = MiPteToAddress(MmDebugPte);

    TempPte = ValidKernelPte;

    Pfn = (PhysicalAddress >> PAGE_SHIFT) as PFN_NUMBER;

    if MiGetPfnEntry(Pfn).is_null() {
        return core::ptr::null_mut();
    } else {
        TempPte.set_page_frame_number(Pfn);
    }

    *MmDebugPte = TempPte;
    KeInvalidateTlbEntry(MappingBaseAddress);

    (MappingBaseAddress as ULONG_PTR + (PhysicalAddress as ULONG_PTR & (PAGE_SIZE - 1))) as PVOID
}

unsafe fn MiDbgUnTranslatePhysicalAddress() {
    let MappingBaseAddress: PVOID;

    MappingBaseAddress = MiPteToAddress(MmDebugPte);

    (*MmDebugPte).Long = 0;
    KeInvalidateTlbEntry(MappingBaseAddress);
}

#[no_mangle]
pub unsafe extern "system" fn MmDbgCopyMemory(
    Address: ULONG64,
    Buffer: PVOID,
    Size: ULONG,
    mut Flags: ULONG,
) -> NTSTATUS {
    let Buffer = opaque_ptr(Buffer);
    let mut TargetAddress: PVOID;
    let CopyDestination: PVOID;
    let CopySource: PVOID;

    if (Flags & MMDBG_COPY_UNSAFE) == 0 {
        static mut Warned: BOOLEAN = 0u8;
        if Warned == 0u8 {
            Warned = 1u8;
        }
    }

    if (Size != 1) &&
       (Size != 2) &&
       (Size != 4) &&
       (Size != 8)
    {
        return STATUS_INVALID_PARAMETER_3;
    }

    if (Address & ((Size as ULONG64) - 1)) != 0 {
        return STATUS_INVALID_PARAMETER_3;
    }

    if (Flags & MMDBG_COPY_PHYSICAL) != 0 {
        TargetAddress = MiDbgTranslatePhysicalAddress(Address, Flags);

        if TargetAddress.is_null() {
            return STATUS_UNSUCCESSFUL;
        }
    } else {
        TargetAddress = Address as ULONG_PTR as PVOID;

        if MmIsAddressValid(TargetAddress) == 0u8 {
            return STATUS_UNSUCCESSFUL;
        }

        if MmIsSessionAddress(TargetAddress) != 0u8 {
        }

        let PointerPte = MiAddressToPte(TargetAddress);
        if ((Flags & MMDBG_COPY_WRITE) != 0) &&
           (((*PointerPte).Long & 0x2) == 0)
        {
            Flags |= MMDBG_COPY_PHYSICAL;

            let PhysicalAddress = ((*PointerPte).page_frame_number() << PAGE_SHIFT) as ULONG64;
            let PhysicalAddress = PhysicalAddress + (Address & (PAGE_SIZE as ULONG64 - 1));

            TargetAddress = MiDbgTranslatePhysicalAddress(PhysicalAddress, Flags);

            if TargetAddress.is_null() {
                return STATUS_UNSUCCESSFUL;
            }
        }
    }

    if (Flags & MMDBG_COPY_WRITE) != 0 {
        CopyDestination = TargetAddress;
        CopySource = Buffer;
    } else {
        CopyDestination = Buffer;
        CopySource = TargetAddress;
    }

    match Size {
        1 => {
            *(CopyDestination as *mut UCHAR) = *(CopySource as *const UCHAR);
        }
        2 => {
            *(CopyDestination as *mut USHORT) = *(CopySource as *const USHORT);
        }
        4 => {
            *(CopyDestination as *mut ULONG) = *(CopySource as *const ULONG);
        }
        8 => {
            *(CopyDestination as PULONGLONG) = *(CopySource as *const ULONGLONG);
        }
        // C is `/* Size is sanitized above */ DEFAULT_UNREACHABLE;` (mmdbg.c:276)
        // — Size is validated to 1/2/4/8 before this switch, so this is a
        // faithful translation of an intentional optimizer hint, not an assert.
        _ => core::hint::unreachable_unchecked(), // nolint: assert-to-ub (C DEFAULT_UNREACHABLE, Size pre-validated)
    }

    if (Flags & MMDBG_COPY_PHYSICAL) != 0 {
        MiDbgUnTranslatePhysicalAddress();
    }

    STATUS_SUCCESS
}
