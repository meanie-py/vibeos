//! id_init.cpp — chip-specific init: chipset detection + IO-resource mapping + LUN-ext
//! allocation for ATA/ATAPI IDE controllers with Busmaster DMA and Serial ATA support.
//! Translated per TRANSLATION_MANIFEST.md.
//!
//! Live compile path (matching config.h / id_init.cpp #defines):
//!   - _DEBUG          : NOT defined  (KdPrint*/KdPrint2/PrintNtConsole dropped)
//!   - __REACTOS__     : DEFINED      (the ReactOS-specific branches are taken)
//!   - USER_MODE       : NOT defined
//!
//! Foundation conventions: every function is `pub unsafe extern "system"` (NTAPI). The
//! GetPciConfigN / SetPciConfigN / ChangePciConfigN macros are expanded inline at each
//! call site to ScsiPortGetBusDataByOffset / ScsiPortSetBusDataByOffset, capturing the
//! caller's local HwDeviceExtension / SystemIoBusNumber / slotNumber (named exactly as in
//! the C source). PSEH is absent here. goto is restructured to loop/break.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]
#![allow(unused_variables, unused_mut, unused_assignments, unused_parens)]

use crate::types::*;
use crate::regidx::*;
use crate::*; // sibling uniata functions + ScsiPortGetBusDataByOffset + BMList + AtapiRegCheckDevValue + WinVer/MajorVersion
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{TRUE, FALSE, PCVOID};
use ntkrnl_sys::driver_ddk::DbgPrint; // ahci markers
use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};
use ntkrnl_sys::mm::NonPagedPool;
use crate::io_port::{
    AtapiReadPort1, AtapiReadPort2, AtapiReadPort4, AtapiReadPortEx1, AtapiReadPortEx4,
    AtapiWritePort1, AtapiWritePort2, AtapiWritePort4, AtapiWritePortEx1, AtapiWritePortEx4,
    AtapiReadBuffer2, AtapiReadBuffer4, AtapiWriteBuffer2, AtapiWriteBuffer4, UniataNanoSleep,
};

// config.h: ULONGIO_PTR == ULONG_PTR on the live (kernel-mode) compile path.
type ULONGIO_PTR = ULONG_PTR;

// ---------------------------------------------------------------------------
// wstr! — build a NUL-terminated UTF-16 (wide) string literal as PCWSTR.
// AtapiRegCheckDevValue takes PCWSTR Name. We materialize the L"..." literal as a
// 'static [WCHAR; N] and hand back a pointer. Registry value names are ASCII-only,
// so each byte maps to one UTF-16 code unit. (Defined early: macro_rules is
// textual-order scoped.)
// ---------------------------------------------------------------------------
macro_rules! wstr {
    ($s:literal) => {{
        const BYTES: &[u8] = $s.as_bytes();
        const N: usize = BYTES.len() + 1;
        static W: [u16; N] = {
            let mut w = [0u16; N];
            let mut i = 0;
            while i < BYTES.len() {
                w[i] = BYTES[i] as u16;
                i += 1;
            }
            w
        };
        W.as_ptr() as PCWSTR
    }};
}

// BUS_DATA_TYPE::PCIConfiguration. Typed as BUS_DATA_TYPE (== c_int) to match
// uniata's own ScsiPortGetBusDataByOffset + HalGetBusData. The kernel imports
// ScsiPortSetBusDataByOffset / ScsiPortGetBusData take a ULONG bus-data-type, so
// those call sites pass `PCIConfiguration as ULONG`.
const PCIConfiguration: BUS_DATA_TYPE = 4;

// ATA_MODE_NOT_SPEC = ((ULONG)(-1))  (atapi.h:341). Not in the bindgen const set.
const ATA_MODE_NOT_SPEC: ULONG = 0xffffffff;

// ---------------------------------------------------------------------------
// PCI_COMMON_CONFIG: reuse the overlay that the id_probe module defines (the ddk
// bindgen type is opaque [UCHAR;192]; id_probe re-spells the full
// PCI_COMMON_HEADER_LAYOUT). Explicit import overrides the ambiguous glob from
// ntkrnl_sys::ddk. Field access mirrors id_probe: `.u.type0.as_ref().BaseAddresses[n]`.
// ---------------------------------------------------------------------------
use crate::id_probe::{PCI_COMMON_CONFIG, PPCI_COMMON_CONFIG};

// PCI class/subclass/progif constants (bsmaster.h).
const PCI_DEV_CLASS_STORAGE: UCHAR = 0x01;
const PCI_DEV_SUBCLASS_IDE: UCHAR = 0x01;
const PCI_DEV_SUBCLASS_SATA: UCHAR = 0x06;
const PCI_DEV_PROGIF_AHCI_1_0: UCHAR = 0x01;

// offsetof(PCI_COMMON_CONFIG, Command) == 4 ; CacheLineSize == 12.
const PCI_CFG_Command: ULONG = 4;
const PCI_CFG_CacheLineSize: ULONG = 12;

// PCI_COMMON_HDR_LENGTH = FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific) == 64.
const PCI_COMMON_HDR_LENGTH: ULONG = 64;

// Ata_is_ahci_dev(pciData) macro (bm_devs_decl.h) inlined.
#[inline]
unsafe fn Ata_is_ahci_dev(pciData: PPCI_COMMON_CONFIG) -> bool {
    if pciData.is_null() {
        return false;
    }
    let type0 = (*pciData).u.type0.as_ref();
    (*pciData).BaseClass == PCI_DEV_CLASS_STORAGE
        && (*pciData).SubClass == PCI_DEV_SUBCLASS_SATA
        && (*pciData).ProgIf == PCI_DEV_PROGIF_AHCI_1_0
        && (type0.BaseAddresses[5] & !0x7u32) != 0
}

// Local copy of the header-inline Ata_is_dev_listed (atapi.h is __inline; each TU
// gets its own — id_probe's copy is module-private, so define ours here).
#[inline]
unsafe fn Ata_is_dev_listed(
    bm_adapters: PBUSMASTER_CONTROLLER_INFORMATION_BASE,
    VendorId: ULONG,
    DeviceId: ULONG,
    RevId: ULONG,
    lim: ULONG,
) -> ULONG {
    let mut k: ULONG = 0;
    while k < lim {
        let e = &*bm_adapters.add(k as usize);
        if e.nVendorId == 0xffff && e.nDeviceId == 0xffff && e.nRevId == 0xff {
            if lim != BMLIST_TERMINATOR {
                k += 1;
                continue;
            }
            return BMLIST_TERMINATOR;
        }
        if e.nVendorId == VendorId
            && (e.nDeviceId == DeviceId || DeviceId == 0xffff)
            && (RevId == 0 || e.nRevId <= RevId)
        {
            return k;
        }
        k += 1;
    }
    BMLIST_TERMINATOR
}

// WinVer_Id() == ((MajorVersion << 8) | MinorVersion) ; WinVer_NT == 0x0400.
// MajorVersion / MinorVersion are ReactOS kernel-exported globals (extern ULONG).
const WinVer_NT: ULONG = 0x0400;
unsafe extern "C" {
    static MajorVersion: ULONG;
    static MinorVersion: ULONG;
}
#[inline]
unsafe fn WinVer_Id() -> ULONG {
    (MajorVersion << 8) | MinorVersion
}

// ---------------------------------------------------------------------------
// id_init.cpp file-scope const table.
//   PCI_DEV_HW_SPEC_BM(idhi, idlo, rev, mode, name, flags)
//     == { nVendorId:0x##idlo, nDeviceId:0x##idhi, nRevId:rev, MaxTransferMode:mode,
//          RaidFlags:flags, FullDevName:name }
// ---------------------------------------------------------------------------
static ATI_SOUTH_NAME: &[u8] = b"ATI South\0";

// BUSMASTER_CONTROLLER_INFORMATION_BASE holds a raw PCSTR, so it is not Sync. The C
// table is `static const` (read-only) in a no-threading miniport context; wrap it in a
// Sync newtype to keep it a `pub static`. Index via `AtiSouthAdapters.0[..]`.
#[repr(transparent)]
pub struct BmTable<const N: usize>(pub [BUSMASTER_CONTROLLER_INFORMATION_BASE; N]);
unsafe impl<const N: usize> Sync for BmTable<N> {}

pub static AtiSouthAdapters: BmTable<2> = BmTable([
    // PCI_DEV_HW_SPEC_BM( 4385, 1002, 0x00, ATA_MODE_NOT_SPEC, "ATI South", 0 )
    BUSMASTER_CONTROLLER_INFORMATION_BASE {
        nVendorId: 0x1002,
        nDeviceId: 0x4385,
        nRevId: 0x00,
        MaxTransferMode: ATA_MODE_NOT_SPEC,
        RaidFlags: 0,
        FullDevName: ATI_SOUTH_NAME.as_ptr() as PCSTR,
    },
    // PCI_DEV_HW_SPEC_BM( ffff, ffff, 0xff, BMLIST_TERMINATOR, NULL, BMLIST_TERMINATOR )
    BUSMASTER_CONTROLLER_INFORMATION_BASE {
        nVendorId: 0xffff,
        nDeviceId: 0xffff,
        nRevId: 0xff,
        MaxTransferMode: BMLIST_TERMINATOR,
        RaidFlags: BMLIST_TERMINATOR,
        FullDevName: core::ptr::null(),
    },
]);

// Helper: build one BUSMASTER_CONTROLLER_INFORMATION_BASE entry from the macro args.
// Used to construct the function-local static tables (SiSAdapters/ViaAdapters/etc.).
const fn bm_entry(
    idhi: ULONG,
    idlo: ULONG,
    rev: ULONG,
    mode: ULONG,
    flags: ULONG,
    name: *const u8,
) -> BUSMASTER_CONTROLLER_INFORMATION_BASE {
    BUSMASTER_CONTROLLER_INFORMATION_BASE {
        nVendorId: idlo,
        nDeviceId: idhi,
        nRevId: rev,
        MaxTransferMode: mode,
        RaidFlags: flags,
        FullDevName: name as PCSTR,
    }
}

// ===========================================================================
// UniataChipDetectChannels
// ===========================================================================
pub unsafe extern "system" fn UniataChipDetectChannels(
    HwDeviceExtension: PVOID,
    pciData: PPCI_COMMON_CONFIG, // optional
    DeviceNumber: ULONG,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
    let ChipType: ULONG = (*deviceExtension).HwFlags & CHIPTYPE_MASK;
    let ChipFlags: ULONG = (*deviceExtension).HwFlags & CHIPFLAG_MASK;
    let mut i: ULONG;
    let mut n: ULONG;

    (*deviceExtension).AHCI_PI_mask = 0;

    if (ChipFlags & (UNIATA_SATA | UNIATA_AHCI)) != 0 {
        if (*deviceExtension).NumberChannels == 0 {
            (*deviceExtension).NumberChannels = 1;
        }
        if AtapiRegCheckDevValue(
            deviceExtension as PVOID,
            CHAN_NOT_SPECIFIED,
            DEVNUM_NOT_SPECIFIED,
            wstr!("IgnoreAhciPM"),
            1,
        ) != 0
        {
            (*deviceExtension).NumberLuns = 2; // we may be in Legacy mode
        } else {
            (*deviceExtension).NumberLuns = SATA_MAX_PM_UNITS;
        }
    }
    if (*deviceExtension).MasterDev != 0 {
        (*deviceExtension).NumberChannels = 1;
    }
    n = 0;
    while n < (*deviceExtension).NumberChannels {
        if AtapiRegCheckDevValue(
            deviceExtension as PVOID,
            n,
            DEVNUM_NOT_SPECIFIED,
            wstr!("Exclude"),
            0,
        ) != 0
        {
            (*deviceExtension).AHCI_PI_mask &= !(1u32 << n);
        } else {
            (*deviceExtension).AHCI_PI_mask |= 1u32 << n;
        }
        n += 1;
    }
    (*deviceExtension).AHCI_PI_mask = AtapiRegCheckDevValue(
        deviceExtension as PVOID,
        CHAN_NOT_SPECIFIED,
        DEVNUM_NOT_SPECIFIED,
        wstr!("PortMask"),
        0xffffffffu32 >> (32 - (*deviceExtension).NumberChannels),
    );

    // for(i=AHCI_PI_mask, n=0; i; n++, i=i>>1);
    i = (*deviceExtension).AHCI_PI_mask;
    n = 0;
    while i != 0 {
        n += 1;
        i >>= 1;
    }
    let channels_before = (*deviceExtension).NumberChannels;
    let default_mask = 0xffffffffu32 >> (32 - channels_before);

    match VendorID {
        ATA_ACER_LABS_ID => {
            match (*deviceExtension).DevID {
                0x528710b9 | 0x528810b9 => {
                    (*deviceExtension).NumberChannels = 4;
                }
                _ => {}
            }
        }
        ATA_SERVERWORKS_ID => {
            // The SATA parts have 4 ports, and UniATA left them at the 2-channel
            // default, hiding ports 2 and 3. 0x0241 (Frodo8) has 8: that is what
            // all three drivers that program the chip do (upstream svw_sata.c,
            // Linux sata_svw chip_svw8, FreeBSD ATA_FRODO8 = 0x02411166 with 8
            // channels). Only NAMES disagree (Linux's comment and NetBSD pcidevs
            // call 0x0241 "Frodo4", and UniATA copied that into its table).
            if ChipType == SWKSMIO {
                (*deviceExtension).NumberChannels =
                    if ((*deviceExtension).DevID >> 16) == 0x0241 { 8 } else { 4 };
            }
        }
        ATA_PROMISE_ID => 'promise: {
            if ChipType != PRMIO {
                break 'promise;
            }
            if (ChipFlags & UNIATA_SATA) == 0 {
                (*deviceExtension).NumberChannels = 4;
            } else if (ChipFlags & PRCMBO) != 0 {
                (*deviceExtension).NumberChannels = 3;
            } else {
                (*deviceExtension).NumberChannels = 4;
            }
        }
        ATA_MARVELL_ID => {
            // AHCI part has own DevID-based workaround
            match (*deviceExtension).DevID {
                0x610111ab => {
                    // 88SX6101 only have 1 PATA channel
                    if (*BMList.add((*deviceExtension).DevIndex as usize)).channel != 0 {
                        return FALSE;
                    }
                    (*deviceExtension).NumberChannels = 1;
                }
                _ => {}
            }
        }
        ATA_ATI_ID => {
            // ATI: fall through to ATA_SILICON_IMAGE_ID handling after the inner switch.
            let mut fallthrough_to_sii = true;
            match (*deviceExtension).DevID {
                ATA_ATI_IXP600 => {
                    // IXP600 only have 1 PATA channel
                    if (*BMList.add((*deviceExtension).DevIndex as usize)).channel != 0 {
                        return FALSE;
                    }
                    (*deviceExtension).NumberChannels = 1;
                    fallthrough_to_sii = false; // C: break (out of inner switch only)
                }
                ATA_ATI_IXP700 => {
                    let mut satacfg: UCHAR = 0;
                    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();
                    /*
                     * "combined mode" emulates an additional PATA channel with two SATA
                     * ports; detectable only via SMB controller.
                     */
                    let j = AtapiFindListedDev(
                        &AtiSouthAdapters.0[0] as *const _ as *mut BUSMASTER_CONTROLLER_INFORMATION_BASE,
                        0xffffffff, // -1
                        HwDeviceExtension,
                        SystemIoBusNumber,
                        PCISLOTNUM_NOT_SPECIFIED,
                        &mut slotData,
                    );
                    if j != BMLIST_TERMINATOR {
                        let slotNumber: ULONG = slotData.u.AsULONG;
                        // GetPciConfig1(0xad, satacfg)
                        ScsiPortGetBusDataByOffset(
                            HwDeviceExtension,
                            PCIConfiguration,
                            SystemIoBusNumber,
                            slotNumber,
                            &mut satacfg as *mut _ as PVOID,
                            0xad,
                            1,
                        );
                        // If SATA enabled but combined mode disabled -> single PATA channel.
                        if (satacfg & 0x09) == 0x01 {
                            if (*BMList.add((*deviceExtension).DevIndex as usize)).channel != 0 {
                                return FALSE;
                            }
                            (*deviceExtension).NumberChannels = 1;
                            fallthrough_to_sii = false; // C: break inner switch
                        } else {
                            (*deviceExtension).NumberChannels = 2;
                        }
                    }
                }
                _ => {}
            }
            if fallthrough_to_sii {
                // FALLTHROUGH to ATA_SILICON_IMAGE_ID
                sii_detect_channels(deviceExtension, pciData, VendorID, ChipType, ChipFlags);
            }
        }
        ATA_SILICON_IMAGE_ID => {
            sii_detect_channels(deviceExtension, pciData, VendorID, ChipType, ChipFlags);
        }
        ATA_VIA_ID => {
            if ((*deviceExtension).HwFlags & VIABAR) != 0 {
                (*deviceExtension).NumberChannels = 3;
            }
            if (ChipFlags & VIASATA) != 0 {
                // 2 SATA without SATA registers on first channel + 1 PATA on second
                (*deviceExtension).NumberLuns = 1;
            }
        }
        ATA_ITE_ID => {
            // ITE ATA133 controller
            if (*deviceExtension).DevID == 0x82131283 {
                if (*BMList.add((*deviceExtension).DevIndex as usize)).channel != 0 {
                    return FALSE;
                }
                (*deviceExtension).NumberChannels = 1;
            }
        }
        ATA_CYRIX_ID => {
            if ChipType == CYRIX_OLD {
                let mut tmp8: UCHAR = 0;
                let slotNumber: ULONG = (*deviceExtension).slotNumber;
                // GetPciConfig1(0x60, tmp8)
                ScsiPortGetBusDataByOffset(
                    HwDeviceExtension,
                    PCIConfiguration,
                    SystemIoBusNumber,
                    slotNumber,
                    &mut tmp8 as *mut _ as PVOID,
                    0x60,
                    1,
                );
                let ch = (*BMList.add((*deviceExtension).DevIndex as usize)).channel;
                if (tmp8 & (1u8 << ch)) != 0 {
                    // ok
                } else {
                    return FALSE;
                }
            }
        }
        _ => {}
    } // end match(VendorID)

    // The C sized `n` (and the default PortMask) from the channel count BEFORE
    // the match above, and clamps to it below, so every arm that RAISES the
    // count (ALi 5287/5288 and Promise MIO to 4, ServerWorks SATA to 4/8) was
    // cut back to the 2-channel default: dead code since UniATA was written.
    // When an arm raised it and the mask is still the computed default (no
    // user PortMask), size both from the arm's count. An explicit PortMask or
    // NumberChannels registry value still limits it, which is what the clamp
    // is for.
    if (*deviceExtension).NumberChannels > channels_before
        && (*deviceExtension).AHCI_PI_mask == default_mask
    {
        n = (*deviceExtension).NumberChannels;
        (*deviceExtension).AHCI_PI_mask = 0xffffffffu32 >> (32 - n);
    }

    i = AtapiRegCheckDevValue(
        deviceExtension as PVOID,
        CHAN_NOT_SPECIFIED,
        DEVNUM_NOT_SPECIFIED,
        wstr!("NumberChannels"),
        n,
    );
    if i == 0 {
        i = n;
    }

    (*deviceExtension).NumberChannels = core::cmp::min(i, (*deviceExtension).NumberChannels);
    if (*deviceExtension).NumberChannels == 0 {
        return FALSE;
    }
    (*deviceExtension).AHCI_PI_mask &= 0xffffffffu32 >> (32 - (*deviceExtension).NumberChannels);

    TRUE
}

