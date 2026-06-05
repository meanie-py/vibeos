// SPDX-License-Identifier: GPL-2.0-or-later
//
// win32ss/gdi/ntgdi/gdikdbgext.c → Rust (Phase 2c wave 26)
//
// KDBG extension for GDI. Provides !gdi.* KDBG commands.
//
// DBG_ENABLE_EVENT_LOGGING = 0 (win32ss/win32kp.h) → the eventlist command
// and its helper are compile-time dead; those #if blocks are omitted.
//
// ABI oracle (llvm-nm on gdikdbgext.c.obj):
//   i386:  _KdbIsMemoryValid          (T, cdecl)
//          _DbgGdiKdbgCliCallback@12  (T, stdcall/NTAPI — 3 args × 4 bytes)
//   amd64: KdbIsMemoryValid           (T, cdecl == C on amd64)
//          DbgGdiKdbgCliCallback      (T, cdecl == NTAPI on amd64)
//
// "extern \"system\"" resolves to stdcall on i386 and C on amd64/arm64, which
// matches NTAPI exactly.

use win32k_sys::win32k::*;
use core::ffi::c_char;

// ── External declarations ─────────────────────────────────────────────────────

// CRT string functions (libcntpr / libstrtol) — cdecl on all architectures.
//
// NOTE on i386 decoration: in Rust `extern "C"`, a name `foo` resolves to the
// symbol `_foo` on i386 (the GNU/MSVC cdecl leading underscore). So:
//   `fn _stricmp` → `__stricmp` in .obj  (matches libcntpr_rust.a symbol `__stricmp`)
//   `fn strtoul`  → `_strtoul`  in .obj  (matches libstrtol.a symbol `_strtoul`)
// Do NOT use #[link_name] here — that bypasses the automatic underscore and
// would produce `___stricmp` / `__strtoul` (one extra underscore) on i386.
extern "C" {
    /// Case-insensitive ASCII string compare.
    fn _stricmp(s1: *const c_char, s2: *const c_char) -> i32;

    /// Parse unsigned long from null-terminated string at base `base`.
    fn strtoul(nptr: *const c_char, endptr: *mut *mut c_char, base: i32) -> ULONG;

    /// Kernel debug print (variadic, cdecl).
    fn DbgPrint(Format: *const c_char, ...) -> u32;
}

// Kernel memory / process-attach APIs (stdcall on i386, cdecl on amd64/arm64).
extern "system" {
    fn MmIsAddressValid(VirtualAddress: PVOID) -> BOOLEAN;
    fn KeStackAttachProcess(Process: PKPROCESS, ApcState: PKAPC_STATE);
    fn KeUnstackDetachProcess(ApcState: PKAPC_STATE);
}

// GDI handle-table globals defined (and #[no_mangle] exported) in gdiobj.rs.
// Referencing them via extern keeps the two files independent — no use of
// the Rust identifier path needed.
extern "C" {
    static mut gpentHmgr: PENTRY;
    static mut gpaulRefCount: PULONG;
    static mut gulFirstUnused: ULONG;
    // gpepCSRSS is in win32k-sys bindgen but also in the GDI globals cluster;
    // declare against win32k-sys PEPROCESS so types match.
    static mut gpepCSRSS: PEPROCESS;
}

// ── Constants ─────────────────────────────────────────────────────────────────

const PAGE_SIZE: ULONG_PTR = 4096;

// GDIObjType_MAX_TYPE = 0x1e = 30 (ntgdityp.h).
const GDIObjType_MAX_TYPE: ULONG = 0x1e;
// RESERVE_ENTRIES_COUNT = 10 (gdiobj.h static const).
const RESERVE_ENTRIES_COUNT: ULONG = 10;

// ── ALIGN_DOWN_POINTER_BY macro expansion ────────────────────────────────────

/// ALIGN_DOWN_POINTER_BY(ptr, align) — rounds ptr down to the nearest
/// multiple of `align` (align must be a power of two).
#[inline(always)]
unsafe fn align_down_pointer_by(ptr: *const u8, align: ULONG_PTR) -> *mut u8 {
    ((ptr as ULONG_PTR) & !(align - 1)) as *mut u8
}

