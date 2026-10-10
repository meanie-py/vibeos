use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::_INTERFACE_TYPE as INTERFACE_TYPE;
// PCI_COMMON_CONFIG / PPCI_COMMON_CONFIG come from glue (ntkrnl_sys's ddk stub lacks the
// header fields VendorID/DeviceID/RevisionID/u that this driver reads).
use core::ffi::c_void;

extern "system" {
    fn StorPortGetPhysicalAddress( HwDeviceExtension: PVOID, Srb: PSCSI_REQUEST_BLOCK, VirtualAddress: PVOID, Length: *mut ULONG, ) -> STOR_PHYSICAL_ADDRESS;
    fn StorPortReadRegisterUlong( HwDeviceExtension: PVOID, Register: *mut ULONG, ) -> ULONG;
    fn StorPortStallExecution(Delay: ULONG);
    fn StorPortWriteRegisterUlong(HwDeviceExtension: PVOID, Register: *mut ULONG, Value: ULONG);
    fn StorPortInitialize( DriverObject: PVOID, RegistryPath: PVOID, HwInitializationData: *mut HW_INITIALIZATION_DATA, Unused: PVOID, ) -> ULONG;
    fn StorPortSetDeviceQueueDepth( HwDeviceExtension: PVOID, PathId: u8, TargetId: u8, Lun: u8, Depth: u32, ) -> BOOLEAN;
    fn StorPortGetScatterGatherList( AdapterExtension: PAHCI_ADAPTER_EXTENSION, Srb: PSCSI_REQUEST_BLOCK, ) -> *mut core::ffi::c_void;
    fn StorPortGetUncachedExtension( AdapterExtension: PAHCI_ADAPTER_EXTENSION, ConfigInfo: PPORT_CONFIGURATION_INFORMATION, NumberOfBytes: ULONG, ) -> PCHAR;
    fn StorPortGetBusData( DeviceExtension: PVOID, BusDataType: u32, SystemIoBusNumber: ULONG, SlotNumber: ULONG, Buffer: PVOID, Length: ULONG, ) -> ULONG;
    fn StorPortGetDeviceBase( DeviceExtension: PVOID, BusType: INTERFACE_TYPE, SystemIoBusNumber: ULONG, IoAddress: STOR_PHYSICAL_ADDRESS, NumberOfBytes: ULONG, InIoSpace: UCHAR, ) -> PVOID;
}
extern "C" {
    fn StorPortNotification( NotificationType: u32, HwDeviceExtension: PVOID, ... ) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// (no file-scope declarations)


const MAXIMUM_AHCI_PRDT_ENTRIES: ULONG = 0x20;


#[no_mangle]
pub unsafe extern "system" fn AhciPortInitialize(DeviceExtension: PVOID) -> BOOLEAN {

    let PortExtension: PAHCI_PORT_EXTENSION;
    let mut cmd: AHCI_PORT_CMD = core::mem::zeroed();
    let abar: PAHCI_MEMORY_REGISTERS;
    let mut mappedLength: ULONG = 0;
    let portNumber: ULONG;
    let mut ticks: ULONG;
    let adapterExtension: PAHCI_ADAPTER_EXTENSION;
    let commandListPhysical: STOR_PHYSICAL_ADDRESS;
    let receivedFISPhysical: STOR_PHYSICAL_ADDRESS;

    AhciDebugPrint!(b"AhciPortInitialize()\n\0".as_ptr() as *const i8);

    PortExtension = DeviceExtension as PAHCI_PORT_EXTENSION;
    adapterExtension = (*PortExtension).AdapterExtension;
    abar = (*adapterExtension).ABAR_Address;
    portNumber = (*PortExtension).PortNumber;

    NT_ASSERT((!abar.is_null()) as BOOLEAN);
    NT_ASSERT((portNumber < (*adapterExtension).PortCount) as BOOLEAN);

    (*PortExtension).Port = core::ptr::addr_of_mut!((*abar).PortList[portNumber as usize]);

    commandListPhysical = StorPortGetPhysicalAddress(
        adapterExtension as PVOID,
        core::ptr::null_mut(),
        (*PortExtension).CommandList as PVOID,
        core::ptr::addr_of_mut!(mappedLength),
    );

    if (mappedLength == 0) || ((commandListPhysical.__bindgen_anon_1.LowPart % 1024) != 0) {
        AhciDebugPrint!(
            b"\tcommandListPhysical mappedLength:%d\n\0".as_ptr() as *const i8,
            mappedLength,
        );
        return 0;
    }

    receivedFISPhysical = StorPortGetPhysicalAddress(
        adapterExtension as PVOID,
        core::ptr::null_mut(),
        (*PortExtension).ReceivedFIS as PVOID,
        core::ptr::addr_of_mut!(mappedLength),
    );

    if (mappedLength == 0) || ((receivedFISPhysical.__bindgen_anon_1.LowPart % 256) != 0) {
        AhciDebugPrint!(
            b"\treceivedFISPhysical mappedLength:%d\n\0".as_ptr() as *const i8,
            mappedLength,
        );
        return 0;
    }

    cmd.set_status(StorPortReadRegisterUlong(
        adapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
    ));
    if (cmd.FR() != 0) || (cmd.CR() != 0) || (cmd.FRE() != 0) || (cmd.ST() != 0) {
        cmd.set_ST(0);
        cmd.set_FRE(0);

        ticks = 3;
        loop {
            StorPortStallExecution(50000);
            cmd.set_status(StorPortReadRegisterUlong(
                adapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
            ));
            if ticks == 0 {
                AhciDebugPrint!(b"\tAttempt to reset port failed: %x\n\0".as_ptr() as *const i8, cmd);
                return 0;
            }
            ticks -= 1;

            if !(cmd.CR() != 0 || cmd.FR() != 0) {
                break;
            }
        }
    }

    StorPortWriteRegisterUlong(
        adapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).CLB),
        commandListPhysical.__bindgen_anon_1.LowPart,
    );
    if IsAdapterCAPS64((*adapterExtension).CAP) != 0 {
        StorPortWriteRegisterUlong(
            adapterExtension as PVOID,
            core::ptr::addr_of_mut!((*(*PortExtension).Port).CLBU),
            commandListPhysical.__bindgen_anon_1.HighPart as ULONG,
        );
    }

    StorPortWriteRegisterUlong(
        adapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).FB),
        receivedFISPhysical.__bindgen_anon_1.LowPart,
    );
    if IsAdapterCAPS64((*adapterExtension).CAP) != 0 {
        StorPortWriteRegisterUlong(
            adapterExtension as PVOID,
            core::ptr::addr_of_mut!((*(*PortExtension).Port).FBU),
            receivedFISPhysical.__bindgen_anon_1.HighPart as ULONG,
        );
    }

    (*PortExtension).IdentifyDeviceDataPhysicalAddress = StorPortGetPhysicalAddress(
        adapterExtension as PVOID,
        core::ptr::null_mut(),
        (*PortExtension).IdentifyDeviceData as PVOID,
        core::ptr::addr_of_mut!(mappedLength),
    );

    (*PortExtension).DevicePowerState = StorPowerDeviceD0;

    StorPortWriteRegisterUlong(
        adapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).SERR),
        !0u32 as ULONG,
    );
    StorPortWriteRegisterUlong(
        adapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).IS),
        !0u32 as ULONG,
    );
    StorPortWriteRegisterUlong(
        adapterExtension as PVOID,
        (*adapterExtension).IS,
        (1u32 << (*PortExtension).PortNumber) as ULONG,
    );

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn AhciAllocateResourceForAdapter(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
) -> BOOLEAN {

    const MAXIMUM_AHCI_PORT_COUNT: ULONG = 32;
    const FALSE: BOOLEAN = 0;
    const TRUE: BOOLEAN = 1;

    let mut nonCachedExtension: PCHAR;
    let mut tmp: PCHAR;
    let mut index: ULONG;
    let NCS: ULONG;
    let AlignedNCS: ULONG;
    let mut portCount: ULONG;
    let portImplemented: ULONG;
    let nonCachedExtensionSize: ULONG;
    let mut PortExtension: PAHCI_PORT_EXTENSION;

    AhciDebugPrint!(b"AhciAllocateResourceForAdapter()\n\0".as_ptr() as *const i8);

    NCS = AHCI_Global_Port_CAP_NCS((*AdapterExtension).CAP);
    AlignedNCS = crate::glue::ROUND_UP(NCS, 8);

    // get port count -- Number of set bits in `AdapterExtension->PortImplemented`
    portCount = 0;
    portImplemented = (*AdapterExtension).PortImplemented;

    NT_ASSERT((portImplemented != 0) as BOOLEAN);
    index = MAXIMUM_AHCI_PORT_COUNT - 1;
    while index > 0 {
        if (portImplemented & (1 << index)) != 0 {
            break;
        }
        index -= 1;
    }

    portCount = index + 1;
    AhciDebugPrint!(b"\tPort Count: %d\n\0".as_ptr() as *const i8, portCount);

    (*AdapterExtension).PortCount = portCount;
    nonCachedExtensionSize = (core::mem::size_of::<AHCI_COMMAND_HEADER>() as ULONG) * AlignedNCS + //should be 1K aligned
                             (core::mem::size_of::<AHCI_RECEIVED_FIS>() as ULONG) +
                             (core::mem::size_of::<IDENTIFY_DEVICE_DATA>() as ULONG);

    // align nonCachedExtensionSize to 1024
    let nonCachedExtensionSize = crate::glue::ROUND_UP(nonCachedExtensionSize, 1024);

    (*AdapterExtension).NonCachedExtension = StorPortGetUncachedExtension(
        AdapterExtension,
        ConfigInfo,
        nonCachedExtensionSize * portCount,
    ) as PVOID;

    if (*AdapterExtension).NonCachedExtension == core::ptr::null_mut() {
        AhciDebugPrint!(b"\tadapterExtension->NonCachedExtension == NULL\n\0".as_ptr() as *const i8);
        return FALSE;
    }

    nonCachedExtension = (*AdapterExtension).NonCachedExtension as PCHAR;
    AhciZeroMemory(nonCachedExtension, nonCachedExtensionSize * portCount);

    index = 0;
    while index < portCount {
        PortExtension = core::ptr::addr_of_mut!((*AdapterExtension).PortExtension[index as usize]);

        (*PortExtension).DeviceParams.IsActive = FALSE;
        if ((*AdapterExtension).PortImplemented & (1 << index)) != 0 {
            (*PortExtension).PortNumber = index;
            (*PortExtension).DeviceParams.IsActive = TRUE;
            (*PortExtension).AdapterExtension = AdapterExtension;
            (*PortExtension).CommandList = nonCachedExtension as PAHCI_COMMAND_HEADER;

            tmp = (nonCachedExtension.offset(
                (core::mem::size_of::<AHCI_COMMAND_HEADER>() as ULONG * AlignedNCS) as isize,
            )) as PCHAR;

            (*PortExtension).ReceivedFIS = tmp as PAHCI_RECEIVED_FIS;
            (*PortExtension).IdentifyDeviceData =
                (tmp.offset(core::mem::size_of::<AHCI_RECEIVED_FIS>() as isize)) as PIDENTIFY_DEVICE_DATA;
            (*PortExtension).MaxPortQueueDepth = NCS;
            nonCachedExtension = nonCachedExtension.offset(nonCachedExtensionSize as isize);
        }
        index += 1;
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn AhciStartPort(
    PortExtension: PAHCI_PORT_EXTENSION,
) -> BOOLEAN {

    let mut index: ULONG;
    let mut cmd: AHCI_PORT_CMD = core::mem::zeroed();
    let mut tfd: AHCI_TASK_FILE_DATA = core::mem::zeroed();
    let mut ie: AHCI_INTERRUPT_ENABLE = core::mem::zeroed();
    let mut ssts: AHCI_SERIAL_ATA_STATUS = core::mem::zeroed();
    let mut sctl: AHCI_SERIAL_ATA_CONTROL = core::mem::zeroed();
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;

    AhciDebugPrint!(b"AhciStartPort()\n\0".as_ptr() as *const i8);

    AdapterExtension = (*PortExtension).AdapterExtension;
    cmd.set_status(StorPortReadRegisterUlong(
        AdapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
    ));

    if (cmd.FR() == 1) && (cmd.CR() == 1) && (cmd.FRE() == 1) && (cmd.ST() == 1) {
        // Already Running
        return 1;
    }

    cmd.set_SUD(1);
    StorPortWriteRegisterUlong(
        AdapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
        cmd.status(),
    );

    if ((cmd.FR() == 1) && (cmd.FRE() == 0)) || ((cmd.CR() == 1) && (cmd.ST() == 0)) {
        AhciDebugPrint!(b"\tCOMRESET\n\0".as_ptr() as *const i8);

        sctl.set_status(StorPortReadRegisterUlong(
            AdapterExtension as PVOID,
            core::ptr::addr_of_mut!((*(*PortExtension).Port).SCTL),
        ));
        sctl.set_DET(1);
        StorPortWriteRegisterUlong(
            AdapterExtension as PVOID,
            core::ptr::addr_of_mut!((*(*PortExtension).Port).SCTL),
            sctl.status(),
        );

        StorPortStallExecution(1000);

        sctl.set_status(StorPortReadRegisterUlong(
            AdapterExtension as PVOID,
            core::ptr::addr_of_mut!((*(*PortExtension).Port).SCTL),
        ));
        sctl.set_DET(0);
        StorPortWriteRegisterUlong(
            AdapterExtension as PVOID,
            core::ptr::addr_of_mut!((*(*PortExtension).Port).SCTL),
            sctl.status(),
        );

        // Poll DET to verify if a device is attached to the port
        index = 0;
        loop {
            StorPortStallExecution(1000);
            ssts.set_status(StorPortReadRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).SSTS),
            ));

            index += 1;
            if ssts.DET() != 0 {
                break;
            }
            if !(index < 30) {
                break;
            }
        }
    }

    ssts.set_status(StorPortReadRegisterUlong(
        AdapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).SSTS),
    ));
    match ssts.DET() {
        0x3 => {
            NT_ASSERT(cmd.ST() == 0);

            // make sure FIS Recieve is enabled (cmd.FRE)
            index = 0;
            loop {
                StorPortStallExecution(10000);
                cmd.set_status(StorPortReadRegisterUlong(
                    AdapterExtension as PVOID,
                    core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
                ));
                cmd.set_FRE(1);
                StorPortWriteRegisterUlong(
                    AdapterExtension as PVOID,
                    core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
                    cmd.status(),
                );
                index += 1;
                if !((cmd.FR() != 1) && (index < 3)) {
                    break;
                }
            }

            if cmd.FR() != 1 {
                // failed to start FIS DMA engine
                return 0;
            }

            // start port channel
            // set cmd.ST

            NT_ASSERT(cmd.FRE() == 1);
            NT_ASSERT(cmd.CR() == 0);

            tfd.set_status(StorPortReadRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).TFD),
            ));

            if (tfd.__bindgen_anon_1.as_ref().STS.BSY() != 0) || (tfd.__bindgen_anon_1.as_ref().STS.DRQ() != 0) {
                AhciDebugPrint!(b"\tUnhandled Case BSY-DRQ\n\0".as_ptr() as *const i8);
            }

            // clear pending interrupts
            StorPortWriteRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).SERR),
                !0u32 as ULONG,
            );
            StorPortWriteRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).IS),
                !0u32 as ULONG,
            );
            StorPortWriteRegisterUlong(
                AdapterExtension as PVOID,
                (*AdapterExtension).IS,
                (1u32 << (*PortExtension).PortNumber) as ULONG,
            );

            // set IE
            ie.set_status(StorPortReadRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).IE),
            ));
            ie.set_DHRE(1);
            ie.set_PSE(1);
            ie.set_DSE(1);
            ie.set_SDBE(1);
            ie.set_UFE(0);
            ie.set_DPE(0);
            ie.set_PCE(1);
            ie.set_DMPE(0);
            ie.set_PRCE(1);
            ie.set_IPME(0);
            ie.set_OFE(1);
            ie.set_INFE(1);
            ie.set_IFE(1);
            ie.set_HBDE(1);
            ie.set_HBFE(1);
            ie.set_TFEE(1);

            cmd.set_status(StorPortReadRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
            ));
            if cmd.CPD() != 0 {
                // disable it for now
                ie.set_CPDE(0);
            }

            StorPortWriteRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).IE),
                ie.status(),
            );

            cmd.set_ST(1);
            StorPortWriteRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
                cmd.status(),
            );
            cmd.set_status(StorPortReadRegisterUlong(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
            ));

            if cmd.ST() != 1 {
                AhciDebugPrint!(b"\tFailed to start Port\n\0".as_ptr() as *const i8);
                return 0;
            }

            return 1;
        }
        _ => {
            // unhandled case
            AhciDebugPrint!(
                b"\tDET == %x Unsupported\n\0".as_ptr() as *const i8,
                ssts.DET(),
            );
            return 0;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AhciCommandCompletionDpcRoutine(
    Dpc: PSTOR_DPC,
    HwDeviceExtension: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {

    let Srb: PSCSI_REQUEST_BLOCK;
    let SrbExtension: PAHCI_SRB_EXTENSION;
    let mut lockhandle: STOR_LOCK_HANDLE = core::mem::zeroed();
    let CompletionRoutine: PAHCI_COMPLETION_ROUTINE;
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;
    let PortExtension: PAHCI_PORT_EXTENSION;

    let _ = Dpc;
    let _ = SystemArgument2;

    AhciDebugPrint!(b"AhciCommandCompletionDpcRoutine()\n\0".as_ptr() as *const i8);

    AdapterExtension = HwDeviceExtension as PAHCI_ADAPTER_EXTENSION;
    PortExtension = SystemArgument1 as PAHCI_PORT_EXTENSION;

    StorPortAcquireSpinLock(
        AdapterExtension as PVOID,
        InterruptLock,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(lockhandle),
    );
    Srb = RemoveQueue(core::ptr::addr_of_mut!((*PortExtension).CompletionQueue)) as PSCSI_REQUEST_BLOCK;
    StorPortReleaseSpinLock(AdapterExtension as PVOID, core::ptr::addr_of_mut!(lockhandle));

    // NT_ASSERT(Srb != NULL);

    if (*Srb).SrbStatus as u32 == SRB_STATUS_PENDING {
        (*Srb).SrbStatus = SRB_STATUS_SUCCESS as u8;
    } else {
        return;
    }

    SrbExtension = GetSrbExtension(Srb);

    CompletionRoutine = (*SrbExtension).CompletionRoutine;
    // NT_ASSERT(CompletionRoutine != NULL);

    // now it's completion routine responsibility to set SrbStatus
    (CompletionRoutine.unwrap())(PortExtension as PVOID, Srb as PVOID);

    StorPortNotification(RequestComplete, AdapterExtension as PVOID, Srb);

    return;
}
#[no_mangle]
pub unsafe extern "C" fn AhciHwPassiveInitialize(
    DeviceExtension: PVOID,
) -> BOOLEAN {

    let mut index: ULONG;
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;
    let mut PortExtension: PAHCI_PORT_EXTENSION;

    AhciDebugPrint!(b"AhciHwPassiveInitialize()\n\0".as_ptr() as *const i8);

    AdapterExtension = DeviceExtension as PAHCI_ADAPTER_EXTENSION;

    index = 0;
    while index < (*AdapterExtension).PortCount {
        if ((*AdapterExtension).PortImplemented & (0x1 << index)) != 0 {
            PortExtension = core::ptr::addr_of_mut!((*AdapterExtension).PortExtension[index as usize]);
            (*PortExtension).DeviceParams.IsActive = AhciStartPort(PortExtension);
            StorPortInitializeDpc(
                AdapterExtension as PVOID,
                core::ptr::addr_of_mut!((*PortExtension).CommandCompletion),
                core::mem::transmute::<_, PHW_DPC_ROUTINE>(AhciCommandCompletionDpcRoutine as *const ()),
            );
        }
        index += 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AhciHwInitialize(
    DeviceExtension: PVOID,
) -> BOOLEAN {

    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;
    let mut ghc: AHCI_GHC = core::mem::zeroed();

    AhciDebugPrint!(b"AhciHwInitialize()\n\0".as_ptr() as *const i8);

    AdapterExtension = DeviceExtension as PAHCI_ADAPTER_EXTENSION;
    (*AdapterExtension).StateFlags.set_MessagePerPort(0);

    // First check what type of interrupt/synchronization device is using
    ghc.set_status(StorPortReadRegisterUlong(
        AdapterExtension as PVOID,
        core::ptr::addr_of_mut!((*(*AdapterExtension).ABAR_Address).GHC),
    ));

    // When set to '1' by hardware, indicates that the HBA requested more than one MSI vector
    // but has reverted to using the first vector only.  When this bit is cleared to '0',
    // the HBA has not reverted to single MSI mode (i.e. hardware is already in single MSI mode,
    // software has allocated the number of messages requested
    if ghc.MRSM() == 0 {
        (*AdapterExtension).StateFlags.set_MessagePerPort(1);
        AhciDebugPrint!(b"\tMultiple MSI based message not supported\n\0".as_ptr() as *const i8);
    }

    StorPortEnablePassiveInitialization(
        AdapterExtension as PVOID,
        AhciHwPassiveInitialize as PVOID,
    );

    1
}

#[no_mangle]
pub unsafe extern "C" fn AhciCompleteIssuedSrb(
    PortExtension: PAHCI_PORT_EXTENSION,
    CommandsToComplete: ULONG,
) {

    let mut NCS: ULONG;
    let mut i: ULONG;
    let mut Srb: PSCSI_REQUEST_BLOCK;
    let mut SrbExtension: PAHCI_SRB_EXTENSION;
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;

    AhciDebugPrint!(b"AhciCompleteIssuedSrb()\n\0".as_ptr() as *const i8);

    // NT_ASSERT(CommandsToComplete != 0);

    AhciDebugPrint!(
        b"\tCompleted Commands: %d\n\0".as_ptr() as *const i8,
        CommandsToComplete,
    );

    AdapterExtension = (*PortExtension).AdapterExtension;
    NCS = AHCI_Global_Port_CAP_NCS((*AdapterExtension).CAP);

    i = 0;
    while i < NCS {
        if (((1 << i) & CommandsToComplete) != 0) {
            Srb = (*PortExtension).Slot[i as usize];

            if Srb.is_null() {
                i = i.wrapping_add(1);
                continue;
            }

            SrbExtension = GetSrbExtension(Srb);
            // NT_ASSERT(SrbExtension != NULL);

            if (*SrbExtension).CompletionRoutine.is_some() {
                AddQueue(
                    core::ptr::addr_of_mut!((*PortExtension).CompletionQueue),
                    Srb as PVOID,
                );
                StorPortIssueDpc(
                    AdapterExtension as PVOID,
                    core::ptr::addr_of_mut!((*PortExtension).CommandCompletion) as PVOID,
                    PortExtension as PVOID,
                    Srb as PVOID,
                );
            } else {
                // NT_ASSERT(Srb->SrbStatus == SRB_STATUS_PENDING);
                (*Srb).SrbStatus = SRB_STATUS_SUCCESS as u8;
                StorPortNotification(RequestComplete, AdapterExtension as PVOID, Srb);
            }
        }

        i = i.wrapping_add(1);
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn AhciInterruptHandler(PortExtension: PAHCI_PORT_EXTENSION) {

    let mut is: ULONG;
    let mut ci: ULONG;
    let mut sact: ULONG;
    let mut outstanding: ULONG;
    let mut PxIS: AHCI_INTERRUPT_STATUS = core::mem::zeroed();
    let mut PxISMasked: AHCI_INTERRUPT_STATUS = core::mem::zeroed();
    let mut AdapterExtension: PAHCI_ADAPTER_EXTENSION;

    AhciDebugPrint!(b"AhciInterruptHandler()\n\0".as_ptr() as *const i8);
    AhciDebugPrint!(
        b"\tPort Number: %d\n\0".as_ptr() as *const i8,
        (*PortExtension).PortNumber,
    );

    AdapterExtension = (*PortExtension).AdapterExtension;
    NT_ASSERT(IsPortValid(AdapterExtension, (*PortExtension).PortNumber));

    PxISMasked.set_status(0);
    PxIS.set_status(StorPortReadRegisterUlong(
        AdapterExtension as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).IS),
    ));

    if PxIS.HBFS() != 0 || PxIS.HBDS() != 0 || PxIS.IFS() != 0 || PxIS.TFES() != 0 {
        AhciDebugPrint!(
            b"\tFatal Error: %x\n\0".as_ptr() as *const i8,
            PxIS.status(),
        );
    }

    PxISMasked.set_DHRS(PxIS.DHRS());
    PxISMasked.set_PSS(PxIS.PSS());
    PxISMasked.set_DSS(PxIS.DSS());
    PxISMasked.set_SDBS(PxIS.SDBS());
    PxISMasked.set_DPS(PxIS.DPS());

    if PxISMasked.status() != 0 {
        StorPortWriteRegisterUlong(
            AdapterExtension as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*(*PortExtension).Port).IS),
            PxISMasked.status(),
        );
    }

    is = (1 << (*PortExtension).PortNumber) as ULONG;
    StorPortWriteRegisterUlong(
        AdapterExtension as *mut core::ffi::c_void,
        (*AdapterExtension).IS,
        is,
    );

    ci = StorPortReadRegisterUlong(
        AdapterExtension as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).CI),
    );
    sact = StorPortReadRegisterUlong(
        AdapterExtension as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).SACT),
    );

    outstanding = ci | sact;
    if ((*PortExtension).CommandIssuedSlots & (!outstanding)) != 0 {
        AhciCompleteIssuedSrb(
            PortExtension,
            (*PortExtension).CommandIssuedSlots & (!outstanding),
        );
        (*PortExtension).CommandIssuedSlots &= outstanding;
    }

    return;
}

