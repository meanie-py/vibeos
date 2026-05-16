//! AI-translated from drivers/base/bootvid/ (bootvid); extern blocks
//! stripped — header inlines/macros/FFI provided by crate::pc.

use crate::pc::*;
#[allow(unused_imports)] use crate::fontdata::*;
#[allow(unused_imports)] use crate::bootdata::*;
#[allow(unused_imports)] use crate::common::*;
#[allow(unused_imports)] use crate::console::*;
#[allow(unused_imports)] use crate::vga::*;
use ntkrnl_sys::base::*;
use core::ffi::c_void;

pub unsafe fn VgaInterpretCmdStream(
    CmdStream: *const u16
) -> u8 {
    let mut Cmd: u16;
    let mut Major: u16;
    let mut Minor: u16;
    let mut Port: u16;
    let mut Count: u16;
    let mut Index: u8;
    let mut Value: u8;
    let mut ShortValue: u16;
    let mut CmdStreamPtr = CmdStream;

    if CmdStreamPtr.is_null() {
        return 1u8;
    }

    while *CmdStreamPtr != EOD {
        Cmd = *CmdStreamPtr;
        CmdStreamPtr = CmdStreamPtr.offset(1);
        Major = Cmd & 0xF0;
        Minor = Cmd & 0x0F;

        if Major == INOUT {
            if Minor & IO != 0 {
                if Minor & BW != 0 {
                    Port = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    ShortValue = __inpw(Port);
                } else {
                    Port = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    Value = __inpb(Port);
                }
            } else if Minor & MULTI != 0 {
                if Minor & BW != 0 {
                    Port = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    Count = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);

                    WRITE_PORT_BUFFER_USHORT(
                        (VgaRegisterBase + Port as usize) as *mut u16,
                        CmdStreamPtr,
                        Count as u32
                    );

                    CmdStreamPtr = CmdStreamPtr.offset(Count as isize);
                } else {
                    Port = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    Count = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);

                    let mut i = Count;
                    while i > 0 {
                        Value = (*CmdStreamPtr) as u8;
                        __outpb(Port, Value);
                        i -= 1;
                        CmdStreamPtr = CmdStreamPtr.offset(1);
                    }
                }
            } else if Minor & BW != 0 {
                Port = *CmdStreamPtr;
                CmdStreamPtr = CmdStreamPtr.offset(1);

                ShortValue = *CmdStreamPtr;
                CmdStreamPtr = CmdStreamPtr.offset(1);
                __outpw(Port, ShortValue);
            } else {
                Port = *CmdStreamPtr;
                CmdStreamPtr = CmdStreamPtr.offset(1);

                Value = (*CmdStreamPtr) as u8;
                CmdStreamPtr = CmdStreamPtr.offset(1);
                __outpb(Port, Value);
            }
        } else if Major == METAOUT {
            match Minor {
                INDXOUT => {
                    Port = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    Count = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    Index = (*CmdStreamPtr) as u8;
                    CmdStreamPtr = CmdStreamPtr.offset(1);

                    let mut i = Count;
                    while i > 0 {
                        ShortValue = Index as u16 + ((*CmdStreamPtr) << 8);
                        __outpw(Port, ShortValue);
                        i -= 1;
                        Index = Index.wrapping_add(1);
                        CmdStreamPtr = CmdStreamPtr.offset(1);
                    }
                }

                ATCOUT => {
                    Port = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    Count = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    Index = (*CmdStreamPtr) as u8;
                    CmdStreamPtr = CmdStreamPtr.offset(1);

                    let mut i = Count;
                    while i > 0 {
                        __outpb(Port, Index);

                        Value = (*CmdStreamPtr) as u8;
                        __outpb(Port, Value);
                        i -= 1;
                        Index = Index.wrapping_add(1);
                        CmdStreamPtr = CmdStreamPtr.offset(1);
                    }
                }

                MASKOUT => {
                    Port = *CmdStreamPtr;
                    CmdStreamPtr = CmdStreamPtr.offset(1);

                    Value = __inpb(Port);
                    Value &= (*CmdStreamPtr) as u8;
                    CmdStreamPtr = CmdStreamPtr.offset(1);
                    Value ^= (*CmdStreamPtr) as u8;
                    CmdStreamPtr = CmdStreamPtr.offset(1);

                    __outpb(Port, Value);
                }

                _ => {
                    return 0u8;
                }
            }
        } else if Major != NCMD {
            return 0u8;
        }
    }

    return 1u8;
}