// SiI / ATI-SATA channel-count tail shared by the ATA_ATI_ID FALLTHROUGH and
// ATA_SILICON_IMAGE_ID arm of UniataChipDetectChannels.
#[inline]
unsafe fn sii_detect_channels(
    deviceExtension: PHW_DEVICE_EXTENSION,
    pciData: PPCI_COMMON_CONFIG,
    VendorID: ULONG,
    ChipType: ULONG,
    ChipFlags: ULONG,
) {
    if (ChipFlags & SIIBUG) != 0 {
        // work around errata in early chips
        (*deviceExtension).DmaSegmentLength = 15 * DEV_BSIZE;
        (*deviceExtension).DmaSegmentAlignmentMask = 8192 - 1;
    }
    if ChipType != SIIMIO {
        return;
    }
    if pciData.is_null() {
        return;
    }
    if ((*deviceExtension).HwFlags & SII4CH) != 0 {
        (*deviceExtension).NumberChannels = 4;
    }
}

// ===========================================================================
// UniataChipDetect
// ===========================================================================
pub unsafe extern "system" fn UniataChipDetect(
    HwDeviceExtension: PVOID,
    pciData: PPCI_COMMON_CONFIG, // optional
    DeviceNumber: ULONG,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    simplexOnly: *mut BOOLEAN,
) -> NTSTATUS {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let slotNumber: ULONG = (*deviceExtension).slotNumber;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
    let DeviceID: ULONG = ((*deviceExtension).DevID >> 16) & 0xffff;
    let RevID: ULONG = (*deviceExtension).RevID;
    let mut i: ULONG;
    let mut c: ULONG;
    let mut DevTypeInfo: *mut BUSMASTER_CONTROLLER_INFORMATION_BASE;
    let mut chan: PHW_CHANNEL;
    let mut ChipType: ULONG;
    let mut ChipFlags: ULONG;
    let mut tmp32: ULONG;
    let mut tmp8: UCHAR = 0;
    let mut BaseMemAddress: ULONG_PTR;
    let mut BaseIoAddress1: ULONG_PTR;
    let mut BaseIoAddress2: ULONG_PTR;
    let mut BaseIoAddressBM: ULONG_PTR;
    let mut MemIo: BOOLEAN = FALSE;
    let mut IsPata: BOOLEAN = FALSE;

    i = Ata_is_dev_listed(
        &BusMasterAdapters.0[0] as *const _ as PBUSMASTER_CONTROLLER_INFORMATION_BASE,
        VendorID,
        0xffff,
        0,
        _NUM_BUSMASTER_ADAPTERS,
    );
    DbgPrint(b"[UNIATA] ChipDetect: ven=%04x dev=%04x rev=%u vendor-listed i=%u\n\0"
        .as_ptr().cast(), VendorID, DeviceID, RevID, i); // ahci marker

    c = AtapiRegCheckDevValue(
        deviceExtension as PVOID,
        CHAN_NOT_SPECIFIED,
        DEVNUM_NOT_SPECIFIED,
        wstr!("ForceSimplex"),
        0,
    );
    if c != 0 {
        *simplexOnly = TRUE;
    }

    // defaults
    BaseIoAddressBM =
        ((*pciData).u.type0.as_ref().BaseAddresses[4] as ULONG_PTR) & !0x07;
    (*deviceExtension).MaxTransferMode = if BaseIoAddressBM != 0 { ATA_DMA } else { ATA_PIO4 };
    (*ConfigInfo).MaximumTransferLength = DEV_BSIZE * 256;
    (*deviceExtension).MaximumDmaTransferLength = (*ConfigInfo).MaximumTransferLength;
    (*deviceExtension).DmaSegmentLength = 0x10000;
    (*deviceExtension).DmaSegmentAlignmentMask = 0xffff;

    // The C source uses `goto unknown_dev;` from several points and `goto for_ugly_chips;`.
    // We restructure: a closure-free state machine using labeled blocks.
    //   - `DevTypeInfo` defaulted; `i` carries the listed index.
    //   - 'unknown handles the unknown_dev: label.
    //   - 'ugly handles the for_ugly_chips: label.
    //
    // First, resolve DevTypeInfo from the top BusMasterAdapters lookup.
    let mut go_unknown = false;
    if i != BMLIST_TERMINATOR {
        DevTypeInfo =
            &BusMasterAdapters.0[0] as *const _ as PBUSMASTER_CONTROLLER_INFORMATION_BASE;
        DevTypeInfo = DevTypeInfo.add(i as usize);
        // reset to base; we index via DevTypeInfo[i] below — keep base pointer instead.
        DevTypeInfo =
            &BusMasterAdapters.0[0] as *const _ as PBUSMASTER_CONTROLLER_INFORMATION_BASE;
    } else {
        DevTypeInfo = core::ptr::null_mut();
        go_unknown = true;
    }

    // ---- function-local device tables (mirrors id_init.cpp's local statics) ----
    let SiSAdapters = sis_adapters();
    let ViaAdapters = via_adapters();
    let ViaSouthAdapters = via_south_adapters();

    // We use a small state enum to emulate the C goto control flow.
    enum Flow {
        Switch,
        Ugly,
        Unknown,
    }
    let mut flow = if go_unknown { Flow::Unknown } else { Flow::Switch };

    'control: loop {
        match flow {
            Flow::Switch => {
                match VendorID {
                    ATA_SIS_ID => {
                        // Scan bus for SIS bridge.
                        DevTypeInfo = SiSAdapters.as_ptr() as *mut _;
                        i = AtapiFindListedDev(
                            DevTypeInfo,
                            0xffffffff,
                            HwDeviceExtension,
                            SystemIoBusNumber,
                            PCISLOTNUM_NOT_SPECIFIED,
                            core::ptr::null_mut(),
                        );
                        if i != BMLIST_TERMINATOR {
                            (*deviceExtension).FullDevName =
                                SiSAdapters[i as usize].FullDevName as PCCH;
                        }
                        flow = Flow::Ugly;
                        continue 'control;
                    }
                    ATA_VIA_ID => {
                        // New chips have own DeviceId
                        if (*deviceExtension).DevID != ATA_VIA82C571
                            && (*deviceExtension).DevID != ATA_VIACX700IDE
                            && (*deviceExtension).DevID != ATA_VIASATAIDE
                            && (*deviceExtension).DevID != ATA_VIASATAIDE2
                            && (*deviceExtension).DevID != ATA_VIASATAIDE3
                        {
                            // Via new -> break out of switch (default flow).
                        } else {
                            // Distinguish traditional chips by ISA Bridge DeviceId.
                            DevTypeInfo = ViaSouthAdapters.as_ptr() as *mut _;
                            i = AtapiFindListedDev(
                                DevTypeInfo,
                                0xffffffff,
                                HwDeviceExtension,
                                SystemIoBusNumber,
                                PCISLOTNUM_NOT_SPECIFIED,
                                core::ptr::null_mut(),
                            );
                            if i != BMLIST_TERMINATOR {
                                (*deviceExtension).HwFlags |= VIASOUTH;
                            }
                            DevTypeInfo = ViaAdapters.as_ptr() as *mut _;
                            i = AtapiFindListedDev(
                                DevTypeInfo,
                                0xffffffff,
                                HwDeviceExtension,
                                SystemIoBusNumber,
                                PCISLOTNUM_NOT_SPECIFIED,
                                core::ptr::null_mut(),
                            );
                            if i != BMLIST_TERMINATOR {
                                (*deviceExtension).FullDevName =
                                    ViaAdapters[i as usize].FullDevName as PCCH;
                            }
                            flow = Flow::Ugly;
                            continue 'control;
                        }
                    }
                    _ => {
                        // do nothing
                    }
                }

                // i = Ata_is_dev_listed(DevTypeInfo, VendorID, DeviceID, RevID, -1);
                i = Ata_is_dev_listed(DevTypeInfo, VendorID, DeviceID, RevID, 0xffffffff);
                DbgPrint(b"[UNIATA] ChipDetect: dev-listed i=%u (DevTypeInfo=%p)\n\0"
                    .as_ptr().cast(), i, DevTypeInfo as PVOID); // ahci marker
                flow = Flow::Ugly;
                continue 'control;
            }

            Flow::Ugly => {
                // for_ugly_chips:
                if i == BMLIST_TERMINATOR {
                    DbgPrint(b"[UNIATA] ChipDetect: -> unknown_dev path\n\0"
                        .as_ptr().cast()); // ahci marker
                    flow = Flow::Unknown;
                    continue 'control;
                }
                (*deviceExtension).MaxTransferMode =
                    (*DevTypeInfo.add(i as usize)).MaxTransferMode;
                (*deviceExtension).HwFlags |= (*DevTypeInfo.add(i as usize)).RaidFlags;

                tmp32 = AtapiRegCheckDevValue(
                    deviceExtension as PVOID,
                    CHAN_NOT_SPECIFIED,
                    DEVNUM_NOT_SPECIFIED,
                    wstr!("HwFlagsOverride"),
                    (*deviceExtension).HwFlags,
                );
                (*deviceExtension).HwFlags = tmp32;

                tmp32 = AtapiRegCheckDevValue(
                    deviceExtension as PVOID,
                    CHAN_NOT_SPECIFIED,
                    DEVNUM_NOT_SPECIFIED,
                    wstr!("HwFlagsAdd"),
                    0,
                );
                (*deviceExtension).HwFlags |= tmp32;

                if ((*deviceExtension).HwFlags & UNIATA_SIMPLEX_ONLY) != 0 {
                    *simplexOnly = TRUE;
                }

                tmp32 = AtapiRegCheckDevValue(
                    deviceExtension as PVOID,
                    CHAN_NOT_SPECIFIED,
                    DEVNUM_NOT_SPECIFIED,
                    wstr!("MaxTransferMode"),
                    (*deviceExtension).MaxTransferMode,
                );
                if tmp32 != 0xffffffff {
                    (*deviceExtension).MaxTransferMode = tmp32;
                }

                if (*deviceExtension).MaxTransferMode >= ATA_SA150 {
                    (*deviceExtension).HwFlags |= UNIATA_SATA;
                }

                ChipType = (*deviceExtension).HwFlags & CHIPTYPE_MASK;
                ChipFlags = (*deviceExtension).HwFlags & CHIPFLAG_MASK;

                // for even more ugly AHCI-capable chips
                if (ChipFlags & UNIATA_AHCI) != 0 {
                    match VendorID {
                        ATA_NVIDIA_ID | ATA_ATI_ID => {
                            if (*pciData).SubClass == PCI_DEV_SUBCLASS_IDE {
                                ChipFlags &= !UNIATA_AHCI;
                                (*deviceExtension).HwFlags &= !UNIATA_AHCI;
                            }
                        }
                        _ => {
                            // __REACTOS__: RangeStart assumed 0; updated by UniataAhciDetect().
                        }
                    }
                }

                if (ChipFlags & UNIATA_AHCI) != 0 {
                    (*deviceExtension).NumberChannels = 0;
                    if UniataAhciDetect(HwDeviceExtension, pciData as *mut _, ConfigInfo) == 0 {
                        return STATUS_UNSUCCESSFUL;
                    }
                } else if UniataChipDetectChannels(
                    HwDeviceExtension,
                    pciData,
                    DeviceNumber,
                    ConfigInfo,
                ) == 0
                {
                    return STATUS_UNSUCCESSFUL;
                }
                // UniataAhciDetect() sets proper number of channels
                if UniataAllocateLunExt(deviceExtension, UNIATA_ALLOCATE_NEW_LUNS) == 0 {
                    return STATUS_UNSUCCESSFUL;
                }

                // === per-vendor IO-resource mapping ===
                match VendorID {
                    ATA_ACER_LABS_ID => {
                        if (ChipFlags & UNIATA_SATA) != 0 {
                            (*deviceExtension).AltRegMap = TRUE;
                            BaseIoAddress1 = AtapiGetIoRange(
                                HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                0, 0, 0x10,
                            );
                            BaseIoAddress2 = AtapiGetIoRange(
                                HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                1, 0, 0x10,
                            );
                            BaseIoAddressBM = AtapiGetIoRange(
                                HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                4, 0,
                                (*deviceExtension).NumberChannels
                                    * core::mem::size_of::<IDE_BUSMASTER_REGISTERS>() as ULONG,
                            );
                            c = 0;
                            while c < (*deviceExtension).NumberChannels {
                                let unit10 = c & 2;
                                chan = (*deviceExtension).chan.add(c as usize);
                                i = 0;
                                while i <= IDX_IO1_SZ as ULONG {
                                    UniataInitIoRes(
                                        chan,
                                        IDX_IO1 as ULONG + i,
                                        BaseIoAddress1 + i as ULONG_PTR
                                            + if unit10 != 0 { 8 } else { 0 },
                                        FALSE, FALSE,
                                    );
                                    i += 1;
                                }
                                UniataInitIoRes(
                                    chan,
                                    IDX_IO2_AltStatus as ULONG,
                                    BaseIoAddress2 + 2 + if unit10 != 0 { 4 } else { 0 },
                                    FALSE, FALSE,
                                );
                                UniataInitSyncBaseIO(chan);
                                i = 0;
                                while i <= IDX_BM_IO_SZ as ULONG {
                                    UniataInitIoRes(
                                        chan,
                                        IDX_BM_IO as ULONG + i,
                                        BaseIoAddressBM + i as ULONG_PTR
                                            + (c as ULONG_PTR
                                                * core::mem::size_of::<IDE_BUSMASTER_REGISTERS>()
                                                    as ULONG_PTR),
                                        FALSE, FALSE,
                                    );
                                    i += 1;
                                }
                                (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                                c += 1;
                            }
                        }
                    }
                    ATA_NVIDIA_ID => {
                        if (ChipFlags & UNIATA_SATA) != 0 {
                            BaseMemAddress = AtapiGetIoRange(
                                HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                5, 0,
                                (if (ChipFlags & NV4OFF) != 0 { 0x400 } else { 0 }) + 0x40 * 2,
                            );
                            if BaseMemAddress == 0 {
                                return STATUS_UNSUCCESSFUL;
                            }
                            if access_range_in_memory(ConfigInfo, 5) {
                                MemIo = TRUE;
                            }
                            UniataInitIoResEx(
                                &mut (*deviceExtension).BaseIoAddressSATA_0,
                                BaseMemAddress, MemIo, FALSE,
                            );
                            c = 0;
                            while c < (*deviceExtension).NumberChannels {
                                chan = (*deviceExtension).chan.add(c as usize);
                                UniataInitIoRes(chan, IDX_SATA_SStatus as ULONG,
                                    BaseMemAddress + ((c as ULONG_PTR) << 6), MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SError as ULONG,
                                    BaseMemAddress + 4 + ((c as ULONG_PTR) << 6), MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SControl as ULONG,
                                    BaseMemAddress + 8 + ((c as ULONG_PTR) << 6), MemIo, FALSE);
                                (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                                c += 1;
                            }
                        }
                    }
                    ATA_PROMISE_ID => 'prom: {
                        if ChipType != PRMIO {
                            break 'prom;
                        }
                        if pciData.is_null() {
                            break 'prom;
                        }
                        (*deviceExtension).AltRegMap = TRUE;

                        // BAR4 -> res1
                        BaseMemAddress = AtapiGetIoRange(
                            HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                            4, 0, 0x4000,
                        );
                        if BaseMemAddress == 0 {
                            return STATUS_UNSUCCESSFUL;
                        }
                        if access_range_in_memory(ConfigInfo, 4) {
                            MemIo = TRUE;
                        }
                        UniataInitIoResEx(
                            &mut (*deviceExtension).BaseIoAddressSATA_0,
                            BaseMemAddress, MemIo, FALSE,
                        );

                        // BAR3 -> res2
                        BaseMemAddress = AtapiGetIoRange(
                            HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                            3, 0, 0xd0000,
                        );
                        if BaseMemAddress == 0 {
                            return STATUS_UNSUCCESSFUL;
                        }
                        if access_range_in_memory(ConfigInfo, 3) {
                            MemIo = TRUE;
                        }
                        UniataInitIoResEx(
                            &mut (*deviceExtension).BaseIoAddressBM_0,
                            BaseMemAddress, MemIo, FALSE,
                        );

                        if (ChipFlags & UNIATA_SATA) == 0 {
                            let reg48: UCHAR = AtapiReadPortEx1(
                                core::ptr::null_mut(),
                                &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _
                                    as ULONGIO_PTR,
                                0x48,
                            );
                            (*deviceExtension).NumberChannels =
                                (if (reg48 & 0x01) != 0 { 1 } else { 0 })
                                    + (if (reg48 & 0x02) != 0 { 1 } else { 0 })
                                    + 2;
                        }

                        c = 0;
                        while c < (*deviceExtension).NumberChannels {
                            // res2-based
                            chan = (*deviceExtension).chan.add(c as usize);
                            let offs8: ULONG_PTR = (c as ULONG_PTR) << 8;
                            let offs7: ULONG_PTR = (c as ULONG_PTR) << 7;

                            i = 0;
                            while i <= IDX_IO1_SZ as ULONG {
                                UniataInitIoRes(chan, IDX_IO1 as ULONG + i,
                                    BaseMemAddress + 0x200 + ((i as ULONG_PTR) << 2) + offs8,
                                    MemIo, FALSE);
                                i += 1;
                            }
                            UniataInitIoRes(chan, IDX_IO2_AltStatus as ULONG,
                                BaseMemAddress + 0x238 + offs7, MemIo, FALSE);
                            UniataInitSyncBaseIO(chan);
                            UniataInitIoRes(chan, IDX_BM_Command as ULONG,
                                BaseMemAddress + 0x260 + offs7, MemIo, FALSE);
                            UniataInitIoRes(chan, IDX_BM_PRD_Table as ULONG,
                                BaseMemAddress + 0x244 + offs7, MemIo, FALSE);
                            UniataInitIoRes(chan, IDX_BM_DeviceSpecific0 as ULONG,
                                BaseMemAddress + ((c as ULONG_PTR) << 2), MemIo, FALSE);

                            if (ChipFlags & PRSATA) != 0
                                || ((ChipFlags & PRCMBO) != 0 && c < 2)
                            {
                                UniataInitIoRes(chan, IDX_SATA_SStatus as ULONG,
                                    BaseMemAddress + 0x400 + offs7, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SError as ULONG,
                                    BaseMemAddress + 0x404 + offs7, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SControl as ULONG,
                                    BaseMemAddress + 0x408 + offs7, MemIo, FALSE);
                                (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                            } else {
                                (*chan).MaxTransferMode = core::cmp::min(
                                    (*deviceExtension).MaxTransferMode, ATA_UDMA6);
                            }
                            c += 1;
                        }
                    }
                    ATA_ATI_ID | ATA_SILICON_IMAGE_ID => 'sii: {
                        if VendorID == ATA_ATI_ID {
                            if ChipType == ATI700 {
                                if (ChipFlags & UNIATA_AHCI) == 0 {
                                    chan = (*deviceExtension).chan.add(0);
                                    (*chan).MaxTransferMode = core::cmp::min(
                                        (*deviceExtension).MaxTransferMode, ATA_UDMA5);
                                }
                                break 'sii;
                            }
                            // else FALLTHROUGH to SII
                        }

                        if ChipType != SIIMIO {
                            break 'sii;
                        }
                        if pciData.is_null() {
                            break 'sii;
                        }

                        (*deviceExtension).AltRegMap = TRUE;
                        BaseMemAddress = AtapiGetIoRange(
                            HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                            5, 0, 0x800,
                        );
                        if BaseMemAddress == 0 {
                            return STATUS_UNSUCCESSFUL;
                        }
                        if access_range_in_memory(ConfigInfo, 5) {
                            MemIo = TRUE;
                        }
                        UniataInitIoResEx(
                            &mut (*deviceExtension).BaseIoAddressSATA_0,
                            BaseMemAddress, MemIo, FALSE,
                        );

                        c = 0;
                        while c < (*deviceExtension).NumberChannels {
                            let unit01: ULONG_PTR = (c & 1) as ULONG_PTR;
                            let unit10: ULONG_PTR = (c & 2) as ULONG_PTR;
                            chan = (*deviceExtension).chan.add(c as usize);

                            if (*deviceExtension).AltRegMap != 0 {
                                i = 0;
                                while i <= IDX_IO1_SZ as ULONG {
                                    UniataInitIoRes(chan, IDX_IO1 as ULONG + i,
                                        BaseMemAddress + 0x80 + i as ULONG_PTR
                                            + (unit01 << 6) + (unit10 << 8),
                                        MemIo, FALSE);
                                    i += 1;
                                }
                                UniataInitIoRes(chan, IDX_IO2_AltStatus as ULONG,
                                    BaseMemAddress + 0x8a + (unit01 << 6) + (unit10 << 8),
                                    MemIo, FALSE);
                                UniataInitSyncBaseIO(chan);

                                UniataInitIoRes(chan, IDX_BM_Command as ULONG,
                                    BaseMemAddress + 0x00 + (unit01 << 3) + (unit10 << 8),
                                    MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_BM_Status as ULONG,
                                    BaseMemAddress + 0x02 + (unit01 << 3) + (unit10 << 8),
                                    MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_BM_PRD_Table as ULONG,
                                    BaseMemAddress + 0x04 + (unit01 << 3) + (unit10 << 8),
                                    MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_BM_DeviceSpecific0 as ULONG,
                                    BaseMemAddress + 0x10 + (unit01 << 3) + (unit10 << 8),
                                    MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_BM_DeviceSpecific1 as ULONG,
                                    BaseMemAddress + 0x40 + (unit01 << 2) + (unit10 << 8),
                                    MemIo, FALSE);
                            }

                            if (*chan).MaxTransferMode < ATA_SA150 {
                                // do nothing for PATA part
                            } else if (ChipFlags & UNIATA_SATA) != 0 {
                                UniataInitIoRes(chan, IDX_SATA_SStatus as ULONG,
                                    BaseMemAddress + 0x104 + (unit01 << 7) + (unit10 << 8),
                                    MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SError as ULONG,
                                    BaseMemAddress + 0x108 + (unit01 << 2) + (unit10 << 8),
                                    MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SControl as ULONG,
                                    BaseMemAddress + 0x100 + (unit01 << 2) + (unit10 << 8),
                                    MemIo, FALSE);
                                (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                            }
                            c += 1;
                        }
                    }
                    ATA_SERVERWORKS_ID => 'swks: {
                        if ChipType != SWKSMIO {
                            break 'swks;
                        }
                        if pciData.is_null() {
                            break 'swks;
                        }
                        (*deviceExtension).AltRegMap = TRUE;
                        // 0x100 per port: the C mapped a fixed 0x400 (4 ports).
                        BaseMemAddress = AtapiGetIoRange(
                            HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                            5, 0, core::cmp::max(0x400, 0x100 * (*deviceExtension).NumberChannels),
                        );
                        if BaseMemAddress == 0 {
                            return STATUS_UNSUCCESSFUL;
                        }
                        if access_range_in_memory(ConfigInfo, 5) {
                            MemIo = TRUE;
                        }
                        UniataInitIoResEx(
                            &mut (*deviceExtension).BaseIoAddressSATA_0,
                            BaseMemAddress, MemIo, FALSE,
                        );
                        {
                            // Upstream svw_sata.c errata: clear SICR bit 18 so the
                            // chip stops waiting for a status message before
                            // sending a FIS (fixes Seagate drives), then clear
                            // pending interrupts.
                            let sata0 = (&(*deviceExtension).BaseIoAddressSATA_0) as *const _ as ULONG_PTR;
                            AtapiWritePortEx4(core::ptr::null_mut(), sata0, 0x80,
                                AtapiReadPortEx4(core::ptr::null_mut(), sata0, 0x80) & !0x00040000);
                            AtapiWritePortEx4(core::ptr::null_mut(), sata0, 0x44, 0xFFFFFFFF);
                            AtapiWritePortEx4(core::ptr::null_mut(), sata0, 0x88, 0);
                        }
                        c = 0;
                        while c < (*deviceExtension).NumberChannels {
                            let offs: ULONG_PTR = (c as ULONG_PTR) * 0x100;
                            chan = (*deviceExtension).chan.add(c as usize);
                            i = 0;
                            while i <= IDX_IO1_SZ as ULONG {
                                UniataInitIoRes(chan, IDX_IO1 as ULONG + i,
                                    BaseMemAddress + offs + (i as ULONG_PTR) * 4, MemIo, FALSE);
                                i += 1;
                            }
                            UniataInitIoRes(chan, IDX_IO2_AltStatus as ULONG,
                                BaseMemAddress + offs + 0x20, MemIo, FALSE);
                            UniataInitSyncBaseIO(chan);
                            UniataInitIoRes(chan, IDX_BM_Command as ULONG,
                                BaseMemAddress + offs + 0x30, MemIo, FALSE);
                            UniataInitIoRes(chan, IDX_BM_Status as ULONG,
                                BaseMemAddress + offs + 0x32, MemIo, FALSE);
                            UniataInitIoRes(chan, IDX_BM_PRD_Table as ULONG,
                                BaseMemAddress + offs + 0x34, MemIo, FALSE);
                            UniataInitIoRes(chan, IDX_SATA_SStatus as ULONG,
                                BaseMemAddress + offs + 0x40, MemIo, FALSE);
                            UniataInitIoRes(chan, IDX_SATA_SError as ULONG,
                                BaseMemAddress + offs + 0x44, MemIo, FALSE);
                            UniataInitIoRes(chan, IDX_SATA_SControl as ULONG,
                                BaseMemAddress + offs + 0x48, MemIo, FALSE);
                            (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                            c += 1;
                        }
                    }
                    ATA_SIS_ID => 'sis: {
                        let mut SIS_182: BOOLEAN = FALSE;
                        if (ChipFlags & SIS_BASE) == 0 {
                            break 'sis; // Found SIS_SOUTH
                        }
                        // ChangePciConfig1(0x57, (a & 0x7f))
                        change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x57, |a| a & 0x7f);
                        // GetPciConfig4(0x00, tmp32)
                        ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                            SystemIoBusNumber, slotNumber,
                            &mut tmp32 as *mut _ as PVOID, 0x00, 4);
                        if tmp32 == ATA_SIS5518 {
                            ChipType = SIS133NEW;
                            (*deviceExtension).HwFlags =
                                ((*deviceExtension).HwFlags & !CHIPTYPE_MASK) | SIS133NEW;
                            (*deviceExtension).MaxTransferMode = ATA_UDMA6;
                            // Restore device ID
                            change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                                0x57, |a| a | 0x80);
                        } else {
                            let SiSSouthAdapters: [BUSMASTER_CONTROLLER_INFORMATION_BASE; 2] = [
                                bm_entry(0x0008, 0x1039, 0x10, ATA_MODE_NOT_SPEC, 0,
                                    b"SiS 961\0".as_ptr()),
                                bm_entry(0xffff, 0xffff, 0xff, ATA_MODE_NOT_SPEC, 0xffffffff,
                                    core::ptr::null()),
                            ];
                            // Save settings: GetPciConfig1(0x4a, tmp8)
                            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                                SystemIoBusNumber, slotNumber,
                                &mut tmp8 as *mut _ as PVOID, 0x4a, 1);
                            change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                                0x4a, |a| a | 0x10);
                            if tmp32 == ATA_SIS5513 || tmp32 == ATA_SIS5517 {
                                i = AtapiFindListedDev(
                                    SiSSouthAdapters.as_ptr() as *mut _,
                                    0xffffffff, HwDeviceExtension, SystemIoBusNumber,
                                    PCISLOTNUM_NOT_SPECIFIED, core::ptr::null_mut());
                                if i != BMLIST_TERMINATOR {
                                    (*deviceExtension).HwFlags =
                                        ((*deviceExtension).HwFlags & !CHIPTYPE_MASK) | SIS133OLD;
                                    (*deviceExtension).MaxTransferMode = ATA_UDMA6;
                                    if (SiSSouthAdapters[i as usize].RaidFlags & UNIATA_SATA) != 0 {
                                        (*deviceExtension).HwFlags |= UNIATA_SATA;
                                        if SiSSouthAdapters[i as usize].nDeviceId == 0x1182
                                            || SiSSouthAdapters[i as usize].nDeviceId == 0x1183
                                        {
                                            SIS_182 = TRUE;
                                        }
                                    }
                                } else {
                                    // SiS-South not found
                                    if tmp32 == ATA_SIS5517 {
                                        (*deviceExtension).HwFlags =
                                            ((*deviceExtension).HwFlags & !CHIPTYPE_MASK)
                                                | SIS100NEW;
                                        (*deviceExtension).MaxTransferMode = ATA_UDMA5;
                                    } else {
                                        // generic SiS33
                                    }
                                }
                            }
                            // Restore settings: SetPciConfig1(0x4a, tmp8)
                            ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG,
                                SystemIoBusNumber, slotNumber,
                                &mut tmp8 as *mut _ as PVOID, 0x4a, 1);
                            if ((*deviceExtension).HwFlags & UNIATA_SATA) != 0 {
                                BaseMemAddress = AtapiGetIoRange(
                                    HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                    5, 0, 0x400);
                                if BaseMemAddress != 0 {
                                    if access_range_in_memory(ConfigInfo, 5) {
                                        MemIo = TRUE;
                                    }
                                    UniataInitIoResEx(
                                        &mut (*deviceExtension).BaseIoAddressSATA_0,
                                        BaseMemAddress, MemIo, FALSE);
                                    c = 0;
                                    while c < (*deviceExtension).NumberChannels {
                                        let offs: ULONG_PTR = (c as ULONG_PTR)
                                            << (if SIS_182 != 0 { 5 } else { 6 });
                                        chan = (*deviceExtension).chan.add(c as usize);
                                        UniataInitIoRes(chan, IDX_SATA_SStatus as ULONG,
                                            BaseMemAddress + 0 + offs, MemIo, FALSE);
                                        UniataInitIoRes(chan, IDX_SATA_SError as ULONG,
                                            BaseMemAddress + 4 + offs, MemIo, FALSE);
                                        UniataInitIoRes(chan, IDX_SATA_SControl as ULONG,
                                            BaseMemAddress + 8 + offs, MemIo, FALSE);
                                        (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                                        c += 1;
                                    }
                                }
                            }
                        }
                    }
                    ATA_VIA_ID => 'via: {
                        if (ChipFlags & VIASATA) != 0 {
                            // 2 SATA without SATA registers on first channel + 1 PATA on 2nd
                            break 'via;
                        }
                        if (ChipFlags & UNIATA_SATA) != 0 {
                            let mut IoSize: ULONG = 0;
                            BaseMemAddress = 0;

                            match DeviceID {
                                0x3149 => IoSize = 0x80, // VIA 6420
                                0x3249 => IoSize = 0x40, // VIA 6421
                                _ => {}
                            }
                            if IoSize != 0 {
                                BaseMemAddress = AtapiGetIoRange(
                                    HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                    5, 0, IoSize * (*deviceExtension).NumberChannels);
                                if BaseMemAddress != 0 && access_range_in_memory(ConfigInfo, 5) {
                                    MemIo = TRUE;
                                }
                                UniataInitIoResEx(
                                    &mut (*deviceExtension).BaseIoAddressSATA_0,
                                    BaseMemAddress, MemIo, FALSE);
                            }
                            if BaseMemAddress != 0 {
                                if (ChipFlags & VIABAR) != 0 {
                                    let mut BaseIoAddressBM_0: ULONG;
                                    let mut BaseIo: ULONG_PTR;
                                    BaseIoAddressBM_0 = AtapiGetIoRange(
                                        HwDeviceExtension, ConfigInfo, pciData,
                                        SystemIoBusNumber, 4, 0,
                                        core::mem::size_of::<IDE_BUSMASTER_REGISTERS>() as ULONG
                                            * (*deviceExtension).NumberChannels) as ULONG;
                                    (*deviceExtension).AltRegMap = TRUE;
                                    c = 0;
                                    while c < (*deviceExtension).NumberChannels {
                                        chan = (*deviceExtension).chan.add(c as usize);
                                        BaseIo = AtapiGetIoRange(
                                            HwDeviceExtension, ConfigInfo, pciData,
                                            SystemIoBusNumber, c, 0,
                                            core::mem::size_of::<IDE_REGISTERS_1>() as ULONG
                                                + core::mem::size_of::<IDE_REGISTERS_2>()
                                                    as ULONG * 2);
                                        i = 0;
                                        while i <= IDX_IO1_SZ as ULONG {
                                            UniataInitIoRes(chan, IDX_IO1 as ULONG + i,
                                                BaseIo + i as ULONG_PTR, FALSE, FALSE);
                                            i += 1;
                                        }
                                        UniataInitIoRes(chan, IDX_IO2_AltStatus as ULONG,
                                            BaseIo
                                                + core::mem::size_of::<IDE_REGISTERS_1>()
                                                    as ULONG_PTR
                                                + 2,
                                            FALSE, FALSE);
                                        UniataInitSyncBaseIO(chan);
                                        i = 0;
                                        while i <= IDX_BM_IO_SZ as ULONG {
                                            UniataInitIoRes(chan, IDX_BM_IO as ULONG + i,
                                                BaseIoAddressBM_0 as ULONG_PTR
                                                    + core::mem::size_of::<IDE_BUSMASTER_REGISTERS>()
                                                        as ULONG_PTR
                                                        * c as ULONG_PTR
                                                    + i as ULONG_PTR,
                                                FALSE, FALSE);
                                            i += 1;
                                        }
                                        c += 1;
                                    }
                                }
                                c = 0;
                                while c < (*deviceExtension).NumberChannels {
                                    chan = (*deviceExtension).chan.add(c as usize);
                                    if (ChipFlags & VIABAR) != 0 && c == 2 {
                                        // Do not setup SATA registers for PATA part
                                        i = 0;
                                        while i <= IDX_SATA_IO_SZ as ULONG {
                                            UniataInitIoRes(chan, IDX_SATA_IO as ULONG + i,
                                                0, FALSE, FALSE);
                                            i += 1;
                                        }
                                        break;
                                    }
                                    UniataInitIoRes(chan, IDX_SATA_SStatus as ULONG,
                                        BaseMemAddress + (c as ULONG_PTR * IoSize as ULONG_PTR),
                                        MemIo, FALSE);
                                    UniataInitIoRes(chan, IDX_SATA_SError as ULONG,
                                        BaseMemAddress + 4
                                            + (c as ULONG_PTR * IoSize as ULONG_PTR),
                                        MemIo, FALSE);
                                    UniataInitIoRes(chan, IDX_SATA_SControl as ULONG,
                                        BaseMemAddress + 8
                                            + (c as ULONG_PTR * IoSize as ULONG_PTR),
                                        MemIo, FALSE);
                                    (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                                    c += 1;
                                }
                            }
                        }
                    }
                    ATA_INTEL_ID => 'intel: {
                        if (ChipFlags & UNIATA_SATA) == 0 {
                            break 'intel;
                        }
                        // the intel 31244 needs special care if in DPA mode
                        if DeviceID == 3200
                            && (*pciData).SubClass != PCI_DEV_SUBCLASS_IDE
                        {
                            BaseMemAddress = AtapiGetIoRange(
                                HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                0, 0, 0x0c00);
                            if BaseMemAddress == 0 {
                                return STATUS_UNSUCCESSFUL;
                            }
                            if access_range_in_memory(ConfigInfo, 0) {
                                MemIo = TRUE;
                            }
                            (*deviceExtension).AltRegMap = TRUE;
                            UniataInitIoResEx(
                                &mut (*deviceExtension).BaseIoAddressSATA_0,
                                BaseMemAddress, MemIo, FALSE);
                            c = 0;
                            while c < (*deviceExtension).NumberChannels {
                                let offs: ULONG_PTR = 0x200 + (c as ULONG_PTR) * 0x200;
                                chan = (*deviceExtension).chan.add(c as usize);
                                i = 0;
                                while i <= IDX_IO1_SZ as ULONG {
                                    UniataInitIoRes(chan, IDX_BM_IO as ULONG + i,
                                        BaseMemAddress + (i as ULONG_PTR) * 4 + offs,
                                        MemIo, FALSE);
                                    i += 1;
                                }
                                UniataInitSyncBaseIO(chan);
                                UniataInitIoRes(chan, IDX_IO1_o_Command as ULONG,
                                    BaseMemAddress + 0x1d + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_IO1_o_Feature as ULONG,
                                    BaseMemAddress + 0x06 + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_IO2_o_Control as ULONG,
                                    BaseMemAddress + 0x29 + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_IO2_AltStatus as ULONG,
                                    BaseMemAddress + 0x28 + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_BM_Command as ULONG,
                                    BaseMemAddress + 0x70 + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_BM_Status as ULONG,
                                    BaseMemAddress + 0x72 + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_BM_PRD_Table as ULONG,
                                    BaseMemAddress + 0x74 + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SStatus as ULONG,
                                    BaseMemAddress + 0x100 + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SError as ULONG,
                                    BaseMemAddress + 0x104 + offs, MemIo, FALSE);
                                UniataInitIoRes(chan, IDX_SATA_SControl as ULONG,
                                    BaseMemAddress + 0x108 + offs, MemIo, FALSE);
                                (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                                c += 1;
                            }
                            break 'intel;
                        }
                        if (*deviceExtension).MaxTransferMode >= ATA_SA150 {
                            let mut OrigAHCI: BOOLEAN = FALSE;
                            // GetPciConfig1(0x90, tmp8)
                            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                                SystemIoBusNumber, slotNumber,
                                &mut tmp8 as *mut _ as PVOID, 0x90, 1);
                            MemIo = FALSE;
                            if (ChipFlags & UNIATA_AHCI) != 0 {
                                OrigAHCI = TRUE;
                                DbgPrint(b"[UNIATA] Intel MAP(0x90)=%02x subclass=%02x ar5=%x\n\0"
                                    .as_ptr().cast(), tmp8 as ULONG,
                                    (*pciData).SubClass as ULONG,
                                    access_range_start(ConfigInfo, 5)); // ahci marker
                                // The MAP/SMS check below only makes sense for parts that can
                                // sit in IDE-compat mode. A function whose PCI class already
                                // says AHCI (01/06, prog-if 01) is in AHCI mode by definition —
                                // QEMU's ich9-ahci leaves MAP(0x90)=0, and trusting it here
                                // dropped AHCI on a controller with no compat mode at all.
                                if (*pciData).SubClass == PCI_DEV_SUBCLASS_SATA
                                    && access_range_start(ConfigInfo, 5) != 0
                                {
                                    DbgPrint(b"[UNIATA] Intel: AHCI-class dev -> keep AHCI\n\0"
                                        .as_ptr().cast()); // ahci marker
                                    break 'intel;
                                }
                                if (tmp8 & 0xc0) != 0 {
                                    // MAP reg says AHCI/RAID mode -> try run AHCI (C: id_init.cpp
                                    // "try run AHCI"). AccessRanges[5] is NOT "always 0" here:
                                    // UniataAhciDetect() already ran above and its
                                    // AtapiGetIoRange(rid=5) self-heals the range from raw PCI
                                    // config (the CORE-13346 "assumed 0" note applies only
                                    // BEFORE detect). Non-zero -> keep AHCI, skip compat mode.
                                    // (Mistranslation here dropped AHCI unconditionally: every
                                    // Intel AHCI controller fell to legacy probe — dead on
                                    // AHCI-only parts like QEMU ich9-ahci.)
                                    if access_range_start(ConfigInfo, 5) != 0 {
                                        DbgPrint(b"[UNIATA] Intel MAP=%02x: keep AHCI\n\0"
                                            .as_ptr().cast(), tmp8 as ULONG); // ahci marker
                                        break 'intel;
                                    }
                                    DbgPrint(b"[UNIATA] Intel MAP=%02x: no BAR5 -> drop AHCI\n\0"
                                        .as_ptr().cast(), tmp8 as ULONG); // ahci marker
                                    (*deviceExtension).HwFlags &= !UNIATA_AHCI;
                                }
                                BaseIoAddressBM = AtapiGetIoRange(
                                    HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                    4, 0,
                                    core::mem::size_of::<IDE_BUSMASTER_REGISTERS>() as ULONG);
                                if BaseIoAddressBM != 0 {
                                    if access_range_in_memory(ConfigInfo, 4) {
                                        MemIo = TRUE;
                                    }
                                    UniataInitIoResEx(
                                        &mut (*deviceExtension).BaseIoAddressBM_0,
                                        BaseIoAddressBM, MemIo, FALSE);
                                    tmp8 = AtapiReadPortEx1(
                                        core::ptr::null_mut(),
                                        &mut (*deviceExtension).BaseIoAddressBM_0 as *mut _
                                            as ULONGIO_PTR,
                                        IDX_BM_Status as ULONG);
                                    // cleanup
                                    ScsiPortFreeDeviceBase(HwDeviceExtension,
                                        BaseIoAddressBM as PVOID);
                                    UniataInitIoResEx(
                                        &mut (*deviceExtension).BaseIoAddressBM_0,
                                        0, 0, FALSE);
                                    if tmp8 == 0xff {
                                        // invalid BM status, keep AHCI mode
                                        break 'intel;
                                    }
                                }
                                (*deviceExtension).NumberLuns = 2; // Legacy mode
                                if UniataAllocateLunExt(deviceExtension, 2) == 0 {
                                    return STATUS_UNSUCCESSFUL;
                                }
                            }
                            (*deviceExtension).HwFlags &= !UNIATA_AHCI;

                            MemIo = FALSE;
                            // if BAR(5) is IO it should point to SATA interface registers
                            if OrigAHCI != 0 {
                                BaseMemAddress = 0;
                            } else if (*deviceExtension).DevID == 0x28288086
                                && (*pciData).u.type0.as_ref().SubVendorID == 0x106b
                            {
                                // Skip BAR(5) on ICH8M Apples
                                BaseMemAddress = 0;
                            } else {
                                BaseMemAddress = AtapiGetIoRange(
                                    HwDeviceExtension, ConfigInfo, pciData, SystemIoBusNumber,
                                    5, 0, 0x10);
                                if BaseMemAddress != 0 && access_range_in_memory(ConfigInfo, 5) {
                                    MemIo = TRUE;
                                }
                            }
                            UniataInitIoResEx(
                                &mut (*deviceExtension).BaseIoAddressSATA_0,
                                BaseMemAddress, MemIo, FALSE);

                            c = 0;
                            while c < (*deviceExtension).NumberChannels {
                                chan = (*deviceExtension).chan.add(c as usize);
                                AtapiSetupLunPtrs(chan, deviceExtension, c);
                                IsPata = FALSE;
                                if (ChipFlags & ICH5) != 0 {
                                    if (tmp8 & 0x04) == 0 {
                                        (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                                    } else if (tmp8 & 0x02) == 0 {
                                        if c != 0 {
                                            IsPata = TRUE;
                                        }
                                    } else if (tmp8 & 0x02) != 0 {
                                        if c != 1 {
                                            IsPata = TRUE;
                                        }
                                    }
                                } else if (ChipFlags & I6CH2) != 0 {
                                    (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                                } else {
                                    match tmp8 & 0x03 {
                                        2 => {
                                            if c != 0 {
                                                IsPata = TRUE;
                                            }
                                        }
                                        1 => {
                                            if c != 1 {
                                                IsPata = TRUE;
                                            }
                                        }
                                        _ => {}
                                    }
                                }

                                if IsPata != 0 {
                                    (*chan).MaxTransferMode = core::cmp::min(
                                        (*deviceExtension).MaxTransferMode, ATA_UDMA5);
                                } else {
                                    if (ChipFlags & ICH7) == 0 && BaseMemAddress != 0 {
                                        UniataInitIoRes(chan, IDX_INDEXED_ADDR as ULONG,
                                            BaseMemAddress + 0, MemIo, FALSE);
                                        UniataInitIoRes(chan, IDX_INDEXED_DATA as ULONG,
                                            BaseMemAddress + 4, MemIo, FALSE);
                                    }
                                    if (ChipFlags & ICH5) != 0 || BaseMemAddress != 0 {
                                        // Rather interesting way of register access...
                                        ChipType = INTEL_IDX;
                                        (*deviceExtension).HwFlags &= !CHIPTYPE_MASK;
                                        (*deviceExtension).HwFlags |= ChipType;
                                        // these are fake non-zero values
                                        UniataInitIoRes(chan, IDX_SATA_SStatus as ULONG,
                                            0x200 * c as ULONG_PTR + 0, FALSE, TRUE);
                                        UniataInitIoRes(chan, IDX_SATA_SError as ULONG,
                                            0x200 * c as ULONG_PTR + 2, FALSE, TRUE);
                                        UniataInitIoRes(chan, IDX_SATA_SControl as ULONG,
                                            0x200 * c as ULONG_PTR + 1, FALSE, TRUE);
                                    }
                                }
                                c += 1;
                            }
                            // rest of INIT staff is in AtapiChipInit()
                        }
                    }
                    ATA_CYRIX_ID => {
                        // Cyrix 5530 ATA33 controller
                        if (*deviceExtension).DevID == 0x01021078 {
                            (*ConfigInfo).AlignmentMask = 0x0f;
                            (*deviceExtension).MaximumDmaTransferLength = 63 * 1024;
                        }
                    }
                    ATA_JMICRON_ID => {
                        // New JMicron PATA/SATA controllers
                        // GetPciConfig1(0xdf, tmp8)
                        ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                            SystemIoBusNumber, slotNumber,
                            &mut tmp8 as *mut _ as PVOID, 0xdf, 1);
                        if (tmp8 & 0x40) != 0 {
                            if Ata_is_ahci_dev(pciData) {
                                ChipFlags |= UNIATA_AHCI;
                                (*deviceExtension).HwFlags |= UNIATA_AHCI;
                            } else {
                                // JMicron PATA/SATA
                            }
                        } else {
                            // JMicron Combined
                        }
                    }
                    _ => {}
                }

                return STATUS_SUCCESS;
            }

            Flow::Unknown => {
                // unknown_dev:
                if Ata_is_ahci_dev(pciData) {
                    (*deviceExtension).NumberChannels = 0;
                    if UniataAhciDetect(HwDeviceExtension, pciData as *mut _, ConfigInfo) == 0 {
                        return STATUS_UNSUCCESSFUL;
                    }
                }
                DevTypeInfo = core::ptr::null_mut();

                if UniataChipDetectChannels(
                    HwDeviceExtension, pciData, DeviceNumber, ConfigInfo,
                ) == 0
                {
                    return STATUS_UNSUCCESSFUL;
                }
                if UniataAllocateLunExt(deviceExtension, UNIATA_ALLOCATE_NEW_LUNS) == 0 {
                    return STATUS_UNSUCCESSFUL;
                }
                return STATUS_SUCCESS;
            }
        }
    }
}

