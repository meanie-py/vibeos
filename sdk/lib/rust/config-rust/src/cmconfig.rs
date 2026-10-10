use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    ANSI_STRING, CM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_LIST, CMP_MF_TYPE,
    CONFIGURATION_COMPONENT, INTERFACE_TYPE, NtClose, NtCreateKey, NtSetValueKey,
    _OBJECT_ATTRIBUTES, PCM_FULL_RESOURCE_DESCRIPTOR, PCONFIGURATION_COMPONENT,
    PCONFIGURATION_COMPONENT_DATA, PHANDLE, PLOADER_PARAMETER_BLOCK,
    RtlAnsiStringToUnicodeString, RtlFreeUnicodeString, RtlInitAnsiString,
    RtlInitEmptyUnicodeString, RtlInitUnicodeString, RtlIntegerToChar, _UNICODE_STRING,
    _CONFIGURATION_CLASS_AdapterClass, _CONFIGURATION_CLASS_SystemClass,
    _CONFIGURATION_TYPE_ArcSystem, _CONFIGURATION_TYPE_EisaAdapter,
    _CONFIGURATION_TYPE_TcAdapter, _CONFIGURATION_TYPE_ScsiAdapter,
    _CONFIGURATION_TYPE_MultiFunctionAdapter,
    _INTERFACE_TYPE_Eisa, _INTERFACE_TYPE_TurboChannel, _INTERFACE_TYPE_Internal,
    _INTERFACE_TYPE_InterfaceTypeUndefined,
};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;

macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// KEY_WRITE — defined in winreg.h as STANDARD_RIGHTS_WRITE | KEY_SET_VALUE | KEY_CREATE_SUB_KEY.
const KEY_WRITE: ULONG = STANDARD_RIGHTS_WRITE | KEY_SET_VALUE | KEY_CREATE_SUB_KEY;

// REG_FULL_RESOURCE_DESCRIPTOR — registry value type from winnt.h.
const REG_FULL_RESOURCE_DESCRIPTOR: ULONG = 9;

// MaximumType + 1 from the original C code.
const CM_MAXIMUM_TYPE_COUNT: usize = 42;

// Friendly aliases for bindgen-prefixed enum constants (C enum-style names).
const SystemClass: u32 = _CONFIGURATION_CLASS_SystemClass;
const AdapterClass: u32 = _CONFIGURATION_CLASS_AdapterClass;
const ArcSystem: u32 = _CONFIGURATION_TYPE_ArcSystem;
const EisaAdapter: u32 = _CONFIGURATION_TYPE_EisaAdapter;
const TcAdapter: u32 = _CONFIGURATION_TYPE_TcAdapter;
const ScsiAdapter: u32 = _CONFIGURATION_TYPE_ScsiAdapter;
const MultiFunctionAdapter: u32 = _CONFIGURATION_TYPE_MultiFunctionAdapter;
const Eisa: INTERFACE_TYPE = _INTERFACE_TYPE_Eisa;
const TurboChannel: INTERFACE_TYPE = _INTERFACE_TYPE_TurboChannel;
const Internal: INTERFACE_TYPE = _INTERFACE_TYPE_Internal;
const InterfaceTypeUndefined: INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined;

extern "C" {
    fn _stricmp(s1: *const i8, s2: *const i8) -> i32;

    // Globals from ntoskrnl/config/cmdata.c.
    static mut CmpConfigurationData: PCM_FULL_RESOURCE_DESCRIPTOR;
    static CmpConfigurationAreaSize: ULONG;
    static mut CmpUnknownBusCount: USHORT;
    // CmpTypeCount: ULONG[MaximumType + 1] — MaximumType = _CONFIGURATION_TYPE_MaximumType (41).
    static mut CmpTypeCount: [ULONG; 42];
    // CmTypeName: UNICODE_STRING[MaximumType + 1] — same length. Use bindgen
    // alias `_UNICODE_STRING` so `&mut CmTypeName[i]` matches the
    // `OBJECT_ATTRIBUTES.ObjectName` pointer type (`*mut _UNICODE_STRING`).
    static mut CmTypeName: [_UNICODE_STRING; 42];
    // CmpMultifunctionTypes: CMP_MF_TYPE[] — null-terminated array.
    // Length unknown to Rust; access via raw pointer arithmetic only.
    static mut CmpMultifunctionTypes: CMP_MF_TYPE;
}

