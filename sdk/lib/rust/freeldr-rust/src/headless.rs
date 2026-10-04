use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{CPPORT, HEADLESS_LOADER_BLOCK, PCSTR, PGUID, RtlCompareMemory};
// freeldr_ddk_imports dropped (not in ddk): PortNumber
use core::ffi::c_void;
use crate::ffi::RtlAssert;

// cportlib Cp* are NTAPI (stdcall on i386); libcportlib.a exports
// `_CpInitialize@12` / `_CpPutByte@8` / `_CpDoesPortExist@4` /
// `_CpGetByte@16` / `_CpEnableFifo@8`. Declaring them cdecl
// (`extern "C"`) made the i386 link fail with undefined `_CpInitialize`
// (no `@N`). Must be `extern "system"` (matches rs232.rs).
extern "system" {
    fn CpDoesPortExist(PortAddress: PUCHAR) -> BOOLEAN;
    fn CpInitialize(Port: *mut CPPORT, PortAddress: PUCHAR, BaudRate: ULONG);
    fn CpPutByte(port: *mut CPPORT, byte: UCHAR);
    fn CpGetByte(Port: *mut CPPORT, Byte: *mut u8, Wait: u8, Poll: u8) -> u32;
    fn CpEnableFifo(Address: PUCHAR, Enable: BOOLEAN);
}

extern "C" {
    fn StallExecutionProcessor(Microseconds: ULONG);
    fn strlen(s: *const i8) -> usize;
    fn NtLdrGetOption(BootOptions: PCSTR, OptionName: PCSTR) -> PCSTR;
    fn UiDrawStatusText(Text: PCSTR);
    fn _strnicmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32;
    fn atoi(s: PCSTR) -> i32;
    fn strncmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32;
    fn strtoul(s: PCSTR, endptr: *mut PCSTR, base: i32) -> u32;
    #[cfg(target_pointer_width = "64")]
    fn strtoull(s: PCSTR, endptr: *mut PCHAR, base: i32) -> u64;
}

// ---- rustconv dropped file-local content from headless.c, restored faithfully ----

pub const SYSID_TYPE_UUID: &[u8; 7] = b"_UUID_\0";
pub const SYSID_UUID_DATA_SIZE: ULONG = 16;
// cportlib.h: #define CP_GET_SUCCESS 0
pub const CP_GET_SUCCESS: u32 = 0;

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct _SYSID_UUID_ENTRY {
    pub Type: [UCHAR; 6],
    pub Checksum: UCHAR,
    pub Length: USHORT,
    pub UUID: [UCHAR; SYSID_UUID_DATA_SIZE as usize],
}
pub type SYSID_UUID_ENTRY = _SYSID_UUID_ENTRY;
pub type PSYSID_UUID_ENTRY = *mut _SYSID_UUID_ENTRY;

// C: HEADLESS_LOADER_BLOCK LoaderRedirectionInformation; (external linkage, ref'd by winldr.c)
#[no_mangle]
pub static mut LoaderRedirectionInformation: HEADLESS_LOADER_BLOCK =
    unsafe { core::mem::zeroed() };
// C: BOOLEAN WinLdrTerminalConnected; (external linkage, ref'd by winldr.c — declared u32 there)
#[no_mangle]
pub static mut WinLdrTerminalConnected: BOOLEAN = 0;
#[no_mangle]
pub static mut WinLdrTerminalDeviceId: ULONG = 0;
#[no_mangle]
pub static mut WinLdrTerminalDelay: ULONG = 0;

// C: CPPORT Port[4] = { {NULL,0,TRUE}, ... };
#[no_mangle]
pub static mut Port: [CPPORT; 4] = unsafe { core::mem::zeroed() };