// ConfigInfo->AccessRanges[rid].RangeInMemory accessor.
#[inline]
unsafe fn access_range_in_memory(
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    rid: usize,
) -> bool {
    let ranges = (*ConfigInfo).AccessRanges; // PACCESS_RANGE (== *[ACCESS_RANGE; N])
    if ranges.is_null() {
        return false;
    }
    let ar = (ranges as *const ACCESS_RANGE).add(rid);
    (*ar).RangeInMemory != 0
}

// C: ScsiPortConvertPhysicalAddressToUlong((*ConfigInfo->AccessRanges)[rid].RangeStart)
unsafe fn access_range_start(
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    rid: usize,
) -> ULONG {
    let ranges = (*ConfigInfo).AccessRanges;
    if ranges.is_null() || ((*ConfigInfo).NumberOfAccessRanges as usize) <= rid {
        return 0;
    }
    let ar = (ranges as *const ACCESS_RANGE).add(rid);
    ScsiPortConvertPhysicalAddressToUlong((*ar).RangeStart)
}

// ChangePciConfigN inline helper: read-modify-write one byte at `offs`.
#[inline]
unsafe fn change_pci_config1(
    HwDeviceExtension: PVOID,
    SystemIoBusNumber: ULONG,
    slotNumber: ULONG,
    offs: ULONG,
    f: impl Fn(UCHAR) -> UCHAR,
) {
    let mut a: UCHAR = 0;
    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
        slotNumber, &mut a as *mut _ as PVOID, offs, 1);
    a = f(a);
    ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
        slotNumber, &mut a as *mut _ as PVOID, offs, 1);
}

