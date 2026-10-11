//! id_dma.rs — Rust translation of drivers/storage/ide/uniata/id_dma.cpp
//! (UniATA busmaster-DMA setup + per-chipset transfer-timing programming).
//!
//! This is the DMA/PRD machinery: AtapiVirtToPhysAddr_, AtapiDmaAlloc/Setup/
//! Start/Done/Reinit, AtaSetTransferMode, the giant per-vendor AtapiDmaInit,
//! and the cyrix/promise/hpt/via82c timing helpers. USE_OWN_DMA is defined in
//! config.h, so AtapiVirtToPhysAddr(...) expands to AtapiVirtToPhysAddr_(...)
//! (5-arg), the AtapiDmaAlloc body is live, and AtapiDmaPioSync collapses to
//! `return TRUE` (its body is #ifndef USE_OWN_DMA).
//!
//! Translated per the fan-out contract (TRANSLATION_MANIFEST.md): every function
//! is `pub unsafe extern "system"`; PCI config macros expand inline capturing the
//! caller's HwDeviceExtension/SystemIoBusNumber/slotNumber; KdPrint dropped.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]
use crate::types::*;
use crate::regidx::*;
use crate::*;                      // sibling uniata functions + ScsiPortGetBusDataByOffset + CPU_num
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{TRUE, FALSE};
use ntkrnl_sys::rtl::RtlCopyMemory;
use crate::io_port::{
    AtapiReadPort1, AtapiWritePort1, AtapiWritePort4,
    AtapiWritePortEx4, AtapiWritePortEx1, AtapiReadPortEx1,
};

// ---------------------------------------------------------------------------
// Constants the foundation types crate doesn't carry (header #defines that
// bindgen dropped because they're macro-expression valued, not plain literals).
// ---------------------------------------------------------------------------
const PAGE_SIZE: ULONG = 0x1000;
// PAGE_MASK already in crate::types (= 4095).
const ATA_AHCI_DMA_ENTRIES: ULONG = 128;                  // PAGE_SIZE/2/sizeof(IDE_AHCI_PRD_ENTRY)
const AHCI_CLB_ALIGNEMENT_MASK: ULONGLONG = 1024 - 1;     // ((ULONGLONG)(1024-1))
const ATA_MAX_LBA28: LONGLONG = 0x0fffffff;               // DEF_U64(0x0fffffff)
// BUS_DATA_TYPE::PCIConfiguration enumerator (== 4); GetPciConfig macros pass this.
const PCIConfiguration: BUS_DATA_TYPE = 4;
// INTERFACE_TYPE::PCIBus enumerator (== 5).
const PCIBus: ULONG = 5;

// ---------------------------------------------------------------------------
// File-scope const tables (id_dma.cpp top-of-file, near line 50).
// ---------------------------------------------------------------------------
pub static valid_udma: [ULONG; 7] = [0, 0, 2, 0, 4, 5, 6];
pub static retry_Wdma: [CHAR; (MAX_RETRIES + 1) as usize] = [2, 2, 2, -1, -1, -1, -1];
pub static retry_Udma: [CHAR; (MAX_RETRIES + 1) as usize] = [6, 2, -1, -1, -1, -1, -1];

// PHYSICAL_ADDRESS ph4gb = {{0xFFFFFFFF, 0}};  (LowPart=0xFFFFFFFF, HighPart=0)
#[inline(always)]
unsafe fn make_ph4gb() -> PHYSICAL_ADDRESS {
    let mut p: PHYSICAL_ADDRESS = core::mem::zeroed();
    p.__bindgen_anon_1.LowPart = 0xFFFFFFFF;
    p.__bindgen_anon_1.HighPart = 0;
    p
}

// ---------------------------------------------------------------------------
// ATA_REQ field accessors — bindgen buries the real fields under nested anon
// unions. Mirrors id_ata_part3.rs' ata_req_core/ahci helpers (kept private so
// they don't collide with the sibling's via the flat re-export).
// ---------------------------------------------------------------------------
#[inline(always)]
unsafe fn dma_req_core(r: PATA_REQ) -> *mut _ATA_REQ__bindgen_ty_1__bindgen_ty_1 {
    &mut (*r).__bindgen_anon_1.as_mut().__bindgen_anon_1
}
// AtaReq->ata  (anon union: .ata.dma_base / .ata.dma_baseu)
#[inline(always)]
unsafe fn dma_req_ata(
    r: PATA_REQ,
) -> *mut _ATA_REQ__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1 {
    (*dma_req_core(r)).__bindgen_anon_1.ata.as_mut() as *mut _
}
// AtaReq->ahci  (anon union: .ahci.ahci_base64 / .ahci.ahci_cmd_ptr)
#[inline(always)]
unsafe fn dma_req_ahci(
    r: PATA_REQ,
) -> *mut _ATA_REQ__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1__bindgen_ty_2 {
    (*dma_req_core(r)).__bindgen_anon_1.ahci.as_mut() as *mut _
}
// AtaReq->dma_tab  (second sub-union of the request)
#[inline(always)]
unsafe fn dma_req_dma_tab(r: PATA_REQ) -> *mut BM_DMA_ENTRY {
    (*r).__bindgen_anon_1
        .as_mut()
        .__bindgen_anon_2
        .__bindgen_anon_1
        .dma_tab
        .as_mut()
        .as_mut_ptr()
}
// AtaReq->ahci_cmd0  (the embedded IDE_AHCI_CMD)
#[inline(always)]
unsafe fn dma_req_ahci_cmd0(r: PATA_REQ) -> *mut IDE_AHCI_CMD {
    (*r).__bindgen_anon_1
        .as_mut()
        .__bindgen_anon_2
        .__bindgen_anon_1
        .ahci_cmd0
        .as_mut() as *mut _
}

// ---------------------------------------------------------------------------
// Small inline header helpers used here that may not be translated elsewhere as
// re-exported siblings. Kept private (no `pub`) so they shadow any glob import
// and never collide on the flat re-export.
// ---------------------------------------------------------------------------
// ata_is_sata(ident): ident->SataCapabilities && != 0xffff
#[inline(always)]
pub unsafe fn ata_is_sata(ident: PIDENTIFY_DATA) -> BOOLEAN {
    let sc = (*ident).SataCapabilities;
    (sc != 0 && sc != 0xffff) as BOOLEAN
}
// UniataIsSATARangeAvailable(de, lChannel): RegTranslation[IDX_SATA_SStatus].Addr|Proc
#[inline(always)]
unsafe fn dma_is_sata_range_available(deviceExtension: PHW_DEVICE_EXTENSION, lChannel: ULONG) -> BOOLEAN {
    let chan = (*deviceExtension).chan.add(lChannel as usize);
    let ior = &(*chan).RegTranslation[IDX_SATA_SStatus as usize];
    if *ior.__bindgen_anon_1.Addr.as_ref() != 0 || ior.Proc() != 0 {
        TRUE as BOOLEAN
    } else {
        FALSE as BOOLEAN
    }
}

// WinVer_Id() == ((MajorVersion << 8) | MinorVersion) ; WinVer_NT == 0x0400.
const WinVer_NT: ULONG = 0x0400;
unsafe extern "C" {
    static MajorVersion: ULONG;
    static MinorVersion: ULONG;
}
#[inline(always)]
unsafe fn WinVer_Id() -> ULONG {
    (MajorVersion << 8) | MinorVersion
}

// C min()/max() on the integer types used here.
#[inline(always)]
fn min_u32(a: ULONG, b: ULONG) -> ULONG { if a < b { a } else { b } }
#[inline(always)]
fn min_i32(a: LONG, b: LONG) -> LONG { if a < b { a } else { b } }
// Generic signed-min so CHAR/SCHAR (both i8 but distinct nominal types) mix freely.
#[inline(always)]
fn min_c<T: Ord>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline(always)]
fn max_uc(a: UCHAR, b: UCHAR) -> UCHAR { if a > b { a } else { b } }

// ===========================================================================
//  AtapiVirtToPhysAddr_
// ===========================================================================
pub unsafe extern "system" fn AtapiVirtToPhysAddr_(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    data: PUCHAR,
    count: PULONG,
    ph_addru: PULONG,
) -> ULONG {
    let _ = HwDeviceExtension;
    let ph_addr: PHYSICAL_ADDRESS;
    let addr: ULONG;

    if MmIsAddressValid(data as PVOID) != 0 {
        // Touch memory, this prevents a not-ready page-table condition for valid
        // addresses. See ROS-11894 bug.
        core::ptr::read_volatile(data as *const u8);
    }
    ph_addr = MmGetPhysicalAddress(data as PVOID);
    let high = ph_addr.__bindgen_anon_1.HighPart;
    let low = ph_addr.__bindgen_anon_1.LowPart;

    if ph_addru.is_null() && high != 0 {
        // No 64-bit address support yet (or some workaround).
        *count = 0;
        return ULONG::MAX; // -1
    }

    *ph_addru = high as ULONG;
    addr = low;
    if addr == 0 && high == 0 {
        *count = 0;
        return 0;
    }
    if Srb.is_null() {
        *count = core::mem::size_of::<BM_DMA_ENTRY>() as ULONG * ATA_DMA_ENTRIES;
    } else {
        *count = PAGE_SIZE - (addr & (PAGE_SIZE - 1));
    }
    addr
} // end AtapiVirtToPhysAddr_()

// ===========================================================================
//  AtapiDmaAlloc  (USE_OWN_DMA body)
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaAlloc(
    HwDeviceExtension: PVOID,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    lChannel: ULONG,
) {
    let _ = ConfigInfo;
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let c = lChannel;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let mut i: ULONG = 0;
    let mut ph_addru: ULONG = 0;
    let ph4gb = make_ph4gb();

    (*(*deviceExtension).chan.add(c as usize)).CopyDmaBuffer = FALSE as BOOLEAN;

    if (*deviceExtension).Host64 == 0 && (WinVer_Id() > WinVer_NT) {
        if !(*chan).DB_PRD.is_null() {
            return;
        }
        (*chan).DB_PRD = MmAllocateContiguousMemory(
            core::mem::size_of::<[BM_DMA_ENTRY; 256]>() as SIZE_T, // sizeof(((PATA_REQ)NULL)->dma_tab)
            ph4gb,
        );
        if !(*chan).DB_PRD.is_null() {
            (*chan).DB_PRD_PhAddr = AtapiVirtToPhysAddr_(
                HwDeviceExtension,
                core::ptr::null_mut(),
                (*chan).DB_PRD as PUCHAR,
                &mut i,
                &mut ph_addru,
            );
            if (*chan).DB_PRD_PhAddr == 0 || i == 0 || ((*chan).DB_PRD_PhAddr as LONG) == -1 {
                (*chan).DB_PRD = core::ptr::null_mut();
                (*chan).DB_PRD_PhAddr = 0;
                return;
            }
            if ph_addru != 0 {
                // goto err_1
                MmFreeContiguousMemory((*chan).DB_PRD);
                (*chan).DB_PRD = core::ptr::null_mut();
                (*chan).DB_PRD_PhAddr = 0;
                (*chan).DB_IO = core::ptr::null_mut();
                (*chan).DB_IO_PhAddr = 0;
                return;
            }
        }
        (*chan).DB_IO = MmAllocateContiguousMemory(
            (*deviceExtension).MaximumDmaTransferLength as SIZE_T,
            ph4gb,
        );
        if !(*chan).DB_IO.is_null() {
            (*chan).DB_IO_PhAddr = AtapiVirtToPhysAddr_(
                HwDeviceExtension,
                core::ptr::null_mut(),
                (*chan).DB_IO as PUCHAR,
                &mut i,
                &mut ph_addru,
            );
            if (*chan).DB_IO_PhAddr == 0 || i == 0 || ((*chan).DB_IO_PhAddr as LONG) == -1 {
                // err_1:
                MmFreeContiguousMemory((*chan).DB_PRD);
                (*chan).DB_PRD = core::ptr::null_mut();
                (*chan).DB_PRD_PhAddr = 0;
                (*chan).DB_IO = core::ptr::null_mut();
                (*chan).DB_IO_PhAddr = 0;
                return;
            }
            if ph_addru != 0 {
                MmFreeContiguousMemory((*chan).DB_IO);
                // err_1:
                MmFreeContiguousMemory((*chan).DB_PRD);
                (*chan).DB_PRD = core::ptr::null_mut();
                (*chan).DB_PRD_PhAddr = 0;
                (*chan).DB_IO = core::ptr::null_mut();
                (*chan).DB_IO_PhAddr = 0;
                return;
            }
        }
    }

    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        if !(*chan).AhciCtlBlock.is_null() {
            return;
        }
        // Need 1K-byte alignment.
        (*chan).AhciCtlBlock0 = MmAllocateContiguousMemory(
            (core::mem::size_of::<IDE_AHCI_CHANNEL_CTL_BLOCK>() as ULONGLONG
                + AHCI_CLB_ALIGNEMENT_MASK) as SIZE_T,
            ph4gb,
        ) as PIDE_AHCI_CHANNEL_CTL_BLOCK;
        if !(*chan).AhciCtlBlock0.is_null() {
            let mut AhciCtlBlock64: ULONGLONG = (*chan).AhciCtlBlock0 as usize as ULONGLONG;

            AhciCtlBlock64 += AHCI_CLB_ALIGNEMENT_MASK;
            AhciCtlBlock64 &= !AHCI_CLB_ALIGNEMENT_MASK;

            (*chan).AhciCtlBlock = AhciCtlBlock64 as usize as PIDE_AHCI_CHANNEL_CTL_BLOCK;

            (*chan).AHCI_CTL_PhAddr = AtapiVirtToPhysAddr_(
                HwDeviceExtension,
                core::ptr::null_mut(),
                (*chan).AhciCtlBlock as PUCHAR,
                &mut i,
                &mut ph_addru,
            ) as ULONGLONG;
            if (*chan).AHCI_CTL_PhAddr == 0
                || i == 0
                || ((*chan).AHCI_CTL_PhAddr as LONG) == -1
            {
                (*chan).AhciCtlBlock = core::ptr::null_mut();
                (*chan).AHCI_CTL_PhAddr = 0;
                return;
            }
            if ph_addru != 0 {
                MmFreeContiguousMemory((*chan).AhciCtlBlock0 as PVOID);
                (*chan).AhciCtlBlock = core::ptr::null_mut();
                (*chan).AHCI_CTL_PhAddr = 0;
                return;
            }
        }
    }
} // end AtapiDmaAlloc()

