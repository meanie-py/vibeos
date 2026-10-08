use crate::shim::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{PIMAGE_NT_HEADERS, PLDR_DATA_TABLE_ENTRY, PRTL_TRACE_BLOCK, PRTL_TRACE_DATABASE, PRTL_TRACE_ENUMERATE, PRTL_UNLOAD_EVENT_TRACE, RTL_TRACE_HASH_FUNCTION, RTL_UNLOAD_EVENT_TRACE, RtlImageNtHeader};

// ---- file-scope preamble (translated) ----
pub static mut RtlpUnloadEventTrace: [RTL_UNLOAD_EVENT_TRACE; RTL_UNLOAD_EVENT_TRACE_NUMBER as usize] =
    [unsafe { core::mem::zeroed() }; RTL_UNLOAD_EVENT_TRACE_NUMBER as usize];
pub static mut RtlpUnloadEventTraceIndex: UINT = 0;

#[no_mangle]
pub unsafe extern "system" fn RtlGetUnloadEventTrace() -> PRTL_UNLOAD_EVENT_TRACE {
    (&raw mut RtlpUnloadEventTrace).cast::<RTL_UNLOAD_EVENT_TRACE>()
}
#[no_mangle]
pub unsafe extern "system" fn LdrpRecordUnloadEvent(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    let NtHeaders: PIMAGE_NT_HEADERS;
    let Sequence = RtlpUnloadEventTraceIndex;
    RtlpUnloadEventTraceIndex = RtlpUnloadEventTraceIndex.wrapping_add(1);
    let Index = (Sequence as usize) % (*&raw const RtlpUnloadEventTrace).len();
    let StringLen: u16;

    RtlpUnloadEventTrace[Index].BaseAddress = (*LdrEntry).DllBase;
    RtlpUnloadEventTrace[Index].SizeOfImage = (*LdrEntry).SizeOfImage as _;
    RtlpUnloadEventTrace[Index].Sequence = Sequence;

    NtHeaders = RtlImageNtHeader((*LdrEntry).DllBase);

    if !NtHeaders.is_null() {
        RtlpUnloadEventTrace[Index].TimeDateStamp = (*NtHeaders).FileHeader.TimeDateStamp;
        RtlpUnloadEventTrace[Index].CheckSum = (*NtHeaders).OptionalHeader.CheckSum;
    } else {
        RtlpUnloadEventTrace[Index].TimeDateStamp = 0;
        RtlpUnloadEventTrace[Index].CheckSum = 0;
    }

    let nameCount = ((*LdrEntry).BaseDllName.Length as usize) / core::mem::size_of::<u16>();
    let imageNameLen = RtlpUnloadEventTrace[Index].ImageName.len();
    StringLen = core::cmp::min(nameCount, imageNameLen) as u16;

    RtlCopyMemory(
        RtlpUnloadEventTrace[Index].ImageName.as_mut_ptr() as _,
        (*LdrEntry).BaseDllName.Buffer as _,
        (StringLen as usize) * core::mem::size_of::<u16>(),
    );

    if (StringLen as usize) < imageNameLen {
        RtlpUnloadEventTrace[Index].ImageName[StringLen as usize] = 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlTraceDatabaseAdd(
    Database: PRTL_TRACE_DATABASE,
    Count: ULONG,
    Trace: *mut PVOID,
    TraceBlock: *mut PRTL_TRACE_BLOCK,
) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlTraceDatabaseCreate(
    Buckets: ULONG,
    MaximumSize: SIZE_T,
    Flags: ULONG,
    Tag: ULONG,
    HashFunction: RTL_TRACE_HASH_FUNCTION,
) -> PRTL_TRACE_DATABASE {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlTraceDatabaseDestroy(Database: PRTL_TRACE_DATABASE) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlTraceDatabaseEnumerate(
    Database: PRTL_TRACE_DATABASE,
    TraceEnumerate: PRTL_TRACE_ENUMERATE,
    TraceBlock: *mut PRTL_TRACE_BLOCK,
) -> u8 {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlTraceDatabaseFind(
    Database: PRTL_TRACE_DATABASE,
    Count: u32,
    Trace: *mut *mut core::ffi::c_void,
    TraceBlock: *mut PRTL_TRACE_BLOCK,
) -> u8 {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlTraceDatabaseLock(Database: PRTL_TRACE_DATABASE) -> u8 {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlTraceDatabaseUnlock(_Database: PRTL_TRACE_DATABASE) -> u8 {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlTraceDatabaseValidate(Database: PRTL_TRACE_DATABASE) -> BOOLEAN {
    0u8
}
