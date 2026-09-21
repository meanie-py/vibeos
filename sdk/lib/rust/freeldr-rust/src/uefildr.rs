use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::settings::BootMgrInfo;
use ntkrnl_sys::ddk::PCSTR;
use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderOsloaderStack as LoaderOsloaderStack;
use core::ffi::c_void;

// EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL minimal surface (frldr.rs ConOut is
// opaque `*mut c_void`; cast through this here). Only OutputString is
// used in this file; keep the rest as opaque pads to match the C layout.
#[repr(C)]
pub struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL {
    pub Reset: *mut c_void,
    pub OutputString: Option<unsafe extern "C" fn(This: *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL, String: *mut u16) -> EFI_STATUS>,
    pub TestString: *mut c_void,
    pub QueryMode: *mut c_void,
    pub SetMode: *mut c_void,
    pub SetAttribute: *mut c_void,
    pub ClearScreen: *mut c_void,
    pub SetCursorPosition: *mut c_void,
    pub EnableCursor: *mut c_void,
    pub Mode: *mut c_void,
}

// FreeLDR debug-no-op variadic shims — frldr.rs #[no_mangle]-exports the
// non-variadic real bodies (cdecl, caller-cleans). Variadic call-form
// here typechecks the format-string + args without changing link.
extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn WARN(fmt: *const i8, ...);
}

// uefildr.c is the OWNER of the uefildr.exe binary entry (EfiEntry) — it
// lives in UEFILDR_BASE_SOURCE (add_executable), not in the .a archive.
// Per AGENTS rule: never delete a Rust translation; instead, gate the
// #[no_mangle] exports behind the off-by-default `uefildr-rust-entry`
// feature so the .rs compiles into libfreeldr_rust.a without competing
// for the symbol against uefildr.c. A future cohabitation flip will
// flip the feature on AND comment out uefildr.c.
//
// Local extern decls — these live in other .a-bound units, not frldr.rs.
extern "C" {
    fn LoadSettings(CmdLine: PCSTR);
    fn DebugInit(DebugString: PCSTR);
    fn MachInit(CmdLine: *const i8);
    fn UiInitialize(ShowUi: BOOLEAN) -> BOOLEAN;
    fn MmInitializeMemoryManager() -> BOOLEAN;
    fn FsInit();
    fn PeLdrInitializeModuleList() -> BOOLEAN;
    // MachInitializeBootDevices() is a C macro (MachVtbl.InitializeBootDevices()),
    // not a real symbol — provided by the shared crate::frldr helper instead.
    fn MmAllocateMemoryWithType(MemorySize: SIZE_T, MemoryType: ULONG) -> PVOID;
    fn RunLoader();
    fn _changestack();
}

// uefildr.c globals — OWNED here. Gate the #[no_mangle] like the entry
// fns so we don't dup-define against the C until the full flip.
#[cfg_attr(feature = "uefildr-rust-entry", no_mangle)]
pub static mut GlobalImageHandle: EFI_HANDLE = core::ptr::null_mut();
#[cfg_attr(feature = "uefildr-rust-entry", no_mangle)]
pub static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE = core::ptr::null_mut();
#[cfg_attr(feature = "uefildr-rust-entry", no_mangle)]
pub static mut UefiServiceStack: PVOID = core::ptr::null_mut();
#[cfg_attr(feature = "uefildr-rust-entry", no_mangle)]
pub static mut BasicStack: PVOID = core::ptr::null_mut();

