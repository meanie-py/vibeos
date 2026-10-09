use crate::prelude::*;
use core::ffi::c_void;
pub unsafe extern "C" fn AtaDevHasCdbInterrupt(
    mut IdentifyPacketData: PIDENTIFY_PACKET_DATA,
) -> BOOLEAN {
    return ((*IdentifyPacketData).GeneralConfiguration.DrqDelay() == 1) as BOOLEAN;
}
pub unsafe extern "C" fn AtaDevCdbSizeInWords(IdentifyPacketData: *mut IDENTIFY_PACKET_DATA) -> u8 {
    return if (*IdentifyPacketData).GeneralConfiguration.PacketType() != 0 {
        8
    } else {
        6
    };
}

unsafe extern "C" fn AtaDevMaxLun(mut IdentifyPacketData: PIDENTIFY_PACKET_DATA) -> BOOLEAN {
    let mut LastLunIdentifier: USHORT = ((*IdentifyPacketData).ReservedWord126
        as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as USHORT;
    if LastLunIdentifier as ::core::ffi::c_int != 7 as ::core::ffi::c_int {
        return (LastLunIdentifier as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as BOOLEAN;
    }
    return 1 as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevIsDmaDirectionRequired(
    mut IdentifyPacketData: PIDENTIFY_PACKET_DATA,
) -> BOOLEAN {
    if (*IdentifyPacketData).DMADIR.DMADIRBitRequired() != 0 {
        return ((*IdentifyPacketData).MultiWordDMASupport() as ::core::ffi::c_int
            & 0x7 as ::core::ffi::c_int
            == 0
            && (*IdentifyPacketData).Capabilities.DmaSupported() == 0
            && (*IdentifyPacketData).Capabilities.InterleavedDmaSupported() == 0
            && (*IdentifyPacketData).UltraDMASupport() as ::core::ffi::c_int
                & 0x7F as ::core::ffi::c_int
                == 0) as ::core::ffi::c_int as BOOLEAN;
    }
    return FALSE as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevIsTape(mut IdentifyPacketData: PIDENTIFY_PACKET_DATA) -> BOOLEAN {
    return ((*IdentifyPacketData).GeneralConfiguration.CommandPacketType() as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int) as ::core::ffi::c_int as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevIsIdentifyDataValid(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    let mut i: ULONG = 0;
    let mut Crc: UCHAR = 0;

    if (*IdentifyData).Signature() != 0xA5 {
        return TRUE;
    }

    Crc = 0;
    i = 0;
    while (i as usize) < core::mem::size_of::<IDENTIFY_DEVICE_DATA>() {
        Crc = (Crc as u32 + *((IdentifyData as *mut u8).add(i as usize)) as u32) as UCHAR;
        i = i.wrapping_add(1);
    }

    return (Crc == 0) as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevHasLbaTranslation(IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    return (*IdentifyData).Capabilities.LbaSupported() as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevUserAddressableSectors28Bit(
    IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> ULONG {
    return (*IdentifyData).UserAddressableSectors;
}

pub unsafe extern "C" fn AtaDevUserAddressableSectors48Bit(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> ULONG64 {
    return ((*IdentifyData).Max48BitLBA[1 as usize] as ULONG64)
        << 32
        | (*IdentifyData).Max48BitLBA[0 as usize] as ULONG64;
}
pub unsafe extern "C" fn AtaDevHas48BitAddressFeature(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).CommandSetSupport.WordValid83() == 1 {
        return ((*IdentifyData).CommandSetSupport.BigLba()) as u8;
    }
    return FALSE;
}

pub unsafe extern "C" fn AtaDevIsCurrentGeometryValid(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    return (((*IdentifyData).TranslationFieldsValid() & 1) != 0
        && ((*IdentifyData).NumberOfCurrentCylinders != 0)
        && ((*IdentifyData).NumberOfCurrentCylinders <= 63)
        && ((*IdentifyData).NumberOfCurrentHeads != 0)
        && ((*IdentifyData).NumberOfCurrentHeads <= 16)
        && ((*IdentifyData).CurrentSectorsPerTrack != 0)) as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevDefaultChsTranslation(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
    mut Cylinders: PUSHORT,
    mut Heads: PUSHORT,
    mut SectorsPerTrack: PUSHORT,
) {
    *Cylinders = (*IdentifyData).NumCylinders as ::core::ffi::c_ushort;
    *Heads = (*IdentifyData).NumHeads as ::core::ffi::c_ushort;
    *SectorsPerTrack = (*IdentifyData).NumSectorsPerTrack as ::core::ffi::c_ushort;
}

pub unsafe extern "C" fn AtaDevCurrentChsTranslation(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
    mut Cylinders: PUSHORT,
    mut Heads: PUSHORT,
    mut SectorsPerTrack: PUSHORT,
) {
    *Cylinders = (*IdentifyData).NumberOfCurrentCylinders as ::core::ffi::c_ushort;
    *Heads = (*IdentifyData).NumberOfCurrentHeads as ::core::ffi::c_ushort;
    *SectorsPerTrack = (*IdentifyData).CurrentSectorsPerTrack as ::core::ffi::c_ushort;
}

unsafe extern "C" fn AtaDevCurrentSectorsPerDrq(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> UCHAR {
    let mut MultiSectorCurrent: UCHAR = 0;
    if (*IdentifyData).MultiSectorSettingValid() == 0 {
        return 0;
    }
    MultiSectorCurrent = (*IdentifyData).CurrentMultiSectorSetting;
    if MultiSectorCurrent as ::core::ffi::c_int > 0
        && MultiSectorCurrent as ::core::ffi::c_int
            & MultiSectorCurrent as ::core::ffi::c_int - 1
            == 0
    {
        return MultiSectorCurrent;
    }
    return 0;
}

pub unsafe extern "C" fn AtaDevMaximumSectorsPerDrq(mut IdentifyData: PIDENTIFY_DEVICE_DATA) -> UCHAR {
    let mut MultiSectorMax: UCHAR = 0;
    MultiSectorMax = (*IdentifyData).MaximumBlockTransfer;
    if MultiSectorMax as ::core::ffi::c_int > 0 as ::core::ffi::c_int
        && MultiSectorMax as ::core::ffi::c_int
            & MultiSectorMax as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        return MultiSectorMax;
    }
    return 0 as UCHAR;
}

pub unsafe extern "C" fn AtaDevBytesPerLogicalSector(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> ULONG {
    let mut WordCount: ULONG = 0;
    if (*IdentifyData).PhysicalLogicalSectorSize.Reserved1() as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && (*IdentifyData)
            .PhysicalLogicalSectorSize
            .LogicalSectorLongerThan256Words() as ::core::ffi::c_int
            != 0
    {
        WordCount = (*IdentifyData).WordsPerLogicalSector[0 as usize] as ULONG;
        WordCount |= ((*IdentifyData).WordsPerLogicalSector[1 as usize]
            as ULONG)
            << 16 as ::core::ffi::c_int;
    } else {
        WordCount = 256 as ULONG;
    }
    return (WordCount as usize)
        .wrapping_mul(::core::mem::size_of::<USHORT>() as usize) as ULONG;
}
pub unsafe extern "C" fn AtaDevLogicalSectorsPerPhysicalSector(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
    Exponent: *mut u32,
) -> u32 {
    if (*IdentifyData).PhysicalLogicalSectorSize.Reserved1() == 1 &&
        (*IdentifyData).PhysicalLogicalSectorSize.MultipleLogicalSectorsPerPhysicalSector() != 0
    {
        *Exponent = ((*IdentifyData).PhysicalLogicalSectorSize.LogicalSectorsPerPhysicalSector()) as _;
        return 1 << *Exponent;
    }

    *Exponent = 0;
    return 1 << 0;
}

pub unsafe extern "C" fn AtaDevLogicalSectorAlignment(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> ULONG {
    if (*IdentifyData).BlockAlignment.Word209Supported() as ::core::ffi::c_int != 0
        && (*IdentifyData).BlockAlignment.Reserved0() as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        return (*IdentifyData)
            .BlockAlignment
            .AlignmentOfLogicalWithinPhysical() as ULONG;
    }
    return 0 as ULONG;
}
pub unsafe extern "C" fn AtaDevInPuisState(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    return ((*IdentifyData).SpecificConfiguration == 0x37C8
        || (*IdentifyData).SpecificConfiguration == 0x738C) as BOOLEAN;
}
pub unsafe extern "C" fn AtaDevIsIdentifyDataIncomplete(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    return (*IdentifyData).GeneralConfiguration.ResponseIncomplete() as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevIsRemovable(IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    return (*IdentifyData).GeneralConfiguration.RemovableMedia() as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevHasRemovableMediaFeature(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if AtaDevIsRemovable(IdentifyData) != 0 {
        if (*IdentifyData).CommandSetSupport.WordValid() as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            return (*IdentifyData).CommandSetSupport.RemovableMediaFeature() as BOOLEAN;
        }
    }
    return FALSE as BOOLEAN;
}
pub unsafe extern "C" fn AtaDevQueueDepth(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> u32 {
    if (*IdentifyData).SerialAtaCapabilities.NCQ() != 0 {
        return ((*IdentifyData).QueueDepth() as i32 + 1) as u32;
    }
    return 0;
}

pub unsafe extern "C" fn AtaDevHasNcqAutosense(mut IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    let mut Word76: USHORT = *(IdentifyData as PUSHORT).offset(76 as isize) as USHORT;
    if Word76 as i32 != 0x0000 as i32 && Word76 as i32 != 0xFFFF as i32 {
        return (*IdentifyData).SerialAtaFeaturesSupported.NCQAutosense() as BOOLEAN;
    }
    return FALSE as BOOLEAN;
}
unsafe extern "C" fn AtaDevHasSenseDataReporting(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).CommandSetActive.Words119_120Valid() != 0
        && (*IdentifyData).CommandSetSupportExt.WordValid() == 1
    {
        return ((*IdentifyData).CommandSetSupportExt.SenseDataReporting()) as u8;
    }

    return FALSE;
}
pub unsafe extern "C" fn AtaDevHasRemovableMediaStatusNotification(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> i32 {
    return ((*IdentifyData).MsnSupport() == 1) as i32;
}
pub unsafe extern "C" fn AtaDevHasIeee1667(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).TrustedComputing.FeatureSupported() != 0 {
        return (*IdentifyData).AdditionalSupported.IEEE1667() as BOOLEAN;
    }
    return FALSE as BOOLEAN;
}
pub unsafe extern "C" fn AtaDevHasWorldWideName(
    IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).CommandSetActive.Reserved4() == 1 {
        return ((*IdentifyData).CommandSetActive.WWN64Bit()) as u8;
    }
    return FALSE;
}

unsafe extern "C" fn AtaDevIsZonedDevice(IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    return ((*IdentifyData).AdditionalSupported.ZonedCapabilities() != 0) as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevZonedCapabilities(IdentifyData: PIDENTIFY_DEVICE_DATA) -> UCHAR {
    return ((*IdentifyData).AdditionalSupported.ZonedCapabilities()) as u8;
}
pub unsafe extern "C" fn AtaDevHasSecurityModeFeature(
    IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).CommandSetSupport.WordValid() == 1 {
        return ((*IdentifyData).CommandSetSupport.SecurityMode()) as u8;
    }
    return FALSE;
}

pub unsafe extern "C" fn AtaDevHasSmartFeature(IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    return (*IdentifyData).CommandSetSupport.SmartCommands() as BOOLEAN;
}
pub unsafe extern "C" fn AtaDevIsVolatileWriteCacheEnabled(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> i32 {
    if (*IdentifyData).CommandSetSupport.WordValid() == 1 {
        return ((*IdentifyData).CommandSetSupport.WriteCache() != 0
            && (*IdentifyData).CommandSetActive.WriteCache() != 0) as i32;
    }

    return (FALSE) as _;
}

pub unsafe extern "C" fn AtaDevIsReadLookAHeadEnabled(
    mut IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).CommandSetSupport.WordValid() as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        return ((*IdentifyData).CommandSetSupport.LookAhead() as ::core::ffi::c_int != 0
            && (*IdentifyData).CommandSetActive.LookAhead() as ::core::ffi::c_int != 0)
            as ::core::ffi::c_int as BOOLEAN;
    }
    return FALSE as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevNominalFormFactor(IdentifyData: PIDENTIFY_DEVICE_DATA) -> UCHAR {
    return ((*IdentifyData).NominalFormFactor()) as u8;
}

pub unsafe extern "C" fn AtaDevMediumRotationRate(IdentifyData: PIDENTIFY_DEVICE_DATA) -> USHORT {
    return (*IdentifyData).NominalMediaRotationRate;
}

pub unsafe extern "C" fn AtaDevIsRotatingDevice(IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    return ((*IdentifyData).NominalMediaRotationRate as ::core::ffi::c_int
        >= 0x0401 as ::core::ffi::c_int
        && (*IdentifyData).NominalMediaRotationRate as ::core::ffi::c_int
            <= 0xFFFE as ::core::ffi::c_int) as ::core::ffi::c_int as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevIsSsd(IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    return ((*IdentifyData).NominalMediaRotationRate == 1) as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevHasTrimFunction(IdentifyData: *mut IDENTIFY_DEVICE_DATA) -> i32 {
    return (*IdentifyData).DataSetManagementFeature.SupportsTrim() as i32;
}

pub unsafe extern "C" fn AtaDevHasRzatFunction(IdentifyData: PIDENTIFY_DEVICE_DATA) -> BOOLEAN {
    return (*IdentifyData).AdditionalSupported.ReadZeroAfterTrimSupported() as BOOLEAN;
}
pub unsafe extern "C" fn AtaDevHasDratFunction(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    return (*IdentifyData)
        .AdditionalSupported
        .DeterministicReadAfterTrimSupported() as BOOLEAN;
}

pub unsafe extern "C" fn AtaDevHasForceUnitAccessCommands(
    IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).CommandSetSupport.WordValid83() == 1 {
        return ((*IdentifyData).CommandSetSupport.WriteFua()) as u8;
    }

    return FALSE;
}
pub unsafe extern "C" fn AtaDevHasFlushCache(
    IdentifyData: *mut IDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).CommandSetSupport.WordValid() == 1 {
        return ((*IdentifyData).CommandSetSupport.FlushCache()) as u8;
    }
    return FALSE;
}

pub unsafe extern "C" fn AtaDevHasFlushCacheExt(
    IdentifyData: PIDENTIFY_DEVICE_DATA,
) -> BOOLEAN {
    if (*IdentifyData).CommandSetSupport.WordValid() == 1 {
        return ((*IdentifyData).CommandSetSupport.FlushCacheExt()) as u8;
    }
    return FALSE;
}