// ===========================================================================
//  AtapiDmaSetup
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaSetup(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Srb: PSCSI_REQUEST_BLOCK,
    mut data: PUCHAR,
    count_in: ULONG,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut dma_count: ULONG = 0;
    let mut dma_base: ULONG;
    let mut dma_baseu: ULONG = 0;
    let mut dma_count0: ULONG;
    let mut dma_base0: ULONG;
    let mut dma_baseu0: ULONG;
    let mut i: ULONG = 0;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    let mut use_DB_IO: BOOLEAN = FALSE as BOOLEAN;
    let use_AHCI: BOOLEAN = if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN };
    let orig_count: ULONG = count_in;
    let mut count: ULONG = count_in;
    let max_entries: ULONG = if use_AHCI != 0 { ATA_AHCI_DMA_ENTRIES } else { ATA_DMA_ENTRIES };
    let max_frag: ULONG = (*deviceExtension).DmaSegmentLength;
    let seg_align: ULONG = (*deviceExtension).DmaSegmentAlignmentMask;

    if (*dma_req_core(AtaReq)).dma_entries != 0 {
        (*dma_req_core(AtaReq)).Flags |= REQ_FLAG_DMA_OPERATION as UCHAR;
        return TRUE as BOOLEAN;
    }
    (*dma_req_ata(AtaReq)).dma_base = 0;
    (*dma_req_core(AtaReq)).Flags &= !(REQ_FLAG_DMA_OPERATION as UCHAR);

    // (TransferMode < ATA_DMA) check is informational-only in the C (commented return).
    if count == 0 {
        return FALSE as BOOLEAN;
    }
    if count > (*deviceExtension).MaximumDmaTransferLength {
        return FALSE as BOOLEAN;
    }
    if (data as ULONG_PTR) & ((*deviceExtension).AlignmentMask as ULONG_PTR) != 0 {
        return FALSE as BOOLEAN;
    }

    if use_AHCI != 0 {
        dma_base = AtapiVirtToPhysAddr_(
            HwDeviceExtension,
            core::ptr::null_mut(),
            (*dma_req_ahci(AtaReq)).ahci_cmd_ptr as PUCHAR,
            &mut i,
            &mut dma_baseu,
        );
        (*dma_req_ahci(AtaReq)).ahci_base64 = 0; // clear before setup
    } else {
        dma_base = AtapiVirtToPhysAddr_(
            HwDeviceExtension,
            core::ptr::null_mut(),
            dma_req_dma_tab(AtaReq) as PUCHAR,
            &mut i,
            &mut dma_baseu,
        );
    }
    if dma_baseu != 0 && i != 0 {
        if (*deviceExtension).Host64 == 0 {
            dma_base = (*chan).DB_PRD_PhAddr;
            (*dma_req_core(AtaReq)).Flags |= REQ_FLAG_DMA_DBUF_PRD as UCHAR;
        }
    } else if dma_base == 0 || i == 0 || (dma_base as LONG) == -1 {
        return FALSE as BOOLEAN;
    }
    if use_AHCI != 0 {
        // The command table's full address: AHCI_CL->cmd_table_phys is 64-bit.
        // Upstream stores only the low half (ata.dma_base aliases ahci_base64's
        // low dword), which loses a table above 4 GiB on a 64-bit HBA. A table
        // moved to DB_PRD sits below 4 GiB.
        let high = if (*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_DBUF_PRD != 0 { 0 } else { dma_baseu };
        (*dma_req_ahci(AtaReq)).ahci_base64 = (dma_base as ULONGLONG) | ((high as ULONGLONG) << 32);
    } else {
        (*dma_req_ata(AtaReq)).dma_base = dma_base; // aliased to ahci.ahci_base64
    }

    dma_base = AtapiVirtToPhysAddr_(HwDeviceExtension, Srb, data, &mut dma_count, &mut dma_baseu);
    if dma_baseu != 0 && dma_count != 0 {
        if (*deviceExtension).Host64 == 0 {
            // retry_DB_IO:
            use_DB_IO = TRUE as BOOLEAN;
            dma_base = (*chan).DB_IO_PhAddr;
            data = (*chan).DB_IO as PUCHAR;
            dma_baseu = 0; // DB_IO is below 4 GiB
        }
        // A 64-bit HBA takes the page as is: each PRD entry below carries its
        // high half. Upstream stored this DATA address in ahci_base64 here,
        // i.e. as the command table's address, and the HBA then fetched its
        // command from the data buffer (q35 with 6 GiB hung in the bus scan).
    } else if dma_count == 0 || (dma_base as LONG) == -1 {
        (*dma_req_ahci(AtaReq)).ahci_base64 = 0;
        return FALSE as BOOLEAN;
    }

    dma_count = min_u32(count, PAGE_SIZE - ((data as ULONG_PTR & PAGE_MASK as ULONG_PTR) as ULONG));
    data = data.add(dma_count as usize);
    count -= dma_count;
    i = 0;

    dma_count0 = dma_count;
    dma_base0 = dma_base;
    dma_baseu0 = dma_baseu;

    'retry_db_io: loop {
        while count != 0 {
            if dma_baseu0 == dma_baseu && dma_base0 + dma_count0 == dma_base && dma_count0 + dma_count <= max_frag {
                // 'i' should always be > 0 here; for BM we can't cross 64k boundary
                if dma_base & seg_align != 0 {
                    // ASSERT(i)
                    i -= 1;
                    dma_base = dma_base0;
                    dma_count += dma_count0;
                }
            }
            if use_AHCI != 0 {
                let prd = (*(*dma_req_ahci(AtaReq)).ahci_cmd_ptr).prd_tab.as_mut_ptr().add(i as usize);
                // .base (low32) + .baseu (high32) == .base64
                *(*prd).__bindgen_anon_1.base64.as_mut() =
                    (dma_base as ULONGLONG) | ((dma_baseu as ULONGLONG) << 32);
                (*prd).Reserved1 = 0;
                *(*prd).__bindgen_anon_2.DBC_ULONG.as_mut() = (dma_count - 1) & 0x3fffff;
            } else {
                let ent = dma_req_dma_tab(AtaReq).add(i as usize);
                (*ent).base = dma_base;
                (*ent).count = dma_count & 0xffff;
            }
            dma_count0 = dma_count;
            dma_base0 = dma_base;
            dma_baseu0 = dma_baseu;
            i += 1;
            if i >= max_entries {
                (*dma_req_ahci(AtaReq)).ahci_base64 = 0;
                return FALSE as BOOLEAN;
            }
            dma_base = AtapiVirtToPhysAddr_(HwDeviceExtension, Srb, data, &mut dma_count, &mut dma_baseu);
            if dma_baseu != 0 && dma_count != 0 {
                if (*deviceExtension).Host64 == 0 {
                    if use_DB_IO != 0 {
                        return FALSE as BOOLEAN;
                    }
                    count = orig_count;
                    // goto retry_DB_IO
                    use_DB_IO = TRUE as BOOLEAN;
                    dma_base = (*chan).DB_IO_PhAddr;
                    data = (*chan).DB_IO as PUCHAR;
                    dma_baseu = 0; // DB_IO is below 4 GiB
                    dma_count = min_u32(count, PAGE_SIZE - ((data as ULONG_PTR & PAGE_MASK as ULONG_PTR) as ULONG));
                    data = data.add(dma_count as usize);
                    count -= dma_count;
                    i = 0;
                    dma_count0 = dma_count;
                    dma_base0 = dma_base;
                    dma_baseu0 = dma_baseu;
                    continue 'retry_db_io;
                }
            } else if dma_count == 0 || dma_base == 0 || (dma_base as LONG) == -1 {
                (*dma_req_ahci(AtaReq)).ahci_base64 = 0;
                return FALSE as BOOLEAN;
            }

            dma_count = min_u32(count, PAGE_SIZE);
            data = data.add(min_u32(count, PAGE_SIZE) as usize);
            count -= min_u32(count, PAGE_SIZE);
        }
        break;
    }

    // set TERM
    if dma_baseu0 == dma_baseu && dma_base0 + dma_count0 == dma_base && dma_count0 + dma_count <= max_frag {
        if dma_base & seg_align != 0 {
            // ASSERT(i)
            i -= 1;
            dma_base = dma_base0;
            dma_count += dma_count0;
        }
    }
    if use_AHCI != 0 {
        let cmd_ptr = (*dma_req_ahci(AtaReq)).ahci_cmd_ptr;
        let prd_base = (*cmd_ptr).prd_tab.as_mut_ptr();
        let prd = prd_base.add(i as usize);
        *(*prd).__bindgen_anon_1.base64.as_mut() =
            (dma_base as ULONGLONG) | ((dma_baseu as ULONGLONG) << 32);
        (*prd).Reserved1 = 0;
        *(*prd).__bindgen_anon_2.DBC_ULONG.as_mut() = (dma_count - 1) & 0x3fffff;
        let _ = prd_base; // page-boundary diagnostic in C was KdPrint-only
    } else {
        let tab = dma_req_dma_tab(AtaReq);
        let ent = tab.add(i as usize);
        (*ent).base = dma_base;
        (*ent).count = (dma_count & 0xffff) | ATA_DMA_EOT;
        let _ = tab; // page-boundary diagnostic in C was KdPrint-only
    }
    (*dma_req_core(AtaReq)).dma_entries = i + 1;

    if use_DB_IO != 0 {
        (*dma_req_core(AtaReq)).Flags |= REQ_FLAG_DMA_DBUF as UCHAR;
    }
    (*dma_req_core(AtaReq)).Flags |= REQ_FLAG_DMA_OPERATION as UCHAR;

    TRUE as BOOLEAN
} // end AtapiDmaSetup()

// ===========================================================================
//  AtapiDmaPioSync  (body is #ifndef USE_OWN_DMA → just returns TRUE)
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaPioSync(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    data: PUCHAR,
    count: ULONG,
) -> BOOLEAN {
    let _ = (HwDeviceExtension, Srb, data, count);
    TRUE as BOOLEAN
} // end AtapiDmaPioSync()

