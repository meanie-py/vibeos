//! IO operation length/access lookup tables.
//!
//! Replaces `ntoskrnl/io/iomgr/io_tables.c`, which used to materialize these
//! tables by `#include "internal/io_i.h"`. The C version computed each entry
//! as `sizeof(FILE_FOO_INFORMATION)` against kernel headers; the Rust
//! version uses `core::mem::size_of::<_FILE_FOO_INFORMATION>()` against the
//! same bindgen-generated structs in `ntkrnl-sys`. Sizes are guaranteed
//! identical by `ntkrnl-sys`'s compile-time `size_check_*.rs` assertions.
//!
//! These statics are consumed by `iofunc.rs` via `extern "C"` declarations
//! and read past the declared single-element bound through `c_array_get` /
//! pointer arithmetic, terminated by the sentinel value (`0xFF` for UCHAR
//! length tables, `0xFFFFFFFF` for ACCESS_MASK tables). The original C
//! arrays were `static`-less (kernel-global) and the Rust translations use
//! `#[no_mangle] pub static` to expose the same C-visible symbols.
//!
//! Each consumer indexes by the `Max*InfoClass`-bounded request value;
//! reading past index `N-1` reads the sentinel and the caller bails out.

use core::mem::size_of;
use ntkrnl_sys::ddk::{
    _FILE_ACCESS_INFORMATION, _FILE_ALIGNMENT_INFORMATION, _FILE_ALLOCATION_INFORMATION,
    _FILE_ALL_INFORMATION, _FILE_ATTRIBUTE_TAG_INFORMATION, _FILE_BASIC_INFORMATION,
    _FILE_COMPLETION_INFORMATION, _FILE_COMPRESSION_INFORMATION,
    _FILE_DISPOSITION_INFORMATION, _FILE_EA_INFORMATION, _FILE_END_OF_FILE_INFORMATION,
    _FILE_FS_ATTRIBUTE_INFORMATION, _FILE_FS_CONTROL_INFORMATION,
    _FILE_FS_DEVICE_INFORMATION, _FILE_FS_DRIVER_PATH_INFORMATION,
    _FILE_FS_FULL_SIZE_INFORMATION, _FILE_FS_LABEL_INFORMATION,
    _FILE_FS_OBJECTID_INFORMATION, _FILE_FS_SIZE_INFORMATION,
    _FILE_FS_VOLUME_INFORMATION, _FILE_INTERNAL_INFORMATION, _FILE_LINK_INFORMATION,
    _FILE_MAILSLOT_QUERY_INFORMATION, _FILE_MAILSLOT_SET_INFORMATION,
    _FILE_MODE_INFORMATION, _FILE_MOVE_CLUSTER_INFORMATION, _FILE_NAME_INFORMATION,
    _FILE_NETWORK_OPEN_INFORMATION, _FILE_OBJECTID_INFORMATION,
    _FILE_PIPE_INFORMATION, _FILE_PIPE_LOCAL_INFORMATION, _FILE_PIPE_REMOTE_INFORMATION,
    _FILE_POSITION_INFORMATION, _FILE_QUOTA_INFORMATION, _FILE_RENAME_INFORMATION,
    _FILE_REPARSE_POINT_INFORMATION, _FILE_STANDARD_INFORMATION,
    _FILE_STREAM_INFORMATION, _FILE_TRACKING_INFORMATION,
    _FILE_VALID_DATA_LENGTH_INFORMATION, _UNICODE_STRING,
};
use ntkrnl_sys::constants::{
    DELETE, FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_READ_EA, FILE_WRITE_ATTRIBUTES,
    FILE_WRITE_DATA, FILE_WRITE_EA,
};

// ---- File Information Classes -----------------------------------------------

