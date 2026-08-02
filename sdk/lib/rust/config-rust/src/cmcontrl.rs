use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CmpFindControlSet, CmpFindSubKeyByName, CmpFindValueByName, CmpGetNextName, CmpInitHiveViewList, CmpInitSecurityCache, HCELL_INDEX, LANGID, LCID, LPWSTR, PCMHIVE, PCM_KEY_NODE, PCM_KEY_VALUE, PCM_SYSTEM_CONTROL_VECTOR, PHBASE_BLOCK, PHHIVE, RtlInitUnicodeString, _UNICODE_STRING, _HHIVE};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
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

const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;

extern "system" {
    fn HvInitialize( RegistryHive: PHHIVE, OperationType: u32, HiveFlags: u32, FileType: u32, HiveData: *mut core::ffi::c_void, Allocate: *mut core::ffi::c_void, Free: *mut core::ffi::c_void, FileSetSize: *mut core::ffi::c_void, FileWrite: *mut core::ffi::c_void, FileRead: *mut core::ffi::c_void, FileFlush: *mut core::ffi::c_void, Cluster: u32, FileName: *mut core::ffi::c_void, ) -> u32;
    static mut CmControlHive: [u8; 4096];
    static mut CmDefaultLanguageIdType: u32;
    static mut CmDefaultLanguageId: [u16; 12];
    static mut CmDefaultLanguageIdLength: u32;
    static mut CmInstallUILanguageIdType: u32;
    static mut CmInstallUILanguageId: [u16; 12];
    static mut CmInstallUILanguageIdLength: u32;
    static mut PsDefaultSystemLocaleId: LCID;
    static mut PsInstallUILanguageId: u16;
    static mut PsDefaultThreadLocaleId: LCID;
    static mut PsDefaultUILanguageId: u16;
}
extern "C" {
    fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
    fn CmpIsKeyValueSmall(Length: *mut u32, DataLength: u32) -> u8;
    fn LANGIDFROMLCID(lcid: LCID) -> u16;
}

unsafe fn CmpConvertLangId(Name: LPWSTR, NameLength: u32) -> LANGID {
    let mut LangId: LANGID = 0;
    let mut IdCode: u32;
    
    let NameLength = NameLength / core::mem::size_of::<u16>() as u32;
    
    for i in 0..NameLength {
        let p = *Name.offset(i as isize);
        
        if p >= 0x0030 && p <= 0x0039 {
            IdCode = (p - 0x0030) as u32;
        } else if p >= 0x0041 && p <= 0x0046 {
            IdCode = (p - 0x0041 + 10) as u32;
        } else if p >= 0x0061 && p <= 0x0066 {
            IdCode = (p - 0x0061 + 10) as u32;
        } else {
            break;
        }
        
        if IdCode >= 16 {
            break;
        }
        
        LangId = (LangId << 4) | IdCode as LANGID;
    }
    
    LangId
}