// ===========================================================================
//  AtapiDmaDBSync
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaDBSync(
    chan: PHW_CHANNEL,
    Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    // Only when the transfer really ran as DMA: AtapiDmaSetup sets DMA_DBUF
    // with DMA_OPERATION, but the PIO fallbacks clear only DMA_OPERATION, and
    // a PIO transfer has already put its data in the SRB's buffer. Copying the
    // untouched bounce buffer over it returned zeros for every CD sector.
    if ((*Srb).SrbFlags & SRB_FLAGS_DATA_IN) != 0
        && ((*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_DBUF) != 0
        && ((*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_OPERATION) != 0
    {
        // DBUF (Read). Into the SRB's buffer, which DataTransferLength is
        // measured from. Upstream copies to AtaReq->DataBuffer, but by now
        // completion has advanced that past the transfer (the PIO progress
        // pointer), so the copy landed DataTransferLength bytes beyond the
        // buffer: the pool block after a 2 KB CD read on a 6 GiB amd64 boot.
        // Only buffers above 4 GiB on a 32-bit controller take this path, and
        // upstream marks it ASSERT(FALSE): an unexercised path.
        RtlCopyMemory(
            (*Srb).DataBuffer,
            (*chan).DB_IO as PVOID,
            (*Srb).DataTransferLength as usize,
        );
    }
    TRUE as BOOLEAN
} // end AtapiDmaDBSync()

// ===========================================================================
//  AtapiDmaDBPreSync
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaDBPreSync(
    HwDeviceExtension: PVOID,
    chan: PHW_CHANNEL,
    Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;

    if (*dma_req_ata(AtaReq)).dma_base == 0 {
        return FALSE as BOOLEAN;
    }
    if (*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_DBUF_PRD != 0 {
        // DBUF_PRD
        if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
            RtlCopyMemory(
                (*chan).DB_PRD,
                (*dma_req_ahci(AtaReq)).ahci_cmd_ptr as PVOID,
                core::mem::size_of::<IDE_AHCI_CMD>() as usize, // sizeof(AtaReq->ahci_cmd0)
            );
        } else {
            RtlCopyMemory(
                (*chan).DB_PRD,
                dma_req_dma_tab(AtaReq) as PVOID,
                core::mem::size_of::<[BM_DMA_ENTRY; 256]>() as usize, // sizeof(AtaReq->dma_tab)
            );
        }
    }
    if ((*Srb).SrbFlags & SRB_FLAGS_DATA_IN) == 0
        && ((*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_DBUF) != 0
        && ((*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_OPERATION) != 0
    {
        // DBUF (Write): from the SRB's buffer, as AtapiDmaDBSync reads into it.
        RtlCopyMemory(
            (*chan).DB_IO,
            (*Srb).DataBuffer,
            (*Srb).DataTransferLength as usize,
        );
    }
    TRUE as BOOLEAN
} // end AtapiDmaDBPreSync()

// ===========================================================================
//  AtapiDmaStart
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaStart(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Srb: PSCSI_REQUEST_BLOCK,
) {
    let _ = DeviceNumber;
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let AtaReq = (*Srb).SrbExtension as PATA_REQ;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);

    let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
    let ChipType: ULONG = (*deviceExtension).HwFlags & CHIPTYPE_MASK;

    if (*dma_req_ata(AtaReq)).dma_base == 0 {
        return;
    }

    // set flag
    (*chan).ChannelCtrlFlags |= CTRFLAGS_DMA_ACTIVE;

    match VendorID {
        x if x == ATA_PROMISE_ID => {
            if ChipType == PRNEW {
                let Channel = (*deviceExtension).Channel + lChannel;
                if (*chan).ChannelCtrlFlags & CTRFLAGS_LBA48 != 0 {
                    AtapiWritePortEx4(
                        chan,
                        (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR,
                        if Channel != 0 { 0x24 } else { 0x20 },
                        (if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 { 0x05000000 } else { 0x06000000 })
                            | ((*Srb).DataTransferLength >> 1),
                    );
                }
            }
        }
        _ => {}
    }

    // set pointer to Pointer Table
    AtapiWritePort4(chan, IDX_BM_PRD_Table as ULONG_PTR, (*dma_req_ata(AtaReq)).dma_base);
    // set transfer direction
    AtapiWritePort1(
        chan,
        IDX_BM_Command as ULONG_PTR,
        if (*Srb).SrbFlags & SRB_FLAGS_DATA_IN != 0 { BM_COMMAND_READ as UCHAR } else { BM_COMMAND_WRITE as UCHAR },
    );
    // clear Error & Intr bits (writing 1 clears) + set DMA capability bit
    AtapiWritePort1(
        chan,
        IDX_BM_Status as ULONG_PTR,
        AtapiReadPort1(chan, IDX_BM_Status as ULONG_PTR) | (BM_STATUS_INTR | BM_STATUS_ERR) as UCHAR,
    );
    // begin transaction
    AtapiWritePort1(
        chan,
        IDX_BM_Command as ULONG_PTR,
        AtapiReadPort1(chan, IDX_BM_Command as ULONG_PTR) | BM_COMMAND_START_STOP as UCHAR,
    );
} // end AtapiDmaStart()

// ===========================================================================
//  AtapiDmaDone
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaDone(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Srb: PSCSI_REQUEST_BLOCK,
) -> UCHAR {
    let _ = (DeviceNumber, Srb);
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let dma_status: UCHAR;

    let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
    let ChipType: ULONG = (*deviceExtension).HwFlags & CHIPTYPE_MASK;

    if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        return IDE_STATUS_WRONG as UCHAR;
    }

    match VendorID {
        x if x == ATA_PROMISE_ID => {
            if ChipType == PRNEW {
                let Channel = (*deviceExtension).Channel + lChannel;
                if (*chan).ChannelCtrlFlags & CTRFLAGS_LBA48 != 0 {
                    AtapiWritePortEx4(
                        chan,
                        (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR,
                        if Channel != 0 { 0x24 } else { 0x20 },
                        0,
                    );
                }
            }
        }
        _ => {}
    }

    // get status
    dma_status = AtapiReadPort1(chan, IDX_BM_Status as ULONG_PTR) & BM_STATUS_MASK as UCHAR;
    // end transaction
    AtapiWritePort1(
        chan,
        IDX_BM_Command as ULONG_PTR,
        AtapiReadPort1(chan, IDX_BM_Command as ULONG_PTR) & !(BM_COMMAND_START_STOP as UCHAR),
    );
    // clear interrupt and error status
    AtapiWritePort1(chan, IDX_BM_Status as ULONG_PTR, (BM_STATUS_ERR | BM_STATUS_INTR) as UCHAR);
    // clear flag
    (*chan).ChannelCtrlFlags &= !CTRFLAGS_DMA_ACTIVE;

    dma_status
} // end AtapiDmaDone()

// ===========================================================================
//  AtapiDmaReinit
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaReinit(
    deviceExtension: PHW_DEVICE_EXTENSION,
    LunExt: PHW_LU_EXTENSION,
    AtaReq: PATA_REQ,
) {
    let apiomode: SCHAR;

    if ((*deviceExtension).HwFlags & UNIATA_AHCI != 0)
        && ((*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE == 0)
    {
        // An AHCI disk has no transfer mode to lower, so the C did nothing on
        // an error retry. The link speed can be lowered instead: once the same
        // request has failed again (retry >= 1, i.e. not an occasional error),
        // cap the link one SATA generation down.
        if (*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_FORCE_DOWNRATE != 0
            && (*dma_req_core(AtaReq)).retry >= 1
        {
            UniataSataDowngradeLinkSpeed(
                deviceExtension as PVOID,
                (*(*LunExt).chan).lChannel,
                if (*(*LunExt).chan).ChannelCtrlFlags & CTRFLAGS_AHCI_PM != 0 { (*LunExt).Lun } else { 0 },
            );
        }
        return;
    }

    apiomode = AtaPioMode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as SCHAR;

    // Decision flags mirroring the C goto-graph:
    //   need_limit_pio, need_limit_lba48 then fall into limit_pio.
    let mut do_limit_pio = false;
    let mut do_limit_lba48 = false;

    if (*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_DMA_OPERATION == 0 {
        do_limit_pio = true;
    } else if (*deviceExtension).HwFlags & UNIATA_AHCI != 0 {
        if (*dma_req_ahci(AtaReq)).ahci_base64 == 0 {
            // fatal — goto exit
            return;
        }
    } else if (*dma_req_ata(AtaReq)).dma_base == 0 {
        do_limit_pio = true;
    }

    if !do_limit_pio {
        if ((*deviceExtension).HbaCtrlFlags & HBAFLAGS_DMA_DISABLED_LBA48 != 0)
            && ((*dma_req_core(AtaReq)).lba >= ATA_MAX_LBA28)
            && ((*LunExt).TransferMode as ULONG > ATA_PIO5)
        {
            // goto limit_lba48
            do_limit_lba48 = true;
        }
    }

    if do_limit_lba48 {
        (*LunExt).DeviceFlags |= REQ_FLAG_FORCE_DOWNRATE_LBA48;
        do_limit_pio = true;
    }

    if do_limit_pio {
        do_limit_pio_block(deviceExtension, LunExt, apiomode);
        return;
    }

    if (*dma_req_core(AtaReq)).Flags as ULONG & REQ_FLAG_FORCE_DOWNRATE != 0 {
        if (*dma_req_core(AtaReq)).lba >= ATA_MAX_LBA28 {
            // limit_lba48: -> limit_pio
            (*LunExt).DeviceFlags |= REQ_FLAG_FORCE_DOWNRATE_LBA48;
            do_limit_pio_block(deviceExtension, LunExt, apiomode);
        } else {
            // set MAX mode
            AtapiDmaInit(
                deviceExtension as PVOID,
                (*LunExt).Lun,
                (*(*LunExt).chan).lChannel,
                apiomode,
                min_c(
                    retry_Wdma[(*dma_req_core(AtaReq)).retry as usize],
                    AtaWmode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as CHAR,
                ),
                min_c(
                    retry_Udma[(*dma_req_core(AtaReq)).retry as usize],
                    AtaUmode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as CHAR,
                ),
            );
        }
    } else if ((*LunExt).LimitedTransferMode > (*LunExt).TransferMode)
        || ((*LunExt).DeviceFlags & DFLAGS_REINIT_DMA != 0)
        || (((*deviceExtension).HwFlags & UNIATA_CHAN_TIMINGS != 0)
            && ((*LunExt).Lun != (*(*LunExt).chan).last_cdev))
    {
        // restore IO mode
        AtapiDmaInit__(deviceExtension, LunExt);
    }
    // exit:
} // end AtapiDmaReinit()

// limit_pio: block shared by the goto targets in AtapiDmaReinit.
#[inline(always)]
unsafe fn do_limit_pio_block(
    deviceExtension: PHW_DEVICE_EXTENSION,
    LunExt: PHW_LU_EXTENSION,
    apiomode: SCHAR,
) {
    // do not make extra work if we already use PIO
    if ((*LunExt).TransferMode as ULONG > ATA_PIO5)
        && ((*LunExt).TransferMode as ULONG != ATA_PIO0 + apiomode as ULONG)
    {
        AtapiDmaInit(
            deviceExtension as PVOID,
            (*LunExt).Lun,
            (*(*LunExt).chan).lChannel,
            apiomode,
            -1,
            -1,
        );
    } else if (*LunExt).LimitedTransferMode < (*LunExt).TransferMode {
        AtapiDmaInit(
            deviceExtension as PVOID,
            (*LunExt).Lun,
            (*(*LunExt).chan).lChannel,
            ((*LunExt).LimitedTransferMode as LONG - ATA_PIO0 as LONG) as SCHAR,
            -1,
            -1,
        );
    }
}

// ===========================================================================
//  AtapiDmaInit__
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaInit__(
    deviceExtension: PHW_DEVICE_EXTENSION,
    LunExt: PHW_LU_EXTENSION,
) {
    let supports_dma = (*LunExt).IdentifyData.SupportDma() != 0
        || ((*LunExt).IdentifyData.__bindgen_anon_1.AtapiDMA.as_ref().DMASupport() != 0
            && ((*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0));
    if supports_dma {
        AtapiDmaInit(
            deviceExtension as PVOID,
            (*LunExt).Lun,
            (*(*LunExt).chan).lChannel,
            AtaPioMode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as SCHAR,
            AtaWmode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as SCHAR,
            AtaUmode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as SCHAR,
        );
    } else {
        AtapiDmaInit(
            deviceExtension as PVOID,
            (*LunExt).Lun,
            (*(*LunExt).chan).lChannel,
            AtaPioMode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as SCHAR,
            -1,
            -1,
        );
    }
} // end AtapiDmaInit__()

// ===========================================================================
//  AtaSetTransferMode
// ===========================================================================
pub unsafe extern "system" fn AtaSetTransferMode(
    deviceExtension: PHW_DEVICE_EXTENSION,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    LunExt: PHW_LU_EXTENSION,
    mut mode: ULONG,
) -> BOOLEAN {
    let mut statusByte: LONG;
    let apiomode: CHAR;
    let disable_intr: BOOLEAN =
        ((((*deviceExtension).HwFlags & UNIATA_AHCI) != 0) || (CPU_num > 1)) as BOOLEAN;

    if (*LunExt).DeviceFlags & DFLAGS_MANUAL_CHS != 0 {
        statusByte = if mode <= ATA_PIO2 { IDE_STATUS_IDLE as LONG } else { IDE_STATUS_ERROR as LONG };
    } else {
        let umode: UCHAR;
        if disable_intr != 0 {
            AtapiDisableInterrupts(deviceExtension as PVOID, lChannel);
        }
        if mode > ATA_UDMA6 {
            // For SATA there's no option to set transfer rate; this just switches
            // between PIO and DMA. Devices may support only lower rates.
            let mut u = min_u32(AtaUmode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as ULONG, 6) as UCHAR;
            u = u.wrapping_add(ATA_UDMA0 as UCHAR);
            umode = u;
        } else {
            umode = mode as UCHAR;
        }
        statusByte = AtaCommand(
            deviceExtension,
            DeviceNumber,
            lChannel,
            IDE_COMMAND_SET_FEATURES as UCHAR,
            0,
            0,
            0,
            umode,
            ATA_C_F_SETXFER as UCHAR,
            ATA_WAIT_BASE_READY,
        ) as LONG;
        if disable_intr != 0 {
            AtapiEnableInterrupts(deviceExtension as PVOID, lChannel);
        }
    }
    if statusByte & IDE_STATUS_ERROR as LONG != 0 {
        if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 {
            ScsiPortStallExecution(10);
        } else {
            ScsiPortStallExecution(100);
        }
        apiomode = AtaPioMode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as CHAR;
        if (apiomode > 0)
            && (AtaWmode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as CHAR > 0)
            && (AtaUmode(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) as CHAR > 0)
        {
            return FALSE as BOOLEAN;
        }
        if mode > ATA_PIO2 {
            return FALSE as BOOLEAN;
        }
        mode = ATA_PIO0 + apiomode as ULONG;
    }
    // SATA sets actual transfer rate in LunExt on init; no run-time SATA adjust.
    (*LunExt).TransferMode = mode as UCHAR;
    if (*deviceExtension).HwFlags & UNIATA_SATA != 0 {
        if mode < ATA_SA150 {
            (*LunExt).PhyTransferMode = max_uc((*LunExt).PhyTransferMode, (*LunExt).TransferMode);
        } else {
            (*LunExt).PhyTransferMode = (*LunExt).TransferMode;
        }
    } else if (*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE != 0 {
        (*LunExt).PhyTransferMode = max_uc((*LunExt).LimitedTransferMode, (*LunExt).TransferMode);
    } else {
        (*LunExt).PhyTransferMode = (*LunExt).TransferMode;
    }
    TRUE as BOOLEAN
} // end AtaSetTransferMode()

// ===========================================================================
//  AtapiDmaInit  — the big per-vendor mode/timing programmer.
// ===========================================================================
pub unsafe extern "system" fn AtapiDmaInit(
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    mut apiomode: SCHAR,
    mut wdmamode: SCHAR,
    mut udmamode: SCHAR,
) {
    let deviceExtension = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let Channel: ULONG = (*deviceExtension).Channel + lChannel;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let dev: ULONG = Channel * 2 + DeviceNumber; // for non-SATA/AHCI only!
    let isAtapi: BOOLEAN =
        (((*(*chan).lun[DeviceNumber as usize]).DeviceFlags & DFLAGS_ATAPI_DEVICE) != 0) as BOOLEAN;
    let slotNumber: ULONG = (*deviceExtension).slotNumber;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    let mut i: LONG;
    let LunExt: PHW_LU_EXTENSION = (*chan).lun[DeviceNumber as usize];
    let mut ModeByte: UCHAR;

    let VendorID: ULONG = (*deviceExtension).DevID & 0xffff;
    let ChipType: ULONG = (*deviceExtension).HwFlags & CHIPTYPE_MASK;
    let ChipFlags: ULONG = (*deviceExtension).HwFlags & CHIPFLAG_MASK;

    let mut statusByte: LONG;

    (*LunExt).DeviceFlags &= !DFLAGS_REINIT_DMA;
    // set our most pessimistic default mode
    (*LunExt).TransferMode = ATA_PIO as UCHAR;
    if (*deviceExtension).BusMaster == 0 {
        udmamode = -1;
        wdmamode = -1;
    }

    // Limit transfer mode (controller limitation)
    if (*chan).MaxTransferMode as LONG >= ATA_UDMA as LONG {
        udmamode = min_c(udmamode, ((*chan).MaxTransferMode - ATA_UDMA) as CHAR);
    } else if (*chan).MaxTransferMode as LONG >= ATA_WDMA as LONG {
        udmamode = -1;
        wdmamode = min_c(wdmamode, ((*chan).MaxTransferMode - ATA_WDMA) as CHAR);
    } else if (*chan).MaxTransferMode as LONG >= ATA_PIO0 as LONG {
        wdmamode = -1;
        udmamode = -1;
        apiomode = min_c(apiomode, ((*chan).MaxTransferMode - ATA_PIO0) as CHAR);
    } else {
        wdmamode = -1;
        udmamode = -1;
        apiomode = 0;
    }
    // Limit transfer mode (device limitation)
    if (*LunExt).LimitedTransferMode as LONG >= ATA_UDMA as LONG {
        udmamode = min_c(udmamode, ((*LunExt).LimitedTransferMode as ULONG - ATA_UDMA) as CHAR);
    } else if (*LunExt).LimitedTransferMode as LONG >= ATA_WDMA as LONG {
        udmamode = -1;
        wdmamode = min_c(wdmamode, ((*LunExt).LimitedTransferMode as ULONG - ATA_WDMA) as CHAR);
    } else if (*LunExt).LimitedTransferMode as LONG >= ATA_PIO0 as LONG {
        wdmamode = -1;
        udmamode = -1;
        apiomode = min_c(apiomode, ((*LunExt).LimitedTransferMode as ULONG - ATA_PIO0) as CHAR);
    } else {
        wdmamode = -1;
        udmamode = -1;
        apiomode = 0;
    }

    // this is necessary for future PM support
    SelectDrive(chan, DeviceNumber);
    statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus as ULONG_PTR) as LONG; // GetStatus(chan, statusByte)
    // we can see IDE_STATUS_ERROR here after a previous operation
    if statusByte
        != 0
        && UniataIsIdle(
            deviceExtension,
            (statusByte as ULONG & !IDE_STATUS_ERROR) as UCHAR,
        ) as ULONG
            != IDE_STATUS_IDLE
    {
        return;
    }

    (*chan).last_cdev = DeviceNumber;
    // ServerWorks K2/Frodo/HT1000 SATA cannot DMA to ATAPI devices (upstream
    // svw_sata.c HW_FLAGS_NO_ATAPI_DMA on every part UniATA lists; only the
    // HT1100, which it does not list, can). UniATA never checked.
    if VendorID == ATA_SERVERWORKS_ID && ChipType == SWKSMIO && isAtapi != 0 {
        AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt,
            ATA_PIO0 + core::cmp::max(apiomode, 0) as ULONG);
        return;
    }
    if dma_is_sata_range_available(deviceExtension, lChannel) != 0
        || (ChipFlags & UNIATA_AHCI != 0)
        || ((*chan).MaxTransferMode >= ATA_SA150)
    {
        /* SATA Generic */
        if (udmamode >= 5)
            || (ChipFlags & UNIATA_AHCI != 0)
            || ((udmamode >= 0) && ((*chan).MaxTransferMode >= ATA_SA150))
        {
            if ata_is_sata(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) != 0 {
                if AtaSetTransferMode(
                    deviceExtension,
                    DeviceNumber,
                    lChannel,
                    LunExt,
                    min_u32((*LunExt).LimitedTransferMode as ULONG, (*LunExt).OrigTransferMode as ULONG),
                ) != 0
                {
                    return;
                }
                udmamode = min_c(udmamode, 6);
            } else {
                if udmamode > 2
                    && ((*LunExt).IdentifyData.HwResCableId() == 0
                        && ((*LunExt).IdentifyData.HwResValid() == IDENTIFY_CABLE_ID_VALID as USHORT))
                {
                    udmamode = 2;
                    apiomode = min_c(apiomode, ((*LunExt).LimitedTransferMode as ULONG - ATA_PIO0) as CHAR);
                } else {
                    udmamode = min_c(udmamode, 6);
                }
            }
        }
        if udmamode >= 0 {
            ModeByte = (ATA_UDMA0 as CHAR + udmamode) as UCHAR;
        } else if wdmamode >= 0 {
            ModeByte = (ATA_WDMA0 as CHAR + wdmamode) as UCHAR;
        } else if apiomode >= 0 {
            ModeByte = (ATA_PIO0 as CHAR + apiomode) as UCHAR;
        } else {
            ModeByte = ATA_PIO as UCHAR;
        }

        AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, ModeByte as ULONG);
        return;
    }

    let mut go_generic = false;
    if (*deviceExtension).UnknownDev != 0 {
        go_generic = true;
    }

    if !go_generic {
        if udmamode > 2
            && ((*LunExt).IdentifyData.HwResCableId() == 0
                && ((*LunExt).IdentifyData.HwResValid() == IDENTIFY_CABLE_ID_VALID as USHORT))
        {
            if ata_is_sata(&mut (*LunExt).IdentifyData as PIDENTIFY_DATA) != 0 {
                // SATA beyond adapter or controller compat mode
            } else {
                udmamode = 2;
                apiomode = min_c(apiomode, ((*LunExt).LimitedTransferMode as ULONG - ATA_PIO) as CHAR);
            }
        }

        // ---- the giant per-vendor switch. Returns out of the function in most
        //      arms; `break`/fallthrough to try_generic_dma maps to setting
        //      go_generic and dropping out. ----
        if dma_init_vendor(
            deviceExtension, HwDeviceExtension, DeviceNumber, lChannel, Channel, dev,
            chan, LunExt, isAtapi, slotNumber, SystemIoBusNumber,
            VendorID, ChipType, ChipFlags, &mut apiomode, &mut wdmamode, &mut udmamode,
        ) {
            return;
        }
    }

    // try_generic_dma:
    // unknown controller chip — better not try generic DMA on ATAPI devices
    if isAtapi != 0 {
        udmamode = -1;
        wdmamode = -1;
    }

    // if controller says it's setup for DMA take the easy way out
    if (udmamode >= 0 || wdmamode >= 0)
        && ((*deviceExtension).BusMaster as ULONG == DMA_MODE_BM)
        && ({
            // GetDmaStatus(deviceExtension, lChannel)
            let st = if (*deviceExtension).BusMaster as ULONG == DMA_MODE_BM {
                AtapiReadPort1((*deviceExtension).chan.add(lChannel as usize), IDX_BM_Status as ULONG_PTR)
            } else {
                0
            };
            (st as ULONG
                & (if DeviceNumber & 1 == 0 { BM_STATUS_DRIVE_0_DMA } else { BM_STATUS_DRIVE_1_DMA }))
                != 0
        })
    {
        if AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, ATA_DMA) != 0 {
            return;
        }
    }

    if AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, ATA_PIO0 + apiomode as ULONG) == 0 {
        (*LunExt).TransferMode = ATA_PIO as UCHAR;
    }
    let _ = i; // i is used inside dma_init_vendor's loops, declared here in C scope
} // end AtapiDmaInit()

// The per-vendor switch body of AtapiDmaInit, factored out. Returns `true` when
// the C code would `return` (mode programmed), `false` when it would fall to
// try_generic_dma (the `break`/end-of-switch path).
unsafe fn dma_init_vendor(
    deviceExtension: PHW_DEVICE_EXTENSION,
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Channel: ULONG,
    dev: ULONG,
    chan: PHW_CHANNEL,
    LunExt: PHW_LU_EXTENSION,
    isAtapi: BOOLEAN,
    slotNumber: ULONG,
    SystemIoBusNumber: ULONG,
    VendorID: ULONG,
    ChipType: ULONG,
    ChipFlags: ULONG,
    apiomode_io: &mut SCHAR,
    wdmamode_io: &mut SCHAR,
    udmamode_io: &mut SCHAR,
) -> bool {
    let mut apiomode = *apiomode_io;
    let mut wdmamode = *wdmamode_io;
    let mut udmamode = *udmamode_io;
    let mut i: LONG;

    // tiny PCI config helpers (inline GetPciConfig/SetPciConfig macro expansion).
    // They capture HwDeviceExtension/SystemIoBusNumber/slotNumber from this scope.
    macro_rules! get_pci {
        ($offs:expr, $var:expr, $sz:expr) => {{
            ScsiPortGetBusDataByOffset(
                HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber,
                &mut $var as *mut _ as PVOID, $offs as ULONG, $sz,
            );
        }};
    }
    macro_rules! set_pci {
        ($offs:expr, $val:expr, $ty:ty, $sz:expr) => {{
            let mut _a: $ty = $val as $ty;
            ScsiPortSetBusDataByOffset(
                HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber,
                &mut _a as *mut _ as PVOID, $offs as ULONG, $sz,
            );
        }};
    }

    // wrappers to keep the giant match readable
    let setmode = |m: ULONG| -> BOOLEAN {
        AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, m)
    };

    // restore on early `false`/`true` returns
    macro_rules! ret {
        ($v:expr) => {{
            *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode;
            return $v;
        }};
    }

    // Goto labels in the original Cyrix/National/ATI/SiliconImage arms are routed
    // with these flags.
    let mut goto_dma_cs55xx = false;
    let mut goto_sii = false;

    match VendorID {
        x if x == ATA_ACARD_ID => {
            let reg4a: USHORT = 0xa6;
            let reg: UCHAR = 0x40 + dev as UCHAR;
            if ChipType == ATPOLD {
                let reg4x2: USHORT = 0x0301;
                let mut done = false;
                i = udmamode as LONG;
                while i >= 0 {
                    if setmode(ATA_UDMA + i as ULONG) != 0 {
                        // set_old_acard:
                        let mut a: UCHAR = 0; get_pci!(0x54, a, 1);
                        set_pci!(0x54, a as ULONG | (0x01 << dev) | ((i as ULONG + 1) << (dev * 2)), UCHAR, 1);
                        set_pci!(0x4a, reg4a, UCHAR, 1);
                        set_pci!(reg, reg4x2, USHORT, 2);
                        ret!(true);
                    }
                    i -= 1;
                }
                if wdmamode >= 2 && apiomode >= 4 {
                    if setmode(ATA_WDMA2) != 0 {
                        let mut a: UCHAR = 0; get_pci!(0x54, a, 1);
                        set_pci!(0x54, a as ULONG | (0x01 << dev) | (((udmamode as ULONG)+1) << (dev*2)), UCHAR, 1);
                        set_pci!(0x4a, reg4a, UCHAR, 1);
                        set_pci!(reg, reg4x2, USHORT, 2);
                        ret!(true);
                    }
                }
                let _ = done;
            } else {
                let reg4x: UCHAR = 0x31;
                i = udmamode as LONG;
                while i >= 0 {
                    if setmode(ATA_UDMA + i as ULONG) != 0 {
                        // set_new_acard:
                        let mut a: USHORT = 0; get_pci!(0x44, a, 2);
                        set_pci!(0x44, (a as ULONG & !(0x000f << (dev*4))) | ((i as ULONG+1) << (dev*4)), USHORT, 2);
                        set_pci!(0x4a, reg4a, UCHAR, 1);
                        set_pci!(reg, reg4x, UCHAR, 1);
                        ret!(true);
                    }
                    i -= 1;
                }
                if wdmamode >= 2 && apiomode >= 4 {
                    if setmode(ATA_WDMA2) != 0 {
                        let mut a: USHORT = 0; get_pci!(0x44, a, 2);
                        set_pci!(0x44, (a as ULONG & !(0x000f << (dev*4))) | (((udmamode as ULONG)+1) << (dev*4)), USHORT, 2);
                        set_pci!(0x4a, reg4a, UCHAR, 1);
                        set_pci!(reg, reg4x, UCHAR, 1);
                        ret!(true);
                    }
                }
            }
            // Use GENERIC PIO (break)
        }
        x if x == ATA_ACER_LABS_ID => {
            let ali_udma: [UCHAR; 6] = [0x0c, 0x0b, 0x0a, 0x09, 0x08, 0x0f];
            let ali_pio: [ULONG; 6] =
                [0x006d0003, 0x00580002, 0x00440001, 0x00330001, 0x00310001, 0x00440001];
            // older Aladdin doesn't support ATAPI DMA on both master & slave
            let mut broke = false;
            if (ChipFlags & ALIOLD != 0) && (udmamode >= 0 || wdmamode >= 0) {
                let atapi0 = ((*(*chan).lun[0]).DeviceFlags & DFLAGS_ATAPI_DEVICE) != 0;
                let atapi1 = ((*(*chan).lun[1]).DeviceFlags & DFLAGS_ATAPI_DEVICE) != 0;
                if atapi0 && atapi1 {
                    (*chan).MaxTransferMode = min_u32((*chan).MaxTransferMode, ATA_PIO4);
                    udmamode = -1;
                    wdmamode = -1;
                    broke = true; // break out of switch -> generic
                }
            }
            if !broke {
                i = udmamode as LONG;
                while i >= 0 {
                    if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                        let mut word54: ULONG = 0; get_pci!(0x54, word54, 4);
                        word54 &= !(0x000f000f << (dev * 4));
                        word54 |= ((ali_udma[i as usize] as ULONG) << 16 | 5) << (dev * 4);
                        set_pci!(0x54, word54, ULONG, 4);
                        let mut a: UCHAR = 0; get_pci!(0x53, a, 1);
                        set_pci!(0x53, a as ULONG | 0x03, UCHAR, 1);
                        set_pci!(0x58 + (Channel << 2), 0x00310001u32, ULONG, 4);
                        ret!(true);
                    }
                    i -= 1;
                }
                // make sure eventual UDMA mode from the BIOS is disabled
                {
                    let mut a: USHORT = 0; get_pci!(0x56, a, 2);
                    set_pci!(0x56, a as ULONG & !(0x0008 << (dev * 4)), USHORT, 2);
                }
                if wdmamode >= 2 && apiomode >= 4 {
                    if setmode(ATA_WDMA2) != 0 {
                        let mut a: UCHAR = 0; get_pci!(0x53, a, 1);
                        set_pci!(0x53, a as ULONG | 0x03, UCHAR, 1);
                        (*chan).ChannelCtrlFlags |= CTRFLAGS_DMA_RO;
                        ret!(true);
                    }
                }
                {
                    let mut a: UCHAR = 0; get_pci!(0x53, a, 1);
                    set_pci!(0x53, (a as ULONG & !0x01) | 0x02, UCHAR, 1);
                }
                i = apiomode as LONG;
                while i >= 0 {
                    if setmode(ATA_PIO0 + i as ULONG) != 0 {
                        let mut a: ULONG = 0; get_pci!(0x54, a, 4);
                        set_pci!(0x54, a & !(0x0008000f << (dev * 4)), ULONG, 4);
                        set_pci!(0x58 + (Channel << 2), ali_pio[i as usize], ULONG, 4);
                        ret!(true);
                    }
                    i -= 1;
                }
                ret!(false);
            }
        }
        x if x == ATA_AMD_ID || x == ATA_NVIDIA_ID || x == ATA_VIA_ID => {
            if VendorID == ATA_AMD_ID {
                // Two AMD rules from upstream's chipset/amd.c that UniATA lacks.
                // 1. Read/write prefetch only on a channel with nothing but
                //    fixed disks: AtapiChipInit turns it on for both channels
                //    (reg 0x41 |= 0xf0) before any device is known, and amd.c
                //    clears a channel's bits (0xC0 >> 2*ch) once a non-disk
                //    device is on it. AMDBUG parts (766) already have it off.
                if isAtapi != 0 && (ChipFlags & AMDBUG) == 0 {
                    let mut a: UCHAR = 0; get_pci!(0x41, a, 1);
                    set_pci!(0x41, a & !(0xC0u8 >> (Channel * 2)), UCHAR, 1);
                }
                // 2. AMD-8111 on the Serenade board (subsystem 36C0h) cannot do
                //    UDMA6.
                if ((*deviceExtension).DevID >> 16) == 0x7469 {
                    let mut subsys: USHORT = 0; get_pci!(0x2E, subsys, 2);
                    if subsys == 0x36C0 {
                        udmamode = min_c(udmamode, 5);
                    }
                }
            }
            if VendorID == ATA_VIA_ID {
                if (ChipFlags & VIASATA != 0) && (Channel == 0) {
                    setmode(ATA_SA150);
                    ret!(true);
                }
                if (ChipFlags & VIABAR != 0) && (Channel < 2) {
                    setmode(ATA_SA150);
                    ret!(true);
                }
            }
            let via_modes: [[UCHAR; 7]; 6] = [
                [0xc2, 0xc1, 0xc0, 0x00, 0x00, 0x00, 0x00],
                [0xee, 0xec, 0xea, 0xe9, 0xe8, 0x00, 0x00],
                [0xf7, 0xf6, 0xf4, 0xf2, 0xf1, 0xf0, 0x00],
                [0xf7, 0xf7, 0xf6, 0xf4, 0xf2, 0xf1, 0xf0],
                [0xc2, 0xc1, 0xc0, 0xc4, 0xc5, 0xc6, 0xc7],
                [0xee, 0xe8, 0xe6, 0xe4, 0xe2, 0xe1, 0xe0],
            ];
            let via_pio: [UCHAR; 14] = [
                0xa8, 0x65, 0x42, 0x22, 0x20, 0x42, 0x22, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            ];
            let mut reg: UCHAR = 0x53u32.wrapping_sub(dev) as UCHAR;
            let mut reg2: UCHAR = reg.wrapping_sub(0x08);
            if ChipFlags & VIABAR != 0 {
                reg = 0xb3;
                reg2 = 0xab;
            }
            let reg_val = &via_modes[ChipType as usize];
            if VendorID == ATA_NVIDIA_ID {
                reg = reg.wrapping_add(0x10);
                reg2 = reg2.wrapping_add(0x10);
            }
            i = udmamode as LONG;
            while i >= 0 {
                if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                    set_pci!(reg2, via_pio[8 + i as usize], UCHAR, 1);
                    set_pci!(reg, reg_val[i as usize], UCHAR, 1);
                    ret!(true);
                }
                i -= 1;
            }
            if ChipFlags & VIABAR == 0 {
                i = wdmamode as LONG;
                while i >= 0 {
                    if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                        set_pci!(reg2, via_pio[5 + i as usize], UCHAR, 1);
                        set_pci!(reg, 0x8bu32, UCHAR, 1);
                        ret!(true);
                    }
                    i -= 1;
                }
            }
            if (apiomode >= 0) && (ChipType != VIA133) {
                set_pci!(reg2, via_pio[apiomode as usize], UCHAR, 1);
            }
            if VendorID == ATA_VIA_ID {
                via82c_timing(deviceExtension, dev, ATA_PIO0 as CHAR + apiomode);
            }
            setmode(ATA_PIO0 + apiomode as ULONG);
            ret!(true);
        }
        x if x == ATA_CYRIX_ID => {
            goto_dma_cs55xx = true; // fall into the cs55xx code below
        }
        x if x == ATA_NATIONAL_ID => {
            if ChipType == 0 {
                let nat_piotiming: [ULONG; 11] = [
                    0x9172d132, 0x21717121, 0x00803020, 0x20102010, 0x00100010, 0x00803020,
                    0x20102010, 0x00100010, 0x00100010, 0x00100010, 0x00100010,
                ];
                let nat_dmatiming: [ULONG; 3] = [0x80077771, 0x80012121, 0x80002020];
                let nat_udmatiming: [ULONG; 3] = [0x80921250, 0x80911140, 0x80911030];
                if apiomode >= 4 { apiomode = 4; }
                i = udmamode as LONG;
                while i >= 0 {
                    if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                        set_pci!(0x44 + (dev * 8), nat_udmatiming[i as usize], ULONG, 4);
                        set_pci!(0x40 + (dev * 8), nat_piotiming[i as usize + 8], ULONG, 4);
                        ret!(true);
                    }
                    i -= 1;
                }
                i = wdmamode as LONG;
                while i >= 0 {
                    if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                        set_pci!(0x44 + (dev * 8), nat_dmatiming[i as usize], ULONG, 4);
                        set_pci!(0x40 + (dev * 8), nat_piotiming[i as usize + 5], ULONG, 4);
                        ret!(true);
                    }
                    i -= 1;
                }
                if setmode(ATA_PIO0 + apiomode as ULONG) != 0 {
                    let mut a: ULONG = 0; get_pci!(0x44 + (dev * 8), a, 4);
                    set_pci!(0x44 + (dev * 8), a | 0x80000000, ULONG, 4);
                    set_pci!(0x40 + (dev * 8), nat_piotiming[apiomode as usize], ULONG, 4);
                    ret!(true);
                }
                // Use GENERIC PIO (break)
            } else {
                goto_dma_cs55xx = true;
            }
        }
        x if x == ATA_CYPRESS_ID => {
            if wdmamode >= 2 && apiomode >= 4 {
                if setmode(ATA_WDMA2) != 0 {
                    set_pci!(if Channel != 0 { 0x4e } else { 0x4c }, 0x2020u32, USHORT, 2);
                    ret!(true);
                }
            }
        }
        x if x == ATA_MARVELL_ID => {
            i = udmamode as LONG;
            while i >= 0 { if setmode(ATA_UDMA0 + i as ULONG) != 0 { ret!(true); } i -= 1; }
            i = wdmamode as LONG;
            while i >= 0 { if setmode(ATA_WDMA0 + i as ULONG) != 0 { ret!(true); } i -= 1; }
            if wdmamode >= 0 && apiomode >= 4 {
                if setmode(ATA_DMA) != 0 { ret!(true); }
            }
            setmode(ATA_PIO0 + apiomode as ULONG);
            ret!(true);
        }
        x if x == ATA_NETCELL_ID => {
            if wdmamode >= 2 && apiomode >= 4 {
                if setmode(ATA_WDMA2) != 0 { ret!(true); }
            }
        }
        x if x == ATA_HIGHPOINT_ID => {
            i = udmamode as LONG;
            while i >= 0 {
                if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                    hpt_timing(deviceExtension, dev, (ATA_UDMA as LONG + i) as CHAR);
                    ret!(true);
                }
                i -= 1;
            }
            i = wdmamode as LONG;
            while i >= 0 {
                if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                    hpt_timing(deviceExtension, dev, (ATA_WDMA0 as LONG + i) as CHAR);
                    ret!(true);
                }
                i -= 1;
            }
            if wdmamode >= 0 && apiomode >= 4 {
                if setmode(ATA_DMA) != 0 { ret!(true); }
            }
            setmode(ATA_PIO0 + apiomode as ULONG);
            hpt_timing(deviceExtension, dev, ATA_PIO0 as CHAR + apiomode);
            ret!(true);
        }
        x if x == ATA_INTEL_ID => {
            if dma_init_intel(
                deviceExtension, HwDeviceExtension, DeviceNumber, lChannel, Channel, dev,
                isAtapi, slotNumber, SystemIoBusNumber, ChipFlags,
                &mut apiomode, &mut wdmamode, &mut udmamode,
            ) {
                ret!(true);
            }
            ret!(true); // intel arm always returns in C
        }
        x if x == ATA_PROMISE_ID => {
            let sel66: UCHAR = if Channel != 0 { 0x08 } else { 0x02 };
            if ChipType < PRTX {
                if isAtapi != 0 { udmamode = -1; wdmamode = -1; }
            }
            i = udmamode as LONG;
            while i >= 0 {
                if ChipType == PRNEW {
                    if i > 2 {
                        let v = AtapiReadPortEx1(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x11);
                        AtapiWritePortEx1(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x11, v | sel66);
                    } else {
                        let v = AtapiReadPortEx1(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x11);
                        AtapiWritePortEx1(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x11, v & !sel66);
                    }
                }
                if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                    promise_timing(deviceExtension, dev, (ATA_UDMA as LONG + i) as CHAR);
                    ret!(true);
                }
                i -= 1;
            }
            if ChipType == PRNEW {
                let v = AtapiReadPortEx1(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x11);
                AtapiWritePortEx1(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, 0x11, v & !sel66);
            }
            i = wdmamode as LONG;
            while i >= 0 {
                if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                    promise_timing(deviceExtension, dev, (ATA_WDMA0 as LONG + i) as CHAR);
                    ret!(true);
                }
                i -= 1;
            }
            setmode(ATA_PIO0 + apiomode as ULONG);
            promise_timing(deviceExtension, dev, ATA_PIO0 as CHAR + apiomode);
            ret!(true);
        }
        x if x == ATA_ATI_ID => {
            if ChipType == SIIMIO {
                goto_sii = true;
            } else {
                // FALL THROUGH to ServerWorks
                if dma_init_serverworks(
                    deviceExtension, HwDeviceExtension, DeviceNumber, lChannel, Channel, dev,
                    slotNumber, SystemIoBusNumber, VendorID,
                    &mut apiomode, &mut wdmamode, &mut udmamode,
                ) { ret!(true); }
                ret!(true); // serverworks arm always returns
            }
        }
        x if x == ATA_SERVERWORKS_ID => {
            if dma_init_serverworks(
                deviceExtension, HwDeviceExtension, DeviceNumber, lChannel, Channel, dev,
                slotNumber, SystemIoBusNumber, VendorID,
                &mut apiomode, &mut wdmamode, &mut udmamode,
            ) { ret!(true); }
            ret!(true);
        }
        x if x == ATA_SILICON_IMAGE_ID => {
            goto_sii = true;
        }
        x if x == ATA_SIS_ID => {
            if dma_init_sis(
                deviceExtension, HwDeviceExtension, DeviceNumber, lChannel, dev,
                chan, LunExt, slotNumber, SystemIoBusNumber, ChipType,
                &mut apiomode, &mut wdmamode, &mut udmamode,
            ) { ret!(true); }
            ret!(true);
        }
        0x16ca => {
            // Cenatek Rocket Drive controller
            if wdmamode >= 0
                && (AtapiReadPort1(chan, IDX_BM_Status as ULONG_PTR)
                    & (if DeviceNumber != 0 { BM_STATUS_DRIVE_1_DMA } else { BM_STATUS_DRIVE_0_DMA }) as UCHAR
                    != 0)
            {
                setmode(ATA_WDMA0 + wdmamode as ULONG);
            } else {
                setmode(ATA_PIO0 + apiomode as ULONG);
            }
            ret!(true);
        }
        x if x == ATA_ITE_ID => {
            if dma_init_ite(
                deviceExtension, HwDeviceExtension, DeviceNumber, lChannel, Channel, dev,
                isAtapi, slotNumber, SystemIoBusNumber, ChipType,
                &mut apiomode, &mut wdmamode, &mut udmamode,
            ) { ret!(true); }
            ret!(true);
        }
        0x3388 => {
            // HiNT Corp. VXPro II EIDE
            if wdmamode >= 0
                && (AtapiReadPort1(chan, IDX_BM_Status as ULONG_PTR)
                    & (if DeviceNumber != 0 { BM_STATUS_DRIVE_1_DMA } else { BM_STATUS_DRIVE_0_DMA }) as UCHAR
                    != 0)
            {
                setmode(ATA_DMA);
            } else {
                setmode(ATA_PIO0 + apiomode as ULONG);
            }
            ret!(true);
        }
        x if x == ATA_JMICRON_ID => {
            let mut reg40: UCHAR = 0; get_pci!(0x40, reg40, 1);
            // controller snoops SET_FEATURE; nothing to do to set mode
            if apiomode >= 4 { apiomode = 4; }
            i = udmamode as LONG;
            while i >= 0 { if setmode(ATA_UDMA0 + i as ULONG) != 0 { ret!(true); } i -= 1; }
            i = wdmamode as LONG;
            while i >= 0 { if setmode(ATA_WDMA0 + i as ULONG) != 0 { ret!(true); } i -= 1; }
            if setmode(ATA_PIO0 + apiomode as ULONG) != 0 { ret!(true); }
            ret!(true);
        }
        _ => {}
    }

    // dma_cs55xx: (Cyrix/National goto target)
    if goto_dma_cs55xx {
        if apiomode >= 4 { apiomode = 4; }
        if ChipType == CYRIX_3x {
            let cyr_piotiming: [ULONG; 5] =
                [0x00009172, 0x00012171, 0x00020080, 0x00032010, 0x00040010];
            let cyr_wdmatiming: [ULONG; 3] = [0x00077771, 0x00012121, 0x00002020];
            let cyr_udmatiming: [ULONG; 3] = [0x00921250, 0x00911140, 0x00911030];
            let mode_reg = 0x24 + (dev << 3);
            i = udmamode as LONG;
            while i >= 0 {
                if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                    AtapiWritePortEx4(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, mode_reg, cyr_udmatiming[udmamode as usize]);
                    ret!(true);
                }
                i -= 1;
            }
            i = wdmamode as LONG;
            while i >= 0 {
                if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                    AtapiWritePortEx4(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, mode_reg, cyr_wdmatiming[wdmamode as usize]);
                    ret!(true);
                }
                i -= 1;
            }
            if setmode(ATA_PIO0 + apiomode as ULONG) != 0 {
                AtapiWritePortEx4(chan, (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, mode_reg, cyr_piotiming[apiomode as usize]);
                ret!(true);
            }
        } else if ChipType == CYRIX_OLD {
            let cyr_piotiming_old: [UCHAR; 5] = [11, 6, 3, 2, 1];
            let timing: UCHAR;
            i = wdmamode as LONG;
            while i >= 0 { if setmode(ATA_WDMA0 + i as ULONG) != 0 { ret!(true); } i -= 1; }
            if setmode(ATA_PIO0 + apiomode as ULONG) != 0 {
                timing = ((6 - apiomode as LONG) as UCHAR) | cyr_piotiming_old[i as usize];
                set_pci!(0x62 + Channel, timing, UCHAR, 1);
                set_pci!(0x64 + Channel * 4 + dev, timing, UCHAR, 1);
                set_pci!(0x66 + Channel * 4 + dev, timing, UCHAR, 1);
                ret!(true);
            }
        } else if ChipType == CYRIX_35 {
            // No MSR support yet, do not touch any regs
            i = udmamode as LONG;
            while i >= 0 { if setmode(ATA_UDMA0 + i as ULONG) != 0 { ret!(true); } i -= 1; }
            i = wdmamode as LONG;
            while i >= 0 { if setmode(ATA_WDMA0 + i as ULONG) != 0 { ret!(true); } i -= 1; }
            if setmode(ATA_PIO0 + apiomode as ULONG) != 0 { ret!(true); }
        }
        ret!(true);
    }

    // l_ATA_SILICON_IMAGE_ID: (SiliconImage/CMD goto target)
    if goto_sii {
        if dma_init_sii(
            deviceExtension, HwDeviceExtension, DeviceNumber, lChannel, Channel, dev,
            slotNumber, SystemIoBusNumber, ChipType,
            &mut apiomode, &mut wdmamode, &mut udmamode,
        ) { ret!(true); }
        ret!(true);
    }

    // break -> try_generic_dma
    *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode;
    false
}

