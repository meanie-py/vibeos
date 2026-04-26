//! RTL Prefix Table Routines — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/prefix.c` (47 lines).
//! Four functions: all UNIMPLEMENTED stubs in the original C code.
//! These are placeholder symbols required by the linker but never
//! functionally used in practice.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

// ============================================================================
// FFI entry points (all stubs)
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use core::ffi::c_void;

    #[no_mangle]
    pub unsafe extern "system" fn PfxInitialize(_prefix_table: *mut c_void) {
        // UNIMPLEMENTED in original C
    }

    #[no_mangle]
    pub unsafe extern "system" fn PfxInsertPrefix(
        _prefix_table: *mut c_void,
        _prefix: *const c_void,
        _prefix_table_entry: *mut c_void,
    ) -> u8 {
        // UNIMPLEMENTED in original C, returns FALSE (0)
        0
    }

    #[no_mangle]
    pub unsafe extern "system" fn PfxRemovePrefix(
        _prefix_table: *mut c_void,
        _prefix_table_entry: *mut c_void,
    ) {
        // UNIMPLEMENTED in original C
    }

    #[no_mangle]
    pub unsafe extern "system" fn PfxFindPrefix(
        _prefix_table: *mut c_void,
        _full_name: *const c_void,
    ) -> *mut c_void {
        // UNIMPLEMENTED in original C, returns NULL
        core::ptr::null_mut()
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    // Stubs — nothing to test beyond existence.
    // The functions are UNIMPLEMENTED in the original C code.

    #[test]
    fn test_stubs_exist() {
        // Verify the module compiles. The actual functions are
        // #[cfg(not(test))] so we just verify the crate builds.
        assert!(true);
    }
}
