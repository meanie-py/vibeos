use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

// ---- dns_type_to_str ----
#[no_mangle]
pub unsafe extern "C" fn dns_type_to_str(type_: u16) -> *const u8 {
    match type_ as u32 {
        DNS_TYPE_ZERO => b"DNS_TYPE_ZERO\0".as_ptr() as *const u8,
        DNS_TYPE_A => b"DNS_TYPE_A\0".as_ptr() as *const u8,
        DNS_TYPE_NS => b"DNS_TYPE_NS\0".as_ptr() as *const u8,
        DNS_TYPE_MD => b"DNS_TYPE_MD\0".as_ptr() as *const u8,
        DNS_TYPE_MF => b"DNS_TYPE_MF\0".as_ptr() as *const u8,
        DNS_TYPE_CNAME => b"DNS_TYPE_CNAME\0".as_ptr() as *const u8,
        DNS_TYPE_SOA => b"DNS_TYPE_SOA\0".as_ptr() as *const u8,
        DNS_TYPE_MB => b"DNS_TYPE_MB\0".as_ptr() as *const u8,
        DNS_TYPE_MG => b"DNS_TYPE_MG\0".as_ptr() as *const u8,
        DNS_TYPE_MR => b"DNS_TYPE_MR\0".as_ptr() as *const u8,
        DNS_TYPE_NULL => b"DNS_TYPE_NULL\0".as_ptr() as *const u8,
        DNS_TYPE_WKS => b"DNS_TYPE_WKS\0".as_ptr() as *const u8,
        DNS_TYPE_PTR => b"DNS_TYPE_PTR\0".as_ptr() as *const u8,
        DNS_TYPE_HINFO => b"DNS_TYPE_HINFO\0".as_ptr() as *const u8,
        DNS_TYPE_MINFO => b"DNS_TYPE_MINFO\0".as_ptr() as *const u8,
        DNS_TYPE_MX => b"DNS_TYPE_MX\0".as_ptr() as *const u8,
        DNS_TYPE_TEXT => b"DNS_TYPE_TEXT\0".as_ptr() as *const u8,
        DNS_TYPE_RP => b"DNS_TYPE_RP\0".as_ptr() as *const u8,
        DNS_TYPE_AFSDB => b"DNS_TYPE_AFSDB\0".as_ptr() as *const u8,
        DNS_TYPE_X25 => b"DNS_TYPE_X25\0".as_ptr() as *const u8,
        DNS_TYPE_ISDN => b"DNS_TYPE_ISDN\0".as_ptr() as *const u8,
        DNS_TYPE_RT => b"DNS_TYPE_RT\0".as_ptr() as *const u8,
        DNS_TYPE_NSAP => b"DNS_TYPE_NSAP\0".as_ptr() as *const u8,
        DNS_TYPE_NSAPPTR => b"DNS_TYPE_NSAPPTR\0".as_ptr() as *const u8,
        DNS_TYPE_SIG => b"DNS_TYPE_SIG\0".as_ptr() as *const u8,
        DNS_TYPE_KEY => b"DNS_TYPE_KEY\0".as_ptr() as *const u8,
        DNS_TYPE_PX => b"DNS_TYPE_PX\0".as_ptr() as *const u8,
        DNS_TYPE_GPOS => b"DNS_TYPE_GPOS\0".as_ptr() as *const u8,
        DNS_TYPE_AAAA => b"DNS_TYPE_AAAA\0".as_ptr() as *const u8,
        DNS_TYPE_LOC => b"DNS_TYPE_LOC\0".as_ptr() as *const u8,
        DNS_TYPE_NXT => b"DNS_TYPE_NXT\0".as_ptr() as *const u8,
        DNS_TYPE_EID => b"DNS_TYPE_EID\0".as_ptr() as *const u8,
        DNS_TYPE_NIMLOC => b"DNS_TYPE_NIMLOC\0".as_ptr() as *const u8,
        DNS_TYPE_SRV => b"DNS_TYPE_SRV\0".as_ptr() as *const u8,
        DNS_TYPE_ATMA => b"DNS_TYPE_ATMA\0".as_ptr() as *const u8,
        DNS_TYPE_NAPTR => b"DNS_TYPE_NAPTR\0".as_ptr() as *const u8,
        DNS_TYPE_KX => b"DNS_TYPE_KX\0".as_ptr() as *const u8,
        DNS_TYPE_CERT => b"DNS_TYPE_CERT\0".as_ptr() as *const u8,
        DNS_TYPE_A6 => b"DNS_TYPE_A6\0".as_ptr() as *const u8,
        DNS_TYPE_DNAME => b"DNS_TYPE_DNAME\0".as_ptr() as *const u8,
        DNS_TYPE_SINK => b"DNS_TYPE_SINK\0".as_ptr() as *const u8,
        DNS_TYPE_OPT => b"DNS_TYPE_OPT\0".as_ptr() as *const u8,
        DNS_TYPE_UINFO => b"DNS_TYPE_UINFO\0".as_ptr() as *const u8,
        DNS_TYPE_UID => b"DNS_TYPE_UID\0".as_ptr() as *const u8,
        DNS_TYPE_GID => b"DNS_TYPE_GID\0".as_ptr() as *const u8,
        DNS_TYPE_UNSPEC => b"DNS_TYPE_UNSPEC\0".as_ptr() as *const u8,
        DNS_TYPE_ADDRS => b"DNS_TYPE_ADDRS\0".as_ptr() as *const u8,
        DNS_TYPE_TKEY => b"DNS_TYPE_TKEY\0".as_ptr() as *const u8,
        DNS_TYPE_TSIG => b"DNS_TYPE_TSIG\0".as_ptr() as *const u8,
        DNS_TYPE_IXFR => b"DNS_TYPE_IXFR\0".as_ptr() as *const u8,
        DNS_TYPE_AXFR => b"DNS_TYPE_AXFR\0".as_ptr() as *const u8,
        DNS_TYPE_MAILB => b"DNS_TYPE_MAILB\0".as_ptr() as *const u8,
        DNS_TYPE_MAILA => b"DNS_TYPE_MAILA\0".as_ptr() as *const u8,
        DNS_TYPE_ANY => b"DNS_TYPE_ANY\0".as_ptr() as *const u8,
        DNS_TYPE_WINS => b"DNS_TYPE_WINS\0".as_ptr() as *const u8,
        DNS_TYPE_WINSR => b"DNS_TYPE_WINSR\0".as_ptr() as *const u8,
        _ => {
            static mut TMP: [i8; 7] = [0i8; 7];
            extern "C" {
                fn sprintf(s: *mut i8, fmt: *const i8, ...) -> i32;
            }
            sprintf(
                TMP.as_mut_ptr(),
                b"0x%04x\0".as_ptr() as *const i8,
                type_ as core::ffi::c_int,
            );
            TMP.as_ptr() as *const u8
        }
    }
}

