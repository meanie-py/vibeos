//! RTL Handle Table Routines — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/handle.c` (233 lines).
//! Six functions for managing a simple handle table backed by virtual memory.
//! All architectures.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

// ============================================================================
// Constants
// ============================================================================

const RTL_HANDLE_VALID: u32 = 0x1;

// ============================================================================
// Types
// ============================================================================

/// RTL_HANDLE_TABLE_ENTRY — union of Flags (u32) and NextFree (pointer).
/// The entry is pointer-sized. Flags occupies the low 32 bits.
#[repr(C)]
pub struct RtlHandleTableEntry {
    /// When on free list: pointer to next free entry.
    /// When allocated: low 32 bits contain Flags (bit 0 = RTL_HANDLE_VALID).
    pub next_free: *mut RtlHandleTableEntry,
}

impl RtlHandleTableEntry {
    #[inline]
    fn flags(&self) -> u32 {
        // Read low 32 bits of the pointer-sized field
        self.next_free as usize as u32
    }
}

/// RTL_HANDLE_TABLE — handle table header.
#[repr(C)]
pub struct RtlHandleTable {
    pub maximum_number_of_handles: u32,
    pub size_of_handle_table_entry: u32,
    pub reserved: [u32; 2],
    pub free_handles: *mut RtlHandleTableEntry,
    pub committed_handles: *mut RtlHandleTableEntry,
    pub uncommitted_handles: *mut RtlHandleTableEntry,
    pub max_reserved_handles: *mut RtlHandleTableEntry,
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;
    use core::ffi::c_void;

    const MEM_RESERVE: u32 = 0x2000;
    const MEM_COMMIT: u32 = 0x1000;
    const MEM_RELEASE: u32 = 0x8000;
    const PAGE_READWRITE: u32 = 0x04;
    const PAGE_SIZE: usize = 0x1000;

    #[inline]
    fn nt_current_process() -> isize { -1isize }

    #[inline]
    fn nt_success(status: i32) -> bool { status >= 0 }

