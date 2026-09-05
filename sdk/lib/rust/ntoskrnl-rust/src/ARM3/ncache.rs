use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    PHYSICAL_ADDRESS,
    PMDL, _MDL,
    PPFN_NUMBER,
    MiAllocatePagesForMdl,
    MmFreePagesFromMdl,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub unsafe extern "system" fn MmAllocateNonCachedMemory(NumberOfBytes: SIZE_T) -> PVOID {
    let mut PageCount: PFN_COUNT;
    let MdlPageCount: PFN_COUNT;
    let mut PageFrameIndex: PFN_NUMBER;
    let mut LowAddress: PHYSICAL_ADDRESS;
    let mut HighAddress: PHYSICAL_ADDRESS;
    let mut SkipBytes: PHYSICAL_ADDRESS;
    let CacheAttribute: MI_PFN_CACHE_ATTRIBUTE;
    let Mdl: PMDL;
    let mut BaseAddress: PVOID;
    let mut MdlPages: PPFN_NUMBER;
    let mut PointerPte: PMMPTE;
    let mut TempPte: MMPTE;

    ntkrnl_sys::ASSERT!(NumberOfBytes != 0);
    PageCount = BYTES_TO_PAGES(NumberOfBytes) as PFN_COUNT;

    LowAddress = core::mem::zeroed();
    HighAddress = core::mem::zeroed();
    SkipBytes = core::mem::zeroed();

    LowAddress.QuadPart = 0;
    HighAddress.QuadPart = -1i64;
    SkipBytes.QuadPart = 0;
    CacheAttribute = MiPlatformCacheAttributes[0][MmNonCached as usize];

    Mdl = MiAllocatePagesForMdl(
        LowAddress,
        HighAddress,
        SkipBytes,
        NumberOfBytes as _,
        CacheAttribute,
        0,
    );
    if Mdl.is_null() {
        return core::ptr::null_mut();
    }

    BaseAddress = ((*Mdl).StartVa as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID;
    MdlPageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(BaseAddress, (*Mdl).ByteCount as SIZE_T) as PFN_COUNT;

    if PageCount != MdlPageCount {
        ntkrnl_sys::ASSERT!(PageCount > MdlPageCount);
        MmFreePagesFromMdl(Mdl);
        ExFreePoolWithTag(Mdl as PVOID, TAG_MDL);
        return core::ptr::null_mut();
    }

    PointerPte = MiReserveSystemPtes(PageCount + 1, SystemPteSpace);
    if PointerPte.is_null() {
        MmFreePagesFromMdl(Mdl);
        ExFreePoolWithTag(Mdl as PVOID, TAG_MDL);
        return core::ptr::null_mut();
    }

    *(PointerPte as *mut PMDL) = Mdl;
    PointerPte = PointerPte.add(1);

    BaseAddress = MiPteToAddress(PointerPte);

    MdlPages = (Mdl as ULONG_PTR + core::mem::size_of::<_MDL>()) as PPFN_NUMBER;

    TempPte = ValidKernelPte;

    match CacheAttribute {
        x if x == MiNonCached => {
            MI_PAGE_DISABLE_CACHE(&mut TempPte);
            MI_PAGE_WRITE_THROUGH(&mut TempPte);
        }
        x if x == MiWriteCombined => {
            MI_PAGE_DISABLE_CACHE(&mut TempPte);
            MI_PAGE_WRITE_COMBINED(&mut TempPte);
        }
        _ => {}
    }

    loop {
        PageFrameIndex = *MdlPages as PFN_NUMBER;
        MdlPages = MdlPages.add(1);

        TempPte.set_page_frame_number(PageFrameIndex);
        MI_WRITE_VALID_PTE(PointerPte, TempPte);
        PointerPte = PointerPte.add(1);

        PageCount -= 1;
        if PageCount == 0 {
            break;
        }
    }

    BaseAddress
}

#[no_mangle]
pub unsafe extern "system" fn MmFreeNonCachedMemory(
    BaseAddress: PVOID,
    NumberOfBytes: SIZE_T,
) {
    let BaseAddress = opaque_ptr(BaseAddress);
    let Mdl: PMDL;
    let mut PointerPte: PMMPTE;
    let PageCount: PFN_COUNT;

    ntkrnl_sys::ASSERT!(NumberOfBytes != 0);
    ntkrnl_sys::ASSERT!(PAGE_ALIGN(BaseAddress as usize) == BaseAddress as usize);

    PageCount = BYTES_TO_PAGES(NumberOfBytes) as PFN_COUNT;

    PointerPte = MiAddressToPte(BaseAddress);

    PointerPte = PointerPte.sub(1);
    Mdl = *(PointerPte as *mut PMDL);

    MmFreePagesFromMdl(Mdl);
    ExFreePoolWithTag(Mdl as PVOID, TAG_MDL);

    MiReleaseSystemPtes(PointerPte, PageCount + 1, SystemPteSpace);
}