#[no_mangle]
pub unsafe extern "system" fn AhciHwInterrupt(DeviceExtension: PVOID) -> BOOLEAN {

    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;
    let mut portPending: ULONG;
    let mut nextPort: ULONG;
    let mut i: ULONG;
    let portCount: ULONG;

    AdapterExtension = DeviceExtension as PAHCI_ADAPTER_EXTENSION;

    if (*AdapterExtension).StateFlags.Removed() != 0 {
        return 0;
    }

    portPending = StorPortReadRegisterUlong(
        AdapterExtension as PVOID,
        (*AdapterExtension).IS,
    );

    // we process interrupt for implemented ports only
    portCount = (*AdapterExtension).PortCount;
    portPending = portPending & (*AdapterExtension).PortImplemented;

    if portPending == 0 {
        return 0;
    }

    i = 1;
    while i <= portCount {
        nextPort = ((*AdapterExtension).LastInterruptPort + i) % portCount;
        if (portPending & (0x1 << nextPort)) == 0 {
            i += 1;
            continue;
        }

        NT_ASSERT(IsPortValid(AdapterExtension, nextPort));

        if (*AdapterExtension).PortExtension[nextPort as usize].DeviceParams.IsActive == 0 {
            i += 1;
            continue;
        }

        // we can assign this interrupt to this port
        (*AdapterExtension).LastInterruptPort = nextPort;
        AhciInterruptHandler(
            core::ptr::addr_of_mut!((*AdapterExtension).PortExtension[nextPort as usize]),
        );

        portPending &= !(1 << nextPort);

        // interrupt belongs to this device
        // should always return TRUE
        return 1;
    }

    AhciDebugPrint!(b"\tSomething went wrong\0".as_ptr() as *const i8);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn AhciHwStartIo(
    DeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {

    const SRB_STATUS_NO_DEVICE: UCHAR = 0x08;
    const SRB_STATUS_SUCCESS: UCHAR = 0x01;
    const SRB_STATUS_INVALID_REQUEST: UCHAR = 0x06;
    const SRB_STATUS_BAD_FUNCTION: UCHAR = 0x22;
    const SRB_STATUS_PENDING: UCHAR = 0x00;

    const SRB_FUNCTION_EXECUTE_SCSI: UCHAR = 0x00;
    const SRB_FUNCTION_PNP: UCHAR = 0x25;

    const SRB_PNP_FLAGS_ADAPTER_REQUEST: ULONG = 0x00000001;

    const StorRemoveDevice: u32 = 2;
    const StorSurpriseRemoval: u32 = 3;
    const StorStopDevice: u32 = 4;

    const SCSIOP_INQUIRY: UCHAR = 0x12;
    const SCSIOP_REPORT_LUNS: UCHAR = 0xA0;
    const SCSIOP_READ_CAPACITY: UCHAR = 0x25;
    const SCSIOP_TEST_UNIT_READY: UCHAR = 0x00;
    const SCSIOP_MODE_SENSE: UCHAR = 0x1A;
    const SCSIOP_READ: UCHAR = 0x28;
    const SCSIOP_WRITE: UCHAR = 0x2A;

    const RequestComplete: u32 = 0;

    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;

    AhciDebugPrint!(b"AhciHwStartIo()\n\0".as_ptr() as *const i8);

    AdapterExtension = DeviceExtension as PAHCI_ADAPTER_EXTENSION;

    if IsPortValid(AdapterExtension, (*Srb).PathId as ULONG) == 0 {
        (*Srb).SrbStatus = SRB_STATUS_NO_DEVICE;
        StorPortNotification(RequestComplete, AdapterExtension as PVOID, Srb);
        return 1;
    }

    match (*Srb).Function {
        SRB_FUNCTION_PNP => {
            let pnpRequest: PSCSI_PNP_REQUEST_BLOCK;
            pnpRequest = Srb as PSCSI_PNP_REQUEST_BLOCK;
            if ((*pnpRequest).SrbPnPFlags & SRB_PNP_FLAGS_ADAPTER_REQUEST) != 0 {
                match (*pnpRequest).PnPAction {
                    StorRemoveDevice | StorSurpriseRemoval => {
                        (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
                        (*AdapterExtension).StateFlags.set_Removed(1);
                        AhciDebugPrint!(b"\tAdapter removed\n\0".as_ptr() as *const i8);
                    }
                    StorStopDevice => {
                        (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
                        AhciDebugPrint!(b"\tRequested to Stop the adapter\n\0".as_ptr() as *const i8);
                    }
                    _ => {
                        (*Srb).SrbStatus = SRB_STATUS_INVALID_REQUEST;
                    }
                }
            } else {
                (*Srb).SrbStatus = SRB_STATUS_INVALID_REQUEST;
            }
        }
        SRB_FUNCTION_EXECUTE_SCSI => {
            let cdb: PCDB = core::ptr::addr_of_mut!((*Srb).Cdb) as PCDB;
            if (*Srb).CdbLength == 0 {
                AhciDebugPrint!(
                    b"\tOperationCode: %d\n\0".as_ptr() as *const i8,
                    (*cdb).CDB10.as_ref().OperationCode as i32,
                );
                (*Srb).SrbStatus = SRB_STATUS_BAD_FUNCTION;
            } else {
                NT_ASSERT(!cdb.is_null());

                match (*cdb).CDB10.as_ref().OperationCode {
                    SCSIOP_INQUIRY => {
                        (*Srb).SrbStatus = DeviceInquiryRequest(AdapterExtension, Srb, cdb);
                    }
                    SCSIOP_REPORT_LUNS => {
                        (*Srb).SrbStatus = DeviceReportLuns(AdapterExtension, Srb, cdb);
                    }
                    SCSIOP_READ_CAPACITY => {
                        (*Srb).SrbStatus = DeviceRequestCapacity(AdapterExtension, Srb, cdb);
                    }
                    SCSIOP_TEST_UNIT_READY => {
                        (*Srb).SrbStatus = DeviceRequestComplete(AdapterExtension, Srb, cdb);
                    }
                    SCSIOP_MODE_SENSE => {
                        (*Srb).SrbStatus = DeviceRequestSense(AdapterExtension, Srb, cdb);
                    }
                    SCSIOP_READ | SCSIOP_WRITE => {
                        (*Srb).SrbStatus = DeviceRequestReadWrite(AdapterExtension, Srb, cdb);
                    }
                    _ => {
                        AhciDebugPrint!(
                            b"\tOperationCode: %d\n\0".as_ptr() as *const i8,
                            (*cdb).CDB10.as_ref().OperationCode as i32,
                        );
                        (*Srb).SrbStatus = SRB_STATUS_INVALID_REQUEST;
                    }
                }
            }
        }
        _ => {
            AhciDebugPrint!(
                b"\tUnknown function code recieved: %x\n\0".as_ptr() as *const i8,
                (*Srb).Function as i32,
            );
            (*Srb).SrbStatus = SRB_STATUS_INVALID_REQUEST;
        }
    }

    if (*Srb).SrbStatus != SRB_STATUS_PENDING {
        StorPortNotification(RequestComplete, AdapterExtension as PVOID, Srb);
    } else {
        AhciProcessIO(AdapterExtension, (*Srb).PathId, Srb);
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn AhciHwResetBus(
    AdapterExtension: PVOID,
    PathId: ULONG,
) -> BOOLEAN {
    const InterruptLock: STOR_SPINLOCK = 1;

    let mut lockhandle: STOR_LOCK_HANDLE = core::mem::zeroed();

    AhciDebugPrint!(b"AhciHwResetBus()\n\0".as_ptr() as *const i8);

    if IsPortValid(AdapterExtension as PAHCI_ADAPTER_EXTENSION, PathId) != 0 {
        // Acquire Lock
        StorPortAcquireSpinLock(
            AdapterExtension,
            InterruptLock,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(lockhandle),
        );

        // TODO: Perform port reset

        // Release lock
        StorPortReleaseSpinLock(AdapterExtension, core::ptr::addr_of_mut!(lockhandle));
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn AhciHwFindAdapter(
    DeviceExtension: PVOID,
    HwContext: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Reserved3: PBOOLEAN,
) -> ULONG {
    let mut ghc: AHCI_GHC = core::mem::zeroed();
    let mut index: ULONG;
    let pci_cfg_len: ULONG;
    let mut accessRange: PACCESS_RANGE;
    let mut pci_cfg_buf: [UCHAR; core::mem::size_of::<PCI_COMMON_CONFIG>()] =
        [0; core::mem::size_of::<PCI_COMMON_CONFIG>()];

    let mut abar: PAHCI_MEMORY_REGISTERS;
    let pciConfigData: PPCI_COMMON_CONFIG;
    let adapterExtension: PAHCI_ADAPTER_EXTENSION;

    AhciDebugPrint!(b"AhciHwFindAdapter()\n\0".as_ptr() as *const i8);

    let _ = HwContext;
    let _ = BusInformation;
    let _ = ArgumentString;
    let _ = Reserved3;

    adapterExtension = DeviceExtension as PAHCI_ADAPTER_EXTENSION;
    (*adapterExtension).SlotNumber = (*ConfigInfo).SlotNumber;
    (*adapterExtension).SystemIoBusNumber = (*ConfigInfo).SystemIoBusNumber;

    // get PCI configuration header
    pci_cfg_len = StorPortGetBusData(
        adapterExtension as PVOID,
        PCIConfiguration as u32,
        (*adapterExtension).SystemIoBusNumber,
        (*adapterExtension).SlotNumber,
        pci_cfg_buf.as_mut_ptr() as PVOID,
        core::mem::size_of::<PCI_COMMON_CONFIG>() as ULONG,
    );

    if pci_cfg_len != core::mem::size_of::<PCI_COMMON_CONFIG>() as ULONG {
        AhciDebugPrint!(
            b"\tpci_cfg_len != %d :: %d\0".as_ptr() as *const i8,
            core::mem::size_of::<PCI_COMMON_CONFIG>(),
            pci_cfg_len,
        );
        return SP_RETURN_ERROR; //Not a valid device at the given bus number
    }

    pciConfigData = pci_cfg_buf.as_mut_ptr() as PPCI_COMMON_CONFIG;
    (*adapterExtension).VendorID = (*pciConfigData).VendorID;
    (*adapterExtension).DeviceID = (*pciConfigData).DeviceID;
    (*adapterExtension).RevisionID = (*pciConfigData).RevisionID as _;
    // The last PCI base address register (BAR[5], header offset 0x24) points to the AHCI base memory.
    (*adapterExtension).AhciBaseAddress =
        (*pciConfigData).u.type0.BaseAddresses[5] & 0xFFFFFFF0;

    AhciDebugPrint!(
        b"\tVendorID: %04x  DeviceID: %04x  RevisionID: %02x\n\0".as_ptr() as *const i8,
        (*adapterExtension).VendorID as u32,
        (*adapterExtension).DeviceID as u32,
        (*adapterExtension).RevisionID as u32,
    );

    // 2.1.11
    abar = core::ptr::null_mut();
    if (*ConfigInfo).NumberOfAccessRanges > 0 {
        accessRange = (*(*ConfigInfo).AccessRanges).as_mut_ptr();
        index = 0;
        while index < (*ConfigInfo).NumberOfAccessRanges {
            if (*accessRange.offset(index as isize)).RangeStart.QuadPart
                == (*adapterExtension).AhciBaseAddress as i64
            {
                abar = StorPortGetDeviceBase(
                    adapterExtension as PVOID,
                    (*ConfigInfo).AdapterInterfaceType,
                    (*ConfigInfo).SystemIoBusNumber,
                    (*accessRange.offset(index as isize)).RangeStart,
                    (*accessRange.offset(index as isize)).RangeLength,
                    !((*accessRange.offset(index as isize)).RangeInMemory) as UCHAR,
                ) as PAHCI_MEMORY_REGISTERS;
                break;
            }
            index = index.wrapping_add(1);
        }
    }

    if abar.is_null() {
        AhciDebugPrint!(b"\tabar == NULL\n\0".as_ptr() as *const i8);
        return SP_RETURN_ERROR; // corrupted information supplied
    }

    (*adapterExtension).ABAR_Address = abar;
    (*adapterExtension).CAP =
        StorPortReadRegisterUlong(adapterExtension as PVOID, core::ptr::addr_of_mut!((*abar).CAP));
    (*adapterExtension).CAP2 =
        StorPortReadRegisterUlong(adapterExtension as PVOID, core::ptr::addr_of_mut!((*abar).CAP2));
    (*adapterExtension).Version =
        StorPortReadRegisterUlong(adapterExtension as PVOID, core::ptr::addr_of_mut!((*abar).VS));
    (*adapterExtension).LastInterruptPort = -1i32 as ULONG;

    // 10.1.2
    ghc.set_status(StorPortReadRegisterUlong(adapterExtension as PVOID, core::ptr::addr_of_mut!((*abar).GHC)));
    // AE := Highest Significant bit of GHC
    if ghc.AE() != 0 {
        // Hmm, controller was already in power state
        AhciDebugPrint!(b"\tAE Already set, Reset()\n\0".as_ptr() as *const i8);
        if AhciAdapterReset(adapterExtension) == 0 {
            AhciDebugPrint!(b"\tReset Failed!\n\0".as_ptr() as *const i8);
            return SP_RETURN_ERROR; // reset failed
        }
    }

    ghc.set_status(0);
    ghc.set_AE(1); // only AE=1
    // tell the controller that we know about AHCI
    StorPortWriteRegisterUlong(
        adapterExtension as PVOID,
        core::ptr::addr_of_mut!((*abar).GHC),
        ghc.status(),
    );

    (*adapterExtension).IS = core::ptr::addr_of_mut!((*abar).IS);
    (*adapterExtension).PortImplemented =
        StorPortReadRegisterUlong(adapterExtension as PVOID, core::ptr::addr_of_mut!((*abar).PI));

    if (*adapterExtension).PortImplemented == 0 {
        AhciDebugPrint!(b"\tadapterExtension->PortImplemented == 0\n\0".as_ptr() as *const i8);
        return SP_RETURN_ERROR;
    }

    (*ConfigInfo).Master = TRUE as UCHAR;
    (*ConfigInfo).AlignmentMask = 0x3;
    (*ConfigInfo).ScatterGather = TRUE as UCHAR;
    (*ConfigInfo).DmaWidth = Width32Bits;
    (*ConfigInfo).WmiDataProvider = FALSE as UCHAR;
    (*ConfigInfo).Dma32BitAddresses = TRUE as UCHAR;

    if IsAdapterCAPS64((*adapterExtension).CAP) != 0 {
        (*ConfigInfo).Dma64BitAddresses = TRUE as UCHAR;
    }

    (*ConfigInfo).MaximumNumberOfTargets = 1;
    (*ConfigInfo).ResetTargetSupported = TRUE as UCHAR;
    (*ConfigInfo).NumberOfPhysicalBreaks = 0x21;
    (*ConfigInfo).MaximumNumberOfLogicalUnits = 1;
    (*ConfigInfo).NumberOfBuses = MAXIMUM_AHCI_PORT_COUNT as UCHAR;
    (*ConfigInfo).MaximumTransferLength = MAXIMUM_TRANSFER_LENGTH;
    (*ConfigInfo).SynchronizationModel = StorSynchronizeFullDuplex;

    // Turn IE -- Interrupt Enabled
    ghc.set_status(StorPortReadRegisterUlong(adapterExtension as PVOID, core::ptr::addr_of_mut!((*abar).GHC)));
    ghc.set_IE(1);
    StorPortWriteRegisterUlong(
        adapterExtension as PVOID,
        core::ptr::addr_of_mut!((*abar).GHC),
        ghc.status(),
    );

    // allocate necessary resource for each port
    if AhciAllocateResourceForAdapter(adapterExtension, ConfigInfo) == 0 {
        NT_ASSERT(FALSE);
        return SP_RETURN_ERROR;
    }

    index = 0;
    while index < (*adapterExtension).PortCount {
        if ((*adapterExtension).PortImplemented & (0x1u32 << index)) != 0 {
            AhciPortInitialize(
                core::ptr::addr_of_mut!((*adapterExtension).PortExtension[index as usize]) as PVOID,
            );
        }
        index = index.wrapping_add(1);
    }

    SP_RETURN_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PVOID,
    RegistryPath: PVOID,
) -> ULONG {
    const TRUE: u8 = 1;
    const STOR_MAP_NON_READ_WRITE_BUFFERS: u32 = 2;

    let status: ULONG;
    // initialize the hardware data structure
    let mut hwInitializationData: HW_INITIALIZATION_DATA = core::mem::zeroed();

    // set size of hardware initialization structure
    hwInitializationData.HwInitializationDataSize =
        core::mem::size_of::<HW_INITIALIZATION_DATA>() as ULONG;

    // identity required miniport entry point routines
    hwInitializationData.HwStartIo = core::mem::transmute::<_, PHW_STARTIO>(AhciHwStartIo as *const ());
    hwInitializationData.HwResetBus = core::mem::transmute::<_, PHW_RESET_BUS>(AhciHwResetBus as *const ());
    hwInitializationData.HwInterrupt = core::mem::transmute::<_, PHW_INTERRUPT>(AhciHwInterrupt as *const ());
    hwInitializationData.HwInitialize = core::mem::transmute::<_, PHW_INITIALIZE>(AhciHwInitialize as *const ());
    hwInitializationData.HwFindAdapter = core::mem::transmute::<_, PHW_FIND_ADAPTER>(AhciHwFindAdapter as *const ());

    // adapter specific information
    hwInitializationData.TaggedQueuing = TRUE;
    hwInitializationData.AutoRequestSense = TRUE;
    hwInitializationData.MultipleRequestPerLu = TRUE;
    hwInitializationData.NeedPhysicalAddresses = TRUE;

    hwInitializationData.NumberOfAccessRanges = 6;
    hwInitializationData.AdapterInterfaceType = PCIBus;
    hwInitializationData.MapBuffers = STOR_MAP_NON_READ_WRITE_BUFFERS as _;

    // set required extension sizes
    hwInitializationData.SrbExtensionSize = core::mem::size_of::<AHCI_SRB_EXTENSION>() as ULONG;
    hwInitializationData.DeviceExtensionSize =
        core::mem::size_of::<AHCI_ADAPTER_EXTENSION>() as ULONG;

    // register our hw init data
    status = StorPortInitialize(
        DriverObject,
        RegistryPath,
        core::ptr::addr_of_mut!(hwInitializationData),
        core::ptr::null_mut(),
    );

    NT_ASSERT((status == STATUS_SUCCESS as _) as i32);
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn AhciATA_CFIS(
    PortExtension: PAHCI_PORT_EXTENSION,
    SrbExtension: PAHCI_SRB_EXTENSION,
) -> ULONG {

    let cmdTable: PAHCI_COMMAND_TABLE;

    let _ = PortExtension;

    AhciDebugPrint!(b"AhciATA_CFIS()\n\0".as_ptr() as *const i8);

    cmdTable = SrbExtension as PAHCI_COMMAND_TABLE;

    AhciZeroMemory(
        (*cmdTable).CFIS.as_mut_ptr() as PCHAR,
        core::mem::size_of_val(&(*cmdTable).CFIS) as ULONG,
    );

    (*cmdTable).CFIS[AHCI_ATA_CFIS_FisType as usize] = FIS_TYPE_REG_H2D as UCHAR;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_PMPort_C as usize] = (1 << 7);
    (*cmdTable).CFIS[AHCI_ATA_CFIS_CommandReg as usize] = (*SrbExtension).CommandReg;

    (*cmdTable).CFIS[AHCI_ATA_CFIS_FeaturesLow as usize] = (*SrbExtension).FeaturesLow;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA0 as usize] = (*SrbExtension).LBA0;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA1 as usize] = (*SrbExtension).LBA1;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA2 as usize] = (*SrbExtension).LBA2;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_Device as usize] = (*SrbExtension).Device;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA3 as usize] = (*SrbExtension).LBA3;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA4 as usize] = (*SrbExtension).LBA4;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA5 as usize] = (*SrbExtension).LBA5;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_FeaturesHigh as usize] = (*SrbExtension).FeaturesHigh;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_SectorCountLow as usize] = (*SrbExtension).SectorCountLow;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_SectorCountHigh as usize] = (*SrbExtension).SectorCountHigh;

    return 5;
}

