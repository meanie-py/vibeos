use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DriverProc(
    driver_id: DWORD,
    driver_handle: HANDLE,
    message: UINT,
    parameter1: LONG,
    parameter2: LONG,
) -> LONG {
    if message == DRV_LOAD {
        DPRINT!("DRV_LOAD\n");
        return 1;
    } else if message == DRV_FREE {
        DPRINT!("DRV_FREE\n");
        return 1;
    } else if message == DRV_OPEN {
        DPRINT!("DRV_OPEN\n");
        return 1;
    } else if message == DRV_CLOSE {
        DPRINT!("DRV_CLOSE\n");
        return 1;
    } else if message == DRV_ENABLE {
        DPRINT!("DRV_ENABLE\n");
        return 1;
    } else if message == DRV_DISABLE {
        DPRINT!("DRV_DISABLE\n");
        return 1;
    } else if message == DRV_QUERYCONFIGURE {
        // We don't provide configuration capabilities. This used to be
        // for things like I/O port, IRQ, DMA settings, etc.
        DPRINT!("DRV_QUERYCONFIGURE\n");
        return 0;
    } else if message == DRV_CONFIGURE {
        DPRINT!("DRV_CONFIGURE\n");
        return 0;
    } else if message == DRV_INSTALL {
        DPRINT!("DRV_INSTALL\n");
        return (DRVCNF_RESTART) as LONG;
    }

    return DefDriverProc(
        driver_id as DWORD_PTR,
        driver_handle as HDRVR,
        message,
        parameter1 as LPARAM,
        parameter2 as LPARAM,
    ) as LONG;
}