// --- Intel arm (always returns; helper returns true) ---
/// Intel 450KX/450NX host-bridge errata that make IDE bus-master DMA unsafe
/// (450KX/GX Spec Update 243109-015, 450NX 243848-009; upstream intel.c
/// IntelPciBridgeErrataMatch). UniATA never checked. Scans bus 0, where these
/// bridges live, once; the answer is cached (0 = not yet, 1 = no, 2 = yes).
unsafe fn intel_450_bridge_errata(HwDeviceExtension: PVOID) -> bool {
    use core::sync::atomic::{AtomicU8, Ordering};
    static CACHED: AtomicU8 = AtomicU8::new(0);
    match CACHED.load(Ordering::Relaxed) {
        1 => return false,
        2 => return true,
        _ => {}
    }
    let mut hit = false;
    for slot in 0..32u32 {
        let mut ids: ULONG = 0;
        let mut rev: UCHAR = 0;
        if ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, 0, slot,
            &mut ids as *mut _ as PVOID, 0, 4) < 4 || (ids & 0xffff) != 0x8086 {
            continue;
        }
        ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, 0, slot,
            &mut rev as *mut _ as PVOID, 8, 1);
        match ids >> 16 {
            // 450KX before B0: 0-byte length write failure (#2)
            0x84C4 if rev < 0x04 => hit = true,
            0x84CB => {
                let mut pxb: USHORT = 0;
                ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, 0, slot,
                    &mut pxb as *mut _ as PVOID, 0x40, 2);
                // B0 PXB livelock (#19); arbiter deadlock with PCI bus lock on
                // anything but B1 (#20); data corruption with re-streaming on C0 (#25)
                if rev == 0x00 || (rev != 0x02 && pxb & 0x4000 != 0) || (rev == 0x04 && pxb & 0x0040 != 0) {
                    hit = true;
                }
            }
            _ => {}
        }
        if hit {
            break;
        }
    }
    CACHED.store(if hit { 2 } else { 1 }, Ordering::Relaxed);
    hit
}