#[no_mangle]
pub unsafe extern "C" fn AhciATAPI_CFIS(
    PortExtension: PAHCI_PORT_EXTENSION,
    SrbExtension: PAHCI_SRB_EXTENSION,
) -> ULONG {
    let cmdTable: PAHCI_COMMAND_TABLE;
    let _ = PortExtension;

    AhciDebugPrint!(b"AhciATAPI_CFIS()\n\0".as_ptr() as *const i8);

    cmdTable = SrbExtension as PAHCI_COMMAND_TABLE;

    // NT_ASSERT(SrbExtension->CommandReg == IDE_COMMAND_ATAPI_PACKET);

    AhciZeroMemory(
        (*cmdTable).CFIS.as_mut_ptr() as PCHAR,
        core::mem::size_of_val(&(*cmdTable).CFIS) as ULONG,
    );

    (*cmdTable).CFIS[AHCI_ATA_CFIS_FisType as usize] = FIS_TYPE_REG_H2D as UCHAR; // FIS Type
    (*cmdTable).CFIS[AHCI_ATA_CFIS_PMPort_C as usize] = (1 << 7); // PM Port & C
    (*cmdTable).CFIS[AHCI_ATA_CFIS_CommandReg as usize] = (*SrbExtension).CommandReg;

    (*cmdTable).CFIS[AHCI_ATA_CFIS_FeaturesLow as usize] = (*SrbExtension).FeaturesLow;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA0 as usize] = (*SrbExtension).LBA0;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA1 as usize] = (*SrbExtension).LBA1;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA2 as usize] = (*SrbExtension).LBA2;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_Device as usize] = (*SrbExtension).Device;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA3 as usize] = (*SrbExtension).LBA3;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA4 as usize] = (*SrbExtension).LBA4;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_LBA5 as usize] = (*SrbExtension).LBA5;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_FeaturesHigh as usize] = (*SrbExtension).FeaturesHigh;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_SectorCountLow as usize] = (*SrbExtension).SectorCountLow;
    (*cmdTable).CFIS[AHCI_ATA_CFIS_SectorCountHigh as usize] = (*SrbExtension).SectorCountHigh;

    return 5;
}
#[no_mangle]
pub unsafe extern "C" fn AhciBuild_PRDT(
    PortExtension: PAHCI_PORT_EXTENSION,
    SrbExtension: PAHCI_SRB_EXTENSION,
) -> ULONG {
    let mut index: ULONG;
    let cmdTable: PAHCI_COMMAND_TABLE;
    let sgl: PLOCAL_SCATTER_GATHER_LIST;
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;

    AhciDebugPrint!(b"AhciBuild_PRDT()\n\0".as_ptr() as *const i8);

    sgl = (*SrbExtension).pSgl;
    cmdTable = SrbExtension as PAHCI_COMMAND_TABLE;
    AdapterExtension = (*PortExtension).AdapterExtension;

    NT_ASSERT((!sgl.is_null()) as BOOLEAN);
    NT_ASSERT(((*sgl).NumberOfElements < MAXIMUM_AHCI_PRDT_ENTRIES) as BOOLEAN);

    index = 0;
    while index < (*sgl).NumberOfElements {
        NT_ASSERT(((*sgl).List[index as usize].Length <= MAXIMUM_TRANSFER_LENGTH) as BOOLEAN);

        (*cmdTable).PRDT[index as usize].DBA =
            (*sgl).List[index as usize].PhysicalAddress.__bindgen_anon_1.LowPart;
        if IsAdapterCAPS64((*AdapterExtension).CAP) != 0 {
            (*cmdTable).PRDT[index as usize].DBAU =
                (*sgl).List[index as usize].PhysicalAddress.__bindgen_anon_1.HighPart as _;
        }

        // Data Byte Count (DBC): A '0' based value that Indicates the length, in bytes, of the data block.
        // A maximum of length of 4MB may exist for any entry. Bit '0' of this field must always be '1' to
        // indicate an even byte count. A value of '1' indicates 2 bytes, '3' indicates 4 bytes, etc.
        (*cmdTable).PRDT[index as usize].set_DBC((*sgl).List[index as usize].Length - 1);

        index += 1;
    }

    return (*sgl).NumberOfElements;
}
#[no_mangle]
pub unsafe extern "C" fn AhciProcessSrb(
    PortExtension: PAHCI_PORT_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    SlotIndex: ULONG,
) {

    let prdtlen: ULONG;
    let sig: ULONG;
    let mut length: ULONG = 0;
    let cfl: ULONG;
    let SrbExtension: PAHCI_SRB_EXTENSION;
    let CommandHeader: PAHCI_COMMAND_HEADER;
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;
    let CommandTablePhysicalAddress: STOR_PHYSICAL_ADDRESS;

    AhciDebugPrint!(b"AhciProcessSrb()\n\0".as_ptr() as *const i8);

    NT_ASSERT(((*Srb).PathId as ULONG == (*PortExtension).PortNumber) as i32);

    SrbExtension = GetSrbExtension(Srb);
    AdapterExtension = (*PortExtension).AdapterExtension;

    NT_ASSERT((!SrbExtension.is_null()) as i32);
    NT_ASSERT(((*SrbExtension).AtaFunction != 0) as i32);

    if ((*SrbExtension).AtaFunction == ATA_FUNCTION_ATA_IDENTIFY)
        && ((*SrbExtension).CommandReg == IDE_COMMAND_NOT_VALID as UCHAR)
    {
        // Here we are safe to check SIG register
        let sig_val = StorPortReadRegisterUlong(
            AdapterExtension as PVOID,
            core::ptr::addr_of_mut!((*(*PortExtension).Port).SIG),
        );
        sig = sig_val;
        if sig == 0x101 {
            AhciDebugPrint!(b"\tATA Device Found!\n\0".as_ptr() as *const i8);
            (*SrbExtension).CommandReg = IDE_COMMAND_IDENTIFY as UCHAR;
        } else {
            AhciDebugPrint!(b"\tATAPI Device Found!\n\0".as_ptr() as *const i8);
            (*SrbExtension).CommandReg = IDE_COMMAND_ATAPI_IDENTIFY as UCHAR;
        }
    }

    NT_ASSERT((SlotIndex < AHCI_Global_Port_CAP_NCS((*AdapterExtension).CAP)) as i32);
    (*SrbExtension).SlotIndex = SlotIndex;

    // program the CFIS in the CommandTable
    CommandHeader = core::ptr::addr_of_mut!(*(*PortExtension).CommandList.offset(SlotIndex as isize));

    if IsAtapiCommand((*SrbExtension).AtaFunction) != 0 {
        cfl = AhciATAPI_CFIS(PortExtension, SrbExtension);
    } else if IsAtaCommand((*SrbExtension).AtaFunction) != 0 {
        cfl = AhciATA_CFIS(PortExtension, SrbExtension);
    } else {
        NT_ASSERT(0);
        cfl = 0;
    }

    if IsDataTransferNeeded(SrbExtension) != 0 {
        prdtlen = AhciBuild_PRDT(PortExtension, SrbExtension);
        NT_ASSERT((prdtlen != (-1i32 as ULONG)) as i32);
    } else {
        prdtlen = 0;
    }

    // Program the command header
    (*CommandHeader).DI.set_PRDTL(prdtlen); // number of entries in PRD table
    (*CommandHeader).DI.set_CFL(cfl);
    (*CommandHeader).DI.set_A(
        if ((*SrbExtension).AtaFunction & ATA_FUNCTION_ATAPI_COMMAND) != 0 {
            1
        } else {
            0
        },
    );
    (*CommandHeader).DI.set_W(
        if ((*SrbExtension).Flags & ATA_FLAGS_DATA_OUT) != 0 {
            1
        } else {
            0
        },
    );
    (*CommandHeader).DI.set_P(0); // ATA Specifications says so
    (*CommandHeader).DI.set_PMP(0); // Port Multiplier

    // Reset -- Manual Configuation
    (*CommandHeader).DI.set_R(0);
    (*CommandHeader).DI.set_B(0);
    (*CommandHeader).DI.set_C(0);

    (*CommandHeader).PRDBC = 0;

    (*CommandHeader).Reserved[0] = 0;
    (*CommandHeader).Reserved[1] = 0;
    (*CommandHeader).Reserved[2] = 0;
    (*CommandHeader).Reserved[3] = 0;

    // set CommandHeader CTBA
    CommandTablePhysicalAddress = StorPortGetPhysicalAddress(
        AdapterExtension as PVOID,
        core::ptr::null_mut(),
        SrbExtension as PVOID,
        core::ptr::addr_of_mut!(length),
    );

    NT_ASSERT((length != 0) as i32);

    // command table alignment
    NT_ASSERT(((CommandTablePhysicalAddress.__bindgen_anon_1.LowPart % 128) == 0) as i32);

    (*CommandHeader).CTBA = CommandTablePhysicalAddress.__bindgen_anon_1.LowPart;

    if IsAdapterCAPS64((*AdapterExtension).CAP) != 0 {
        (*CommandHeader).CTBA_U = CommandTablePhysicalAddress.__bindgen_anon_1.HighPart as ULONG;
    }

    // mark this slot
    *(*PortExtension).Slot.as_mut_ptr().offset(SlotIndex as isize) = Srb;
    (*PortExtension).QueueSlots |= 1 << SlotIndex;
    return;
}
#[no_mangle]
pub unsafe extern "C" fn AhciActivatePort(
    PortExtension: PAHCI_PORT_EXTENSION,
) {
    let mut cmd: AHCI_PORT_CMD = core::mem::zeroed();
    let mut QueueSlots: ULONG;
    let slotToActivate: ULONG;
    let tmp: ULONG;
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;

    AhciDebugPrint!(b"AhciActivatePort()\n\0".as_ptr() as *const i8);

    AdapterExtension = (*PortExtension).AdapterExtension;
    QueueSlots = (*PortExtension).QueueSlots;

    if QueueSlots == 0 {
        return;
    }

    // section 3.3.14
    // Bits in this field shall only be set to '1' by software when PxCMD.ST is set to '1'
    cmd.set_status(StorPortReadRegisterUlong(
        AdapterExtension as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).CMD),
    ));

    if cmd.ST() == 0
    // PxCMD.ST == 0
    {
        return;
    }

    // get the lowest set bit
    tmp = QueueSlots & (QueueSlots - 1);

    if tmp == 0 {
        slotToActivate = QueueSlots;
    } else {
        slotToActivate = QueueSlots & (!tmp);
    }

    // mark that bit off in QueueSlots
    // so we can know we it is really needed to activate port or not
    (*PortExtension).QueueSlots &= !slotToActivate;
    // mark this CommandIssuedSlots
    // to validate in completeIssuedCommand
    (*PortExtension).CommandIssuedSlots |= slotToActivate;

    // tell the HBA to issue this Command Slot to the given port
    StorPortWriteRegisterUlong(
        AdapterExtension as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*(*PortExtension).Port).CI),
        slotToActivate,
    );

    return;
}
#[no_mangle]
pub unsafe extern "C" fn AhciProcessIO(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    PathId: UCHAR,
    Srb: PSCSI_REQUEST_BLOCK,
) {
    const InterruptLock: STOR_SPINLOCK = 1;

    let mut tmpSrb: PSCSI_REQUEST_BLOCK;
    let mut lockhandle: STOR_LOCK_HANDLE = core::mem::zeroed();
    let PortExtension: PAHCI_PORT_EXTENSION;
    let mut commandSlotMask: ULONG;
    let occupiedSlots: ULONG;
    let mut slotIndex: ULONG;
    let NCS: ULONG;

    AhciDebugPrint!(b"AhciProcessIO()\n\0".as_ptr() as *const i8);
    AhciDebugPrint!(b"\tPathId: %d\n\0".as_ptr() as *const i8, PathId as i32);

    PortExtension = core::ptr::addr_of_mut!((*AdapterExtension).PortExtension[PathId as usize]);

    NT_ASSERT(((PathId as ULONG) < (*AdapterExtension).PortCount) as BOOLEAN);

    // Acquire Lock
    StorPortAcquireSpinLock(
        AdapterExtension as PVOID,
        InterruptLock,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(lockhandle),
    );

    // add Srb to queue
    AddQueue(core::ptr::addr_of_mut!((*PortExtension).SrbQueue), Srb as PVOID);

    if (*PortExtension).DeviceParams.IsActive == 0 {
        // Release Lock
        StorPortReleaseSpinLock(AdapterExtension as PVOID, core::ptr::addr_of_mut!(lockhandle));
        return; // we should wait for device to get active
    }

    occupiedSlots = (*PortExtension).QueueSlots | (*PortExtension).CommandIssuedSlots; // Busy command slots for given port
    NCS = AHCI_Global_Port_CAP_NCS((*AdapterExtension).CAP);
    commandSlotMask = (1 << NCS) - 1; // available slots mask

    commandSlotMask = commandSlotMask & !occupiedSlots;
    if commandSlotMask != 0 {
        // iterate over HBA port slots
        slotIndex = 0;
        while slotIndex < NCS {
            // find first free slot
            if (commandSlotMask & (1 << slotIndex)) != 0 {
                tmpSrb = RemoveQueue(core::ptr::addr_of_mut!((*PortExtension).SrbQueue)) as PSCSI_REQUEST_BLOCK;
                if !tmpSrb.is_null() {
                    NT_ASSERT(((*tmpSrb).PathId == PathId) as BOOLEAN);
                    AhciProcessSrb(PortExtension, tmpSrb, slotIndex);
                } else {
                    break;
                }
            } else {
                break;
            }
            slotIndex += 1;
        }
    }

    // program HBA port
    AhciActivatePort(PortExtension);

    // Release Lock
    StorPortReleaseSpinLock(AdapterExtension as PVOID, core::ptr::addr_of_mut!(lockhandle));

    return;
}
#[no_mangle]
pub unsafe extern "C" fn AtapiInquiryCompletion(_Extension: PVOID, _Srb: PVOID) {
    let PortExtension: PAHCI_PORT_EXTENSION;
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;
    let Srb: PSCSI_REQUEST_BLOCK;
    let status: BOOLEAN;

    AhciDebugPrint!(b"AtapiInquiryCompletion()\n\0".as_ptr() as *const i8);

    PortExtension = _Extension as PAHCI_PORT_EXTENSION;
    Srb = _Srb as PSCSI_REQUEST_BLOCK;

    NT_ASSERT((!Srb.is_null()) as BOOLEAN);
    NT_ASSERT((!PortExtension.is_null()) as BOOLEAN);

    AdapterExtension = (*PortExtension).AdapterExtension;

    // send queue depth
    status = StorPortSetDeviceQueueDepth(
        (*PortExtension).AdapterExtension as PVOID,
        (*Srb).PathId,
        (*Srb).TargetId,
        (*Srb).Lun,
        AHCI_Global_Port_CAP_NCS((*AdapterExtension).CAP),
    );

    NT_ASSERT((status == 1) as BOOLEAN);
    return;
}

