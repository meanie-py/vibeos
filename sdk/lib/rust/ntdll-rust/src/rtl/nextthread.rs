//! `NtGetNextThread` (Vista): open the thread of `ProcessHandle` that follows
//! `ThreadHandle` (or the first, when it is NULL) in the process's thread list.
//!
//! Windows walks the kernel's thread list; our kernel has no such service, so this
//! walks the same list as the system process snapshot reports it (threads in the
//! order the process's list holds them) and opens the next one by client id. A
//! thread that exits between the snapshot and the open is skipped, as one that exits
//! during the kernel's walk would be. Reverse order (Flags 1, which Windows defines
//! only for NtGetNextProcess) is not supported.

#![allow(non_snake_case)]

use core::ptr::{null_mut, write_volatile};
use ntdll_sys::nt::*;

const STATUS_INFO_LENGTH_MISMATCH_: NTSTATUS = 0xC000_0004_u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER_: NTSTATUS = 0xC000_000D_u32 as NTSTATUS;
const STATUS_NO_MEMORY_: NTSTATUS = 0xC000_0017_u32 as NTSTATUS;
const STATUS_NO_MORE_ENTRIES_: NTSTATUS = 0x8000_001A_u32 as NTSTATUS;
const STATUS_INVALID_CID_: NTSTATUS = 0xC000_000B_u32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn NtGetNextThread(
    ProcessHandle: HANDLE,
    ThreadHandle: HANDLE,
    DesiredAccess: ACCESS_MASK,
    HandleAttributes: ULONG,
    Flags: ULONG,
    NewThreadHandle: PHANDLE,
) -> NTSTATUS {
    if NewThreadHandle.is_null() || Flags != 0 {
        return STATUS_INVALID_PARAMETER_;
    }
    write_volatile(NewThreadHandle, null_mut());

    let mut pbi: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let status = NtQueryInformationProcess(
        ProcessHandle,
        ProcessBasicInformation,
        &mut pbi as *mut _ as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        null_mut(),
    );
    if status < 0 {
        return status;
    }
    let pid = pbi.UniqueProcessId as HANDLE;

    let mut after: HANDLE = null_mut();
    if !ThreadHandle.is_null() {
        let mut tbi: THREAD_BASIC_INFORMATION = core::mem::zeroed();
        let status = NtQueryInformationThread(
            ThreadHandle,
            ThreadBasicInformation,
            &mut tbi as *mut _ as PVOID,
            core::mem::size_of::<THREAD_BASIC_INFORMATION>() as ULONG,
            null_mut(),
        );
        if status < 0 {
            return status;
        }
        after = tbi.ClientId.UniqueThread;
    }

    // The snapshot, grown until it fits.
    let heap = (*NtCurrentPeb()).ProcessHeap;
    let mut size: ULONG = 0x10000;
    let mut buf: PVOID;
    loop {
        buf = RtlAllocateHeap(heap, 0, size as SIZE_T);
        if buf.is_null() {
            return STATUS_NO_MEMORY_;
        }
        let status = NtQuerySystemInformation(SystemProcessInformation, buf, size, null_mut());
        if status == STATUS_INFO_LENGTH_MISMATCH_ {
            RtlFreeHeap(heap, 0, buf);
            size *= 2;
            continue;
        }
        if status < 0 {
            RtlFreeHeap(heap, 0, buf);
            return status;
        }
        break;
    }

    let mut result = STATUS_NO_MORE_ENTRIES_;
    let mut proc = buf as *const u8;
    loop {
        let info = proc as *const SYSTEM_PROCESS_INFORMATION;
        if (*info).UniqueProcessId == pid {
            let threads = proc.add(core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>())
                as *const SYSTEM_THREAD_INFORMATION;
            let n = (*info).NumberOfThreads as usize;
            let mut i = 0;
            if !after.is_null() {
                while i < n && (*threads.add(i)).ClientId.UniqueThread != after {
                    i += 1;
                }
                i += 1; // the one after it (none if `after` has left the list)
            }
            while i < n {
                let mut cid = (*threads.add(i)).ClientId;
                let mut attr: OBJECT_ATTRIBUTES = core::mem::zeroed();
                attr.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
                attr.Attributes = HandleAttributes;
                let status = NtOpenThread(NewThreadHandle, DesiredAccess, &mut attr, &mut cid);
                if status != STATUS_INVALID_CID_ {
                    result = status;
                    break;
                }
                i += 1;
            }
            break;
        }
        if (*info).NextEntryOffset == 0 {
            break;
        }
        proc = proc.add((*info).NextEntryOffset as usize);
    }
    RtlFreeHeap(heap, 0, buf);
    result
}