pub unsafe extern "system" fn CmpWalkPath(
    SystemHive: PHHIVE,
    ParentCell: HCELL_INDEX,
    Path: LPWSTR,
) -> HCELL_INDEX {
    let mut UnicodePath: _UNICODE_STRING = core::mem::zeroed();
    let mut NextName: _UNICODE_STRING = core::mem::zeroed();
    let mut LastName: u8 = 0;
    let mut CurrentCell: HCELL_INDEX = ParentCell;
    let mut Node: PCM_KEY_NODE;

    RtlInitUnicodeString(&mut UnicodePath, Path);
    loop {
        CmpGetNextName(&mut UnicodePath, &mut NextName, &mut LastName);
        if NextName.Length == 0 {
            return CurrentCell;
        }

        Node = HvGetCell(SystemHive, CurrentCell) as PCM_KEY_NODE;
        if Node.is_null() {
            return HCELL_NIL;
        }
        CurrentCell = CmpFindSubKeyByName(SystemHive, Node, &mut NextName);
        if CurrentCell == HCELL_NIL {
            return CurrentCell;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmGetSystemControlValues(
    SystemHiveData: *mut core::ffi::c_void,
    ControlVector: PCM_SYSTEM_CONTROL_VECTOR,
) {
    let SystemHiveData = opaque_ptr(SystemHiveData);
    let ControlVector = opaque_ptr(ControlVector);
    
    const HINIT_FLAT: u32 = 4;
    const HIVE_VOLATILE: u32 = 1;
    const HFILE_TYPE_PRIMARY: u32 = 0;
    const BAD_SYSTEM_CONFIG_INFO: u32 = 0x74;
    const CM_KEY_VALUE_SMALL: u32 = 0x4;
    const REG_SZ: u32 = 1;
    
    let SystemHive: PHHIVE = (*(&raw mut CmControlHive)).as_mut_ptr() as PHHIVE;
    let mut Status: u32;
    let mut RootCell: HCELL_INDEX;
    let mut BaseCell: HCELL_INDEX;
    let mut KeyCell: HCELL_INDEX;
    let mut ValueCell: HCELL_INDEX;
    let mut Length: u32;
    let mut DataSize: u32;
    let mut Node: PCM_KEY_NODE;
    let mut ValueData: PCM_KEY_VALUE;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Auto: u8 = 0;
    let mut IsSmallKey: u8;
    let mut Buffer: *mut core::ffi::c_void;
    
    if SystemHiveData.is_null() {
        return;
    }
    
    RtlZeroMemory(SystemHive as *mut core::ffi::c_void, core::mem::size_of::<_HHIVE>());
    CmpInitHiveViewList(SystemHive as PCMHIVE);
    CmpInitSecurityCache(SystemHive as PCMHIVE);
    
    Status = HvInitialize(
        SystemHive,
        HINIT_FLAT,
        HIVE_VOLATILE,
        HFILE_TYPE_PRIMARY,
        SystemHiveData,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        1,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status as i32) {
        KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO, 1, 1, 0, 0);
    }
    
    RootCell = (*(SystemHiveData as PHBASE_BLOCK)).RootCell;
    
    RtlInitUnicodeString(&mut KeyName, w!("current").as_ptr() as *const u16);
    BaseCell = CmpFindControlSet(SystemHive, RootCell, &mut KeyName, &mut Auto);
    if BaseCell == HCELL_NIL {
        KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO, 1, 2, 0, 0);
    }
    
    RtlInitUnicodeString(&mut KeyName, w!("control").as_ptr() as *const u16);
    Node = HvGetCell(SystemHive, BaseCell) as PCM_KEY_NODE;
    BaseCell = CmpFindSubKeyByName(SystemHive, Node, &mut KeyName);
    if BaseCell == HCELL_NIL {
        KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO, 1, 3, 0, 0);
    }
    
    let mut CurrentVector = ControlVector;
    while !(*CurrentVector).KeyPath.is_null() {
        Length = 0xFFFFFFFF;
        
        KeyCell = CmpWalkPath(SystemHive, BaseCell, (*CurrentVector).KeyPath);
        if KeyCell != HCELL_NIL {
            RtlInitUnicodeString(&mut KeyName, (*CurrentVector).ValueName);
            Node = HvGetCell(SystemHive, KeyCell) as PCM_KEY_NODE;
            ValueCell = CmpFindValueByName(SystemHive, Node, &mut KeyName);
            if ValueCell != HCELL_NIL {
                if (*CurrentVector).BufferLength.is_null() {
                    DataSize = core::mem::size_of::<u32>() as u32;
                } else {
                    DataSize = *(*CurrentVector).BufferLength;
                }
                
                ValueData = HvGetCell(SystemHive, ValueCell) as PCM_KEY_VALUE;
                
                IsSmallKey = CmpIsKeyValueSmall(&mut Length, (*ValueData).DataLength);
                
                if DataSize < Length {
                    Length = DataSize;
                }
                
                if Length > 0 {
                    if IsSmallKey != 0 {
                        Buffer = core::ptr::addr_of_mut!((*ValueData).Data) as *mut core::ffi::c_void;
                    } else {
                        Buffer = HvGetCell(SystemHive, (*ValueData).Data);
                    }
                    
                    RtlCopyMemory((*CurrentVector).Buffer, Buffer, Length as usize);
                }
                
                if !(*CurrentVector).Type.is_null() {
                    *(*CurrentVector).Type = (*ValueData).Type;
                }
            }
        }
        
        if !(*CurrentVector).BufferLength.is_null() {
            *(*CurrentVector).BufferLength = Length;
        }
        
        CurrentVector = CurrentVector.offset(1);
    }
    
    if CmDefaultLanguageIdType == REG_SZ {
        PsDefaultSystemLocaleId = CmpConvertLangId(
            (*(&raw mut CmDefaultLanguageId)).as_mut_ptr(),
            CmDefaultLanguageIdLength,
        ) as LCID;
    } else {
        PsDefaultSystemLocaleId = 0x409;
    }
    
    if CmInstallUILanguageIdType == REG_SZ {
        PsInstallUILanguageId = CmpConvertLangId(
            (*(&raw mut CmInstallUILanguageId)).as_mut_ptr(),
            CmInstallUILanguageIdLength,
        );
    } else {
        PsInstallUILanguageId = LANGIDFROMLCID(PsDefaultSystemLocaleId);
    }
    
    PsDefaultThreadLocaleId = PsDefaultSystemLocaleId;
    PsDefaultUILanguageId = PsInstallUILanguageId;
}