#[no_mangle]
pub static IopQueryOperationLength: [u8; 42] = [
    0,                                                  // 0
    0,                                                  // 1
    0,                                                  // 2
    0,                                                  // 3
    size_of::<_FILE_BASIC_INFORMATION>() as u8,         // 4
    size_of::<_FILE_STANDARD_INFORMATION>() as u8,      // 5
    size_of::<_FILE_INTERNAL_INFORMATION>() as u8,      // 6
    size_of::<_FILE_EA_INFORMATION>() as u8,            // 7
    size_of::<_FILE_ACCESS_INFORMATION>() as u8,        // 8
    size_of::<_FILE_NAME_INFORMATION>() as u8,          // 9
    0,                                                  // 10
    0,                                                  // 11
    0,                                                  // 12
    0,                                                  // 13
    size_of::<_FILE_POSITION_INFORMATION>() as u8,      // 14
    0,                                                  // 15
    size_of::<_FILE_MODE_INFORMATION>() as u8,          // 16
    size_of::<_FILE_ALIGNMENT_INFORMATION>() as u8,     // 17
    size_of::<_FILE_ALL_INFORMATION>() as u8,           // 18
    0,                                                  // 19
    0,                                                  // 20
    size_of::<_FILE_NAME_INFORMATION>() as u8,          // 21
    size_of::<_FILE_STREAM_INFORMATION>() as u8,        // 22
    size_of::<_FILE_PIPE_INFORMATION>() as u8,          // 23
    size_of::<_FILE_PIPE_LOCAL_INFORMATION>() as u8,    // 24
    size_of::<_FILE_PIPE_REMOTE_INFORMATION>() as u8,   // 25
    size_of::<_FILE_MAILSLOT_QUERY_INFORMATION>() as u8,// 26
    0,                                                  // 27
    size_of::<_FILE_COMPRESSION_INFORMATION>() as u8,   // 28
    size_of::<_FILE_OBJECTID_INFORMATION>() as u8,      // 29
    0,                                                  // 30
    0,                                                  // 31
    size_of::<_FILE_QUOTA_INFORMATION>() as u8,         // 32
    size_of::<_FILE_REPARSE_POINT_INFORMATION>() as u8, // 33
    size_of::<_FILE_NETWORK_OPEN_INFORMATION>() as u8,  // 34
    size_of::<_FILE_ATTRIBUTE_TAG_INFORMATION>() as u8, // 35
    0,                                                  // 36
    0,                                                  // 37
    0,                                                  // 38
    0,                                                  // 39
    0,                                                  // 40
    0xFF,                                               // 41 — sentinel
];

#[no_mangle]
pub static IopSetOperationLength: [u8; 42] = [
    0,                                                  // 0
    0,                                                  // 1
    0,                                                  // 2
    0,                                                  // 3
    size_of::<_FILE_BASIC_INFORMATION>() as u8,         // 4
    0,                                                  // 5
    0,                                                  // 6
    0,                                                  // 7
    0,                                                  // 8
    0,                                                  // 9
    size_of::<_FILE_RENAME_INFORMATION>() as u8,        // 10
    size_of::<_FILE_LINK_INFORMATION>() as u8,          // 11
    0,                                                  // 12
    size_of::<_FILE_DISPOSITION_INFORMATION>() as u8,   // 13
    size_of::<_FILE_POSITION_INFORMATION>() as u8,      // 14
    0,                                                  // 15
    size_of::<_FILE_MODE_INFORMATION>() as u8,          // 16
    0,                                                  // 17
    0,                                                  // 18
    size_of::<_FILE_ALLOCATION_INFORMATION>() as u8,    // 19
    size_of::<_FILE_END_OF_FILE_INFORMATION>() as u8,   // 20
    0,                                                  // 21
    0,                                                  // 22
    size_of::<_FILE_PIPE_INFORMATION>() as u8,          // 23
    0,                                                  // 24
    0,                                                  // 25
    0,                                                  // 26
    size_of::<_FILE_MAILSLOT_SET_INFORMATION>() as u8,  // 27
    0,                                                  // 28
    size_of::<_FILE_OBJECTID_INFORMATION>() as u8,      // 29
    size_of::<_FILE_COMPLETION_INFORMATION>() as u8,    // 30
    size_of::<_FILE_MOVE_CLUSTER_INFORMATION>() as u8,  // 31
    size_of::<_FILE_QUOTA_INFORMATION>() as u8,         // 32
    0,                                                  // 33
    0,                                                  // 34
    0,                                                  // 35
    size_of::<_FILE_TRACKING_INFORMATION>() as u8,      // 36
    0,                                                  // 37
    0,                                                  // 38
    size_of::<_FILE_VALID_DATA_LENGTH_INFORMATION>() as u8, // 39
    size_of::<_UNICODE_STRING>() as u8,                 // 40
    0xFF,                                               // 41 — sentinel
];

