use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{
    // Types
    INTERFACE_TYPE, PINTERFACE_TYPE,
    PHYSICAL_ADDRESS, PPHYSICAL_ADDRESS,
    PCSTR,
    CM_MONITOR_DEVICE_DATA, PCM_MONITOR_DEVICE_DATA,
    CM_PARTIAL_RESOURCE_LIST, PCM_PARTIAL_RESOURCE_LIST, PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    _CM_PARTIAL_RESOURCE_LIST,
    CONFIGURATION_COMPONENT_DATA, PCONFIGURATION_COMPONENT, PCONFIGURATION_COMPONENT_DATA,
    _CONFIGURATION_COMPONENT_DATA,
    // Functions
    HalTranslateBusAddress, KeFindConfigurationNextEntry,
    // _CONFIGURATION_CLASS variants
    _CONFIGURATION_CLASS_AdapterClass as AdapterClass,
    _CONFIGURATION_CLASS_ControllerClass as ControllerClass,
    _CONFIGURATION_CLASS_PeripheralClass as PeripheralClass,
    _CONFIGURATION_CLASS_SystemClass as SystemClass,
    // _CONFIGURATION_TYPE variants
    _CONFIGURATION_TYPE_DisplayController as DisplayController,
    _CONFIGURATION_TYPE_MonitorPeripheral as MonitorPeripheral,
    _CONFIGURATION_TYPE_EisaAdapter as EisaAdapter,
    _CONFIGURATION_TYPE_TcAdapter as TcAdapter,
    _CONFIGURATION_TYPE_ScsiAdapter as ScsiAdapter,
    _CONFIGURATION_TYPE_MultiFunctionAdapter as MultiFunctionAdapter,
    // _IDENTIFIER_FLAG variants
    _IDENTIFIER_FLAG_ConsoleOut as ConsoleOut,
    _IDENTIFIER_FLAG_Output as Output,
    // _INTERFACE_TYPE variants
    _INTERFACE_TYPE_InterfaceTypeUndefined as InterfaceTypeUndefined,
    _INTERFACE_TYPE_Internal as Internal,
    _INTERFACE_TYPE_Isa as Isa,
    _INTERFACE_TYPE_Eisa as Eisa,
    _INTERFACE_TYPE_MicroChannel as MicroChannel,
    _INTERFACE_TYPE_TurboChannel as TurboChannel,
    _INTERFACE_TYPE_PCIBus as PCIBus,
    _INTERFACE_TYPE_VMEBus as VMEBus,
    _INTERFACE_TYPE_PCMCIABus as PCMCIABus,
    _INTERFACE_TYPE_CBus as CBus,
    _INTERFACE_TYPE_MPIBus as MPIBus,
    _INTERFACE_TYPE_MPSABus as MPSABus,
    _INTERFACE_TYPE_MaximumInterfaceType as MaximumInterfaceType,
};
// Framebuffer config-data type + CM_RESOURCE_* / CmResourceType* / STATUS_*
// constants, _stricmp and the HalFindBusAddressTranslation dispatch macro
// live in the shared `pc` substrate module (header types rustconv skips).
use crate::pc::{
    CM_FRAMEBUF_DEVICE_DATA, PCM_FRAMEBUF_DEVICE_DATA,
    CmResourceTypePort, CmResourceTypeInterrupt, CmResourceTypeMemory,
    CmResourceTypeDeviceSpecific,
    CM_RESOURCE_PORT_IO, CM_RESOURCE_MEMORY_READ_ONLY, CM_RESOURCE_MEMORY_WRITEABILITY_MASK,
    STATUS_DEVICE_DOES_NOT_EXIST, STATUS_DEVICE_CONFIGURATION_ERROR,
    _stricmp, HalFindBusAddressTranslation,
};
use core::ffi::c_void;

// KeLoaderBlock is an ntoskrnl DATA export consumed cross-DLL by bootvid.dll
// (FindBootDisplay walks KeLoaderBlock->ConfigurationRoot). bootvid.dll is
// loaded by FREELDR's PE loader, which can't handle the 2nd `ntoskrnl.exe`
// import descriptor `raw-dylib` would add — so reference the import lib's
// `__imp_` IAT thunk directly (single normal descriptor, like the C build).
// `__imp__KeLoaderBlock` (i386) / `__imp_KeLoaderBlock` (amd64/arm64) is a
// slot holding the value of KeLoaderBlock (itself a PLOADER_PARAMETER_BLOCK
// pointer). Read it via `KeLoaderBlock_get()`. See
// feedback_win32k_raw_dylib_freeldr_oom.
extern "C" {
    // i386: rustc prepends one `_`, so `_imp__X` -> import lib's `__imp__X`.
    #[cfg_attr(target_arch = "x86", link_name = "_imp__KeLoaderBlock")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp_KeLoaderBlock")]
    static __imp_KeLoaderBlock: *mut ntkrnl_sys::ddk::PLOADER_PARAMETER_BLOCK;
}
#[inline]
unsafe fn KeLoaderBlock_get() -> ntkrnl_sys::ddk::PLOADER_PARAMETER_BLOCK {
    *__imp_KeLoaderBlock
}

// ---- file-scope preamble (translated) ----
pub const DPRINT_TRACE: () = ();