// --- function-local device-table builders (mirror id_init.cpp's local statics) ---
fn sis_adapters() -> [BUSMASTER_CONTROLLER_INFORMATION_BASE; 31] {
    [
        bm_entry(0x1183, 0x1039, 0x00, ATA_SA150, SIS133NEW, b"SiS 1183 IDE\0".as_ptr()),
        bm_entry(0x1182, 0x1039, 0x00, ATA_SA150, SISSATA | UNIATA_SATA, b"SiS 1182\0".as_ptr()),
        bm_entry(0x0183, 0x1039, 0x00, ATA_SA150, SISSATA | UNIATA_SATA, b"SiS 183 RAID\0".as_ptr()),
        bm_entry(0x0182, 0x1039, 0x00, ATA_SA150, SISSATA | UNIATA_SATA, b"SiS 182\0".as_ptr()),
        bm_entry(0x0181, 0x1039, 0x00, ATA_SA150, SISSATA | UNIATA_SATA, b"SiS 181\0".as_ptr()),
        bm_entry(0x0180, 0x1039, 0x00, ATA_SA150, SISSATA | UNIATA_SATA, b"SiS 180\0".as_ptr()),
        bm_entry(0x0965, 0x1039, 0x00, ATA_UDMA6, SIS133NEW, b"SiS 965\0".as_ptr()),
        bm_entry(0x0964, 0x1039, 0x00, ATA_UDMA6, SIS133NEW, b"SiS 964\0".as_ptr()),
        bm_entry(0x0963, 0x1039, 0x00, ATA_UDMA6, SIS133NEW, b"SiS 963\0".as_ptr()),
        bm_entry(0x0962, 0x1039, 0x00, ATA_UDMA6, SIS133NEW, b"SiS 962\0".as_ptr()),
        bm_entry(0x0745, 0x1039, 0x00, ATA_UDMA5, SIS100NEW, b"SiS 745\0".as_ptr()),
        bm_entry(0x0735, 0x1039, 0x00, ATA_UDMA5, SIS100NEW, b"SiS 735\0".as_ptr()),
        bm_entry(0x0733, 0x1039, 0x00, ATA_UDMA5, SIS100NEW, b"SiS 733\0".as_ptr()),
        bm_entry(0x0730, 0x1039, 0x00, ATA_UDMA5, SIS100OLD, b"SiS 730\0".as_ptr()),
        bm_entry(0x0646, 0x1039, 0x00, ATA_UDMA6, SIS133NEW, b"SiS 645DX\0".as_ptr()),
        bm_entry(0x0635, 0x1039, 0x00, ATA_UDMA5, SIS100NEW, b"SiS 635\0".as_ptr()),
        bm_entry(0x0633, 0x1039, 0x00, ATA_UDMA5, SIS100NEW, b"SiS 633\0".as_ptr()),
        bm_entry(0x0630, 0x1039, 0x30, ATA_UDMA5, SIS100OLD, b"SiS 630S\0".as_ptr()),
        bm_entry(0x0630, 0x1039, 0x00, ATA_UDMA4, SIS66, b"SiS 630\0".as_ptr()),
        bm_entry(0x0620, 0x1039, 0x00, ATA_UDMA4, SIS66, b"SiS 620\0".as_ptr()),
        bm_entry(0x0550, 0x1039, 0x00, ATA_UDMA5, SIS66, b"SiS 550\0".as_ptr()),
        bm_entry(0x0540, 0x1039, 0x00, ATA_UDMA4, SIS66, b"SiS 540\0".as_ptr()),
        bm_entry(0x0530, 0x1039, 0x00, ATA_UDMA4, SIS66, b"SiS 530\0".as_ptr()),
        bm_entry(0x5517, 0x1039, 0x00, ATA_UDMA5, SIS100NEW | SIS_BASE, b"SiS 961\0".as_ptr()),
        bm_entry(0x5518, 0x1039, 0x00, ATA_UDMA6, SIS133NEW | SIS_BASE, b"SiS 962/3\0".as_ptr()),
        bm_entry(0x5513, 0x1039, 0xc2, ATA_UDMA2, SIS33 | SIS_BASE, b"SiS 5513\0".as_ptr()),
        bm_entry(0x5513, 0x1039, 0x00, ATA_WDMA2, SIS33 | SIS_BASE, b"SiS 5513\0".as_ptr()),
        bm_entry(0x0601, 0x1039, 0x00, ATA_UDMA2, SIS33 | SIS_BASE, b"SiS 5513\0".as_ptr()),
        bm_entry(0xffff, 0xffff, 0xff, BMLIST_TERMINATOR, BMLIST_TERMINATOR, core::ptr::null()),
        // padding entries to keep a fixed-size array (terminator already present above);
        // duplicate terminator is harmless — AtapiFindListedDev stops at the first.
        bm_entry(0xffff, 0xffff, 0xff, BMLIST_TERMINATOR, BMLIST_TERMINATOR, core::ptr::null()),
        bm_entry(0xffff, 0xffff, 0xff, BMLIST_TERMINATOR, BMLIST_TERMINATOR, core::ptr::null()),
    ]
}

