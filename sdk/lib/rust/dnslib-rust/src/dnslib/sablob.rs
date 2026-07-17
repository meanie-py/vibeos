use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::dnslib::types::*;
use crate::dnslib::dnsaddr::{
    DnsAddrArray_Create, DnsAddrArray_Free, DnsAddrArray_AddIp4,
    DnsAddrArray_AddAddr, DnsAddr_BuildFromDnsRecord,
};
use crate::dnslib::string::UnicodeString;
use crate::dnslib::memory::{Dns_Free, Dns_AllocZero};
use crate::dnslib::flatbuf::FlatBuf_Arg_Reserve;
use crate::dnslib::hostent::{Hostent_Init, Hostent_ConvertToOffsets};
use crate::dnslib::straddr::Dns_ReverseNameToDnsAddr_W;

// Extern declarations for functions defined in other translation units (dnsapi, etc.)
unsafe extern "system" {
    fn Dns_CreateStringCopy_W(String: *mut u16) -> *mut u16;
    fn GetLastError() -> u32;
    fn Dns_GetBufferLengthForStringCopy(
        p1: *const c_void, p2: u32, p3: u32, p4: u32,
    ) -> u32;
    fn Dns_StringCopy(
        dst: *mut c_void, dst_size: *mut u32,
        src: *const c_void, src_size: u32,
        src_type: u32, dst_type: u32,
    ) -> u32;
    fn FamilyInfo_GetForFamily(family: u16) -> *mut DNS_FAMILY_INFO;
    fn SetLastError(dwErrCode: u32);
    fn DnsQuery_W(
        lpName: *mut u16, wType: u16, Options: u32,
        pExtra: *mut c_void, ppQueryResultsSet: *mut *mut DNS_RECORD,
        pReserved: *mut *mut c_void,
    ) -> i32;
    fn DnsApiFree(pData: *mut c_void);
    fn DnsNameCompare_W(pName1: *mut u16, pName2: *mut u16) -> i32;
    fn DnsQueryConfigAllocEx(
        dwConfigFlags: u32, pDnsQueryConfig: *mut c_void,
        pReserved: *mut c_void,
    ) -> *mut u16;
    fn DnsRecordListFree(pRecordList: *mut DNS_RECORD, FreeType: u32);
}

// Constants matching dnslib.h / windns.h
const ERROR_SUCCESS: i32 = 0;
const ERROR_OUTOFMEMORY: u32 = 14;
const STATUS_INVALID_PARAMETER: u32 = 0xC000000D;
const DNSREC_ANSWER: u32 = 1;
const DNS_TYPE_A: u16 = 0x0001;
const DNS_TYPE_AAAA: u16 = 0x001c;
const DNS_TYPE_ATMA: u16 = 0x0022;
const DNS_TYPE_CNAME: u16 = 0x0005;
const DNS_TYPE_PTR: u16 = 0x000c;
const IpV4Address: u32 = 3;
const DnsConfigFullHostName_W: u32 = 15;
const DnsFreeRecordList: u32 = 0;

const LOCALHOST: [u16; 10] = [
    0x006c, 0x006f, 0x0063, 0x0061, 0x006c, 0x0068, 0x006f, 0x0073, 0x0074, 0x0000,
];
const LOOPBACK: [u16; 9] = [
    0x006c, 0x006f, 0x006f, 0x0070, 0x0062, 0x0061, 0x0063, 0x006b, 0x0000,
];

// ---- FlatBuf_Arg_ReserveAlignPointer ----
#[no_mangle]
pub unsafe extern "system" fn FlatBuf_Arg_ReserveAlignPointer(
    Position: *mut usize,
    FreeSize: *mut usize,
    Size: usize,
) -> *mut c_void {
    FlatBuf_Arg_Reserve(
        Position,
        FreeSize,
        Size,
        core::mem::size_of::<*mut c_void>(),
    )
}

// ---- SaBlob_Create ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_Create(Count: u32) -> *mut DNS_BLOB {
    let blob = Dns_AllocZero(core::mem::size_of::<DNS_BLOB>()) as *mut DNS_BLOB;
    if !blob.is_null() {
        if Count != 0 {
            let dns_addr_array = DnsAddrArray_Create(Count);
            if dns_addr_array.is_null() {
                SaBlob_Free(blob);
                SetLastError(ERROR_OUTOFMEMORY);
            } else {
                (*blob).DnsAddrArray = dns_addr_array;
            }
        }
    }
    blob
}

