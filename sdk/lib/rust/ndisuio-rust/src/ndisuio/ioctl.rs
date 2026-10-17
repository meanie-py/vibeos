use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, _KWAIT_REASON_Executive, KeWaitForSingleObject, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

unsafe extern "C" fn WaitForBind(Irp: PIRP, IrpSp: PIO_STACK_LOCATION) -> NTSTATUS {
    DPRINT!("Wait for bind complete\n");

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return STATUS_SUCCESS;
}

unsafe extern "C" fn QueryBinding(Irp: PIRP, IrpSp: PIO_STACK_LOCATION) -> NTSTATUS {
    let mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT = core::ptr::null_mut();
    let QueryBinding: PNDISUIO_QUERY_BINDING = (*Irp).AssociatedIrp.SystemBuffer as PNDISUIO_QUERY_BINDING;
    let BindingLength: ULONG = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;
    let mut Status: NTSTATUS = 0;
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut i: ULONG = 0;
    let mut BytesCopied: ULONG = 0;

    if !QueryBinding.is_null() && BindingLength as usize >= core::mem::size_of::<NDISUIO_QUERY_BINDING>() {
        KeAcquireSpinLock(core::ptr::addr_of_mut!(GlobalAdapterListLock), &mut OldIrql);
        i = 0;
        CurrentEntry = GlobalAdapterList.Flink as PLIST_ENTRY;
        while CurrentEntry != core::ptr::addr_of_mut!(GlobalAdapterList) as PLIST_ENTRY {
            if i == (*QueryBinding).BindingIndex {
                AdapterContext = CONTAINING_RECORD!(CurrentEntry, NDISUIO_ADAPTER_CONTEXT, ListEntry);
                break;
            }
            i += 1;
            CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;
        }
        KeReleaseSpinLock(core::ptr::addr_of_mut!(GlobalAdapterListLock), OldIrql);
        if !AdapterContext.is_null() {
            DPRINT!(
                "Query binding for index %d is adapter %wZ\n",
                i,
                core::ptr::addr_of_mut!((*AdapterContext).DeviceName)
            );
            BytesCopied = core::mem::size_of::<NDISUIO_QUERY_BINDING>() as ULONG;
            if (*AdapterContext).DeviceName.Length as ULONG <= BindingLength - BytesCopied {
                (*QueryBinding).DeviceNameOffset = BytesCopied;
                (*QueryBinding).DeviceNameLength = (*AdapterContext).DeviceName.Length as ULONG;
                RtlCopyMemory!(
                    (QueryBinding as PUCHAR).add((*QueryBinding).DeviceNameOffset as usize) as *mut core::ffi::c_void,
                    (*AdapterContext).DeviceName.Buffer as *const core::ffi::c_void,
                    (*QueryBinding).DeviceNameLength as usize,
                );
                BytesCopied += (*AdapterContext).DeviceName.Length as ULONG;

                /* FIXME: Copy description too */
                (*QueryBinding).DeviceDescrOffset = BytesCopied;
                (*QueryBinding).DeviceDescrLength = 0;

                /* Successful */
                Status = STATUS_SUCCESS;
            } else {
                /* Not enough buffer space */
                Status = STATUS_BUFFER_TOO_SMALL;
            }
        } else {
            /* Invalid index */
            Status = STATUS_NO_MORE_ENTRIES;
        }
    } else {
        /* Invalid parameters */
        Status = STATUS_INVALID_PARAMETER;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = BytesCopied as ULONG_PTR;

    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}

unsafe extern "C" fn SetAdapterOid(Irp: PIRP, IrpSp: PIO_STACK_LOCATION) -> NTSTATUS {
    let AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        (*(*IrpSp).FileObject).FsContext as PNDISUIO_ADAPTER_CONTEXT;
    let mut SetOidRequest: PNDISUIO_SET_OID;
    let mut Request: NDIS_REQUEST = core::mem::zeroed();
    let mut RequestLength: ULONG;
    let mut Status: NDIS_STATUS = 0;

    (*Irp).IoStatus.Information = 0;

    SetOidRequest = (*Irp).AssociatedIrp.SystemBuffer as PNDISUIO_SET_OID;
    RequestLength = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;
    if !SetOidRequest.is_null()
        && RequestLength as usize >= core::mem::size_of::<NDIS_OID>()
    {
        /* Setup the NDIS request */
        Request.RequestType = NdisRequestSetInformation;
        Request.DATA.SET_INFORMATION.Oid = (*SetOidRequest).Oid;
        Request.DATA.SET_INFORMATION.InformationBufferLength =
            (RequestLength as usize).wrapping_sub(core::mem::size_of::<NDIS_OID>())
                as UINT;
        if Request.DATA.SET_INFORMATION.InformationBufferLength
            != 0 as UINT
        {
            Request.DATA.SET_INFORMATION.InformationBuffer =
                core::ptr::addr_of_mut!((*SetOidRequest).Data) as PVOID;
        } else {
            Request.DATA.SET_INFORMATION.InformationBuffer =
                core::ptr::null_mut();
        }
        Request.DATA.SET_INFORMATION.BytesRead =
            0 as UINT;

        DPRINT!(
            "Setting OID 0x%x on adapter %wZ\n",
            (*SetOidRequest).Oid,
            core::ptr::addr_of_mut!((*AdapterContext).DeviceName)
        );

        /* Dispatch the request */
        NdisRequest(
            core::ptr::addr_of_mut!(Status),
            (*AdapterContext).BindingHandle,
            core::ptr::addr_of_mut!(Request),
        );

        /* Wait for the request */
        if Status == NDIS_STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent) as PVOID,
                _KWAIT_REASON_Executive,
                (0i32) as KPROCESSOR_MODE,
                FALSE,
                core::ptr::null_mut(),
            );
            Status = (*AdapterContext).AsyncStatus;
        }

        /* Return the bytes read */
        if Status == NDIS_STATUS_INVALID_LENGTH || Status == NDIS_STATUS_BUFFER_TOO_SHORT {
            Status = STATUS_BUFFER_TOO_SMALL;
        } else if Status == NDIS_STATUS_SUCCESS {
            (*Irp).IoStatus.Information = (core::mem::size_of::<NDIS_OID>() as ULONG_PTR)
                .wrapping_add(Request.DATA.SET_INFORMATION.BytesRead as ULONG_PTR);
        }

        DPRINT!(
            "Final request status: 0x%x (%d)\n",
            Status,
            (*Irp).IoStatus.Information
        );
    } else {
        /* Bad parameters */
        Status = STATUS_INVALID_PARAMETER;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return Status;
}

