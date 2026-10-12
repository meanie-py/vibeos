use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn IdePropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    DPRINT1!("IdePropPageProvider(%p %p %lx)\n",
           lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
    (FALSE as i32)
}