#[no_mangle]
pub unsafe extern "C" fn InquiryCompletion(
    _Extension: PVOID,
    _Srb: PVOID,
) {

    const SRB_STATUS_SUCCESS: u8 = 0x01;
    const SRB_STATUS_NO_DEVICE: u8 = 0x08;
    const AHCI_DEVICE_TYPE_NODEVICE: UCHAR = 3;
    const AHCI_DEVICE_TYPE_ATA: UCHAR = 1;
    const AHCI_DEVICE_TYPE_ATAPI: UCHAR = 2;
    const IDE_COMMAND_IDENTIFY: u8 = 0xEC;
    const DIRECT_ACCESS_DEVICE: UCHAR = 0x00;
    const READ_ONLY_DIRECT_ACCESS_DEVICE: UCHAR = 0x05;
    const DEVICE_ATA_BLOCK_SIZE: u32 = 512;
    const DEVICE_CONNECTED: u32 = 0x00;
    const INQUIRYDATABUFFERSIZE: u32 = 36;
    const FALSE: BOOLEAN = 0;
    const TRUE: BOOLEAN = 1;

    let PortExtension: PAHCI_PORT_EXTENSION;
    let Srb: PSCSI_REQUEST_BLOCK;

    let status: BOOLEAN;
    let InquiryData: PINQUIRYDATA;
    let SrbExtension: PAHCI_SRB_EXTENSION;
    let AdapterExtension: PAHCI_ADAPTER_EXTENSION;
    let IdentifyDeviceData: PIDENTIFY_DEVICE_DATA;

    AhciDebugPrint!(b"InquiryCompletion()\n\0".as_ptr() as *const i8);

    PortExtension = _Extension as PAHCI_PORT_EXTENSION;
    Srb = _Srb as PSCSI_REQUEST_BLOCK;

    NT_ASSERT((!Srb.is_null()) as BOOLEAN);
    NT_ASSERT((!PortExtension.is_null()) as BOOLEAN);

    InquiryData = (*Srb).DataBuffer as PINQUIRYDATA;
    SrbExtension = GetSrbExtension(Srb);
    AdapterExtension = (*PortExtension).AdapterExtension;
    IdentifyDeviceData = (*PortExtension).IdentifyDeviceData;

    if (*Srb).SrbStatus != SRB_STATUS_SUCCESS {
        if (*Srb).SrbStatus == SRB_STATUS_NO_DEVICE {
            (*PortExtension).DeviceParams.DeviceType = AHCI_DEVICE_TYPE_NODEVICE;
        }
        return;
    }

    NT_ASSERT((!InquiryData.is_null()) as BOOLEAN);
    NT_ASSERT(((*Srb).SrbStatus == SRB_STATUS_SUCCESS) as BOOLEAN);

    // Device specific data
    (*PortExtension).DeviceParams.MaxLba.QuadPart = 0;

    if (*SrbExtension).CommandReg == IDE_COMMAND_IDENTIFY {
        (*PortExtension).DeviceParams.DeviceType = AHCI_DEVICE_TYPE_ATA;
        if (*IdentifyDeviceData).GeneralConfiguration.RemovableMedia() != 0 {
            (*PortExtension).DeviceParams.RemovableDevice = 1;
        }

        if ((*IdentifyDeviceData).CommandSetSupport.BigLba() != 0)
            && ((*IdentifyDeviceData).CommandSetActive.BigLba() != 0)
        {
            (*PortExtension).DeviceParams.Lba48BitMode = 1;
        }

        (*PortExtension).DeviceParams.AccessType = DIRECT_ACCESS_DEVICE;

        /* Device max address lba */
        if (*PortExtension).DeviceParams.Lba48BitMode != 0 {
            (*PortExtension).DeviceParams.MaxLba.__bindgen_anon_1.LowPart = (*IdentifyDeviceData).Max48BitLBA[0];
            (*PortExtension).DeviceParams.MaxLba.__bindgen_anon_1.HighPart = (*IdentifyDeviceData).Max48BitLBA[1] as _;
        } else {
            (*PortExtension).DeviceParams.MaxLba.__bindgen_anon_1.LowPart =
                (*IdentifyDeviceData).UserAddressableSectors;
        }

        /* Bytes Per Logical Sector */
        if (*IdentifyDeviceData)
            .PhysicalLogicalSectorSize
            .LogicalSectorLongerThan256Words()
            != 0
        {
            AhciDebugPrint!(
                b"\tBytesPerLogicalSector != DEVICE_ATA_BLOCK_SIZE\n\0".as_ptr() as *const i8,
            );
            NT_ASSERT(FALSE);
        }

        (*PortExtension).DeviceParams.BytesPerLogicalSector = DEVICE_ATA_BLOCK_SIZE;

        /* Bytes Per Physical Sector */
        if (*IdentifyDeviceData)
            .PhysicalLogicalSectorSize
            .MultipleLogicalSectorsPerPhysicalSector()
            != 0
        {
            AhciDebugPrint!(
                b"\tBytesPerPhysicalSector != DEVICE_ATA_BLOCK_SIZE\n\0".as_ptr() as *const i8,
            );
            NT_ASSERT(FALSE);
        }

        (*PortExtension).DeviceParams.BytesPerPhysicalSector = DEVICE_ATA_BLOCK_SIZE;

        // last byte should be NULL
        StorPortCopyMemory(
            (*PortExtension).DeviceParams.VendorId.as_mut_ptr() as PVOID,
            (*IdentifyDeviceData).ModelNumber.as_mut_ptr() as PVOID,
            core::mem::size_of_val(&(*PortExtension).DeviceParams.VendorId) - 1,
        );
        StorPortCopyMemory(
            (*PortExtension).DeviceParams.RevisionID.as_mut_ptr() as PVOID,
            (*IdentifyDeviceData).FirmwareRevision.as_mut_ptr() as PVOID,
            core::mem::size_of_val(&(*PortExtension).DeviceParams.RevisionID) - 1,
        );
        StorPortCopyMemory(
            (*PortExtension).DeviceParams.SerialNumber.as_mut_ptr() as PVOID,
            (*IdentifyDeviceData).SerialNumber.as_mut_ptr() as PVOID,
            core::mem::size_of_val(&(*PortExtension).DeviceParams.SerialNumber) - 1,
        );

        let vidlen = core::mem::size_of_val(&(*PortExtension).DeviceParams.VendorId) - 1;
        (*PortExtension).DeviceParams.VendorId[vidlen] = b'\0';
        let revlen = core::mem::size_of_val(&(*PortExtension).DeviceParams.RevisionID) - 1;
        (*PortExtension).DeviceParams.RevisionID[revlen] = b'\0';
        let serlen = core::mem::size_of_val(&(*PortExtension).DeviceParams.SerialNumber) - 1;
        (*PortExtension).DeviceParams.SerialNumber[serlen] = b'\0';

        // TODO: Add other device params
        AhciDebugPrint!(b"\tATA Device\n\0".as_ptr() as *const i8);
    } else {
        AhciDebugPrint!(b"\tATAPI Device\n\0".as_ptr() as *const i8);
        (*PortExtension).DeviceParams.DeviceType = AHCI_DEVICE_TYPE_ATAPI;
        (*PortExtension).DeviceParams.AccessType = READ_ONLY_DIRECT_ACCESS_DEVICE;
    }

    // INQUIRYDATABUFFERSIZE = 36 ; Defined in storport.h
    if (*Srb).DataTransferLength < INQUIRYDATABUFFERSIZE {
        AhciDebugPrint!(
            b"\tDataBufferLength < sizeof(INQUIRYDATA), Could crash the driver.\n\0".as_ptr()
                as *const i8,
        );
        NT_ASSERT(FALSE);
    }

    // update data transfer length
    (*Srb).DataTransferLength = INQUIRYDATABUFFERSIZE;

    // prepare data to send
    *(*InquiryData).__bindgen_anon_1.Versions.as_mut() = 2; // InquiryData->Versions = 2
    (*InquiryData).set_Wide32Bit(1);
    (*InquiryData).set_CommandQueue(0); // NCQ not supported
    (*InquiryData).set_ResponseDataFormat(0x2);
    (*InquiryData).set_DeviceTypeModifier(0);
    (*InquiryData).set_DeviceTypeQualifier(DEVICE_CONNECTED as u8);
    (*InquiryData).AdditionalLength = (INQUIRYDATABUFFERSIZE - 5) as u8;
    (*InquiryData).set_DeviceType((*PortExtension).DeviceParams.AccessType as u8);
    (*InquiryData).set_RemovableMedia((*PortExtension).DeviceParams.RemovableDevice as u8);

    // Fill VendorID, Product Revision Level and other string fields
    StorPortCopyMemory(
        (*InquiryData).VendorId.as_mut_ptr() as PVOID,
        (*PortExtension).DeviceParams.VendorId.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&(*InquiryData).VendorId) - 1,
    );
    StorPortCopyMemory(
        (*InquiryData).ProductId.as_mut_ptr() as PVOID,
        (*PortExtension).DeviceParams.RevisionID.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&(*PortExtension).DeviceParams.RevisionID),
    );
    StorPortCopyMemory(
        (*InquiryData).ProductRevisionLevel.as_mut_ptr() as PVOID,
        (*PortExtension).DeviceParams.SerialNumber.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&(*InquiryData).ProductRevisionLevel) - 1,
    );

    let ividlen = core::mem::size_of_val(&(*InquiryData).VendorId) - 1;
    (*InquiryData).VendorId[ividlen] = b'\0';
    let ipidlen = core::mem::size_of_val(&(*InquiryData).ProductId) - 1;
    (*InquiryData).ProductId[ipidlen] = b'\0';
    let iprllen = core::mem::size_of_val(&(*InquiryData).ProductRevisionLevel) - 1;
    (*InquiryData).ProductRevisionLevel[iprllen] = b'\0';

    // send queue depth
    status = StorPortSetDeviceQueueDepth(
        (*PortExtension).AdapterExtension as PVOID,
        (*Srb).PathId,
        (*Srb).TargetId,
        (*Srb).Lun,
        AHCI_Global_Port_CAP_NCS((*AdapterExtension).CAP),
    );

    NT_ASSERT((status == TRUE) as BOOLEAN);
    return;
}