pub unsafe fn WinLdrLoadGUID(SystemGuid: PGUID) {
    // Upstream ea93b886df7 builds EMS on every platform, and only a PC BIOS
    // boot has the SYSID table in the 0xE0000-0xFFFFF BIOS area; elsewhere
    // (UEFI, arm64) the C warns "needs SMBIOS table reading" and returns a
    // zero GUID instead of scanning whatever is mapped there.
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
    {
    let mut CurrentAddress: PSYSID_UUID_ENTRY = 0xE0000 as PSYSID_UUID_ENTRY;
    while (CurrentAddress as ULONG_PTR) < 0x100000 {
        if RtlCompareMemory(
            core::ptr::addr_of!((*CurrentAddress).Type) as *const core::ffi::c_void,
            SYSID_TYPE_UUID.as_ptr() as *const core::ffi::c_void,
            6,
        ) == 6
        {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*CurrentAddress).UUID) as *const u8,
                SystemGuid as *mut u8,
                SYSID_UUID_DATA_SIZE as usize,
            );
            return;
        }
        CurrentAddress = ((CurrentAddress as ULONG_PTR) + 1) as PSYSID_UUID_ENTRY;
    }
    }

    core::ptr::write_bytes(SystemGuid as *mut u8, 0, SYSID_UUID_DATA_SIZE as usize);
}

