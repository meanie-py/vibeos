#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

// fs_rec.sys — Rust translation of the filesystem recognizer driver
// (drivers/filesystems/fs_rec). staticlib -> libfs_rec_rust.a, cohab-linked into the
// cmake fs_rec.sys with the translated .c commented out. BOOT-loaded recognizer
// (Group "Boot File System") → smoke-exercised. No GUID data symbols.

pub mod types;
pub mod glue;
pub mod inlines;
pub mod ddk_shims;

pub mod blockdev;
pub mod fs_rec;
pub mod fat;
pub mod fatx;
pub mod cdfs;
pub mod ntfs;
pub mod udfs;
pub mod ext;
pub mod btrfs;
pub mod reiserfs;
pub mod ffs;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