unsafe extern "C" fn QueryAdapterOid(mut Irp: PIRP, mut IrpSp: PIO_STACK_LOCATION) -> NTSTATUS {
    let mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        (*(*IrpSp).FileObject).FsContext as PNDISUIO_ADAPTER_CONTEXT;
    let mut QueryOidRequest: PNDISUIO_QUERY_OID = core::ptr::null_mut();
    let mut Request: NDIS_REQUEST = core::mem::zeroed();
    let mut RequestLength: ULONG = 0;
    let mut Status: NDIS_STATUS = 0;

    (*Irp).IoStatus.Information = 0 as ULONG_PTR;

    QueryOidRequest = (*Irp).AssociatedIrp.SystemBuffer as PNDISUIO_QUERY_OID;
    RequestLength = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;
    if !QueryOidRequest.is_null()
        && (RequestLength as usize) >= core::mem::size_of::<NDIS_OID>()
    {
        /* Setup the NDIS request */
        Request.RequestType = NdisRequestQueryInformation;
        Request.DATA.QUERY_INFORMATION.Oid = (*QueryOidRequest).Oid;
        Request.DATA.QUERY_INFORMATION.InformationBufferLength =
            ((RequestLength as usize).wrapping_sub(core::mem::size_of::<NDIS_OID>()))
                as UINT;
        if Request.DATA.QUERY_INFORMATION.InformationBufferLength
            != 0 as UINT
        {
            Request.DATA.QUERY_INFORMATION.InformationBuffer =
                core::ptr::addr_of_mut!((*QueryOidRequest).Data) as PVOID;
        } else {
            Request.DATA.QUERY_INFORMATION.InformationBuffer =
                core::ptr::null_mut();
        }
        Request.DATA.QUERY_INFORMATION.BytesWritten =
            0 as UINT;

        DPRINT!(
            "Querying OID 0x%x on adapter %wZ\n",
            (*QueryOidRequest).Oid,
            core::ptr::addr_of_mut!((*AdapterContext).DeviceName)
        );

        /* Dispatch the request */
        NdisRequest(
            core::ptr::addr_of_mut!(Status),
            (*AdapterContext).BindingHandle,
            core::ptr::addr_of_mut!(Request),
        );

        /* Wait for the request */
        if Status == NDIS_STATUS_PENDING as NDIS_STATUS {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*AdapterContext).AsyncEvent) as PVOID,
                _KWAIT_REASON_Executive,
                (0i32) as KPROCESSOR_MODE,
                FALSE,
                core::ptr::null_mut(),
            );
            Status = (*AdapterContext).AsyncStatus;
        }

        if Status == NDIS_STATUS_INVALID_LENGTH as NDIS_STATUS
            || Status == NDIS_STATUS_BUFFER_TOO_SHORT as NDIS_STATUS
            || Status == NDIS_STATUS_BUFFER_OVERFLOW as NDIS_STATUS
        {
            Status = STATUS_BUFFER_OVERFLOW as NDIS_STATUS;
        } else if Status == NDIS_STATUS_SUCCESS as NDIS_STATUS {
            (*Irp).IoStatus.Information = (core::mem::size_of::<NDIS_OID>() as usize)
                .wrapping_add(Request.DATA.QUERY_INFORMATION.BytesWritten as usize)
                as ULONG_PTR;
        }

        DPRINT!(
            "Final request status: 0x%x (%d)\n",
            Status,
            (*Irp).IoStatus.Information
        );
    } else {
        /* Bad parameters */
        Status = STATUS_INVALID_PARAMETER as NDIS_STATUS;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status as NTSTATUS;

    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return Status as NTSTATUS;
}