// ---- SaBlob_CreateFromIp4 ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_CreateFromIp4(
    Name: *mut u16,
    Count: u32,
    AddressArray: *const IN_ADDR,
) -> *mut DNS_BLOB {
    let mut Blob: *mut DNS_BLOB = core::ptr::null_mut();
    loop {
        Blob = SaBlob_Create(Count);
        if Blob.is_null() {
            break;
        }
        if !Name.is_null() {
            let NameCopy = Dns_CreateStringCopy_W(Name);
            if NameCopy.is_null() {
                break;
            }
            (*Blob).Name = NameCopy;
        }
        for i in 0..Count {
            let addr = *AddressArray.add(i as usize);
            DnsAddrArray_AddIp4(
                (*Blob).DnsAddrArray as PDNS_ARRAY,
                addr,
                IpV4Address,
            );
        }
        return Blob;
    }
    SaBlob_Free(Blob);
    SetLastError(ERROR_OUTOFMEMORY);
    core::ptr::null_mut()
}

// ---- SaBlob_Free ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_Free(Blob: *mut DNS_BLOB) {
    if !Blob.is_null() {
        Dns_Free((*Blob).Name as *mut c_void);
        while (*Blob).AliasCount != 0 {
            let count = (*Blob).AliasCount;
            let alias = *(*Blob).Aliases.as_ptr().add(count as usize);
            Dns_Free(alias as *mut c_void);
            (*Blob).AliasCount -= 1;
        }
        DnsAddrArray_Free((*Blob).DnsAddrArray);
        Dns_Free(Blob as *mut c_void);
    }
}

