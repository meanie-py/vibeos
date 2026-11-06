//! Fiber-local storage in ntdll: `RtlFlsAlloc` / `RtlFlsFree` (Vista), `RtlFlsGetValue` /
//! `RtlFlsSetValue` (Windows 10), `RtlFlsAllocEx` / `RtlFlsGetValue2` (Windows 11), and the
//! `RtlProcessFlsData` placeholder. Port of upstream `sdk/lib/rtl/fls.c` (bf45318bc5b).
//!
//! Every export is `-version=0x600+` or `0xA00+` in ntdll.spec, so the shipping 0x502 build
//! carries the bodies without exporting them; kernel32's `FlsAlloc` family keeps its own
//! bodies over the same PEB/TEB state (kernel32-rust `client/fiber.rs`). The two agree on the
//! callback-array layout: `PEB.FlsCallback` is an array of `FLS_CALLBACK_INFO` pairs
//! (Windows' layout, which the apitest reads), so a process mixing the two families sees one
//! set of slots.
//!
//! Where upstream and this port differ: upstream's `RtlFlsGetValue` returns the Win32 code
//! `ERROR_INVALID_PARAMETER` (87) when the thread has no FLS data. 87 is a positive NTSTATUS,
//! i.e. success, and `*FlsData` is never written, so the caller reads an uninitialized
//! pointer. This returns `STATUS_INVALID_PARAMETER`, the status the function's other
//! rejections already use.

use core::ffi::c_void;
use ntdll_sys::nt::*;

extern "system" {
    // The rtl bitmap bodies are linked into ntdll (crate `bitmap`); RtlClearBit is not
    // exported at 0x502, so ntdll-sys's bindgen does not see it as an import.
    fn RtlClearBit(BitMapHeader: PRTL_BITMAP, BitNumber: ULONG);
}


/// One `PEB.FlsCallback` slot. The second pointer is unused by Vista's ntdll and by this port.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FLS_CALLBACK_INFO {
    pub Callback: PFLS_CALLBACK_FUNCTION,
    pub Unknown: PVOID,
}

#[inline(always)]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

/// The FLS lock. Upstream keeps an SRW lock commented out and uses the PEB lock, because
/// kernel32's FLS functions serialize on the PEB lock and the two must agree.
#[inline(always)]
unsafe fn RtlpAcquireFlsLock() {
    RtlAcquirePebLock();
}

#[inline(always)]
unsafe fn RtlpReleaseFlsLock() {
    RtlReleasePebLock();
}

#[inline(always)]
unsafe fn fls_slot(data: PRTL_FLS_DATA, index: ULONG) -> *mut PVOID {
    core::ptr::addr_of_mut!((*data).Data).cast::<PVOID>().add(index as usize)
}

#[inline(always)]
unsafe fn fls_callbacks(peb: PPEB) -> *mut FLS_CALLBACK_INFO {
    (*peb).FlsCallback as *mut FLS_CALLBACK_INFO
}

#[inline(always)]
unsafe fn fls_data_from_entry(entry: PLIST_ENTRY) -> PRTL_FLS_DATA {
    (entry as *mut u8).sub(core::mem::offset_of!(RTL_FLS_DATA, ListEntry)) as PRTL_FLS_DATA
}

unsafe fn RtlpAllocateFlsCallbackArray() -> NTSTATUS {
    let peb = NtCurrentPeb();

    let mut newCallbacks = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (RTL_FLS_MAXIMUM_AVAILABLE as usize * core::mem::size_of::<FLS_CALLBACK_INFO>()) as SIZE_T,
    );
    if newCallbacks.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlpAcquireFlsLock();
    if (*peb).FlsCallback.is_null() {
        (*peb).FlsCallback = newCallbacks as *mut _;
        newCallbacks = core::ptr::null_mut();
    }
    RtlpReleaseFlsLock();

    // Another thread installed an array first.
    if !newCallbacks.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, newCallbacks);
    }

    STATUS_SUCCESS
}

