use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut OldProcessListWndProc: WNDPROC = None;

// Prototype `ProcessListWndProc` intentionally omitted: it is emitted with its own function definition.

#[no_mangle]
pub unsafe extern "system" fn ProcessListWndProc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut hbrBackground: HBRUSH = core::ptr::null_mut();
    let mut rcItem: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcClip: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut hDC: HDC = core::ptr::null_mut();
    let mut DcSave: i32 = 0;

    match message {
        WM_ERASEBKGND => {
            /*
             * Get the device context and save it's state
             * to be restored after we're done
             */
            hDC = wParam as HDC;
            DcSave = SaveDC(hDC);

            /*
             * Get the background brush
             */
            hbrBackground = GetClassLongPtrW(hWnd, GCL_HBRBACKGROUND) as LONG_PTR as HBRUSH;

            /*
             * Calculate the clip rect by getting the RECT
             * of the first and last items and adding them up.
             */
            rcClip.left = LVIR_BOUNDS as LONG;
            SendMessageW(
                hWnd,
                LVM_GETITEMRECT as UINT,
                0 as WPARAM,
                &mut rcClip as *mut RECT as LPARAM,
            );
            rcItem.left = LVIR_BOUNDS as LONG;
            SendMessageW(
                hWnd,
                LVM_GETITEMRECT as UINT,
                (ListView_GetItemCount!(hWnd) - 1) as WPARAM,
                &mut rcItem as *mut RECT as LPARAM,
            );
            rcClip.bottom = rcItem.bottom;
            rcClip.right = rcItem.right;
            rcItem.left = LVIR_ICON as LONG;
            SendMessageW(
                hWnd,
                LVM_GETITEMRECT as UINT,
                0 as WPARAM,
                &mut rcItem as *mut RECT as LPARAM,
            );
            rcClip.left = rcItem.right;

            /*
             * Now exclude the clip rect
             */
            ExcludeClipRect(
                hDC,
                rcClip.left as i32,
                rcClip.top as i32,
                rcClip.right as i32,
                rcClip.bottom as i32,
            );

            /*
             * Now erase the background
             */
            GetClientRect(hWnd, &mut rcItem as *mut RECT);
            FillRect(hDC, &mut rcItem as *mut RECT, hbrBackground);

            /*
             * Now restore the DC state that we
             * saved earlier
             */
            RestoreDC(hDC, DcSave);

            return TRUE as INT_PTR;
        }
        _ => {}
    }

    /*
     * We pass on all messages except WM_ERASEBKGND
     */
    return CallWindowProcW(OldProcessListWndProc, hWnd, message, wParam, lParam);
}
