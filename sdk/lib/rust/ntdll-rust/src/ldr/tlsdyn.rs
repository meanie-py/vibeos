//! Implicit TLS (`__declspec(thread)`, a PE TLS directory) for DLLs loaded after process start.
//!
//! NT 5.2's loader gives TLS indices only to the modules present when the process starts
//! (`LdrpInitializeTls`); a DLL that `LoadLibrary` brings in later keeps `_tls_index == 0` and
//! every thread-local it touches lands in slot 0, the EXE's block. Vista lifted that, and
//! modern programs depend on it: Chrome's chrome.exe loads chrome.dll dynamically, and its
//! first thread-local read came back as the EXE's bytes (a 1 where a pointer belonged) and
//! faulted at address 0x19. This is Vista's behaviour: the DLL gets the next index, and
//! every live thread gets a block, copied from the template, in a vector grown to fit.
//!
//! Windows grows the other threads' vectors in the kernel (ProcessTlsInformation); here ntdll
//! walks its own threads with NtGetNextThread under the loader lock, which already serializes
//! every other writer of these vectors (thread init and exit both take it).

use super::ldrinit::{LdrpNumberOfTlsEntries, LdrpTlsList, LDRP_TLS_DATA};
use core::ffi::c_void;
use ntdll_sys::nt::*;

#[repr(C)]
struct ThreadBasicInformation {
    exit_status: NTSTATUS,
    teb_base_address: *mut c_void,
    client_id: CLIENT_ID,
    affinity_mask: usize,
    priority: i32,
    base_priority: i32,
}

extern "system" {
    fn NtGetNextThread(
        ProcessHandle: HANDLE,
        ThreadHandle: HANDLE,
        DesiredAccess: ACCESS_MASK,
        HandleAttributes: ULONG,
        Flags: ULONG,
        NewThreadHandle: *mut HANDLE,
    ) -> NTSTATUS;
    fn NtQueryInformationThread(
        ThreadHandle: HANDLE,
        ThreadInformationClass: ULONG,
        ThreadInformation: *mut c_void,
        ThreadInformationLength: ULONG,
        ReturnLength: *mut ULONG,
    ) -> NTSTATUS;
    fn NtClose(Handle: HANDLE) -> NTSTATUS;
}

#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}
/// FORCEINLINE in rtlfuncs.h, so no export to import.
#[inline(always)]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

const THREAD_BASIC_INFORMATION_CLASS: ULONG = 0;
const THREAD_QUERY_INFORMATION_ACCESS: ACCESS_MASK = 0x0040;
const STATUS_PENDING_CODE: NTSTATUS = 0x0000_0103;

/// One thread's copy of a module's TLS template: the raw data, then `SizeOfZeroFill` zero bytes.
/// (The C sized it as raw data only, so the zero-filled tail of every block was past its end.)
pub unsafe fn LdrpAllocateTlsBlock(dir: *const IMAGE_TLS_DIRECTORY) -> *mut c_void {
    let raw = ((*dir).EndAddressOfRawData - (*dir).StartAddressOfRawData) as usize;
    let total = raw + (*dir).SizeOfZeroFill as usize;
    let block = RtlAllocateHeap(RtlGetProcessHeap(), 0, total.max(1) as SIZE_T) as *mut u8;
    if block.is_null() {
        return block as *mut c_void;
    }
    core::ptr::copy_nonoverlapping((*dir).StartAddressOfRawData as *const u8, block, raw);
    core::ptr::write_bytes(block.add(raw), 0, total - raw);
    block as *mut c_void
}