/// Upstream intel.c's TYPE_ICH parts (82801AA ... ICH8-M PATA).
const INTEL_ICH_PATA_IDS: [ULONG; 16] = [
    0x2411, 0x2421, 0x244A, 0x244B, 0x245B, 0x248A, 0x248B, 0x24C1,
    0x24CA, 0x24CB, 0x24DB, 0x25A2, 0x266F, 0x269E, 0x27DF, 0x2850,
];

unsafe fn dma_init_intel(
    deviceExtension: PHW_DEVICE_EXTENSION,
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Channel: ULONG,
    dev: ULONG,
    isAtapi: BOOLEAN,
    slotNumber: ULONG,
    SystemIoBusNumber: ULONG,
    ChipFlags: ULONG,
    apiomode_io: &mut SCHAR,
    wdmamode_io: &mut SCHAR,
    udmamode_io: &mut SCHAR,
) -> bool {
    let mut apiomode = *apiomode_io;
    let mut wdmamode = *wdmamode_io;
    let mut udmamode = *udmamode_io;
    let mut i: LONG;
    let LunExt = (*(*deviceExtension).chan.add(lChannel as usize)).lun[DeviceNumber as usize];
    // Intel errata from upstream's intel.c that UniATA lacks:
    let devid = (*deviceExtension).DevID >> 16;
    let is_ich = INTEL_ICH_PATA_IDS.contains(&devid);
    // ICH/ICH0 revision 0: "IDE Bus Master Concurrency", PIO only.
    if (devid == 0x2411 || devid == 0x2421) && (*deviceExtension).RevID == 0 {
        udmamode = -1;
        wdmamode = -1;
    }
    if intel_450_bridge_errata(HwDeviceExtension) {
        udmamode = -1;
        wdmamode = -1;
    }
    // Every ICH: "MW DMA Mode-1 Tdh", so MWDMA0/1 cannot be driven; only
    // MWDMA2 is offered below.
    let wdma_floor: LONG = if is_ich { 2 } else { 0 };
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    macro_rules! set_pci { ($o:expr,$val:expr,$ty:ty,$s:expr) => {{ let mut _a: $ty = $val as $ty; ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber, &mut _a as *mut _ as PVOID, $o as ULONG, $s); }}; }
    let setmode = |m: ULONG| -> BOOLEAN { AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, m) };

    let mut udma_ok: BOOLEAN = FALSE as BOOLEAN;
    let mut idx: ULONG = 0;
    let intel_timings: [UCHAR; 14] =
        [0x00, 0x00, 0x10, 0x21, 0x23, 0x10, 0x21, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23];
    let intel_utimings: [UCHAR; 7] = [0x00, 0x01, 0x02, 0x01, 0x02, 0x01, 0x02];
    let needed_pio: [UCHAR; 3] = [ATA_PIO0 as UCHAR, ATA_PIO3 as UCHAR, ATA_PIO4 as UCHAR];

    if (*deviceExtension).DevID == ATA_I82371FB {
        let mut reg4x: USHORT = 0;
        let mut control: USHORT = 0;
        i = wdmamode as LONG;
        while i >= 0 {
            idx = (5 + i) as ULONG;
            if setmode(ATA_WDMA0 + i as ULONG) != 0 { udma_ok = TRUE as BOOLEAN; break; }
            i -= 1;
        }
        if udma_ok == 0 {
            setmode(ATA_PIO0 + apiomode as ULONG);
            idx = apiomode as ULONG;
        } else {
            if (apiomode as UCHAR) < needed_pio[wdmamode as usize] {
                control |= 8;
            }
        }
        get_pci!(0x40 + Channel * 2, reg4x, 2);
        if apiomode as ULONG > ATA_PIO0 { control |= 0x03; } else { control |= 0x02; }
        if isAtapi == 0 { control |= 4; }
        reg4x &= 0xCC00 | (0x0E << (DeviceNumber * 4));
        reg4x |= control << (DeviceNumber * 4);
        reg4x |= (intel_timings[idx as usize] as USHORT) << 8;
        set_pci!(0x40 + Channel * 2, reg4x, USHORT, 2);
        *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode;
        return true;
    }

    if (*deviceExtension).DevID == ATA_ISCH {
        let mut tim: ULONG = 0;
        get_pci!(0x80 + dev * 4, tim, 4);
        i = udmamode as LONG;
        while i >= 0 {
            if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                tim |= 0x1 << 31;
                tim &= !(0x7 << 16);
                tim |= (i as ULONG) << 16;
                udma_ok = TRUE as BOOLEAN;
                apiomode = ATA_PIO4 as SCHAR;
                break;
            }
            i -= 1;
        }
        if udma_ok == 0 {
            i = wdmamode as LONG;
            while i >= 0 {
                if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                    tim &= !(0x1 << 31);
                    tim &= !(0x3 << 8);
                    tim |= (i as ULONG) << 8;
                    udma_ok = TRUE as BOOLEAN;
                    apiomode = if i == 0 { ATA_PIO0 as SCHAR } else if i == 1 { ATA_PIO3 as SCHAR } else { ATA_PIO4 as SCHAR };
                    break;
                }
                i -= 1;
            }
        }
        if udma_ok == 0 {
            setmode(ATA_PIO0 + apiomode as ULONG);
        }
        tim &= !0x7;
        tim |= apiomode as ULONG & 0x7;
        set_pci!(0x80 + dev * 4, tim, ULONG, 4);
        *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode;
        return true;
    }

    let mut reg48: USHORT = 0; get_pci!(0x48, reg48, 2);
    let mut reg4a: USHORT = 0; get_pci!(0x4a, reg4a, 2);
    let mut reg54: USHORT = 0; get_pci!(0x54, reg54, 2);
    reg54 |= 0x400;

    i = udmamode as LONG;
    while i >= 0 {
        if setmode(ATA_UDMA0 + i as ULONG) != 0 {
            set_pci!(0x48, reg48 | (0x0001 << dev), USHORT, 2);
            if (*deviceExtension).MaxTransferMode == ATA_UDMA3 {
                set_pci!(0x4a, reg4a | (0x03 << (dev << 2)), USHORT, 2);
            } else {
                set_pci!(0x4a, (reg4a & !(0x03 << (dev << 2))) | ((intel_utimings[i as usize] as USHORT) << (dev << 2)), USHORT, 2);
            }
            reg54 &= !(0x1001 << dev);
            if i > 2 { reg54 |= 0x1 << dev; }
            if i >= 5 { reg54 |= 0x1000 << dev; }
            set_pci!(0x54, reg54, USHORT, 2);
            udma_ok = TRUE as BOOLEAN;
            idx = (i + 8) as ULONG;
            if ChipFlags & ICH4_FIX != 0 {
                *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode;
                return true;
            }
            break;
        }
        i -= 1;
    }

    if udma_ok == 0 {
        set_pci!(0x48, reg48 & !(0x0001 << dev), USHORT, 2);
        if ChipFlags & ICH4_FIX == 0 {
            set_pci!(0x4a, reg4a & !(0x3 << (dev << 2)), USHORT, 2);
        }
        set_pci!(0x54, reg54 & !(0x1001 << dev), USHORT, 2);
        i = wdmamode as LONG;
        while i >= wdma_floor {
            if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                udma_ok = TRUE as BOOLEAN;
                idx = (i + 5) as ULONG;
                if ChipFlags & ICH4_FIX != 0 {
                    *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode;
                    return true;
                }
                break;
            }
            i -= 1;
        }
    }

    if udma_ok == 0 {
        setmode(ATA_PIO0 + apiomode as ULONG);
        idx = apiomode as ULONG;
    }

    let mut reg40: ULONG = 0; get_pci!(0x40, reg40, 4);
    let mut reg44: UCHAR = 0; get_pci!(0x44, reg44, 1);
    reg40 &= !0x00ff00ff;
    reg40 |= 0x40774077;

    let mut mask40: ULONG = 0x000000ff;
    let mut new40: ULONG = 0;
    let mut mask44: UCHAR = 0;
    let mut new44: UCHAR = 0;
    if DeviceNumber & 1 == 0 {
        mask40 = 0x00003300;
        new40 = (intel_timings[idx as usize] as ULONG) << 8;
    } else {
        mask44 = 0x0f;
        new44 = ((intel_timings[idx as usize] & 0x30) >> 2) | (intel_timings[idx as usize] & 0x03);
    }
    if Channel != 0 {
        mask40 <<= 16;
        new40 <<= 16;
        mask44 <<= 4;
        new44 <<= 4;
    }
    set_pci!(0x40, (reg40 & !mask40) | new40, ULONG, 4);
    set_pci!(0x44, (reg44 & !mask44) | new44, UCHAR, 1);
    *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode;
    true
}

