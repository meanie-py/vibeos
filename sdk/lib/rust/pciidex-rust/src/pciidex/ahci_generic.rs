use crate::prelude::*;
use ntkrnl_sys::ddk::{HalGetBusDataByOffset, IoConnectInterrupt, IoDisconnectInterrupt, KeAcquireInterruptSpinLock, KeReleaseInterruptSpinLock, KeStallExecutionProcessor, PCI_COMMON_HEADER, PCI_SLOT_NUMBER, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PDMA_ADAPTER, PDMA_OPERATIONS, PHYSICAL_ADDRESS, PPCI_COMMON_HEADER};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_164 {
    pub VendorID: USHORT,
    pub DeviceID: USHORT,
}

static mut AhciControllerList: [C2RustUnnamed_164; 2] = [
    C2RustUnnamed_164 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: 0x550 as USHORT,
    },
    C2RustUnnamed_164 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: 0x584 as USHORT,
    },
];

#[repr(C)]
pub struct AhciControllerListEntry {
    pub VendorID: u16,
    pub DeviceID: u16,
}

unsafe extern "C" fn AtaAhciAllocateMemory(mut ChannelContext: PVOID) -> NTSTATUS {
    let mut ChanData: PCHANNEL_DATA_AHCI = ChannelContext as PCHANNEL_DATA_AHCI;
    let mut DmaOperations: PDMA_OPERATIONS = (*(*ChanData).__bindgen_anon_1.DmaAdapter).DmaOperations;
    let mut i: ULONG = 0;
    let mut j: ULONG = 0;
    let mut SlotNumber: ULONG = 0;
    let mut CommandSlots: ULONG = 0;
    let mut BlockSize: ULONG = 0;
    let mut CommandListSize: ULONG = 0;
    let mut CommandTableLength: ULONG = 0;
    let mut CommandTablesPerPage: ULONG = 0;
    let mut Buffer: PVOID = core::ptr::null_mut();
    let mut BufferVa: ULONG_PTR = 0;
    let mut BufferPa: ULONG64 = 0;
    let mut PhysicalAddress: PHYSICAL_ADDRESS = PHYSICAL_ADDRESS { QuadPart: 0 };

    PAGED_CODE!();

    CommandSlots = (*(*ChanData).__bindgen_anon_1.Controller).QueueDepth;
    CommandListSize = (FIELD_OFFSET!(AHCI_COMMAND_LIST, CommandHeader[CommandSlots as usize]) as usize) as ULONG;
    BlockSize = CommandListSize.wrapping_add((AHCI_COMMAND_LIST_ALIGNMENT - 1) as ULONG);

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_HAS_FBS == 0 {
        BlockSize = BlockSize.wrapping_add(core::mem::size_of::<AHCI_RECEIVED_FIS>() as ULONG);
        BlockSize = BlockSize.wrapping_add(
            (ALIGN_UP_BY((CommandListSize) as _, (AHCI_RECEIVED_FIS_ALIGNMENT) as _).wrapping_sub((CommandListSize) as _)) as _,
        );
    }

    Buffer = (*DmaOperations)
        .AllocateCommonBuffer
        .unwrap_unchecked()(
        (*ChanData).__bindgen_anon_1.DmaAdapter,
        BlockSize,
        &raw mut PhysicalAddress,
        TRUE,
    );
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlZeroMemory!(Buffer, BlockSize as usize);

    (*ChanData).Mem.CommandListSize = BlockSize;
    (*ChanData).Mem.CommandListOriginal = Buffer;
    (*ChanData).Mem.CommandListPhysOriginal.QuadPart = PhysicalAddress.QuadPart;

    BufferVa = Buffer as ULONG_PTR;
    BufferPa = (PhysicalAddress.QuadPart) as u64;

    BufferVa = (ALIGN_UP_BY((BufferVa) as _, (AHCI_COMMAND_LIST_ALIGNMENT) as _)) as _;
    BufferPa = (ALIGN_UP_BY((BufferPa) as _, (AHCI_COMMAND_LIST_ALIGNMENT) as _)) as _;
    (*ChanData).CommandList = BufferVa as PVOID as PAHCI_COMMAND_LIST;
    (*ChanData).Mem.CommandListPhys = BufferPa;
    BufferVa = BufferVa.wrapping_add(CommandListSize as ULONG_PTR);
    BufferPa = BufferPa.wrapping_add(CommandListSize as ULONG64);

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_HAS_FBS == 0 {
        BufferVa = (ALIGN_UP_BY((BufferVa) as _, (AHCI_RECEIVED_FIS_ALIGNMENT) as _)) as _;
        BufferPa = (ALIGN_UP_BY((BufferPa) as _, (AHCI_RECEIVED_FIS_ALIGNMENT) as _)) as _;
        (*ChanData).ReceivedFis = BufferVa as PVOID as PAHCI_RECEIVED_FIS;
        (*ChanData).Mem.ReceivedFisPhys = BufferPa;
        BufferVa = BufferVa.wrapping_add(core::mem::size_of::<AHCI_RECEIVED_FIS>() as ULONG_PTR);
        BufferPa = BufferPa.wrapping_add(core::mem::size_of::<AHCI_RECEIVED_FIS>() as ULONG64);
    }

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_HAS_FBS != 0 {
        BlockSize = PAGE_SIZE;

        C_ASSERT!(PAGE_SIZE % AHCI_RECEIVED_FIS_FBS_ALIGNMENT == 0);

        Buffer = (*DmaOperations)
            .AllocateCommonBuffer
            .unwrap_unchecked()(
            (*ChanData).__bindgen_anon_1.DmaAdapter,
            BlockSize,
            &raw mut PhysicalAddress,
            TRUE,
        );
        if Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        RtlZeroMemory!(Buffer, BlockSize as usize);

        (*ChanData).Mem.ReceivedFisOriginal = Buffer;
        (*ChanData).Mem.ReceivedFisPhysOriginal.QuadPart = PhysicalAddress.QuadPart;

        BufferVa = Buffer as ULONG_PTR;
        BufferPa = (PhysicalAddress.QuadPart) as u64;

        (*ChanData).ReceivedFis = BufferVa as PVOID as PAHCI_RECEIVED_FIS;
        (*ChanData).Mem.ReceivedFisPhys = BufferPa;
    }

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_64_BIT_DMA == 0 {
    }

    CommandTableLength = (FIELD_OFFSET!(AHCI_COMMAND_TABLE, PrdTable[(*ChanData).__bindgen_anon_1.MaximumPhysicalPages as usize]) as usize) as ULONG;

    CommandTablesPerPage = PAGE_SIZE / (CommandTableLength + (AHCI_COMMAND_TABLE_ALIGNMENT - 1));

    SlotNumber = 0;
    i = CommandSlots;
    while i > 0 {
        let mut TableCount: ULONG;

        TableCount = min(i, CommandTablesPerPage);
        BlockSize = (CommandTableLength + (AHCI_COMMAND_TABLE_ALIGNMENT - 1)) * TableCount;

        Buffer = (*DmaOperations)
            .AllocateCommonBuffer
            .unwrap_unchecked()(
            (*ChanData).__bindgen_anon_1.DmaAdapter,
            BlockSize,
            &raw mut PhysicalAddress,
            TRUE,
        );
        if Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        RtlZeroMemory!(Buffer, BlockSize as usize);

        (*ChanData).Mem.CommandTableOriginal[SlotNumber as usize] = Buffer;
        (*ChanData).Mem.CommandTablePhysOriginal[SlotNumber as usize].QuadPart = PhysicalAddress.QuadPart;
        (*ChanData).Mem.CommandTableSize[SlotNumber as usize] = BlockSize;

        BufferVa = Buffer as ULONG_PTR;
        BufferPa = (PhysicalAddress.QuadPart) as u64;

        j = 0;
        while j < TableCount {
            let mut CommandHeader: PAHCI_COMMAND_HEADER;

            BufferVa = (ALIGN_UP_BY((BufferVa) as _, (AHCI_COMMAND_TABLE_ALIGNMENT as usize))) as _;
            BufferPa = (ALIGN_UP_BY((BufferPa) as _, (AHCI_COMMAND_TABLE_ALIGNMENT as usize))) as _;

            if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_64_BIT_DMA == 0 {
            }

            (*ChanData).CommandTable[SlotNumber as usize] = BufferVa as PAHCI_COMMAND_TABLE;

            CommandHeader = (&raw mut (*(*ChanData).CommandList).CommandHeader as *mut AHCI_COMMAND_HEADER)
                .offset(SlotNumber as isize);
            (*CommandHeader).CommandTableBaseLow = BufferPa as ULONG;
            (*CommandHeader).CommandTableBaseHigh = (BufferPa >> 32) as ULONG;

            SlotNumber = SlotNumber.wrapping_add(1);
            BufferVa = BufferVa.wrapping_add(CommandTableLength as ULONG_PTR);
            BufferPa = BufferPa.wrapping_add(CommandTableLength as ULONG64);
            j = j.wrapping_add(1);
        }

        i = i.wrapping_sub(TableCount);
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaAhciFreeMemory(mut ChannelContext: PVOID) {
    let mut ChanData: PCHANNEL_DATA_AHCI = ChannelContext as PCHANNEL_DATA_AHCI;
    let mut DmaAdapter: PDMA_ADAPTER = (*ChanData).__bindgen_anon_1.DmaAdapter;
    let mut DmaOperations: PDMA_OPERATIONS = (*(*ChanData).__bindgen_anon_1.DmaAdapter).DmaOperations;
    let mut i: ULONG = 0;

    PAGED_CODE!();

    if !(*ChanData).Mem.CommandListOriginal.is_null() {
        (*DmaOperations).FreeCommonBuffer.unwrap_unchecked()(
            DmaAdapter,
            (*ChanData).Mem.CommandListSize,
            (*ChanData).Mem.CommandListPhysOriginal,
            (*ChanData).Mem.CommandListOriginal,
            TRUE,
        );
        (*ChanData).Mem.CommandListOriginal = core::ptr::null_mut();
    }

    if !(*ChanData).Mem.ReceivedFisOriginal.is_null() {
        (*DmaOperations).FreeCommonBuffer.unwrap_unchecked()(
            DmaAdapter,
            PAGE_SIZE,
            (*ChanData).Mem.ReceivedFisPhysOriginal,
            (*ChanData).Mem.ReceivedFisOriginal,
            TRUE,
        );
        (*ChanData).Mem.ReceivedFisOriginal = core::ptr::null_mut();
    }

    i = 0;
    while i < AHCI_MAX_COMMAND_SLOTS {
        if !(*ChanData).Mem.CommandTableOriginal[i as usize].is_null() {
            (*DmaOperations).FreeCommonBuffer.unwrap_unchecked()(
                DmaAdapter,
                (*ChanData).Mem.CommandTableSize[i as usize],
                (*ChanData).Mem.CommandTablePhysOriginal[i as usize],
                (*ChanData).Mem.CommandTableOriginal[i as usize],
                TRUE,
            );
            (*ChanData).Mem.CommandTableOriginal[i as usize] = core::ptr::null_mut();
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn AtaAhciIsVbox() -> i32 {
    let mut Buffer: [u8; RTL_SIZEOF_THROUGH_FIELD!(PCI_COMMON_HEADER, DeviceID)] = [0; RTL_SIZEOF_THROUGH_FIELD!(PCI_COMMON_HEADER, DeviceID)];
    let mut PciData: PPCI_COMMON_HEADER = core::ptr::addr_of_mut!(Buffer) as *mut u8 as PPCI_COMMON_HEADER;
    let mut BytesRead: u32 = 0;
    let mut Slot: PCI_SLOT_NUMBER = PCI_SLOT_NUMBER { u: core::mem::zeroed() };

    if KeGetCurrentIrql() as i32 > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/chipset/ahci_generic.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                270 as u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        }
    }

    Slot.u.AsULONG = 0;
    Slot.u.bits.set_DeviceNumber((4) as _);
    Slot.u.bits.set_FunctionNumber((0) as _);

    BytesRead = HalGetBusDataByOffset(
        PCIConfiguration,
        0,
        Slot.u.AsULONG,
        core::ptr::addr_of_mut!(Buffer) as *mut core::ffi::c_void,
        (FIELD_OFFSET!(PCI_COMMON_HEADER, VendorID)) as ULONG,
        core::mem::size_of_val(&Buffer) as u32,
    );

    return (BytesRead as usize == core::mem::size_of_val(&Buffer) as usize
        && (*PciData).VendorID as i32 == 0x80EE as i32
        && (*PciData).DeviceID as i32 == 0xCAFE as i32) as i32;
}

unsafe extern "C" fn AtaAhciHbaRequestOsOwnership(mut IoBase: PVOID) {
    let mut i: ULONG = 0;
    let mut Control: ULONG = 0;

    PAGED_CODE!();

    Control = AHCI_HBA_READ(IoBase, HbaBiosHandoffControl);
    if Control & AHCI_BOHC_OS_SEMAPHORE != 0 {
        return;
    }

    INFO!(b"HBA ownership change\n\0".as_ptr() as *const i8);

    AHCI_HBA_WRITE(IoBase, HbaBiosHandoffControl, Control | AHCI_BOHC_OS_SEMAPHORE);

    i = 0;
    while i < 200000 {
        Control = AHCI_HBA_READ(IoBase, HbaBiosHandoffControl);

        if Control & (AHCI_BOHC_BIOS_BUSY | AHCI_BOHC_BIOS_SEMAPHORE) == 0 {
            return;
        }

        KeStallExecutionProcessor(10);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn AtaAhciIsPortRemovable(
    mut AhciCapabilities: ULONG,
    mut CmdStatus: ULONG,
) -> BOOLEAN {
    if CmdStatus & AHCI_PXCMD_HPCP != 0 {
        return TRUE;
    }
    if AhciCapabilities & AHCI_CAP_SXS != 0 && CmdStatus & AHCI_PXCMD_ESP != 0 {
        return TRUE;
    }
    if AhciCapabilities & AHCI_CAP_SMPS != 0 && CmdStatus & AHCI_PXCMD_MPSP != 0 {
        return TRUE;
    }
    return FALSE;
}

unsafe extern "C" fn AtaAhciCreateChannelData(Controller: PATA_CONTROLLER) -> NTSTATUS {
    let mut ChanData: PCHANNEL_DATA_AHCI;
    let mut i: ULONG;

    PAGED_CODE!();

    ChanData = (ExAllocatePoolZero(
        NonPagedPool,
        ((core::mem::size_of::<CHANNEL_DATA_AHCI>() as ULONG) * (*Controller).MaxChannels) as _,
        TAG_PCIIDEX,
    )).cast();
    if ChanData.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Controller).ChanDataBlock = ChanData as PVOID;

    i = 0;
    while i < AHCI_MAX_PORTS {
        let mut CmdStatus: ULONG;

        if ((*Controller).ChannelBitmap & (1 << i)) == 0 {
            i = i.wrapping_add(1);
            continue;
        }

        (*Controller).Channels[i as usize] = ChanData as PVOID;

        (*ChanData).__bindgen_anon_1.Channel = i;
        (*ChanData).__bindgen_anon_1.Controller = Controller;

        (*ChanData).__bindgen_anon_1.AllocateMemory = Some(AtaAhciAllocateMemory);
        (*ChanData).__bindgen_anon_1.FreeMemory = Some(AtaAhciFreeMemory);
        (*ChanData).__bindgen_anon_1.EnableInterrupts = Some(AtaAhciEnableInterrupts);
        (*ChanData).__bindgen_anon_1.PreparePrdTable = Some(AtaAhciPreparePrdTable);
        (*ChanData).__bindgen_anon_1.PrepareIo = Some(AtaAhciPrepareIo);
        (*ChanData).__bindgen_anon_1.StartIo = Some(AtaAhciStartIo as unsafe extern "C" fn(PVOID, PATA_DEVICE_REQUEST) -> BOOLEAN);
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(SataSetTransferMode);
        (*ChanData).__bindgen_anon_1.TransferModeSupported = SATA_ALL;

        (*ChanData).IoBase = AHCI_PORT_BASE!((*Controller).IoBase, i) as PVOID;

        (((*ChanData).__bindgen_anon_1.EnableInterrupts).unwrap_unchecked())(ChanData as PVOID, FALSE);

        CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
        if (CmdStatus & AHCI_PXCMD_ST) != 0 {
            CmdStatus &= !AHCI_PXCMD_ST;
            AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
        }

        (*ChanData).__bindgen_anon_1.ChanInfo = CHANNEL_FLAG_PIO_VIA_DMA;

        if ((*Controller).AhciCapabilities & AHCI_CAP_S64A) != 0 {
            (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_64_BIT_DMA;
        }

        if ((*Controller).AhciCapabilities & AHCI_CAP_SNCQ) != 0 {
            (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_HAS_NCQ;
        }

        if ((*Controller).AhciCapabilities & AHCI_CAP_SPM) != 0
            && ((*Controller).AhciCapabilities & AHCI_CAP_FBSS) != 0
            && (CmdStatus & AHCI_PXCMD_FBSCP) != 0
        {
            INFO!("CH %lu: FBS supported\n", (*ChanData).Channel);
            (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_HAS_FBS;
        }

        if AtaAhciIsPortRemovable((*Controller).AhciCapabilities, CmdStatus) != 0 {
            INFO!("CH %lu: Port is external\n", (*ChanData).Channel);
            (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_IS_EXTERNAL;
        }

        ChanData = ChanData.offset(1);
        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaAhciGetAbar(mut Controller: PATA_CONTROLLER) -> PVOID {
    let mut Abar: PVOID = core::ptr::null_mut();
    let mut i: ULONG = 0;
    let mut Index: ULONG = 0;
    // ahci_generic.c's table is an anonymous `static const struct { USHORT VendorID;
    // USHORT DeviceID; ULONG Index; }`; this is that struct, named for Rust.
    #[derive(Copy, Clone)]
    #[repr(C)]
    struct AbarLocation {
        VendorID: USHORT,
        DeviceID: USHORT,
        Index: ULONG,
    }
    static mut AbarLocations: [AbarLocation; 1] = [AbarLocation {
        VendorID: PCI_VEN_CAVIUM as USHORT,
        DeviceID: 0xA01C,
        Index: 0,
    }];

    PAGED_CODE!();

    Index = 5;

    i = 0;
    while (i as usize) < RTL_NUMBER_OF!(AbarLocations) as usize {
        if (*Controller).Pci.VendorID == AbarLocations[i as usize].VendorID
            && (*Controller).Pci.DeviceID == AbarLocations[i as usize].DeviceID
        {
            Index = AbarLocations[i as usize].Index;
            break;
        }
        i = i.wrapping_add(1);
    }

    if (*Controller).AccessRange[Index as usize].Flags & RANGE_IS_MEMORY == 0 {
        return core::ptr::null_mut();
    }

    Abar = AtaCtrlPciMapBar(Controller, Index, 0);
    if Abar.is_null() {
        return core::ptr::null_mut();
    }

    return Abar;
}

unsafe extern "C" fn AtaAhciAttachChannel(mut ChannelContext: PVOID, mut Attach: BOOLEAN) -> NTSTATUS {
    let mut ChanData: PCHANNEL_DATA_AHCI = ChannelContext as PCHANNEL_DATA_AHCI;
    PAGED_CODE!();
    if !(Attach != 0) {
        AtaChanEnableInterruptsSync(ChanData as PVOID, FALSE as BOOLEAN);
        AtaAhciStopDma(ChanData);
    }
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaAhciHbaStart(Controller: PATA_CONTROLLER) {
    let mut GlobalControl: ULONG;

    GlobalControl = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaGlobalControl);
    if !(GlobalControl & AHCI_GHC_AE) != 0 {
        GlobalControl |= AHCI_GHC_AE;
        AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaGlobalControl, GlobalControl);
    }

    AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaInterruptStatus, 0xFFFFFFFF);

    GlobalControl |= AHCI_GHC_IE;
    AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaGlobalControl, GlobalControl);
}

unsafe extern "C" fn AtaAhciHbaStop(mut Controller: PATA_CONTROLLER) {
    let mut GlobalControl: ULONG = 0;
    let mut OldIrql: KIRQL = 0;
    if (*Controller).InterruptObject.is_null() {
        return;
    }
    OldIrql = KeAcquireInterruptSpinLock((*Controller).InterruptObject);
    GlobalControl = AHCI_HBA_READ((*Controller).IoBase as PVOID, HbaGlobalControl);
    GlobalControl &= !AHCI_GHC_IE as ULONG;
    AHCI_HBA_WRITE(
        (*Controller).IoBase as PVOID,
        HbaGlobalControl,
        GlobalControl,
    );
    AHCI_HBA_WRITE(
        (*Controller).IoBase as PVOID,
        HbaInterruptStatus,
        0xFFFFFFFF as ULONG,
    );
    KeReleaseInterruptSpinLock((*Controller).InterruptObject, OldIrql);
}

unsafe extern "C" fn AtaAhciHbaFreeResouces(mut Controller: PATA_CONTROLLER) {
    PAGED_CODE!();
    if !(*Controller).InterruptObject.is_null() {
        IoDisconnectInterrupt((*Controller).InterruptObject);
        (*Controller).InterruptObject = core::ptr::null_mut();
    }
}

unsafe extern "C" fn AhciControllerIsSubClassCheckNeeded(
    mut Controller: PATA_CONTROLLER,
) -> BOOLEAN {
    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/storage/ide/pciidex/chipset/ahci_generic.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                564 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    if (*Controller).Pci.VendorID as ::core::ffi::c_int == (PCI_VEN_INTEL as i32) {
        if (*Controller).Pci.DeviceID as ::core::ffi::c_int == 0x2652
            || (*Controller).Pci.DeviceID as ::core::ffi::c_int == 0x2653
        {
            return TRUE;
        }
    }
    if (*Controller).Pci.VendorID as ::core::ffi::c_int == (PCI_VEN_VIA as i32)
        && (*Controller).Pci.DeviceID as ::core::ffi::c_int == 0x3349
    {
        return TRUE;
    }
    return FALSE;
}

unsafe extern "C" fn AhciMatchController(Controller: PATA_CONTROLLER) -> BOOLEAN {
    let mut i: ULONG;

    PAGED_CODE!();

    if AhciControllerIsSubClassCheckNeeded(Controller) != 0 {
        if (*Controller).Pci.SubClass != (PCI_SUBCLASS_MSC_AHCI_CTLR as u8) {
            return FALSE;
        }
    }

    i = 0;
    while i < RTL_NUMBER_OF!(AhciControllerList) {
        if (*Controller).Pci.VendorID == AhciControllerList[i as usize].VendorID
            && (*Controller).Pci.DeviceID == AhciControllerList[i as usize].DeviceID
        {
            return TRUE;
        }
        i = i.wrapping_add(1);
    }

    if (*Controller).Pci.BaseClass == (PCI_CLASS_MASS_STORAGE_CTLR as u8)
        && (*Controller).Pci.SubClass == (PCI_SUBCLASS_MSC_AHCI_CTLR as u8)
    {
        return TRUE;
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn AhciGetControllerProperties(
    Controller: PATA_CONTROLLER,
) -> NTSTATUS {
    let InterruptDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR = &raw mut (*Controller).InterruptDesc;
    let mut i: ULONG;
    let mut GlobalControl: ULONG;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    if ((AhciMatchController(Controller)) == 0) {
        return STATUS_NO_MATCH;
    }

    if (*Controller).InterruptDesc.Type != CmResourceTypeInterrupt {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Controller).IoBase = (AtaAhciGetAbar(Controller)).cast();
    if (*Controller).IoBase.is_null() {
        return STATUS_NO_MATCH;
    }

    (*Controller).Flags |= CTRL_FLAG_IS_AHCI | CTRL_FLAG_SATA_HBA_ACPI;
    (*Controller).Start = Some(AtaAhciHbaStart);
    (*Controller).Stop = Some(AtaAhciHbaStop);
    (*Controller).FreeResources =
        Some(AtaAhciHbaFreeResouces);
    (*Controller).AttachChannel =
        Some(AtaAhciAttachChannel);

    GlobalControl = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaGlobalControl);
    GlobalControl |= AHCI_GHC_AE;
    AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaGlobalControl, GlobalControl);

    (*Controller).AhciCapabilities = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaCapabilities);
    (*Controller).ChannelBitmap = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaPortBitmap);
    (*Controller).MaxChannels = CountSetBits((*Controller).ChannelBitmap);
    if (*Controller).MaxChannels == 0 {
        return STATUS_DEVICE_HARDWARE_ERROR;
    }

    (*Controller).AhciVersion = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaAhciVersion);
    if (*Controller).AhciVersion >= AHCI_VERSION_1_2 {
        (*Controller).AhciCapabilitiesEx =
            AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaCapabilitiesEx);

        if (*Controller).AhciCapabilitiesEx & AHCI_CAP2_BOH != 0 {
            AtaAhciHbaRequestOsOwnership(((*Controller).IoBase).cast());
        }
    }

    GlobalControl = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaGlobalControl);
    GlobalControl |= AHCI_GHC_HR;
    AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaGlobalControl, GlobalControl);

    i = 100000;
    while i > 0 {
        GlobalControl = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaGlobalControl);
        if GlobalControl & AHCI_GHC_HR == 0 {
            break;
        }

        KeStallExecutionProcessor(10);
        i = i.wrapping_sub(1);
    }
    if i == 0 {
        return STATUS_IO_TIMEOUT;
    }

    GlobalControl |= AHCI_GHC_AE;
    AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaGlobalControl, GlobalControl);

    GlobalControl = AHCI_HBA_READ(((*Controller).IoBase).cast(), HbaGlobalControl);
    if GlobalControl & AHCI_GHC_IE != 0 {
        GlobalControl &= !AHCI_GHC_IE;
        AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaGlobalControl, GlobalControl);
    }

    if AtaAhciIsVbox() != 0 {
        (*Controller).AhciCapabilities |= AHCI_CAP_SPM;
    }

    Status = AtaAhciCreateChannelData(Controller);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    (*Controller).QueueDepth = (((*Controller).AhciCapabilities & AHCI_CAP_NCS)
        >> 8)
        .wrapping_add(1);

    INFO!(
        b"%04X:%04X.%02X: Ver %08lX, PI %08lX, CAP %08lX, CAP2 %08lX\n\0".as_ptr() as *const i8,
        (*Controller).Pci.VendorID,
        (*Controller).Pci.DeviceID,
        (*Controller).Pci.RevisionID,
        (*Controller).AhciVersion,
        (*Controller).ChannelBitmap,
        (*Controller).AhciCapabilities,
        (*Controller).AhciCapabilitiesEx,
    );

    AHCI_HBA_WRITE(((*Controller).IoBase).cast(), HbaInterruptStatus, 0xFFFFFFFF);

    Status = IoConnectInterrupt(
        &raw mut (*Controller).InterruptObject,
        Some(AtaAhciHbaIsr),
        Controller as PVOID,
        core::ptr::null_mut(),
        (*InterruptDesc).u.Interrupt.Vector,
        (*InterruptDesc).u.Interrupt.Level as KIRQL,
        (*InterruptDesc).u.Interrupt.Level as KIRQL,
        if (*InterruptDesc).Flags & (CM_RESOURCE_INTERRUPT_LATCHED as u16) != 0 {
            Latched
        } else {
            LevelSensitive
        },
        (((*InterruptDesc).ShareDisposition) as u32 == CmResourceShareShared) as BOOLEAN,
        (*InterruptDesc).u.Interrupt.Affinity,
        FALSE,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    return STATUS_SUCCESS;
}