/// Give `entry` (being initialized after process start) a TLS index, and every live thread a
/// block for it. Called with the loader lock held, before the DLL's TLS callbacks and DllMain.
pub unsafe fn LdrpHandleDynamicTls(entry: PLDR_DATA_TABLE_ENTRY) -> NTSTATUS {
    let mut size: ULONG = 0;
    if (*entry).TlsIndex != 0 || crate::ldr::ldrutils::LdrpIsForeignMachineImage((*entry).DllBase) {
        return STATUS_SUCCESS;
    }
    let dir = RtlImageDirectoryEntryToData((*entry).DllBase, 1, IMAGE_DIRECTORY_ENTRY_TLS as u16, &mut size)
        as *const IMAGE_TLS_DIRECTORY;
    if dir.is_null() {
        return STATUS_SUCCESS;
    }
    let data = RtlAllocateHeap(RtlGetProcessHeap(), 0, core::mem::size_of::<LDRP_TLS_DATA>() as SIZE_T)
        as *mut LDRP_TLS_DATA;
    if data.is_null() {
        return STATUS_NO_MEMORY;
    }
    let index = LdrpNumberOfTlsEntries;
    (*data).TlsDirectory = *dir;
    (*data).TlsDirectory.Characteristics = index;
    let count = index as usize + 1;

    // Every live thread: a vector one slot longer, old slots carried over, the new block filled.
    // The old vector is NOT freed: another thread may be between reading its vector pointer and
    // indexing it, and a few dozen bytes per DLL per thread is the price of not tearing that read.
    let teb_delta = own_teb_delta();
    let mut prev: HANDLE = core::ptr::null_mut();
    loop {
        let mut next: HANDLE = core::ptr::null_mut();
        if !NT_SUCCESS(NtGetNextThread(NtCurrentProcess(), prev, THREAD_QUERY_INFORMATION_ACCESS, 0, 0, &mut next)) {
            break;
        }
        if !prev.is_null() {
            NtClose(prev);
        }
        prev = next;
        let mut info: ThreadBasicInformation = core::mem::zeroed();
        if !NT_SUCCESS(NtQueryInformationThread(
            next,
            THREAD_BASIC_INFORMATION_CLASS,
            &mut info as *mut _ as *mut c_void,
            core::mem::size_of::<ThreadBasicInformation>() as ULONG,
            core::ptr::null_mut(),
        )) || info.teb_base_address.is_null()
            || info.exit_status != STATUS_PENDING_CODE
        {
            continue;
        }
        let teb = (info.teb_base_address as isize + teb_delta) as PTEB;
        let old = (*teb).ThreadLocalStoragePointer as *mut *mut c_void;
        if old.is_null() && index != 0 {
            // Not through LdrpInitializeThread yet: it allocates every listed slot itself, under
            // the lock we hold, once this entry is on the list.
            continue;
        }
        let vector = RtlAllocateHeap(RtlGetProcessHeap(), 0, (count * core::mem::size_of::<PVOID>()) as SIZE_T)
            as *mut *mut c_void;
        if vector.is_null() {
            continue;
        }
        if !old.is_null() {
            core::ptr::copy_nonoverlapping(old, vector, index as usize);
        }
        *vector.add(index as usize) = LdrpAllocateTlsBlock(dir);
        (*teb).ThreadLocalStoragePointer = vector as PVOID;
    }
    if !prev.is_null() {
        NtClose(prev);
    }

    InsertTailList(core::ptr::addr_of_mut!(LdrpTlsList), core::ptr::addr_of_mut!((*data).TlsLinks));
    *((*dir).AddressOfIndex as *mut LONG) = index as LONG;
    LdrpNumberOfTlsEntries = index + 1;
    (*entry).TlsIndex = -1i32 as u16;
    // Pinned, as the static-TLS DLLs are: the list keeps this directory, and a thread created
    // after an unload would copy its template from unmapped memory. Windows frees the slot on
    // the last FreeLibrary instead; a TLS DLL that stays loaded is the safe side of that.
    (*entry).LoadCount = 0xFFFF;
    STATUS_SUCCESS
}

/// What to add to a TEB address the kernel reports to get the one this ntdll's code uses. Zero
/// natively; in a WoW64 process the kernel reports the 64-bit TEB and the 32-bit one sits beside
/// it. Measured on the current thread rather than assumed, so it holds on every arch.
unsafe fn own_teb_delta() -> isize {
    let me = NtCurrentTeb();
    let mut prev: HANDLE = core::ptr::null_mut();
    let mut delta = 0isize;
    loop {
        let mut next: HANDLE = core::ptr::null_mut();
        if !NT_SUCCESS(NtGetNextThread(NtCurrentProcess(), prev, THREAD_QUERY_INFORMATION_ACCESS, 0, 0, &mut next)) {
            break;
        }
        if !prev.is_null() {
            NtClose(prev);
        }
        prev = next;
        let mut info: ThreadBasicInformation = core::mem::zeroed();
        if NT_SUCCESS(NtQueryInformationThread(
            next,
            THREAD_BASIC_INFORMATION_CLASS,
            &mut info as *mut _ as *mut c_void,
            core::mem::size_of::<ThreadBasicInformation>() as ULONG,
            core::ptr::null_mut(),
        )) && info.client_id.UniqueThread == (*me).ClientId.UniqueThread
        {
            delta = me as isize - info.teb_base_address as isize;
            break;
        }
    }
    if !prev.is_null() {
        NtClose(prev);
    }
    delta
}