pub unsafe fn VgaIsPresent() -> u8 {
    let mut OrgGCAddr: u8;
    let mut OrgReadMap: u8;
    let mut OrgBitMask: u8;
    let mut OrgSCAddr: u8;
    let mut OrgMemMode: u8;
    let mut i: u8;

    OrgGCAddr = __inpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT);

    __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_READ_MAP);

    if (__inpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT) & GRAPH_ADDR_MASK) != IND_READ_MAP {
        return 0u8;
    }

    OrgReadMap = __inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT);
    __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_TEST_SETTING);

    if __inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT) != READ_MAP_TEST_SETTING {
        __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_DEFAULT);
        return 0u8;
    }

    __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_BIT_MASK);

    if (__inpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT) & GRAPH_ADDR_MASK) != IND_BIT_MASK {
        __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_READ_MAP);
        __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_DEFAULT);
        return 0u8;
    }

    OrgBitMask = __inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT);

    i = 0xBB;
    while i != 0 {
        __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, i);

        if __inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT) != i {
            __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, BIT_MASK_DEFAULT);
            __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_READ_MAP);
            __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_DEFAULT);
            return 0u8;
        }

        i >>= 1;
    }

    __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_READ_MAP);

    if __inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT) != READ_MAP_TEST_SETTING {
        __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_DEFAULT);
        __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_BIT_MASK);
        __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, BIT_MASK_DEFAULT);
        return 0u8;
    }

    __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, OrgReadMap);
    __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_BIT_MASK);
    __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, OrgBitMask);
    __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, OrgGCAddr);

    OrgSCAddr = __inpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT);
    __outpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, IND_MEMORY_MODE);

    if (__inpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT) & SEQ_ADDR_MASK) != IND_MEMORY_MODE {
        return 0u8;
    }

    OrgMemMode = __inpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT);

    __outpw(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, (IND_SYNC_RESET as u16 + ((START_SYNC_RESET_VALUE as u16) << 8)));

    __outpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, IND_MEMORY_MODE);
    __outpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT, OrgMemMode ^ CHAIN4_MASK);

    if __inpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT) != (OrgMemMode ^ CHAIN4_MASK) {
        __outpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT, MEMORY_MODE_TEXT_DEFAULT);

        __outpw(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, (IND_SYNC_RESET as u16 + ((END_SYNC_RESET_VALUE as u16) << 8)));

        return 0u8;
    }

    __outpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT, OrgMemMode);

    __outpw(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, (IND_SYNC_RESET as u16 + ((END_SYNC_RESET_VALUE as u16) << 8)));

    __outpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, OrgSCAddr);

    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn VidInitialize(
    SetMode: u8
) -> u8 {
    let mut Context: usize = 0;
    let mut TranslatedAddress = PHYSICAL_ADDRESS { LowPart: 0, HighPart: 0 };
    let mut NullAddress = PHYSICAL_ADDRESS { LowPart: 0, HighPart: 0 };
    let mut VgaAddress: PHYSICAL_ADDRESS;
    let mut AddressSpace: u32;
    let mut Result: u8;
    let mut Base: usize;

    // C: `if (!HalFindBusAddressTranslation) return FALSE;`
    // (HALPRIVATEDISPATCH->HalFindBusAddressTranslation macro).
    let hpd = HALPRIVATEDISPATCH();
    let fbt = match (*hpd).HalFindBusAddressTranslation {
        None => return 0u8,
        Some(f) => f,
    };

    loop {
        AddressSpace = 1;
        Result = fbt(
            NullAddress,
            &mut AddressSpace,
            &mut TranslatedAddress,
            &mut Context,
            1u8
        );
        if Result == 0 {
            return 0u8;
        }

        if AddressSpace == 0 {
            Base = MmMapIoSpace(TranslatedAddress, 0x400, MmNonCached) as usize;
        } else {
            Base = TranslatedAddress.LowPart as usize;
        }

        VgaRegisterBase = Base;
        if VgaIsPresent() != 0 {
            VgaAddress = PHYSICAL_ADDRESS { LowPart: MEM_VGA, HighPart: 0 };
            AddressSpace = 0;
            Result = fbt(
                VgaAddress,
                &mut AddressSpace,
                &mut TranslatedAddress,
                &mut Context,
                0u8
            );
            if Result != 0 {
                break;
            }
        } else {
            if AddressSpace == 0 {
                MmUnmapIoSpace(VgaRegisterBase as *mut core::ffi::c_void, 0x400);
            }
        }
    }

    if AddressSpace == 0 {
        Base = MmMapIoSpace(
            TranslatedAddress,
            MEM_VGA_SIZE,
            MmNonCached
        ) as usize;
    } else {
        Base = TranslatedAddress.LowPart as usize;
    }

    VgaBase = Base;

    /* Check whether we have to set the video mode */
    if SetMode != 0 {
        VidpCurrentX = 0;
        VidpCurrentY = 0;

        if HalResetDisplay() != 0 {
            /* HAL handled it — re-init only the AC registers */
            VgaInterpretCmdStream(AT_Initialization.as_ptr());
        } else {
            /* HAL didn't — fully re-initialize the VGA */
            VgaInterpretCmdStream(VGA_640x480.as_ptr());
        }
    }

    1u8
}