fn via_adapters() -> [BUSMASTER_CONTROLLER_INFORMATION_BASE; 24] {
    [
        bm_entry(0x0586, 0x1106, 0x41, ATA_UDMA2, VIA33 | 0x00, b"VIA 82C586B\0".as_ptr()),
        bm_entry(0x0586, 0x1106, 0x40, ATA_UDMA2, VIA33 | VIAPRQ, b"VIA 82C586B\0".as_ptr()),
        bm_entry(0x0586, 0x1106, 0x02, ATA_UDMA2, VIA33 | 0x00, b"VIA 82C586B\0".as_ptr()),
        bm_entry(0x0586, 0x1106, 0x00, ATA_WDMA2, VIA33 | 0x00, b"VIA 82C586\0".as_ptr()),
        bm_entry(0x0596, 0x1106, 0x12, ATA_UDMA4, VIA66 | VIACLK, b"VIA 82C596B\0".as_ptr()),
        bm_entry(0x0596, 0x1106, 0x00, ATA_UDMA2, VIA33 | 0x00, b"VIA 82C596\0".as_ptr()),
        bm_entry(0x0686, 0x1106, 0x40, ATA_UDMA5, VIA100 | VIABUG, b"VIA 82C686B\0".as_ptr()),
        bm_entry(0x0686, 0x1106, 0x10, ATA_UDMA4, VIA66 | VIACLK, b"VIA 82C686A\0".as_ptr()),
        bm_entry(0x0686, 0x1106, 0x00, ATA_UDMA2, VIA33 | 0x00, b"VIA 82C686\0".as_ptr()),
        bm_entry(0x8231, 0x1106, 0x00, ATA_UDMA5, VIA100 | VIABUG, b"VIA 8231\0".as_ptr()),
        bm_entry(0x3074, 0x1106, 0x00, ATA_UDMA5, VIA100 | 0x00, b"VIA 8233\0".as_ptr()),
        bm_entry(0x3109, 0x1106, 0x00, ATA_UDMA5, VIA100 | 0x00, b"VIA 8233C\0".as_ptr()),
        bm_entry(0x3147, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA 8233A\0".as_ptr()),
        bm_entry(0x3177, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA 8235\0".as_ptr()),
        bm_entry(0x3227, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA 8237\0".as_ptr()),
        bm_entry(0x0591, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA 8237A\0".as_ptr()),
        bm_entry(0x5337, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA 8237S\0".as_ptr()),
        bm_entry(0x5372, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA 8237\0".as_ptr()),
        bm_entry(0x7372, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA 8237\0".as_ptr()),
        bm_entry(0x3349, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA 8251\0".as_ptr()),
        bm_entry(0x8324, 0x1106, 0x00, ATA_SA150, VIANEW | VIASATA, b"VIA CX700\0".as_ptr()),
        bm_entry(0x8353, 0x1106, 0x00, ATA_SA150, VIANEW | VIASATA, b"VIA VX800\0".as_ptr()),
        bm_entry(0x8409, 0x1106, 0x00, ATA_UDMA6, VIA133 | 0x00, b"VIA VX855\0".as_ptr()),
        bm_entry(0x8410, 0x1106, 0x00, ATA_SA300, VIANEW | VIASATA, b"VIA VX900\0".as_ptr()),
    ]
}

fn via_south_adapters() -> [BUSMASTER_CONTROLLER_INFORMATION_BASE; 5] {
    [
        bm_entry(0x3112, 0x1106, 0x00, ATA_MODE_NOT_SPEC, VIASOUTH, b"VIA 8361\0".as_ptr()),
        bm_entry(0x0305, 0x1106, 0x00, ATA_MODE_NOT_SPEC, VIASOUTH, b"VIA 8363\0".as_ptr()),
        bm_entry(0x0391, 0x1106, 0x00, ATA_MODE_NOT_SPEC, VIASOUTH, b"VIA 8371\0".as_ptr()),
        bm_entry(0x3102, 0x1106, 0x00, ATA_MODE_NOT_SPEC, VIASOUTH, b"VIA 8662\0".as_ptr()),
        bm_entry(0xffff, 0xffff, 0xff, BMLIST_TERMINATOR, BMLIST_TERMINATOR, core::ptr::null()),
    ]
}

// ===========================================================================
// AtapiViaSouthBridgeFixup
// ===========================================================================
pub unsafe extern "system" fn AtapiViaSouthBridgeFixup(
    HwDeviceExtension: PVOID,
    BusDataType: BUS_DATA_TYPE,
    SystemIoBusNumber: ULONG,
    slotNumber: ULONG,
) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();
    let mut found: BOOLEAN = FALSE;

    slotData.u.AsULONG = slotNumber;
    let mut funcNumber: ULONG = 0;
    while funcNumber < PCI_MAX_FUNCTION {
        slotData.u.bits.set_FunctionNumber(funcNumber);

        let busDataRead = ScsiPortGetBusData(
            HwDeviceExtension,
            PCIConfiguration as ULONG,
            SystemIoBusNumber,
            slotData.u.AsULONG,
            &mut pciData as *mut _ as PVOID,
            PCI_COMMON_HDR_LENGTH,
        );
        if busDataRead < PCI_COMMON_HDR_LENGTH {
            funcNumber += 1;
            continue;
        }

        let VendorID = pciData.VendorID as ULONG;
        let DeviceID = pciData.DeviceID as ULONG;
        let dev_id = VendorID | (DeviceID << 16);

        if dev_id == 0x03051106
            || dev_id == 0x03911106
            || dev_id == 0x31021106
            || dev_id == 0x31121106
        {
            let mut reg76: UCHAR = 0;
            // GetPciConfig1(0x76, reg76) — local slotNumber == this func's slotData
            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
                slotNumber, &mut reg76 as *mut _ as PVOID, 0x76, 1);
            if (reg76 & 0xf0) != 0xd0 {
                let mut v: UCHAR = 0x80;
                ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
                    slotNumber, &mut v as *mut _ as PVOID, 0x75, 1);
                let mut v2: UCHAR = (reg76 & 0x0f) | 0xd0;
                ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
                    slotNumber, &mut v2 as *mut _ as PVOID, 0x76, 1);
            }
            found = TRUE;
            break;
        }
        funcNumber += 1;
    }
    if found == 0 {
        (*deviceExtension).HwFlags &= !VIABUG;
    }
}

// ===========================================================================
// AtapiRosbSouthBridgeFixup
// ===========================================================================
pub unsafe extern "system" fn AtapiRosbSouthBridgeFixup(
    HwDeviceExtension: PVOID,
    BusDataType: BUS_DATA_TYPE,
    SystemIoBusNumber: ULONG,
    slotNumber: ULONG,
) {
    let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();

    slotData.u.AsULONG = slotNumber;
    let mut funcNumber: ULONG = 0;
    while funcNumber < PCI_MAX_FUNCTION {
        slotData.u.bits.set_FunctionNumber(funcNumber);

        let busDataRead = ScsiPortGetBusData(
            HwDeviceExtension,
            PCIConfiguration as ULONG, SystemIoBusNumber,
            slotData.u.AsULONG, &mut pciData as *mut _ as PVOID, PCI_COMMON_HDR_LENGTH,
        );
        if busDataRead < PCI_COMMON_HDR_LENGTH {
            funcNumber += 1;
            continue;
        }

        let VendorID = pciData.VendorID as ULONG;
        let DeviceID = pciData.DeviceID as ULONG;
        let dev_id = VendorID | (DeviceID << 16);

        if dev_id == ATA_ROSB4_ISA {
            // ChangePciConfig4(0x64, ((a & ~0x00002000) | 0x00004000))
            let mut a: ULONG = 0;
            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
                slotNumber, &mut a as *mut _ as PVOID, 0x64, 4);
            a = (a & !0x00002000) | 0x00004000;
            ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
                slotNumber, &mut a as *mut _ as PVOID, 0x64, 4);
            break;
        }
        funcNumber += 1;
    }
}

// ===========================================================================
// AtapiAliSouthBridgeFixup
// ===========================================================================
pub unsafe extern "system" fn AtapiAliSouthBridgeFixup(
    HwDeviceExtension: PVOID,
    BusDataType: BUS_DATA_TYPE,
    SystemIoBusNumber: ULONG,
    slotNumber: ULONG,
    c: ULONG,
) {
    let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();

    slotData.u.AsULONG = slotNumber;
    let mut funcNumber: ULONG = 0;
    while funcNumber < PCI_MAX_FUNCTION {
        slotData.u.bits.set_FunctionNumber(funcNumber);

        let busDataRead = ScsiPortGetBusData(
            HwDeviceExtension,
            PCIConfiguration as ULONG, SystemIoBusNumber,
            slotData.u.AsULONG, &mut pciData as *mut _ as PVOID, PCI_COMMON_HDR_LENGTH,
        );
        if busDataRead < PCI_COMMON_HDR_LENGTH {
            funcNumber += 1;
            continue;
        }

        let VendorID = pciData.VendorID as ULONG;
        let DeviceID = pciData.DeviceID as ULONG;
        let dev_id = VendorID | (DeviceID << 16);

        if dev_id == ATA_ALI_1533 {
            // ChangePciConfig1(0x58, (a & ~(0x04 << c)));  then (a | (0x04 << c));
            change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x58,
                |a| a & !((0x04u8) << c));
            change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x58,
                |a| a | ((0x04u8) << c));
            break;
        }
        funcNumber += 1;
    }
}

// ===========================================================================
// hpt_cable80
// ===========================================================================
pub unsafe extern "system" fn hpt_cable80(
    deviceExtension: PHW_DEVICE_EXTENSION,
    channel: ULONG, // physical channel number (0-1)
) -> ULONG {
    let HwDeviceExtension = deviceExtension as PVOID;
    let slotNumber = (*deviceExtension).slotNumber;
    let SystemIoBusNumber = (*deviceExtension).SystemIoBusNumber;
    let ChipType = (*deviceExtension).HwFlags & CHIPTYPE_MASK;

    let mut reg: ULONG;
    let mut val: UCHAR = 0;
    let mut res: UCHAR = 0;
    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();

    let c = channel - (*deviceExtension).Channel; // logical channel
    let chan = (*deviceExtension).chan.add(c as usize);

    slotData.u.AsULONG = (*deviceExtension).slotNumber;

    if ((*deviceExtension).HwFlags & UNIATA_NO80CHK) != 0 {
        return TRUE as ULONG;
    }

    if ChipType == HPT374 && slotData.u.bits.FunctionNumber() == 1 {
        reg = if channel != 0 { 0x57 } else { 0x53 };
        // GetPciConfig1(reg, val); SetPciConfig1(reg, val | 0x80)
        ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
            slotNumber, &mut val as *mut _ as PVOID, reg, 1);
        let mut v: UCHAR = val | 0x80;
        ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
            slotNumber, &mut v as *mut _ as PVOID, reg, 1);
    } else {
        reg = 0x5b;
        ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
            slotNumber, &mut val as *mut _ as PVOID, reg, 1);
        let mut v: UCHAR = val & 0xfe;
        ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
            slotNumber, &mut v as *mut _ as PVOID, reg, 1);
    }
    // GetPciConfig1(0x5a, res)
    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
        slotNumber, &mut res as *mut _ as PVOID, 0x5a, 1);
    res = res & (if channel != 0 { 0x01 } else { 0x02 });
    // SetPciConfig1(reg, val)
    ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
        slotNumber, &mut val as *mut _ as PVOID, reg, 1);
    if (*chan).Force80pin != 0 {
        res = 0;
    }
    (res == 0) as ULONG // return !res
}

// ===========================================================================
// generic_cable80
// ===========================================================================
pub unsafe extern "system" fn generic_cable80(
    deviceExtension: PHW_DEVICE_EXTENSION,
    channel: ULONG, // physical channel number (0-1)
    pci_reg: ULONG,
    bit_offs: ULONG,
) -> BOOLEAN {
    let HwDeviceExtension = deviceExtension as PVOID;
    let slotNumber = (*deviceExtension).slotNumber;
    let SystemIoBusNumber = (*deviceExtension).SystemIoBusNumber;

    if (*deviceExtension).MaxTransferMode <= ATA_UDMA2 {
        return FALSE;
    }

    let c = channel - (*deviceExtension).Channel; // logical channel
    let chan = (*deviceExtension).chan.add(c as usize);
    let mut tmp8: UCHAR = 0;

    if (*chan).Force80pin != 0 {
        return TRUE;
    }

    // GetPciConfig1(pci_reg, tmp8)
    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
        slotNumber, &mut tmp8 as *mut _ as PVOID, pci_reg, 1);
    if (tmp8 & (1u8 << (channel << bit_offs))) == 0 {
        (*chan).MaxTransferMode =
            core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
        return FALSE;
    }

    TRUE
}