#[cfg_attr(feature = "uefildr-rust-entry", no_mangle)]
pub unsafe extern "C" fn EfiEntry(
    ImageHandle: EFI_HANDLE,
    SystemTable: *mut EFI_SYSTEM_TABLE,
) -> EFI_STATUS {
    let CmdLine: PCSTR = b"\0".as_ptr() as PCSTR;

    let conout = (*SystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    ((*conout).OutputString.unwrap())(
        conout,
        crate::wide!("UEFI EntryPoint: Starting freeldr from UEFI\r\n\0") as *mut u16,
    );
    GlobalImageHandle = ImageHandle;
    GlobalSystemTable = SystemTable;

    #[cfg(target_arch = "aarch64")]
    {
        if !crate::arm64_debug::initialize(SystemTable) {
            if let Some(output) = (*conout).OutputString {
                output(
                    conout,
                    crate::wide!("[freeldr] No supported PL011 UART in ACPI SPCR/DBG2; cannot continue.\r\n\0") as *mut u16,
                );
            }
            return EFI_UNSUPPORTED;
        }
        dbgcon_str(b"[freeldr] ACPI PL011 base=0x");
        dbgcon_hex64(crate::arm64_debug::uart_base() as u64);
        dbgcon_str(b"\r\n");
    }

    LoadSettings(CmdLine);

    DebugInit(BootMgrInfo.DebugString);

    MachInit(CmdLine);
    let conout = (*SystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    ((*conout).OutputString.unwrap())(
        conout,
        crate::wide!("[freeldr] MachInit done\r\n\0") as *mut u16,
    );

    if UiInitialize(0u8) == 0 {
        UiMessageBoxCritical(b"Unable to initialize UI.\0".as_ptr() as *const i8);
        Reboot();
    }
    let conout = (*SystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    ((*conout).OutputString.unwrap())(
        conout,
        crate::wide!("[freeldr] UI initialized\r\n\0") as *mut u16,
    );

    if MmInitializeMemoryManager() == 0 {
        UiMessageBoxCritical(b"Unable to initialize memory manager.\0".as_ptr() as *const i8);
        Reboot();
    }
    let conout = (*SystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    ((*conout).OutputString.unwrap())(
        conout,
        crate::wide!("[freeldr] Memory manager initialized\r\n\0") as *mut u16,
    );

    FsInit();
    let conout = (*SystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    ((*conout).OutputString.unwrap())(
        conout,
        crate::wide!("[freeldr] Filesystem initialized\r\n\0") as *mut u16,
    );

    if PeLdrInitializeModuleList() == 0 {
        UiMessageBoxCritical(b"Unable to initialize module list.\0".as_ptr() as *const i8);
        Reboot();
    }
    let conout = (*SystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    ((*conout).OutputString.unwrap())(
        conout,
        crate::wide!("[freeldr] Module list initialized\r\n\0") as *mut u16,
    );

    if MachInitializeBootDevices() == 0 {
        UiMessageBoxCritical(b"Error when detecting hardware.\0".as_ptr() as *const i8);
        Reboot();
    }
    let conout = (*SystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    ((*conout).OutputString.unwrap())(
        conout,
        crate::wide!("[freeldr] Boot devices initialized\r\n\0") as *mut u16,
    );

    BasicStack = (0x32000 as ULONG_PTR + MmAllocateMemoryWithType(0x32000, LoaderOsloaderStack as ULONG) as ULONG_PTR) as PVOID;
    let conout = (*SystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    ((*conout).OutputString.unwrap())(
        conout,
        crate::wide!("[freeldr] Switching stack...\r\n\0") as *mut u16,
    );
    _changestack();

    Reboot();
}

#[cfg_attr(feature = "uefildr-rust-entry", no_mangle)]
pub unsafe extern "C" fn ExecuteLoaderCleanly(PreviousStack: PVOID) -> ! {
    TRACE(b"ExecuteLoaderCleanly Entry\n\0".as_ptr() as *const i8);
    UefiServiceStack = PreviousStack;

    RunLoader();

    WARN(b"RunLoader returned, halting\n\0".as_ptr() as *const i8);
    Reboot();
}

// C: `#if !defined(_M_ARM)` — gate out 32-bit ARM; arm64 is allowed.
#[cfg(not(target_arch = "arm"))]
#[cfg_attr(feature = "uefildr-rust-entry", no_mangle)]
pub unsafe extern "C" fn Reboot() -> ! {
    WARN(b"Something has gone wrong - halting FreeLoader\n\0".as_ptr() as *const i8);
    loop {}
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Entry point and helpers
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// #include <uefildr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WARNING);
//
// /* GLOBALS ********************************************************************/
//
// EFI_HANDLE GlobalImageHandle;
// EFI_SYSTEM_TABLE *GlobalSystemTable;
// PVOID UefiServiceStack;
// PVOID BasicStack;
//
// void _changestack(VOID);
//
// /* FUNCTIONS ******************************************************************/
//
// EFI_STATUS
// EfiEntry(
//     _In_ EFI_HANDLE ImageHandle,
//     _In_ EFI_SYSTEM_TABLE *SystemTable)
// {
//     PCSTR CmdLine = ""; // FIXME: Determine a command-line from UEFI boot options
//
//     SystemTable->ConOut->OutputString(SystemTable->ConOut, L"UEFI EntryPoint: Starting freeldr from UEFI\r\n");
//     GlobalImageHandle = ImageHandle;
//     GlobalSystemTable = SystemTable;
//
//     /* Load the default settings from the command-line */
//     LoadSettings(CmdLine);
//
//     /* Debugger pre-initialization */
//     DebugInit(BootMgrInfo.DebugString);
//
//     MachInit(CmdLine);
//     SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] MachInit done\r\n");
//
//     /* UI pre-initialization */
//     if (!UiInitialize(FALSE))
//     {
//         UiMessageBoxCritical("Unable to initialize UI.");
//         goto Quit;
//     }
//     SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] UI initialized\r\n");
//
//     /* Initialize memory manager */
//     if (!MmInitializeMemoryManager())
//     {
//         UiMessageBoxCritical("Unable to initialize memory manager.");
//         goto Quit;
//     }
//     SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Memory manager initialized\r\n");
//
//     /* Initialize I/O subsystem */
//     FsInit();
//     SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Filesystem initialized\r\n");
//
//     /* Initialize the module list */
//     if (!PeLdrInitializeModuleList())
//     {
//         UiMessageBoxCritical("Unable to initialize module list.");
//         goto Quit;
//     }
//     SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Module list initialized\r\n");
//
//     if (!MachInitializeBootDevices())
//     {
//         UiMessageBoxCritical("Error when detecting hardware.");
//         goto Quit;
//     }
//     SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Boot devices initialized\r\n");
//
//     /* 0x32000 is what UEFI defines, but we can go smaller if we want */
//     BasicStack = (PVOID)((ULONG_PTR)0x32000 + (ULONG_PTR)MmAllocateMemoryWithType(0x32000, LoaderOsloaderStack));
//     SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Switching stack...\r\n");
//     _changestack();
//
// Quit:
//     /* If we reach this point, something went wrong before, therefore reboot */
//     Reboot();
//
//     UNREACHABLE;
//     return 0;
// }
//
// void
// ExecuteLoaderCleanly(PVOID PreviousStack)
// {
//     TRACE("ExecuteLoaderCleanly Entry\n");
//     UefiServiceStack = PreviousStack;
//
//     RunLoader();
//
//     /* RunLoader can return if no boot configuration was found */
//     WARN("RunLoader returned, halting\n");
//     Reboot();
// }
//
// #if !defined(_M_ARM)
// DECLSPEC_NORETURN
// VOID __cdecl Reboot(VOID)
// {
//     //TODO: Replace with a true firmware reboot eventually
//     WARN("Something has gone wrong - halting FreeLoader\n");
//     for (;;)
//     {
//         NOTHING;
//     }
// }
// #endif
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
