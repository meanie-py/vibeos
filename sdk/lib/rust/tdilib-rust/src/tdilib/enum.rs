use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn tdiGetSetOfThings(
    mut tcpFile: HANDLE,
    mut toiClass: DWORD,
    mut toiType: DWORD,
    mut toiId: DWORD,
    mut teiEntity: DWORD,
    mut teiInstance: DWORD,
    mut fixedPart: DWORD,
    mut entrySize: DWORD,
    mut tdiEntitySet: *mut PVOID,
    mut numEntries: PDWORD,
) -> NTSTATUS {
    // iphlpapi_private.h: TCP_REQUEST_QUERY_INFORMATION_INIT is `{ { { 0 } } }`, all zero.
    let mut req: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();
    let mut entitySet: PVOID = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut allocationSizeForEntityArray: DWORD = entrySize.wrapping_mul(MAX_TDI_ENTITIES as DWORD);
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    req.ID.toi_class = toiClass as ULONG;
    req.ID.toi_type = toiType as ULONG;
    req.ID.toi_id = toiId as ULONG;
    req.ID.toi_entity.tei_entity = teiEntity as ULONG;
    req.ID.toi_entity.tei_instance = teiInstance as ULONG;

    /* There's a subtle problem here...
     * If an interface is added at this exact instant, (as if by a PCMCIA
     * card insertion), the array will still not have enough entries after
     * have allocated it after the first DeviceIoControl call.
     *
     * We'll get around this by repeating until the number of interfaces
     * stabilizes.
     */
    loop {
        status = NtDeviceIoControlFile(
            tcpFile,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Iosb),
            (IOCTL_TCP_QUERY_INFORMATION_EX) as ULONG,
            core::ptr::addr_of_mut!(req) as PVOID,
            core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as ULONG,
            core::ptr::null_mut(),
            (0) as ULONG,
        );
        if status == STATUS_PENDING {
            status = NtWaitForSingleObject(
                tcpFile,
                FALSE as BOOLEAN,
                core::ptr::null_mut::<_LARGE_INTEGER>(),
            );
            if NT_SUCCESS!(status) {
                status = Iosb.__bindgen_anon_1.Status;
            }
        }

        if !NT_SUCCESS!(status) {
            return status;
        }

        allocationSizeForEntityArray = Iosb.Information as DWORD;
        entitySet = HeapAlloc(
            GetProcessHeap(),
            (0) as DWORD,
            allocationSizeForEntityArray as SIZE_T,
        );

        if entitySet.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            return status;
        }

        status = NtDeviceIoControlFile(
            tcpFile,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Iosb),
            (IOCTL_TCP_QUERY_INFORMATION_EX) as ULONG,
            core::ptr::addr_of_mut!(req) as PVOID,
            core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as ULONG,
            entitySet,
            allocationSizeForEntityArray as ULONG,
        );
        if status == STATUS_PENDING {
            status = NtWaitForSingleObject(
                tcpFile,
                FALSE as BOOLEAN,
                core::ptr::null_mut::<_LARGE_INTEGER>(),
            );
            if NT_SUCCESS!(status) {
                status = Iosb.__bindgen_anon_1.Status;
            }
        }

        /* This is why we have the loop -- we might have added an adapter */
        if Iosb.Information == allocationSizeForEntityArray as ULONG_PTR {
            break;
        }

        HeapFree(
            GetProcessHeap(),
            (0) as DWORD,
            entitySet,
        );
        entitySet = (0) as PVOID;

        if !NT_SUCCESS!(status) {
            return status;
        }

        /* We break if the array we received was the size we
         * expected.  Therefore, we got here because it wasn't */
        if !((TRUE) != 0) {
            break;
        }
    }

    *numEntries = allocationSizeForEntityArray
        .wrapping_sub(fixedPart)
        .wrapping_div(entrySize);
    *tdiEntitySet = entitySet;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn tdiFreeThingSet(things: PVOID) {
    HeapFree(GetProcessHeap(), 0, things);
}
#[no_mangle]
pub unsafe extern "C" fn tdiGetEntityIDSet(
    tcpFile: HANDLE,
    entitySet: *mut *mut TDIEntityID,
    numEntities: PDWORD,
) -> NTSTATUS {
    let status: NTSTATUS = tdiGetSetOfThings(
        tcpFile,
        INFO_CLASS_GENERIC as DWORD,
        INFO_TYPE_PROVIDER as DWORD,
        ENTITY_LIST_ID as DWORD,
        GENERIC_ENTITY as DWORD,
        0 as DWORD,
        0 as DWORD,
        core::mem::size_of::<TDIEntityID>() as DWORD,
        entitySet as *mut PVOID,
        numEntities,
    );

    return status;
}