#[no_mangle]
pub unsafe extern "C" fn AhciATAPICommand(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Cdb: PCDB,
) -> UCHAR {

    const ATA_FUNCTION_ATAPI_COMMAND: ULONG = 0x200; // drivers/storage/port/storahci/storahci.h
    const AHCI_DEVICE_TYPE_ATAPI: UCHAR = 2;
    const SRB_FLAGS_DATA_IN: ULONG = 0x00000040;
    const SRB_FLAGS_DATA_OUT: ULONG = 0x00000080;
    const ATA_FLAGS_DATA_IN: ULONG = 1 << 1;
    const ATA_FLAGS_DATA_OUT: ULONG = 1 << 2;
    const ATA_FLAGS_USE_DMA: ULONG = 1 << 4;
    const SCSIOP_INQUIRY: u8 = 0x12;
    const SCSIOP_READ: u8 = 0x28;
    const SCSIOP_WRITE: u8 = 0x2A;
    const IDE_COMMAND_ATAPI_PACKET: UCHAR = 0xA0;
    const SRB_STATUS_PENDING: UCHAR = 0x00;

    let SrbFlags: ULONG;
    let DataBufferLength: ULONG;
    let SrbExtension: PAHCI_SRB_EXTENSION;
    let PortExtension: PAHCI_PORT_EXTENSION;

    AhciDebugPrint!(b"AhciATAPICommand()\n\0".as_ptr() as *const i8);

    SrbFlags = (*Srb).SrbFlags;
    SrbExtension = GetSrbExtension(Srb);
    DataBufferLength = (*Srb).DataTransferLength;
    PortExtension = core::ptr::addr_of_mut!(
        (*AdapterExtension).PortExtension[(*Srb).PathId as usize]
    );

    // NT_ASSERT(PortExtension->DeviceParams.DeviceType == AHCI_DEVICE_TYPE_ATAPI);
    if !((*PortExtension).DeviceParams.DeviceType == AHCI_DEVICE_TYPE_ATAPI) {
        // NT_ASSERT failure
    }

    // NT_ASSERT(SrbExtension != NULL);
    if SrbExtension.is_null() {
        // NT_ASSERT failure
    }

    (*SrbExtension).AtaFunction = ATA_FUNCTION_ATAPI_COMMAND;
    (*SrbExtension).Flags = 0;

    if (SrbFlags & SRB_FLAGS_DATA_IN) != 0 {
        (*SrbExtension).Flags |= ATA_FLAGS_DATA_IN;
    }

    if (SrbFlags & SRB_FLAGS_DATA_OUT) != 0 {
        (*SrbExtension).Flags |= ATA_FLAGS_DATA_OUT;
    }

    (*SrbExtension).FeaturesLow = 0;

    (*SrbExtension).CompletionRoutine = None;

    // NT_ASSERT(Cdb != NULL);
    if Cdb.is_null() {
        // NT_ASSERT failure
    }

    match (*Cdb).CDB10.as_ref().OperationCode {
        SCSIOP_INQUIRY => {
            (*SrbExtension).Flags |= ATA_FLAGS_DATA_IN;
            (*SrbExtension).CompletionRoutine = core::mem::transmute::<_, PAHCI_COMPLETION_ROUTINE>(AtapiInquiryCompletion as *const ());
        }
        SCSIOP_READ => {
            (*SrbExtension).Flags |= ATA_FLAGS_USE_DMA;
            (*SrbExtension).FeaturesLow = 0x5;
        }
        SCSIOP_WRITE => {
            (*SrbExtension).Flags |= ATA_FLAGS_USE_DMA;
            (*SrbExtension).FeaturesLow = 0x1;
        }
        _ => {}
    }

    (*SrbExtension).CommandReg = IDE_COMMAND_ATAPI_PACKET;

    (*SrbExtension).LBA0 = 0;
    (*SrbExtension).LBA1 = (DataBufferLength >> 0) as UCHAR;
    (*SrbExtension).LBA2 = (DataBufferLength >> 8) as UCHAR;
    (*SrbExtension).Device = 0;
    (*SrbExtension).LBA3 = 0;
    (*SrbExtension).LBA4 = 0;
    (*SrbExtension).LBA5 = 0;
    (*SrbExtension).FeaturesHigh = 0;
    (*SrbExtension).SectorCountLow = 0;
    (*SrbExtension).SectorCountHigh = 0;

    if ((*SrbExtension).Flags & ATA_FLAGS_DATA_IN) != 0
        || ((*SrbExtension).Flags & ATA_FLAGS_DATA_OUT) != 0
    {
        (*SrbExtension).pSgl =
            StorPortGetScatterGatherList(AdapterExtension, Srb) as PLOCAL_SCATTER_GATHER_LIST;
    }

    return SRB_STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceRequestSense(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Cdb: PCDB,
) -> UCHAR {
    let mut ModeHeader: PMODE_PARAMETER_HEADER;
    let PortExtension: PAHCI_PORT_EXTENSION;

    AhciDebugPrint!(b"DeviceRequestSense()\n\0".as_ptr() as *const i8);

    NT_ASSERT(IsPortValid(AdapterExtension, (*Srb).PathId as ULONG) as i32);
    NT_ASSERT(((*Cdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_MODE_SENSE as u32) as i32);

    PortExtension = core::ptr::addr_of_mut!((*AdapterExtension).PortExtension[(*Srb).PathId as usize]);

    if (*PortExtension).DeviceParams.DeviceType == AHCI_DEVICE_TYPE_ATAPI as UCHAR {
        return AhciATAPICommand(AdapterExtension, Srb, Cdb);
    }

    ModeHeader = (*Srb).DataBuffer as PMODE_PARAMETER_HEADER;

    NT_ASSERT((!ModeHeader.is_null()) as i32);

    AhciZeroMemory(ModeHeader as PCHAR, (*Srb).DataTransferLength);

    (*ModeHeader).ModeDataLength = core::mem::size_of::<MODE_PARAMETER_HEADER>() as UCHAR;
    (*ModeHeader).MediumType = 0;
    (*ModeHeader).DeviceSpecificParameter = 0;
    (*ModeHeader).BlockDescriptorLength = 0;

    if (*Cdb).MODE_SENSE.as_ref().PageCode() == MODE_SENSE_CURRENT_VALUES as UCHAR {
        (*ModeHeader).ModeDataLength = (core::mem::size_of::<MODE_PARAMETER_HEADER>()
            + core::mem::size_of::<MODE_PARAMETER_BLOCK>()) as UCHAR;
        (*ModeHeader).BlockDescriptorLength = core::mem::size_of::<MODE_PARAMETER_BLOCK>() as UCHAR;
    }

    return SRB_STATUS_SUCCESS as UCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceRequestReadWrite(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Cdb: PCDB,
) -> UCHAR {

    const AHCI_DEVICE_TYPE_ATAPI: UCHAR = 2;
    const SCSIOP_READ: UCHAR = 0x28;
    const SCSIOP_WRITE: UCHAR = 0x2A;
    const IDE_COMMAND_READ_DMA: UCHAR = 0xC8;
    const IDE_COMMAND_WRITE_DMA: UCHAR = 0xCA;
    const IDE_COMMAND_READ_DMA_EXT: UCHAR = 0x25;
    const IDE_COMMAND_WRITE_DMA_EXT: UCHAR = 0x35;
    const IDE_LBA_MODE: UCHAR = 0x40;
    const SRB_STATUS_PENDING: UCHAR = 0x00;

    let IsReading: BOOLEAN;
    let StartOffset: ULONG64;
    let SrbExtension: PAHCI_SRB_EXTENSION;
    let PortExtension: PAHCI_PORT_EXTENSION;
    let DataTransferLength: ULONG;
    let BytesPerSector: ULONG;
    let SectorCount: ULONG;

    AhciDebugPrint!(b"DeviceRequestReadWrite()\n\0".as_ptr() as *const i8);

    NT_ASSERT(IsPortValid(AdapterExtension, (*Srb).PathId as ULONG));
    NT_ASSERT(
        ((*Cdb).CDB10.as_ref().OperationCode == SCSIOP_READ)
            || ((*Cdb).CDB10.as_ref().OperationCode == SCSIOP_WRITE),
    );

    SrbExtension = GetSrbExtension(Srb);
    PortExtension = core::ptr::addr_of_mut!((*AdapterExtension).PortExtension[(*Srb).PathId as usize]);

    if (*PortExtension).DeviceParams.DeviceType == AHCI_DEVICE_TYPE_ATAPI {
        return AhciATAPICommand(AdapterExtension, Srb, Cdb);
    }

    DataTransferLength = (*Srb).DataTransferLength;
    BytesPerSector = (*PortExtension).DeviceParams.BytesPerLogicalSector;

    NT_ASSERT(BytesPerSector > 0);

    //ROUND_UP(DataTransferLength, BytesPerSector);

    SectorCount = DataTransferLength / BytesPerSector;

    (*Srb).DataTransferLength = SectorCount * BytesPerSector;

    StartOffset = AhciGetLba(Cdb, (*Srb).CdbLength as ULONG);
    IsReading = ((*Cdb).CDB10.as_ref().OperationCode == SCSIOP_READ) as BOOLEAN;

    NT_ASSERT(SectorCount > 0);

    (*SrbExtension).AtaFunction = ATA_FUNCTION_ATA_READ;
    (*SrbExtension).Flags |= ATA_FLAGS_USE_DMA;
    (*SrbExtension).CompletionRoutine = None;

    if IsReading != 0 {
        (*SrbExtension).Flags |= ATA_FLAGS_DATA_IN;
        (*SrbExtension).CommandReg = IDE_COMMAND_READ_DMA;
    } else {
        (*SrbExtension).Flags |= ATA_FLAGS_DATA_OUT;
        (*SrbExtension).CommandReg = IDE_COMMAND_WRITE_DMA;
    }

    (*SrbExtension).FeaturesLow = 0;
    (*SrbExtension).LBA0 = ((StartOffset >> 0) & 0xFF) as UCHAR;
    (*SrbExtension).LBA1 = ((StartOffset >> 8) & 0xFF) as UCHAR;
    (*SrbExtension).LBA2 = ((StartOffset >> 16) & 0xFF) as UCHAR;

    (*SrbExtension).Device = (0xA0 | IDE_LBA_MODE);

    if (*PortExtension).DeviceParams.Lba48BitMode != 0 {
        (*SrbExtension).Flags |= ATA_FLAGS_48BIT_COMMAND;

        if IsReading != 0 {
            (*SrbExtension).CommandReg = IDE_COMMAND_READ_DMA_EXT;
        } else {
            (*SrbExtension).CommandReg = IDE_COMMAND_WRITE_DMA_EXT;
        }

        (*SrbExtension).LBA3 = ((StartOffset >> 24) & 0xFF) as UCHAR;
        (*SrbExtension).LBA4 = ((StartOffset >> 32) & 0xFF) as UCHAR;
        (*SrbExtension).LBA5 = ((StartOffset >> 40) & 0xFF) as UCHAR;
    } else {
        NT_ASSERT(0 != 0);
    }

    (*SrbExtension).FeaturesHigh = 0;
    (*SrbExtension).SectorCountLow = ((SectorCount >> 0) & 0xFF) as UCHAR;
    (*SrbExtension).SectorCountHigh = ((SectorCount >> 8) & 0xFF) as UCHAR;

    NT_ASSERT(SectorCount < 0x100);

    (*SrbExtension).pSgl = StorPortGetScatterGatherList(
        AdapterExtension,
        Srb,
    ) as PLOCAL_SCATTER_GATHER_LIST;

    return SRB_STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceRequestCapacity(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Cdb: PCDB,
) -> UCHAR {

    let mut MaxLba: ULONG;
    let mut BytesPerLogicalSector: ULONG;
    let mut ReadCapacity: PREAD_CAPACITY_DATA;
    let mut PortExtension: PAHCI_PORT_EXTENSION;

    AhciDebugPrint!(b"DeviceRequestCapacity()\n\0".as_ptr() as *const i8);

    // UNREFERENCED_PARAMETER(AdapterExtension);
    let _ = AdapterExtension;
    // UNREFERENCED_PARAMETER(Cdb);
    let _ = Cdb;

    // NT_ASSERT(Srb->DataBuffer != NULL);
    // NT_ASSERT(IsPortValid(AdapterExtension, Srb->PathId));

    PortExtension = core::ptr::addr_of_mut!(
        (*(*AdapterExtension).PortExtension.as_mut_ptr().offset((*Srb).PathId as isize))
    );

    if (*PortExtension).DeviceParams.DeviceType as u32 == AHCI_DEVICE_TYPE_ATAPI {
        return AhciATAPICommand(AdapterExtension, Srb, Cdb);
    }

    if (*Cdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_READ_CAPACITY {
        ReadCapacity = (*Srb).DataBuffer as PREAD_CAPACITY_DATA;

        BytesPerLogicalSector = (*PortExtension).DeviceParams.BytesPerLogicalSector;
        MaxLba = (*PortExtension).DeviceParams.MaxLba.QuadPart as ULONG - 1;

        // I trust you windows :D
        // NT_ASSERT(Srb->DataTransferLength >= sizeof(READ_CAPACITY_DATA));

        // I trust you user :D
        // NT_ASSERT(PortExtension->DeviceParams.MaxLba.QuadPart < (ULONG)-1);

        // Actually I don't trust anyone :p
        (*Srb).DataTransferLength = core::mem::size_of::<READ_CAPACITY_DATA>() as ULONG;

        REVERSE_BYTES(
            core::ptr::addr_of_mut!((*ReadCapacity).BytesPerBlock),
            core::ptr::addr_of_mut!(BytesPerLogicalSector),
        );
        REVERSE_BYTES(
            core::ptr::addr_of_mut!((*ReadCapacity).LogicalBlockAddress),
            core::ptr::addr_of_mut!(MaxLba),
        );
    } else {
        AhciDebugPrint!(b"\tSCSIOP_READ_CAPACITY16 not supported\n\0".as_ptr() as *const i8);
        // NT_ASSERT(FALSE);
    }

    return SRB_STATUS_SUCCESS as UCHAR;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceRequestComplete(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Cdb: PCDB,
) -> UCHAR {

    AhciDebugPrint!(b"DeviceRequestComplete()\n\0".as_ptr() as *const i8);

    let _ = AdapterExtension;
    let _ = Cdb;

    (*Srb).ScsiStatus = SCSISTAT_GOOD as UCHAR;

    return SRB_STATUS_SUCCESS as UCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceReportLuns(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Cdb: PCDB,
) -> UCHAR {

    let LunList: PLUN_LIST;
    let PortExtension: PAHCI_PORT_EXTENSION;

    AhciDebugPrint!(b"DeviceReportLuns()\n\0".as_ptr() as *const i8);

    UNREFERENCED_PARAMETER(Cdb as usize);

    PortExtension = core::ptr::addr_of_mut!(
        (*(*AdapterExtension).PortExtension.as_mut_ptr().offset((*Srb).PathId as isize))
    );

    NT_ASSERT(((*Srb).DataTransferLength as usize >= core::mem::size_of::<LUN_LIST>()) as i32);
    NT_ASSERT(((*Cdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_REPORT_LUNS as u32) as i32);

    if (*PortExtension).DeviceParams.DeviceType == AHCI_DEVICE_TYPE_ATAPI as UCHAR {
        return AhciATAPICommand(AdapterExtension, Srb, Cdb);
    }

    LunList = (*Srb).DataBuffer as PLUN_LIST;

    NT_ASSERT((!LunList.is_null()) as i32);

    AhciZeroMemory(LunList as PCHAR, core::mem::size_of::<LUN_LIST>() as ULONG);

    (*LunList).LunListLength[3] = 8;

    (*Srb).ScsiStatus = SCSISTAT_GOOD as UCHAR;
    (*Srb).DataTransferLength = core::mem::size_of::<LUN_LIST>() as ULONG;

    return SRB_STATUS_SUCCESS as UCHAR;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceInquiryRequest(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Cdb: PCDB,
) -> UCHAR {

    const SCSIOP_INQUIRY: u32 = 0x12;
    const AHCI_DEVICE_TYPE_ATAPI: UCHAR = 2;
    const SRB_STATUS_SELECTION_TIMEOUT: UCHAR = 0x0A;
    const ATA_FLAGS_DATA_IN: u32 = 1 << 1;
    const SRB_STATUS_PENDING: UCHAR = 0x00;
    const SRB_STATUS_INVALID_REQUEST: UCHAR = 0x06;
    const SRB_STATUS_SUCCESS: UCHAR = 0x01;
    const VPD_SUPPORTED_PAGES: u32 = 0x00;
    const VPD_SERIAL_NUMBER: u32 = 0x80;
    const VPD_DEVICE_IDENTIFIERS: u32 = 0x83;
    const DIRECT_ACCESS_DEVICE: UCHAR = 0;

    let DataBuffer: PVOID;
    let SrbExtension: PAHCI_SRB_EXTENSION;
    let PortExtension: PAHCI_PORT_EXTENSION;
    let VpdOutputBuffer: PVPD_SUPPORTED_PAGES_PAGE;
    let mut DataBufferLength: ULONG;
    let mut RequiredDataBufferLength: ULONG;

    AhciDebugPrint!(b"DeviceInquiryRequest()\n\0".as_ptr() as *const i8);

    NT_ASSERT(((*Cdb).CDB10.as_ref().OperationCode as u32 == SCSIOP_INQUIRY) as BOOLEAN);
    NT_ASSERT(IsPortValid(AdapterExtension, (*Srb).PathId as ULONG));

    SrbExtension = GetSrbExtension(Srb);
    PortExtension = core::ptr::addr_of_mut!((*AdapterExtension).PortExtension[(*Srb).PathId as usize]);

    if (*PortExtension).DeviceParams.DeviceType == AHCI_DEVICE_TYPE_ATAPI {
        return AhciATAPICommand(AdapterExtension, Srb, Cdb);
    }

    if (*Srb).Lun != 0 {
        return SRB_STATUS_SELECTION_TIMEOUT;
    } else if (*Cdb).CDB6INQUIRY3.as_ref().EnableVitalProductData() == 0 {
        // 3.6.1
        // If the EVPD bit is set to zero, the device server shall return the standard INQUIRY data
        AhciDebugPrint!(b"\tEVPD Inquired\n\0".as_ptr() as *const i8);
        NT_ASSERT((!SrbExtension.is_null()) as BOOLEAN);

        (*SrbExtension).AtaFunction = ATA_FUNCTION_ATA_IDENTIFY;
        (*SrbExtension).Flags |= ATA_FLAGS_DATA_IN;
        (*SrbExtension).CompletionRoutine = core::mem::transmute::<_, PAHCI_COMPLETION_ROUTINE>(InquiryCompletion as *const ());
        (*SrbExtension).CommandReg = IDE_COMMAND_NOT_VALID as UCHAR;

        // TODO: Should use AhciZeroMemory
        (*SrbExtension).FeaturesLow = 0;
        (*SrbExtension).LBA0 = 0;
        (*SrbExtension).LBA1 = 0;
        (*SrbExtension).LBA2 = 0;
        (*SrbExtension).Device = 0xA0;
        (*SrbExtension).LBA3 = 0;
        (*SrbExtension).LBA4 = 0;
        (*SrbExtension).LBA5 = 0;
        (*SrbExtension).FeaturesHigh = 0;
        (*SrbExtension).SectorCountLow = 0;
        (*SrbExtension).SectorCountHigh = 0;

        (*SrbExtension).Sgl.NumberOfElements = 1;
        (*SrbExtension).Sgl.List[0].PhysicalAddress.__bindgen_anon_1.LowPart =
            (*PortExtension).IdentifyDeviceDataPhysicalAddress.__bindgen_anon_1.LowPart;
        (*SrbExtension).Sgl.List[0].PhysicalAddress.__bindgen_anon_1.HighPart =
            (*PortExtension).IdentifyDeviceDataPhysicalAddress.__bindgen_anon_1.HighPart;
        (*SrbExtension).Sgl.List[0].Length =
            core::mem::size_of::<IDENTIFY_DEVICE_DATA>() as ULONG;

        (*SrbExtension).pSgl = core::ptr::addr_of_mut!((*SrbExtension).Sgl);
        return SRB_STATUS_PENDING;
    } else {
        AhciDebugPrint!(b"\tVPD Inquired\n\0".as_ptr() as *const i8);

        DataBuffer = (*Srb).DataBuffer;
        DataBufferLength = (*Srb).DataTransferLength;
        RequiredDataBufferLength = DataBufferLength; // make the compiler happy :p

        if DataBuffer.is_null() {
            return SRB_STATUS_INVALID_REQUEST;
        }

        AhciZeroMemory(DataBuffer as PCHAR, DataBufferLength);

        match (*Cdb).CDB6INQUIRY3.as_ref().PageCode as u32 {
            VPD_SUPPORTED_PAGES => {
                AhciDebugPrint!(b"\tVPD_SUPPORTED_PAGES\n\0".as_ptr() as *const i8);
                RequiredDataBufferLength =
                    (core::mem::size_of::<VPD_SUPPORTED_PAGES_PAGE>() + 1) as ULONG;

                if DataBufferLength < RequiredDataBufferLength {
                    AhciDebugPrint!(
                        b"\tDataBufferLength: %d Required: %d\n\0".as_ptr() as *const i8,
                        DataBufferLength,
                        RequiredDataBufferLength,
                    );
                    return SRB_STATUS_INVALID_REQUEST;
                }

                VpdOutputBuffer = DataBuffer as PVPD_SUPPORTED_PAGES_PAGE;

                (*VpdOutputBuffer).set_DeviceType((*PortExtension).DeviceParams.AccessType);
                (*VpdOutputBuffer).set_DeviceTypeQualifier(0);
                (*VpdOutputBuffer).PageCode = VPD_SUPPORTED_PAGES as u8;
                (*VpdOutputBuffer).PageLength = 1;
                *(*VpdOutputBuffer).SupportedPageList.as_mut_ptr().offset(0) = VPD_SUPPORTED_PAGES as UCHAR;
                //VpdOutputBuffer->SupportedPageList[1] = VPD_SERIAL_NUMBER;
                //VpdOutputBuffer->SupportedPageList[2] = VPD_DEVICE_IDENTIFIERS;

                NT_ASSERT(((*VpdOutputBuffer).DeviceType() as u32 == DIRECT_ACCESS_DEVICE as u32) as BOOLEAN);
            }
            VPD_SERIAL_NUMBER => {
                AhciDebugPrint!(b"\tVPD_SERIAL_NUMBER\n\0".as_ptr() as *const i8);
            }
            VPD_DEVICE_IDENTIFIERS => {
                AhciDebugPrint!(b"\tVPD_DEVICE_IDENTIFIERS\n\0".as_ptr() as *const i8);
            }
            _ => {
                AhciDebugPrint!(
                    b"\tPageCode: %x\n\0".as_ptr() as *const i8,
                    (*Cdb).CDB6INQUIRY3.as_ref().PageCode as u32,
                );
                return SRB_STATUS_INVALID_REQUEST;
            }
        }

        (*Srb).DataTransferLength = RequiredDataBufferLength;
        return SRB_STATUS_SUCCESS;
    }
}
#[no_mangle]
pub unsafe extern "C" fn AhciAdapterReset(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
) -> BOOLEAN {

    let mut ticks: ULONG;
    let mut ghc: AHCI_GHC = core::mem::zeroed();
    let mut abar: PAHCI_MEMORY_REGISTERS = core::ptr::null_mut();

    AhciDebugPrint!(b"AhciAdapterReset()\n\0".as_ptr() as *const i8);

    abar = (*AdapterExtension).ABAR_Address;
    if abar.is_null()
    {
        return 0;
    }

    // HR -- Very first bit (lowest significant)
    ghc.set_HR(1);
    StorPortWriteRegisterUlong(
        AdapterExtension as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*abar).GHC) as *mut ULONG,
        ghc.status(),
    );

    ticks = 0;
    while ticks < 50 {
        ghc.set_status(StorPortReadRegisterUlong(
            AdapterExtension as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*abar).GHC) as *mut ULONG,
        ));
        if ghc.HR() == 0
        {
            break;
        }
        StorPortStallExecution(20000);
        ticks += 1;
    }

    if ticks == 50
    {
        AhciDebugPrint!(b"\tDevice Timeout\n\0".as_ptr() as *const i8);
        return 0;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn AhciZeroMemory(Buffer: PCHAR, BufferSize: ULONG) {
    let mut i: ULONG;
    i = 0;
    while i < BufferSize {
        *Buffer.offset(i as isize) = 0;
        i = i.wrapping_add(1);
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn IsPortValid(
    AdapterExtension: PAHCI_ADAPTER_EXTENSION,
    pathId: ULONG,
) -> BOOLEAN {
    NT_ASSERT(pathId < MAXIMUM_AHCI_PORT_COUNT);

    if pathId >= (*AdapterExtension).PortCount {
        return 0;
    }

    (*AdapterExtension).PortExtension[pathId as usize].DeviceParams.IsActive
}

#[no_mangle]
pub unsafe extern "C" fn AddQueue(
    Queue: PAHCI_QUEUE,
    Srb: PVOID,
) -> BOOLEAN {
    NT_ASSERT((*Queue).Head < MAXIMUM_QUEUE_BUFFER_SIZE);
    NT_ASSERT((*Queue).Tail < MAXIMUM_QUEUE_BUFFER_SIZE);

    if (*Queue).Tail == (((*Queue).Head + 1) % MAXIMUM_QUEUE_BUFFER_SIZE) {
        return 0;
    }

    let fresh = (*Queue).Head;
    (*Queue).Head += 1;
    (*Queue).Buffer[fresh as usize] = Srb;
    (*Queue).Head %= MAXIMUM_QUEUE_BUFFER_SIZE;

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn RemoveQueue(Queue: PAHCI_QUEUE) -> PVOID {
    let mut Srb: PVOID;

    NT_ASSERT(((*Queue).Head < MAXIMUM_QUEUE_BUFFER_SIZE) as i32);
    NT_ASSERT(((*Queue).Tail < MAXIMUM_QUEUE_BUFFER_SIZE) as i32);

    if (*Queue).Head == (*Queue).Tail {
        return core::ptr::null_mut();
    }

    Srb = (*Queue).Buffer[(*Queue).Tail as usize];
    (*Queue).Tail += 1;
    (*Queue).Tail %= MAXIMUM_QUEUE_BUFFER_SIZE;

    return Srb;
}

unsafe extern "C" fn GetSrbExtension(Srb: PSCSI_REQUEST_BLOCK) -> PAHCI_SRB_EXTENSION {
    let mut Offset: ULONG;
    let mut SrbExtension: ULONG_PTR;

    SrbExtension = (*Srb).SrbExtension as ULONG_PTR;
    Offset = (SrbExtension % 128) as ULONG;

    // CommandTable should be 128 byte aligned
    if Offset != 0 {
        Offset = 128 - Offset;
    }

    (SrbExtension + Offset as ULONG_PTR) as PAHCI_SRB_EXTENSION
}
#[no_mangle]
pub unsafe extern "C" fn AhciGetLba(
    Cdb: PCDB,
    CdbLength: ULONG,
) -> ULONG64 {
    let mut lba: ULONG64 = 0;

    NT_ASSERT((Cdb != core::ptr::null_mut()) as i32);
    NT_ASSERT((CdbLength != 0) as i32);

    if CdbLength == 0x10 {
        REVERSE_BYTES_QUAD(
            core::ptr::addr_of_mut!(lba) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*Cdb).CDB16.as_ref().LogicalBlock) as *const core::ffi::c_void,
        );
    } else {
        lba |= (((*Cdb).CDB10.as_ref().LogicalBlockByte3 as ULONG64) << 0) as ULONG64;
        lba |= (((*Cdb).CDB10.as_ref().LogicalBlockByte2 as ULONG64) << 8) as ULONG64;
        lba |= (((*Cdb).CDB10.as_ref().LogicalBlockByte1 as ULONG64) << 16) as ULONG64;
        lba |= (((*Cdb).CDB10.as_ref().LogicalBlockByte0 as ULONG64) << 24) as ULONG64;
    }

    return lba;
}