#[no_mangle]
pub unsafe extern "system" fn CmpSaveBootControlSet(ControlSet: u16) -> i32 {
    crate::dbg_print!(b"CmpSaveBootControlSet(%lu)\n\0", ControlSet);
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmcontrl.c
//  * PURPOSE:         Configuration Manager - Control Set Management
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS *******************************************************************/
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// LANGID
// NTAPI
// CmpConvertLangId(IN LPWSTR Name,
//                  IN ULONG NameLength)
// {
//     ULONG i;
//     WCHAR p;
//     LANGID LangId = 0;
//     ULONG IdCode;
//
//     /* Convert the length in chars and loop */
//     NameLength = NameLength / sizeof(WCHAR);
//     for (i = 0; i < NameLength; i++)
//     {
//         /* Get the character */
//         p = Name[i];
//
//         /* Handle each case */
//         if ((p >= L'0') && (p <= L'9'))
//         {
//             /* Handle digits*/
//             IdCode = p - L'0';
//         }
//         else if ((p >= L'A') && (p <= L'F'))
//         {
//             /* Handle upper-case letters */
//             IdCode = p - L'A' + 10;
//         }
//         else if ((p >= L'a') && (p <= L'f'))
//         {
//             /* Handle lower-case letters */
//             IdCode = p - L'a' + 10;
//         }
//         else
//         {
//             /* Unhandled case, return what we have till now */
//             break;
//         }
//
//         /* If the ID Code is >= 16, then we're done */
//         if (IdCode >= 16) break;
//
//         /* Build the Language ID */
//         LangId = (LangId << 4) | (LANGID)IdCode;
//     }
//
//     /* Return the Language ID */
//     return LangId;
// }
//
// CODE_SEG("INIT")
// HCELL_INDEX
// NTAPI
// CmpWalkPath(IN PHHIVE SystemHive,
//             IN HCELL_INDEX ParentCell,
//             IN LPWSTR Path)
// {
//     UNICODE_STRING UnicodePath, NextName;
//     BOOLEAN LastName;
//     HCELL_INDEX CurrentCell = ParentCell;
//     PCM_KEY_NODE Node;
//
//     /* We shouldn't have a release routine at this point */
//     ASSERT(SystemHive->ReleaseCellRoutine == NULL);
//
//     /* Initialize the Unicode path and start looping */
//     RtlInitUnicodeString(&UnicodePath, Path);
//     while (TRUE)
//     {
//         /* Get the next name */
//         CmpGetNextName(&UnicodePath, &NextName, &LastName);
//         if (!NextName.Length) return CurrentCell;
//
//         /* Get the subkey */
//         Node = (PCM_KEY_NODE)HvGetCell(SystemHive, CurrentCell);
//         if (!Node) return HCELL_NIL;
//         CurrentCell = CmpFindSubKeyByName(SystemHive, Node, &NextName);
//         if (CurrentCell == HCELL_NIL) return CurrentCell;
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CmGetSystemControlValues(IN PVOID SystemHiveData,
//                          IN PCM_SYSTEM_CONTROL_VECTOR ControlVector)
// {
//     PHHIVE SystemHive = (PHHIVE)&CmControlHive;
//     NTSTATUS Status;
//     HCELL_INDEX RootCell, BaseCell, KeyCell, ValueCell;
//     ULONG Length, DataSize;
//     PCM_KEY_NODE Node;
//     PCM_KEY_VALUE ValueData;
//     UNICODE_STRING KeyName;
//     BOOLEAN Auto, IsSmallKey;
//     PVOID Buffer;
//
//     /* LUDDDIIIICRROOOUUSSSS KI^H^H HACKKKK */
//     if (!SystemHiveData) return;
//
//     /* Initialize the Hive View List and the security cache */
//     RtlZeroMemory(SystemHive, sizeof(*SystemHive));
//     CmpInitHiveViewList((PCMHIVE)SystemHive);
//     CmpInitSecurityCache((PCMHIVE)SystemHive);
//
//     /* Initialize the Hive */
//     Status = HvInitialize(SystemHive,
//                           HINIT_FLAT,
//                           HIVE_VOLATILE,
//                           HFILE_TYPE_PRIMARY,
//                           SystemHiveData,
//                           NULL,
//                           NULL,
//                           NULL,
//                           NULL,
//                           NULL,
//                           NULL,
//                           1,
//                           NULL);
//     if (!NT_SUCCESS(Status)) KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO, 1, 1, 0, 0);
//
//     /* Sanity check, flat hives don't have release routines */
//     ASSERT(SystemHive->ReleaseCellRoutine == NULL);
//
//     /* Set the Root Cell */
//     RootCell = ((PHBASE_BLOCK)SystemHiveData)->RootCell;
//
//     /* Find the current control set */
//     RtlInitUnicodeString(&KeyName, L"current");
//     BaseCell = CmpFindControlSet(SystemHive, RootCell, &KeyName, &Auto);
//     if (BaseCell == HCELL_NIL) KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO, 1, 2, 0, 0);
//
//     /*  Find the control subkey */
//     RtlInitUnicodeString(&KeyName, L"control");
//     Node = (PCM_KEY_NODE)HvGetCell(SystemHive, BaseCell);
//     BaseCell = CmpFindSubKeyByName(SystemHive, Node, &KeyName);
//     if (BaseCell == HCELL_NIL) KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO,1 , 3, 0, 0);
//
//     /* Loop each key */
//     while (ControlVector->KeyPath)
//     {
//         /*  Assume failure */
//         Length = -1;
//
//         /* Get the cell for this key */
//         KeyCell = CmpWalkPath(SystemHive, BaseCell, ControlVector->KeyPath);
//         if (KeyCell != HCELL_NIL)
//         {
//             /* Now get the cell for the value */
//             RtlInitUnicodeString(&KeyName, ControlVector->ValueName);
//             Node = (PCM_KEY_NODE)HvGetCell(SystemHive, KeyCell);
//             ValueCell = CmpFindValueByName(SystemHive, Node, &KeyName);
//             if (ValueCell != HCELL_NIL)
//             {
//                 /* Check if there's any data */
//                 if (!ControlVector->BufferLength)
//                 {
//                     /* No, the buffer will only be large enough for a ULONG */
//                     DataSize = sizeof(ULONG);
//                 }
//                 else
//                 {
//                     /* Yes, save the data size */
//                     DataSize = *ControlVector->BufferLength;
//                 }
//
//                 /* Get the actual data */
//                 ValueData = (PCM_KEY_VALUE)HvGetCell(SystemHive, ValueCell);
//
//                 /* Check if this is a small key */
//                 IsSmallKey = CmpIsKeyValueSmall(&Length, ValueData->DataLength);
//
//                 /* If the length is bigger then our buffer, normalize it */
//                 if (DataSize < Length) Length = DataSize;
//
//                 /* Make sure we have some data */
//                 if (Length > 0)
//                 {
//                     /* Check if this was a small key */
//                     if (IsSmallKey)
//                     {
//                         /* The buffer is directly safe to read */
//                         Buffer = (PVOID)(&(ValueData->Data));
//                     }
//                     else
//                     {
//                         /* Use the longer path */
//                         Buffer = (PVOID)HvGetCell(SystemHive, ValueData->Data);
//                     }
//
//                     /* Sanity check if this is a small key */
//                     ASSERT(IsSmallKey ? (Length <= CM_KEY_VALUE_SMALL) : TRUE);
//
//                     /* Copy the data in the buffer */
//                     RtlCopyMemory(ControlVector->Buffer, Buffer, Length);
//                 }
//
//                 /* Check if we should return the data type */
//                 if (ControlVector->Type)
//                 {
//                     /* Return the type that we read */
//                     *ControlVector->Type = ValueData->Type;
//                 }
//             }
//         }
//
//         /* Return the size that we read */
//         if (ControlVector->BufferLength) *ControlVector->BufferLength = Length;
//
//         /* Go to the next entry */
//         ControlVector++;
//     }
//
//     /* Check if the ID is in the registry */
//     if (CmDefaultLanguageIdType == REG_SZ)
//     {
//         /* Read it */
//         PsDefaultSystemLocaleId =
//             (LCID)CmpConvertLangId(CmDefaultLanguageId,
//                                    CmDefaultLanguageIdLength);
//     }
//     else
//     {
//         /* Use EN_US by default */
//         PsDefaultSystemLocaleId = 0x409;
//     }
//
//     /* Check if the ID Is in the registry */
//     if (CmInstallUILanguageIdType == REG_SZ)
//     {
//         /* Read it */
//         PsInstallUILanguageId =  CmpConvertLangId(CmInstallUILanguageId,
//                                                   CmInstallUILanguageIdLength);
//     }
//     else
//     {
//         /* Otherwise, use the default */
//         PsInstallUILanguageId = LANGIDFROMLCID(PsDefaultSystemLocaleId);
//     }
//
//     /* Set the defaults for the Thread UI */
//     PsDefaultThreadLocaleId = PsDefaultSystemLocaleId;
//     PsDefaultUILanguageId = PsInstallUILanguageId;
// }
//
// NTSTATUS
// NTAPI
// CmpSaveBootControlSet(IN USHORT ControlSet)
// {
//     DPRINT1("CmpSaveBootControlSet(%lu)\n", ControlSet);
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