#[inline(always)]
unsafe fn cmp_mf_types() -> *mut CMP_MF_TYPE {
    core::ptr::addr_of_mut!(CmpMultifunctionTypes)
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitializeRegistryNode(
    CurrentEntry: PCONFIGURATION_COMPONENT_DATA,
    NodeHandle: HANDLE,
    NewHandle: PHANDLE,
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    DeviceIndexTable: PUSHORT,
) -> NTSTATUS {
    let CurrentEntry = opaque_ptr(CurrentEntry);
    let NodeHandle = opaque_ptr(NodeHandle);
    let NewHandle = opaque_ptr(NewHandle);
    let DeviceIndexTable = opaque_ptr(DeviceIndexTable);
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueData: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ParentHandle: HANDLE;
    let mut TempString: ANSI_STRING = core::mem::zeroed();
    let mut TempBuffer: [CHAR; 12] = [0; 12];
    let mut Buffer: [WCHAR; 12] = [0; 12];
    let mut Component: PCONFIGURATION_COMPONENT;
    let mut Disposition: ULONG = 0;
    let mut Length: ULONG = 0;

    Component = core::ptr::addr_of_mut!((*CurrentEntry).ComponentEntry);

    if (*Component).Class == SystemClass {
        (*Component).Type = ArcSystem;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = NodeHandle;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(CmTypeName[(*Component).Type as usize]);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateKey(
        core::ptr::addr_of_mut!(KeyHandle),
        KEY_READ | KEY_WRITE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(Disposition),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*Component).Class != SystemClass {
        Status = RtlIntegerToChar(
            {
                let idx = (*Component).Type as isize;
                let old_val = *DeviceIndexTable.offset(idx);
                *DeviceIndexTable.offset(idx) = old_val.wrapping_add(1);
                old_val as ULONG
            },
            10,
            12,
            TempBuffer.as_mut_ptr(),
        );

        RtlInitAnsiString(core::ptr::addr_of_mut!(TempString), TempBuffer.as_ptr());

        KeyName.Length = 0;
        KeyName.MaximumLength = core::mem::size_of_val(&Buffer) as USHORT;
        KeyName.Buffer = Buffer.as_mut_ptr();

        Status = RtlAnsiStringToUnicodeString(
            core::ptr::addr_of_mut!(KeyName),
            core::ptr::addr_of_mut!(TempString),
            0u8,
        );
        if !NT_SUCCESS(Status) {
            NtClose(KeyHandle);
            return Status;
        }

        ParentHandle = KeyHandle;

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = ParentHandle;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtCreateKey(
            core::ptr::addr_of_mut!(KeyHandle),
            KEY_READ | KEY_WRITE,
            core::ptr::addr_of_mut!(ObjectAttributes),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(Disposition),
        );
        NtClose(ParentHandle);

        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ValueName),
        w!("Component Information").as_ptr(),
    );
    Status = NtSetValueKey(
        KeyHandle,
        core::ptr::addr_of_mut!(ValueName),
        0,
        REG_BINARY,
        core::ptr::addr_of_mut!((*Component).Flags) as PVOID,
        (core::mem::offset_of!(CONFIGURATION_COMPONENT, ConfigurationDataLength)
            - core::mem::offset_of!(CONFIGURATION_COMPONENT, Flags)) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    if (*Component).IdentifierLength != 0 {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(ValueName),
            w!("Identifier").as_ptr(),
        );
        RtlInitAnsiString(core::ptr::addr_of_mut!(TempString), (*Component).Identifier);
        Status = RtlAnsiStringToUnicodeString(
            core::ptr::addr_of_mut!(ValueData),
            core::ptr::addr_of_mut!(TempString),
            1u8,
        );
        if NT_SUCCESS(Status) {
            Status = NtSetValueKey(
                KeyHandle,
                core::ptr::addr_of_mut!(ValueName),
                0,
                REG_SZ,
                ValueData.Buffer as PVOID,
                ValueData.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG,
            );
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(ValueData));
        }

        if !NT_SUCCESS(Status) {
            NtClose(KeyHandle);
            return Status;
        }
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ValueName),
        w!("Configuration Data").as_ptr(),
    );

    if !(*CurrentEntry).ConfigurationData.is_null() {
        Length = (*Component).ConfigurationDataLength
            + core::mem::offset_of!(CM_FULL_RESOURCE_DESCRIPTOR, PartialResourceList) as ULONG;
        if Length > CmpConfigurationAreaSize {
        } else {
            core::ptr::copy_nonoverlapping(
                (*CurrentEntry).ConfigurationData as *const u8,
                core::ptr::addr_of_mut!((*CmpConfigurationData).PartialResourceList.Version) as *mut u8,
                (*Component).ConfigurationDataLength as usize,
            );
        }
    } else {
        (*CmpConfigurationData).PartialResourceList.Version = 0;
        (*CmpConfigurationData).PartialResourceList.Revision = 0;
        (*CmpConfigurationData).PartialResourceList.Count = 0;
        Length = core::mem::offset_of!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as ULONG
            + core::mem::offset_of!(CM_FULL_RESOURCE_DESCRIPTOR, PartialResourceList) as ULONG;
    }

    (*CmpConfigurationData).InterfaceType = InterfaceType;
    (*CmpConfigurationData).BusNumber = BusNumber;

    Status = NtSetValueKey(
        KeyHandle,
        core::ptr::addr_of_mut!(ValueName),
        0,
        REG_FULL_RESOURCE_DESCRIPTOR,
        CmpConfigurationData as PVOID,
        Length,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
    } else {
        *NewHandle = KeyHandle;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpSetupConfigurationTree(
    mut CurrentEntry: PCONFIGURATION_COMPONENT_DATA,
    ParentHandle: HANDLE,
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
) -> NTSTATUS {
    let mut CurrentEntry = opaque_ptr(CurrentEntry);
    let ParentHandle = opaque_ptr(ParentHandle);
    let mut DeviceIndexTable: [USHORT; CM_MAXIMUM_TYPE_COUNT] = [0; CM_MAXIMUM_TYPE_COUNT];
    let mut Interface: INTERFACE_TYPE = InterfaceType;
    let mut Bus: ULONG = BusNumber;
    let mut i: ULONG;
    let mut Status: NTSTATUS;
    let mut NewHandle: HANDLE = core::ptr::null_mut();
    let mut Component: PCONFIGURATION_COMPONENT;

    while !CurrentEntry.is_null() {
        Component = core::ptr::addr_of_mut!((*CurrentEntry).ComponentEntry);
        if (*Component).Class == _CONFIGURATION_CLASS_AdapterClass &&
            (*(*CurrentEntry).Parent).ComponentEntry.Class == _CONFIGURATION_CLASS_SystemClass
        {
            match (*Component).Type {
                _CONFIGURATION_TYPE_EisaAdapter => {
                    Interface = _INTERFACE_TYPE_Eisa;
                    Bus = CmpTypeCount[_CONFIGURATION_TYPE_EisaAdapter as usize];
                    CmpTypeCount[_CONFIGURATION_TYPE_EisaAdapter as usize] += 1;
                }
                _CONFIGURATION_TYPE_TcAdapter => {
                    Interface = _INTERFACE_TYPE_TurboChannel;
                    Bus = CmpTypeCount[_INTERFACE_TYPE_TurboChannel as usize];
                    CmpTypeCount[_INTERFACE_TYPE_TurboChannel as usize] += 1;
                }
                _CONFIGURATION_TYPE_MultiFunctionAdapter => {
                    if !(*Component).Identifier.is_null() {
                        i = 0;
                        let base = cmp_mf_types();
                        while !(*base.add(i as usize)).Identifier.is_null() {
                            if _stricmp((*base.add(i as usize)).Identifier, (*Component).Identifier) == 0 {
                                break;
                            }
                            i += 1;
                        }
                        Interface = (*base.add(i as usize)).InterfaceType as INTERFACE_TYPE;
                        Bus = (*base.add(i as usize)).Count as ULONG;
                        (*base.add(i as usize)).Count += 1;
                    }
                }
                _CONFIGURATION_TYPE_ScsiAdapter => {
                    Interface = _INTERFACE_TYPE_Internal;
                    Bus = CmpTypeCount[_CONFIGURATION_TYPE_ScsiAdapter as usize];
                    CmpTypeCount[_CONFIGURATION_TYPE_ScsiAdapter as usize] += 1;
                }
                _ => {
                    Interface = _INTERFACE_TYPE_InterfaceTypeUndefined;
                    Bus = CmpUnknownBusCount as ULONG;
                    CmpUnknownBusCount += 1;
                }
            }
        }

        Status = CmpInitializeRegistryNode(
            CurrentEntry,
            ParentHandle,
            core::ptr::addr_of_mut!(NewHandle),
            Interface,
            Bus,
            DeviceIndexTable.as_mut_ptr(),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if !(*CurrentEntry).Child.is_null() {
            Status = CmpSetupConfigurationTree(
                (*CurrentEntry).Child,
                NewHandle,
                Interface,
                Bus,
            );
            if !NT_SUCCESS(Status) {
                NtClose(NewHandle);
                return Status;
            }
        }

        NtClose(NewHandle);
        CurrentEntry = (*CurrentEntry).Sibling;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitializeHardwareConfiguration(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Disposition: ULONG = 0;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(
        &mut KeyName,
        w!("\\Registry\\Machine\\Hardware\\DeviceMap").as_ptr(),
    );
    
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = 0x00000040u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    ARM64_UART_TRACE!(b"[CFG] NtCreateKey DeviceMap\r\n\0".as_ptr());
    Status = NtCreateKey(
        &mut KeyHandle,
        KEY_READ | KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        &mut Disposition,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    NtClose(KeyHandle);
    ARM64_UART_TRACE!(b"[CFG] DeviceMap created\r\n\0".as_ptr());

    RtlInitUnicodeString(
        &mut KeyName,
        w!("\\Registry\\Machine\\Hardware\\Description").as_ptr(),
    );
    
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = 0x00000040u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateKey(
        &mut KeyHandle,
        KEY_READ | KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        &mut Disposition,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    CmpConfigurationData = ExAllocatePoolWithTag(
        PagedPool,
        CmpConfigurationAreaSize as usize,
        0x20206D43u32,
    ) as PCM_FULL_RESOURCE_DESCRIPTOR;
    if CmpConfigurationData.is_null() {
        NtClose(KeyHandle);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ARM64_UART_TRACE!(b"[CFG] Description created\r\n\0".as_ptr());

    ARM64_UART_TRACE!(b"[CFG] ConfigurationRoot=\0".as_ptr());
    if !(*LoaderBlock).ConfigurationRoot.is_null() {
        ARM64_UART_TRACE!(b"non-null\0".as_ptr());
    } else {
        ARM64_UART_TRACE!(b"NULL\0".as_ptr());
    }
    ARM64_UART_TRACE!(b"\r\n\0".as_ptr());
    
    if !(*LoaderBlock).ConfigurationRoot.is_null() {
        Status = CmpSetupConfigurationTree(
            (*LoaderBlock).ConfigurationRoot,
            KeyHandle,
            InterfaceTypeUndefined,
            0xFFFFFFFFu32,
        );
    } else {
        Status = STATUS_SUCCESS;
    }

    ARM64_UART_TRACE!(b"[CFG] CmpSetupConfigurationTree done\r\n\0".as_ptr());

    ExFreePoolWithTag(CmpConfigurationData as *mut core::ffi::c_void, 0x20206D43u32);
    ARM64_UART_TRACE!(b"[CFG] NtClose Description\r\n\0".as_ptr());
    NtClose(KeyHandle);
    ARM64_UART_TRACE!(b"[CFG] CmpInitializeHardwareConfiguration returning\r\n\0".as_ptr());
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmconfig.c
//  * PURPOSE:         Configuration Manager - System Configuration Routines
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// CmpInitializeRegistryNode(IN PCONFIGURATION_COMPONENT_DATA CurrentEntry,
//                           IN HANDLE NodeHandle,
//                           OUT PHANDLE NewHandle,
//                           IN INTERFACE_TYPE InterfaceType,
//                           IN ULONG BusNumber,
//                           IN PUSHORT DeviceIndexTable)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName, ValueName, ValueData;
//     HANDLE KeyHandle, ParentHandle;
//     ANSI_STRING TempString;
//     CHAR TempBuffer[12];
//     WCHAR Buffer[12];
//     PCONFIGURATION_COMPONENT Component;
//     ULONG Disposition, Length = 0;
//
//     /* Get the component */
//     Component = &CurrentEntry->ComponentEntry;
//
//     /* Set system class components to ARC system type */
//     if (Component->Class == SystemClass) Component->Type = ArcSystem;
//
//     /* Create a key for the component */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &CmTypeName[Component->Type],
//                                OBJ_CASE_INSENSITIVE,
//                                NodeHandle,
//                                NULL);
//     Status = NtCreateKey(&KeyHandle,
//                          KEY_READ | KEY_WRITE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          &Disposition);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if this is anything but a system class component */
//     if (Component->Class != SystemClass)
//     {
//         /* Build the sub-component string */
//         RtlIntegerToChar(DeviceIndexTable[Component->Type]++,
//                          10,
//                          12,
//                          TempBuffer);
//         RtlInitAnsiString(&TempString, TempBuffer);
//
//         /* Convert it to Unicode */
//         RtlInitEmptyUnicodeString(&KeyName, Buffer, sizeof(Buffer));
//         Status = RtlAnsiStringToUnicodeString(&KeyName, &TempString, FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             NtClose(KeyHandle);
//             return Status;
//         }
//
//         /* Create the key */
//         ParentHandle = KeyHandle;
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    ParentHandle,
//                                    NULL);
//         Status = NtCreateKey(&KeyHandle,
//                              KEY_READ | KEY_WRITE,
//                              &ObjectAttributes,
//                              0,
//                              NULL,
//                              0,
//                              &Disposition);
//         NtClose(ParentHandle);
//
//         /* Fail if the key couldn't be created, and make sure it's a new key */
//         if (!NT_SUCCESS(Status)) return Status;
//         ASSERT(Disposition == REG_CREATED_NEW_KEY);
//     }
//
//     /* Setup the component information key */
//     RtlInitUnicodeString(&ValueName, L"Component Information");
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_BINARY,
//                            &Component->Flags,
//                            FIELD_OFFSET(CONFIGURATION_COMPONENT,
//                                         ConfigurationDataLength) -
//                            FIELD_OFFSET(CONFIGURATION_COMPONENT, Flags));
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         NtClose(KeyHandle);
//         return Status;
//     }
//
//     /* Check if we have an identifier */
//     if (Component->IdentifierLength)
//     {
//         /* Build the string and convert it to Unicode */
//         RtlInitUnicodeString(&ValueName, L"Identifier");
//         RtlInitAnsiString(&TempString, Component->Identifier);
//         Status = RtlAnsiStringToUnicodeString(&ValueData,
//                                               &TempString,
//                                               TRUE);
//         if (NT_SUCCESS(Status))
//         {
//             /* Save the identifier in the registry */
//             Status = NtSetValueKey(KeyHandle,
//                                    &ValueName,
//                                    0,
//                                    REG_SZ,
//                                    ValueData.Buffer,
//                                    ValueData.Length + sizeof(UNICODE_NULL));
//             RtlFreeUnicodeString(&ValueData);
//         }
//
//         /* Check for failure during conversion or registry write */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             NtClose(KeyHandle);
//             return Status;
//         }
//     }
//
//     /* Setup the configuration data string */
//     RtlInitUnicodeString(&ValueName, L"Configuration Data");
//
//     /* Check if we got configuration data */
//     if (CurrentEntry->ConfigurationData)
//     {
//         /* Calculate the total length and check if it fits into our buffer */
//         Length = Component->ConfigurationDataLength +
//                  FIELD_OFFSET(CM_FULL_RESOURCE_DESCRIPTOR, PartialResourceList);
//         if (Length > CmpConfigurationAreaSize)
//         {
//             ASSERTMSG("Component too large -- need reallocation!\n", FALSE);
//         }
//         else
//         {
//             /* Copy the data */
//             RtlCopyMemory(&CmpConfigurationData->PartialResourceList.Version,
//                           CurrentEntry->ConfigurationData,
//                           Component->ConfigurationDataLength);
//         }
//     }
//     else
//     {
//         /* No configuration data, setup defaults */
//         CmpConfigurationData->PartialResourceList.Version = 0;
//         CmpConfigurationData->PartialResourceList.Revision = 0;
//         CmpConfigurationData->PartialResourceList.Count = 0;
//         Length = FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) +
//                  FIELD_OFFSET(CM_FULL_RESOURCE_DESCRIPTOR, PartialResourceList);
//     }
//
//     /* Set the interface type and bus number */
//     CmpConfigurationData->InterfaceType = InterfaceType;
//     CmpConfigurationData->BusNumber = BusNumber;
//
//     /* Save the actual data */
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_FULL_RESOURCE_DESCRIPTOR,
//                            CmpConfigurationData,
//                            Length);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         NtClose(KeyHandle);
//     }
//     else
//     {
//         /* Return the new handle */
//         *NewHandle = KeyHandle;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// CmpSetupConfigurationTree(IN PCONFIGURATION_COMPONENT_DATA CurrentEntry,
//                           IN HANDLE ParentHandle,
//                           IN INTERFACE_TYPE InterfaceType,
//                           IN ULONG BusNumber)
// {
//     PCONFIGURATION_COMPONENT Component;
//     USHORT DeviceIndexTable[MaximumType + 1] = {0};
//     INTERFACE_TYPE Interface = InterfaceType;
//     ULONG Bus = BusNumber, i;
//     NTSTATUS Status;
//     HANDLE NewHandle;
//
//     /* Loop each entry */
//     while (CurrentEntry)
//     {
//         /* Check if this is an adapter */
//         Component = &CurrentEntry->ComponentEntry;
//         if ((Component->Class == AdapterClass) &&
//             (CurrentEntry->Parent->ComponentEntry.Class == SystemClass))
//         {
//             /* Check what kind of adapter it is */
//             switch (Component->Type)
//             {
//                 /* EISA */
//                 case EisaAdapter:
//                 {
//                     /* Fixup information */
//                     Interface = Eisa;
//                     Bus = CmpTypeCount[EisaAdapter]++;
//                     break;
//                 }
//
//                 /* Turbo-channel */
//                 case TcAdapter:
//                 {
//                     /* Fixup information */
//                     Interface = TurboChannel;
//                     Bus = CmpTypeCount[TurboChannel]++;
//                     break;
//                 }
//
//                 /* ISA, PCI, etc busses */
//                 case MultiFunctionAdapter:
//                 {
//                     /* Check if we have an  identifier */
//                     if (Component->Identifier)
//                     {
//                         /* Loop each multi-function adapter type */
//                         for (i = 0; CmpMultifunctionTypes[i].Identifier; i++)
//                         {
//                             /* Check for a name match */
//                             if (!_stricmp(CmpMultifunctionTypes[i].Identifier,
//                                           Component->Identifier))
//                             {
//                                 /* Match found */
//                                 break;
//                             }
//                         }
//
//                         /* Fix up information */
//                         Interface = CmpMultifunctionTypes[i].InterfaceType;
//                         Bus = CmpMultifunctionTypes[i].Count++;
//                     }
//                     break;
//                 }
//
//                 /* SCSI Bus */
//                 case ScsiAdapter:
//                 {
//                     /* Fix up */
//                     Interface = Internal;
//                     Bus = CmpTypeCount[ScsiAdapter]++;
//                     break;
//                 }
//
//                 /* Unknown */
//                 default:
//                 {
//                     Interface = InterfaceTypeUndefined;
//                     Bus = CmpUnknownBusCount++;
//                     break;
//                 }
//             }
//         }
//
//         /* Dump information on the component */
//
//         /* Setup the hardware node */
//         Status = CmpInitializeRegistryNode(CurrentEntry,
//                                            ParentHandle,
//                                            &NewHandle,
//                                            Interface,
//                                            Bus,
//                                            DeviceIndexTable);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Check for children */
//         if (CurrentEntry->Child)
//         {
//             /* Recurse child */
//             Status = CmpSetupConfigurationTree(CurrentEntry->Child,
//                                                NewHandle,
//                                                Interface,
//                                                Bus);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Fail */
//                 NtClose(NewHandle);
//                 return Status;
//             }
//         }
//
//         /* Get to the next entry */
//         NtClose(NewHandle);
//         CurrentEntry = CurrentEntry->Sibling;
//     }
//
//     /* We're done */
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// CmpInitializeHardwareConfiguration(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle;
//     ULONG Disposition;
//     UNICODE_STRING KeyName;
//
//     /* Set the alternative system architecture information */
// #if defined(SARCH_PC98)
//     SharedUserData->AlternativeArchitecture = NEC98x86;
// #endif
//
//     /* Setup the key name */
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\Hardware\\DeviceMap");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Create the device map key */
//     ARM64_UART_TRACE("[CFG] NtCreateKey DeviceMap\r\n");
//     Status = NtCreateKey(&KeyHandle,
//                          KEY_READ | KEY_WRITE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          &Disposition);
//     if (!NT_SUCCESS(Status))
//         return Status;
//     NtClose(KeyHandle);
//     ARM64_UART_TRACE("[CFG] DeviceMap created\r\n");
//
//     /* Nobody should've created this key yet! */
//     ASSERT(Disposition == REG_CREATED_NEW_KEY);
//
//     /* Setup the key name */
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\Hardware\\Description");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Create the description key */
//     Status = NtCreateKey(&KeyHandle,
//                           KEY_READ | KEY_WRITE,
//                           &ObjectAttributes,
//                           0,
//                           NULL,
//                           0,
//                           &Disposition);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Nobody should've created this key yet! */
//     ASSERT(Disposition == REG_CREATED_NEW_KEY);
//
//     /* Allocate the configuration data buffer */
//     CmpConfigurationData = ExAllocatePoolWithTag(PagedPool,
//                                                  CmpConfigurationAreaSize,
//                                                  TAG_CM);
//     if (!CmpConfigurationData)
//     {
//         NtClose(KeyHandle);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     ARM64_UART_TRACE("[CFG] Description created\r\n");
//
//     /* Check if we got anything from NTLDR */
//     ARM64_UART_TRACE("[CFG] ConfigurationRoot=");
//     if (LoaderBlock->ConfigurationRoot) ARM64_UART_TRACE("non-null"); else ARM64_UART_TRACE("NULL");
//     ARM64_UART_TRACE("\r\n");
//     if (LoaderBlock->ConfigurationRoot)
//     {
//         /* Setup the configuration tree */
//         Status = CmpSetupConfigurationTree(LoaderBlock->ConfigurationRoot,
//                                            KeyHandle,
//                                            InterfaceTypeUndefined,
//                                            -1);
//     }
//     else
//     {
//         /* Nothing else to do */
//         Status = STATUS_SUCCESS;
//     }
//
//     ARM64_UART_TRACE("[CFG] CmpSetupConfigurationTree done\r\n");
//
//     /* Free the buffer, close our handle and return status */
//     ExFreePoolWithTag(CmpConfigurationData, TAG_CM);
//     ARM64_UART_TRACE("[CFG] NtClose Description\r\n");
//     NtClose(KeyHandle);
//     ARM64_UART_TRACE("[CFG] CmpInitializeHardwareConfiguration returning\r\n");
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