// --- ServerWorks / ATI arm (always returns) ---
unsafe fn dma_init_serverworks(
    deviceExtension: PHW_DEVICE_EXTENSION,
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Channel: ULONG,
    dev: ULONG,
    slotNumber: ULONG,
    SystemIoBusNumber: ULONG,
    VendorID: ULONG,
    apiomode_io: &mut SCHAR,
    wdmamode_io: &mut SCHAR,
    udmamode_io: &mut SCHAR,
) -> bool {
    let apiomode = *apiomode_io;
    let wdmamode = *wdmamode_io;
    let udmamode = *udmamode_io;
    let mut i: LONG;
    let LunExt = (*(*deviceExtension).chan.add(lChannel as usize)).lun[DeviceNumber as usize];
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    macro_rules! set_pci { ($o:expr,$val:expr,$ty:ty,$s:expr) => {{ let mut _a: $ty = $val as $ty; ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber, &mut _a as *mut _ as PVOID, $o as ULONG, $s); }}; }
    let setmode = |m: ULONG| -> BOOLEAN { AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, m) };

    let sw_dma_modes: [ULONG; 3] = [0x70, 0x21, 0x20];
    let sw_pio_modes: [ULONG; 14] =
        [0x5d, 0x47, 0x34, 0x22, 0x20, 0x34, 0x22, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20];
    let offset = dev ^ 0x01;
    let bit_offset = offset * 8;

    macro_rules! ret { ($v:expr) => {{ *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode; return $v; }}; }

    // sw_pio_modes[8 + i] has no slot past UDMA5: no ServerWorks/ATI PATA part
    // does UDMA6 (upstream ati.c caps every IXP at UDMA5). The chip table said
    // UDMA6 for the IXP300+ PATA parts, and the C read one past the array (UB);
    // in Rust that is a bounds panic, i.e. a BUGCHECK at boot. The table is fixed
    // too, but this function's own table is the limit, so enforce it here.
    i = core::cmp::min(udmamode as LONG, 5);
    while i >= 0 {
        if setmode(ATA_UDMA0 + i as ULONG) != 0 {
            let mut reg56: USHORT = 0; get_pci!(0x56, reg56, 2);
            reg56 &= !(0xf << (dev * 4));
            reg56 |= (i as USHORT) << (dev * 4);
            set_pci!(0x56, reg56, USHORT, 2);
            let mut a: UCHAR = 0; get_pci!(0x54, a, 1);
            set_pci!(0x54, a as ULONG | (0x01 << dev), UCHAR, 1);
            let mut reg44: ULONG = 0; get_pci!(0x44, reg44, 4);
            reg44 = (reg44 & !(0xff << bit_offset)) | (sw_dma_modes[2] << bit_offset);
            set_pci!(0x44, reg44, ULONG, 4);
            let mut reg40: ULONG = 0; get_pci!(0x40, reg40, 4);
            reg40 = (reg40 & !(0xff << bit_offset)) | (sw_pio_modes[8 + i as usize] << bit_offset);
            set_pci!(0x40, reg40, ULONG, 4);
            ret!(true);
        }
        i -= 1;
    }
    i = wdmamode as LONG;
    while i >= 0 {
        if setmode(ATA_WDMA0 + i as ULONG) != 0 {
            let mut a: UCHAR = 0; get_pci!(0x54, a, 1);
            set_pci!(0x54, a as ULONG & !(0x01 << dev), UCHAR, 1);
            let mut reg44: ULONG = 0; get_pci!(0x44, reg44, 4);
            // The C indexed with `wdmamode` (the drive's ceiling) instead of `i` (the
            // mode just accepted), so a drive that refused its top WDMA mode still
            // got the fastest timing. Upstream's svw_pata.c programs the mode set.
            reg44 = (reg44 & !(0xff << bit_offset)) | (sw_dma_modes[i as usize] << bit_offset);
            set_pci!(0x44, reg44, ULONG, 4);
            let mut reg40: ULONG = 0; get_pci!(0x40, reg40, 4);
            reg40 = (reg40 & !(0xff << bit_offset)) | (sw_pio_modes[5 + i as usize] << bit_offset);
            set_pci!(0x40, reg40, ULONG, 4);
            ret!(true);
        }
        i -= 1;
    }
    setmode(ATA_PIO0 + apiomode as ULONG);
    if VendorID == ATA_ATI_ID {
        let mut reg4a: USHORT = 0; get_pci!(0x4a, reg4a, 2);
        reg4a = (reg4a & !(0xf << (dev * 4))) | ((apiomode as USHORT) << (dev * 4));
        set_pci!(0x4a, reg4a, USHORT, 2);
    }
    let mut reg40: ULONG = 0; get_pci!(0x40, reg40, 4);
    reg40 = (reg40 & !(0xff << bit_offset)) | (sw_pio_modes[apiomode as usize] << bit_offset);
    set_pci!(0x40, reg40, ULONG, 4);
    ret!(true);
}