// ---- dns_strcmpX ----
unsafe fn dns_strcmpX(str1: LPCVOID, str2: LPCVOID, wide: BOOL) -> i32 {
    if wide != 0 {
        lstrcmpiW(str1 as *const u16, str2 as *const u16)
    } else {
        lstrcmpiA(str1 as *const i8, str2 as *const i8)
    }
}

// ---- dns_strcpyX ----
unsafe fn dns_strcpyX(src: LPCVOID, in_charset: DNS_CHARSET, out_charset: DNS_CHARSET) -> LPVOID {
    match in_charset {
        DnsCharSetUnicode => {
            match out_charset {
                DnsCharSetUnicode => dns_strdup_w(src as LPCWSTR) as LPVOID,
                DnsCharSetUtf8 => dns_strdup_wu(src as LPCWSTR) as LPVOID,
                DnsCharSetAnsi => dns_strdup_wa(src as LPCWSTR) as LPVOID,
                _ => {
                    DbgPrint(b"unhandled target charset: %d\n\0".as_ptr() as *const i8, out_charset);
                    core::ptr::null_mut()
                }
            }
        }
        DnsCharSetUtf8 => {
            match out_charset {
                DnsCharSetUnicode => dns_strdup_uw(src as LPCSTR) as LPVOID,
                DnsCharSetUtf8 => dns_strdup_u(src as LPCSTR) as LPVOID,
                DnsCharSetAnsi => dns_strdup_ua(src as LPCSTR) as LPVOID,
                _ => {
                    DbgPrint(b"unhandled target charset: %d\n\0".as_ptr() as *const i8, out_charset);
                    core::ptr::null_mut()
                }
            }
        }
        DnsCharSetAnsi => {
            match out_charset {
                DnsCharSetUnicode => dns_strdup_aw(src as LPCSTR) as LPVOID,
                DnsCharSetUtf8 => dns_strdup_au(src as LPCSTR) as LPVOID,
                DnsCharSetAnsi => dns_strdup_a(src as LPCSTR) as LPVOID,
                _ => {
                    DbgPrint(b"unhandled target charset: %d\n\0".as_ptr() as *const i8, out_charset);
                    core::ptr::null_mut()
                }
            }
        }
        _ => {
            DbgPrint(b"unhandled source charset: %d\n\0".as_ptr() as *const i8, in_charset);
            core::ptr::null_mut()
        }
    }
}

