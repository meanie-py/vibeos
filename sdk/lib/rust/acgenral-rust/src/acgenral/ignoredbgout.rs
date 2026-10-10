use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble ----
// WIN32_NO_STATUS: valueless feature flag, nothing to emit.
// SHIM_NS IgnoreDebugOutput: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(IgnoreDebugOutput);

pub unsafe extern "system" fn OutputDebugStringA(lpOutputString: LPCSTR) {
    let _ = lpOutputString; // (VOID)lpOutputString;
}

pub unsafe extern "system" fn OutputDebugStringW(lpOutputString: LPCWSTR) {
    let _ = lpOutputString; // (VOID)lpOutputString;
}

// SHIM_NUM_HOOKS 2, no SHIM_NOTIFY_FN
implement_shim!(IgnoreDebugOutput, 2usize, None, [
    (0, "KERNEL32.DLL", "OutputDebugStringA", OutputDebugStringA),
    (1, "KERNEL32.DLL", "OutputDebugStringW", OutputDebugStringW),
]);
