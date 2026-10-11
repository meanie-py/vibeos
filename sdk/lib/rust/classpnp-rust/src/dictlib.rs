use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{FILE_OBJECT_EXTENSION, KeInitializeSpinLock, ExAllocatePoolWithTag};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{DICTIONARY, PDICTIONARY};
use ntkrnl_sys::ddk::{POOL_TYPE};
use ntkrnl_sys::mm::{NonPagedPool, NonPagedPoolCacheAligned};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{ExFreePool};

extern "C" {
    fn DbgPrint(format: *const u8, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
pub const MdlMappingNoExecute: u32 = 0;
// classp.h: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
pub const NonPagedPoolNx: POOL_TYPE = 0;
// classp.h: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
pub const NonPagedPoolNxCacheAligned: POOL_TYPE = 4;
// classp.h / dictlib.c `#define POOL_NX_ALLOCATION 0` under `#ifdef __REACTOS__`, like the
// pool types above: 0x200 is the Windows value, which no TU here compiles.
pub const POOL_NX_ALLOCATION: u32 = 0;
// C: dictlib.c `#define DICTIONARY_SIGNATURE 'tciD'` = 0x74636944 (big-endian).
// 0x44696374 is those bytes REVERSED, and only InitializeDictionary used it --
// TestDictionarySignature compared against a function-local shadow holding the
// correct value, so every signature test on a dictionary we initialised failed.
pub const DICTIONARY_SIGNATURE: u32 = u32::from_be_bytes(*b"tciD");

#[repr(C)]
pub struct DICTIONARY_HEADER {
    pub Next: *mut DICTIONARY_HEADER,
    pub Key: u64,
    pub Data: [u8; 0],
}
pub type PDICTIONARY_HEADER = *mut DICTIONARY_HEADER;

#[no_mangle]
pub unsafe extern "C" fn InitializeDictionary(Dictionary: PDICTIONARY) {
    core::ptr::write_bytes(Dictionary as *mut u8, 0, core::mem::size_of::<DICTIONARY>());
    (*Dictionary).Signature = (DICTIONARY_SIGNATURE) as u64;
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Dictionary).SpinLock));
}
#[no_mangle]
pub unsafe extern "C" fn TestDictionarySignature(Dictionary: PDICTIONARY) -> BOOLEAN {
    ((*Dictionary).Signature == DICTIONARY_SIGNATURE as u64) as BOOLEAN
}
#[no_mangle]
pub unsafe extern "C" fn GetDictionaryEntry(
    Dictionary: PDICTIONARY,
    Key: ULONGLONG,
) -> PVOID {
    let mut entry: PDICTIONARY_HEADER;
    let mut data: PVOID;
    let mut oldIrql: KIRQL = 0;

    data = core::ptr::null_mut::<core::ffi::c_void>();
    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Dictionary).SpinLock)).cast(),
        core::ptr::addr_of_mut!(oldIrql),
    );
    entry = ((*Dictionary).List).cast();
    while !entry.is_null() {
        if (*entry).Key == Key {
            data = core::ptr::addr_of_mut!((*entry).Data).cast::<core::ffi::c_void>();
            break;
        } else {
            entry = (*entry).Next;
        }
    }
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Dictionary).SpinLock)).cast(),
        oldIrql,
    );
    data
}
// AllocateDictionaryEntry — hand-translated (uses ReactOS PSEH TRY/LEAVE/FINALLY, which the
// tree-sitter cataloger truncates on). Under DUMMY_PSEH the TRY block is linear; LEAVE → break,
// FINALLY → fallthrough cleanup. Mirrors dictlib.c:77.
#[no_mangle]
pub unsafe extern "C" fn AllocateDictionaryEntry(
    Dictionary: PDICTIONARY,
    Key: ULONGLONG,
    Size: ULONG,
    Tag: ULONG,
    Entry: *mut PVOID,
) -> NTSTATUS {
    let mut header: PDICTIONARY_HEADER;
    let mut oldIrql: KIRQL = 0;
    let mut entry: *mut PDICTIONARY_HEADER;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    *Entry = core::ptr::null_mut();

    header = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        (Size as usize + core::mem::size_of::<DICTIONARY_HEADER>()) as _,
        Tag,
    ) as PDICTIONARY_HEADER;
    if header.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(
        header as *mut u8,
        0,
        core::mem::size_of::<DICTIONARY_HEADER>() + Size as usize,
    );
    (*header).Key = Key;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*Dictionary).SpinLock).cast(),
        core::ptr::addr_of_mut!(oldIrql),
    );

    // TRY { ... LEAVE ... }
    'try_block: {
        entry = core::ptr::addr_of_mut!((*Dictionary).List) as *mut PDICTIONARY_HEADER;
        while !(*entry).is_null() {
            if (**entry).Key == Key {
                // Dictionary must have unique keys.
                status = STATUS_OBJECT_NAME_COLLISION;
                break 'try_block;
            } else if (**entry).Key < Key {
                break;
            } else {
                entry = core::ptr::addr_of_mut!((**entry).Next) as *mut PDICTIONARY_HEADER;
            }
        }
        (*header).Next = *entry;
        *entry = header;
    }
    // FINALLY
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*Dictionary).SpinLock).cast(),
        oldIrql,
    );
    if status < 0 {
        FREE_POOL(header as PVOID);
    } else {
        *Entry = core::ptr::addr_of_mut!((*header).Data) as PVOID;
    }
    status
}

#[no_mangle]
pub unsafe extern "C" fn FreeDictionaryEntry(Dictionary: *mut DICTIONARY, Entry: *mut core::ffi::c_void) {
    let mut header: *mut DICTIONARY_HEADER = core::ptr::null_mut();
    let mut entry: *mut *mut DICTIONARY_HEADER = core::ptr::null_mut();
    let mut oldIrql: KIRQL = 0;
    let mut found: u8 = 0;

    found = 0u8;
    // CONTAINING_RECORD macro expansion
    header = (Entry as usize)
        .wrapping_sub(core::mem::offset_of!(DICTIONARY_HEADER, Data))
        as *mut DICTIONARY_HEADER;

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*Dictionary).SpinLock)).cast(), core::ptr::addr_of_mut!(oldIrql));

    entry = core::ptr::addr_of_mut!((*Dictionary).List) as *mut *mut DICTIONARY_HEADER;
    while !(*entry).is_null() {
        if *entry == header {
            *entry = (*header).Next;
            found = 1u8;
            break;
        } else {
            entry = core::ptr::addr_of_mut!((**entry).Next) as *mut *mut DICTIONARY_HEADER;
        }
    }

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*Dictionary).SpinLock)).cast(), oldIrql);

    // NT_ASSERT(found)
    if found == 0 {
        {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
                b"dictlib.c\0".as_ptr() as *const core::ffi::c_char,
                0 /* placeholder line number */, // The original C line number is unknown here, but we can just use 0 or some constant
                b"found\0".as_ptr() as *const core::ffi::c_char,
            );
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] core::arch::asm!("int 0x2c");
        #[cfg(target_arch = "aarch64")] core::arch::asm!("brk #0");
    }

    if found != 0 {
        if !header.is_null() {
            ExFreePool(header as *mut core::ffi::c_void);
        }
    }
}
