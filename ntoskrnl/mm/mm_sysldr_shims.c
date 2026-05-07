/*
 * mm_sysldr_shims.c — C implementations for sysldr functions that are stubs
 * in the Rust translation (ntoskrnl-rust/src/ARM3/sysldr.rs).
 *
 * MmLoadSystemImage and its internal helpers are complex (~1000 lines total)
 * and depend on many kernel subsystems. Rather than translating them to Rust
 * immediately, we keep the proven C implementations and let the Rust code
 * handle the simpler sysldr functions (module list management, exports, etc.).
 *
 * Copied from ntoskrnl/mm/ARM3/sysldr.c with minimal modifications.
 */

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

#define MODULE_INVOLVED_IN_ARM3
#include <mm/ARM3/miarm.h>

/* Globals provided by Rust sysldr.rs (#[no_mangle]) */
extern LIST_ENTRY PsLoadedModuleList;
extern LIST_ENTRY MmLoadedUserImageList;
extern KSPIN_LOCK PsLoadedModuleSpinLock;
extern ERESOURCE PsLoadedModuleResource;
extern ULONG_PTR PsNtosImageBase;
extern KMUTANT MmSystemLoadLock;
extern BOOLEAN MmEnforceWriteProtection;

/* Functions provided by Rust sysldr.rs (#[no_mangle]) */
NTSTATUS NTAPI MmCallDllInitialize(PLDR_DATA_TABLE_ENTRY, PLIST_ENTRY);
NTSTATUS NTAPI MmUnloadSystemImage(PVOID);
NTSTATUS NTAPI MmCheckSystemImage(HANDLE);

/* Forward declarations for static helpers in this file */
/* Not static — also called from driver.c LdrProcessDriverModule */
NTSTATUS NTAPI MiResolveImageReferences(PVOID, PUNICODE_STRING, PUNICODE_STRING, PCHAR *, PWCHAR *, PLOAD_IMPORTS *);
/* MiSnapThunk + NameToOrdinal: translated to Rust (sysldr.rs); externs supplied
 * under the same names by the kernel-rust archive. */
NTSTATUS NTAPI MiSnapThunk(PVOID, PVOID, PIMAGE_THUNK_DATA, PIMAGE_THUNK_DATA, PIMAGE_EXPORT_DIRECTORY, ULONG, BOOLEAN, PCHAR *);

/* MmLoadSystemImage + MiResolveImageReferences + MiSnapThunk + NameToOrdinal
 * + MiLoadImageSection + MiLookupDataTableEntry are all in Rust now
 * (sdk/lib/rust/ntoskrnl-rust/src/ARM3/sysldr.rs). The 7 small static helpers
 * (MiCacheImageSymbols/MiProcessLoaderEntry/MiUseLargeDriverPage/etc.) moved
 * with MmLoadSystemImage and live there too. This file is intentionally
 * minimal — just the externs the kernel_rust archive wants resolved. */

/* MiLookupDataTableEntry — translated to Rust (sysldr.rs::MiLookupDataTableEntry)
 * and exported under the same name; C body deleted. */