unsafe fn WinLdrPortInitialize(
    mut BaudRate: ULONG,
    mut PortNumber: ULONG,
    mut PortAddress: PUCHAR,
    TerminalConnected: BOOLEAN,
    PortId: PULONG,
) -> BOOLEAN {

    if BaudRate == 0 {
        BaudRate = 19200;
    }

    if PortNumber != 0 {
        if PortAddress.is_null() {
            match PortNumber {
                1 => {
                    PortAddress = 0x3F8 as PUCHAR;
                }
                2 => {
                    PortAddress = 0x2F8 as PUCHAR;
                }
                3 => {
                    PortAddress = 0x3E8 as PUCHAR;
                }
                _ => {
                    PortNumber = 4;
                    PortAddress = 0x2E8 as PUCHAR;
                }
            }
        }
    } else {
        PortAddress = 0x2F8 as PUCHAR;
        if CpDoesPortExist(PortAddress) != 0 {
            PortNumber = 2;
        } else {
            PortAddress = 0x3F8 as PUCHAR;
            if CpDoesPortExist(PortAddress) == 0 {
                return 0u8;
            }
            PortNumber = 1;
        }
    }

    if LoaderRedirectionInformation.IsMMIODevice != 0 {
        RtlAssert(
            b"LoaderRedirectionInformation.IsMMIODevice == FALSE\0".as_ptr() as *mut core::ffi::c_void,
            b"headless.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }

    if CpDoesPortExist(PortAddress) != 0 || CpDoesPortExist(PortAddress) != 0 {
        if (TerminalConnected != 0 && !Port[(PortNumber - 1) as usize].Address.is_null())
            || Port[(PortNumber - 1) as usize].Address.is_null()
        {
            CpInitialize(
                core::ptr::addr_of_mut!(Port[(PortNumber - 1) as usize]),
                PortAddress,
                BaudRate,
            );
            *PortId = PortNumber - 1;
            return 1u8;
        }
    }

    0u8
}

unsafe fn WinLdrPortPutByte(PortId: ULONG, Byte: UCHAR) {
    CpPutByte(&mut Port[PortId as usize], Byte);
}

unsafe fn WinLdrPortGetByte(PortId: u32, Byte: *mut u8) -> u8 {
    (CpGetByte(Port.as_mut_ptr().add(PortId as usize), Byte, 1u8, 0u8) == CP_GET_SUCCESS) as u8
}

unsafe fn WinLdrPortPollOnly(PortId: u32) -> u8 {
    let mut Dummy: u8 = 0;
    
    
    if CpGetByte(core::ptr::addr_of_mut!(Port[PortId as usize]), &mut Dummy, 0u8, 1u8) == CP_GET_SUCCESS {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrEnableFifo(PortId: ULONG, Enable: BOOLEAN) {
    CpEnableFifo((*Port.as_ptr().add(PortId as usize)).Address, Enable);
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrInitializeHeadlessPort() {

    let PortNumber: ULONG;
    let mut BaudRate: ULONG;
    let mut PortAddress: PUCHAR;
    let AnsiReset: PCSTR = b"\x1B[m\0".as_ptr() as PCSTR;
    let mut i: ULONG;

    PortNumber = LoaderRedirectionInformation.PortNumber;
    PortAddress = LoaderRedirectionInformation.PortAddress;
    BaudRate = LoaderRedirectionInformation.BaudRate;

    if PortNumber != 0 {
        if PortAddress.is_null() {
            match PortNumber {
                2 => {
                    LoaderRedirectionInformation.PortAddress = 0x2F8 as PUCHAR;
                }
                3 => {
                    LoaderRedirectionInformation.PortAddress = 0x3E8 as PUCHAR;
                }
                4 => {
                    LoaderRedirectionInformation.PortAddress = 0x2E8 as PUCHAR;
                }
                _ => {
                    LoaderRedirectionInformation.PortAddress = 0x3F8 as PUCHAR;
                }
            }
        }
    } else {
        WinLdrTerminalConnected = 0u8;
        return;
    }

    PortAddress = LoaderRedirectionInformation.PortAddress;
    WinLdrTerminalConnected = WinLdrPortInitialize(
        BaudRate,
        PortNumber,
        PortAddress,
        WinLdrTerminalConnected,
        core::ptr::addr_of_mut!(WinLdrTerminalDeviceId),
    );
    
    if WinLdrTerminalConnected != 0 {
        WinLdrEnableFifo(WinLdrTerminalDeviceId, 1u8);

        WinLdrLoadGUID(core::ptr::addr_of_mut!(LoaderRedirectionInformation.SystemGUID));

        if BaudRate == 0 {
            BaudRate = 9600;
            LoaderRedirectionInformation.BaudRate = BaudRate;
        }

        WinLdrTerminalDelay = (10 * 1000 * 1000) / (BaudRate / 10) / 6;

        i = 0;
        while i < strlen(AnsiReset as *const i8) as ULONG {
            WinLdrPortPutByte(WinLdrTerminalDeviceId, (*AnsiReset.add(i as usize)) as u8 );
            StallExecutionProcessor(WinLdrTerminalDelay);
            i += 1;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrSetupEms(BootOptions: PCSTR) {

    let mut Option: PCSTR;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(LoaderRedirectionInformation) as *mut core::ffi::c_void,
        core::mem::size_of::<HEADLESS_LOADER_BLOCK>(),
    );
    LoaderRedirectionInformation.PciDeviceId = 0xFFFF;

    Option = NtLdrGetOption(BootOptions, b"redirect=\0".as_ptr() as PCSTR);
    if !Option.is_null() {
        Option = Option.add(9);
        if _strnicmp(Option, b"com\0".as_ptr() as PCSTR, 3) == 0 {
            Option = Option.add(3);
            LoaderRedirectionInformation.PortNumber = atoi(Option) as u32;
            LoaderRedirectionInformation.TerminalType = 1;
        } else if _strnicmp(Option, b"usebiossettings\0".as_ptr() as PCSTR, 15) == 0 {
            UiDrawStatusText(b"ACPI SRT/SPCR Table Not Supported...\0".as_ptr() as PCSTR);
            return;
        } else {
            // strtoulptr (upstream ea93b886df7): strtoull on _WIN64, so an
            // MMIO UART above 4 GiB is not truncated.
            #[cfg(target_pointer_width = "64")]
            {
                LoaderRedirectionInformation.PortAddress =
                    strtoull(Option, core::ptr::null_mut(), 16) as usize as *mut u8;
            }
            #[cfg(not(target_pointer_width = "64"))]
            {
                LoaderRedirectionInformation.PortAddress =
                    strtoul(Option, core::ptr::null_mut(), 16) as usize as *mut u8;
            }
            if !LoaderRedirectionInformation.PortAddress.is_null() {
                LoaderRedirectionInformation.PortNumber = 3;
            }
        }
    }

    Option = NtLdrGetOption(BootOptions, b"redirectbaudrate=\0".as_ptr() as PCSTR);
    if !Option.is_null() {
        Option = Option.add(17);
        if strncmp(Option, b"115200\0".as_ptr() as PCSTR, 6) == 0 {
            LoaderRedirectionInformation.BaudRate = 115200;
        } else if strncmp(Option, b"57600\0".as_ptr() as PCSTR, 5) == 0 {
            LoaderRedirectionInformation.BaudRate = 57600;
        } else if strncmp(Option, b"19200\0".as_ptr() as PCSTR, 5) == 0 {
            LoaderRedirectionInformation.BaudRate = 19200;
        } else {
            LoaderRedirectionInformation.BaudRate = 9600;
        }
    }

    if LoaderRedirectionInformation.PortNumber != 0 {
        if LoaderRedirectionInformation.BaudRate == 0 {
            LoaderRedirectionInformation.BaudRate = 9600;
        }

        WinLdrInitializeHeadlessPort();
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Boot Loader
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            boot/freeldr/freeldr/windows/headless.c
//  * PURPOSE:         Provides support for Windows Emergency Management Services
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
// #include <cportlib/cportlib.h>
// #include "ntldropts.h"
//
// /* Note: Move these to some smbios.h header */
// #define SYSID_TYPE_UUID "_UUID_"
// #define SYSID_UUID_DATA_SIZE 16
// #include <pshpack1.h>
// typedef struct _SYSID_UUID_ENTRY
// {
//     UCHAR Type[6];
//     UCHAR Checksum;
//     USHORT Length;
//     UCHAR UUID[SYSID_UUID_DATA_SIZE];
// } SYSID_UUID_ENTRY, *PSYSID_UUID_ENTRY;
// #include <poppack.h>
//
// /* GLOBALS ********************************************************************/
//
// HEADLESS_LOADER_BLOCK LoaderRedirectionInformation;
// BOOLEAN WinLdrTerminalConnected;
// ULONG WinLdrTerminalDeviceId;
// ULONG WinLdrTerminalDelay;
//
// CPPORT Port[4] =
// {
//     {NULL, 0, TRUE},
//     {NULL, 0, TRUE},
//     {NULL, 0, TRUE},
//     {NULL, 0, TRUE}
// };
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// WinLdrLoadGUID(OUT PGUID SystemGuid)
// {
//     PSYSID_UUID_ENTRY CurrentAddress;
//
//     CurrentAddress = (PSYSID_UUID_ENTRY)0xE0000;
//     while (CurrentAddress < (PSYSID_UUID_ENTRY)0x100000)
//     {
//         if (RtlCompareMemory(&CurrentAddress->Type, SYSID_TYPE_UUID, 6) == 6)
//         {
//             RtlCopyMemory(SystemGuid, &CurrentAddress->UUID, SYSID_UUID_DATA_SIZE);
//             return;
//         }
//         CurrentAddress = (PSYSID_UUID_ENTRY)((ULONG_PTR)CurrentAddress + 1);
//     }
//
//     RtlZeroMemory(SystemGuid, SYSID_UUID_DATA_SIZE);
// }
//
// BOOLEAN
// WinLdrPortInitialize(IN ULONG BaudRate,
//                      IN ULONG PortNumber,
//                      IN PUCHAR PortAddress,
//                      IN BOOLEAN TerminalConnected,
//                      OUT PULONG PortId)
// {
//     /* Set default baud rate */
//     if (BaudRate == 0) BaudRate = 19200;
//
//     /* Check if port or address given */
//     if (PortNumber)
//     {
//         /* Pick correct address for port */
//        if (!PortAddress)
//         {
//             switch (PortNumber)
//             {
//                 case 1:
//                     PortAddress = (PUCHAR)0x3F8;
//                     break;
//
//                 case 2:
//                     PortAddress = (PUCHAR)0x2F8;
//                     break;
//
//                 case 3:
//                     PortAddress = (PUCHAR)0x3E8;
//                     break;
//
//                 default:
//                     PortNumber = 4;
//                     PortAddress = (PUCHAR)0x2E8;
//             }
//         }
//     }
//     else
//     {
//         /* Pick correct port for address */
//         PortAddress = (PUCHAR)0x2F8;
//         if (CpDoesPortExist(PortAddress))
//         {
//             PortNumber = 2;
//         }
//         else
//         {
//             PortAddress = (PUCHAR)0x3F8;
//             if (!CpDoesPortExist(PortAddress)) return FALSE;
//             PortNumber = 1;
//          }
//     }
//
//     /* Not yet supported */
//     ASSERT(LoaderRedirectionInformation.IsMMIODevice == FALSE);
//
//     /* Check if port exists */
//     if ((CpDoesPortExist(PortAddress)) || (CpDoesPortExist(PortAddress)))
//     {
//         /* Initialize port for first time, or re-initialize if specified */
//         if (((TerminalConnected) && (Port[PortNumber - 1].Address)) ||
//             !(Port[PortNumber - 1].Address))
//         {
//             /* Initialize the port, return it */
//             CpInitialize(&Port[PortNumber - 1], PortAddress, BaudRate);
//             *PortId = PortNumber - 1;
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// VOID
// WinLdrPortPutByte(IN ULONG PortId,
//                   IN UCHAR Byte)
// {
//     CpPutByte(&Port[PortId], Byte);
// }
//
// BOOLEAN
// WinLdrPortGetByte(IN  ULONG  PortId,
//                   OUT PUCHAR Byte)
// {
//     return CpGetByte(&Port[PortId], Byte, TRUE, FALSE) == CP_GET_SUCCESS;
// }
//
// BOOLEAN
// WinLdrPortPollOnly(IN ULONG PortId)
// {
//     UCHAR Dummy;
//
//     return CpGetByte(&Port[PortId], &Dummy, FALSE, TRUE) == CP_GET_SUCCESS;
// }
//
// VOID
// WinLdrEnableFifo(IN ULONG PortId,
//                  IN BOOLEAN Enable)
// {
//     CpEnableFifo(Port[PortId].Address, Enable);
// }
//
// VOID
// WinLdrInitializeHeadlessPort(VOID)
// {
//     ULONG PortNumber, BaudRate;
//     PUCHAR PortAddress;
//     PCSTR AnsiReset = "\x1B[m";
//     ULONG i;
//
//     PortNumber = LoaderRedirectionInformation.PortNumber;
//     PortAddress = LoaderRedirectionInformation.PortAddress;
//     BaudRate = LoaderRedirectionInformation.BaudRate;
//
//     /* Pick a port address */
//     if (PortNumber)
//     {
//         if (!PortAddress)
//         {
//             switch (PortNumber)
//             {
//                 case 2:
//                     LoaderRedirectionInformation.PortAddress = (PUCHAR)0x2F8;
//                     break;
//
//                 case 3:
//                     LoaderRedirectionInformation.PortAddress = (PUCHAR)0x3E8;
//                     break;
//
//                 case 4:
//                     LoaderRedirectionInformation.PortAddress = (PUCHAR)0x2E8;
//                     break;
//
//                 default:
//                     LoaderRedirectionInformation.PortAddress = (PUCHAR)0x3F8;
//                     break;
//             }
//         }
//     }
//     else
//     {
//         /* No number, so no EMS */
//         WinLdrTerminalConnected = FALSE;
//         return;
//     }
//
//     /* Call arch code to initialize the port */
//     PortAddress = LoaderRedirectionInformation.PortAddress;
//     WinLdrTerminalConnected = WinLdrPortInitialize(BaudRate,
//                                                    PortNumber,
//                                                    PortAddress,
//                                                    WinLdrTerminalConnected,
//                                                    &WinLdrTerminalDeviceId);
//     if (WinLdrTerminalConnected)
//     {
//         /* Port seems usable, set it up and get the BIOS GUID */
//         WinLdrEnableFifo(WinLdrTerminalDeviceId, TRUE);
//
//         WinLdrLoadGUID(&LoaderRedirectionInformation.SystemGUID);
//
//         /* Calculate delay in us based on the baud, assume 9600 if none given */
//         if (!BaudRate)
//         {
//             BaudRate = 9600;
//             LoaderRedirectionInformation.BaudRate = BaudRate;
//         }
//
//         WinLdrTerminalDelay = (10 * 1000 * 1000) / (BaudRate / 10) / 6;
//
//         /* Sent an ANSI reset sequence to get the terminal up and running */
//         for (i = 0; i < strlen(AnsiReset); i++)
//         {
//             WinLdrPortPutByte(WinLdrTerminalDeviceId, AnsiReset[i]);
//             StallExecutionProcessor(WinLdrTerminalDelay);
//         }
//     }
// }
//
// VOID
// WinLdrSetupEms(IN PCSTR BootOptions)
// {
//     PCSTR Option;
//
//     /* Start fresh */
//     RtlZeroMemory(&LoaderRedirectionInformation, sizeof(HEADLESS_LOADER_BLOCK));
//     LoaderRedirectionInformation.PciDeviceId = PCI_INVALID_VENDORID;
//
//     /* Use a direction port if one was given, or use ACPI to detect one instead */
//     Option = NtLdrGetOption(BootOptions, "redirect=");
//     if (Option)
//     {
//         Option += 9;
//         if (_strnicmp(Option, "com", 3) == 0)
//         {
//             Option += 3;
//             LoaderRedirectionInformation.PortNumber = atoi(Option);
//             LoaderRedirectionInformation.TerminalType = 1; // VT100+
//         }
//         else if (_strnicmp(Option, "usebiossettings", 15) == 0)
//         {
//             // FIXME: TODO!
//             UiDrawStatusText("ACPI SRT/SPCR Table Not Supported...");
//             return;
//         }
//         else
//         {
//             LoaderRedirectionInformation.PortAddress = (PUCHAR)strtoul(Option, 0, 16);
//             if (LoaderRedirectionInformation.PortAddress)
//             {
//                 LoaderRedirectionInformation.PortNumber = 3;
//             }
//         }
//     }
//
//     /* Use a direction baudrate if one was given */
//     Option = NtLdrGetOption(BootOptions, "redirectbaudrate=");
//     if (Option)
//     {
//         Option += 17;
//         // LoaderRedirectionInformation.BaudRate = atoi(Option);
//         if (strncmp(Option, "115200", 6) == 0)
//         {
//             LoaderRedirectionInformation.BaudRate = 115200;
//         }
//         else if (strncmp(Option, "57600", 5) == 0)
//         {
//             LoaderRedirectionInformation.BaudRate = 57600;
//         }
//         else if (strncmp(Option, "19200", 5) == 0)
//         {
//             LoaderRedirectionInformation.BaudRate = 19200;
//         }
//         else
//         {
//             LoaderRedirectionInformation.BaudRate = 9600;
//         }
//     }
//
//     /* Enable headless support if parameters were found */
//     if (LoaderRedirectionInformation.PortNumber)
//     {
//         if (!LoaderRedirectionInformation.BaudRate)
//         {
//             LoaderRedirectionInformation.BaudRate = 9600;
//         }
//
//         WinLdrInitializeHeadlessPort();
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
