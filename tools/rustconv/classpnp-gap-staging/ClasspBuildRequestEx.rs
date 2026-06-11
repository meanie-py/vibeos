use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoSetCompletionRoutine, LARGE_INTEGER, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _CSCAN_LIST_ENTRY {
    pub Entry: _LIST_ENTRY,
    pub BlockNumber: u64,
}

pub type CSCAN_LIST_ENTRY = _CSCAN_LIST_ENTRY;
pub type PCSCAN_LIST_ENTRY = *mut _CSCAN_LIST_ENTRY;
#[no_mangle]
pub unsafe extern "C" fn ClasspBuildRequestEx(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Irp: PIRP,
    Srb: PSCSI_REQUEST_BLOCK
) {
    let currentIrpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let nextIrpStack: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);

    let startingOffset: LARGE_INTEGER = (*currentIrpStack).__bindgen_anon_1.Read.ByteOffset;

    let mut cdb: PCDB;
    let mut logicalBlockAddress: ULONG;
    let mut transferBlocks: USHORT;
    let mut status: NTSTATUS;
    let srbHeader: PSTORAGE_REQUEST_BLOCK_HEADER = Srb as PSTORAGE_REQUEST_BLOCK_HEADER;

    if Srb.is_null() {
        NT_ASSERT(0);
        return;
    }

    logicalBlockAddress = Int64ShrlMod32(
        (*startingOffset.QuadPart()).QuadPart,
        (*FdoExtension).SectorShift
    ) as ULONG;

    if (*(*FdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        status = InitializeStorageRequestBlock(
            Srb as PSTORAGE_REQUEST_BLOCK,
            STORAGE_ADDRESS_TYPE_BTL8,
            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE,
            1,
            SrbExDataTypeScsiCdb16
        );
        if !NT_SUCCESS(status) {
            NT_ASSERT(0);
            return;
        }

        (*(Srb as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
    } else {
        RtlZeroMemory(Srb as *mut core::ffi::c_void, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

        (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        (*Srb).Function = SRB_FUNCTION_EXECUTE_SCSI;
    }

    SrbSetOriginalRequest(srbHeader, Irp as PVOID);

    SrbSetDataBuffer(
        srbHeader,
        MmGetMdlVirtualAddress((*Irp).MdlAddress)
    );

    SrbSetDataTransferLength(
        srbHeader,
        (*currentIrpStack).__bindgen_anon_1.Read.Length
    );

    SrbSetRequestAttribute(srbHeader, SRB_SIMPLE_TAG_REQUEST);

    SrbSetQueueSortKey(srbHeader, logicalBlockAddress);

    SrbSetSenseInfoBuffer(srbHeader, (*FdoExtension).SenseData);
    SrbSetSenseInfoBufferLength(srbHeader, GET_FDO_EXTENSON_SENSE_DATA_LENGTH(FdoExtension));

    SrbSetTimeOutValue(
        srbHeader,
        ((SrbGetDataTransferLength(srbHeader) + 0xFFFF) >> 16) * (*FdoExtension).TimeOutValue
    );

    SrbSetCdbLength(srbHeader, 10);

    cdb = SrbGetCdb(srbHeader);
    NT_ASSERT(!cdb.is_null());

    transferBlocks = ((*currentIrpStack).__bindgen_anon_1.Read.Length >> (*FdoExtension).SectorShift) as USHORT;

    (*cdb).CDB10.LogicalBlockByte0 = ((&logicalBlockAddress as *const ULONG as *const FOUR_BYTE).cast::<u8>().offset(3).read());
    (*cdb).CDB10.LogicalBlockByte1 = ((&logicalBlockAddress as *const ULONG as *const FOUR_BYTE).cast::<u8>().offset(2).read());
    (*cdb).CDB10.LogicalBlockByte2 = ((&logicalBlockAddress as *const ULONG as *const FOUR_BYTE).cast::<u8>().offset(1).read());
    (*cdb).CDB10.LogicalBlockByte3 = ((&logicalBlockAddress as *const ULONG as *const FOUR_BYTE).cast::<u8>().offset(0).read());

    (*cdb).CDB10.TransferBlocksMsb = ((&transferBlocks as *const USHORT as *const FOUR_BYTE).cast::<u8>().offset(1).read());
    (*cdb).CDB10.TransferBlocksLsb = ((&transferBlocks as *const USHORT as *const FOUR_BYTE).cast::<u8>().offset(0).read());

    if (*currentIrpStack).MajorFunction == IRP_MJ_READ {
        TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, b"ClassBuildRequest: Read Command\n\0".as_ptr() as PCHAR));

        SrbSetSrbFlags(srbHeader, SRB_FLAGS_DATA_IN);
        (*cdb).CDB10.OperationCode = SCSIOP_READ;
    } else {
        TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, b"ClassBuildRequest: Write Command\n\0".as_ptr() as PCHAR));

        SrbSetSrbFlags(srbHeader, SRB_FLAGS_DATA_OUT);
        (*cdb).CDB10.OperationCode = SCSIOP_WRITE;
    }

    if ((*currentIrpStack).Flags & SL_WRITE_THROUGH) == 0 {
        SrbSetSrbFlags(srbHeader, SRB_FLAGS_ADAPTER_CACHE_ENABLE);
    } else {
        (*cdb).CDB10.ForceUnitAccess = (*FdoExtension).CdbForceUnitAccess;
    }

    if TEST_FLAG((*Irp).Flags, IRP_PAGING_IO | IRP_SYNCHRONOUS_PAGING_IO) != 0 {
        SrbSetSrbFlags(srbHeader, SRB_CLASS_FLAGS_PAGING);
    }

    SrbSetSrbFlags(srbHeader, (*FdoExtension).SrbFlags);

    (*nextIrpStack).MajorFunction = IRP_MJ_SCSI;

    (*nextIrpStack).__bindgen_anon_1.Scsi.Srb = Srb;

    (*currentIrpStack).__bindgen_anon_1.Others.Argument4 = MAXIMUM_RETRIES as PVOID;

    IoSetCompletionRoutine(Irp, Some(ClassIoComplete), Srb as PVOID, 1u8, 1u8, 1u8);
}