// ---- SaBlob_CreateHostent ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_CreateHostent(
    BufferPosition: *mut usize,
    FreeBufferSpace: *mut usize,
    HostEntrySize: *mut usize,
    Blob: *mut DNS_BLOB,
    StringType: u32,
    Relative: u8,
    BufferAllocated: u8,
) -> *mut HOSTENT {
    let DnsAddrArray: *mut DNS_ARRAY = (*Blob).DnsAddrArray;
    let AliasCount: u32 = (*Blob).AliasCount;
    let mut AddressFamily: u16 = 0; // AF_UNSPEC = 0 (matches C: WORD AddressFamily = AF_UNSPEC)
    let mut AddressCount: u32 = 0;
    let mut AddressSize: u32 = 0;
    let TotalSize: u32;
    let NamePointerSize: u32;
    let AliasPointerSize: u32;
    let mut FamilyInfo: *mut DNS_FAMILY_INFO = core::ptr::null_mut();
    let mut StringLength: u32 = 0;
    let mut i: u32;
    let mut HostentSize: u32 = 0;
    let mut Hostent: *mut HOSTENT = core::ptr::null_mut();
    let mut HostentPtr: usize;
    let mut CurrentAddress: *mut c_void;

    if !DnsAddrArray.is_null() {
        AddressFamily = (*DnsAddrArray).Addresses[0].Ip4Address.sin_family;
        FamilyInfo = FamilyInfo_GetForFamily(AddressFamily);
        AddressCount = (*DnsAddrArray).UsedAddresses;
        AddressSize = (*FamilyInfo).AddressSize;
    }

    TotalSize = AddressSize.wrapping_mul(AddressCount);
    NamePointerSize = AddressCount
        .wrapping_mul(core::mem::size_of::<*mut c_void>() as u32)
        .wrapping_add(core::mem::size_of::<*mut c_void>() as u32);

    if !(*Blob).Name.is_null() {
        StringLength = (Dns_GetBufferLengthForStringCopy(
            (*Blob).Name as *const c_void,
            0,
            UnicodeString,
            StringType,
        )
        .wrapping_add(1)
            & !1) as u32;
    }

    i = AliasCount;
    while i != 0 {
        let alias_ptr = *((*Blob).Aliases.as_ptr() as *const *const c_void).add(i as usize);
        HostentSize = HostentSize.wrapping_add(
            (Dns_GetBufferLengthForStringCopy(
                alias_ptr,
                0,
                UnicodeString,
                StringType,
            )
            .wrapping_add(1)
                & !1) as u32,
        );
        i = i.wrapping_sub(1);
    }

    AliasPointerSize = AliasCount
        .wrapping_mul(core::mem::size_of::<*mut c_void>() as u32)
        .wrapping_add(core::mem::size_of::<*mut c_void>() as u32);

    HostentSize = HostentSize
        .wrapping_add(TotalSize)
        .wrapping_add(NamePointerSize)
        .wrapping_add(AliasPointerSize)
        .wrapping_add(StringLength)
        .wrapping_add(core::mem::size_of::<HOSTENT>() as u32);

    if BufferAllocated == 0 {
        HostentPtr = Dns_AllocZero(HostentSize as usize) as usize;
    } else {
        HostentPtr = FlatBuf_Arg_ReserveAlignPointer(
            BufferPosition,
            FreeBufferSpace,
            HostentSize as usize,
        ) as usize;
    }

    if HostentPtr != 0 {
        // C passes (PVOID)&HostentPtr — address of the local variable so Hostent_Init advances it.
        Hostent = Hostent_Init(
            &mut HostentPtr as *mut usize as *mut *mut c_void,
            AddressFamily,
            AddressSize,
            AddressCount,
            AliasCount,
        );
    }

    i = 0;
    while i < AddressCount {
        let base_addr = &raw const (*DnsAddrArray).Addresses[i as usize] as *const u8;
        CurrentAddress = base_addr.add((*FamilyInfo).AddressOffset as usize) as *mut c_void;
        let hlist = &raw mut (*Hostent).h_addr_list as *mut *mut i8;
        *hlist.add(i as usize) = HostentPtr as *mut i8;
        core::ptr::copy_nonoverlapping(
            CurrentAddress as *const u8,
            HostentPtr as *mut u8,
            AddressSize as usize,
        );
        HostentPtr = HostentPtr.wrapping_add(AddressSize as usize);
        i = i.wrapping_add(1);
    }

    if !(*Blob).Name.is_null() {
        HostentPtr = (HostentPtr + 1) & !1;
        (*Hostent).h_name = HostentPtr as *mut i8;
        HostentPtr = HostentPtr.wrapping_add(
            Dns_StringCopy(
                HostentPtr as *mut c_void,
                core::ptr::null_mut(),
                (*Blob).Name as *const c_void,
                0,
                UnicodeString,
                StringType,
            ) as usize,
        );
    }

    i = AliasCount;
    while i != 0 {
        HostentPtr = (HostentPtr + 1) & !1;
        let hlist = &raw mut (*Hostent).h_aliases as *mut *mut i8;
        *hlist.add(i as usize) = HostentPtr as *mut i8;
        let alias_ptr = *((*Blob).Aliases.as_ptr() as *const *const c_void).add(i as usize);
        HostentPtr = HostentPtr.wrapping_add(
            Dns_StringCopy(
                HostentPtr as *mut c_void,
                core::ptr::null_mut(),
                alias_ptr,
                0,
                UnicodeString,
                StringType,
            ) as usize,
        );
        i = i.wrapping_sub(1);
    }

    if BufferAllocated == 0 {
        // C: *HostEntrySize = *BufferPosition - (ULONG_PTR)HostentPtr
        core::ptr::write_volatile(HostEntrySize, (*BufferPosition).wrapping_sub(HostentPtr));
    }

    if Relative != 0 {
        Hostent_ConvertToOffsets(Hostent);
    }

    Hostent
}

// ---- SaBlob_WriteNameOrAlias ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_WriteNameOrAlias(
    Blob: *mut DNS_BLOB,
    String: *mut u16,
    IsAlias: i32,
) -> i32 {
    if IsAlias == 0 {
        (*Blob).Name = Dns_CreateStringCopy_W(String);
        if (*Blob).Name.is_null() {
            return GetLastError() as i32;
        }
    } else {
        if !(*Blob).Name.is_null() && (*Blob).AliasCount <= 8 {
            (*Blob).Aliases[(*Blob).AliasCount as usize] = Dns_CreateStringCopy_W(String);
            (*Blob).AliasCount += 1;
        } else {
            return ntdll_sys::nt::ERROR_MORE_DATA as i32;
        }
    }
    ERROR_SUCCESS
}

