use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, Reason: DWORD, Reserved: LPVOID) -> BOOL {
    match Reason {
        r if r == DLL_PROCESS_ATTACH => {
            /* Initialize Context list */
            InitializeListHead(core::ptr::addr_of_mut!(ContextListHead));
            /* no break */
        }
        r if r == DLL_THREAD_ATTACH => {
        }
        r if r == DLL_THREAD_DETACH => {
            /* Set NULL context for this thread */
            wglMakeCurrent(core::ptr::null_mut(), core::ptr::null_mut());
        }
        r if r == DLL_PROCESS_DETACH => {
            /* Clean up */
            if Reserved.is_null() {
                /* The process is not shutting down: release everything */
                wglMakeCurrent(core::ptr::null_mut(), core::ptr::null_mut());
                IntDeleteAllContexts();
                IntDeleteAllICDs();
            }
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
