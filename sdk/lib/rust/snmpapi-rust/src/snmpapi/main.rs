use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut startTime: ULONGLONG = 0;
unsafe extern "C" fn asn_any_copy(dst: *mut AsnAny, src: *const AsnAny) -> INT {
    memset(dst as *mut core::ffi::c_void, 0, core::mem::size_of::<AsnAny>());
    // Scrutinee widened to the constants' type; patterns stay NAMES so an unresolved one
    // cannot silently become a catch-all binding (field manual §9.112).
    match (*src).asnType as u32 {
        ASN_INTEGER32 => { (*dst).asnValue.number = (*src).asnValue.number; }
        ASN_UNSIGNED32 => { (*dst).asnValue.unsigned32 = (*src).asnValue.unsigned32; }
        ASN_COUNTER64 => { (*dst).asnValue.counter64 = (*src).asnValue.counter64; }
        ASN_COUNTER32 => { (*dst).asnValue.counter = (*src).asnValue.counter; }
        ASN_GAUGE32 => { (*dst).asnValue.gauge = (*src).asnValue.gauge; }
        ASN_TIMETICKS => { (*dst).asnValue.ticks = (*src).asnValue.ticks; }

        ASN_OCTETSTRING | ASN_BITS | ASN_SEQUENCE | ASN_IPADDRESS | ASN_OPAQUE => {
            let length: UINT = (*src).asnValue.string.length;
            let stream = HeapAlloc(GetProcessHeap(), 0, length as SIZE_T) as *mut BYTE;
            if stream.is_null() {
                return (SNMPAPI_ERROR) as i32;
            }
            memcpy(stream as *mut core::ffi::c_void, (*src).asnValue.string.stream as *const core::ffi::c_void, length as usize);

            (*dst).asnValue.string.stream = stream;
            (*dst).asnValue.string.length = length;
            (*dst).asnValue.string.dynamic = TRUE as INT;
        }
        ASN_OBJECTIDENTIFIER => {
            let size: UINT = (*src).asnValue.object.idLength * core::mem::size_of::<UINT>() as UINT;
            let ids = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut UINT;
            if ids.is_null() {
                return (SNMPAPI_ERROR) as i32;
            }

            (*dst).asnValue.object.ids = ids;
            (*dst).asnValue.object.idLength = (*src).asnValue.object.idLength;

            let mut i: UINT = 0;
            while i < (*dst).asnValue.object.idLength {
                *(*dst).asnValue.object.ids.offset(i as isize) = *(*src).asnValue.object.ids.offset(i as isize);
                i += 1;
            }
        }
        _ => {
            return (SNMPAPI_ERROR) as i32;
        }
    }
    (*dst).asnType = (*src).asnType;
    (SNMPAPI_NOERROR) as i32
}
unsafe extern "C" fn asn_any_free(any: *mut AsnAny) {
    match (*any).asnType as u32 {
        ASN_OCTETSTRING | ASN_BITS | ASN_SEQUENCE | ASN_IPADDRESS | ASN_OPAQUE => {
            if (*any).asnValue.string.dynamic != 0 {
                HeapFree(GetProcessHeap(), 0, (*any).asnValue.string.stream as LPVOID);
                (*any).asnValue.string.stream = core::ptr::null_mut();
            }
        }
        ASN_OBJECTIDENTIFIER => {
            HeapFree(GetProcessHeap(), 0, (*any).asnValue.object.ids as LPVOID);
            (*any).asnValue.object.ids = core::ptr::null_mut();
        }
        _ => {}
    }
    (*any).asnType = (ASN_NULL) as u8;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hInstDLL: HINSTANCE,
    fdwReason: u32,
    lpvReserved: LPVOID,
) -> i32 {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstDLL);
            startTime = GetTickCount64();
        }
        _ => {}
    }

    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SnmpSvcGetUptime() -> DWORD {
    let now: ULONGLONG = GetTickCount64();

    ((now - startTime) / 10) as DWORD
}

