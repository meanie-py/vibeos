use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble ----
// WIN32_NO_STATUS: valueless feature flag, nothing to emit.
// SHIM_NS DisableThemes: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(DisableThemes);

extern "system" {
    // uxtheme.h (imported: add_importlibs(acgenral uxtheme ...))
    fn SetThemeAppProperties(dwFlags: DWORD);
}

// SHIM_NUM_HOOKS 0; SHIM_NOTIFY_FN = SHIM_OBJ_NAME(Notify)
pub unsafe extern "system" fn Notify(fdwReason: DWORD, ptr: PVOID) -> BOOL {
    if fdwReason == SHIM_REASON_INIT {
        /* Disable themes for non-client, comctl controls and webcontent */
        SetThemeAppProperties(0);
    }
    return TRUE as BOOL;
}

implement_shim!(DisableThemes, 0usize, Some(Notify), []);
