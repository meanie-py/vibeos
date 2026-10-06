use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut SpFileExports: SPFILE_EXPORTS = _SPFILE_EXPORTS {
    SpFileQueueOpen: None,
    SpFileQueueClose: None,
    SpFileQueueCopy: None,
    SpFileQueueDelete: None,
    SpFileQueueRename: None,
    SpFileQueueCommit: None,
};