// ===========================================================================
// UniAtaReadLunConfig
// ===========================================================================
pub unsafe extern "system" fn UniAtaReadLunConfig(
    deviceExtension: PHW_DEVICE_EXTENSION,
    channel: ULONG, // physical channel
    mut DeviceNumber: ULONG,
) {
    let mut tmp32: ULONG;
    let c = channel - (*deviceExtension).Channel; // logical channel
    let chan = (*deviceExtension).chan.add(c as usize);
    DeviceNumber = DeviceNumber % (*deviceExtension).NumberLuns;
    let LunExt = (*chan).lun[DeviceNumber as usize];

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("ReadCacheEnable"), 1);
    (*LunExt).opt_ReadCacheEnable = if tmp32 != 0 { TRUE } else { FALSE };

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("WriteCacheEnable"), 1);
    (*LunExt).opt_WriteCacheEnable = if tmp32 != 0 { TRUE } else { FALSE };

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("MaxTransferMode"), (*chan).MaxTransferMode);
    (*LunExt).opt_MaxTransferMode = tmp32;

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("PreferedTransferMode"), 0xffffffff);
    (*LunExt).opt_PreferedTransferMode = tmp32;

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("AdvancedPowerMode"), ATA_C_F_APM_CNT_MIN_NO_STANDBY);
    if tmp32 > 0xfe {
        tmp32 = 0xfe;
    }
    (*LunExt).opt_AdvPowerMode = tmp32 as UCHAR;

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("AcousticMgmt"), ATA_C_F_AAM_CNT_MAX_POWER_SAVE);
    if tmp32 > 0xfe {
        tmp32 = 0xfe;
    } else if tmp32 < 0x80 {
        tmp32 = 0x0;
    }
    (*LunExt).opt_AcousticMode = tmp32 as UCHAR;

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("StandbyTimer"), 0);
    if tmp32 == 0xfe {
        tmp32 = 0xff;
    }
    (*LunExt).opt_StandbyTimer = tmp32 as UCHAR;

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("ReadOnly"), 0);
    if tmp32 <= 2 {
        (*LunExt).opt_ReadOnly = tmp32 as UCHAR;
    }

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("GeomType"), 0xffffffff);
    if tmp32 > GEOM_MANUAL {
        tmp32 = 0xffffffff;
    }
    (*LunExt).opt_GeomType = tmp32;
    if tmp32 == GEOM_MANUAL {
        (*LunExt).DeviceFlags |= DFLAGS_MANUAL_CHS;
        (*LunExt).opt_GeomType = GEOM_ORIG;
        // assume IdentifyData is already zero-filled
        tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
            wstr!("C"), 0);
        (*LunExt).IdentifyData.NumberOfCylinders = tmp32 as USHORT;
        (*LunExt).IdentifyData.NumberOfCurrentCylinders = tmp32 as USHORT;
        tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
            wstr!("H"), 0);
        (*LunExt).IdentifyData.NumberOfHeads = tmp32 as USHORT;
        (*LunExt).IdentifyData.NumberOfCurrentHeads = tmp32 as USHORT;
        tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
            wstr!("S"), 0);
        (*LunExt).IdentifyData.SectorsPerTrack = tmp32 as USHORT;
        (*LunExt).IdentifyData.CurrentSectorsPerTrack = tmp32 as USHORT;
        // memcpy(ModelNumber, "SEIDH DD", 8) etc. — element-wise byte copies.
        copy_bytes(
            core::ptr::addr_of_mut!((*LunExt).IdentifyData.ModelNumber) as *mut u8,
            b"SEIDH DD",
            8,
        );
        copy_bytes(
            core::ptr::addr_of_mut!((*LunExt).IdentifyData.SerialNumber) as *mut u8,
            b".10\0",
            4,
        );
        copy_bytes(
            core::ptr::addr_of_mut!((*LunExt).IdentifyData.FirmwareRevision) as *mut u8,
            b".10\0",
            4,
        );
        if (*LunExt).IdentifyData.SectorsPerTrack == 0
            || (*LunExt).IdentifyData.NumberOfCylinders == 0
            || (*LunExt).IdentifyData.NumberOfHeads == 0
        {
            (*LunExt).opt_GeomType = GEOM_AUTO;
        } else {
            (*LunExt).DeviceFlags |= DFLAGS_MANUAL_CHS;
            (*LunExt).opt_GeomType = GEOM_ORIG;
        }
    }

    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("Hidden"), 0);
    if tmp32 != 0 {
        (*LunExt).DeviceFlags |= DFLAGS_HIDDEN;
    }
    tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel, DeviceNumber,
        wstr!("Exclude"), 0);
    if tmp32 != 0 {
        (*LunExt).DeviceFlags |= DFLAGS_HIDDEN;
    }
}

#[inline]
unsafe fn copy_bytes(dst: *mut u8, src: &[u8], n: usize) {
    let m = core::cmp::min(n, src.len());
    core::ptr::copy_nonoverlapping(src.as_ptr(), dst, m);
}

// ===========================================================================
// AtapiReadChipConfig
// ===========================================================================
pub unsafe extern "system" fn AtapiReadChipConfig(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    channel: ULONG, // physical channel
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut chan: PHW_CHANNEL;
    let mut tmp32: ULONG;
    let mut c: ULONG; // logical channel
    let mut i: ULONG;

    if channel != CHAN_NOT_SPECIFIED {
        c = channel - (*deviceExtension).Channel;
    } else {
        c = CHAN_NOT_SPECIFIED;
    }

    if channel == CHAN_NOT_SPECIFIED {
        if AtapiRegCheckDevValue(deviceExtension as PVOID, CHAN_NOT_SPECIFIED,
            DEVNUM_NOT_SPECIFIED, wstr!("ForceSimplex"), FALSE as ULONG) != 0
        {
            (*deviceExtension).simplexOnly = TRUE;
        }
        (*deviceExtension).opt_AtapiDmaZeroTransfer = FALSE;
        (*deviceExtension).opt_AtapiDmaControlCmd = FALSE;
        (*deviceExtension).opt_AtapiDmaRawRead = g_opt_AtapiDmaRawRead;
        (*deviceExtension).opt_AtapiDmaReadWrite = TRUE;
    }

    if c == CHAN_NOT_SPECIFIED {
        c = 0;
        while c < (*deviceExtension).NumberChannels {
            chan = (*deviceExtension).chan.add(c as usize);
            (*chan).MaxTransferMode = (*deviceExtension).MaxTransferMode;
            tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, channel,
                DEVNUM_NOT_SPECIFIED, wstr!("MaxTransferMode"), (*chan).MaxTransferMode);
            if tmp32 != 0xffffffff {
                (*chan).MaxTransferMode = tmp32;
            }
            tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, c,
                DEVNUM_NOT_SPECIFIED, wstr!("Force80pin"), FALSE as ULONG);
            (*chan).Force80pin = if tmp32 != 0 { TRUE } else { FALSE };
            if (*chan).Force80pin != 0 {
                (*deviceExtension).HwFlags |= UNIATA_NO80CHK;
            }
            c += 1;
        }

        (*deviceExtension).opt_AtapiDmaZeroTransfer =
            if AtapiRegCheckDevValue(deviceExtension as PVOID, CHAN_NOT_SPECIFIED,
                DEVNUM_NOT_SPECIFIED, wstr!("AtapiDmaZeroTransfer"),
                (*deviceExtension).opt_AtapiDmaZeroTransfer as ULONG) != 0 { TRUE } else { FALSE };

        (*deviceExtension).opt_AtapiDmaControlCmd =
            if AtapiRegCheckDevValue(deviceExtension as PVOID, CHAN_NOT_SPECIFIED,
                DEVNUM_NOT_SPECIFIED, wstr!("AtapiDmaControlCmd"),
                (*deviceExtension).opt_AtapiDmaControlCmd as ULONG) != 0 { TRUE } else { FALSE };

        (*deviceExtension).opt_AtapiDmaRawRead =
            if AtapiRegCheckDevValue(deviceExtension as PVOID, CHAN_NOT_SPECIFIED,
                DEVNUM_NOT_SPECIFIED, wstr!("AtapiDmaRawRead"),
                (*deviceExtension).opt_AtapiDmaRawRead as ULONG) != 0 { TRUE } else { FALSE };

        (*deviceExtension).opt_AtapiDmaReadWrite =
            if AtapiRegCheckDevValue(deviceExtension as PVOID, CHAN_NOT_SPECIFIED,
                DEVNUM_NOT_SPECIFIED, wstr!("AtapiDmaReadWrite"),
                (*deviceExtension).opt_AtapiDmaReadWrite as ULONG) != 0 { TRUE } else { FALSE };
    } else {
        chan = (*deviceExtension).chan.add(c as usize);
        (*chan).MaxTransferMode = (*deviceExtension).MaxTransferMode;
        tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, c,
            DEVNUM_NOT_SPECIFIED, wstr!("MaxTransferMode"), (*chan).MaxTransferMode);
        if tmp32 != 0xffffffff {
            (*chan).MaxTransferMode = tmp32;
        }
        tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, c,
            DEVNUM_NOT_SPECIFIED, wstr!("ReorderEnable"), TRUE as ULONG);
        (*chan).UseReorder = if tmp32 != 0 { TRUE } else { FALSE };

        tmp32 = AtapiRegCheckDevValue(deviceExtension as PVOID, c,
            DEVNUM_NOT_SPECIFIED, wstr!("Force80pin"), FALSE as ULONG);
        (*chan).Force80pin = if tmp32 != 0 { TRUE } else { FALSE };

        i = 0;
        while i < (*deviceExtension).NumberLuns {
            UniAtaReadLunConfig(deviceExtension, channel, i);
            i += 1;
        }
    }

    TRUE
}

