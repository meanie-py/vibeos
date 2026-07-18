use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeInitializeEvent, PsGetProcessWin32Process};
use ntkrnl_sys::base::{STATUS_SUCCESS, InitializeListHead};

extern "C" {
    fn DbgPrint(format: *const u8, ...) -> u32;
}
// APIENTRY (stdcall i386 / cdecl else) + NTAPI helpers — all extern "system".
extern "system" {
    fn DxDdStartupDxGraphics(arg0: u32, arg1: *mut core::ffi::c_void, arg2: u32, arg3: *mut core::ffi::c_void, arg4: *mut core::ffi::c_void, arg5: PEPROCESS) -> i32;
    fn DxStartupDxgkInt();
    fn GDI_MapHandleTable(Process: PEPROCESS) -> *mut core::ffi::c_void;
    fn GDI_CleanupForProcess(Process: PEPROCESS) -> u8;
}
// InitFontSupport is FASTCALL (i386 @InitFontSupport@0); cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InitFontSupport() -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InitFontSupport() -> BOOL;
}

// ExInitializeFastMutex is a DDK FORCEINLINE (no exported symbol) — inline it.
#[inline(always)]
unsafe fn ExInitializeFastMutex_rs(fm: *mut FAST_MUTEX) {
    (*fm).Count = 1; // FM_LOCK_BIT
    (*fm).Owner = core::ptr::null_mut();
    (*fm).Contention = 0;
    // SynchronizationEvent = 1, initial state FALSE
    KeInitializeEvent(core::ptr::addr_of_mut!((*fm).Event) as *mut _, 1, 0);
}

// ERR(fmt, ...) is a win32k debug macro -> route to DbgPrint.
unsafe fn err_msg(msg: *const u8) {
    DbgPrint(msg);
}

// WoW64 guests consume win32k's user-visible mappings through 32-bit
// pointers, so every section view / VM reservation win32k hands such a
// process must land below 4GB. ZeroBits mask form (value > 32 ⇒ the value IS
// the address ceiling) — honored by both MmMapViewOfSection and
// Zw/NtAllocateVirtualMemory (ARM3). 0x7FFF_FFFF keeps us below 2GB, safe
// for any 32-bit guest regardless of /LARGEADDRESSAWARE.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub(crate) unsafe fn W32Wow64ZeroBits(Process: PEPROCESS) -> usize {
    if (*Process).Wow64Process.is_null() {
        0
    } else {
        0x7FFF_FFFF
    }
}
// A 32-bit host has no WoW64 layer (and its EPROCESS has no Wow64Process).
#[cfg(target_arch = "x86")]
#[inline]
pub(crate) unsafe fn W32Wow64ZeroBits(_Process: PEPROCESS) -> usize {
    0
}

// ---- file-scope preamble (translated) ----
// gusLanguageID is read by freetype.c (still C) — must export the symbol.
#[no_mangle]
pub static mut gusLanguageID: USHORT = 0;

