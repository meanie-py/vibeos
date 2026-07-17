use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::rtl::RtlCopyMemory;

// SafeProbeForRead/Write: NTAPI (extern "system"), Length = SIZE_T (canonical — icm.rs).
extern "system" {
    fn SafeProbeForRead(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForWrite(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
}

// Callers reach these through the MmCopyFromCaller/MmCopyToCaller macros (= the
// `_`-prefixed names, cdecl). PAGED_CODE()/ASSERT(ExGetPreviousMode()==UserMode)
// are retail no-ops, dropped. extern "C" + #[no_mangle].
#[no_mangle]
pub unsafe extern "C" fn _MmCopyFromCaller(
    Target: PVOID,
    Source: PVOID,
    Bytes: UINT,
) -> NTSTATUS {
    let Status = SafeProbeForRead(Source as LPVOID, Bytes as SIZE_T, 1);
    if NT_SUCCESS(Status) {
        RtlCopyMemory(Target, Source, Bytes as usize);
    }
    Status
}

#[no_mangle]
pub unsafe extern "C" fn _MmCopyToCaller(
    Target: PVOID,
    Source: PVOID,
    Bytes: UINT,
) -> NTSTATUS {
    let Status = SafeProbeForWrite(Target as LPVOID, Bytes as SIZE_T, 1);
    if NT_SUCCESS(Status) {
        RtlCopyMemory(Target, Source, Bytes as usize);
    }
    Status
}
