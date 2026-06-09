use crate::shim::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{LDR_RESOURCE_INFO, LdrAccessResource, LdrFindResource_U, PIMAGE_RESOURCE_DATA_ENTRY, PMESSAGE_RESOURCE_DATA, PMESSAGE_RESOURCE_ENTRY, va_list};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.

#[no_mangle]
pub unsafe extern "system" fn RtlFindMessage(
    BaseAddress: PVOID,
    Type: ULONG,
    Language: ULONG,
    MessageId: ULONG,
    MessageResourceEntry: *mut PMESSAGE_RESOURCE_ENTRY,
) -> NTSTATUS {
    let mut ResourceInfo: LDR_RESOURCE_INFO = core::mem::zeroed();
    let mut ResourceDataEntry: PIMAGE_RESOURCE_DATA_ENTRY = core::ptr::null_mut();
    let mut MessageTable: PMESSAGE_RESOURCE_DATA = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut EntryOffset: ULONG = 0;
    let mut IdOffset: ULONG = 0;
    let mut MessageEntry: PMESSAGE_RESOURCE_ENTRY;
    let mut i: ULONG;

    ResourceInfo.Type = Type as _;
    ResourceInfo.Name = 1;
    ResourceInfo.Language = Language as _;

    Status = LdrFindResource_U(
        BaseAddress,
        &mut ResourceInfo,
        RESOURCE_DATA_LEVEL as ULONG,
        &mut ResourceDataEntry,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LdrAccessResource(
        BaseAddress,
        ResourceDataEntry,
        &mut MessageTable as *mut PMESSAGE_RESOURCE_DATA as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    i = 0;
    while i < (*MessageTable).NumberOfBlocks {
        let block = (*MessageTable).Blocks.as_ptr().add(i as usize);
        if MessageId >= (*block).LowId && MessageId <= (*block).HighId {
            EntryOffset = (*block).OffsetToEntries;
            IdOffset = MessageId - (*block).LowId;
            break;
        }

        if MessageId < (*block).LowId {
            return STATUS_MESSAGE_NOT_FOUND;
        }

        i += 1;
    }

    let _ = EntryOffset;

    if (*MessageTable).NumberOfBlocks <= i {
        return STATUS_MESSAGE_NOT_FOUND;
    }

    let block = (*MessageTable).Blocks.as_ptr().add(i as usize);
    MessageEntry = (MessageTable as PUCHAR).add((*block).OffsetToEntries as usize)
        as PMESSAGE_RESOURCE_ENTRY;

    i = 0;
    while i < IdOffset {
        MessageEntry = (MessageEntry as PUCHAR).add((*MessageEntry).Length as usize)
            as PMESSAGE_RESOURCE_ENTRY;
        i += 1;
    }

    if !MessageResourceEntry.is_null() {
        core::ptr::write_volatile(MessageResourceEntry, MessageEntry);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlFormatMessageEx(
    Message: PWSTR,
    MaxWidth: u32,
    IgnoreInserts: u8,
    ArgumentsAreAnsi: u8,
    ArgumentsAreAnArray: u8,
    Arguments: *mut va_list,
    Buffer: PWSTR,
    BufferSize: u32,
    ReturnLength: *mut u32,
    Flags: u32,
) -> i32 {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlFormatMessage(
    Message: PWSTR,
    MaxWidth: u32,
    IgnoreInserts: u8,
    ArgumentsAreAnsi: u8,
    ArgumentsAreAnArray: u8,
    Arguments: *mut va_list,
    Buffer: PWSTR,
    BufferSize: u32,
    ReturnLength: *mut u32,
) -> i32 {
    RtlFormatMessageEx(
        Message,
        MaxWidth,
        IgnoreInserts,
        ArgumentsAreAnsi,
        ArgumentsAreAnArray,
        Arguments,
        Buffer,
        BufferSize,
        ReturnLength,
        0,
    )
}