// All four process/thread callouts and InitializeGreCSRSS are plain cdecl
// (declared without NTAPI in ntuser/main.c + misc.h, called directly) — export
// them as extern "C" so the win32k init path links.
#[no_mangle]
pub unsafe extern "C" fn GdiProcessCreate(Process: PEPROCESS) -> NTSTATUS {
    let ppiCurrent: PPROCESSINFO = PsGetProcessWin32Process(Process as *mut _) as PPROCESSINFO;
    // The W32PROCESS base is bindgen-opaque padding in _PROCESSINFO; reach its
    // GDI*AttrFreeList fields by casting to the base type (it sits at offset 0).
    let pw32: *mut _W32PROCESS = ppiCurrent as *mut _W32PROCESS;

    InitializeListHead(core::ptr::addr_of_mut!((*ppiCurrent).PrivateFontListHead) as *mut _);
    InitializeListHead(core::ptr::addr_of_mut!((*ppiCurrent).PrivateMemFontListHead) as *mut _);
    (*ppiCurrent).PrivateMemFontHandleCount = 0;
    ExInitializeFastMutex_rs(core::ptr::addr_of_mut!((*ppiCurrent).PrivateFontListLock));

    InitializeListHead(core::ptr::addr_of_mut!((*pw32).GDIBrushAttrFreeList) as *mut _);
    InitializeListHead(core::ptr::addr_of_mut!((*pw32).GDIDcAttrFreeList) as *mut _);

    (*(*Process).Peb).GdiSharedHandleTable = GDI_MapHandleTable(Process);
    (*(*Process).Peb).GdiDCAttributeList = GDI_BATCH_LIMIT as _;

    // WoW64: the guest's 32-bit gdi32 reads PEB32->GdiSharedHandleTable, not
    // PEB64 — mirror the (below-4GB, see W32Wow64ZeroBits) mapping into it.
    // (x86 host: no WoW64 layer, and no _PEB32/Wow64Process in its bindgen.)
    #[cfg(not(target_arch = "x86"))]
    {
        let Peb32 = (*Process).Wow64Process as usize as *mut ntkrnl_sys::ddk::_PEB32;
        if !Peb32.is_null() {
            (*Peb32).GdiSharedHandleTable =
                (*(*Process).Peb).GdiSharedHandleTable as usize as u32;
            (*Peb32).GdiDCAttributeList = GDI_BATCH_LIMIT as _;
        }
    }

    (*ppiCurrent).pPoolDcAttr = GdiPoolCreate(core::mem::size_of::<DC_ATTR>() as _, u32::from_le_bytes(*b"Gdca"));
    (*ppiCurrent).pPoolBrushAttr = GdiPoolCreate(core::mem::size_of::<BRUSH_ATTR>() as _, u32::from_le_bytes(*b"Gbra"));
    (*ppiCurrent).pPoolRgnAttr = GdiPoolCreate(core::mem::size_of::<RGN_ATTR>() as _, u32::from_le_bytes(*b"Grga"));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn GdiProcessDestroy(Process: PEPROCESS) -> NTSTATUS {
    let ppiCurrent: PPROCESSINFO = PsGetProcessWin32Process(Process as *mut _) as PPROCESSINFO;

    IntGdiCleanupPrivateFontsForProcess();

    GDI_CleanupForProcess(Process);

    GdiPoolDestroy((*ppiCurrent).pPoolDcAttr);
    GdiPoolDestroy((*ppiCurrent).pPoolBrushAttr);
    GdiPoolDestroy((*ppiCurrent).pPoolRgnAttr);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn GdiThreadCreate(Thread: PETHREAD) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn GdiThreadDestroy(Thread: PETHREAD) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn InitializeGreCSRSS() -> BOOL {
    // Initialize Dxgkrnl interfaces and run startup routine
    DxStartupDxgkInt();

    // Initialize Legacy DirectX graphics driver
    if DxDdStartupDxGraphics(
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        gpepCSRSS,
    ) != STATUS_SUCCESS as i32
    {
        #[cfg(target_arch = "aarch64")]
        {
            // dxg.sys is not available on ARM64 — continue without DirectX
            err_msg(b"DirectX graphics unavailable on ARM64 (non-fatal)\n\0".as_ptr());
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            err_msg(b"Unable to initialize DirectX graphics\n\0".as_ptr());
            return 0;
        }
    }

    // Get global language ID
    gusLanguageID = UserGetLanguageID();

    DbgPrint(b"InitializeGreCSRSS: calling InitFontSupport\n\0".as_ptr());
    // Initialize FreeType library
    if InitFontSupport() == 0 {
        err_msg(b"Unable to initialize font support\n\0".as_ptr());
        return 0;
    }
    DbgPrint(b"InitializeGreCSRSS: InitFontSupport done\n\0".as_ptr());

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiInit() -> BOOL {
    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiInit() -> BOOL {
    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Initialization of GDI
//  * FILE:             win32ss/gdi/ntgdi/init.c
//  * PROGRAMER:
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(UserMisc);
//
// USHORT gusLanguageID;
//
// BOOL NTAPI GDI_CleanupForProcess(struct _EPROCESS *Process);
//
// NTSTATUS
// GdiProcessCreate(PEPROCESS Process)
// {
//     PPROCESSINFO ppiCurrent = PsGetProcessWin32Process(Process);
//     ASSERT(ppiCurrent);
//
//     InitializeListHead(&ppiCurrent->PrivateFontListHead);
//     InitializeListHead(&ppiCurrent->PrivateMemFontListHead);
//     ppiCurrent->PrivateMemFontHandleCount = 0;
//     ExInitializeFastMutex(&ppiCurrent->PrivateFontListLock);
//
//     InitializeListHead(&ppiCurrent->GDIBrushAttrFreeList);
//     InitializeListHead(&ppiCurrent->GDIDcAttrFreeList);
//
//     /* Map the GDI handle table to user land */
//     Process->Peb->GdiSharedHandleTable = GDI_MapHandleTable(Process);
//     Process->Peb->GdiDCAttributeList = GDI_BATCH_LIMIT;
//
//     /* Create pools for GDI object attributes */
//     ppiCurrent->pPoolDcAttr = GdiPoolCreate(sizeof(DC_ATTR), 'acdG');
//     ppiCurrent->pPoolBrushAttr = GdiPoolCreate(sizeof(BRUSH_ATTR), 'arbG');
//     ppiCurrent->pPoolRgnAttr = GdiPoolCreate(sizeof(RGN_ATTR), 'agrG');
//     ASSERT(ppiCurrent->pPoolDcAttr);
//     ASSERT(ppiCurrent->pPoolBrushAttr);
//     ASSERT(ppiCurrent->pPoolRgnAttr);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// GdiProcessDestroy(PEPROCESS Process)
// {
//     PPROCESSINFO ppiCurrent = PsGetProcessWin32Process(Process);
//     ASSERT(ppiCurrent);
//     ASSERT(ppiCurrent->peProcess == Process);
//
//     IntGdiCleanupPrivateFontsForProcess();
//
//     /* And GDI ones too */
//     GDI_CleanupForProcess(Process);
//
//     /* So we can now free the pools */
//     GdiPoolDestroy(ppiCurrent->pPoolDcAttr);
//     GdiPoolDestroy(ppiCurrent->pPoolBrushAttr);
//     GdiPoolDestroy(ppiCurrent->pPoolRgnAttr);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// GdiThreadCreate(PETHREAD Thread)
// {
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// GdiThreadDestroy(PETHREAD Thread)
// {
//     return STATUS_SUCCESS;
// }
//
// BOOL
// InitializeGreCSRSS(VOID)
// {
//     /* Initialize Dxgkrnl interfaces and run startup routine */
//     DxStartupDxgkInt();
//
//     /* Initialize Legacy DirectX graphics driver */
//     if (DxDdStartupDxGraphics(0, NULL, 0, NULL, NULL, gpepCSRSS) != STATUS_SUCCESS)
//     {
// #ifdef _M_ARM64
//         /* dxg.sys is not available on ARM64 — continue without DirectX */
//         ERR("DirectX graphics unavailable on ARM64 (non-fatal)\n");
// #else
//         ERR("Unable to initialize DirectX graphics\n");
//         return FALSE;
// #endif
//     }
//
//     /* Get global language ID */
//     gusLanguageID = UserGetLanguageID();
//
//     DbgPrint("InitializeGreCSRSS: calling InitFontSupport\n");
//     /* Initialize FreeType library */
//     if (!InitFontSupport())
//     {
//         ERR("Unable to initialize font support\n");
//         return FALSE;
//     }
//     DbgPrint("InitializeGreCSRSS: InitFontSupport done\n");
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// NtGdiInit(VOID)
// {
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