unsafe fn IsNewConfigData(
    ConfigurationData: *mut core::ffi::c_void,
    ConfigurationDataLength: u32,
) -> u8 {
    let ResourceList: PCM_PARTIAL_RESOURCE_LIST =
        ConfigurationData as PCM_PARTIAL_RESOURCE_LIST;

    if ConfigurationData.is_null() {
        return 0u8;
    }

    let partial_descriptors_offset =
        core::mem::offset_of!(_CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as u32;

    if ConfigurationDataLength < partial_descriptors_offset {
        return 0u8;
    }

    if ((*ResourceList).Version < 1)
        || (((*ResourceList).Version == 1) && ((*ResourceList).Revision <= 1))
    {
        return 0u8;
    }

    1u8
}

unsafe fn GetVideoData(
    ResourceList: PCM_PARTIAL_RESOURCE_LIST,
    Interrupt: *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    ControlPort: *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    CursorPort: *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    VideoRam: *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    DeviceSpecific: *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) {
    let mut Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut PortCount: ULONG = 0;
    let mut IntCount: ULONG = 0;
    let mut MemCount: ULONG = 0;
    let mut i: ULONG;

    if !Interrupt.is_null() {
        *Interrupt = core::ptr::null_mut();
    }
    if !ControlPort.is_null() {
        *ControlPort = core::ptr::null_mut();
    }
    if !CursorPort.is_null() {
        *CursorPort = core::ptr::null_mut();
    }
    *VideoRam = core::ptr::null_mut();
    if !DeviceSpecific.is_null() {
        *DeviceSpecific = core::ptr::null_mut();
    }

    i = 0;
    while i < (*ResourceList).Count {
        // PartialDescriptors is a C ANYSIZE_ARRAY (bindgen `[T; 1]`); indexing
        // `[i]` past 0 panic-aborts (silent hang). Use pointer arithmetic.
        Descriptor = (*ResourceList).PartialDescriptors.as_mut_ptr().add(i as usize);

        match (*Descriptor).Type as u32 {
            x if x == CmResourceTypePort as u32 => {
                if (*Descriptor).Flags & CM_RESOURCE_PORT_IO as u16 != 0 {
                    i += 1;
                    continue;
                }

                if PortCount > 2 {
                    i += 1;
                    continue;
                }
                PortCount += 1;

                if PortCount == 1 {
                    if !ControlPort.is_null() {
                        *ControlPort = Descriptor;
                    }
                } else {
                    if !CursorPort.is_null() {
                        *CursorPort = Descriptor;
                    }
                }
            }

            x if x == CmResourceTypeInterrupt as u32 => {
                if IntCount > 1 {
                    i += 1;
                    continue;
                }
                IntCount += 1;

                if !Interrupt.is_null() {
                    *Interrupt = Descriptor;
                }
            }

            x if x == CmResourceTypeMemory as u32 => {
                if MemCount > 1 {
                    i += 1;
                    continue;
                }
                MemCount += 1;

                if ((*Descriptor).Flags & CM_RESOURCE_MEMORY_WRITEABILITY_MASK as u16)
                    == CM_RESOURCE_MEMORY_READ_ONLY as u16
                {
                    i += 1;
                    continue;
                }
                *VideoRam = Descriptor;
            }

            x if x == CmResourceTypeDeviceSpecific as u32 => {
                i = (*ResourceList).Count;

                if !DeviceSpecific.is_null() {
                    *DeviceSpecific = Descriptor;
                }
            }

            _ => {}
        }

        i += 1;
    }
}

unsafe fn GetMonitorData(
    ResourceList: PCM_PARTIAL_RESOURCE_LIST,
    DeviceSpecific: *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) {
    let mut Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    *DeviceSpecific = core::ptr::null_mut();

    let mut i: u32 = 0;
    while i < (*ResourceList).Count {
        // PartialDescriptors is a C ANYSIZE_ARRAY (bindgen `[T; 1]`); indexing
        // `[i]` past 0 panic-aborts (silent hang). Use pointer arithmetic.
        Descriptor = (*ResourceList).PartialDescriptors.as_mut_ptr().add(i as usize);
        if (*Descriptor).Type as u32 == CmResourceTypeDeviceSpecific {
            if !DeviceSpecific.is_null() {
                *DeviceSpecific = Descriptor;
            }
            break;
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetFramebufferVideoData(
    VideoRamAddress: PPHYSICAL_ADDRESS,
    VideoRamSize: PULONG,
    VideoConfigData: PCM_FRAMEBUF_DEVICE_DATA,
    ConfigurationData: PVOID,
    ConfigurationDataLength: ULONG,
) -> NTSTATUS {
    if ConfigurationData.is_null()
        || (ConfigurationDataLength
            < core::mem::size_of::<CM_PARTIAL_RESOURCE_LIST>() as ULONG)
    {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    if IsNewConfigData(ConfigurationData, ConfigurationDataLength) != 0u8 {
        let mut VideoRam: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
        let mut Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();

        GetVideoData(
            ConfigurationData as PCM_PARTIAL_RESOURCE_LIST,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(VideoRam),
            core::ptr::addr_of_mut!(Descriptor),
        );

        if !VideoRam.is_null() {
            *VideoRamAddress = (*VideoRam).u.Memory.Start;
            *VideoRamSize = (*VideoRam).u.Memory.Length;
        } else {
            (*VideoRamAddress).QuadPart = 0;
            *VideoRamSize = 0;
        }

        if !Descriptor.is_null()
            && ((*Descriptor).u.DeviceSpecificData.DataSize
                >= core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>() as ULONG)
        {
            let VideoData: PCM_FRAMEBUF_DEVICE_DATA =
                Descriptor.add(1) as PCM_FRAMEBUF_DEVICE_DATA;

            *VideoConfigData = *VideoData;
        }

        STATUS_SUCCESS
    } else {
        STATUS_DEVICE_CONFIGURATION_ERROR
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetFramebufferMonitorData(
    MonitorConfigData: PCM_MONITOR_DEVICE_DATA,
    ConfigurationData: *mut core::ffi::c_void,
    ConfigurationDataLength: u32,
) -> NTSTATUS {
    if ConfigurationData.is_null()
        || (ConfigurationDataLength < core::mem::size_of::<CM_PARTIAL_RESOURCE_LIST>() as u32)
    {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    if IsNewConfigData(ConfigurationData, ConfigurationDataLength) != 0 {
        let mut Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();

        GetMonitorData(
            ConfigurationData as PCM_PARTIAL_RESOURCE_LIST,
            core::ptr::addr_of_mut!(Descriptor),
        );

        if !Descriptor.is_null()
            && ((*Descriptor).u.DeviceSpecificData.DataSize
                >= core::mem::size_of::<CM_MONITOR_DEVICE_DATA>() as u32)
        {
            let MonitorData: PCM_MONITOR_DEVICE_DATA =
                Descriptor.add(1) as PCM_MONITOR_DEVICE_DATA;

            *MonitorConfigData = *MonitorData;
        }

        return STATUS_SUCCESS;
    }

    STATUS_DEVICE_CONFIGURATION_ERROR
}

unsafe fn GetArcComponentInterface(
    mut Component: PCONFIGURATION_COMPONENT,
    BusNumber: *mut u32,
) -> INTERFACE_TYPE {
    struct CmpType {
        Identifier: PCSTR,
        InterfaceType: INTERFACE_TYPE,
    }

    let CmpMultifunctionTypes: [CmpType; 9] = [
        CmpType { Identifier: b"ISA\0".as_ptr() as PCSTR, InterfaceType: Isa },
        CmpType { Identifier: b"MCA\0".as_ptr() as PCSTR, InterfaceType: MicroChannel },
        CmpType { Identifier: b"PCI\0".as_ptr() as PCSTR, InterfaceType: PCIBus },
        CmpType { Identifier: b"VME\0".as_ptr() as PCSTR, InterfaceType: VMEBus },
        CmpType { Identifier: b"PCMCIA\0".as_ptr() as PCSTR, InterfaceType: PCMCIABus },
        CmpType { Identifier: b"CBUS\0".as_ptr() as PCSTR, InterfaceType: CBus },
        CmpType { Identifier: b"MPIPI\0".as_ptr() as PCSTR, InterfaceType: MPIBus },
        CmpType { Identifier: b"MPSA\0".as_ptr() as PCSTR, InterfaceType: MPSABus },
        CmpType { Identifier: core::ptr::null(), InterfaceType: Internal },
    ];

    let mut Interface: INTERFACE_TYPE = InterfaceTypeUndefined;

    let mut ComponentData: PCONFIGURATION_COMPONENT_DATA = (Component as *mut u8)
        .offset(-(core::mem::offset_of!(_CONFIGURATION_COMPONENT_DATA, ComponentEntry) as isize))
        as PCONFIGURATION_COMPONENT_DATA;

    while !(*ComponentData).Parent.is_null() {
        if (*(*ComponentData).Parent).ComponentEntry.Class == SystemClass {
            break;
        }
        ComponentData = (*ComponentData).Parent;
    }
    Component = core::ptr::addr_of_mut!((*ComponentData).ComponentEntry);

    if (*ComponentData).Parent.is_null()
        || (*(*ComponentData).Parent).ComponentEntry.Class != SystemClass
        || (*Component).Class != AdapterClass
    {
        return Interface;
    }

    match (*Component).Type {
        x if x == EisaAdapter => {
            Interface = Eisa;
        }
        x if x == TcAdapter => {
            Interface = TurboChannel;
        }
        x if x == MultiFunctionAdapter => {
            if !(*Component).Identifier.is_null() {
                let mut i: u32 = 0;
                while !CmpMultifunctionTypes[i as usize].Identifier.is_null() {
                    if _stricmp(
                        CmpMultifunctionTypes[i as usize].Identifier,
                        (*Component).Identifier,
                    ) == 0
                    {
                        break;
                    }
                    i += 1;
                }
                Interface = CmpMultifunctionTypes[i as usize].InterfaceType;
            }
        }
        x if x == ScsiAdapter => {
            Interface = Internal;
        }
        _ => {
            Interface = InterfaceTypeUndefined;
        }
    }

    if !BusNumber.is_null() {
        *BusNumber = (*Component).Key;
    }
    Interface
}

unsafe fn FindBootDisplayFromLoaderARCTree(
    VideoRamAddress: PPHYSICAL_ADDRESS,
    VideoRamSize: PULONG,
    VideoConfigData: PCM_FRAMEBUF_DEVICE_DATA,
    MonitorConfigData: PCM_MONITOR_DEVICE_DATA,
    Interface: PINTERFACE_TYPE,
    BusNumber: PULONG,
) -> NTSTATUS {
    let ConfigurationRoot: PCONFIGURATION_COMPONENT_DATA;
    let mut Entry: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let Status: NTSTATUS;

    let loader_block = KeLoaderBlock_get();
    ConfigurationRoot = if !loader_block.is_null() {
        (*loader_block).ConfigurationRoot
    } else {
        core::ptr::null_mut()
    };
    if ConfigurationRoot.is_null() {
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    loop {
        Entry = KeFindConfigurationNextEntry(
            ConfigurationRoot,
            ControllerClass,
            DisplayController,
            core::ptr::null_mut(),
            &mut Entry,
        );
        if Entry.is_null() {
            break;
        }
        if (*Entry).ComponentEntry.Flags as u32 & (Output | ConsoleOut) as u32 != 0 {
            break;
        }
    }

    if Entry.is_null() {
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    Status = GetFramebufferVideoData(
        VideoRamAddress,
        VideoRamSize,
        VideoConfigData,
        (*Entry).ConfigurationData,
        (*Entry).ComponentEntry.ConfigurationDataLength,
    );
    if !NT_SUCCESS(Status)
        || ((*VideoRamAddress).QuadPart == 0)
        || (*VideoRamSize == 0)
    {
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    if !Interface.is_null() {
        *Interface = GetArcComponentInterface(
            core::ptr::addr_of_mut!((*Entry).ComponentEntry),
            BusNumber,
        );
    }

    if MonitorConfigData.is_null() {
        return STATUS_SUCCESS;
    }

    Entry = (*Entry).Child;
    if Entry.is_null() {
        return STATUS_SUCCESS;
    }

    if ((*Entry).ComponentEntry.Class != PeripheralClass)
        || ((*Entry).ComponentEntry.Type != MonitorPeripheral)
        || ((*Entry).ComponentEntry.Flags as u32 & (Output | ConsoleOut) as u32 == 0)
    {
        return STATUS_SUCCESS;
    }

    GetFramebufferMonitorData(
        MonitorConfigData,
        (*Entry).ConfigurationData,
        (*Entry).ComponentEntry.ConfigurationDataLength,
    );

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn FindBootDisplay(
    VideoRamAddress: PPHYSICAL_ADDRESS,
    VideoRamSize: PULONG,
    VideoConfigData: PCM_FRAMEBUF_DEVICE_DATA,
    MonitorConfigData: PCM_MONITOR_DEVICE_DATA,
    Interface: PINTERFACE_TYPE,
    BusNumber: PULONG,
) -> NTSTATUS {
    let mut LocalMonitorConfigData: CM_MONITOR_DEVICE_DATA = core::mem::zeroed();
    let mut LocalInterface: INTERFACE_TYPE = core::mem::zeroed();
    let mut LocalBusNumber: ULONG = 0;
    let Status: NTSTATUS;

    Status = FindBootDisplayFromLoaderARCTree(
        VideoRamAddress,
        VideoRamSize,
        VideoConfigData,
        core::ptr::addr_of_mut!(LocalMonitorConfigData),
        core::ptr::addr_of_mut!(LocalInterface),
        core::ptr::addr_of_mut!(LocalBusNumber),
    );
    if !NT_SUCCESS(Status) {
        DbgPrint(b"Boot Display not found\n\0".as_ptr() as PCHAR);
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    /* If the screen sizes are not already initialized by now, use monitor data */
    if (*VideoConfigData).ScreenWidth == 0 || (*VideoConfigData).ScreenHeight == 0 {
        (*VideoConfigData).ScreenWidth = LocalMonitorConfigData.HorizontalResolution as ULONG;
        (*VideoConfigData).ScreenHeight = LocalMonitorConfigData.VerticalResolution as ULONG;
    }

    /* Return any optional data to the caller if required */
    if !MonitorConfigData.is_null() {
        *MonitorConfigData = LocalMonitorConfigData;
    }
    if !Interface.is_null() {
        *Interface = LocalInterface;
    }
    if !BusNumber.is_null() {
        *BusNumber = LocalBusNumber;
    }

    if (*VideoConfigData).ScreenWidth <= 1 || (*VideoConfigData).ScreenHeight <= 1 {
        DbgPrint(b"Cannot obtain current screen resolution\n\0".as_ptr() as PCHAR);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn BootTranslateBusAddress(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: PULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    let mut Context: ULONG_PTR = 0;

    if InterfaceType <= InterfaceTypeUndefined {
        (*TranslatedAddress).QuadPart = BusAddress.QuadPart;
        return 1u8;
    }

    if InterfaceType < MaximumInterfaceType {
        if HalTranslateBusAddress(
            InterfaceType,
            BusNumber,
            BusAddress,
            AddressSpace,
            TranslatedAddress,
        ) != 0
        {
            return 1u8;
        }
    }

    // `HalFindBusAddressTranslation` is the `HALPRIVATEDISPATCH->...`
    // dispatch macro: an Option<fn>. Bail if the HAL hasn't installed one.
    let translate = match HalFindBusAddressTranslation() {
        Some(f) => f,
        None => return 0u8,
    };
    translate(
        BusAddress,
        AddressSpace,
        TranslatedAddress,
        core::ptr::addr_of_mut!(Context),
        1u8,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Boot Video Driver
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Boot-time (POST) display discovery helper functions.
//  * COPYRIGHT:   Copyright 2023-2026 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// #include <arc/arc.h>   // For CONFIGURATION_COMPONENT*
// #include <ndk/kefuncs.h>
// #include <drivers/bootvid/framebuf.h>
//
// #if DBG
// #define DPRINT_TRACE DPRINT1
// #else
// #define DPRINT_TRACE(...)
// #endif
//
// static BOOLEAN
// IsNewConfigData(
//      PVOID ConfigurationData,
//      ULONG ConfigurationDataLength)
// {
//     /* Cast to PCM_PARTIAL_RESOURCE_LIST to access
//      * the common Version and Revision fields */
//     PCM_PARTIAL_RESOURCE_LIST ResourceList =
//         (PCM_PARTIAL_RESOURCE_LIST)ConfigurationData;
//
//     if (!ConfigurationData)
//     {
//         DPRINT_TRACE("IsNewConfigData:FALSE - ConfigurationData == NULL\n");
//         return FALSE;
//     }
//
//     if (ConfigurationDataLength < FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors))
//     {
//         DPRINT_TRACE("IsNewConfigData:FALSE - ConfigurationDataLength == %lu\n",
//                 ConfigurationDataLength);
//         return FALSE;
//     }
//
//     /* If Version/Revision is strictly lower than 1.2, this cannot be
//      * a new configuration data (even if the length appears to match
//      * a CM_FULL_RESOURCE_DESCRIPTOR with zero or more descriptors) */
//     if ( (ResourceList->Version <  1) ||
//         ((ResourceList->Version == 1) && (ResourceList->Revision <= 1)) )
//     {
//         DPRINT_TRACE("IsNewConfigData:FALSE - Version %lu, Revision %lu\n",
//                 ResourceList->Version, ResourceList->Revision);
//         return FALSE;
//     }
//
//     /* This should be a new configuration data */
//     DPRINT_TRACE("IsNewConfigData:TRUE\n");
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Given a CM_PARTIAL_RESOURCE_LIST, obtain pointers to resource descriptors
//  * for legacy video configuration: interrupt, control and cursor I/O ports,
//  * and video RAM memory descriptors. In addition, retrieve any device-specific
//  * resource present.
//  **/
// static VOID
// GetVideoData(
//      PCM_PARTIAL_RESOURCE_LIST ResourceList,
//      PCM_PARTIAL_RESOURCE_DESCRIPTOR* Interrupt,
//      PCM_PARTIAL_RESOURCE_DESCRIPTOR* ControlPort,
//      PCM_PARTIAL_RESOURCE_DESCRIPTOR* CursorPort,
//      PCM_PARTIAL_RESOURCE_DESCRIPTOR* VideoRam,
//      PCM_PARTIAL_RESOURCE_DESCRIPTOR* DeviceSpecific)
// {
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR Descriptor;
//     ULONG PortCount = 0, IntCount = 0, MemCount = 0;
//     ULONG i;
//
//     /* Initialize the return values */
//     if (Interrupt)   *Interrupt   = NULL;
//     if (ControlPort) *ControlPort = NULL;
//     if (CursorPort)  *CursorPort  = NULL;
//     *VideoRam = NULL;
//     if (DeviceSpecific) *DeviceSpecific = NULL;
//
//     DPRINT_TRACE("GetVideoData\n");
//
//     for (i = 0; i < ResourceList->Count; ++i)
//     {
//         Descriptor = &ResourceList->PartialDescriptors[i];
//
//         switch (Descriptor->Type)
//         {
//             case CmResourceTypePort:
//             {
//                 DPRINT_TRACE("    CmResourceTypePort\n");
//                 /* We only check for memory I/O ports */
//                 // if (!(Descriptor->Flags & CM_RESOURCE_PORT_MEMORY))
//                 if (Descriptor->Flags & CM_RESOURCE_PORT_IO)
//                     break;
//
//                 /* If more than two memory I/O ports
//                  * have been encountered, ignore them */
//                 if (PortCount > 2)
//                     break;
//                 ++PortCount;
//
//                 /* First port is Control; second port is Cursor */
//                 if (PortCount == 1)
//                 {
//                     if (ControlPort)
//                         *ControlPort = Descriptor;
//                 }
//                 else // if (PortCount == 2)
//                 {
//                     if (CursorPort)
//                         *CursorPort = Descriptor;
//                 }
//                 break;
//             }
//
//             case CmResourceTypeInterrupt:
//             {
//                 DPRINT_TRACE("    CmResourceTypeInterrupt\n");
//                 /* If more than one interrupt resource
//                  * has been encountered, ignore them */
//                 if (IntCount > 1)
//                     break;
//                 ++IntCount;
//
//                 if (Interrupt)
//                     *Interrupt = Descriptor;
//                 break;
//             }
//
//             case CmResourceTypeMemory:
//             {
//                 DPRINT_TRACE("    CmResourceTypeMemory\n");
//                 /* If more than one memory resource
//                  * has been encountered, ignore them */
//                 if (MemCount > 1)
//                     break;
//                 ++MemCount;
//
//                 /* Video RAM should be writable (but may or may not be readable) */
//                 if ((Descriptor->Flags & CM_RESOURCE_MEMORY_WRITEABILITY_MASK)
//                         == CM_RESOURCE_MEMORY_READ_ONLY)
//                 {
//                     break; /* Cannot use this memory */
//                 }
//                 *VideoRam = Descriptor;
//                 break;
//             }
//
//             case CmResourceTypeDeviceSpecific:
//             {
//                 DPRINT_TRACE("    CmResourceTypeDeviceSpecific\n");
//                 /* NOTE: This descriptor *MUST* be the last one.
//                  * The actual device data follows the descriptor. */
//                 ASSERT(i == ResourceList->Count - 1);
//                 i = ResourceList->Count; // To force-break the for-loop.
//
//                 if (DeviceSpecific)
//                     *DeviceSpecific = Descriptor;
//                 break;
//             }
//         }
//     }
// }
//
// /**
//  * @brief
//  * Given a CM_PARTIAL_RESOURCE_LIST, obtain a pointer to resource descriptor
//  * for monitor configuration data, listed as a device-specific resource.
//  **/
// static VOID
// GetMonitorData(
//      PCM_PARTIAL_RESOURCE_LIST ResourceList,
//      PCM_PARTIAL_RESOURCE_DESCRIPTOR* DeviceSpecific)
// {
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR Descriptor;
//     ULONG i;
//
//     /* Initialize the return values */
//     *DeviceSpecific = NULL;
//
//     /* Find the CmResourceTypeDeviceSpecific CM_MONITOR_DEVICE_DATA */
//     for (i = 0; i < ResourceList->Count; ++i)
//     {
//         Descriptor = &ResourceList->PartialDescriptors[i];
//         if (Descriptor->Type == CmResourceTypeDeviceSpecific)
//         {
//             /* NOTE: This descriptor *MUST* be the last one.
//              * The actual device data follows the descriptor. */
//             ASSERT(i == ResourceList->Count - 1);
//
//             if (DeviceSpecific)
//                 *DeviceSpecific = Descriptor;
//             break;
//         }
//     }
// }
//
// NTSTATUS
// GetFramebufferVideoData(
//      PPHYSICAL_ADDRESS VideoRamAddress,
//      PULONG VideoRamSize,
//      PCM_FRAMEBUF_DEVICE_DATA VideoConfigData,
//      PVOID ConfigurationData,
//      ULONG ConfigurationDataLength)
// {
//     if (!ConfigurationData ||
//         (ConfigurationDataLength < sizeof(CM_PARTIAL_RESOURCE_LIST)))
//     {
//         /* Invalid entry */
//         return STATUS_DEVICE_CONFIGURATION_ERROR;
//     }
//
//     /* Initialize the display adapter parameters, converting
//      * them to the new format if needed */
//     // TODO: Handle legacy VIDEO_HARDWARE_CONFIGURATION_DATA based data
//     if (IsNewConfigData(ConfigurationData, ConfigurationDataLength))
//     {
//         /* New configuration data */
//         PCM_PARTIAL_RESOURCE_DESCRIPTOR VideoRam, Descriptor;
//
//         DPRINT_TRACE("**  FbBVid: New Config data found\n");
//         GetVideoData((PCM_PARTIAL_RESOURCE_LIST)ConfigurationData,
//                      NULL, // Interrupt
//                      NULL, // ControlPort
//                      NULL, // CursorPort
//                      &VideoRam,
//                      &Descriptor);
//
//         if (VideoRam)
//         {
//             /* Save the video RAM base and size */
//             *VideoRamAddress = VideoRam->u.Memory.Start;
//             *VideoRamSize = VideoRam->u.Memory.Length;
//             DPRINT_TRACE("**  FbBVid: Got video RAM address: 0x%I64X - Size: %lu\n",
//                 VideoRamAddress->QuadPart, *VideoRamSize);
//         }
//         else
//         {
//             /* No video RAM base: zero it out */
//             DPRINT1("**  FbBVid: No video RAM available\n");
//             VideoRamAddress->QuadPart = 0;
//             *VideoRamSize = 0;
//         }
//
//         if (Descriptor &&
//             (Descriptor->u.DeviceSpecificData.DataSize >= sizeof(CM_FRAMEBUF_DEVICE_DATA)))
//         {
//             /* NOTE: This descriptor *MUST* be the last one.
//              * The actual device data follows the descriptor. */
//             PCM_FRAMEBUF_DEVICE_DATA VideoData = (PCM_FRAMEBUF_DEVICE_DATA)(Descriptor + 1);
//
//             /* Just copy the data */
//             *VideoConfigData = *VideoData;
//
//             DPRINT_TRACE("**  FbBVid: Framebuffer at: 0x%I64X\n",
//                 VideoRamAddress->QuadPart + VideoData->FrameBufferOffset);
//         }
//         else
//         {
//             /* The configuration data does not contain any
//              * framebuffer information (format, etc.)
//              * We will calculate default values later. */
//             DPRINT1("**  FbBVid: Framebuffer data NOT FOUND\n");
//         }
//
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Unknown configuration, invalid entry? */
//         return STATUS_DEVICE_CONFIGURATION_ERROR;
//     }
//
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// GetFramebufferMonitorData(
//      PCM_MONITOR_DEVICE_DATA MonitorConfigData,
//      PVOID ConfigurationData,
//      ULONG ConfigurationDataLength)
// {
//     if (!ConfigurationData ||
//         (ConfigurationDataLength < sizeof(CM_PARTIAL_RESOURCE_LIST)))
//     {
//         /* Invalid entry */
//         return STATUS_DEVICE_CONFIGURATION_ERROR;
//     }
//
//     /* Initialize the monitor parameters, converting
//      * them to the new format if needed */
//     // TODO: Handle legacy MONITOR_CONFIGURATION_DATA
//     if (IsNewConfigData(ConfigurationData, ConfigurationDataLength))
//     {
//         /* New configuration data */
//         PCM_PARTIAL_RESOURCE_DESCRIPTOR Descriptor;
//
//         GetMonitorData((PCM_PARTIAL_RESOURCE_LIST)ConfigurationData,
//                        &Descriptor);
//
//         if (Descriptor &&
//             (Descriptor->u.DeviceSpecificData.DataSize >= sizeof(CM_MONITOR_DEVICE_DATA)))
//         {
//             /* NOTE: This descriptor *MUST* be the last one.
//              * The actual device data follows the descriptor. */
//             PCM_MONITOR_DEVICE_DATA MonitorData = (PCM_MONITOR_DEVICE_DATA)(Descriptor + 1);
//
//             /* Just copy the data */
//             *MonitorConfigData = *MonitorData;
//         }
//         else
//         {
//             /* The configuration data does not contain any monitor information.
//              * We will calculate default values later. */
//         }
//
//         return STATUS_SUCCESS;
//     }
//
//     /* Unknown configuration, invalid entry? */
//     return STATUS_DEVICE_CONFIGURATION_ERROR;
// }
//
//
// /**
//  * @brief
//  * Retrieve the NT interface type and bus number of a given ARC configuration
//  * component, that can be used in a call to HalTranslateBusAddress() or to
//  * BootTranslateBusAddress().
//  **/
// static INTERFACE_TYPE
// GetArcComponentInterface(
//      PCONFIGURATION_COMPONENT Component,
//      PULONG BusNumber)
// {
//     static const struct
//     {
//         PCSTR Identifier;
//         INTERFACE_TYPE InterfaceType;
//         // USHORT Count;
//     } CmpMultifunctionTypes[] =
//     {
//         {"ISA", Isa},
//         {"MCA", MicroChannel},
//         {"PCI", PCIBus},
//         {"VME", VMEBus},
//         {"PCMCIA", PCMCIABus},
//         {"CBUS", CBus},
//         {"MPIPI", MPIBus},
//         {"MPSA", MPSABus},
//         {NULL, Internal}
//     };
//
//     INTERFACE_TYPE Interface = InterfaceTypeUndefined;
//
//     /* Retrieve the top-level parent adapter component */
//     PCONFIGURATION_COMPONENT_DATA ComponentData =
//         CONTAINING_RECORD(Component, CONFIGURATION_COMPONENT_DATA, ComponentEntry);
//     for (; ComponentData->Parent; ComponentData = ComponentData->Parent)
//     {
//         if (ComponentData->Parent->ComponentEntry.Class == SystemClass)
//             break;
//     }
//     Component = &ComponentData->ComponentEntry;
//
//     /* Check if this is an adapter */
//     if (!ComponentData->Parent ||
//         (ComponentData->Parent->ComponentEntry.Class != SystemClass) ||
//         (Component->Class != AdapterClass))
//     {
//         return Interface; /* It's not, return early */
//     }
//
//     /* Check what kind of adapter it is */
//     switch (Component->Type)
//     {
//         /* EISA */
//         case EisaAdapter:
//         {
//             Interface = Eisa;
//             // Bus = CmpTypeCount[EisaAdapter]++;
//             break;
//         }
//
//         /* Turbo-channel */
//         case TcAdapter:
//         {
//             Interface = TurboChannel;
//             // Bus = CmpTypeCount[TurboChannel]++;
//             break;
//         }
//
//         /* ISA, PCI, etc buses */
//         case MultiFunctionAdapter:
//         {
//             /* Check if we have an identifier */
//             if (Component->Identifier)
//             {
//                 /* Loop each multi-function adapter type */
//                 ULONG i;
//                 for (i = 0; CmpMultifunctionTypes[i].Identifier; i++)
//                 {
//                     /* Check for a name match */
//                     if (!_stricmp(CmpMultifunctionTypes[i].Identifier,
//                                   Component->Identifier))
//                     {
//                         /* Match found */
//                         break;
//                     }
//                 }
//
//                 Interface = CmpMultifunctionTypes[i].InterfaceType;
//                 // Bus = CmpMultifunctionTypes[i].Count++;
//             }
//             break;
//         }
//
//         /* SCSI Bus */
//         case ScsiAdapter:
//         {
//             Interface = Internal;
//             // Bus = CmpTypeCount[ScsiAdapter]++;
//             break;
//         }
//
//         /* Unknown */
//         default:
//         {
//             Interface = InterfaceTypeUndefined;
//             // Bus = CmpUnknownBusCount++;
//             break;
//         }
//     }
//
//     if (BusNumber)
//         *BusNumber = Component->Key;
//     return Interface;
// }
//
// static NTSTATUS
// FindBootDisplayFromLoaderARCTree(
//      PPHYSICAL_ADDRESS VideoRamAddress,
//      PULONG VideoRamSize,
//      PCM_FRAMEBUF_DEVICE_DATA VideoConfigData,
//      PCM_MONITOR_DEVICE_DATA MonitorConfigData,
//      PINTERFACE_TYPE Interface,
//      PULONG BusNumber)
// {
//     PCONFIGURATION_COMPONENT_DATA ConfigurationRoot;
//     PCONFIGURATION_COMPONENT_DATA Entry = NULL;
//     // ULONG ComponentKey = 0; // First controller.
//     NTSTATUS Status;
//
//     ConfigurationRoot = (KeLoaderBlock ? KeLoaderBlock->ConfigurationRoot : NULL);
//     if (!ConfigurationRoot)
//         return STATUS_DEVICE_DOES_NOT_EXIST;
//
//     /* Enumerate and find the boot-time console display controller */
//     while (TRUE)
//     {
//         Entry = KeFindConfigurationNextEntry(ConfigurationRoot,
//                                              ControllerClass,
//                                              DisplayController,
//                                              NULL, // &ComponentKey
//                                              &Entry);
//         if (!Entry)
//             break; /* Not found */
//         if (Entry->ComponentEntry.Flags & (Output | ConsoleOut))
//             break; /* Found it */
//     }
//
//     if (!Entry)
//         return STATUS_DEVICE_DOES_NOT_EXIST;
//
//     // Entry->ComponentEntry.IdentifierLength;
//     DPRINT_TRACE("Display: '%s'\n", Entry->ComponentEntry.Identifier);
//
//     Status = GetFramebufferVideoData(VideoRamAddress,
//                                      VideoRamSize,
//                                      VideoConfigData,
//                                      Entry->ConfigurationData,
//                                      Entry->ComponentEntry.ConfigurationDataLength);
//     if (!NT_SUCCESS(Status) ||
//         (VideoRamAddress->QuadPart == 0) || (*VideoRamSize == 0))
//     {
//         /* Fail if no framebuffer was provided */
//         DPRINT1("No framebuffer found!\n");
//         return STATUS_DEVICE_DOES_NOT_EXIST;
//     }
//
//     if (Interface)
//         *Interface = GetArcComponentInterface(&Entry->ComponentEntry, BusNumber);
//     //if (BusNumber)
//     //    *BusNumber = -1;
//
//     /* If no monitor data to retrieve, just return success */
//     if (!MonitorConfigData)
//         return STATUS_SUCCESS;
//
//     /* Now find the MonitorPeripheral to obtain more information.
//      * It should be a child of the display controller. */
//     Entry = Entry->Child;
//     /* Ignore if no monitor data is given */
//     if (!Entry)
//         return STATUS_SUCCESS;
//
//     // Entry->ComponentEntry.IdentifierLength;
//     DPRINT_TRACE("Monitor: '%s'\n", Entry->ComponentEntry.Identifier);
//
//     if ((Entry->ComponentEntry.Class != PeripheralClass)   ||
//         (Entry->ComponentEntry.Type  != MonitorPeripheral) ||
//        !(Entry->ComponentEntry.Flags & (Output | ConsoleOut)))
//     {
//         /* Ignore */
//         return STATUS_SUCCESS;
//     }
//
//     /* Retrieve monitor configuration data. Use the local variable
//      * since we may need to do some adjustments in the video data
//      * using monitor data, even if the caller does not require the
//      * monitor data itself to be returned. */
//     Status = GetFramebufferMonitorData(MonitorConfigData,
//                                        Entry->ConfigurationData,
//                                        Entry->ComponentEntry.ConfigurationDataLength);
//     if (!NT_SUCCESS(Status))
//         DPRINT_TRACE("Invalid monitor data, ignoring.\n");
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Retrieves configuration data for the boot-time (POST) display controller
//  * and monitor peripheral.
//  *
//  * This data is initialized by the firmware and the NT bootloader,
//  * and can be retrieved from the loader block configuration ARC tree.
//  *
//  * @note
//  * Data from the loader block is available only for boot
//  * drivers and gets freed later on during NT loading.
//  **/
// NTSTATUS
// FindBootDisplay(
//      PPHYSICAL_ADDRESS VideoRamAddress,
//      PULONG VideoRamSize,
//      PCM_FRAMEBUF_DEVICE_DATA VideoConfigData,
//      PCM_MONITOR_DEVICE_DATA MonitorConfigData,
//      PINTERFACE_TYPE Interface,
//      PULONG BusNumber)
// {
//     CM_MONITOR_DEVICE_DATA LocalMonitorConfigData = {0};
//     INTERFACE_TYPE LocalInterface;
//     ULONG LocalBusNumber;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /*
//      * Retrieve video and monitor configuration data.
//      * For monitor data, use the local variable since we may need
//      * to do some adjustments in the video data using monitor data,
//      * even if the caller does not require the monitor data itself
//      * to be returned.
//      */
//     Status = FindBootDisplayFromLoaderARCTree(VideoRamAddress,
//                                               VideoRamSize,
//                                               VideoConfigData,
//                                               &LocalMonitorConfigData,
//                                               &LocalInterface,
//                                               &LocalBusNumber);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Boot Display not found\n");
//         return STATUS_DEVICE_DOES_NOT_EXIST; // STATUS_SYSTEM_DEVICE_NOT_FOUND; (Vista+)
//     }
//
// #if DBG
//     DPRINT_TRACE("\n");
//     DbgPrint("Boot Display found on Interface %lu, Bus %lu\n",
//              LocalInterface, LocalBusNumber);
//     DbgPrint("    VideoRamAddress   : 0x%I64X\n", VideoRamAddress->QuadPart);
//     DbgPrint("    VideoRamSize      : %lu\n", *VideoRamSize);
//     DbgPrint("    Version           : %u.%u\n", VideoConfigData->Version, VideoConfigData->Revision);
//     DbgPrint("    VideoClock        : %lu\n", VideoConfigData->VideoClock);
//     DbgPrint("Framebuffer format:\n");
//     /* Absolute offset from the start of the video RAM of the framebuffer
//      * to be displayed on the monitor. The framebuffer size is obtained by:
//      * FrameBufferSize = ScreenHeight * PixelsPerScanLine * BytesPerPixel */
//     DbgPrint("    BaseAddress       : 0x%I64X\n", VideoRamAddress->QuadPart + VideoConfigData->FrameBufferOffset);
//     // DbgPrint("    BufferSize        : %lu\n", framebufInfo.BufferSize);
//     DbgPrint("    ScreenWidth       : %lu\n", VideoConfigData->ScreenWidth);
//     DbgPrint("    ScreenHeight      : %lu\n", VideoConfigData->ScreenHeight);
//     DbgPrint("    PixelsPerScanLine : %lu\n", VideoConfigData->PixelsPerScanLine);
//     DbgPrint("    BitsPerPixel      : %lu\n", VideoConfigData->BitsPerPixel);
//     DbgPrint("    ARGB masks:       : %08x/%08x/%08x/%08x\n",
//              VideoConfigData->PixelMasks.ReservedMask,
//              VideoConfigData->PixelMasks.RedMask,
//              VideoConfigData->PixelMasks.GreenMask,
//              VideoConfigData->PixelMasks.BlueMask);
// #endif
//
//     /* If the screen sizes are not already initialized by now, use monitor data */
//     // TODO: Investigate: Do we want to do this here, or in the caller?
//     if ((VideoConfigData->ScreenWidth == 0) || (VideoConfigData->ScreenHeight == 0))
//     {
//         VideoConfigData->ScreenWidth  = LocalMonitorConfigData.HorizontalResolution;
//         VideoConfigData->ScreenHeight = LocalMonitorConfigData.VerticalResolution;
// #if DBG
//         DbgPrint("Video screen dimensions not defined; use monitor resolution\n");
//         DbgPrint("    ScreenWidth       : %lu\n", VideoConfigData->ScreenWidth);
//         DbgPrint("    ScreenHeight      : %lu\n", VideoConfigData->ScreenHeight);
// #endif
//     }
//
//     /* Return any optional data to the caller if required */
//     if (MonitorConfigData)
//         *MonitorConfigData = LocalMonitorConfigData;
//     if (Interface)
//         *Interface = LocalInterface;
//     if (BusNumber)
//         *BusNumber = LocalBusNumber;
//
//     if ((VideoConfigData->ScreenWidth <= 1) || (VideoConfigData->ScreenHeight <= 1))
//     {
//         DPRINT1("Cannot obtain current screen resolution\n");
//         /* Don't fail, but the caller will have to cope with it */
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// /**
//  * @brief
//  * Wrapper around HalTranslateBusAddress() and HALPRIVATEDISPATCH->HalFindBusAddressTranslation().
//  *
//  * @note
//  * Context parameter is supplemental from standard HalTranslateBusAddress().
//  * It allows context passing for the underlying HalFindBusAddressTranslation()
//  * between different calls of the function.
//  **/
// #include <ndk/halfuncs.h>
//
// BOOLEAN
// NTAPI
// BootTranslateBusAddress(
//      INTERFACE_TYPE InterfaceType,
//      ULONG BusNumber,
//      PHYSICAL_ADDRESS BusAddress,
//      PULONG AddressSpace,
//      PPHYSICAL_ADDRESS TranslatedAddress/*,
//      PULONG_PTR Context*/)
// {
//     ULONG_PTR Context = 0;
//
//     /* If InterfaceType is negative, don't do any translation */
//     if (InterfaceType <= InterfaceTypeUndefined)
//     {
//         /* Return the bus address */
//         TranslatedAddress->QuadPart = BusAddress.QuadPart;
//         return TRUE;
//     }
//
//     /* If InterfaceType is valid, attempt to call the original HAL function */
//     if (InterfaceType < MaximumInterfaceType)
//     {
//         if (HalTranslateBusAddress(InterfaceType,
//                                    BusNumber,
//                                    BusAddress,
//                                    AddressSpace,
//                                    TranslatedAddress))
//         {
//             return TRUE;
//         }
//         DPRINT1("HalTranslateBusAddress(%lu, %lu, 0x%I64X) failed, fall back to finding a bus translation\n",
//                 InterfaceType, BusNumber, BusAddress.QuadPart);
//     }
//
//     /* Otherwise, in case of failure, or if InterfaceType is greater than
//      * the maximum supported type (meaning we don't know yet the interface
//      * and bus number), we need to find a bus translation. */
//
//     /* Ensure we have a bus translation function then do the call */
//     if (!HalFindBusAddressTranslation)
//         return FALSE;
//     return HalFindBusAddressTranslation(BusAddress,
//                                         AddressSpace,
//                                         TranslatedAddress,
//                                         &Context,
//                                         TRUE);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