#[no_mangle]
pub static IopQueryOperationAccess: [u32; 41] = [
    0,                       // 0
    0,                       // 1
    0,                       // 2
    0,                       // 3
    FILE_READ_ATTRIBUTES,    // 4
    0,                       // 5
    0,                       // 6
    0,                       // 7
    0,                       // 8
    0,                       // 9
    0,                       // 10
    0,                       // 11
    0,                       // 12
    0,                       // 13
    0,                       // 14
    FILE_READ_EA,            // 15
    0,                       // 16
    0,                       // 17
    FILE_READ_ATTRIBUTES,    // 18
    0,                       // 19
    0,                       // 20
    0,                       // 21
    0,                       // 22
    FILE_READ_ATTRIBUTES,    // 23
    FILE_READ_ATTRIBUTES,    // 24
    FILE_READ_ATTRIBUTES,    // 25
    0,                       // 26
    0,                       // 27
    0,                       // 28
    0,                       // 29
    0,                       // 30
    0,                       // 31
    0,                       // 32
    0,                       // 33
    FILE_READ_ATTRIBUTES,    // 34
    FILE_READ_ATTRIBUTES,    // 35
    0,                       // 36
    0,                       // 37
    0,                       // 38
    0,                       // 39
    0xFFFFFFFF,              // 40 — sentinel
];

#[no_mangle]
pub static IopSetOperationAccess: [u32; 42] = [
    0,                       // 0
    0,                       // 1
    0,                       // 2
    0,                       // 3
    FILE_WRITE_ATTRIBUTES,   // 4
    0,                       // 5
    0,                       // 6
    0,                       // 7
    0,                       // 8
    0,                       // 9
    DELETE,                  // 10
    0,                       // 11
    0,                       // 12
    DELETE,                  // 13
    0,                       // 14
    FILE_WRITE_EA,           // 15
    0,                       // 16
    0,                       // 17
    0,                       // 18
    FILE_WRITE_DATA,         // 19
    FILE_WRITE_DATA,         // 20
    0,                       // 21
    0,                       // 22
    FILE_WRITE_ATTRIBUTES,   // 23
    0,                       // 24
    FILE_WRITE_ATTRIBUTES,   // 25
    0,                       // 26
    0,                       // 27
    0,                       // 28
    0,                       // 29
    0,                       // 30
    FILE_WRITE_DATA,         // 31
    0,                       // 32
    0,                       // 33
    0,                       // 34
    0,                       // 35
    FILE_WRITE_DATA,         // 36
    0,                       // 37
    0,                       // 38
    FILE_WRITE_DATA,         // 39
    DELETE,                  // 40
    0xFFFFFFFF,              // 41 — sentinel
];

// ---- Volume (FS) Information Classes ----------------------------------------

#[no_mangle]
pub static IopQueryFsOperationLength: [u8; 11] = [
    0,                                                      // 0
    size_of::<_FILE_FS_VOLUME_INFORMATION>() as u8,         // 1
    0,                                                      // 2
    size_of::<_FILE_FS_SIZE_INFORMATION>() as u8,           // 3
    size_of::<_FILE_FS_DEVICE_INFORMATION>() as u8,         // 4
    size_of::<_FILE_FS_ATTRIBUTE_INFORMATION>() as u8,      // 5
    size_of::<_FILE_FS_CONTROL_INFORMATION>() as u8,        // 6
    size_of::<_FILE_FS_FULL_SIZE_INFORMATION>() as u8,      // 7
    size_of::<_FILE_FS_OBJECTID_INFORMATION>() as u8,       // 8
    size_of::<_FILE_FS_DRIVER_PATH_INFORMATION>() as u8,    // 9
    0xFF,                                                   // 10 — sentinel
];

#[no_mangle]
pub static IopSetFsOperationLength: [u8; 11] = [
    0,                                              // 0
    0,                                              // 1
    size_of::<_FILE_FS_LABEL_INFORMATION>() as u8,  // 2
    0,                                              // 3
    0,                                              // 4
    0,                                              // 5
    size_of::<_FILE_FS_CONTROL_INFORMATION>() as u8,// 6
    0,                                              // 7
    size_of::<_FILE_FS_OBJECTID_INFORMATION>() as u8,// 8
    0,                                              // 9
    0xFF,                                           // 10 — sentinel
];

#[no_mangle]
pub static IopQueryFsOperationAccess: [u32; 11] = [
    0,                  // 0
    0,                  // 1
    0,                  // 2
    0,                  // 3
    0,                  // 4
    0,                  // 5
    FILE_READ_DATA,     // 6
    0,                  // 7
    0,                  // 8
    0,                  // 9
    0xFFFFFFFF,         // 10 — sentinel
];

#[no_mangle]
pub static IopSetFsOperationAccess: [u32; 11] = [
    0,                  // 0
    0,                  // 1
    FILE_WRITE_DATA,    // 2
    0,                  // 3
    0,                  // 4
    0,                  // 5
    FILE_WRITE_DATA,    // 6
    0,                  // 7
    FILE_WRITE_DATA,    // 8
    0,                  // 9
    0xFFFFFFFF,         // 10 — sentinel
];