// ---- SaBlob_WriteAddress ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_WriteAddress(
    Blob: *mut DNS_BLOB,
    DnsAddr: *mut DNS_ADDRESS,
) -> i32 {
    if (*Blob).DnsAddrArray.is_null() {
        (*Blob).DnsAddrArray = DnsAddrArray_Create(1);
        if (*Blob).DnsAddrArray.is_null() {
            return ERROR_OUTOFMEMORY as i32;
        }
    }
    if DnsAddrArray_AddAddr((*Blob).DnsAddrArray, DnsAddr, 0, 0) != 0 {
        ERROR_SUCCESS
    } else {
        ntdll_sys::nt::ERROR_MORE_DATA as i32
    }
}

// ---- SaBlob_IsSupportedAddrType ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_IsSupportedAddrType(DnsType: u16) -> u8 {
    (DnsType == DNS_TYPE_A || DnsType == DNS_TYPE_ATMA || DnsType == DNS_TYPE_AAAA) as u8
}

// ---- SaBlob_WriteRecords ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_WriteRecords(
    Blob: *mut DNS_BLOB,
    DnsRecord: *const DNS_RECORD,
    DoAlias: BOOLEAN,
) -> i32 {
    let mut dns_record_ptr: *const DNS_RECORD = DnsRecord;
    let mut dns_address: DNS_ADDRESS = core::mem::zeroed();
    let mut error_code: i32 = STATUS_INVALID_PARAMETER as i32;
    let mut wrote_once: BOOLEAN = 0u8;

    while !dns_record_ptr.is_null() {
        if (*dns_record_ptr).Flags.S.Section() != DNSREC_ANSWER {
            dns_record_ptr = (*dns_record_ptr).pNext;
            continue;
        }

        match (*dns_record_ptr).wType {
            DNS_TYPE_A | DNS_TYPE_AAAA | DNS_TYPE_ATMA => {
                DnsAddr_BuildFromDnsRecord(
                    dns_record_ptr as *mut DNS_RECORD,
                    &raw mut dns_address,
                );
                error_code = SaBlob_WriteAddress(Blob, &raw mut dns_address);
                if DoAlias != 0u8
                    && wrote_once == 0u8
                    && (*Blob).Name.is_null()
                    && !(*dns_record_ptr).pName.is_null()
                {
                    error_code = SaBlob_WriteNameOrAlias(
                        Blob,
                        (*dns_record_ptr).pName,
                        0,
                    );
                    wrote_once = 1u8;
                }
            }
            DNS_TYPE_CNAME => {
                error_code = SaBlob_WriteNameOrAlias(
                    Blob,
                    (*dns_record_ptr).pName,
                    1,
                );
            }
            DNS_TYPE_PTR => {
                if !(*Blob).Name.is_null() {
                    error_code = SaBlob_WriteNameOrAlias(
                        Blob,
                        (*dns_record_ptr).pName,
                        0,
                    );
                } else {
                    error_code = SaBlob_WriteNameOrAlias(
                        Blob,
                        (*dns_record_ptr).pName,
                        1,
                    );
                }
            }
            _ => {}
        }
        dns_record_ptr = (*dns_record_ptr).pNext;
    }

    error_code
}