// ===========================================================================
// AtapiChipInit
// ===========================================================================
pub unsafe extern "system" fn AtapiChipInit(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    mut channel: ULONG, // logical channel
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let slotNumber = (*deviceExtension).slotNumber;
    let SystemIoBusNumber = (*deviceExtension).SystemIoBusNumber;
    let mut VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
    let DeviceID: ULONG = ((*deviceExtension).DevID >> 16) & 0xffff;
    let RevID: ULONG = (*deviceExtension).RevID;
    let ChipType: ULONG = (*deviceExtension).HwFlags & CHIPTYPE_MASK;
    let mut ChipFlags: ULONG = (*deviceExtension).HwFlags & CHIPFLAG_MASK;
    let mut chan: PHW_CHANNEL;
    let mut tmp8: UCHAR = 0;
    let mut tmp16: USHORT = 0;
    let mut tmp32: ULONG;
    let mut c: ULONG; // logical channel
    let mut CheckCable: BOOLEAN = FALSE;
    let mut GlobalInit: BOOLEAN = FALSE;

    match channel {
        CHAN_NOT_SPECIFIED_CHECK_CABLE => {
            CheckCable = TRUE;
            c = CHAN_NOT_SPECIFIED;
            GlobalInit = TRUE;
        }
        CHAN_NOT_SPECIFIED => {
            c = CHAN_NOT_SPECIFIED;
            GlobalInit = TRUE;
        }
        _ => {
            c = channel;
            channel += (*deviceExtension).Channel;
        }
    }

    if (*deviceExtension).UnknownDev != 0 {
        VendorID = 0xffffffff;
    }

    if (ChipFlags & UNIATA_AHCI) != 0 {
        // if BAR(5) is IO it should point to SATA interface registers
        if *(*deviceExtension).BaseIoAHCI_0.__bindgen_anon_1.Addr.as_ref() == 0 {
            return FALSE;
        }
        if c == CHAN_NOT_SPECIFIED {
            return UniataAhciInit(HwDeviceExtension);
        } else if c < (*deviceExtension).NumberChannels {
            UniataAhciReset(HwDeviceExtension, c);
            return TRUE;
        } else {
            return FALSE;
        }
    }

    if WinVer_Id() > WinVer_NT && GlobalInit != 0 && (*deviceExtension).MasterDev != 0 {
        let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
        let busDataRead = HalGetBusData(
            PCIConfiguration,
            SystemIoBusNumber,
            slotNumber,
            &mut pciData as *mut _ as PVOID,
            PCI_COMMON_HDR_LENGTH,
        );
        if busDataRead == PCI_COMMON_HDR_LENGTH {
            UniataEnableIoPCI(SystemIoBusNumber, slotNumber, &mut pciData);
        }
    }

    match VendorID {
        ATA_ACER_LABS_ID => {
            if (ChipFlags & UNIATA_SATA) != 0 {
                if c == CHAN_NOT_SPECIFIED {
                    c = 0;
                    while c < (*deviceExtension).NumberChannels {
                        chan = (*deviceExtension).chan.add(c as usize);
                        (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                        if RevID == 0xc2 || RevID == 0xc3 {
                            AtapiAliSouthBridgeFixup(HwDeviceExtension, PCIConfiguration,
                                SystemIoBusNumber, slotNumber, c);
                        }
                        c += 1;
                    }
                    // enable PCI interrupt: ChangePciConfig2(Command, (a & ~0x0400))
                    change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        PCI_CFG_Command, |a| a & !0x0400);
                }
            } else if (ChipFlags & ALINEW) != 0 {
                if c == CHAN_NOT_SPECIFIED {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x4a, |a| a | 0x20);
                    if RevID < 0xc7 {
                        change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x4b, |a| a | 0x09);
                    }
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x53, |a| a | (if RevID >= 0xc7 { 0x03 } else { 0x01 }));
                } else {
                    generic_cable80(deviceExtension, channel, 0x4a, 0);
                }
            } else {
                if c == CHAN_NOT_SPECIFIED {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x53, |a| a | 0x03);
                } else {
                    (*(*deviceExtension).chan.add(c as usize)).ChannelCtrlFlags |=
                        CTRFLAGS_DMA_RO;
                }
            }
        }
        ATA_AMD_ID => 'amd: {
            if c == CHAN_NOT_SPECIFIED {
                if (ChipFlags & AMDBUG) != 0 {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x41, |a| a & 0x0f);
                } else {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x41, |a| a | 0xf0);
                }
            }
            if (*deviceExtension).MaxTransferMode < ATA_UDMA2 {
                break 'amd;
            }
            if (ChipFlags & UNIATA_NO80CHK) == 0 {
                if c == CHAN_NOT_SPECIFIED {
                    // do nothing
                } else {
                    generic_cable80(deviceExtension, channel, 0x42, 0);
                }
            }
        }
        ATA_HIGHPOINT_ID => {
            if c == CHAN_NOT_SPECIFIED {
                if (ChipFlags & HPTOLD) != 0 {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x51, |a| a & !0x80);
                } else {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x51, |a| a & !0x03);
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x55, |a| a & !0x03);
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x5a, |a| a & !0x10);
                    if ChipType < HPT372 {
                        let mut v: UCHAR = 0x22;
                        ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG,
                            SystemIoBusNumber, slotNumber, &mut v as *mut _ as PVOID, 0x5b, 1);
                    } else {
                        change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x5b, |a| (a & 0x01) | 0x20);
                    }
                }
            } else {
                chan = (*deviceExtension).chan.add(c as usize);
                if hpt_cable80(deviceExtension, channel) == 0 {
                    (*chan).MaxTransferMode =
                        core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
                }
            }
        }
        ATA_INTEL_ID => 'intel: {
            if (ChipFlags & UNIATA_SATA) != 0 {
                if (ChipFlags & UNIATA_AHCI) != 0 {
                    change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        PCI_CFG_Command, |a| a & !0x0400);
                    break 'intel;
                }
                if c == CHAN_NOT_SPECIFIED {
                    // force all ports active "the legacy way"
                    change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x92, |a| a | 0x0f);
                    if *(*deviceExtension).BaseIoAddressSATA_0.__bindgen_anon_1.Addr.as_ref() != 0
                        && (ChipFlags & ICH7) != 0
                    {
                        change_pci_config4(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x94, |a| a | (1 << 9));
                        let cur = AtapiReadPortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            0x0c);
                        AtapiWritePortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            0x0c, cur | 0x0f);
                    }
                    change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        PCI_CFG_Command, |a| a & !0x0400);
                } else {
                    // GetPciConfig1(0x90, tmp8)
                    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                        SystemIoBusNumber, slotNumber, &mut tmp8 as *mut _ as PVOID, 0x90, 1);
                    chan = (*deviceExtension).chan.add(c as usize);
                    let mut IsPata: BOOLEAN = FALSE;
                    if (ChipFlags & ICH5) != 0 {
                        if (tmp8 & 0x04) == 0 {
                            (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                            *(*(*chan).lun[0]).__bindgen_anon_1.SATA_lun_map.as_mut() =
                                ((tmp8 & 0x01) as ULONG) ^ c;
                            *(*(*chan).lun[1]).__bindgen_anon_1.SATA_lun_map.as_mut() = 0;
                        } else if (tmp8 & 0x02) == 0 {
                            if c == 0 {
                                *(*(*chan).lun[0]).__bindgen_anon_1.SATA_lun_map.as_mut() =
                                    if (tmp8 & 0x01) != 0 { 1 } else { 0 };
                                *(*(*chan).lun[1]).__bindgen_anon_1.SATA_lun_map.as_mut() =
                                    if (tmp8 & 0x01) != 0 { 0 } else { 1 };
                            } else {
                                IsPata = TRUE;
                            }
                        } else if (tmp8 & 0x02) != 0 {
                            if c == 1 {
                                *(*(*chan).lun[0]).__bindgen_anon_1.SATA_lun_map.as_mut() =
                                    if (tmp8 & 0x01) != 0 { 1 } else { 0 };
                                *(*(*chan).lun[1]).__bindgen_anon_1.SATA_lun_map.as_mut() =
                                    if (tmp8 & 0x01) != 0 { 0 } else { 1 };
                            } else {
                                IsPata = TRUE;
                            }
                        }
                    } else if (ChipFlags & I6CH2) != 0 {
                        (*chan).ChannelCtrlFlags |= CTRFLAGS_NO_SLAVE;
                        *(*(*chan).lun[0]).__bindgen_anon_1.SATA_lun_map.as_mut() =
                            if c != 0 { 0 } else { 1 };
                        *(*(*chan).lun[1]).__bindgen_anon_1.SATA_lun_map.as_mut() = 0;
                    } else {
                        match tmp8 & 0x03 {
                            0 => {
                                *(*(*chan).lun[0]).__bindgen_anon_1.SATA_lun_map.as_mut() = 0 + c;
                                *(*(*chan).lun[1]).__bindgen_anon_1.SATA_lun_map.as_mut() = 2 + c;
                            }
                            2 => {
                                if c == 0 {
                                    *(*(*chan).lun[0]).__bindgen_anon_1.SATA_lun_map.as_mut() = 0;
                                    *(*(*chan).lun[1]).__bindgen_anon_1.SATA_lun_map.as_mut() = 2;
                                } else {
                                    IsPata = TRUE;
                                }
                            }
                            1 => {
                                if c == 1 {
                                    *(*(*chan).lun[0]).__bindgen_anon_1.SATA_lun_map.as_mut() = 1;
                                    *(*(*chan).lun[1]).__bindgen_anon_1.SATA_lun_map.as_mut() = 3;
                                } else {
                                    IsPata = TRUE;
                                }
                            }
                            _ => {}
                        }
                    }

                    if IsPata != 0 {
                        (*chan).MaxTransferMode =
                            core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA5);
                    }

                    if ChipType == INTEL_IDX {
                        chan = (*deviceExtension).chan.add(c as usize);
                        UniataSataWritePort4(chan, IDX_SATA_SError as ULONG, 0xffffffff, 0);
                        if ((*chan).ChannelCtrlFlags & CTRFLAGS_NO_SLAVE) == 0 {
                            UniataSataWritePort4(chan, IDX_SATA_SError as ULONG, 0xffffffff, 1);
                        }
                    }
                }
                break 'intel;
            }
            if (*deviceExtension).MaxTransferMode <= ATA_UDMA2 {
                break 'intel;
            }
            if c == CHAN_NOT_SPECIFIED {
                // do nothing
            } else {
                chan = (*deviceExtension).chan.add(c as usize);
                let mut reg54: USHORT = 0;
                // GetPciConfig2(0x54, reg54)
                ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                    SystemIoBusNumber, slotNumber, &mut reg54 as *mut _ as PVOID, 0x54, 2);
                if ((*deviceExtension).HwFlags & UNIATA_NO80CHK) != 0 {
                    // No check (administrative)
                } else if reg54 == 0x0000 || reg54 == 0xffff {
                    // check failed (not supported)
                } else if ((reg54 >> (channel * 2)) & 30) == 0 {
                    (*chan).MaxTransferMode =
                        core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
                }
            }
        }
        ATA_NVIDIA_ID => {
            if (ChipFlags & UNIATA_SATA) != 0 {
                if c == CHAN_NOT_SPECIFIED {
                    let offs: ULONG = if (ChipFlags & NV4OFF) != 0 { 0x0440 } else { 0x0010 };
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x50, |a| a | 0x04);
                    // MCP55 needs some time
                    ScsiPortStallExecution(10);
                    if (ChipFlags & NVQ) != 0 {
                        tmp32 = AtapiReadPortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            0x0400);
                        if (tmp32 & !0xfffffff9) != 0 {
                            AtapiWritePortEx4(core::ptr::null_mut(),
                                &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _
                                    as ULONGIO_PTR,
                                0x0400, tmp32 & 0xfffffff9);
                        }
                        ChipFlags &= !NVQ;
                        (*deviceExtension).HwFlags = ChipFlags;
                    } else {
                        tmp32 = 0;
                    }
                    if (ChipFlags & NVQ) != 0 {
                        // (dead in practice — NVQ just cleared; preserved for fidelity)
                        change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x7f, |a| a & !(1 << 7));
                        AtapiWritePortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            0x0400, tmp32 | !0x00000006);
                        AtapiWritePortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            offs, 0x00ff00ff);
                        AtapiWritePortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            offs + 4, 0x000d000d);
                    } else {
                        AtapiWritePortEx1(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            offs, 0xff);
                        AtapiWritePortEx1(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            offs + 1, 0xdd);
                    }
                    change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        PCI_CFG_Command, |a| a & !0x0400);
                } else {
                    // UniataSataPhyEnable — commented out in C
                }
            } else {
                if c == CHAN_NOT_SPECIFIED {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x51, |a| a & 0x0f);
                } else {
                    generic_cable80(deviceExtension, channel, 0x52, 1);
                }
            }
        }
        ATA_PROMISE_ID => {
            match ChipType {
                PRNEW => {
                    if c == CHAN_NOT_SPECIFIED {
                        let cur = AtapiReadPortEx1(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressBM_0 as *mut _ as ULONGIO_PTR,
                            0x11);
                        AtapiWritePortEx1(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressBM_0 as *mut _ as ULONGIO_PTR,
                            0x11, cur | 0x0a);
                    }
                    promise_old_init(deviceExtension, c, channel);
                }
                PROLD => {
                    promise_old_init(deviceExtension, c, channel);
                }
                PRTX => {
                    if c == CHAN_NOT_SPECIFIED {
                        // do nothing
                    } else {
                        chan = (*deviceExtension).chan.add(c as usize);
                        AtapiWritePort1(chan, IDX_BM_DeviceSpecific0 as ULONG_PTR, 0x0b);
                        if (AtapiReadPort1(chan, IDX_BM_DeviceSpecific1 as ULONG_PTR) & 0x04) != 0 {
                            (*chan).MaxTransferMode =
                                core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
                        }
                    }
                }
                PRMIO => {
                    if c == CHAN_NOT_SPECIFIED {
                        // clear SATA status and unmask interrupts
                        AtapiWritePortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressBM_0 as *mut _ as ULONGIO_PTR,
                            if (ChipFlags & PRG2) != 0 { 0x60 } else { 0x6c }, 0x000000ff);
                        if (ChipFlags & UNIATA_SATA) != 0 {
                            let cur = AtapiReadPortEx4(core::ptr::null_mut(),
                                &mut (*deviceExtension).BaseIoAddressBM_0 as *mut _
                                    as ULONGIO_PTR, 0x44);
                            AtapiWritePortEx4(core::ptr::null_mut(),
                                &mut (*deviceExtension).BaseIoAddressBM_0 as *mut _
                                    as ULONGIO_PTR, 0x44, cur | 0x2000);
                        }
                    } else {
                        chan = (*deviceExtension).chan.add(c as usize);
                        let cur = AtapiReadPort4(chan, IDX_BM_Command as ULONG_PTR);
                        AtapiWritePort4(chan, IDX_BM_Command as ULONG_PTR,
                            (cur & !0x00000f8f) | channel);
                        AtapiWritePort4(chan, IDX_BM_DeviceSpecific0 as ULONG_PTR, 0x00000001);
                        if (*chan).MaxTransferMode < ATA_SA150
                            && (AtapiReadPort4(chan, IDX_BM_Command as ULONG_PTR) & 0x01000000) != 0
                        {
                            (*chan).MaxTransferMode =
                                core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
                        }
                    }
                }
                _ => {}
            }
        }
        ATA_SERVERWORKS_ID => {
            if c == CHAN_NOT_SPECIFIED {
                if ChipType == SWKS33 {
                    AtapiRosbSouthBridgeFixup(HwDeviceExtension, PCIConfiguration,
                        SystemIoBusNumber, slotNumber);
                } else {
                    let or = if ChipType == SWKS100 { 0x03 } else { 0x02 };
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x5a, |a| (a & !0x40) | or);
                }
            }
        }
        ATA_ATI_ID | ATA_SILICON_IMAGE_ID => 'siichip: {
            if VendorID == ATA_ATI_ID {
                if ChipType == SIIMIO {
                    // fall to SiI
                } else {
                    break 'siichip;
                }
            }
            // FALLTHROUGH (ATI new) / SILICON_IMAGE
            match ChipType {
                SIIMIO => {
                    if c == CHAN_NOT_SPECIFIED {
                        if (ChipFlags & SIISETCLK) != 0 {
                            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                                SystemIoBusNumber, slotNumber,
                                &mut tmp8 as *mut _ as PVOID, 0x8a, 1);
                            if (tmp8 & 0x30) != 0x10 {
                                change_pci_config1(HwDeviceExtension, SystemIoBusNumber,
                                    slotNumber, 0x8a, |a| (a & 0xcf) | 0x10);
                            }
                            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                                SystemIoBusNumber, slotNumber,
                                &mut tmp8 as *mut _ as PVOID, 0x8a, 1);
                            if (tmp8 & 0x30) != 0x10 {
                                (*deviceExtension).MaxTransferMode = core::cmp::min(
                                    (*deviceExtension).MaxTransferMode, ATA_UDMA5);
                            }
                        }
                    }
                    if (*deviceExtension).MaxTransferMode < ATA_SA150 {
                        if c == CHAN_NOT_SPECIFIED {
                            // do nothing
                        } else {
                            chan = (*deviceExtension).chan.add(c as usize);
                            let mut Reg79: USHORT = 0;
                            ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                                SystemIoBusNumber, slotNumber,
                                &mut Reg79 as *mut _ as PVOID, 0x79, 2);
                            if (Reg79 & (if channel != 0 { 0x02 } else { 0x01 })) != 0 {
                                (*chan).MaxTransferMode = core::cmp::min(
                                    (*deviceExtension).MaxTransferMode, ATA_UDMA2);
                            }
                        }
                    } else {
                        if c == CHAN_NOT_SPECIFIED {
                            c = 0;
                            while c < (*deviceExtension).NumberChannels {
                                let unit01 = (c & 1) as ULONG_PTR;
                                let unit10 = (c & 2) as ULONG_PTR;
                                if (ChipFlags & SIINOSATAIRQ) != 0 {
                                    AtapiWritePortEx4(core::ptr::null_mut(),
                                        &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _
                                            as ULONGIO_PTR,
                                        (0x148 + (unit01 << 7) + (unit10 << 8)) as ULONG, 0);
                                }
                                c += 1;
                            }
                        } else {
                            let unit01 = (c & 1) as ULONG_PTR;
                            let unit10 = (c & 2) as ULONG_PTR;
                            if (ChipFlags & SIINOSATAIRQ) != 0 {
                                AtapiWritePortEx4(core::ptr::null_mut(),
                                    &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _
                                        as ULONGIO_PTR,
                                    (0x148 + (unit01 << 7) + (unit10 << 8)) as ULONG, 0);
                            } else {
                                AtapiWritePortEx4(core::ptr::null_mut(),
                                    &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _
                                        as ULONGIO_PTR,
                                    (0x148 + (unit01 << 7) + (unit10 << 8)) as ULONG, 1 << 16);
                            }
                        }
                    }
                    if c == CHAN_NOT_SPECIFIED {
                        change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x8a, |a| a & 0x3f);
                        if (ChipFlags & SII4CH) != 0 {
                            AtapiWritePortEx4(core::ptr::null_mut(),
                                &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _
                                    as ULONGIO_PTR,
                                0x0200, 0x00000002);
                        }
                    } else {
                        chan = (*deviceExtension).chan.add(c as usize);
                        AtapiReadPortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            0x48);
                        AtapiWritePortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            0x48, (1u32 << 22) << c);
                        AtapiReadPortEx4(core::ptr::null_mut(),
                            &mut (*deviceExtension).BaseIoAddressSATA_0 as *mut _ as ULONGIO_PTR,
                            0x48);
                        // Initialize FIFO PCI bus arbitration
                        ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                            SystemIoBusNumber, slotNumber,
                            &mut tmp8 as *mut _ as PVOID, PCI_CFG_CacheLineSize, 1);
                        if tmp8 != 0 {
                            tmp8 = (tmp8 / 8) + 1;
                            AtapiWritePort2(chan, IDX_BM_DeviceSpecific1 as ULONG_PTR,
                                ((tmp8 as USHORT) << 8) | tmp8 as USHORT);
                        }
                    }
                }
                SIICMD => {
                    if c == CHAN_NOT_SPECIFIED {
                        set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x71, 0x01);
                        set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x53, 0x40);
                        set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x54, 0x3f);
                        set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x55, 0x40);
                        set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x56, 0x3f);
                        set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x57, 0x1c);
                        set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x58, 0x3f);
                        set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x5b, 0x3f);
                    }
                }
                ATI700 => {
                    if c == 0 && (ChipFlags & UNIATA_AHCI) == 0 {
                        chan = (*deviceExtension).chan.add(c as usize);
                        (*chan).MaxTransferMode =
                            core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA5);
                    }
                }
                _ => {}
            }
        }
        ATA_SIS_ID => {
            if c == CHAN_NOT_SPECIFIED {
                match ChipType {
                    SIS33 => {}
                    SIS66 | SIS100OLD => {
                        change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x52, |a| a & !0x04);
                    }
                    SIS100NEW | SIS133OLD => {
                        change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x49, |a| a & !0x01);
                    }
                    SIS133NEW => {
                        change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x50, |a| a | 0x0008);
                        change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x52, |a| a | 0x0008);
                    }
                    SISSATA => {
                        change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x04, |a| a & !0x0400);
                    }
                    _ => {}
                }
            }
            if ((*deviceExtension).HwFlags & UNIATA_SATA) != 0 {
                // do nothing for SATA
            } else if ChipType == SIS133NEW {
                if c == CHAN_NOT_SPECIFIED {
                    // do nothing
                } else {
                    chan = (*deviceExtension).chan.add(c as usize);
                    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                        SystemIoBusNumber, slotNumber, &mut tmp16 as *mut _ as PVOID,
                        if channel != 0 { 0x52 } else { 0x50 }, 2);
                    if (tmp16 & 0x8000) != 0 {
                        (*chan).MaxTransferMode =
                            core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
                    }
                }
            } else {
                if c == CHAN_NOT_SPECIFIED {
                    // do nothing
                } else {
                    chan = (*deviceExtension).chan.add(c as usize);
                    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                        SystemIoBusNumber, slotNumber, &mut tmp8 as *mut _ as PVOID, 48, 1);
                    if (tmp8 & (0x10 << channel)) != 0 {
                        (*chan).MaxTransferMode =
                            core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
                    }
                }
            }
        }
        ATA_VIA_ID => 'via: {
            if c == CHAN_NOT_SPECIFIED {
                if (ChipFlags & VIACLK) != 0 {
                    change_pci_config4(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x50, |a| a | 0x030b030b);
                }
                if (ChipFlags & (UNIATA_SATA | VIASATA)) != 0 {
                    change_pci_config2(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        PCI_CFG_Command, |a| a & !0x0400);
                    if DeviceID == 0x3149 || DeviceID == 0x3249 {
                        change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                            0x52, |a| a | (1 << 2));
                    }
                    break 'via;
                }
                if (ChipFlags & VIABUG) != 0 {
                    AtapiViaSouthBridgeFixup(HwDeviceExtension, PCIConfiguration,
                        SystemIoBusNumber, slotNumber);
                }
                if ChipType != VIA133 {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x41, |a| a | 0xf0);
                }
                let prq = if (ChipFlags & VIAPRQ) != 0 { 0x80u8 } else { 0x90u8 };
                change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                    0x43, |a| (a & prq) | 0x2a);
                change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                    0x44, |a| a | 0x08);
                change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                    0x46, |a| (a & 0x0c) | 0xf0);
                set_pci_word(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x60,
                    DEV_BSIZE as USHORT);
                set_pci_word(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x68,
                    DEV_BSIZE as USHORT);
            } else {
                chan = (*deviceExtension).chan.add(c as usize);
                if (ChipFlags & (UNIATA_SATA | VIASATA)) != 0 {
                    if (ChipFlags & VIABAR) != 0 && c >= 2 {
                        (*chan).MaxTransferMode = ATA_UDMA5;
                        break 'via;
                    }
                    UniataSataWritePort4(chan, IDX_SATA_SError as ULONG, 0xffffffff, 0);
                    break 'via;
                }
            }
        }
        ATA_ITE_ID => 'ite: {
            if ChipType == ITE_33 || ChipType == ITE_133_NEW {
                break 'ite;
            }
            if ChipType == ITE_133 {
                if c == CHAN_NOT_SPECIFIED {
                    change_pci_config1(HwDeviceExtension, SystemIoBusNumber, slotNumber,
                        0x50, |a| a & !0x83);
                    set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x54, 0x31);
                    set_pci_byte(HwDeviceExtension, SystemIoBusNumber, slotNumber, 0x56, 0x31);
                } else {
                    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                        SystemIoBusNumber, slotNumber, &mut tmp16 as *mut _ as PVOID, 0x40, 2);
                    chan = (*deviceExtension).chan.add(c as usize);
                    if (tmp16 & (if channel != 0 { 0x08 } else { 0x04 })) == 0 {
                        (*chan).MaxTransferMode =
                            core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
                    }
                }
            }
        }
        ATA_CYRIX_ID => {
            if ChipType == CYRIX_OLD {
                if c == CHAN_NOT_SPECIFIED {
                    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                        SystemIoBusNumber, slotNumber, &mut tmp8 as *mut _ as PVOID, 0x60, 1);
                    if (tmp8 & 0x40) == 0 {
                        tmp8 |= 0x40;
                        ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG,
                            SystemIoBusNumber, slotNumber, &mut tmp8 as *mut _ as PVOID, 0x60, 1);
                    }
                }
            }
        }
        ATA_JMICRON_ID => {
            if (*deviceExtension).DevID == ATA_JMB361
                || (*deviceExtension).DevID == ATA_JMB363
                || (*deviceExtension).DevID == ATA_JMB365
                || (*deviceExtension).DevID == ATA_JMB366
                || (*deviceExtension).DevID == ATA_JMB368
            {
                let mut c_swp: ULONG = 0;
                let mut reg40: ULONG = 0;
                let mut reg80: ULONG = 0;

                ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                    SystemIoBusNumber, slotNumber, &mut reg40 as *mut _ as PVOID, 0x40, 4);
                c_swp = if (reg40 & (1 << 22)) != 0 { 1 } else { 0 };
                ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration,
                    SystemIoBusNumber, slotNumber, &mut reg80 as *mut _ as PVOID, 0x80, 4);

                if c == CHAN_NOT_SPECIFIED {
                    let P1mode: UCHAR =
                        if (reg80 & (1 << 24)) != 0 { ATA_UDMA6 as UCHAR } else { ATA_SA300 as UCHAR };
                    if (reg40 & (1 << 23)) != 0 {
                        (*(*deviceExtension).chan.add((0 ^ c_swp) as usize)).MaxTransferMode =
                            P1mode as ULONG;
                        (*(*deviceExtension).chan.add((1 ^ c_swp) as usize)).MaxTransferMode =
                            ATA_UDMA6;
                        (*(*deviceExtension).chan.add((1 ^ c_swp) as usize)).ChannelCtrlFlags |=
                            CTRFLAGS_NO_SLAVE;
                    } else {
                        (*(*deviceExtension).chan.add((0 ^ c_swp) as usize)).MaxTransferMode =
                            P1mode as ULONG;
                        (*(*deviceExtension).chan.add((1 ^ c_swp) as usize)).MaxTransferMode =
                            ATA_SA300;
                        (*(*deviceExtension).chan.add((1 ^ c_swp) as usize)).ChannelCtrlFlags |=
                            CTRFLAGS_NO_SLAVE;
                    }
                } else {
                    chan = (*deviceExtension).chan.add(c as usize);
                    let ph_channel: UCHAR = (c ^ c_swp) as UCHAR;
                    if (*chan).MaxTransferMode >= ATA_SA150 {
                        // SATA
                    } else {
                        // PATA
                        if ph_channel == 0 {
                            if (reg40 & (1 << 5)) == 0 {
                                // disabled
                            } else if (reg40 & (1 << 3)) == 0 {
                                (*chan).MaxTransferMode = core::cmp::min(
                                    (*deviceExtension).MaxTransferMode, ATA_UDMA2);
                            }
                        } else {
                            if (reg80 & (1 << 21)) == 0 {
                                // disabled
                            } else if (reg80 & (1 << 19)) == 0 {
                                (*chan).MaxTransferMode = core::cmp::min(
                                    (*deviceExtension).MaxTransferMode, ATA_UDMA2);
                            }
                        }
                    }
                }
            }
        }
        _ => {
            if c != CHAN_NOT_SPECIFIED {
                // We can't check 80-pin cable on unknown controllers — leave the hint flag.
                (*deviceExtension).HwFlags |= UNIATA_NO80CHK;
            }
        }
    }

    // Separate channels are inited after common controller init. The only exception is
    // probe; there we may need 40/80-pin + MaxTransferRate info.
    if CheckCable != 0 && (ChipFlags & UNIATA_NO80CHK) == 0 {
        c = 0;
        while c < (*deviceExtension).NumberChannels {
            AtapiChipInit(HwDeviceExtension, DeviceNumber, c);
            c += 1;
        }
    }

    TRUE
}