// ---- DnsRecordListFree ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordListFree(list: PDNS_RECORD, free_type: DNS_FREE_TYPE) {
    if list.is_null() {
        return;
    }

    match free_type {
        DnsFreeRecordList => {
            let mut r = list;
            while !r.is_null() {
                HeapFree(GetProcessHeap(), 0, (*r).pName as LPVOID);

                match (*r).wType as u32 {
                    DNS_TYPE_HINFO | DNS_TYPE_ISDN | DNS_TYPE_TEXT | DNS_TYPE_X25 => {
                        let count = (*r).Data.TXT.as_ref().dwStringCount as usize;
                        for i in 0..count {
                            HeapFree(
                                GetProcessHeap(),
                                0,
                                *(*r).Data.TXT.as_ref().pStringArray.as_ptr().add(i) as LPVOID,
                            );
                        }
                    }
                    DNS_TYPE_MINFO | DNS_TYPE_RP => {
                        HeapFree(GetProcessHeap(), 0, (*r).Data.MINFO.as_ref().pNameMailbox as LPVOID);
                        HeapFree(
                            GetProcessHeap(),
                            0,
                            (*r).Data.MINFO.as_ref().pNameErrorsMailbox as LPVOID,
                        );
                    }
                    DNS_TYPE_AFSDB | DNS_TYPE_RT | DNS_TYPE_MX => {
                        HeapFree(GetProcessHeap(), 0, (*r).Data.MX.as_ref().pNameExchange as LPVOID);
                    }
                    DNS_TYPE_NXT => {
                        HeapFree(GetProcessHeap(), 0, (*r).Data.NXT.as_ref().pNameNext as LPVOID);
                    }
                    DNS_TYPE_CNAME
                    | DNS_TYPE_MB
                    | DNS_TYPE_MD
                    | DNS_TYPE_MF
                    | DNS_TYPE_MG
                    | DNS_TYPE_MR
                    | DNS_TYPE_NS
                    | DNS_TYPE_PTR => {
                        HeapFree(GetProcessHeap(), 0, (*r).Data.PTR.as_ref().pNameHost as LPVOID);
                    }
                    DNS_TYPE_SIG => {
                        HeapFree(GetProcessHeap(), 0, (*r).Data.SIG.as_ref().pNameSigner as LPVOID);
                    }
                    DNS_TYPE_SOA => {
                        HeapFree(
                            GetProcessHeap(),
                            0,
                            (*r).Data.SOA.as_ref().pNamePrimaryServer as LPVOID,
                        );
                        HeapFree(
                            GetProcessHeap(),
                            0,
                            (*r).Data.SOA.as_ref().pNameAdministrator as LPVOID,
                        );
                    }
                    DNS_TYPE_SRV => {
                        HeapFree(GetProcessHeap(), 0, (*r).Data.SRV.as_ref().pNameTarget as LPVOID);
                    }
                    _ => {}
                }

                let next = (*r).pNext;
                HeapFree(GetProcessHeap(), 0, r as LPVOID);
                r = next;
            }
        }
        DnsFreeFlat | DnsFreeParsedMessageFields => {
            // DPRINT1("unhandled free type: %d\n", type); // debug prints omitted
        }
        _ => {
            // DPRINT1("unknown free type: %d\n", type); // debug prints omitted
        }
    }
}

// ---- DnsRecordSetDetach ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordSetDetach(set: PDNS_RECORD) -> PDNS_RECORD {
    let mut r: PDNS_RECORD = set;
    let mut set_temp: PDNS_RECORD;
    loop {
        set_temp = r;
        if set_temp.is_null() {
            break;
        }
        if !(*r).pNext.is_null() && (*(*r).pNext).pNext.is_null() {
            let s = (*r).pNext;
            (*r).pNext = core::ptr::null_mut();
            return s;
        }
        r = (*set_temp).pNext;
    }
    core::ptr::null_mut()
}