// --- SiliconImage/CMD arm (always returns) ---
unsafe fn dma_init_sii(
    deviceExtension: PHW_DEVICE_EXTENSION,
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Channel: ULONG,
    dev: ULONG,
    slotNumber: ULONG,
    SystemIoBusNumber: ULONG,
    ChipType: ULONG,
    apiomode_io: &mut SCHAR,
    wdmamode_io: &mut SCHAR,
    udmamode_io: &mut SCHAR,
) -> bool {
    let apiomode = *apiomode_io;
    let wdmamode = *wdmamode_io;
    let mut udmamode = *udmamode_io;
    let mut i: LONG;
    let LunExt = (*(*deviceExtension).chan.add(lChannel as usize)).lun[DeviceNumber as usize];
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    macro_rules! set_pci { ($o:expr,$val:expr,$ty:ty,$s:expr) => {{ let mut _a: $ty = $val as $ty; ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber, &mut _a as *mut _ as PVOID, $o as ULONG, $s); }}; }
    let setmode = |m: ULONG| -> BOOLEAN { AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, m) };
    macro_rules! ret { ($v:expr) => {{ *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode; return $v; }}; }

    if ChipType == SIIMIO {
        let sil_modes: [UCHAR; 7] = [0xf, 0xb, 0x7, 0x5, 0x3, 0x2, 0x1];
        let sil_wdma_modes: [USHORT; 3] = [0x2208, 0x10c2, 0x10c1];
        let sil_pio_modes: [USHORT; 6] = [0x328a, 0x2283, 0x1104, 0x10c3, 0x10c1, 0x10c1];
        let ureg: UCHAR = 0xac + (DeviceNumber as UCHAR * 0x02) + (Channel as UCHAR * 0x10);
        let mut uval: UCHAR = 0;
        let mreg: UCHAR = if Channel != 0 { 0x84 } else { 0x80 };
        let mask: UCHAR = if DeviceNumber != 0 { 0x30 } else { 0x03 };
        let mut mode: UCHAR = 0;
        get_pci!(ureg, uval, 1);
        get_pci!(mreg, mode, 1);
        i = udmamode as LONG;
        while i >= 0 {
            if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                set_pci!(mreg, mode | mask, UCHAR, 1);
                set_pci!(ureg, (uval & 0x3f) | sil_modes[i as usize], UCHAR, 1);
                ret!(true);
            }
            i -= 1;
        }
        i = wdmamode as LONG;
        while i >= 0 {
            if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                set_pci!(mreg, mode | (mask & 0x22), UCHAR, 1);
                set_pci!(ureg - 0x4, sil_wdma_modes[i as usize], USHORT, 2);
                ret!(true);
            }
            i -= 1;
        }
        setmode(ATA_PIO0 + apiomode as ULONG);
        set_pci!(mreg, mode | (mask & 0x11), UCHAR, 1);
        set_pci!(ureg - 0x8, sil_pio_modes[apiomode as usize], USHORT, 2);
        ret!(true);
    } else {
        let cmd_modes: [[UCHAR; 6]; 2] = [
            [0x31, 0x21, 0x011, 0x25, 0x15, 0x05],
            [0xc2, 0x82, 0x042, 0x8a, 0x4a, 0x0a],
        ];
        let cmd_wdma_modes: [UCHAR; 3] = [0x87, 0x32, 0x3f];
        let cmd_pio_modes: [UCHAR; 5] = [0xa9, 0x57, 0x44, 0x32, 0x3f];
        let treg: ULONG = 0x54 + (if dev < 3 { dev << 1 } else { 7 });
        udmamode = min_c(udmamode, 5);
        i = udmamode as LONG;
        while i >= 0 {
            let mut umode: UCHAR = 0;
            if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                get_pci!(if Channel != 0 { 0x7b } else { 0x73 }, umode, 1);
                umode &= !(if DeviceNumber & 1 == 0 { 0x35 } else { 0xca });
                umode |= cmd_modes[(DeviceNumber & 1) as usize][i as usize];
                set_pci!(if Channel != 0 { 0x7b } else { 0x73 }, umode, UCHAR, 1);
                ret!(true);
            }
            i -= 1;
        }
        {
            let mut a: UCHAR = 0; get_pci!(if Channel != 0 { 0x7b } else { 0x73 }, a, 1);
            set_pci!(if Channel != 0 { 0x7b } else { 0x73 }, a & !(if DeviceNumber & 1 == 0 { 0x35 } else { 0xca }), UCHAR, 1);
        }
        i = wdmamode as LONG;
        while i >= 0 {
            if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                set_pci!(treg, cmd_wdma_modes[i as usize], UCHAR, 1);
                let mut a: UCHAR = 0; get_pci!(if Channel != 0 { 0x7b } else { 0x73 }, a, 1);
                set_pci!(if Channel != 0 { 0x7b } else { 0x73 }, a & !(if DeviceNumber & 1 == 0 { 0x35 } else { 0xca }), UCHAR, 1);
                ret!(true);
            }
            i -= 1;
        }
        setmode(ATA_PIO0 + apiomode as ULONG);
        set_pci!(treg, cmd_pio_modes[apiomode as usize], UCHAR, 1);
        let mut a: UCHAR = 0; get_pci!(if Channel != 0 { 0x7b } else { 0x73 }, a, 1);
        set_pci!(if Channel != 0 { 0x7b } else { 0x73 }, a & !(if DeviceNumber & 1 == 0 { 0x35 } else { 0xca }), UCHAR, 1);
        ret!(true);
    }
}

// --- SiS arm (always returns) ---
unsafe fn dma_init_sis(
    deviceExtension: PHW_DEVICE_EXTENSION,
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    dev: ULONG,
    chan: PHW_CHANNEL,
    LunExt: PHW_LU_EXTENSION,
    slotNumber: ULONG,
    SystemIoBusNumber: ULONG,
    ChipType: ULONG,
    apiomode_io: &mut SCHAR,
    wdmamode_io: &mut SCHAR,
    udmamode_io: &mut SCHAR,
) -> bool {
    let _ = chan;
    let apiomode = *apiomode_io;
    let wdmamode = *wdmamode_io;
    let udmamode = *udmamode_io;
    let mut i: LONG;
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    macro_rules! set_pci { ($o:expr,$val:expr,$ty:ty,$s:expr) => {{ let mut _a: $ty = $val as $ty; ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber, &mut _a as *mut _ as PVOID, $o as ULONG, $s); }}; }
    let setmode = |m: ULONG| -> BOOLEAN { AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, m) };
    macro_rules! ret { ($v:expr) => {{ *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode; return $v; }}; }

    let sis_modes_new133: [ULONG; 15] = [
        0x28269008, 0x0c266008, 0x04263008, 0x0c0a3008, 0x05093008, 0x22196008, 0x0c0a3008,
        0x05093008, 0x050939fc, 0x050936ac, 0x0509347c, 0x0509325c, 0x0509323c, 0x0509322c,
        0x0509321c,
    ];
    let sis_modes_old133: [ULONG; 15] = [
        0x00cb, 0x0067, 0x0044, 0x0033, 0x0031, 0x0044, 0x0033, 0x0031, 0x8f31, 0x8a31, 0x8731,
        0x8531, 0x8331, 0x8231, 0x8131,
    ];
    let sis_modes_old: [ULONG; 14] = [
        0x0c0b, 0x0607, 0x0404, 0x0303, 0x0301, 0x0404, 0x0303, 0x0301, 0xf301, 0xd301, 0xb301,
        0xa301, 0x9301, 0x8301,
    ];
    let sis_modes_new100: [ULONG; 14] = [
        0x00cb, 0x0067, 0x0044, 0x0033, 0x0031, 0x0044, 0x0033, 0x0031, 0x8b31, 0x8731, 0x8531,
        0x8431, 0x8231, 0x8131,
    ];

    let sis_modes: &[ULONG];
    let mut reg: ULONG = 0;
    let mut reg_size: ULONG = 0;

    match ChipType {
        x if x == SIS133NEW => {
            sis_modes = &sis_modes_new133;
            reg_size = 4;
            let mut reg57: UCHAR = 0; get_pci!(0x57, reg57, 1);
            reg = (if reg57 & 0x40 != 0 { 0x70 } else { 0x40 }) + (dev * 4);
        }
        x if x == SIS133OLD => { sis_modes = &sis_modes_old133; reg_size = 2; reg = 0x40 + (dev * 2); }
        x if x == SIS100NEW => { sis_modes = &sis_modes_new100; reg_size = 2; reg = 0x40 + (dev * 2); }
        x if x == SIS100OLD || x == SIS66 || x == SIS33 => { sis_modes = &sis_modes_old; reg_size = 2; reg = 0x40 + (dev * 2); }
        _ => { sis_modes = &sis_modes_old; }
    }

    let mut offs = 5 + 3;
    i = udmamode as LONG;
    while i >= 0 {
        if setmode(ATA_UDMA0 + i as ULONG) != 0 {
            if reg_size == 4 { set_pci!(reg, sis_modes[offs + i as usize], ULONG, 4); }
            else { set_pci!(reg, sis_modes[offs + i as usize] as USHORT, USHORT, 2); }
            ret!(true);
        }
        i -= 1;
    }
    offs = 5;
    i = wdmamode as LONG;
    while i >= 0 {
        if setmode(ATA_WDMA0 + i as ULONG) != 0 {
            if reg_size == 4 { set_pci!(reg, sis_modes[offs + i as usize], ULONG, 4); }
            else { set_pci!(reg, sis_modes[offs + i as usize] as USHORT, USHORT, 2); }
            ret!(true);
        }
        i -= 1;
    }
    setmode(ATA_PIO0 + apiomode as ULONG);
    if reg_size == 4 { set_pci!(reg, sis_modes[apiomode as usize], ULONG, 4); }
    else { set_pci!(reg, sis_modes[apiomode as usize] as USHORT, USHORT, 2); }
    ret!(true);
}

// --- ITE arm (always returns) ---
unsafe fn dma_init_ite(
    deviceExtension: PHW_DEVICE_EXTENSION,
    HwDeviceExtension: PVOID,
    DeviceNumber: ULONG,
    lChannel: ULONG,
    Channel: ULONG,
    dev: ULONG,
    isAtapi: BOOLEAN,
    slotNumber: ULONG,
    SystemIoBusNumber: ULONG,
    ChipType: ULONG,
    apiomode_io: &mut SCHAR,
    wdmamode_io: &mut SCHAR,
    udmamode_io: &mut SCHAR,
) -> bool {
    let apiomode = *apiomode_io;
    let wdmamode = *wdmamode_io;
    let mut udmamode = *udmamode_io;
    let mut i: LONG;
    let LunExt = (*(*deviceExtension).chan.add(lChannel as usize)).lun[DeviceNumber as usize];
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    macro_rules! set_pci { ($o:expr,$val:expr,$ty:ty,$s:expr) => {{ let mut _a: $ty = $val as $ty; ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber, &mut _a as *mut _ as PVOID, $o as ULONG, $s); }}; }
    let setmode = |m: ULONG| -> BOOLEAN { AtaSetTransferMode(deviceExtension, DeviceNumber, lChannel, LunExt, m) };
    macro_rules! ret { ($v:expr) => {{ *apiomode_io = apiomode; *wdmamode_io = wdmamode; *udmamode_io = udmamode; return $v; }}; }

    if ChipType == ITE_33 {
        let a_speed: LONG = 3 << (dev * 4);
        let u_flag: LONG = 1 << dev;
        let u_speed: LONG = 0;
        let mut pio: LONG = 1;
        let mut reg48: UCHAR = 0;
        let mut reg4a: UCHAR = 0;
        let mut drive_enables: USHORT = 0;
        let mut drive_timing: ULONG = 0;
        get_pci!(0x48, reg48, 1);
        get_pci!(0x4a, reg4a, 1);

        let mut did_setup = false;
        i = udmamode as LONG;
        while i >= 0 {
            if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                set_pci!(0x48, reg48 as LONG | u_flag, UCHAR, 1);
                reg4a &= !(a_speed as UCHAR);
                set_pci!(0x4a, reg4a as LONG | u_speed, UCHAR, 1);
                pio = 4;
                did_setup = true; // goto setup_drive_ite
                break;
            }
            i -= 1;
        }
        if !did_setup {
            let mut done_w = false;
            i = wdmamode as LONG;
            while i >= 0 {
                if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                    set_pci!(0x48, reg48 as LONG & !u_flag, UCHAR, 1);
                    set_pci!(0x4a, reg4a as LONG & !a_speed, UCHAR, 1);
                    done_w = true; // return in C
                    break;
                }
                i -= 1;
            }
            if done_w { ret!(true); }
            setmode(ATA_PIO0 + apiomode as ULONG);
            set_pci!(0x48, reg48 as LONG & !u_flag, UCHAR, 1);
            set_pci!(0x4a, reg4a as LONG & !a_speed, UCHAR, 1);
            pio = apiomode as LONG;
        }
        // setup_drive_ite:
        get_pci!(0x40, drive_enables, 2);
        get_pci!(0x44, drive_timing, 4);
        drive_enables |= 0x4000;
        drive_enables &= 0xc000 | (0x06 << (DeviceNumber * 4));
        if pio > 1 { drive_enables |= 0x06 << (DeviceNumber * 4); }
        set_pci!(0x40, drive_enables, USHORT, 2);
        let _ = drive_timing;
        ret!(true);
    } else if ChipType == ITE_133 {
        let udmatiming: [UCHAR; 7] = [0x44, 0x42, 0x31, 0x21, 0x11, 0xa2, 0x91];
        let chtiming: [UCHAR; 8] = [0xaa, 0xa3, 0xa1, 0x33, 0x31, 0x88, 0x32, 0x31];
        let offset = (Channel << 2) + DeviceNumber;
        let mut reg54: UCHAR;
        i = udmamode as LONG;
        while i >= 0 {
            if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                let mut a: UCHAR = 0; get_pci!(0x50, a, 1);
                set_pci!(0x50, a as LONG & !(1 << (dev + 3)), UCHAR, 1);
                set_pci!(0x56 + offset, udmatiming[i as usize], UCHAR, 1);
                ret!(true);
            }
            i -= 1;
        }
        i = wdmamode as LONG;
        while i >= 0 {
            if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                let mut a: UCHAR = 0; get_pci!(0x50, a, 1);
                set_pci!(0x50, a as ULONG | (1 << (dev + 3)), UCHAR, 1);
                reg54 = 0; get_pci!(0x54 + offset, reg54, 1);
                if reg54 < chtiming[i as usize + 5] { set_pci!(0x54 + offset, chtiming[i as usize + 5], UCHAR, 1); }
                ret!(true);
            }
            i -= 1;
        }
        setmode(ATA_PIO0 + apiomode as ULONG);
        let mut a: UCHAR = 0; get_pci!(0x50, a, 1);
        set_pci!(0x50, a as ULONG | (1 << (dev + 3)), UCHAR, 1);
        reg54 = 0; get_pci!(0x54 + offset, reg54, 1);
        if reg54 < chtiming[apiomode as usize] { set_pci!(0x54 + offset, chtiming[apiomode as usize], UCHAR, 1); }
        ret!(true);
    } else if ChipType == ITE_133_NEW {
        let udmatiming: [UCHAR; 7] = [0x00, 0x01, 0x10, 0x01, 0x10, 0x01, 0x10];
        let timings: [UCHAR; 15] =
            [0x00, 0x00, 0x10, 0x21, 0x23, 0x10, 0x21, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23, 0x02, 0x02];
        let mut udma_ok: BOOLEAN = FALSE as BOOLEAN;
        let mut ok: BOOLEAN = FALSE as BOOLEAN;
        let mut timing: UCHAR = 0;
        let mut reg40: USHORT = 0;  // WCHAR in C; used as 16-bit value
        let mut reg44: UCHAR = 0;
        let mut reg4a: USHORT = 0;
        let mut reg54: USHORT = 0;
        let mut mask40: USHORT = 0;
        let mut new40: USHORT = 0;
        let mut mask44: UCHAR = 0;
        let mut new44: UCHAR = 0;
        get_pci!(0x40, reg40, 2);
        get_pci!(0x44, reg44, 1);
        get_pci!(0x4a, reg4a, 2);
        get_pci!(0x54, reg54, 2);
        let _ = reg4a;
        if reg54 & (0x10 << dev) == 0 {
            udmamode = min_c(udmamode, 2);
        }
        i = udmamode as LONG;
        while i >= 0 {
            if setmode(ATA_UDMA0 + i as ULONG) != 0 {
                let mut a: UCHAR = 0; get_pci!(0x48, a, 1);
                set_pci!(0x48, a as ULONG | (1 << dev), UCHAR, 1);
                let mut a2: USHORT = 0; get_pci!(0x4a, a2, 2);
                set_pci!(0x4a, (a2 as ULONG & !(0x3 << (dev * 4))) | ((udmatiming[i as usize] as ULONG) << (dev * 4)), USHORT, 2);
                ok = TRUE as BOOLEAN;
                udma_ok = TRUE as BOOLEAN;
                timing = timings[i as usize + 8];
                break;
            }
            i -= 1;
        }
        i = wdmamode as LONG;
        while ok == 0 && i >= 0 {
            if setmode(ATA_WDMA0 + i as ULONG) != 0 {
                ok = TRUE as BOOLEAN;
                timing = timings[i as usize + 5];
                break;
            }
            i -= 1;
        }
        if ok == 0 {
            setmode(ATA_PIO0 + apiomode as ULONG);
            timing = timings[apiomode as usize];
        }
        if udma_ok == 0 {
            let mut a: UCHAR = 0; get_pci!(0x48, a, 1);
            set_pci!(0x48, a as ULONG & !(1 << dev), UCHAR, 1);
            let mut a2: USHORT = 0; get_pci!(0x4a, a2, 2);
            set_pci!(0x4a, a2 as ULONG & !(0x3 << (dev << 2)), USHORT, 2);
        }
        if udma_ok != 0 && udmamode as ULONG >= ATA_UDMA2 { reg54 |= 0x1 << dev; } else { reg54 &= !(0x1 << dev); }
        if udma_ok != 0 && udmamode as ULONG >= ATA_UDMA5 { reg54 |= 0x1000 << dev; } else { reg54 &= !(0x1000 << dev); }
        set_pci!(0x54, reg54, USHORT, 2);
        reg40 &= 0xff00;
        reg40 |= 0x4033;
        if DeviceNumber & 1 == 0 {
            reg40 |= if isAtapi != 0 { 0x04 } else { 0x00 };
            mask40 = 0x3300;
            new40 = (timing as USHORT) << 8;
        } else {
            reg40 |= if isAtapi != 0 { 0x40 } else { 0x00 };
            mask44 = 0x0f;
            new44 = ((timing & 0x30) >> 2) | (timing & 0x03);
        }
        set_pci!(0x40, (reg40 & !mask40) | new40, USHORT, 2);
        set_pci!(0x44, (reg44 & !mask44) | new44, UCHAR, 1);
        ret!(true);
    }
    ret!(true);
}