unsafe extern "C" fn OpenDeviceReadWrite(mut Irp: PIRP, mut IrpSp: PIO_STACK_LOCATION) -> NTSTATUS {
    let mut FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut NameLength: ULONG = 0;
    let mut Status: NTSTATUS = 0;
    let mut AdapterContext: PNDISUIO_ADAPTER_CONTEXT =
        ::core::ptr::null_mut::<_NDISUIO_ADAPTER_CONTEXT>();
    let mut OpenEntry: PNDISUIO_OPEN_ENTRY = ::core::ptr::null_mut::<_NDISUIO_OPEN_ENTRY>();
    let mut OldIrql: KIRQL = 0;

    NameLength = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;
    if NameLength != 0 {
        DeviceName.Length = NameLength as USHORT;
        DeviceName.MaximumLength = DeviceName.Length;
        DeviceName.Buffer = (*Irp).AssociatedIrp.SystemBuffer as PWCH;

        /* The bound name is a counted string with no terminator, so
         * drop one the caller may have included before matching */
        if DeviceName.Length as usize >= ::core::mem::size_of::<WCHAR>() as usize
            && *DeviceName.Buffer.offset(
                (DeviceName.Length as usize)
                    .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
                == UNICODE_NULL as ::core::ffi::c_int
        {
            DeviceName.Length = (DeviceName.Length as ::core::ffi::c_ulonglong)
                .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize as ::core::ffi::c_ulonglong)
                as USHORT as USHORT;
        }

        /* Check if this already has a context */
        AdapterContext = FindAdapterContextByName(core::ptr::addr_of_mut!(DeviceName));
        if !AdapterContext.is_null() {
            DPRINT!(
                "Binding file object 0x%x to device %wZ\n",
                FileObject,
                core::ptr::addr_of_mut!((*AdapterContext).DeviceName)
            );

            /* Reference the adapter context */
            KeAcquireSpinLock(
                core::ptr::addr_of_mut!((*AdapterContext).Spinlock),
                core::ptr::addr_of_mut!(OldIrql),
            );
            if (*AdapterContext).OpenCount != 0 {
                /* An open for read-write is exclusive,
                 * so we can't have any other open handles */
                KeReleaseSpinLock(
                    core::ptr::addr_of_mut!((*AdapterContext).Spinlock),
                    OldIrql,
                );
                Status = STATUS_INVALID_PARAMETER as NTSTATUS;
            } else {
                /* Add a reference */
                ReferenceAdapterContext(AdapterContext);
                Status = STATUS_SUCCESS as NTSTATUS;
            }
        } else {
            /* Invalid device name */
            Status = STATUS_INVALID_PARAMETER as NTSTATUS;
        }

        /* Check that the bind succeeded */
        if NT_SUCCESS!(Status) {
            OpenEntry = ExAllocatePool(
                NonPagedPool,
                ::core::mem::size_of::<_NDISUIO_OPEN_ENTRY>() as SIZE_T,
            ) as PNDISUIO_OPEN_ENTRY;
            if !OpenEntry.is_null() {
                /* Set the file object pointer */
                (*OpenEntry).FileObject = FileObject;

                /* Set the permissions */
                (*OpenEntry).WriteOnly = FALSE as BOOLEAN;

                /* Associate this FO with the adapter */
                (*FileObject).FsContext = AdapterContext as PVOID;
                (*FileObject).FsContext2 = OpenEntry as PVOID;

                /* Add it to the adapter's list */
                InsertTailList(
                    core::ptr::addr_of_mut!((*AdapterContext).OpenEntryList),
                    core::ptr::addr_of_mut!((*OpenEntry).ListEntry),
                );

                /* Success */
                KeReleaseSpinLock(
                    core::ptr::addr_of_mut!((*AdapterContext).Spinlock),
                    OldIrql,
                );
                Status = STATUS_SUCCESS as NTSTATUS;
            } else {
                /* Remove the reference we added */
                KeReleaseSpinLock(
                    core::ptr::addr_of_mut!((*AdapterContext).Spinlock),
                    OldIrql,
                );
                DereferenceAdapterContextWithOpenEntry(AdapterContext, ::core::ptr::null_mut());
                Status = STATUS_NO_MEMORY as NTSTATUS;
            }
        }
    } else {
        /* Invalid device name */
        Status = STATUS_INVALID_PARAMETER as NTSTATUS;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0 as ULONG_PTR;

    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn NduDispatchDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut OpenEntry: PNDISUIO_OPEN_ENTRY = core::ptr::null_mut();

    ASSERT!(DeviceObject == GlobalDeviceObject);

    /* Handle open IOCTLs first */
    match (*IrpSp).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_NDISUIO_OPEN_DEVICE => {
            return OpenDeviceReadWrite(Irp, IrpSp);
        }
        IOCTL_NDISUIO_BIND_WAIT => {
            return WaitForBind(Irp, IrpSp);
        }

        IOCTL_NDISUIO_QUERY_BINDING => {
            return QueryBinding(Irp, IrpSp);
        }

        _ => {
            /* Fail if this file object has no adapter associated */
            if (*(*IrpSp).FileObject).FsContext.is_null() {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                (*Irp).IoStatus.Information = 0;
                IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

                return STATUS_INVALID_PARAMETER;
            }

            /* Now handle write IOCTLs */
            match (*IrpSp).Parameters.DeviceIoControl.IoControlCode {
                IOCTL_NDISUIO_SET_OID_VALUE => {
                    return SetAdapterOid(Irp, IrpSp);
                }

                _ => {
                    /* Check that we have read permissions */
                    OpenEntry = (*(*IrpSp).FileObject).FsContext2 as PNDISUIO_OPEN_ENTRY;
                    if (*OpenEntry).WriteOnly != 0 {
                        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                        (*Irp).IoStatus.Information = 0;
                        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

                        return STATUS_INVALID_PARAMETER;
                    }

                    match (*IrpSp).Parameters.DeviceIoControl.IoControlCode {
                        IOCTL_NDISUIO_QUERY_OID_VALUE => {
                            return QueryAdapterOid(Irp, IrpSp);
                        }

                        _ => {
                            DPRINT1!("Unimplemented\n");
                            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
                            (*Irp).IoStatus.Information = 0;
                            IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
                            return STATUS_NOT_IMPLEMENTED;
                        }
                    }
                }
            }
        }
    }
}