// ---- DnsRecordCompare ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordCompare(r1: PDNS_RECORD, r2: PDNS_RECORD) -> i32 {
    // DPRINT("(%p,%p)\n", r1, r2);

    if (*r1).wType != (*r2).wType
        || (*r1).wDataLength != (*r2).wDataLength
        || (*r1).Flags.S.as_ref().Section() != (*r2).Flags.S.as_ref().Section()
        || (*r1).Flags.S.as_ref().Delete() != (*r2).Flags.S.as_ref().Delete()
        || (*r1).Flags.S.as_ref().Unused() != (*r2).Flags.S.as_ref().Unused()
        || (*r1).Flags.S.as_ref().Reserved() != (*r2).Flags.S.as_ref().Reserved()
        || (*r1).dwReserved != (*r2).dwReserved
    {
        return 0;
    }

    let wide = ((*r1).Flags.S.as_ref().CharSet() == DnsCharSetUnicode || (*r1).Flags.S.as_ref().CharSet() == DnsCharSetUnknown) as i32;

    if dns_strcmpX((*r1).pName as LPCVOID, (*r2).pName as LPCVOID, wide) != 0 {
        return 0;
    }

    match (*r1).wType as u32 {
        DNS_TYPE_A => {
            if (*r1).Data.A.as_ref().IpAddress != (*r2).Data.A.as_ref().IpAddress {
                return 0;
            }
        }
        DNS_TYPE_SOA => {
            if (*r1).Data.SOA.as_ref().dwSerialNo != (*r2).Data.SOA.as_ref().dwSerialNo
                || (*r1).Data.SOA.as_ref().dwRefresh != (*r2).Data.SOA.as_ref().dwRefresh
                || (*r1).Data.SOA.as_ref().dwRetry != (*r2).Data.SOA.as_ref().dwRetry
                || (*r1).Data.SOA.as_ref().dwExpire != (*r2).Data.SOA.as_ref().dwExpire
                || (*r1).Data.SOA.as_ref().dwDefaultTtl != (*r2).Data.SOA.as_ref().dwDefaultTtl
            {
                return 0;
            }
            if dns_strcmpX(
                (*r1).Data.SOA.as_ref().pNamePrimaryServer as LPCVOID,
                (*r2).Data.SOA.as_ref().pNamePrimaryServer as LPCVOID,
                wide,
            ) != 0
                || dns_strcmpX(
                    (*r1).Data.SOA.as_ref().pNameAdministrator as LPCVOID,
                    (*r2).Data.SOA.as_ref().pNameAdministrator as LPCVOID,
                    wide,
                ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_PTR | DNS_TYPE_NS | DNS_TYPE_CNAME | DNS_TYPE_MB | DNS_TYPE_MD | DNS_TYPE_MF | DNS_TYPE_MG | DNS_TYPE_MR => {
            if dns_strcmpX(
                (*r1).Data.PTR.as_ref().pNameHost as LPCVOID,
                (*r2).Data.PTR.as_ref().pNameHost as LPCVOID,
                wide,
            ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_MINFO | DNS_TYPE_RP => {
            if dns_strcmpX(
                (*r1).Data.MINFO.as_ref().pNameMailbox as LPCVOID,
                (*r2).Data.MINFO.as_ref().pNameMailbox as LPCVOID,
                wide,
            ) != 0
                || dns_strcmpX(
                    (*r1).Data.MINFO.as_ref().pNameErrorsMailbox as LPCVOID,
                    (*r2).Data.MINFO.as_ref().pNameErrorsMailbox as LPCVOID,
                    wide,
                ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_MX | DNS_TYPE_AFSDB | DNS_TYPE_RT => {
            if (*r1).Data.MX.as_ref().wPreference != (*r2).Data.MX.as_ref().wPreference {
                return 0;
            }
            if dns_strcmpX(
                (*r1).Data.MX.as_ref().pNameExchange as LPCVOID,
                (*r2).Data.MX.as_ref().pNameExchange as LPCVOID,
                wide,
            ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_HINFO | DNS_TYPE_ISDN | DNS_TYPE_TEXT | DNS_TYPE_X25 => {
            if (*r1).Data.TXT.as_ref().dwStringCount != (*r2).Data.TXT.as_ref().dwStringCount {
                return 0;
            }
            let mut i: u32 = 0;
            while i < (*r1).Data.TXT.as_ref().dwStringCount {
                if dns_strcmpX(
                    *(*r1).Data.TXT.as_ref().pStringArray.as_ptr().add(i as usize) as LPCVOID,
                    *(*r2).Data.TXT.as_ref().pStringArray.as_ptr().add(i as usize) as LPCVOID,
                    wide,
                ) != 0
                {
                    return 0;
                }
                i += 1;
            }
        }
        DNS_TYPE_NULL => {
            if (*r1).Data.Null.as_ref().dwByteCount != (*r2).Data.Null.as_ref().dwByteCount {
                return 0;
            }
            if memcmp(
                (*r1).Data.Null.as_ref().Data.as_ptr() as *const core::ffi::c_void,
                (*r2).Data.Null.as_ref().Data.as_ptr() as *const core::ffi::c_void,
                (*r1).Data.Null.as_ref().dwByteCount as usize,
            ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_AAAA => {
            let mut i: u32 = 0;
            while i < 4u32 {
                if (*r1).Data.AAAA.as_ref().Ip6Address.IP6Dword[i as usize]
                    != (*r2).Data.AAAA.as_ref().Ip6Address.IP6Dword[i as usize]
                {
                    return 0;
                }
                i += 1;
            }
        }
        DNS_TYPE_KEY => {
            if (*r1).Data.KEY.as_ref().wFlags != (*r2).Data.KEY.as_ref().wFlags
                || (*r1).Data.KEY.as_ref().chProtocol != (*r2).Data.KEY.as_ref().chProtocol
                || (*r1).Data.KEY.as_ref().chAlgorithm != (*r2).Data.KEY.as_ref().chAlgorithm
            {
                return 0;
            }
            if memcmp(
                (*r1).Data.KEY.as_ref().Key.as_ptr() as *const core::ffi::c_void,
                (*r2).Data.KEY.as_ref().Key.as_ptr() as *const core::ffi::c_void,
                ((*r1).wDataLength as u32 - core::mem::size_of::<DNS_KEY_DATA>() as u32 + 1) as usize,
            ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_SIG => {
            if dns_strcmpX(
                (*r1).Data.SIG.as_ref().pNameSigner as LPCVOID,
                (*r2).Data.SIG.as_ref().pNameSigner as LPCVOID,
                wide,
            ) != 0
            {
                return 0;
            }
            if (*r1).Data.SIG.as_ref().wTypeCovered != (*r2).Data.SIG.as_ref().wTypeCovered
                || (*r1).Data.SIG.as_ref().chAlgorithm != (*r2).Data.SIG.as_ref().chAlgorithm
                || (*r1).Data.SIG.as_ref().chLabelCount != (*r2).Data.SIG.as_ref().chLabelCount
                || (*r1).Data.SIG.as_ref().dwOriginalTtl != (*r2).Data.SIG.as_ref().dwOriginalTtl
                || (*r1).Data.SIG.as_ref().dwExpiration != (*r2).Data.SIG.as_ref().dwExpiration
                || (*r1).Data.SIG.as_ref().dwTimeSigned != (*r2).Data.SIG.as_ref().dwTimeSigned
                || (*r1).Data.SIG.as_ref().wKeyTag != (*r2).Data.SIG.as_ref().wKeyTag
            {
                return 0;
            }
            if memcmp(
                (*r1).Data.SIG.as_ref().Signature.as_ptr() as *const core::ffi::c_void,
                (*r2).Data.SIG.as_ref().Signature.as_ptr() as *const core::ffi::c_void,
                ((*r1).wDataLength as u32 - core::mem::size_of::<DNS_SIG_DATAA>() as u32 + 1) as usize,
            ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_ATMA => {
            if (*r1).Data.ATMA.as_ref().AddressType != (*r2).Data.ATMA.as_ref().AddressType {
                return 0;
            }
            let mut i: u32 = 0;
            while i < DNS_ATMA_MAX_ADDR_LENGTH {
                if (*r1).Data.ATMA.as_ref().Address[i as usize]
                    != (*r2).Data.ATMA.as_ref().Address[i as usize]
                {
                    return 0;
                }
                i += 1;
            }
        }
        DNS_TYPE_NXT => {
            if dns_strcmpX(
                (*r1).Data.NXT.as_ref().pNameNext as LPCVOID,
                (*r2).Data.NXT.as_ref().pNameNext as LPCVOID,
                wide,
            ) != 0
            {
                return 0;
            }
            if (*r1).Data.NXT.as_ref().wNumTypes != (*r2).Data.NXT.as_ref().wNumTypes {
                return 0;
            }
            if memcmp(
                (*r1).Data.NXT.as_ref().wTypes.as_ptr() as *const core::ffi::c_void,
                (*r2).Data.NXT.as_ref().wTypes.as_ptr() as *const core::ffi::c_void,
                ((*r1).wDataLength as u32 - core::mem::size_of::<DNS_NXT_DATAA>() as u32 + core::mem::size_of::<WORD>() as u32) as usize,
            ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_SRV => {
            if dns_strcmpX(
                (*r1).Data.SRV.as_ref().pNameTarget as LPCVOID,
                (*r2).Data.SRV.as_ref().pNameTarget as LPCVOID,
                wide,
            ) != 0
            {
                return 0;
            }
            if (*r1).Data.SRV.as_ref().wPriority != (*r2).Data.SRV.as_ref().wPriority
                || (*r1).Data.SRV.as_ref().wWeight != (*r2).Data.SRV.as_ref().wWeight
                || (*r1).Data.SRV.as_ref().wPort != (*r2).Data.SRV.as_ref().wPort
            {
                return 0;
            }
        }
        DNS_TYPE_TKEY => {
            if dns_strcmpX(
                (*r1).Data.TKEY.as_ref().pNameAlgorithm as LPCVOID,
                (*r2).Data.TKEY.as_ref().pNameAlgorithm as LPCVOID,
                wide,
            ) != 0
            {
                return 0;
            }
            if (*r1).Data.TKEY.as_ref().dwCreateTime != (*r2).Data.TKEY.as_ref().dwCreateTime
                || (*r1).Data.TKEY.as_ref().dwExpireTime != (*r2).Data.TKEY.as_ref().dwExpireTime
                || (*r1).Data.TKEY.as_ref().wMode != (*r2).Data.TKEY.as_ref().wMode
                || (*r1).Data.TKEY.as_ref().wError != (*r2).Data.TKEY.as_ref().wError
                || (*r1).Data.TKEY.as_ref().wKeyLength != (*r2).Data.TKEY.as_ref().wKeyLength
                || (*r1).Data.TKEY.as_ref().wOtherLength != (*r2).Data.TKEY.as_ref().wOtherLength
                || (*r1).Data.TKEY.as_ref().cAlgNameLength != (*r2).Data.TKEY.as_ref().cAlgNameLength
                || (*r1).Data.TKEY.as_ref().bPacketPointers != (*r2).Data.TKEY.as_ref().bPacketPointers
            {
                return 0;
            }
            if memcmp(
                (*r1).Data.TKEY.as_ref().pKey as *const core::ffi::c_void,
                (*r2).Data.TKEY.as_ref().pKey as *const core::ffi::c_void,
                (*r1).Data.TKEY.as_ref().wKeyLength as usize,
            ) != 0
                || memcmp(
                    (*r1).Data.TKEY.as_ref().pOtherData as *const core::ffi::c_void,
                    (*r2).Data.TKEY.as_ref().pOtherData as *const core::ffi::c_void,
                    (*r1).Data.TKEY.as_ref().wOtherLength as usize,
                ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_TSIG => {
            if dns_strcmpX(
                (*r1).Data.TSIG.as_ref().pNameAlgorithm as LPCVOID,
                (*r2).Data.TSIG.as_ref().pNameAlgorithm as LPCVOID,
                wide,
            ) != 0
            {
                return 0;
            }
            if (*r1).Data.TSIG.as_ref().i64CreateTime != (*r2).Data.TSIG.as_ref().i64CreateTime
                || (*r1).Data.TSIG.as_ref().wFudgeTime != (*r2).Data.TSIG.as_ref().wFudgeTime
                || (*r1).Data.TSIG.as_ref().wOriginalXid != (*r2).Data.TSIG.as_ref().wOriginalXid
                || (*r1).Data.TSIG.as_ref().wError != (*r2).Data.TSIG.as_ref().wError
                || (*r1).Data.TSIG.as_ref().wSigLength != (*r2).Data.TSIG.as_ref().wSigLength
                || (*r1).Data.TSIG.as_ref().wOtherLength != (*r2).Data.TSIG.as_ref().wOtherLength
                || (*r1).Data.TSIG.as_ref().cAlgNameLength != (*r2).Data.TSIG.as_ref().cAlgNameLength
                || (*r1).Data.TSIG.as_ref().bPacketPointers != (*r2).Data.TSIG.as_ref().bPacketPointers
            {
                return 0;
            }
            if memcmp(
                (*r1).Data.TSIG.as_ref().pSignature as *const core::ffi::c_void,
                (*r2).Data.TSIG.as_ref().pSignature as *const core::ffi::c_void,
                (*r1).Data.TSIG.as_ref().wSigLength as usize,
            ) != 0
                || memcmp(
                    (*r1).Data.TSIG.as_ref().pOtherData as *const core::ffi::c_void,
                    (*r2).Data.TSIG.as_ref().pOtherData as *const core::ffi::c_void,
                    (*r1).Data.TSIG.as_ref().wOtherLength as usize,
                ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_WINS => {
            if (*r1).Data.WINS.as_ref().dwMappingFlag != (*r2).Data.WINS.as_ref().dwMappingFlag
                || (*r1).Data.WINS.as_ref().dwLookupTimeout != (*r2).Data.WINS.as_ref().dwLookupTimeout
                || (*r1).Data.WINS.as_ref().dwCacheTimeout != (*r2).Data.WINS.as_ref().dwCacheTimeout
                || (*r1).Data.WINS.as_ref().cWinsServerCount != (*r2).Data.WINS.as_ref().cWinsServerCount
            {
                return 0;
            }
            if memcmp(
                (*r1).Data.WINS.as_ref().WinsServers.as_ptr() as *const core::ffi::c_void,
                (*r2).Data.WINS.as_ref().WinsServers.as_ptr() as *const core::ffi::c_void,
                ((*r1).wDataLength as u32 - core::mem::size_of::<DNS_WINS_DATA>() as u32 + core::mem::size_of::<IP4_ADDRESS>() as u32) as usize,
            ) != 0
            {
                return 0;
            }
        }
        DNS_TYPE_WINSR => {
            if (*r1).Data.WINSR.as_ref().dwMappingFlag != (*r2).Data.WINSR.as_ref().dwMappingFlag
                || (*r1).Data.WINSR.as_ref().dwLookupTimeout != (*r2).Data.WINSR.as_ref().dwLookupTimeout
                || (*r1).Data.WINSR.as_ref().dwCacheTimeout != (*r2).Data.WINSR.as_ref().dwCacheTimeout
            {
                return 0;
            }
            if dns_strcmpX(
                (*r1).Data.WINSR.as_ref().pNameResultDomain as LPCVOID,
                (*r2).Data.WINSR.as_ref().pNameResultDomain as LPCVOID,
                wide,
            ) != 0
            {
                return 0;
            }
        }
        _ => {
            // DPRINT1("unknown type: %s\n", dns_type_to_str((*r1).wType));
            return 0;
        }
    }
    1
}

// ---- DnsRecordCopyEx ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordCopyEx(
    src: *mut DNS_RECORD,
    mut in_charset: DNS_CHARSET,
    out_charset: DNS_CHARSET,
) -> *mut DNS_RECORD {
    let size = core::mem::offset_of!(DNS_RECORD, Data) + (*src).wDataLength as usize;
    let dst = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size) as *mut DNS_RECORD;
    if dst.is_null() {
        return core::ptr::null_mut();
    }

    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);

    let cs = (*src).Flags.S.as_ref().CharSet();
    if cs == DnsCharSetUtf8 || cs == DnsCharSetAnsi || cs == DnsCharSetUnicode {
        in_charset = cs;
    }

    (*dst).Flags.S.as_mut().set_CharSet(out_charset);
    (*dst).pName = dns_strcpyX((*src).pName as LPCVOID, in_charset, out_charset) as LPWSTR;
    if (*dst).pName.is_null() {
        HeapFree(GetProcessHeap(), 0, dst as LPVOID);
        return core::ptr::null_mut();
    }

    let ok = match (*src).wType as u32 {
        DNS_TYPE_HINFO | DNS_TYPE_ISDN | DNS_TYPE_TEXT | DNS_TYPE_X25 => {
            let count = (*src).Data.TXT.as_ref().dwStringCount;
            let mut i = 0u32;
            while i < count {
                let src_str = core::ptr::read((*src).Data.TXT.as_ref().pStringArray.as_ptr().add(i as usize));
                let new_str = dns_strcpyX(src_str as LPCVOID, in_charset, out_charset) as LPWSTR;
                if new_str.is_null() {
                    while i > 0 {
                        i -= 1;
                        let to_free = core::ptr::read((*dst).Data.TXT.as_ref().pStringArray.as_ptr().add(i as usize));
                        HeapFree(GetProcessHeap(), 0, to_free as LPVOID);
                    }
                    break;
                }
                core::ptr::write((*dst).Data.TXT.as_mut().pStringArray.as_mut_ptr().add(i as usize), new_str);
                i += 1;
            }
            i == count
        }
        DNS_TYPE_MINFO | DNS_TYPE_RP => {
            let mailbox = dns_strcpyX(
                (*src).Data.MINFO.as_ref().pNameMailbox as LPCVOID,
                in_charset,
                out_charset,
            ) as LPWSTR;
            if mailbox.is_null() {
                false
            } else {
                (*dst).Data.MINFO.as_mut().pNameMailbox = mailbox;
                let errors = dns_strcpyX(
                    (*src).Data.MINFO.as_ref().pNameErrorsMailbox as LPCVOID,
                    in_charset,
                    out_charset,
                ) as LPWSTR;
                if errors.is_null() {
                    HeapFree(GetProcessHeap(), 0, mailbox as LPVOID);
                    false
                } else {
                    (*dst).Data.MINFO.as_mut().pNameErrorsMailbox = errors;
                    true
                }
            }
        }
        DNS_TYPE_AFSDB | DNS_TYPE_RT | DNS_TYPE_MX => {
            let exchange = dns_strcpyX(
                (*src).Data.MX.as_ref().pNameExchange as LPCVOID,
                in_charset,
                out_charset,
            ) as LPWSTR;
            if exchange.is_null() {
                false
            } else {
                (*dst).Data.MX.as_mut().pNameExchange = exchange;
                true
            }
        }
        DNS_TYPE_NXT => {
            let next = dns_strcpyX(
                (*src).Data.NXT.as_ref().pNameNext as LPCVOID,
                in_charset,
                out_charset,
            ) as LPWSTR;
            if next.is_null() {
                false
            } else {
                (*dst).Data.NXT.as_mut().pNameNext = next;
                true
            }
        }
        DNS_TYPE_CNAME
        | DNS_TYPE_MB
        | DNS_TYPE_MD
        | DNS_TYPE_MF
        | DNS_TYPE_MG
        | DNS_TYPE_MR
        | DNS_TYPE_NS
        | DNS_TYPE_PTR => {
            let host = dns_strcpyX(
                (*src).Data.PTR.as_ref().pNameHost as LPCVOID,
                in_charset,
                out_charset,
            ) as LPWSTR;
            if host.is_null() {
                false
            } else {
                (*dst).Data.PTR.as_mut().pNameHost = host;
                true
            }
        }
        DNS_TYPE_SIG => {
            let signer = dns_strcpyX(
                (*src).Data.SIG.as_ref().pNameSigner as LPCVOID,
                in_charset,
                out_charset,
            ) as LPWSTR;
            if signer.is_null() {
                false
            } else {
                (*dst).Data.SIG.as_mut().pNameSigner = signer;
                true
            }
        }
        DNS_TYPE_SOA => {
            let primary = dns_strcpyX(
                (*src).Data.SOA.as_ref().pNamePrimaryServer as LPCVOID,
                in_charset,
                out_charset,
            ) as LPWSTR;
            if primary.is_null() {
                false
            } else {
                (*dst).Data.SOA.as_mut().pNamePrimaryServer = primary;
                let admin = dns_strcpyX(
                    (*src).Data.SOA.as_ref().pNameAdministrator as LPCVOID,
                    in_charset,
                    out_charset,
                ) as LPWSTR;
                if admin.is_null() {
                    HeapFree(GetProcessHeap(), 0, primary as LPVOID);
                    false
                } else {
                    (*dst).Data.SOA.as_mut().pNameAdministrator = admin;
                    true
                }
            }
        }
        DNS_TYPE_SRV => {
            let target = dns_strcpyX(
                (*src).Data.SRV.as_ref().pNameTarget as LPCVOID,
                in_charset,
                out_charset,
            ) as LPWSTR;
            if target.is_null() {
                false
            } else {
                (*dst).Data.SRV.as_mut().pNameTarget = target;
                true
            }
        }
        _ => true,
    };

    if !ok {
        HeapFree(GetProcessHeap(), 0, (*dst).pName as LPVOID);
        HeapFree(GetProcessHeap(), 0, dst as LPVOID);
        return core::ptr::null_mut();
    }

    dst
}

// ---- DnsRecordSetCopyEx ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordSetCopyEx(
    mut src_set: PDNS_RECORD,
    in_charset: DNS_CHARSET,
    out_charset: DNS_CHARSET,
) -> PDNS_RECORD {
    let mut dst_set: DNS_RRSET = core::mem::zeroed();
    DNS_RRSET_INIT(&mut dst_set);

    let mut src = src_set;
    loop {
        if src.is_null() {
            break;
        }
        src_set = src;
        let dst = DnsRecordCopyEx(src, in_charset, out_charset);
        if dst.is_null() {
            DNS_RRSET_TERMINATE(&mut dst_set);
            DnsRecordListFree(dst_set.pFirstRR, DnsFreeRecordList);
            return core::ptr::null_mut();
        }
        DNS_RRSET_ADD(&mut dst_set, dst);
        src = (*src).pNext;
    }

    DNS_RRSET_TERMINATE(&mut dst_set);
    dst_set.pFirstRR
}

// ---- DnsRecordSetCompare ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordSetCompare(
    set1: PDNS_RECORD,
    set2: PDNS_RECORD,
    diff1: *mut PDNS_RECORD,
    diff2: *mut PDNS_RECORD,
) -> BOOL {
    let mut ret: BOOL = 1;
    let mut rr1: DNS_RRSET = core::mem::zeroed();
    let mut rr2: DNS_RRSET = core::mem::zeroed();

    if set1.is_null() && set2.is_null() {
        return 0;
    }

    if !diff1.is_null() {
        *diff1 = core::ptr::null_mut();
    }
    if !diff2.is_null() {
        *diff2 = core::ptr::null_mut();
    }

    if !set1.is_null() && set2.is_null() {
        if !diff1.is_null() {
            *diff1 = DnsRecordSetCopyEx(set1, 0, (*set1).Flags.S.as_ref().CharSet());
        }
        return 0;
    }
    if set1.is_null() && !set2.is_null() {
        if !diff2.is_null() {
            *diff2 = DnsRecordSetCopyEx(set2, 0, (*set2).Flags.S.as_ref().CharSet());
        }
        return 0;
    }

    DNS_RRSET_INIT(&mut rr1);
    DNS_RRSET_INIT(&mut rr2);

    let mut error = false;
    'outer1: {
        let mut r = set1;
        while !r.is_null() {
            let mut t = set2;
            while !t.is_null() {
                let u = DnsRecordCopyEx(r, (*r).Flags.S.as_ref().CharSet(), (*t).Flags.S.as_ref().CharSet());
                if u.is_null() {
                    error = true;
                    break 'outer1;
                }
                if DnsRecordCompare(t, u) == 0 {
                    DNS_RRSET_ADD(&mut rr1, u);
                    ret = 0;
                } else {
                    DnsRecordListFree(u, DnsFreeRecordList);
                }
                t = (*t).pNext;
            }
            r = (*r).pNext;
        }
    }

    if !error {
        'outer2: {
            let mut t = set2;
            while !t.is_null() {
                let mut r = set1;
                while !r.is_null() {
                    let u = DnsRecordCopyEx(t, (*t).Flags.S.as_ref().CharSet(), (*r).Flags.S.as_ref().CharSet());
                    if u.is_null() {
                        error = true;
                        break 'outer2;
                    }
                    if DnsRecordCompare(r, u) == 0 {
                        DNS_RRSET_ADD(&mut rr2, u);
                        ret = 0;
                    } else {
                        DnsRecordListFree(u, DnsFreeRecordList);
                    }
                    r = (*r).pNext;
                }
                t = (*t).pNext;
            }
        }
    }

    if error {
        DNS_RRSET_TERMINATE(&mut rr1);
        DNS_RRSET_TERMINATE(&mut rr2);
        DnsRecordListFree(rr1.pFirstRR, DnsFreeRecordList);
        DnsRecordListFree(rr2.pFirstRR, DnsFreeRecordList);
        return 0;
    }

    DNS_RRSET_TERMINATE(&mut rr1);
    DNS_RRSET_TERMINATE(&mut rr2);

    if !diff1.is_null() {
        *diff1 = rr1.pFirstRR;
    } else {
        DnsRecordListFree(rr1.pFirstRR, DnsFreeRecordList);
    }

    if !diff2.is_null() {
        *diff2 = rr2.pFirstRR;
    } else {
        DnsRecordListFree(rr2.pFirstRR, DnsFreeRecordList);
    }

    ret
}