// ===========================================================================
//  cyrix_timing
// ===========================================================================
pub unsafe extern "system" fn cyrix_timing(
    deviceExtension: PHW_DEVICE_EXTENSION,
    dev: ULONG,
    mut mode: CHAR,
) {
    let mut reg20: ULONG = 0x0000e132;
    let mut reg24: ULONG = 0x00017771;

    if mode as ULONG == ATA_PIO5 { mode = ATA_PIO4 as CHAR; }

    match mode as ULONG {
        x if x == ATA_PIO0 => reg20 = 0x0000e132,
        x if x == ATA_PIO1 => reg20 = 0x00018121,
        x if x == ATA_PIO2 => reg20 = 0x00024020,
        x if x == ATA_PIO3 => reg20 = 0x00032010,
        x if x == ATA_PIO4 || x == ATA_PIO5 => reg20 = 0x00040010,
        x if x == ATA_WDMA2 => reg24 = 0x00002020,
        x if x == ATA_UDMA2 => reg24 = 0x00911030,
        _ => {}
    }
    AtapiWritePortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, (dev * 8) + 0x20, reg20);
    AtapiWritePortEx4(core::ptr::null_mut(), (&(*deviceExtension).BaseIoAddressBM_0) as *const _ as ULONG_PTR, (dev * 8) + 0x24, reg24);
} // cyrix_timing()

// ===========================================================================
//  promise_timing  (the Linux-code path; #if 0 FreeBSD path dropped)
// ===========================================================================
pub unsafe extern "system" fn promise_timing(
    deviceExtension: PHW_DEVICE_EXTENSION,
    dev: ULONG,
    mut mode: CHAR,
) {
    let HwDeviceExtension = deviceExtension as PVOID;
    let slotNumber: ULONG = (*deviceExtension).slotNumber;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    macro_rules! set_pci { ($o:expr,$val:expr,$ty:ty,$s:expr) => {{ let mut _a: $ty = $val as $ty; ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber, &mut _a as *mut _ as PVOID, $o as ULONG, $s); }}; }

    let port: ULONG = 0x60 + (dev << 2);

    if mode as ULONG == ATA_PIO5 { mode = ATA_PIO4 as CHAR; }
    if (mode as LONG) < ATA_PIO0 as LONG { mode = ATA_PIO0 as CHAR; }

    let udma_timing: [[UCHAR; 2]; 6] = [
        [0x60, 0x03],
        [0x40, 0x02],
        [0x20, 0x01],
        [0x40, 0x02],
        [0x20, 0x01],
        [0x20, 0x01],
    ];
    let mdma_timing: [[UCHAR; 2]; 3] = [[0xe0, 0x0f], [0x60, 0x04], [0x60, 0x03]];
    let pio_timing: [USHORT; 5] = [0x0913, 0x050C, 0x0308, 0x0206, 0x0104];

    if mode as ULONG > ATA_UDMA5 { return; }

    if mode as ULONG > ATA_WDMA0 {
        let mut r_bp: UCHAR = 0; get_pci!(port + 1, r_bp, 1);
        let mut r_cp: UCHAR = 0; get_pci!(port + 2, r_cp, 1);
        r_bp &= !0xE0;
        r_cp &= !0x0F;
        let idx: ULONG;
        if mode as ULONG >= ATA_UDMA0 {
            idx = mode as ULONG - ATA_UDMA0;
            r_bp |= udma_timing[idx as usize][0];
            r_cp |= udma_timing[idx as usize][1];
        } else {
            idx = mode as ULONG - ATA_WDMA0;
            r_bp |= mdma_timing[idx as usize][0];
            r_cp |= mdma_timing[idx as usize][1];
        }
        set_pci!(port + 1, r_bp, UCHAR, 1);
        set_pci!(port + 2, r_cp, UCHAR, 1);
    } else if mode as ULONG <= ATA_PIO5 {
        let mut r_ap: UCHAR = 0; get_pci!(port + 0, r_ap, 1);
        let mut r_bp: UCHAR = 0; get_pci!(port + 1, r_bp, 1);
        let idx = mode as ULONG - ATA_PIO0;
        r_ap &= !0x3F;
        r_bp &= !0x1F;
        r_ap |= (pio_timing[idx as usize] >> 8) as UCHAR;
        r_bp |= (pio_timing[idx as usize] & 0xFF) as UCHAR;
        r_ap |= 0x20; // IORDY enable
        set_pci!(port + 0, r_ap, UCHAR, 1);
        set_pci!(port + 1, r_bp, UCHAR, 1);
    }
} // end promise_timing()

// ===========================================================================
//  hpt_timing
// ===========================================================================
pub unsafe extern "system" fn hpt_timing(
    deviceExtension: PHW_DEVICE_EXTENSION,
    dev: ULONG,
    mut mode: CHAR,
) {
    let HwDeviceExtension = deviceExtension as PVOID;
    let slotNumber: ULONG = (*deviceExtension).slotNumber;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    macro_rules! set_pci { ($o:expr,$val:expr,$ty:ty,$s:expr) => {{ let mut _a: $ty = $val as $ty; ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber, &mut _a as *mut _ as PVOID, $o as ULONG, $s); }}; }

    let ChipType: ULONG = (*deviceExtension).HwFlags & CHIPTYPE_MASK;
    let mut timing: ULONG = 0;

    if mode as ULONG == ATA_PIO5 { mode = ATA_PIO4 as CHAR; }

    let m = mode as ULONG;
    match ChipType {
        x if x == HPT374 => {
            timing = match m {
                v if v == ATA_PIO0 => 0x0ac1f48a,
                v if v == ATA_PIO1 => 0x0ac1f465,
                v if v == ATA_PIO2 => 0x0a81f454,
                v if v == ATA_PIO3 => 0x0a81f443,
                v if v == ATA_PIO4 => 0x0a81f442,
                v if v == ATA_WDMA0 => 0x228082ea,
                v if v == ATA_WDMA1 => 0x22808254,
                v if v == ATA_WDMA2 => 0x22808242,
                v if v == ATA_UDMA0 => 0x121882ea,
                v if v == ATA_UDMA1 => 0x12148254,
                v if v == ATA_UDMA2 => 0x120c8242,
                v if v == ATA_UDMA3 => 0x128c8242,
                v if v == ATA_UDMA4 => 0x12ac8242,
                v if v == ATA_UDMA5 => 0x12848242,
                v if v == ATA_UDMA6 => 0x12808242,
                _ => 0x0d029d5e,
            };
        }
        x if x == HPT372 => {
            timing = match m {
                v if v == ATA_PIO0 => 0x0d029d5e,
                v if v == ATA_PIO1 => 0x0d029d26,
                v if v == ATA_PIO2 => 0x0c829ca6,
                v if v == ATA_PIO3 => 0x0c829c84,
                v if v == ATA_PIO4 => 0x0c829c62,
                v if v == ATA_WDMA0 => 0x2c82922e,
                v if v == ATA_WDMA1 => 0x2c829266,
                v if v == ATA_WDMA2 => 0x2c829262,
                v if v == ATA_UDMA0 => 0x1c829c62,
                v if v == ATA_UDMA1 => 0x1c9a9c62,
                v if v == ATA_UDMA2 => 0x1c929c62,
                v if v == ATA_UDMA3 => 0x1c8e9c62,
                v if v == ATA_UDMA4 => 0x1c8a9c62,
                v if v == ATA_UDMA5 => 0x1c8a9c62,
                v if v == ATA_UDMA6 => 0x1c869c62,
                _ => 0x0d029d5e,
            };
        }
        x if x == HPT370 => {
            // C: HPT370 has no break and falls through to HPT366, so its own table is never used.
            // fall through to HPT366
            timing = hpt366_timing(deviceExtension, dev, m, slotNumber, SystemIoBusNumber, HwDeviceExtension);
        }
        x if x == HPT366 => {
            timing = hpt366_timing(deviceExtension, dev, m, slotNumber, SystemIoBusNumber, HwDeviceExtension);
        }
        _ => {}
    }

    set_pci!(0x40 + (dev << 2), timing, ULONG, 4);
} // end hpt_timing()

// HPT366 sub-table (also the fall-through target of HPT370).
unsafe fn hpt366_timing(
    _deviceExtension: PHW_DEVICE_EXTENSION,
    dev: ULONG,
    m: ULONG,
    slotNumber: ULONG,
    SystemIoBusNumber: ULONG,
    HwDeviceExtension: PVOID,
) -> ULONG {
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    let mut reg41: UCHAR = 0;
    get_pci!(0x41 + (dev << 2), reg41, 1);
    match reg41 {
        0x85 => match m {
            v if v == ATA_PIO0 => 0x40d08585,
            v if v == ATA_PIO1 => 0x40d08572,
            v if v == ATA_PIO2 => 0x40ca8542,
            v if v == ATA_PIO3 => 0x40ca8532,
            v if v == ATA_PIO4 => 0x40ca8521,
            v if v == ATA_WDMA2 => 0x20ca8521,
            v if v == ATA_UDMA2 => 0x10cf8521,
            v if v == ATA_UDMA4 => 0x10c98521,
            _ => 0x01208585,
        },
        0xd9 => match m {
            v if v == ATA_PIO0 => 0x4018d9d9,
            v if v == ATA_PIO1 => 0x4010d9c7,
            v if v == ATA_PIO2 => 0x4010d997,
            v if v == ATA_PIO3 => 0x4010d974,
            v if v == ATA_PIO4 => 0x4008d963,
            v if v == ATA_WDMA2 => 0x2008d943,
            v if v == ATA_UDMA2 => 0x100bd943,
            v if v == ATA_UDMA4 => 0x100fd943,
            _ => 0x0120d9d9,
        },
        // default + 0xa7 (33MHz)
        _ => match m {
            v if v == ATA_PIO0 => 0x40d0a7aa,
            v if v == ATA_PIO1 => 0x40d0a7a3,
            v if v == ATA_PIO2 => 0x40d0a753,
            v if v == ATA_PIO3 => 0x40c8a742,
            v if v == ATA_PIO4 => 0x40c8a731,
            v if v == ATA_WDMA0 => 0x20c8a797,
            v if v == ATA_WDMA1 => 0x20c8a732,
            v if v == ATA_WDMA2 => 0x20c8a731,
            v if v == ATA_UDMA0 => 0x10c8a731,
            v if v == ATA_UDMA1 => 0x10cba731,
            v if v == ATA_UDMA2 => 0x10caa731,
            v if v == ATA_UDMA3 => 0x10cfa731,
            v if v == ATA_UDMA4 => 0x10c9a731,
            _ => 0x0120a7a7,
        },
    }
}

// ===========================================================================
//  via82c_timing
// ===========================================================================
// FIT(v,min,max) = (((v)>(max)?(max):(v))<(min)?(min):(v))
#[inline(always)]
fn fit(v: LONG, lo: LONG, hi: LONG) -> LONG {
    let t = if v > hi { hi } else { v };
    if t < lo { lo } else { t }
}

pub unsafe extern "system" fn via82c_timing(
    deviceExtension: PHW_DEVICE_EXTENSION,
    dev: ULONG,
    mode: CHAR,
) {
    let HwDeviceExtension = deviceExtension as PVOID;
    let slotNumber: ULONG = (*deviceExtension).slotNumber;
    let SystemIoBusNumber: ULONG = (*deviceExtension).SystemIoBusNumber;
    macro_rules! get_pci { ($o:expr,$v:expr,$s:expr) => {{ ScsiPortGetBusDataByOffset(HwDeviceExtension, PCIConfiguration, SystemIoBusNumber, slotNumber, &mut $v as *mut _ as PVOID, $o as ULONG, $s); }}; }
    macro_rules! set_pci { ($o:expr,$val:expr,$ty:ty,$s:expr) => {{ let mut _a: $ty = $val as $ty; ScsiPortSetBusDataByOffset(HwDeviceExtension, PCIConfiguration as ULONG, SystemIoBusNumber, slotNumber, &mut _a as *mut _ as PVOID, $o as ULONG, $s); }}; }

    // Newer chips dislike this:
    if (*deviceExtension).MaxTransferMode < ATA_UDMA6 {
        return;
    }

    let t_clk: USHORT = 1000 / 33;

    let mut setup: USHORT;
    let mut active: USHORT;
    let mut recover: USHORT;
    let mut cycle: USHORT;

    match mode as ULONG {
        x if x == ATA_PIO1 => { setup = 50; active = 125; recover = 100; cycle = 383; }
        x if x == ATA_PIO2 => { setup = 30; active = 100; recover = 90; cycle = 240; }
        x if x == ATA_PIO3 => { setup = 30; active = 80; recover = 70; cycle = 180; }
        x if x == ATA_PIO4 => { setup = 25; active = 70; recover = 25; cycle = 120; }
        x if x == ATA_PIO5 => { setup = 20; active = 50; recover = 30; cycle = 100; }
        _ /* ATA_PIO0 + default */ => { setup = 70; active = 165; recover = 150; cycle = 600; }
    }

    setup = (setup - 1) / (t_clk + 1);
    active = (active - 1) / (t_clk + 1);
    recover = (recover - 1) / (t_clk + 1);
    cycle = (cycle - 1) / (t_clk + 1);

    if active + recover < cycle {
        active += (cycle - (active + recover)) / 2;
        recover = cycle - active;
    }

    // PIO address setup
    let mut t: UCHAR = 0;
    get_pci!(0x4c, t, 1);
    t = (t & !(3 << ((3 - dev) << 1)) as UCHAR)
        | ((fit(setup as LONG - 1, 0, 3) as UCHAR) << ((3 - dev) << 1));
    set_pci!(0x4c, t, UCHAR, 1);

    // PIO active & recover
    set_pci!(
        0x4b - dev,
        ((fit(active as LONG - 1, 0, 0xf) as UCHAR) << 4) | fit(recover as LONG - 1, 0, 0xf) as UCHAR,
        UCHAR,
        1
    );
} // end via82c_timing()
