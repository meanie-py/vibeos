use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ddk::{
    MI_LARGE_PAGE_RANGES, _EPROCESS,
};

#[no_mangle]
pub static mut MmProcessList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MiLargePageHyperPte: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut MiLargePageRangeIndex: ULONG = 0;
#[no_mangle]
pub static mut MiLargePageRanges: [MI_LARGE_PAGE_RANGES; 64] = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmLargePageDriverBuffer: [WCHAR; 512] = [0; 512];
#[no_mangle]
pub static mut MmLargePageDriverBufferLength: ULONG = 0xFFFFFFFF;
#[no_mangle]
pub static mut MiLargePageDriverList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MiLargePageAllDrivers: BOOLEAN = 0;

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitializeLargePageSupport() {
    // Only for _MI_PAGING_LEVELS <= 2 (i386)
    #[cfg(target_arch = "x86")]
    {
        MiLargePageHyperPte = MiReserveSystemPtes(1, SystemPteSpace);
        (*MiLargePageHyperPte).Long = 0;

        InitializeListHead(core::ptr::addr_of_mut!(MmProcessList));
        InsertTailList(
            core::ptr::addr_of_mut!(MmProcessList),
            core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut _EPROCESS)).MmProcessLinks) as *mut LIST_ENTRY,
        );
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiSyncCachedRanges() {
    for i in 0..MiLargePageRangeIndex {
        let _ = i;
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitializeDriverLargePageList() {
    InitializeListHead(core::ptr::addr_of_mut!(MiLargePageDriverList));

    if MmLargePageDriverBufferLength == 0xFFFFFFFF {
        return;
    }

    let mut p = MmLargePageDriverBuffer.as_ptr();
    let pp = MmLargePageDriverBuffer.as_ptr().add((MmLargePageDriverBufferLength as usize) / core::mem::size_of::<WCHAR>());

    while p < pp {
        let ch = *p;

        if ch == b' ' as u16 || ch == b'\n' as u16 || ch == b'\r' as u16 || ch == b'\t' as u16 {
            p = p.add(1);
            continue;
        }

        if ch == b'*' as u16 {
            MiLargePageAllDrivers = 1;
            break;
        }

        p = p.add(1);
    }
}