#[no_mangle]
// Declared `VOID WINAPIV SnmpUtilDbgPrint(INT, LPSTR, ...)` in C, and its whole body is a
// FIXME that compiles out — it never reads a vararg. *Defining* a C-variadic needs nightly
// (`c_variadic`), so this drops the `...` instead of pulling in the varargs-rust trampoline
// for arguments nobody consumes. ABI-identical: WINAPIV is cdecl, the caller cleans up, and
// surplus arguments are simply ignored on all three arches.
pub unsafe extern "C" fn SnmpUtilDbgPrint(loglevel: INT, format: LPSTR) {
    let _ = (loglevel, format);
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilMemAlloc(nbytes: UINT) -> LPVOID {
    HeapAlloc(GetProcessHeap(), 0, nbytes as SIZE_T)
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilMemReAlloc(mem: LPVOID, nbytes: u32) -> LPVOID {
    HeapReAlloc(GetProcessHeap(), 0, mem, nbytes as SIZE_T)
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilMemFree(mem: LPVOID) {
    HeapFree(GetProcessHeap(), 0, mem);
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilAsnAnyCpy(dst: *mut AsnAny, src: *mut AsnAny) -> INT {
    // TRACE("(%p, %p)\n", dst, src);
    asn_any_copy(dst, src)
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilAsnAnyFree(any: *mut AsnAny) {
    // TRACE("(%p)\n", any);
    asn_any_free(any);
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOctetsCpy(dst: *mut AsnOctetString, src: *mut AsnOctetString) -> INT {
    if dst.is_null() {
        return (SNMPAPI_ERROR) as i32;
    }
    if src.is_null() {
        (*dst).dynamic = (FALSE) as i32;
        (*dst).length = 0;
        (*dst).stream = core::ptr::null_mut();
        return (SNMPAPI_NOERROR) as i32;
    }
    (*dst).stream = HeapAlloc(GetProcessHeap(), 0, (*src).length as SIZE_T) as *mut BYTE;
    if !(*dst).stream.is_null() {
        (*dst).dynamic = (TRUE) as i32;
        (*dst).length = (*src).length;
        let mut i = 0;
        while i < (*dst).length {
            *(*dst).stream.add(i as usize) = *(*src).stream.add(i as usize);
            i += 1;
        }
        return (SNMPAPI_NOERROR) as i32;
    }
    (SNMPAPI_ERROR) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOctetsFree(octets: *mut AsnOctetString) {
    if !octets.is_null() {
        (*octets).length = 0;
        if (*octets).dynamic != 0 {
            HeapFree(GetProcessHeap(), 0, (*octets).stream as LPVOID);
        }
        (*octets).stream = core::ptr::null_mut();
        (*octets).dynamic = (FALSE) as i32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOctetsNCmp(
    octets1: *mut AsnOctetString,
    octets2: *mut AsnOctetString,
    count: UINT,
) -> INT {
    let mut ret: INT;
    let mut i: u32;

    if octets1.is_null() || octets2.is_null() {
        return 0;
    }

    i = 0;
    while i < count {
        ret = (*octets1).stream.offset(i as isize).read() as INT
            - (*octets2).stream.offset(i as isize).read() as INT;
        if ret != 0 {
            return ret;
        }
        i += 1;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOctetsCmp(
    octets1: *mut AsnOctetString,
    octets2: *mut AsnOctetString,
) -> INT {
    if (*octets1).length < (*octets2).length {
        return -1;
    }
    if (*octets1).length > (*octets2).length {
        return 1;
    }

    return SnmpUtilOctetsNCmp(octets1, octets2, (*octets1).length);
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOidAppend(dst: *mut AsnObjectIdentifier, src: *mut AsnObjectIdentifier) -> INT {
    let mut ids: *mut UINT;
    let i: UINT;
    let size: UINT;

    if dst.is_null() {
        return (SNMPAPI_ERROR) as i32;
    }
    if src.is_null() {
        return (SNMPAPI_NOERROR) as i32;
    }

    size = ((*src).idLength + (*dst).idLength) * core::mem::size_of::<UINT>() as u32;
    ids = HeapReAlloc(GetProcessHeap(), 0, (*dst).ids as LPVOID, size as SIZE_T) as *mut UINT;
    if ids.is_null() {
        ids = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut UINT;
        if ids.is_null() {
            SetLastError(SNMP_MEM_ALLOC_ERROR);
            return (SNMPAPI_ERROR) as i32;
        } else {
            memcpy(ids as *mut core::ffi::c_void, (*dst).ids as *const core::ffi::c_void, ((*dst).idLength * core::mem::size_of::<UINT>() as u32) as usize);
        }
    }

    let mut i = 0;
    while i < (*src).idLength {
        *ids.add((i + (*dst).idLength) as usize) = *(*src).ids.add(i as usize);
        i += 1;
    }
    (*dst).idLength = (*dst).idLength + (*src).idLength;
    (*dst).ids = ids;

    (SNMPAPI_NOERROR) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOidCpy(dst: *mut AsnObjectIdentifier, src: *mut AsnObjectIdentifier) -> INT {
    if dst.is_null() {
        return (SNMPAPI_ERROR) as i32;
    }
    if src.is_null() {
        (*dst).idLength = 0;
        (*dst).ids = core::ptr::null_mut();
        return (SNMPAPI_NOERROR) as i32;
    }
    let allocated = HeapAlloc(GetProcessHeap(), 0,
        ((*src).idLength * core::mem::size_of::<UINT>() as UINT) as SIZE_T);
    (*dst).ids = allocated as *mut UINT;
    if !(*dst).ids.is_null() {
        (*dst).idLength = (*src).idLength;
        let mut i = 0;
        while i < (*dst).idLength {
            *(*dst).ids.add(i as usize) = *(*src).ids.add(i as usize);
            i += 1;
        }
        return (SNMPAPI_NOERROR) as i32;
    }
    return (SNMPAPI_ERROR) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOidFree(oid: *mut AsnObjectIdentifier) {
    if oid.is_null() {
        return;
    }

    (*oid).idLength = 0;
    HeapFree(GetProcessHeap(), 0, (*oid).ids as LPVOID);
    (*oid).ids = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOidNCmp(
    oid1: *mut AsnObjectIdentifier,
    oid2: *mut AsnObjectIdentifier,
    count: UINT,
) -> INT {
    let mut i: u32;
    let mut len: u32;

    if oid1.is_null() || oid2.is_null() {
        return 0;
    }

    len = min(count, (*oid1).idLength);
    len = min(len, (*oid2).idLength);
    i = 0;
    while i < len {
        if *(*oid1).ids.offset(i as isize) > *(*oid2).ids.offset(i as isize) {
            return 1;
        }
        if *(*oid1).ids.offset(i as isize) < *(*oid2).ids.offset(i as isize) {
            return -1;
        }
        i += 1;
    }
    if i == count {
        return 0;
    }
    if (*oid1).idLength < (*oid2).idLength {
        return -1;
    }
    if (*oid1).idLength > (*oid2).idLength {
        return 1;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOidCmp(
    oid1: *mut AsnObjectIdentifier,
    oid2: *mut AsnObjectIdentifier,
) -> INT {
    if (*oid1).idLength < (*oid2).idLength {
        return -1;
    }
    if (*oid1).idLength > (*oid2).idLength {
        return 1;
    }

    return SnmpUtilOidNCmp(oid1, oid2, (*oid1).idLength);
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilVarBindCpy(dst: *mut SnmpVarBind, src: *mut SnmpVarBind) -> INT {
    let mut i: u32;
    let size: u32;

    if dst.is_null() {
        return (SNMPAPI_ERROR) as i32;
    }
    if src.is_null() {
        (*dst).value.asnType = (ASN_NULL) as u8;
        return (SNMPAPI_NOERROR) as i32;
    }

    size = (*src).name.idLength * core::mem::size_of::<UINT>() as u32;
    (*dst).name.ids = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut UINT;
    if (*dst).name.ids.is_null() {
        return (SNMPAPI_ERROR) as i32;
    }

    i = 0;
    while i < (*src).name.idLength {
        *(*dst).name.ids.add(i as usize) = *(*src).name.ids.add(i as usize);
        i += 1;
    }
    (*dst).name.idLength = (*src).name.idLength;

    if asn_any_copy(core::ptr::addr_of_mut!((*dst).value), core::ptr::addr_of!((*src).value)) == 0 {
        HeapFree(GetProcessHeap(), 0, (*dst).name.ids as LPVOID);
        return (SNMPAPI_ERROR) as i32;
    }
    return (SNMPAPI_NOERROR) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilVarBindFree(vb: *mut SnmpVarBind) {
    if vb.is_null() {
        return;
    }

    asn_any_free(core::ptr::addr_of_mut!((*vb).value));
    HeapFree(GetProcessHeap(), 0, (*vb).name.ids as LPVOID);
    (*vb).name.idLength = 0;
    (*vb).name.ids = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilVarBindListCpy(dst: *mut SnmpVarBindList, src: *mut SnmpVarBindList) -> INT {
    let mut i: u32;
    let size: u32;
    let mut src_entry: *mut SnmpVarBind;
    let mut dst_entry: *mut SnmpVarBind;

    if src.is_null() {
        (*dst).list = core::ptr::null_mut();
        (*dst).len = 0;
        return (SNMPAPI_NOERROR) as i32;
    }
    size = (*src).len * core::mem::size_of::<SnmpVarBind>() as u32;
    (*dst).list = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut SnmpVarBind;
    if (*dst).list.is_null() {
        return (SNMPAPI_ERROR) as i32;
    }

    src_entry = (*src).list;
    dst_entry = (*dst).list;
    i = 0;
    while i < (*src).len {
        if SnmpUtilVarBindCpy(dst_entry, src_entry) != 0 {
            src_entry = src_entry.offset(1);
            dst_entry = dst_entry.offset(1);
        } else {
            i = i.wrapping_sub(1);
            while i > 0 {
                dst_entry = dst_entry.offset(-1);
                SnmpUtilVarBindFree(dst_entry);
                i = i.wrapping_sub(1);
            }
            HeapFree(GetProcessHeap(), 0, (*dst).list as LPVOID);
            return (SNMPAPI_ERROR) as i32;
        }
        i = i.wrapping_add(1);
    }
    (*dst).len = (*src).len;
    return (SNMPAPI_NOERROR) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilVarBindListFree(vb: *mut SnmpVarBindList) {
    let mut i: u32;
    let mut entry: *mut SnmpVarBind;

    entry = (*vb).list;
    i = 0;
    while i < (*vb).len {
        SnmpUtilVarBindFree(entry);
        entry = entry.offset(1);
        i += 1;
    }
    HeapFree(GetProcessHeap(), 0, (*vb).list as LPVOID);
    (*vb).list = core::ptr::null_mut();
    (*vb).len = 0;
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilIdsToA(ids: *mut UINT, length: UINT) -> LPSTR {
    static mut one: [i8; 10] = [0; 10];
    static mut oid: [i8; 514] = [0; 514];
    static mut null_oid: [i8; 11] = [b'<' as i8, b'n' as i8, b'u' as i8, b'l' as i8, b'l' as i8, b' ' as i8, b'o' as i8, b'i' as i8, b'd' as i8, b'>' as i8, 0];
    let mut i: u32;
    let mut len: u32;
    let mut left: u32 = 514 - 1;

    if ids.is_null() || length == 0 {
        return null_oid.as_mut_ptr();
    }

    oid[0] = 0;
    i = 0;
    while i < length {
        sprintf(one.as_mut_ptr(), b"%d\0".as_ptr() as *const i8, *ids.add(i as usize));
        // C's `UINT len = strlen(one)` truncates silently on 64-bit; `one` holds at most a
        // formatted 32-bit integer, so the narrowing is unreachable in practice.
        len = strlen(one.as_ptr()) as UINT;
        if left >= len {
            strcat(oid.as_mut_ptr(), one.as_ptr());
            left -= len;
        } else {
            return oid.as_mut_ptr();
        }

        if i < length - 1 {
            if left > 0 {
                strcat(oid.as_mut_ptr(), b".\0".as_ptr() as *const i8);
                left -= 1;
            } else {
                return oid.as_mut_ptr();
            }
        }
        i += 1;
    }
    oid.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilOidToA(oid: *mut AsnObjectIdentifier) -> LPSTR {
    static mut null_oid: [u8; 11] = *b"<null oid>\0";

    if !oid.is_null() {
        SnmpUtilIdsToA((*oid).ids, (*oid).idLength)
    } else {
        null_oid.as_mut_ptr() as LPSTR
    }
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilPrintOid(oid: *mut AsnObjectIdentifier) {
    let mut i: u32;

    if oid.is_null() {
        return;
    }

    i = 0;
    while i < (*oid).idLength {
        if i < (*oid).idLength - 1 {
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SnmpUtilPrintAsnAny(any: *mut AsnAny) {
    let mut i: u32;

    match (*any).asnType as u32 {
        ASN_NULL => {
            TRACE!("Null value\n");
            return;
        }
        ASN_INTEGER32 => {
            TRACE!("Integer32 %d\n", (*any).asnValue.number);
            return;
        }
        ASN_UNSIGNED32 => {
            TRACE!("Unsigned32 %u\n", (*any).asnValue.unsigned32);
            return;
        }
        ASN_COUNTER32 => {
            TRACE!("Counter32 %u\n", (*any).asnValue.counter);
            return;
        }
        ASN_GAUGE32 => {
            TRACE!("Gauge32 %u\n", (*any).asnValue.gauge);
            return;
        }
        ASN_TIMETICKS => {
            TRACE!("Timeticks %u\n", (*any).asnValue.ticks);
            return;
        }
        ASN_COUNTER64 => {
            return;
        }
        ASN_OCTETSTRING => {
            i = 0;
            while i < (*any).asnValue.string.length {
                TRACE!("%c", *(*any).asnValue.string.stream.offset(i as isize));
                i += 1;
            }
            return;
        }
        ASN_IPADDRESS => {
            if (*any).asnValue.string.length < 4 {
                return;
            }
            i = 0;
            while i < 4 {
                if i < 3 {
                    TRACE!(".");
                }
                i += 1;
            }
            return;
        }
        ASN_BITS => {
            i = 0;
            while i < (*any).asnValue.string.length {
                if i < (*any).asnValue.object.idLength - 1 {
                    TRACE!(" ");
                }
                i += 1;
            }
            return;
        }
        ASN_OPAQUE => {
            i = 0;
            while i < (*any).asnValue.string.length {
                if i < (*any).asnValue.object.idLength - 1 {
                    TRACE!(" ");
                }
                i += 1;
            }
            return;
        }
        ASN_OBJECTIDENTIFIER => {
            i = 0;
            while i < (*any).asnValue.object.idLength {
                if i < (*any).asnValue.object.idLength - 1 {
                    TRACE!(".");
                }
                i += 1;
            }
            return;
        }
        _ => {
            return;
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Implementation of SNMPAPI.DLL
//  *
//  * Copyright 2002 Patrik Stridvall
//  * Copyright 2007 Hans Leidekker
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <stdio.h>
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "snmp.h"
//
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(snmpapi);
//
// static INT asn_any_copy(AsnAny *dst, const AsnAny *src)
// {
//     memset(dst, 0, sizeof(AsnAny));
//     switch (src->asnType)
//     {
//     case ASN_INTEGER32:  dst->asnValue.number = src->asnValue.number; break;
//     case ASN_UNSIGNED32: dst->asnValue.unsigned32 = src->asnValue.unsigned32; break;
//     case ASN_COUNTER64:  dst->asnValue.counter64 = src->asnValue.counter64; break;
//     case ASN_COUNTER32:  dst->asnValue.counter = src->asnValue.counter; break;
//     case ASN_GAUGE32:    dst->asnValue.gauge = src->asnValue.gauge; break;
//     case ASN_TIMETICKS:  dst->asnValue.ticks = src->asnValue.ticks; break;
//
//     case ASN_OCTETSTRING:
//     case ASN_BITS:
//     case ASN_SEQUENCE:
//     case ASN_IPADDRESS:
//     case ASN_OPAQUE:
//     {
//         BYTE *stream;
//         UINT length = src->asnValue.string.length;
//
//         if (!(stream = HeapAlloc(GetProcessHeap(), 0, length))) return SNMPAPI_ERROR;
//         memcpy(stream, src->asnValue.string.stream, length);
//
//         dst->asnValue.string.stream = stream;
//         dst->asnValue.string.length = length;
//         dst->asnValue.string.dynamic = TRUE;
//         break;
//     }
//     case ASN_OBJECTIDENTIFIER:
//     {
//         UINT *ids, i, size = src->asnValue.object.idLength * sizeof(UINT);
//
//         if (!(ids = HeapAlloc(GetProcessHeap(), 0, size))) return SNMPAPI_ERROR;
//
//         dst->asnValue.object.ids = ids;
//         dst->asnValue.object.idLength = src->asnValue.object.idLength;
//
//         for (i = 0; i < dst->asnValue.object.idLength; i++)
//             dst->asnValue.object.ids[i] = src->asnValue.object.ids[i];
//         break;
//     }
//     default:
//     {
//         WARN("unknown ASN type: %d\n", src->asnType);
//         return SNMPAPI_ERROR;
//     }
//     }
//     dst->asnType = src->asnType;
//     return SNMPAPI_NOERROR;
// }
//
// static void asn_any_free(AsnAny *any)
// {
//     switch (any->asnType)
//     {
//     case ASN_OCTETSTRING:
//     case ASN_BITS:
//     case ASN_SEQUENCE:
//     case ASN_IPADDRESS:
//     case ASN_OPAQUE:
//     {
//         if (any->asnValue.string.dynamic)
//         {
//             HeapFree(GetProcessHeap(), 0, any->asnValue.string.stream);
//             any->asnValue.string.stream = NULL;
//         }
//         break;
//     }
//     case ASN_OBJECTIDENTIFIER:
//     {
//         HeapFree(GetProcessHeap(), 0, any->asnValue.object.ids);
//         any->asnValue.object.ids = NULL;
//         break;
//     }
//     default: break;
//     }
//     any->asnType = ASN_NULL;
// }
//
// static ULONGLONG startTime;
//
// /***********************************************************************
//  *		DllMain for SNMPAPI
//  */
// BOOL WINAPI DllMain(
// 	HINSTANCE hInstDLL,
// 	DWORD fdwReason,
// 	LPVOID lpvReserved)
// {
//     TRACE("(%p,%d,%p)\n", hInstDLL, fdwReason, lpvReserved);
//
//     switch(fdwReason) {
// #ifndef __REACTOS__
//     case DLL_WINE_PREATTACH:
//         return FALSE;  /* prefer native version */
// #endif
//     case DLL_PROCESS_ATTACH:
//         DisableThreadLibraryCalls(hInstDLL);
//         startTime = GetTickCount64();
//         break;
//     }
//
//     return TRUE;
// }
//
// /***********************************************************************
//  *      SnmpSvcGetUptime        (SNMPAPI.@)
//  *
//  * BUGS
//  *  This returns the number of centiseconds since the DLL was loaded,
//  *  rather than the number of centiseconds since the SNMP service was
//  *  started, since there isn't yet any SNMP service in Wine.
//  */
// DWORD WINAPI SnmpSvcGetUptime(void)
// {
//     ULONGLONG now = GetTickCount64();
//
//     return (now - startTime) / 10;
// }
//
// /***********************************************************************
//  *      SnmpUtilDbgPrint        (SNMPAPI.@)
//  *
//  * NOTES
//  *  The Microsoft headers claim this function uses the stdcall calling
//  *  convention. But stdcall functions cannot take a variable number of
//  *  arguments so this does not make sense. The stdcall specification is
//  *  probably ignored by Microsoft's compiler in this case. So declare it
//  *  correctly in Wine so it works with all compilers.
//  */
// VOID __cdecl SnmpUtilDbgPrint(INT loglevel, LPSTR format, ...)
// {
//     FIXME("(%d, %s)\n", loglevel, debugstr_a(format));
// }
//
// /***********************************************************************
//  *      SnmpUtilMemAlloc        (SNMPAPI.@)
//  */
// LPVOID WINAPI SnmpUtilMemAlloc(UINT nbytes)
// {
//     TRACE("(%d)\n", nbytes);
//     return HeapAlloc(GetProcessHeap(), 0, nbytes);
// }
//
// /***********************************************************************
//  *      SnmpUtilMemReAlloc      (SNMPAPI.@)
//  */
// LPVOID WINAPI SnmpUtilMemReAlloc(LPVOID mem, UINT nbytes)
// {
//     TRACE("(%p, %d)\n", mem, nbytes);
//     return HeapReAlloc(GetProcessHeap(), 0, mem, nbytes);
// }
//
// /***********************************************************************
//  *      SnmpUtilMemFree         (SNMPAPI.@)
//  */
// VOID WINAPI SnmpUtilMemFree(LPVOID mem)
// {
//     TRACE("(%p)\n", mem);
//     HeapFree(GetProcessHeap(), 0, mem);
// }
//
// /***********************************************************************
//  *      SnmpUtilAsnAnyCpy       (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilAsnAnyCpy(AsnAny *dst, AsnAny *src)
// {
//     TRACE("(%p, %p)\n", dst, src);
//     return asn_any_copy(dst, src);
// }
//
// /***********************************************************************
//  *      SnmpUtilAsnAnyFree      (SNMPAPI.@)
//  */
// VOID WINAPI SnmpUtilAsnAnyFree(AsnAny *any)
// {
//     TRACE("(%p)\n", any);
//     asn_any_free(any);
// }
//
// /***********************************************************************
//  *      SnmpUtilOctetsCpy       (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilOctetsCpy(AsnOctetString *dst, AsnOctetString *src)
// {
//     TRACE("(%p, %p)\n", dst, src);
//
//     if (!dst) return SNMPAPI_ERROR;
//     if (!src)
//     {
//         dst->dynamic = FALSE;
//         dst->length = 0;
//         dst->stream = NULL;
//         return SNMPAPI_NOERROR;
//     }
//     if ((dst->stream = HeapAlloc(GetProcessHeap(), 0, src->length)))
//     {
//         unsigned int i;
//
//         dst->dynamic = TRUE;
//         dst->length = src->length;
//         for (i = 0; i < dst->length; i++) dst->stream[i] = src->stream[i];
//         return SNMPAPI_NOERROR;
//     }
//     return SNMPAPI_ERROR;
// }
//
// /***********************************************************************
//  *      SnmpUtilOctetsFree      (SNMPAPI.@)
//  */
// VOID WINAPI SnmpUtilOctetsFree(AsnOctetString *octets)
// {
//     TRACE("(%p)\n", octets);
//
//     if (octets)
//     {
//         octets->length = 0;
//         if (octets->dynamic) HeapFree(GetProcessHeap(), 0, octets->stream);
//         octets->stream = NULL;
//         octets->dynamic = FALSE;
//     }
// }
//
// /***********************************************************************
//  *      SnmpUtilOctetsNCmp      (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilOctetsNCmp(AsnOctetString *octets1, AsnOctetString *octets2, UINT count)
// {
//     INT ret;
//     unsigned int i;
//
//     TRACE("(%p, %p, %d)\n", octets1, octets2, count);
//
//     if (!octets1 || !octets2) return 0;
//
//     for (i = 0; i < count; i++)
//         if ((ret = octets1->stream[i] - octets2->stream[i])) return ret;
//
//     return 0;
// }
//
// /***********************************************************************
//  *      SnmpUtilOctetsCmp       (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilOctetsCmp(AsnOctetString *octets1, AsnOctetString *octets2)
// {
//     TRACE("(%p, %p)\n", octets1, octets2);
//
//     if (octets1->length < octets2->length) return -1;
//     if (octets1->length > octets2->length) return 1;
//
//     return SnmpUtilOctetsNCmp(octets1, octets2, octets1->length);
// }
//
// /***********************************************************************
//  *      SnmpUtilOidAppend       (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilOidAppend(AsnObjectIdentifier *dst, AsnObjectIdentifier *src)
// {
//     UINT *ids, i, size;
//
//     TRACE("(%p, %p)\n", dst, src);
//
//     if (!dst) return SNMPAPI_ERROR;
//     if (!src) return SNMPAPI_NOERROR;
//
//     size = (src->idLength + dst->idLength) * sizeof(UINT);
//     if (!(ids = HeapReAlloc(GetProcessHeap(), 0, dst->ids, size)))
//     {
//         if (!(ids = HeapAlloc(GetProcessHeap(), 0, size)))
//         {
//             SetLastError(SNMP_MEM_ALLOC_ERROR);
//             return SNMPAPI_ERROR;
//         }
//         else memcpy(ids, dst->ids, dst->idLength * sizeof(UINT));
//     }
//
//     for (i = 0; i < src->idLength; i++) ids[i + dst->idLength] = src->ids[i];
//     dst->idLength = dst->idLength + src->idLength;
//     dst->ids = ids;
//
//     return SNMPAPI_NOERROR;
// }
//
// /***********************************************************************
//  *      SnmpUtilOidCpy          (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilOidCpy(AsnObjectIdentifier *dst, AsnObjectIdentifier *src)
// {
//     TRACE("(%p, %p)\n", dst, src);
//
//     if (!dst) return SNMPAPI_ERROR;
//     if (!src)
//     {
//         dst->idLength = 0;
//         dst->ids = NULL;
//         return SNMPAPI_NOERROR;
//     }
//     if ((dst->ids = HeapAlloc(GetProcessHeap(), 0, src->idLength * sizeof(UINT))))
//     {
//         unsigned int i;
//
//         dst->idLength = src->idLength;
//         for (i = 0; i < dst->idLength; i++) dst->ids[i] = src->ids[i];
//         return SNMPAPI_NOERROR;
//     }
//     return SNMPAPI_ERROR;
// }
//
// /***********************************************************************
//  *      SnmpUtilOidFree         (SNMPAPI.@)
//  */
// VOID WINAPI SnmpUtilOidFree(AsnObjectIdentifier *oid)
// {
//     TRACE("(%p)\n", oid);
//
//     if (!oid) return;
//
//     oid->idLength = 0;
//     HeapFree(GetProcessHeap(), 0, oid->ids);
//     oid->ids = NULL;
// }
//
// /***********************************************************************
//  *      SnmpUtilOidNCmp         (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilOidNCmp(AsnObjectIdentifier *oid1, AsnObjectIdentifier *oid2, UINT count)
// {
//     unsigned int i, len;
//
//     TRACE("(%p, %p, %d)\n", oid1, oid2, count);
//
//     if (!oid1 || !oid2) return 0;
//
//     len = min(count, oid1->idLength);
//     len = min(len, oid2->idLength);
//     for (i = 0; i < len; i++)
//     {
//         if (oid1->ids[i] > oid2->ids[i]) return 1;
//         if (oid1->ids[i] < oid2->ids[i]) return -1;
//     }
//     if (i == count) return 0;
//     if (oid1->idLength < oid2->idLength) return -1;
//     if (oid1->idLength > oid2->idLength) return 1;
//     return 0;
// }
//
// /***********************************************************************
//  *      SnmpUtilOidCmp          (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilOidCmp(AsnObjectIdentifier *oid1, AsnObjectIdentifier *oid2)
// {
//     TRACE("(%p, %p)\n", oid1, oid2);
//
//     if (oid1->idLength < oid2->idLength) return -1;
//     if (oid1->idLength > oid2->idLength) return 1;
//
//     return SnmpUtilOidNCmp(oid1, oid2, oid1->idLength);
// }
//
// /***********************************************************************
//  *      SnmpUtilVarBindCpy      (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilVarBindCpy(SnmpVarBind *dst, SnmpVarBind *src)
// {
//     unsigned int i, size;
//
//     TRACE("(%p, %p)\n", dst, src);
//
//     if (!dst) return SNMPAPI_ERROR;
//     if (!src)
//     {
//         dst->value.asnType = ASN_NULL;
//         return SNMPAPI_NOERROR;
//     }
//
//     size = src->name.idLength * sizeof(UINT);
//     if (!(dst->name.ids = HeapAlloc(GetProcessHeap(), 0, size))) return SNMPAPI_ERROR;
//
//     for (i = 0; i < src->name.idLength; i++) dst->name.ids[i] = src->name.ids[i];
//     dst->name.idLength = src->name.idLength;
//
//     if (!asn_any_copy(&dst->value, &src->value))
//     {
//         HeapFree(GetProcessHeap(), 0, dst->name.ids);
//         return SNMPAPI_ERROR;
//     }
//     return SNMPAPI_NOERROR;
// }
//
// /***********************************************************************
//  *      SnmpUtilVarBindFree     (SNMPAPI.@)
//  */
// VOID WINAPI SnmpUtilVarBindFree(SnmpVarBind *vb)
// {
//     TRACE("(%p)\n", vb);
//
//     if (!vb) return;
//
//     asn_any_free(&vb->value);
//     HeapFree(GetProcessHeap(), 0, vb->name.ids);
//     vb->name.idLength = 0;
//     vb->name.ids = NULL;
// }
//
// /***********************************************************************
//  *      SnmpUtilVarBindListCpy  (SNMPAPI.@)
//  */
// INT WINAPI SnmpUtilVarBindListCpy(SnmpVarBindList *dst, SnmpVarBindList *src)
// {
//     unsigned int i, size;
//     SnmpVarBind *src_entry, *dst_entry;
//
//     TRACE("(%p, %p)\n", dst, src);
//
//     if (!src)
//     {
//         dst->list = NULL;
//         dst->len = 0;
//         return SNMPAPI_NOERROR;
//     }
//     size = src->len * sizeof(SnmpVarBind);
//     if (!(dst->list = HeapAlloc(GetProcessHeap(), 0, size)))
//         return SNMPAPI_ERROR;
//
//     src_entry = src->list;
//     dst_entry = dst->list;
//     for (i = 0; i < src->len; i++)
//     {
//         if (SnmpUtilVarBindCpy(dst_entry, src_entry))
//         {
//             src_entry++;
//             dst_entry++;
//         }
//         else
//         {
//             for (--i; i > 0; i--) SnmpUtilVarBindFree(--dst_entry);
//             HeapFree(GetProcessHeap(), 0, dst->list);
//             return SNMPAPI_ERROR;
//         }
//     }
//     dst->len = src->len;
//     return SNMPAPI_NOERROR;
// }
//
// /***********************************************************************
//  *      SnmpUtilVarBindListFree (SNMPAPI.@)
//  */
// VOID WINAPI SnmpUtilVarBindListFree(SnmpVarBindList *vb)
// {
//     unsigned int i;
//     SnmpVarBind *entry;
//
//     TRACE("(%p)\n", vb);
//
//     entry = vb->list;
//     for (i = 0; i < vb->len; i++) SnmpUtilVarBindFree(entry++);
//     HeapFree(GetProcessHeap(), 0, vb->list);
//     vb->list = NULL;
//     vb->len = 0;
// }
//
// /***********************************************************************
//  *      SnmpUtilIdsToA          (SNMPAPI.@)
//  */
// LPSTR WINAPI SnmpUtilIdsToA(UINT *ids, UINT length)
// {
//     static char one[10], oid[514], null_oid[] = "<null oid>";
//     unsigned int i, len, left = sizeof(oid) - 1;
//
//     TRACE("(%p, %d)\n", ids, length);
//
//     if (!ids || !length) return null_oid;
//
//     *oid = 0;
//     for (i = 0; i < length; i++)
//     {
//         sprintf(one, "%d", ids[i]);
//         len = strlen(one);
//         if (left >= len)
//         {
//             strcat(oid, one);
//             left -= len;
//         }
//         else return oid;
//
//         if (i < length - 1)
//         {
//             if (left > 0)
//             {
//                 strcat(oid, ".");
//                 left--;
//             }
//             else return oid;
//         }
//     }
//     return oid;
// }
//
// /***********************************************************************
//  *      SnmpUtilOidToA          (SNMPAPI.@)
//  */
// LPSTR WINAPI SnmpUtilOidToA(AsnObjectIdentifier *oid)
// {
//     static char null_oid[] = "<null oid>";
//
//     TRACE("(%p)\n", oid);
//
//     if (oid)
//         return SnmpUtilIdsToA(oid->ids, oid->idLength);
//     else
//         return null_oid;
// }
//
// /***********************************************************************
//  *      SnmpUtilPrintOid        (SNMPAPI.@)
//  */
// VOID WINAPI SnmpUtilPrintOid(AsnObjectIdentifier *oid)
// {
//     unsigned int i;
//
//     TRACE("(%p)\n", oid);
//
//     if (!oid) return;
//
//     for (i = 0; i < oid->idLength; i++)
//     {
//         TRACE("%u", oid->ids[i]);
//         if (i < oid->idLength - 1) TRACE(".");
//     }
//     TRACE("\n");
// }
//
// /***********************************************************************
//  *      SnmpUtilPrintAsnAny     (SNMPAPI.@)
//  */
// VOID WINAPI SnmpUtilPrintAsnAny(AsnAny *any)
// {
//     unsigned int i;
//
//     TRACE("(%p)\n", any);
//
//     switch (any->asnType)
//     {
//     case ASN_NULL:       TRACE("Null value\n"); return;
//     case ASN_INTEGER32:  TRACE("Integer32 %d\n", any->asnValue.number); return;
//     case ASN_UNSIGNED32: TRACE("Unsigned32 %u\n", any->asnValue.unsigned32); return;
//     case ASN_COUNTER32:  TRACE("Counter32 %u\n", any->asnValue.counter); return;
//     case ASN_GAUGE32:    TRACE("Gauge32 %u\n", any->asnValue.gauge); return;
//     case ASN_TIMETICKS:  TRACE("Timeticks %u\n", any->asnValue.ticks); return;
//     case ASN_COUNTER64:
//     {
//         TRACE("Counter64 %x%08x\n", (DWORD)(any->asnValue.counter64.QuadPart>>32),(DWORD)any->asnValue.counter64.QuadPart);
//         return;
//     }
//     case ASN_OCTETSTRING:
//     {
//         TRACE("String ");
//         for (i = 0; i < any->asnValue.string.length; i++)
//             TRACE("%c", any->asnValue.string.stream[i]);
//         TRACE("\n");
//         return;
//     }
//     case ASN_IPADDRESS:
//     {
//         TRACE("IpAddress ");
//         if (any->asnValue.string.length < 4)
//         {
//             TRACE("Invalid\n");
//             return;
//         }
//         for (i = 0; i < 4; i++)
//         {
//             TRACE("%u", any->asnValue.string.stream[i]);
//             if (i < 3) TRACE(".");
//         }
//         TRACE("\n");
//         return;
//     }
//     case ASN_BITS:
//     {
//         TRACE("Bits ");
//         for (i = 0; i < any->asnValue.string.length; i++)
//         {
//             TRACE("0x%02x", any->asnValue.string.stream[i]);
//             if (i < any->asnValue.object.idLength - 1) TRACE(" ");
//         }
//         TRACE("\n");
//         return;
//     }
//     case ASN_OPAQUE:
//     {
//         TRACE("Opaque ");
//         for (i = 0; i < any->asnValue.string.length; i++)
//         {
//             TRACE("0x%02x", any->asnValue.string.stream[i]);
//             if (i < any->asnValue.object.idLength - 1) TRACE(" ");
//         }
//         TRACE("\n");
//         return;
//     }
//     case ASN_OBJECTIDENTIFIER:
//     {
//         TRACE("ObjectID ");
//         for (i = 0; i < any->asnValue.object.idLength; i++)
//         {
//             TRACE("%u", any->asnValue.object.ids[i]);
//             if (i < any->asnValue.object.idLength - 1) TRACE(".");
//         }
//         TRACE("\n");
//         return;
//     }
//     default:
//     {
//         TRACE("Invalid type %d\n", any->asnType);
//         return;
//     }
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