// ---- SaBlob_CreateFromRecords ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_CreateFromRecords(
    DnsRecord: *mut DNS_RECORD,
    DoAliases: u8,
    DnsType: u32,
) -> *mut DNS_BLOB {
    let DNSREC_ANSWER_VAL: u8 = 1;

    let mut ProcessedCount: u32 = 0;
    let mut DnsBlob: *mut DNS_BLOB;
    let mut ErrorCode: i32 = 0;
    let mut DnsAddress: DNS_ADDRESS = core::mem::zeroed();

    let mut LocalDnsRecord = DnsRecord;
    while !LocalDnsRecord.is_null() {
        if ((*LocalDnsRecord).Flags.S.Section() == DNSREC_ANSWER_VAL as u32)
            && (SaBlob_IsSupportedAddrType((*LocalDnsRecord).wType) != 0)
        {
            ProcessedCount += 1;
        }
        LocalDnsRecord = (*LocalDnsRecord).pNext;
    }

    'quickie: {
        DnsBlob = SaBlob_Create(ProcessedCount);
        if DnsBlob.is_null() {
            ErrorCode = GetLastError() as i32;
            break 'quickie;
        }
        ErrorCode = SaBlob_WriteRecords(DnsBlob, DnsRecord, 1u8);
        if ErrorCode != 0 {
            if !(*DnsBlob).Name.is_null() || (*DnsBlob).AliasCount != 0 {
                ErrorCode = 0;
            } else {
                let addr_array = (*DnsBlob).DnsAddrArray;
                if !addr_array.is_null() && (*addr_array).UsedAddresses != 0 {
                    ErrorCode = 0;
                }
            }
            break 'quickie;
        }

        if ((*DnsRecord).wType == DNS_TYPE_PTR)
            || ((DnsType == DNS_TYPE_PTR as u32)
                && ((*DnsRecord).wType == DNS_TYPE_CNAME)
                && ((*DnsRecord).Flags.S.Section() == DNSREC_ANSWER_VAL as u32))
        {
            if Dns_ReverseNameToDnsAddr_W(
                &raw mut DnsAddress,
                (*DnsRecord).pName,
            ) != 0
            {
                if SaBlob_WriteAddress(DnsBlob, &raw mut DnsAddress) != 0 {
                    ErrorCode = 0;
                }
            }
        }

        if (*DnsBlob).Name.is_null() && DoAliases != 0 && !LocalDnsRecord.is_null() {
            ErrorCode = SaBlob_WriteNameOrAlias(DnsBlob, (*LocalDnsRecord).pName, 0);
        }
    }

    if ErrorCode != 0 {
        SaBlob_Free(DnsBlob);
        DnsBlob = core::ptr::null_mut();
        SetLastError(ErrorCode as u32);
    }
    DnsBlob
}

// ---- SaBlob_Query ----
#[no_mangle]
pub unsafe extern "system" fn SaBlob_Query(
    name: *mut u16,
    dns_type: u16,
    flags: u32,
    reserved: *mut *mut c_void,
    _address_family: u32,
) -> *mut DNS_BLOB {
    let mut dns_record: *mut DNS_RECORD = core::ptr::null_mut();
    let mut error_code: i32 = 0;
    let mut dns_blob: *mut DNS_BLOB = core::ptr::null_mut();
    let local_name: *mut u16;
    let local_name_copy: *mut u16;

    'quickie: {
        if !reserved.is_null() {
            core::ptr::write_volatile(reserved, core::ptr::null_mut());
        }

        error_code = DnsQuery_W(
            name,
            dns_type,
            flags,
            core::ptr::null_mut(),
            &raw mut dns_record,
            reserved,
        );

        if error_code != ERROR_SUCCESS {
            if !reserved.is_null() && !(*reserved).is_null() {
                DnsApiFree(*reserved);
                core::ptr::write_volatile(reserved, core::ptr::null_mut());
            }
            if error_code == ntdll_sys::nt::RPC_S_SERVER_UNAVAILABLE as i32 {
                error_code = ntdll_sys::nt::WSATRY_AGAIN as i32;
            }
            break 'quickie;
        }

        dns_blob = SaBlob_CreateFromRecords(dns_record, 1, dns_type as u32);
        if dns_blob.is_null() {
            error_code = GetLastError() as i32;
            break 'quickie;
        }

        if (*dns_blob).Name.is_null() {
            error_code = ntdll_sys::nt::DNS_INFO_NO_RECORDS as i32;
            break 'quickie;
        }

        let match_local = DnsNameCompare_W((*dns_blob).Name, LOCALHOST.as_ptr() as *mut u16);
        let match_loop = DnsNameCompare_W((*dns_blob).Name, LOOPBACK.as_ptr() as *mut u16);

        if match_local == 0 && match_loop == 0 {
            break 'quickie;
        }

        local_name = DnsQueryConfigAllocEx(
            DnsConfigFullHostName_W,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if !local_name.is_null() {
            local_name_copy = Dns_CreateStringCopy_W(local_name);
            if !local_name_copy.is_null() {
                (*dns_blob).Name = local_name_copy;
            } else {
                DnsApiFree(local_name as *mut c_void);
            }
        }
    }

    if !dns_record.is_null() {
        DnsRecordListFree(dns_record, DnsFreeRecordList);
    }

    if error_code != ERROR_SUCCESS && !dns_blob.is_null() {
        SaBlob_Free(dns_blob);
        dns_blob = core::ptr::null_mut();
    }

    SetLastError(error_code as u32);
    dns_blob
}