    extern "system" {
        fn ZwAllocateVirtualMemory(
            process_handle: isize,
            base_address: *mut *mut c_void,
            zero_bits: usize,
            region_size: *mut usize,
            allocation_type: u32,
            protect: u32,
        ) -> i32;

        fn NtFreeVirtualMemory(
            process_handle: isize,
            base_address: *mut *mut c_void,
            region_size: *mut usize,
            free_type: u32,
        ) -> i32;
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlInitializeHandleTable(
        table_size: u32,
        handle_size: u32,
        handle_table: *mut RtlHandleTable,
    ) {
        core::ptr::write_bytes(handle_table as *mut u8, 0, core::mem::size_of::<RtlHandleTable>());
        (*handle_table).maximum_number_of_handles = table_size;
        (*handle_table).size_of_handle_table_entry = handle_size;
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlDestroyHandleTable(
        handle_table: *mut RtlHandleTable,
    ) {
        if !(*handle_table).committed_handles.is_null() {
            let mut array_pointer = (*handle_table).committed_handles as *mut c_void;
            let mut array_size: usize = 0;
            NtFreeVirtualMemory(
                nt_current_process(),
                &mut array_pointer,
                &mut array_size,
                MEM_RELEASE,
            );
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlAllocateHandle(
        handle_table: *mut RtlHandleTable,
        index: *mut u32,
    ) -> *mut RtlHandleTableEntry {
        let ht = &mut *handle_table;

        // Check if we are out of free handle entries
        if ht.free_handles.is_null() {
            // Check if we don't have uncommitted handle entries yet
            if ht.uncommitted_handles.is_null() {
                let mut array_size = ht.size_of_handle_table_entry as usize
                    * ht.maximum_number_of_handles as usize;
                let mut array_pointer: *mut c_void = core::ptr::null_mut();

                // Reserve memory
                let status = ZwAllocateVirtualMemory(
                    nt_current_process(),
                    &mut array_pointer,
                    0,
                    &mut array_size,
                    MEM_RESERVE,
                    PAGE_READWRITE,
                );
                if !nt_success(status) {
                    return core::ptr::null_mut();
                }

                ht.committed_handles = array_pointer as *mut RtlHandleTableEntry;
                ht.uncommitted_handles = array_pointer as *mut RtlHandleTableEntry;
                ht.max_reserved_handles =
                    (array_pointer as usize + array_size) as *mut RtlHandleTableEntry;
            }

            // Commit one page of handle entries
            let mut array_size: usize = PAGE_SIZE;
            let mut array_pointer = ht.uncommitted_handles as *mut c_void;
            let status = ZwAllocateVirtualMemory(
                nt_current_process(),
                &mut array_pointer,
                0,
                &mut array_size,
                MEM_COMMIT,
                PAGE_READWRITE,
            );
            if !nt_success(status) {
                return core::ptr::null_mut();
            }

            ht.free_handles = array_pointer as *mut RtlHandleTableEntry;
            ht.uncommitted_handles =
                (array_pointer as usize + array_size) as *mut RtlHandleTableEntry;

            let number_of_entries = array_size / ht.size_of_handle_table_entry as usize;

            // Build free list: link each entry to the next
            let mut current = ht.free_handles;
            for _ in 0..number_of_entries - 1 {
                let next = (current as usize + ht.size_of_handle_table_entry as usize)
                    as *mut RtlHandleTableEntry;
                (*current).next_free = next;
                current = next;
            }
            // Terminate free list
            (*current).next_free = core::ptr::null_mut();
        }

        // Remove handle from free list
        let handle_entry = ht.free_handles;
        ht.free_handles = (*handle_entry).next_free;
        (*handle_entry).next_free = core::ptr::null_mut();

        if !index.is_null() {
            *index = (handle_entry as usize - ht.committed_handles as usize) as u32
                / ht.size_of_handle_table_entry;
        }

        handle_entry
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlFreeHandle(
        handle_table: *mut RtlHandleTable,
        handle: *mut RtlHandleTableEntry,
    ) -> u8 {
        let ht = &mut *handle_table;

        // Clear the entry
        core::ptr::write_bytes(
            handle as *mut u8,
            0,
            ht.size_of_handle_table_entry as usize,
        );

        // Add to free list
        (*handle).next_free = ht.free_handles;
        ht.free_handles = handle;

        1 // TRUE
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlIsValidHandle(
        handle_table: *mut RtlHandleTable,
        handle: *mut RtlHandleTableEntry,
    ) -> u8 {
        if handle_table.is_null() {
            return 0;
        }
        let ht = &*handle_table;
        if handle >= ht.committed_handles
            && handle < ht.max_reserved_handles
            && ((*handle).flags() & RTL_HANDLE_VALID) != 0
        {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlIsValidIndexHandle(
        handle_table: *mut RtlHandleTable,
        index: u32,
        handle: *mut *mut RtlHandleTableEntry,
    ) -> u8 {
        if handle_table.is_null() {
            return 0;
        }
        let ht = &*handle_table;

        let internal_handle = (ht.committed_handles as usize
            + ht.size_of_handle_table_entry as usize * index as usize)
            as *mut RtlHandleTableEntry;

        if RtlIsValidHandle(handle_table, internal_handle) == 0 {
            return 0;
        }

        if !handle.is_null() {
            *handle = internal_handle;
        }

        1
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_flags_read() {
        let mut entry = RtlHandleTableEntry {
            next_free: core::ptr::null_mut(),
        };
        assert_eq!(entry.flags(), 0);

        // Set flag bit by casting
        entry.next_free = RTL_HANDLE_VALID as usize as *mut RtlHandleTableEntry;
        assert_eq!(entry.flags() & RTL_HANDLE_VALID, RTL_HANDLE_VALID);
    }

    #[test]
    fn test_struct_sizes() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(core::mem::size_of::<RtlHandleTableEntry>(), 8);
            assert_eq!(core::mem::size_of::<RtlHandleTable>(), 48);
        }
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(core::mem::size_of::<RtlHandleTableEntry>(), 4);
            assert_eq!(core::mem::size_of::<RtlHandleTable>(), 32);
        }
    }
}