unsafe fn RtlpAllocateThreadFlsData() -> NTSTATUS {
    let teb = NtCurrentTeb();
    debug_assert!((*teb).FlsData.is_null());

    let newFlsData = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RTL_FLS_DATA>() as SIZE_T,
    ) as PRTL_FLS_DATA;
    if newFlsData.is_null() {
        return STATUS_NO_MEMORY;
    }

    let peb = NtCurrentPeb();
    RtlpAcquireFlsLock();
    InsertTailList(
        core::ptr::addr_of_mut!((*peb).FlsListHead),
        core::ptr::addr_of_mut!((*newFlsData).ListEntry),
    );
    RtlpReleaseFlsLock();

    (*teb).FlsData = newFlsData as PVOID;
    STATUS_SUCCESS
}

/// Windows 11. The second parameter's meaning is unknown; it is not written.
#[no_mangle]
pub unsafe extern "system" fn RtlFlsAllocEx(
    Callback: PFLS_CALLBACK_FUNCTION,
    _Unknown: *mut ULONG,
    FlsIndex: *mut ULONG,
) -> NTSTATUS {
    let peb = NtCurrentPeb();
    let teb = NtCurrentTeb();
    let mut status: NTSTATUS;

    if (*teb).FlsData.is_null() {
        status = RtlpAllocateThreadFlsData();
        if !NT_SUCCESS(status) {
            return status;
        }
    }

    if (*peb).FlsCallback.is_null() {
        status = RtlpAllocateFlsCallbackArray();
        if !NT_SUCCESS(status) {
            return status;
        }
    }

    RtlpAcquireFlsLock();

    // Bit 0 is set by LdrpInitializeProcess, so index 0 is never handed out.
    let index = RtlFindClearBitsAndSet((*peb).FlsBitmap as PRTL_BITMAP, 1, 0);
    if index == u32::MAX {
        RtlpReleaseFlsLock();
        return STATUS_NO_MEMORY;
    }

    (*fls_callbacks(peb).add(index as usize)).Callback = Callback;

    if index > (*peb).FlsHighIndex {
        (*peb).FlsHighIndex = index;
    }

    RtlpReleaseFlsLock();

    *FlsIndex = index;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlFlsAlloc(
    Callback: PFLS_CALLBACK_FUNCTION,
    FlsIndex: *mut ULONG,
) -> NTSTATUS {
    RtlFlsAllocEx(Callback, core::ptr::null_mut(), FlsIndex)
}

struct FreeState {
    peb: PPEB,
    index: ULONG,
    callbacks: *mut FLS_CALLBACK_INFO,
    callback: PFLS_CALLBACK_FUNCTION,
    /// The C's `_SEH2_VOLATILE listEntry`: the body advances it and the finally resumes from
    /// it, so it lives in the state both closures share rather than in either frame.
    entry: PLIST_ENTRY,
}

#[no_mangle]
pub unsafe extern "system" fn RtlFlsFree(FlsIndex: ULONG) -> NTSTATUS {
    let peb = NtCurrentPeb();
    let flsBitmap = (*peb).FlsBitmap as PRTL_BITMAP;

    if FlsIndex == 0 || FlsIndex >= FLS_MAXIMUM_AVAILABLE {
        return STATUS_INVALID_PARAMETER;
    }

    RtlpAcquireFlsLock();

    if RtlTestBit(flsBitmap, FlsIndex) == 0 {
        RtlpReleaseFlsLock();
        return STATUS_INVALID_PARAMETER;
    }

    let callbacks = fls_callbacks(peb);
    let mut st = FreeState {
        peb,
        index: FlsIndex,
        callbacks,
        callback: (*callbacks.add(FlsIndex as usize)).Callback,
        entry: core::ptr::null_mut(),
    };

    RtlClearBit(flsBitmap, FlsIndex);

    // SEH guards against a faulting callback. Vista does not, and leaks the FLS lock;
    // Windows 10 releases it and still clears the remaining values. This is the Windows 10
    // behaviour, as upstream chose.
    ntdll_sys::seh::try_finally(
        &mut st,
        |st| {
            let head = core::ptr::addr_of_mut!((*st.peb).FlsListHead);
            st.entry = (*head).Flink;
            while st.entry != head {
                let data = fls_data_from_entry(st.entry);
                let slot = fls_slot(data, st.index);
                if !(*slot).is_null() {
                    if let Some(cb) = st.callback {
                        cb(*slot);
                    }
                    *slot = core::ptr::null_mut();
                }
                st.entry = (*st.entry).Flink;
            }
            (*st.callbacks.add(st.index as usize)).Callback = None;
        },
        |st, abnormal| {
            if abnormal {
                // Finish clearing from the entry whose callback raised.
                let head = core::ptr::addr_of_mut!((*st.peb).FlsListHead);
                while st.entry != head {
                    *fls_slot(fls_data_from_entry(st.entry), st.index) = core::ptr::null_mut();
                    st.entry = (*st.entry).Flink;
                }
                (*st.callbacks.add(st.index as usize)).Callback = None;
            }
            RtlpReleaseFlsLock();
        },
    );

    STATUS_SUCCESS
}

/// Vista's signature (the one ntdll.spec exports at 0x600). Windows 10 changed it to
/// `(HANDLE ProcessHandle, PVOID* FlsData)`; neither is implemented upstream.
#[no_mangle]
pub unsafe extern "system" fn RtlProcessFlsData(_FlsData: PRTL_FLS_DATA) -> NTSTATUS {
    DbgPrint(b"WARNING:  RtlProcessFlsData is UNIMPLEMENTED!\n\0".as_ptr() as PCSTR);
    STATUS_NOT_IMPLEMENTED
}

/// Windows 10. Does not consult the bitmap, as Windows does not.
#[no_mangle]
pub unsafe extern "system" fn RtlFlsGetValue(FlsIndex: ULONG, FlsData: *mut PVOID) -> NTSTATUS {
    if FlsIndex == 0 || FlsIndex >= FLS_MAXIMUM_AVAILABLE {
        return STATUS_INVALID_PARAMETER;
    }

    let teb = NtCurrentTeb();
    if (*teb).FlsData.is_null() {
        // Upstream returns ERROR_INVALID_PARAMETER here: see the module comment.
        return STATUS_INVALID_PARAMETER;
    }

    *FlsData = *fls_slot((*teb).FlsData as PRTL_FLS_DATA, FlsIndex);
    STATUS_SUCCESS
}

/// Windows 11: `RtlFlsGetValue` with NULL for every failure.
#[no_mangle]
pub unsafe extern "system" fn RtlFlsGetValue2(FlsIndex: ULONG) -> PVOID {
    let mut value: PVOID = core::ptr::null_mut();
    if !NT_SUCCESS(RtlFlsGetValue(FlsIndex, &mut value)) {
        return core::ptr::null_mut();
    }
    value
}

/// Windows 10. Does not consult the bitmap, as Windows does not.
#[no_mangle]
pub unsafe extern "system" fn RtlFlsSetValue(FlsIndex: ULONG, FlsData: PVOID) -> NTSTATUS {
    if FlsIndex == 0 || FlsIndex >= FLS_MAXIMUM_AVAILABLE {
        return STATUS_INVALID_PARAMETER;
    }

    let teb = NtCurrentTeb();
    if (*teb).FlsData.is_null() {
        let status = RtlpAllocateThreadFlsData();
        if !NT_SUCCESS(status) {
            return status;
        }
    }

    *fls_slot((*teb).FlsData as PRTL_FLS_DATA, FlsIndex) = FlsData;
    STATUS_SUCCESS
}

const _: () = assert!(RTL_FLS_MAXIMUM_AVAILABLE == FLS_MAXIMUM_AVAILABLE);
const _: () = assert!(core::mem::size_of::<FLS_CALLBACK_INFO>() == 2 * core::mem::size_of::<*mut c_void>());