// ── Object-type name table ────────────────────────────────────────────────────

// Matches the C `gpszObjectTypes[]` exactly (indices 0..=32).
// Array index == GDI object type; 33 entries total (0x00..=0x1E + "ALL").
static OBJECT_TYPE_NAMES: [&[u8]; 33] = [
    b"FREE\0",      b"DC\0",        b"UNUSED1\0",   b"UNUSED2\0",
    b"RGN\0",       b"SURF\0",      b"CLIENTOBJ\0", b"PATH\0",
    b"PAL\0",       b"ICMLCS\0",   b"LFONT\0",     b"RFONT\0",
    b"PFE\0",       b"PFT\0",      b"ICMCXF\0",    b"SPRITE\0",
    b"BRUSH\0",     b"UMPD\0",     b"UNUSED4\0",   b"SPACE\0",
    b"UNUSED5\0",   b"META\0",     b"EFSTATE\0",   b"BMFD\0",
    b"VTFD\0",      b"TTFD\0",     b"RC\0",        b"TEMP\0",
    b"DRVOBJ\0",    b"DCIOBJ\0",   b"SPOOL\0",     b"RESERVED\0",
    b"ALL\0",
];

#[inline(always)]
unsafe fn obj_type_name(i: usize) -> *const c_char {
    OBJECT_TYPE_NAMES[i].as_ptr() as *const c_char
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// KdbGetHexNumber — parse a hex number from `pszNum` into `*pulValue`.
/// Skips optional 0x/0X prefix; returns TRUE (1) if entire string consumed.
/// Static in C; kept private in Rust.
unsafe fn KdbGetHexNumber(pszNum: *mut c_char, pulValue: *mut ULONG_PTR) -> BOOL {
    let mut p = pszNum;

    // Skip optional '0x' / '0X' prefix.
    if *p == b'0' as c_char
        && (*p.add(1) == b'x' as c_char || *p.add(1) == b'X' as c_char)
    {
        p = p.add(2);
    }

    let mut endptr: *mut c_char = core::ptr::null_mut();
    *pulValue = strtoul(p, &mut endptr, 16) as ULONG_PTR;

    // TRUE iff the whole string was consumed (endptr points at '\0').
    (*endptr == 0) as BOOL
}

// ── Command implementations ───────────────────────────────────────────────────

unsafe fn KdbCommand_Gdi_help() {
    DbgPrint(
        b"GDI KDBG extension.\nAvailable commands:\n\
          - help - Displays this screen.\n\
          - dumpht [<type>] - Dumps all handles of <type> or lists all types\n\
          - handle <handle> - Displays information about a handle\n\
          - entry <entry> - Displays an ENTRY, <entry> can be a pointer or index\n\
          - baseobject <object> - Displays a BASEOBJECT\n\0"
            .as_ptr() as *const c_char,
    );
}

unsafe fn KdbCommand_Gdi_dumpht(argc: ULONG, argv: *mut PCHAR) {
    let mut ApcState: KAPC_STATE = core::mem::zeroed();

    // No CSRSS → no handle table.
    if gpepCSRSS.is_null() {
        return;
    }
    // &gpepCSRSS->Pcb: Pcb (_KPROCESS) is the very first field of _EPROCESS
    // (offset 0), so casting PEPROCESS to PKPROCESS is correct.
    KeStackAttachProcess(gpepCSRSS as PKPROCESS, &mut ApcState);

    if argc == 0 {
        // No type argument: print per-type counts.
        let mut counts = [0u16; (GDIObjType_MAX_TYPE as usize) + 2];

        let mut i = RESERVE_ENTRIES_COUNT;
        while i < gulFirstUnused {
            let pentry = gpentHmgr.add(i as usize);
            if KdbIsMemoryValid(pentry as PVOID, core::mem::size_of::<ENTRY>() as ULONG) != 0 {
                let objt = ((*pentry).Objt & 0x1F) as usize;
                counts[objt] = counts[objt].wrapping_add(1);
            }
            i += 1;
        }

        DbgPrint(b"Type         Count\n\0".as_ptr() as *const c_char);
        DbgPrint(b"-------------------\n\0".as_ptr() as *const c_char);
        let mut i = 0u32;
        while i <= GDIObjType_MAX_TYPE {
            DbgPrint(
                b"%02x %-9s %d\n\0".as_ptr() as *const c_char,
                i,
                obj_type_name(i as usize),
                counts[i as usize] as u32,
            );
            i += 1;
        }
        DbgPrint(b"\n\0".as_ptr() as *const c_char);
    } else {
        let argv0: *mut c_char = *argv;

        // Find type by name (case-insensitive).
        let mut i = 0u32;
        while i <= GDIObjType_MAX_TYPE + 1 {
            if _stricmp(argv0, obj_type_name(i as usize)) == 0 {
                break;
            }
            i += 1;
        }

        // Not found by name — try as hex number.
        if i > GDIObjType_MAX_TYPE + 1 {
            let mut ulArg: ULONG_PTR = 0;
            if KdbGetHexNumber(argv0, &mut ulArg) == 0 {
                DbgPrint(
                    b"Invalid parameter: %s\n\0".as_ptr() as *const c_char,
                    argv0,
                );
                // fall through to leave:
                KeUnstackDetachProcess(&mut ApcState);
                return;
            }

            // Range check.
            if i > GDIObjType_MAX_TYPE {
                DbgPrint(
                    b"Unknown object type: %s\n\0".as_ptr() as *const c_char,
                    argv0,
                );
                // goto leave
                KeUnstackDetachProcess(&mut ApcState);
                return;
            }
        }

        let jRequestedType = i as UCHAR;

        DbgPrint(
            b"Index Handle   Type      pObject    ThreadId cLocks  ulRefCount\n\0"
                .as_ptr() as *const c_char,
        );
        DbgPrint(
            b"---------------------------------------------------------------\n\0"
                .as_ptr() as *const c_char,
        );

        let mut i = RESERVE_ENTRIES_COUNT;
        while i < gulFirstUnused {
            let pentry = gpentHmgr.add(i as usize);

            if MmIsAddressValid(pentry as PVOID) == 0 {
                i += 1;
                continue;
            }

            let pobj = (*pentry).einfo.pobj;
            let objt = (*pentry).Objt & 0x1F;

            if (jRequestedType == (GDIObjType_MAX_TYPE + 1) as UCHAR)
                || (objt == jRequestedType)
            {
                DbgPrint(
                    b"%04lx  %p %-9s 0x%p 0x%06lx %-6ld \0".as_ptr() as *const c_char,
                    i,
                    (*pobj).hHmgr,
                    obj_type_name((objt & 0x1F) as usize),
                    pobj,
                    (*pobj).__bindgen_anon_1.dwThreadId,
                    (*pobj).cExclusiveLock as u32,
                );
                let refcount_ptr = gpaulRefCount.add(i as usize);
                if MmIsAddressValid(refcount_ptr as PVOID) != 0 {
                    DbgPrint(
                        b"0x%08lx\n\0".as_ptr() as *const c_char,
                        *refcount_ptr,
                    );
                } else {
                    DbgPrint(b"??????????\n\0".as_ptr() as *const c_char);
                }
            }

            i += 1;
        }
    }

    // leave:
    KeUnstackDetachProcess(&mut ApcState);
}

unsafe fn KdbCommand_Gdi_handle(argv: *mut c_char) {
    let mut ulObject: ULONG_PTR = 0;
    let mut ApcState: KAPC_STATE = core::mem::zeroed();

    // Convert the parameter into a number.
    if KdbGetHexNumber(argv, &mut ulObject) == 0 {
        DbgPrint(
            b"Invalid parameter: %s\n\0".as_ptr() as *const c_char,
            argv,
        );
        return;
    }

    // No CSRSS → no handle table.
    if gpepCSRSS.is_null() {
        return;
    }
    KeStackAttachProcess(gpepCSRSS as PKPROCESS, &mut ApcState);

    let usIndex = (ulObject & 0xFFFF) as USHORT;
    let pentry = gpentHmgr.add(usIndex as usize);

    if MmIsAddressValid(pentry as PVOID) != 0 {
        let pobj = (*pentry).einfo.pobj;

        DbgPrint(
            b"GDI handle=%p, type=%s, index=0x%lx, pentry=%p.\n\0"
                .as_ptr() as *const c_char,
            ulObject,
            obj_type_name(((ulObject >> 16) & 0x1f) as usize),
            usIndex as ULONG,
            pentry,
        );
        DbgPrint(
            // Two-line format string identical to C source.
            b" ENTRY = {.pobj = %p, ObjectOwner = 0x%lx, FullUnique = 0x%04x,\n\
              \x20 Objt=0x%02x, Flags = 0x%02x, pUser = 0x%p}\n\0"
                .as_ptr() as *const c_char,
            (*pentry).einfo.pobj,
            (*pentry).ObjectOwner.ulObj,
            (*pentry).FullUnique as u32,
            (*pentry).Objt as u32,
            (*pentry).Flags as u32,
            (*pentry).pUser,
        );
        DbgPrint(
            b" BASEOBJECT = {hHmgr = %p, dwThreadId = 0x%lx,\n\
              \x20 cExclusiveLock = %ld, BaseFlags = 0x%lx}\n\0"
                .as_ptr() as *const c_char,
            (*pobj).hHmgr,
            (*pobj).__bindgen_anon_1.dwThreadId,
            (*pobj).cExclusiveLock as u32,
            (*pobj).BaseFlags as u32,
        );
        let refcount_ptr = gpaulRefCount.add(usIndex as usize);
        if MmIsAddressValid(refcount_ptr as PVOID) != 0 {
            DbgPrint(
                b" gpaulRefCount[idx] = %ld\n\0".as_ptr() as *const c_char,
                *refcount_ptr,
            );
        }
    } else {
        DbgPrint(
            b"Coudn't access ENTRY. Probably paged out.\n\0".as_ptr() as *const c_char,
        );
    }

    KeUnstackDetachProcess(&mut ApcState);
}

unsafe fn KdbCommand_Gdi_entry(argv: *mut c_char) {
    let mut ulValue: ULONG_PTR = 0;
    let mut ApcState: KAPC_STATE = core::mem::zeroed();

    // Convert the parameter into a number.
    if KdbGetHexNumber(argv, &mut ulValue) == 0 {
        DbgPrint(
            b"Invalid parameter: %s\n\0".as_ptr() as *const c_char,
            argv,
        );
        return;
    }

    // No CSRSS → no handle table.
    if gpepCSRSS.is_null() {
        return;
    }
    KeStackAttachProcess(gpepCSRSS as PKPROCESS, &mut ApcState);

    // If the parameter is smaller than 0x10000, treat it as a table index.
    let pentry: PENTRY = if ulValue <= 0xFFFF {
        gpentHmgr.add(ulValue as usize)
    } else {
        ulValue as PENTRY
    };

    // Check if the address is readable.
    if MmIsAddressValid(pentry as PVOID) == 0 {
        DbgPrint(
            b"Cannot access entry at %p\n\0".as_ptr() as *const c_char,
            pentry,
        );
        // goto cleanup
        KeUnstackDetachProcess(&mut ApcState);
        return;
    }

    // Dump the entry fields.
    let idx = pentry.offset_from(gpentHmgr) as ULONG;
    DbgPrint(
        b"Dumping ENTRY #%ld, @%p:\n\0".as_ptr() as *const c_char,
        idx,
        pentry,
    );
    if (*pentry).Objt != 0 {
        DbgPrint(
            b" pobj = 0x%p\n\0".as_ptr() as *const c_char,
            (*pentry).einfo.pobj,
        );
    } else {
        DbgPrint(
            b" hFree = 0x%p\n\0".as_ptr() as *const c_char,
            (*pentry).einfo.hFree,
        );
    }
    DbgPrint(
        b" ObjectOwner = 0x%p\n\0".as_ptr() as *const c_char,
        (*pentry).ObjectOwner.ulObj,
    );
    DbgPrint(
        b" FullUnique = 0x%x\n\0".as_ptr() as *const c_char,
        (*pentry).FullUnique as u32,
    );
    let objt = (*pentry).Objt as ULONG;
    let type_name: *const c_char = if objt <= 0x1E {
        obj_type_name(objt as usize)
    } else {
        b"invalid\0".as_ptr() as *const c_char
    };
    DbgPrint(
        b" Objt = 0x%x (%s)\n\0".as_ptr() as *const c_char,
        objt,
        type_name,
    );
    DbgPrint(
        b" Flags = 0x%x\n\0".as_ptr() as *const c_char,
        (*pentry).Flags as u32,
    );
    DbgPrint(
        b" pUser = 0x%p\n\0".as_ptr() as *const c_char,
        (*pentry).pUser,
    );

    // cleanup:
    KeUnstackDetachProcess(&mut ApcState);
}

/// KdbCommand_Gdi_baseobject — empty in C source; preserve as no-op.
unsafe fn KdbCommand_Gdi_baseobject(_argv: *mut c_char) {}

// ── Exported symbols ──────────────────────────────────────────────────────────

/// KdbIsMemoryValid — verify [pvBase, pvBase+cjSize) is mapped/readable.
/// Walks page-aligned probes calling MmIsAddressValid on each page start.
///
/// Symbol: `_KdbIsMemoryValid` (cdecl, no @N on i386; plain cdecl on amd64).
#[no_mangle]
pub unsafe extern "C" fn KdbIsMemoryValid(pvBase: PVOID, cjSize: ULONG) -> BOOLEAN {
    let mut pjAddress = align_down_pointer_by(pvBase as *const u8, PAGE_SIZE);
    let end = (pvBase as *mut u8).add(cjSize as usize);

    while pjAddress < end {
        if MmIsAddressValid(pjAddress as PVOID) == 0 {
            return 0; // FALSE
        }
        pjAddress = pjAddress.add(PAGE_SIZE as usize);
    }

    1 // TRUE
}

/// DbgGdiKdbgCliCallback — KDBG CLI command dispatcher for !gdi.* commands.
///
/// Symbol: `_DbgGdiKdbgCliCallback@12` on i386 (stdcall, 3 × 4-byte args);
///         `DbgGdiKdbgCliCallback` on amd64 (cdecl == NTAPI on 64-bit).
#[no_mangle]
pub unsafe extern "system" fn DbgGdiKdbgCliCallback(
    _pszCommand: PCHAR,
    argc: ULONG,
    argv: *mut PCHAR,
) -> BOOLEAN {
    let argv0: *const c_char = *argv as *const c_char;

    if _stricmp(argv0, b"!gdi.help\0".as_ptr() as *const c_char) == 0 {
        KdbCommand_Gdi_help();
    } else if _stricmp(argv0, b"!gdi.dumpht\0".as_ptr() as *const c_char) == 0 {
        KdbCommand_Gdi_dumpht(argc - 1, argv.add(1));
    } else if _stricmp(argv0, b"!gdi.handle\0".as_ptr() as *const c_char) == 0 {
        KdbCommand_Gdi_handle(*argv.add(1));
    } else if _stricmp(argv0, b"!gdi.entry\0".as_ptr() as *const c_char) == 0 {
        KdbCommand_Gdi_entry(*argv.add(1));
    } else if _stricmp(argv0, b"!gdi.baseobject\0".as_ptr() as *const c_char) == 0 {
        KdbCommand_Gdi_baseobject(*argv.add(1));
    } else {
        // Not handled by this callback.
        return 0; // FALSE
    }

    1 // TRUE
}
