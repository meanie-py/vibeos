use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentProcessWin32Process};

// ---- file-scope preamble (translated) ----
pub const GDIDCATTRFREE: usize = 8;

#[repr(C)]
pub struct _GDI_DC_ATTR_FREELIST {
    pub Entry: _LIST_ENTRY,
    pub nEntries: DWORD,
    pub AttrList: [PVOID; 8],
}

pub type GDI_DC_ATTR_FREELIST = _GDI_DC_ATTR_FREELIST;
pub type PGDI_DC_ATTR_FREELIST = *mut _GDI_DC_ATTR_FREELIST;

#[repr(C)]
pub struct _GDI_DC_ATTR_ENTRY {
    pub Attr: [DC_ATTR; 8],
}

pub type GDI_DC_ATTR_ENTRY = _GDI_DC_ATTR_ENTRY;
pub type PGDI_DC_ATTR_ENTRY = *mut _GDI_DC_ATTR_ENTRY;
#[no_mangle]
pub unsafe extern "system" fn DC_bAllocDcAttr(pdc: PDC) -> BOOL {
    let ppi: PPROCESSINFO;
    let pdcattr: PDC_ATTR;

    ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    pdcattr = GdiPoolAllocate((*ppi).pPoolDcAttr) as PDC_ATTR;
    if pdcattr.is_null() {
        return 0;
    }

    (*pdc).pdcattr = pdcattr;
    *(*pdc).pdcattr = (*pdc).dcattr;

    GDIOBJ_vSetObjectAttr(core::ptr::addr_of_mut!((*pdc).BaseObject), pdcattr as *mut core::ffi::c_void);

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn DC_vFreeDcAttr(pdc: PDC) {
    let ppi: PPROCESSINFO;

    if (*pdc).pdcattr == core::ptr::addr_of_mut!((*pdc).dcattr) {
        return;
    }

    GDIOBJ_vSetObjectAttr(core::ptr::addr_of_mut!((*pdc).BaseObject), core::ptr::null_mut());

    ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    GdiPoolFree((*ppi).pPoolDcAttr, (*pdc).pdcattr as *mut core::ffi::c_void);

    (*pdc).pdcattr = core::ptr::addr_of_mut!((*pdc).dcattr);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           Functions for creation and destruction of DCs
//  * FILE:              win32ss/gdi/ntgdi/dcattr.c
//  * PROGRAMER:         Timo Kreuzer (timo.kreuzer@rectos.org)
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define GDIDCATTRFREE 8
//
// typedef struct _GDI_DC_ATTR_FREELIST
// {
//   LIST_ENTRY Entry;
//   DWORD nEntries;
//   PVOID AttrList[GDIDCATTRFREE];
// } GDI_DC_ATTR_FREELIST, *PGDI_DC_ATTR_FREELIST;
//
// typedef struct _GDI_DC_ATTR_ENTRY
// {
//   DC_ATTR Attr[GDIDCATTRFREE];
// } GDI_DC_ATTR_ENTRY, *PGDI_DC_ATTR_ENTRY;
//
//
// BOOL
// NTAPI
// DC_bAllocDcAttr(PDC pdc)
// {
//     PPROCESSINFO ppi;
//     PDC_ATTR pdcattr;
//
//     ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
//     ASSERT(ppi);
//
//     pdcattr = GdiPoolAllocate(ppi->pPoolDcAttr);
//     if (!pdcattr)
//     {
//         DPRINT1("Could not allocate DC attr\n");
//         return FALSE;
//     }
//
//     /* Copy the content from the kernel mode dc attr */
//     pdc->pdcattr = pdcattr;
//     *pdc->pdcattr = pdc->dcattr;
//
//     /* Set the object attribute in the handle table */
//     GDIOBJ_vSetObjectAttr(&pdc->BaseObject, pdcattr);
//
//     DPRINT("DC_AllocDcAttr: pdc=%p, pdc->pdcattr=%p\n", pdc, pdc->pdcattr);
//     return TRUE;
// }
//
// VOID
// NTAPI
// DC_vFreeDcAttr(PDC pdc)
// {
//     PPROCESSINFO ppi;
//
//     if (pdc->pdcattr == &pdc->dcattr)
//     {
//         // Internal DC object!
//         return;
//     }
//
//     /* Reset the object attribute in the handle table */
//     GDIOBJ_vSetObjectAttr(&pdc->BaseObject, NULL);
//
//     ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
//     ASSERT(ppi);
//     GdiPoolFree(ppi->pPoolDcAttr, pdc->pdcattr);
//
//     /* Reset to kmode dcattr */
//     pdc->pdcattr = &pdc->dcattr;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