/// C macro `HalResetDisplay()` = `HALPRIVATEDISPATCH->HalResetDisplay()`.
#[inline]
unsafe fn HalResetDisplay() -> u8 {
    (*HALPRIVATEDISPATCH()).HalResetDisplay.map_or(0u8, |f| f())
}

pub unsafe fn ResetDisplay(
    SetMode: u8
) {

    if SetMode != 0 && HalResetDisplay() == 0 {
        VgaInterpretCmdStream(VGA_640x480.as_ptr());
    }

    VgaInterpretCmdStream(AT_Initialization.as_ptr());

    // C macro: InitPaletteWithTable((PULONG)VidpDefaultPalette, BV_MAX_COLORS)
    InitPaletteWithTable(VidpDefaultPalette.as_ptr() as *mut ULONG, BV_MAX_COLORS as ULONG);
    VidSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLACK as u8);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Boot Video Driver for VGA-compatible cards
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Main file
//  * COPYRIGHT:   Copyright 2007 Alex Ionescu <alex.ionescu@reactos.org>
//  *              Copyright 2019 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// #include "precomp.h"
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// static BOOLEAN
// VgaInterpretCmdStream(
//     _In_ const USHORT* CmdStream)
// {
//     USHORT Cmd;
//     UCHAR Major, Minor;
//     USHORT Port;
//     USHORT Count;
//     UCHAR Index;
//     UCHAR Value;
//     USHORT ShortValue;
//
//     /* First make sure that we have a Command Stream */
//     if (!CmdStream)
//         return TRUE;
//
//     /* Loop as long as we have commands */
//     while (*CmdStream != EOD)
//     {
//         /* Get the next command and its Major and Minor functions */
//         Cmd = *CmdStream++;
//         Major = Cmd & 0xF0;
//         Minor = Cmd & 0x0F;
//
//         /* Check which major function this is */
//         if (Major == INOUT)
//         {
//             /* Check the minor function */
//             if (Minor & IO /* CMD_STREAM_READ */)
//             {
//                 /* Check the sub-type */
//                 if (Minor & BW /* CMD_STREAM_USHORT */)
//                 {
//                     /* Get the port and read an USHORT from it */
//                     Port = *CmdStream++;
//                     ShortValue = __inpw(Port);
//                 }
//                 else // if (Minor & CMD_STREAM_WRITE)
//                 {
//                     /* Get the port and read an UCHAR from it */
//                     Port = *CmdStream++;
//                     Value = __inpb(Port);
//                 }
//             }
//             else if (Minor & MULTI /* CMD_STREAM_WRITE_ARRAY */)
//             {
//                 /* Check the sub-type */
//                 if (Minor & BW /* CMD_STREAM_USHORT */)
//                 {
//                     /* Get the port and the count of elements */
//                     Port = *CmdStream++;
//                     Count = *CmdStream++;
//
//                     /* Write the USHORT to the port; the buffer is what's in the command stream */
//                     WRITE_PORT_BUFFER_USHORT((PUSHORT)(VgaRegisterBase + Port), (PUSHORT)CmdStream, Count);
//
//                     /* Move past the buffer in the command stream */
//                     CmdStream += Count;
//                 }
//                 else // if (Minor & CMD_STREAM_WRITE)
//                 {
//                     /* Get the port and the count of elements */
//                     Port = *CmdStream++;
//                     Count = *CmdStream++;
//
//                     /* Loop the command array */
//                     for (; Count; --Count, ++CmdStream)
//                     {
//                         /* Get the UCHAR and write it to the port */
//                         Value = (UCHAR)*CmdStream;
//                         __outpb(Port, Value);
//                     }
//                 }
//             }
//             else if (Minor & BW /* CMD_STREAM_USHORT */)
//             {
//                 /* Get the port */
//                 Port = *CmdStream++;
//
//                 /* Get the USHORT and write it to the port */
//                 ShortValue = *CmdStream++;
//                 __outpw(Port, ShortValue);
//             }
//             else // if (Minor & CMD_STREAM_WRITE)
//             {
//                 /* Get the port */
//                 Port = *CmdStream++;
//
//                 /* Get the UCHAR and write it to the port */
//                 Value = (UCHAR)*CmdStream++;
//                 __outpb(Port, Value);
//             }
//         }
//         else if (Major == METAOUT)
//         {
//             /* Check the minor function. Note these are not flags. */
//             switch (Minor)
//             {
//                 case INDXOUT:
//                 {
//                     /* Get the port, the count of elements and the start index */
//                     Port = *CmdStream++;
//                     Count = *CmdStream++;
//                     Index = (UCHAR)*CmdStream++;
//
//                     /* Loop the command array */
//                     for (; Count; --Count, ++Index, ++CmdStream)
//                     {
//                         /* Get the USHORT and write it to the port */
//                         ShortValue = (USHORT)Index + ((*CmdStream) << 8);
//                         __outpw(Port, ShortValue);
//                     }
//                     break;
//                 }
//
//                 case ATCOUT:
//                 {
//                     /* Get the port, the count of elements and the start index */
//                     Port = *CmdStream++;
//                     Count = *CmdStream++;
//                     Index = (UCHAR)*CmdStream++;
//
//                     /* Loop the command array */
//                     for (; Count; --Count, ++Index, ++CmdStream)
//                     {
//                         /* Write the index */
//                         __outpb(Port, Index);
//
//                         /* Get the UCHAR and write it to the port */
//                         Value = (UCHAR)*CmdStream;
//                         __outpb(Port, Value);
//                     }
//                     break;
//                 }
//
//                 case MASKOUT:
//                 {
//                     /* Get the port */
//                     Port = *CmdStream++;
//
//                     /* Read the current value and add the stream data */
//                     Value = __inpb(Port);
//                     Value &= *CmdStream++;
//                     Value ^= *CmdStream++;
//
//                     /* Write the value */
//                     __outpb(Port, Value);
//                     break;
//                 }
//
//                 default:
//                     /* Unknown command, fail */
//                     return FALSE;
//             }
//         }
//         else if (Major != NCMD)
//         {
//             /* Unknown major function, fail */
//             return FALSE;
//         }
//     }
//
//     /* If we got here, return success */
//     return TRUE;
// }
//
// static BOOLEAN
// VgaIsPresent(VOID)
// {
//     UCHAR OrgGCAddr, OrgReadMap, OrgBitMask;
//     UCHAR OrgSCAddr, OrgMemMode;
//     UCHAR i;
//
//     /* Remember the original state of the Graphics Controller Address register */
//     OrgGCAddr = __inpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT);
//
//     /*
//      * Write the Read Map register with a known state so we can verify
//      * that it isn't changed after we fool with the Bit Mask. This ensures
//      * that we're dealing with indexed registers, since both the Read Map and
//      * the Bit Mask are addressed at GRAPH_DATA_PORT.
//      */
//     __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_READ_MAP);
//
//     /*
//      * If we can't read back the Graphics Address register setting we just
//      * performed, it's not readable and this isn't a VGA.
//      */
//     if ((__inpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT) & GRAPH_ADDR_MASK) != IND_READ_MAP)
//         return FALSE;
//
//     /*
//      * Set the Read Map register to a known state.
//      */
//     OrgReadMap = __inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT);
//     __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_TEST_SETTING);
//
//     /* Read it back... it should be the same */
//     if (__inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT) != READ_MAP_TEST_SETTING)
//     {
//         /*
//          * The Read Map setting we just performed can't be read back; not a
//          * VGA. Restore the default Read Map state and fail.
//          */
//         __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_DEFAULT);
//         return FALSE;
//     }
//
//     /* Remember the original setting of the Bit Mask register */
//     __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_BIT_MASK);
//
//     /* Read it back... it should be the same */
//     if ((__inpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT) & GRAPH_ADDR_MASK) != IND_BIT_MASK)
//     {
//         /*
//          * The Graphics Address register setting we just made can't be read
//          * back; not a VGA. Restore the default Read Map state and fail.
//          */
//         __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_READ_MAP);
//         __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_DEFAULT);
//         return FALSE;
//     }
//
//     /* Read the VGA Data Register */
//     OrgBitMask = __inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT);
//
//     /*
//      * Set up the initial test mask we'll write to and read from the Bit Mask,
//      * and loop on the bitmasks.
//      */
//     for (i = 0xBB; i; i >>= 1)
//     {
//         /* Write the test mask to the Bit Mask */
//         __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, i);
//
//         /* Read it back... it should be the same */
//         if (__inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT) != i)
//         {
//             /*
//              * The Bit Mask is not properly writable and readable; not a VGA.
//              * Restore the Bit Mask and Read Map to their default states and fail.
//              */
//             __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, BIT_MASK_DEFAULT);
//             __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_READ_MAP);
//             __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_DEFAULT);
//             return FALSE;
//         }
//     }
//
//     /*
//      * There's something readable at GRAPH_DATA_PORT; now switch back and
//      * make sure that the Read Map register hasn't changed, to verify that
//      * we're dealing with indexed registers.
//      */
//     __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_READ_MAP);
//
//     /* Read it back */
//     if (__inpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT) != READ_MAP_TEST_SETTING)
//     {
//         /*
//          * The Read Map is not properly writable and readable; not a VGA.
//          * Restore the Bit Mask and Read Map to their default states, in case
//          * this is an EGA, so subsequent writes to the screen aren't garbled.
//          * Then fail.
//          */
//         __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, READ_MAP_DEFAULT);
//         __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_BIT_MASK);
//         __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, BIT_MASK_DEFAULT);
//         return FALSE;
//     }
//
//     /*
//      * We've pretty surely verified the existence of the Bit Mask register.
//      * Put the Graphics Controller back to the original state.
//      */
//     __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, OrgReadMap);
//     __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, IND_BIT_MASK);
//     __outpb(VGA_BASE_IO_PORT + GRAPH_DATA_PORT, OrgBitMask);
//     __outpb(VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, OrgGCAddr);
//
//     /*
//      * Now, check for the existence of the Chain4 bit.
//      */
//
//     /*
//      * Remember the original states of the Sequencer Address and Memory Mode
//      * registers.
//      */
//     OrgSCAddr = __inpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT);
//     __outpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, IND_MEMORY_MODE);
//
//     /* Read it back... it should be the same */
//     if ((__inpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT) & SEQ_ADDR_MASK) != IND_MEMORY_MODE)
//     {
//         /*
//          * Couldn't read back the Sequencer Address register setting
//          * we just performed, fail.
//          */
//         return FALSE;
//     }
//
//     /* Read sequencer Data */
//     OrgMemMode = __inpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT);
//
//     /*
//      * Toggle the Chain4 bit and read back the result. This must be done during
//      * sync reset, since we're changing the chaining state.
//      */
//
//     /* Begin sync reset */
//     __outpw(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, (IND_SYNC_RESET + (START_SYNC_RESET_VALUE << 8)));
//
//     /* Toggle the Chain4 bit */
//     __outpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, IND_MEMORY_MODE);
//     __outpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT, OrgMemMode ^ CHAIN4_MASK);
//
//     /* Read it back... it should be the same */
//     if (__inpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT) != (OrgMemMode ^ CHAIN4_MASK))
//     {
//         /*
//          * Chain4 bit is not there, not a VGA.
//          * Set text mode default for Memory Mode register.
//          */
//         __outpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT, MEMORY_MODE_TEXT_DEFAULT);
//
//         /* End sync reset */
//         __outpw(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, (IND_SYNC_RESET + (END_SYNC_RESET_VALUE << 8)));
//
//         /* Fail */
//         return FALSE;
//     }
//
//     /*
//      * It's a VGA.
//      */
//
//     /* Restore the original Memory Mode setting */
//     __outpb(VGA_BASE_IO_PORT + SEQ_DATA_PORT, OrgMemMode);
//
//     /* End sync reset */
//     __outpw(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, (IND_SYNC_RESET + (END_SYNC_RESET_VALUE << 8)));
//
//     /* Restore the original Sequencer Address setting */
//     __outpb(VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, OrgSCAddr);
//
//     /* VGA is present! */
//     return TRUE;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// BOOLEAN
// NTAPI
// VidInitialize(
//     _In_ BOOLEAN SetMode)
// {
//     ULONG_PTR Context = 0;
//     PHYSICAL_ADDRESS TranslatedAddress;
//     PHYSICAL_ADDRESS NullAddress = {{0, 0}}, VgaAddress;
//     ULONG AddressSpace;
//     BOOLEAN Result;
//     ULONG_PTR Base;
//
//     /* Make sure that we have a bus translation function */
//     if (!HalFindBusAddressTranslation) return FALSE;
//
//     /* Loop trying to find possible VGA base addresses */
//     while (TRUE)
//     {
//         /* Get the VGA Register address */
//         AddressSpace = 1;
//         Result = HalFindBusAddressTranslation(NullAddress,
//                                               &AddressSpace,
//                                               &TranslatedAddress,
//                                               &Context,
//                                               TRUE);
//         if (!Result) return FALSE;
//
//         /* See if this is I/O Space, which we need to map */
//         if (!AddressSpace)
//         {
//             /* Map it */
//             Base = (ULONG_PTR)MmMapIoSpace(TranslatedAddress, 0x400, MmNonCached);
//         }
//         else
//         {
//             /* The base is the translated address, no need to map I/O space */
//             Base = TranslatedAddress.LowPart;
//         }
//
//         /* Try to see if this is VGA */
//         VgaRegisterBase = Base;
//         if (VgaIsPresent())
//         {
//             /* Translate the VGA Memory Address */
//             VgaAddress.LowPart = MEM_VGA;
//             VgaAddress.HighPart = 0;
//             AddressSpace = 0;
//             Result = HalFindBusAddressTranslation(VgaAddress,
//                                                   &AddressSpace,
//                                                   &TranslatedAddress,
//                                                   &Context,
//                                                   FALSE);
//             if (Result) break;
//         }
//         else
//         {
//             /* It's not, so unmap the I/O space if we mapped it */
//             if (!AddressSpace) MmUnmapIoSpace((PVOID)VgaRegisterBase, 0x400);
//         }
//
//         /* Continue trying to see if there's any other address */
//     }
//
//     /* Success! See if this is I/O Space, which we need to map */
//     if (!AddressSpace)
//     {
//         /* Map it */
//         Base = (ULONG_PTR)MmMapIoSpace(TranslatedAddress,
//                                        MEM_VGA_SIZE,
//                                        MmNonCached);
//     }
//     else
//     {
//         /* The base is the translated address, no need to map I/O space */
//         Base = TranslatedAddress.LowPart;
//     }
//
//     /* Set the VGA Memory Base */
//     VgaBase = Base;
//
//     /* Check whether we have to set the video mode */
//     if (SetMode)
//     {
//         /* Clear the current position */
//         VidpCurrentX = 0;
//         VidpCurrentY = 0;
//
//         /* Reset the display and initialize it */
//         if (HalResetDisplay())
//         {
//             /* The HAL handled the display, re-initialize only the AC registers */
//             VgaInterpretCmdStream(AT_Initialization);
//         }
//         else
//         {
//             /* The HAL didn't handle the display, fully re-initialize the VGA */
//             VgaInterpretCmdStream(VGA_640x480);
//         }
//     }
//
//     return TRUE;
// }
//
// VOID
// ResetDisplay(
//     _In_ BOOLEAN SetMode)
// {
//     /* Reset the video mode with HAL if requested */
//     if (SetMode && !HalResetDisplay())
//     {
//         /* The HAL didn't handle the display, fully re-initialize the VGA */
//         VgaInterpretCmdStream(VGA_640x480);
//     }
//
//     /* Always re-initialize the AC registers */
//     VgaInterpretCmdStream(AT_Initialization);
//
//     /* Re-initialize the palette and fill the screen black */
//     InitializePalette();
//     VidSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLACK);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
