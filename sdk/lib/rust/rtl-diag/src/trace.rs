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
static mut RtlpUnloadEventTraceElementSize: ULONG = core::mem::size_of::<RTL_UNLOAD_EVENT_TRACE>() as ULONG;
static mut RtlpUnloadEventTraceElementCount: ULONG = RTL_UNLOAD_EVENT_TRACE_NUMBER as ULONG;
static mut RtlpUnloadEventTracePointer: PRTL_UNLOAD_EVENT_TRACE = core::ptr::null_mut();

/// Vista's form of RtlGetUnloadEventTrace: where the element size, the element count and the
/// trace live, so a debugger or crash reporter can read them out of another process. Note the
/// third output is a pointer to a VARIABLE holding the array's address (one more indirection
/// than the first two), as on Windows and in Wine; crashpad reads it that way. Chrome calls it
/// through a pointer it never checks, so without the export every Chrome process crashed on a
/// call to address 0 as its crash reporter started recording unloaded modules.
#[no_mangle]
pub unsafe extern "system" fn RtlGetUnloadEventTraceEx(
    ElementSize: *mut *mut ULONG,
    ElementCount: *mut *mut ULONG,
    EventTrace: *mut *mut PVOID,
) {
    RtlpUnloadEventTracePointer = RtlGetUnloadEventTrace();
    *ElementSize = &raw mut RtlpUnloadEventTraceElementSize;
    *ElementCount = &raw mut RtlpUnloadEventTraceElementCount;
    *EventTrace = (&raw mut RtlpUnloadEventTracePointer).cast::<PVOID>();
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