// Shared PRNEW/PROLD tail (enable burst mode + 80-pin cable check).
#[inline]
unsafe fn promise_old_init(
    deviceExtension: PHW_DEVICE_EXTENSION,
    c: ULONG,
    channel: ULONG,
) {
    if c == CHAN_NOT_SPECIFIED {
        let cur = AtapiReadPortEx1(core::ptr::null_mut(),
            &mut (*deviceExtension).BaseIoAddressBM_0 as *mut _ as ULONGIO_PTR, 0x1f);
        AtapiWritePortEx1(core::ptr::null_mut(),
            &mut (*deviceExtension).BaseIoAddressBM_0 as *mut _ as ULONGIO_PTR, 0x1f, cur | 0x01);
    } else {
        let chan = (*deviceExtension).chan.add(c as usize);
        let HwDeviceExtension = deviceExtension as PVOID;
        let slotNumber = (*deviceExtension).slotNumber;
        let SystemIoBusNumber = (*deviceExtension).SystemIoBusNumber;
        let mut Reg50: USHORT = 0;
        ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
            slotNumber, &mut Reg50 as *mut _ as PVOID, 0x50, 2);
        if (Reg50 & (1 << (channel + 10))) != 0 {
            (*chan).MaxTransferMode =
                core::cmp::min((*deviceExtension).MaxTransferMode, ATA_UDMA2);
        }
    }
}

// ChangePciConfig2/4 + SetPciConfig1/2 helpers (inline macro expansions).
#[inline]
unsafe fn change_pci_config2(
    HwDeviceExtension: PVOID, SystemIoBusNumber: ULONG, slotNumber: ULONG,
    offs: ULONG, f: impl Fn(USHORT) -> USHORT,
) {
    let mut a: USHORT = 0;
    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
        slotNumber, &mut a as *mut _ as PVOID, offs, 2);
    a = f(a);
    ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
        slotNumber, &mut a as *mut _ as PVOID, offs, 2);
}
#[inline]
unsafe fn change_pci_config4(
    HwDeviceExtension: PVOID, SystemIoBusNumber: ULONG, slotNumber: ULONG,
    offs: ULONG, f: impl Fn(ULONG) -> ULONG,
) {
    let mut a: ULONG = 0;
    ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber,
        slotNumber, &mut a as *mut _ as PVOID, offs, 4);
    a = f(a);
    ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
        slotNumber, &mut a as *mut _ as PVOID, offs, 4);
}
#[inline]
unsafe fn set_pci_byte(
    HwDeviceExtension: PVOID, SystemIoBusNumber: ULONG, slotNumber: ULONG,
    offs: ULONG, val: UCHAR,
) {
    let mut v = val;
    ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
        slotNumber, &mut v as *mut _ as PVOID, offs, 1);
}
#[inline]
unsafe fn set_pci_word(
    HwDeviceExtension: PVOID, SystemIoBusNumber: ULONG, slotNumber: ULONG,
    offs: ULONG, val: USHORT,
) {
    let mut v = val;
    ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber,
        slotNumber, &mut v as *mut _ as PVOID, offs, 2);
}

// ===========================================================================
// UniataInitMapBM
// ===========================================================================
pub unsafe extern "system" fn UniataInitMapBM(
    deviceExtension: PHW_DEVICE_EXTENSION,
    mut BaseIoAddressBM_0: PIDE_BUSMASTER_REGISTERS,
    mut MemIo: BOOLEAN,
) {
    if BaseIoAddressBM_0.is_null() {
        MemIo = FALSE;
    }
    let mut c: ULONG = 0;
    while c < (*deviceExtension).NumberChannels {
        let chan = (*deviceExtension).chan.add(c as usize);
        let mut i: ULONG = 0;
        while i < IDX_BM_IO_SZ as ULONG {
            UniataInitIoRes(chan, IDX_BM_IO as ULONG + i,
                if !BaseIoAddressBM_0.is_null() {
                    (BaseIoAddressBM_0 as ULONGIO_PTR) + i as ULONGIO_PTR
                } else { 0 },
                MemIo, FALSE);
            i += 1;
        }
        if !BaseIoAddressBM_0.is_null() {
            BaseIoAddressBM_0 = BaseIoAddressBM_0.add(1);
        }
        c += 1;
    }
}

// ===========================================================================
// UniataInitMapBase
// ===========================================================================
pub unsafe extern "system" fn UniataInitMapBase(
    chan: PHW_CHANNEL,
    BaseIoAddress1: PIDE_REGISTERS_1,
    BaseIoAddress2: PIDE_REGISTERS_2,
) {
    let mut i: ULONG = 0;
    while i < IDX_IO1_SZ as ULONG {
        UniataInitIoRes(chan, IDX_IO1 as ULONG + i,
            if !BaseIoAddress1.is_null() {
                (BaseIoAddress1 as ULONGIO_PTR) + i as ULONGIO_PTR
            } else { 0 },
            FALSE, FALSE);
        i += 1;
    }
    i = 0;
    while i < IDX_IO2_SZ as ULONG {
        UniataInitIoRes(chan, IDX_IO2 as ULONG + i,
            if !BaseIoAddress2.is_null() {
                (BaseIoAddress2 as ULONGIO_PTR) + i as ULONGIO_PTR
            } else { 0 },
            FALSE, FALSE);
        i += 1;
    }
    UniataInitSyncBaseIO(chan);
}

// ===========================================================================
// UniataInitSyncBaseIO
// ===========================================================================
pub unsafe extern "system" fn UniataInitSyncBaseIO(chan: PHW_CHANNEL) {
    let elem = core::mem::size_of::<IORES>();
    let regs = core::ptr::addr_of_mut!((*chan).RegTranslation) as *mut IORES;
    RtlCopyMemory(
        regs.add(IDX_IO1_o as usize) as PVOID,
        regs.add(IDX_IO1 as usize) as PCVOID,
        IDX_IO1_SZ as usize * elem,
    );
    RtlCopyMemory(
        regs.add(IDX_IO2_o as usize) as PVOID,
        regs.add(IDX_IO2 as usize) as PCVOID,
        IDX_IO2_SZ as usize * elem,
    );
}

// ===========================================================================
// UniataInitIoRes
// ===========================================================================
pub unsafe extern "system" fn UniataInitIoRes(
    chan: PHW_CHANNEL,
    idx: ULONG,
    addr: ULONGIO_PTR,
    mut MemIo: BOOLEAN,
    mut Proc: BOOLEAN,
) {
    if addr == 0 {
        MemIo = FALSE;
        Proc = FALSE;
    }
    let r = &mut (*chan).RegTranslation[idx as usize];
    *r.__bindgen_anon_1.Addr.as_mut() = addr;
    r.set_MemIo(MemIo as ULONG);
    r.set_Proc(Proc as ULONG);
}

// ===========================================================================
// UniataInitIoResEx
// ===========================================================================
pub unsafe extern "system" fn UniataInitIoResEx(
    IoRes: PIORES,
    addr: ULONGIO_PTR,
    mut MemIo: BOOLEAN,
    mut Proc: BOOLEAN,
) {
    if addr == 0 {
        MemIo = FALSE;
        Proc = FALSE;
    }
    *(*IoRes).__bindgen_anon_1.Addr.as_mut() = addr;
    (*IoRes).set_MemIo(MemIo as ULONG);
    (*IoRes).set_Proc(Proc as ULONG);
}

// ===========================================================================
// AtapiSetupLunPtrs
// ===========================================================================
pub unsafe extern "system" fn AtapiSetupLunPtrs(
    chan: PHW_CHANNEL,
    deviceExtension: PHW_DEVICE_EXTENSION,
    c: ULONG,
) {
    if (*deviceExtension).NumberLuns == 0 {
        (*deviceExtension).NumberLuns = IDE_MAX_LUN_PER_CHAN;
    }
    (*chan).DeviceExtension = deviceExtension;
    (*chan).lChannel = c;
    (*chan).NumberLuns = (*deviceExtension).NumberLuns;
    let mut i: ULONG = 0;
    while i < (*deviceExtension).NumberLuns {
        (*chan).lun[i as usize] = (*deviceExtension)
            .lun
            .add((c * (*deviceExtension).NumberLuns + i) as usize);
        i += 1;
    }
    (*chan).AltRegMap = (*deviceExtension).AltRegMap;
    (*chan).NextDpcChan = 0xffffffff; // -1
    (*chan).last_devsel = 0xffffffff; // -1
    i = 0;
    while i < (*deviceExtension).NumberLuns {
        (*(*chan).lun[i as usize]).DeviceExtension = deviceExtension;
        (*(*chan).lun[i as usize]).chan = chan;
        (*(*chan).lun[i as usize]).Lun = i;
        i += 1;
    }
    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0
        && !(*deviceExtension).AhciInternalAtaReq0.is_null()
        && !(*deviceExtension).AhciInternalSrb0.is_null()
    {
        (*chan).AhciInternalAtaReq = (*deviceExtension).AhciInternalAtaReq0.add(c as usize);
        (*chan).AhciInternalSrb = (*deviceExtension).AhciInternalSrb0.add(c as usize);
        UniataAhciSetupCmdPtr((*chan).AhciInternalAtaReq);
        (*(*chan).AhciInternalSrb).SrbExtension = (*chan).AhciInternalAtaReq as PVOID;
        (*(*chan).AhciInternalAtaReq)
            .__bindgen_anon_1
            .as_mut()
            .__bindgen_anon_1
            .Srb = (*chan).AhciInternalSrb;
    }
}

// ===========================================================================
// UniataAllocateLunExt
// ===========================================================================
pub unsafe extern "system" fn UniataAllocateLunExt(
    deviceExtension: PHW_DEVICE_EXTENSION,
    NewNumberChannels: ULONG,
) -> BOOLEAN {
    let old_luns = (*deviceExtension).lun;
    let old_chans = (*deviceExtension).chan;

    if !old_luns.is_null() || !old_chans.is_null() {
        if NewNumberChannels == UNIATA_ALLOCATE_NEW_LUNS {
            return FALSE;
        }
    }

    if (*deviceExtension).NumberLuns == 0 {
        (*deviceExtension).NumberLuns = 2;
    }

    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        if (*deviceExtension).AhciInternalAtaReq0.is_null() {
            (*deviceExtension).AhciInternalAtaReq0 = ExAllocatePool(
                NonPagedPool,
                (core::mem::size_of::<ATA_REQ>() * (*deviceExtension).NumberChannels as usize)
                    as SIZE_T,
            ) as PATA_REQ;
            if (*deviceExtension).AhciInternalAtaReq0.is_null() {
                return FALSE;
            }
            RtlZeroMemory(
                (*deviceExtension).AhciInternalAtaReq0 as PVOID,
                core::mem::size_of::<ATA_REQ>() * (*deviceExtension).NumberChannels as usize,
            );
        }
        if (*deviceExtension).AhciInternalSrb0.is_null() {
            (*deviceExtension).AhciInternalSrb0 = ExAllocatePool(
                NonPagedPool,
                (core::mem::size_of::<SCSI_REQUEST_BLOCK>()
                    * (*deviceExtension).NumberChannels as usize) as SIZE_T,
            ) as PSCSI_REQUEST_BLOCK;
            if (*deviceExtension).AhciInternalSrb0.is_null() {
                UniataFreeLunExt(deviceExtension);
                return FALSE;
            }
            RtlZeroMemory(
                (*deviceExtension).AhciInternalSrb0 as PVOID,
                core::mem::size_of::<SCSI_REQUEST_BLOCK>()
                    * (*deviceExtension).NumberChannels as usize,
            );
        }
    }

    (*deviceExtension).lun = ExAllocatePool(
        NonPagedPool,
        (core::mem::size_of::<HW_LU_EXTENSION>()
            * ((*deviceExtension).NumberChannels + 1) as usize
            * (*deviceExtension).NumberLuns as usize) as SIZE_T,
    ) as PHW_LU_EXTENSION;
    if (*deviceExtension).lun.is_null() {
        UniataFreeLunExt(deviceExtension);
        return FALSE;
    }
    RtlZeroMemory(
        (*deviceExtension).lun as PVOID,
        core::mem::size_of::<HW_LU_EXTENSION>()
            * ((*deviceExtension).NumberChannels + 1) as usize
            * (*deviceExtension).NumberLuns as usize,
    );

    (*deviceExtension).chan = ExAllocatePool(
        NonPagedPool,
        (core::mem::size_of::<HW_CHANNEL>() * ((*deviceExtension).NumberChannels + 1) as usize)
            as SIZE_T,
    ) as PHW_CHANNEL;
    if (*deviceExtension).chan.is_null() {
        UniataFreeLunExt(deviceExtension);
        return FALSE;
    }
    RtlZeroMemory(
        (*deviceExtension).chan as PVOID,
        core::mem::size_of::<HW_CHANNEL>() * ((*deviceExtension).NumberChannels + 1) as usize,
    );
    TRUE
}

// ===========================================================================
// UniataFreeLunExt
// ===========================================================================
pub unsafe extern "system" fn UniataFreeLunExt(deviceExtension: PHW_DEVICE_EXTENSION) {
    if !(*deviceExtension).lun.is_null() {
        ExFreePool((*deviceExtension).lun as PVOID);
        (*deviceExtension).lun = core::ptr::null_mut();
    }
    if !(*deviceExtension).chan.is_null() {
        ExFreePool((*deviceExtension).chan as PVOID);
        (*deviceExtension).chan = core::ptr::null_mut();
    }
    if !(*deviceExtension).AhciInternalAtaReq0.is_null() {
        ExFreePool((*deviceExtension).AhciInternalAtaReq0 as PVOID);
        (*deviceExtension).AhciInternalAtaReq0 = core::ptr::null_mut();
    }
    if !(*deviceExtension).AhciInternalSrb0.is_null() {
        ExFreePool((*deviceExtension).AhciInternalSrb0 as PVOID);
        (*deviceExtension).AhciInternalSrb0 = core::ptr::null_mut();
    }
}

