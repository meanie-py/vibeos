use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn TdiAddressSizeFromType(AddressType: UINT) -> UINT {
    match AddressType {
        TDI_ADDRESS_TYPE_IP => {
            return TDI_ADDRESS_LENGTH_IP as UINT;
        }
        TDI_ADDRESS_TYPE_APPLETALK => {
            return TDI_ADDRESS_LENGTH_APPLETALK as UINT;
        }
        TDI_ADDRESS_TYPE_NETBIOS => {
            return TDI_ADDRESS_LENGTH_NETBIOS as UINT;
        }
        /* case TDI_ADDRESS_TYPE_NS: */
        TDI_ADDRESS_TYPE_IPX => {
            return TDI_ADDRESS_LENGTH_IPX as UINT;
        }
        TDI_ADDRESS_TYPE_VNS => {
            return TDI_ADDRESS_LENGTH_VNS as UINT;
        }
        _ => {
            DbgPrint(
                b"TdiAddressSizeFromType - invalid type: %x\n\0" as *const u8 as PCSTR,
                AddressType,
            );
            return 0 as UINT;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn TaLengthOfAddress(Addr: PTA_ADDRESS) -> UINT {
    let mut AddrLen: UINT = (*Addr).AddressLength as UINT;

    if AddrLen == 0 {
        return 0;
    }

    AddrLen = AddrLen.wrapping_add(2 * core::mem::size_of::<USHORT>() as UINT);

    DPRINT!("AddrLen %x\n", AddrLen);

    return AddrLen;
}
#[no_mangle]
pub unsafe extern "C" fn TaLengthOfTransportAddress(Addr: PTRANSPORT_ADDRESS) -> UINT {
    let mut AddrLen: UINT = TaLengthOfAddress(
        core::ptr::addr_of_mut!((*(*Addr).Address.as_mut_ptr().add(0 as usize))) as PTA_ADDRESS,
    );

    if AddrLen == 0 {
        return 0;
    }

    AddrLen = AddrLen.wrapping_add(core::mem::size_of::<ULONG>() as UINT);

    DPRINT!("AddrLen %x\n", AddrLen);

    return AddrLen;
}
#[no_mangle]
pub unsafe extern "C" fn TaLengthOfTransportAddressByType(AddressType: UINT) -> UINT {
    let mut AddrLen: UINT = TdiAddressSizeFromType(AddressType);

    if AddrLen == 0 {
        return 0;
    }

    AddrLen += (core::mem::size_of::<ULONG>() + (2 as usize) * core::mem::size_of::<USHORT>()) as UINT;

    DPRINT!("AddrLen %x\n", AddrLen);

    return AddrLen;
}
#[no_mangle]
pub unsafe extern "C" fn TaCopyTransportAddressInPlace(
    Target: PTRANSPORT_ADDRESS,
    Source: PTRANSPORT_ADDRESS,
) {
    let AddrLen: UINT = TaLengthOfTransportAddress(Source);
    RtlCopyMemory!(Target as *mut core::ffi::c_void, Source as *const core::ffi::c_void, AddrLen as usize);
}
#[no_mangle]
pub unsafe extern "C" fn TaCopyTransportAddress(mut OtherAddress: PTRANSPORT_ADDRESS) -> PTRANSPORT_ADDRESS {
    let mut AddrLen: UINT;
    let mut A: PTRANSPORT_ADDRESS;

    AddrLen = TaLengthOfTransportAddress(OtherAddress);
    if AddrLen == 0 {
        return core::ptr::null_mut();
    }

    A = ExAllocatePoolWithTag(
        NonPagedPool,
        AddrLen as SIZE_T,
        TAG_AFD_TRANSPORT_ADDRESS as ULONG,
    ) as PTRANSPORT_ADDRESS;

    if !A.is_null() {
        TaCopyTransportAddressInPlace(A, OtherAddress);
    }

    return A;
}
#[no_mangle]
pub unsafe extern "C" fn TdiBuildNullTransportAddressInPlace(A: PTRANSPORT_ADDRESS, AddressType: UINT) -> NTSTATUS {
    (*A).TAAddressCount = 1 as LONG;

    (*(core::ptr::addr_of_mut!((*A).Address) as *mut TA_ADDRESS).offset(0 as isize))
        .AddressLength = TdiAddressSizeFromType(AddressType) as USHORT;
    if (*(core::ptr::addr_of_mut!((*A).Address) as *mut TA_ADDRESS).offset(0 as isize))
        .AddressLength == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    (*(core::ptr::addr_of_mut!((*A).Address) as *mut TA_ADDRESS).offset(0 as isize))
        .AddressType = AddressType as USHORT;

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(
            (*(core::ptr::addr_of_mut!((*A).Address) as *mut TA_ADDRESS).offset(0 as isize))
                .Address
        ) as *mut UCHAR as *mut core::ffi::c_void,
        (*(core::ptr::addr_of_mut!((*A).Address) as *mut TA_ADDRESS).offset(0 as isize))
            .AddressLength as usize,
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn TaBuildNullTransportAddress(AddressType: UINT) -> PTRANSPORT_ADDRESS {
    let mut AddrLen: UINT;
    let mut A: PTRANSPORT_ADDRESS;

    AddrLen = TaLengthOfTransportAddressByType(AddressType);
    if AddrLen == 0 {
        return core::ptr::null_mut();
    }

    A = ExAllocatePoolWithTag(NonPagedPool, AddrLen as SIZE_T, TAG_AFD_TRANSPORT_ADDRESS as ULONG) as PTRANSPORT_ADDRESS;

    if !A.is_null() {
        if TdiBuildNullTransportAddressInPlace(A, AddressType) != STATUS_SUCCESS {
            ExFreePoolWithTag(A as PVOID, TAG_AFD_TRANSPORT_ADDRESS as ULONG);
            return core::ptr::null_mut();
        }
    }

    return A;
}

#[no_mangle]
pub unsafe extern "C" fn TdiBuildNullConnectionInfoInPlace(
    ConnInfo: PTDI_CONNECTION_INFORMATION,
    Type: ULONG,
) -> NTSTATUS {
    let mut TdiAddressSize: ULONG;
    let mut TransportAddress: PTRANSPORT_ADDRESS;

    TdiAddressSize = TaLengthOfTransportAddressByType(Type as UINT) as ULONG;
    if TdiAddressSize == 0 {
        DPRINT!("Invalid parameter\n");
        return STATUS_INVALID_PARAMETER;
    }

    RtlZeroMemory!(
        ConnInfo as PVOID,
        (core::mem::size_of::<TDI_CONNECTION_INFORMATION>() as usize)
            .wrapping_add(TdiAddressSize as usize),
    );

    (*ConnInfo).OptionsLength = core::mem::size_of::<ULONG>() as LONG;
    (*ConnInfo).RemoteAddressLength = TdiAddressSize as LONG;
    TransportAddress = ConnInfo.offset(1 as isize) as PTRANSPORT_ADDRESS;
    (*ConnInfo).RemoteAddress = TransportAddress as PVOID;

    return TdiBuildNullTransportAddressInPlace(TransportAddress, Type as UINT);
}
#[no_mangle]
pub unsafe extern "C" fn TdiBuildNullConnectionInfo(
    ConnectionInfo: *mut PTDI_CONNECTION_INFORMATION,
    Type: ULONG,
) -> NTSTATUS {
    let mut ConnInfo: PTDI_CONNECTION_INFORMATION = core::ptr::null_mut();
    let mut TdiAddressSize: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    TdiAddressSize = TaLengthOfTransportAddressByType(Type as UINT) as ULONG;
    if TdiAddressSize == 0 {
        DPRINT!("Invalid parameter\n");
        *ConnectionInfo = core::ptr::null_mut();
        return STATUS_INVALID_PARAMETER;
    }

    ConnInfo = ExAllocatePoolWithTag(
        NonPagedPool,
        (core::mem::size_of::<TDI_CONNECTION_INFORMATION>() as SIZE_T)
            .wrapping_add(TdiAddressSize as SIZE_T),
        TAG_AFD_TDI_CONNECTION_INFORMATION,
    ) as PTDI_CONNECTION_INFORMATION;
    if ConnInfo.is_null() {
        *ConnectionInfo = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = TdiBuildNullConnectionInfoInPlace(ConnInfo, Type);

    if !NT_SUCCESS!(Status) {
        ExFreePoolWithTag(ConnInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
        ConnInfo = core::ptr::null_mut();
    }

    *ConnectionInfo = ConnInfo;

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn TdiBuildConnectionInfoInPlace(
    mut ConnectionInfo: PTDI_CONNECTION_INFORMATION,
    mut Address: PTRANSPORT_ADDRESS,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    // _SEH2_TRY/_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) as a real handler (ntkrnl_sys::seh);
    // the C's except arm is dead under the kernel's DUMMY_PSEH.
    let mut __seh = ();
    if let Err(code) = ntkrnl_sys::seh::try_except(
        &mut __seh,
        |_| {
            RtlCopyMemory!(
                (*ConnectionInfo).RemoteAddress as *mut core::ffi::c_void,
                Address as *const core::ffi::c_void,
                (*ConnectionInfo).RemoteAddressLength as usize,
            );
        },
        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    ) {
        Status = code;
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn TdiBuildConnectionInfo(
    ConnectionInfo: *mut PTDI_CONNECTION_INFORMATION,
    Address: PTRANSPORT_ADDRESS,
) -> NTSTATUS {
    let mut Status: NTSTATUS = TdiBuildNullConnectionInfo(
        ConnectionInfo,
        (*(&raw mut (*Address).Address as *mut TA_ADDRESS).offset(0 as isize))
            .AddressType as ULONG,
    );

    if NT_SUCCESS!(Status) {
        TdiBuildConnectionInfoInPlace(*ConnectionInfo, Address);
    }

    return Status;
}
