//! id_queue_badblock — Rust translation of UniATA's command-queue reorder/load-balance
//! logic (id_queue.cpp) and the artificial badblock-simulation registry layer
//! (id_badblock.cpp). Two small .cpp → one module per the fan-out contract.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]

use crate::types::*;
use crate::regidx::*;
use crate::*; // sibling uniata functions (EncodeVendorStr, UniAtaCalculateLBARegsBack, …)
use ntkrnl_sys::ddk::*;
// TRUE/FALSE/PCVOID/STATUS_SUCCESS live in ntkrnl_sys::base (ddk's TRUE/FALSE are u32);
// RtlCopyMemory is an inline in ntkrnl_sys::rtl. Explicit imports shadow the ddk globs.
use ntkrnl_sys::base::{TRUE, FALSE, PCVOID, STATUS_SUCCESS};
use ntkrnl_sys::rtl::RtlCopyMemory;

// ---------------------------------------------------------------------------
// REORDER_COST_* (bsmaster.h — bindgen drops these 64-bit shift #defines):
//   REORDER_COST_MAX      = (1 << 60) - 1
//   REORDER_COST_TTL      = MAX - 1
//   REORDER_COST_INTERSECT= MAX - 2
//   REORDER_COST_DENIED   = MAX - 3
//   REORDER_COST_RESELECT = MAX / 4
// ---------------------------------------------------------------------------
const REORDER_COST_MAX: LONGLONG = (1i64 << 60) - 1;
const REORDER_COST_TTL: LONGLONG = REORDER_COST_MAX - 1;
const REORDER_COST_INTERSECT: LONGLONG = REORDER_COST_MAX - 2;
const REORDER_COST_DENIED: LONGLONG = REORDER_COST_MAX - 3;
const REORDER_COST_RESELECT: LONGLONG = REORDER_COST_MAX / 4;

// ---------------------------------------------------------------------------
// id_badblock.cpp top-level globals
// ---------------------------------------------------------------------------
pub static mut BBList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
pub static mut BBListInited: BOOLEAN = FALSE as BOOLEAN;

// ---------------------------------------------------------------------------
// Local helpers
// ---------------------------------------------------------------------------

// ATA_REQ field accessor — bindgen buries the linked-list fields (next_req,
// prev_req, Srb, lba, bcount, ttl, Flags, ReqState) under nested anon unions:
// _ATA_REQ.__bindgen_anon_1(union)->bindgen_ty_1.__bindgen_anon_1(struct).
#[inline(always)]
unsafe fn ata_req_core(r: PATA_REQ) -> *mut _ATA_REQ__bindgen_ty_1__bindgen_ty_1 {
    &mut (*r).__bindgen_anon_1.as_mut().__bindgen_anon_1
}

// Minimal CRT/kernel-list helpers (not exported from ntkrnl_sys::ddk for this crate).
#[inline(always)]
unsafe fn init_list_head(head: *mut LIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}

#[inline(always)]
unsafe fn insert_tail_list(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    let blink = (*head).Blink;
    (*entry).Flink = head;
    (*entry).Blink = blink;
    (*blink).Flink = entry;
    (*head).Blink = entry;
}

// wcscmp: returns 0 on equality (only the ==0 result is consumed by callers).
#[inline(always)]
unsafe fn wcscmp(a: *const WCHAR, b: *const WCHAR) -> i32 {
    let mut i = 0isize;
    loop {
        let ca = *a.offset(i);
        let cb = *b.offset(i);
        if ca != cb {
            return if ca < cb { -1 } else { 1 };
        }
        if ca == 0 {
            return 0;
        }
        i += 1;
    }
}

// wcsncpy: copy up to count wchars; pad with NUL like the C runtime.
#[inline(always)]
unsafe fn wcsncpy(dst: *mut WCHAR, src: *const WCHAR, count: usize) {
    let mut i = 0usize;
    while i < count {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    while i < count {
        *dst.add(i) = 0;
        i += 1;
    }
}

#[inline(always)]
fn max_i64(a: LONGLONG, b: LONGLONG) -> LONGLONG {
    if a > b { a } else { b }
}

// ===========================================================================
//  id_queue.cpp
// ===========================================================================

/// Get cost of insertion Req1 before Req2.
pub unsafe extern "system" fn UniataGetCost(
    LunExt: PHW_LU_EXTENSION,
    AtaReq1: PATA_REQ,
    AtaReq2: PATA_REQ,
) -> LONGLONG {
    let write1: BOOLEAN;
    let write2: BOOLEAN;
    let flags1: UCHAR;
    let flags2: UCHAR;
    let mut cost: LONGLONG;

    let r1 = ata_req_core(AtaReq1);
    let r2 = ata_req_core(AtaReq2);

    // can't insert Req1 before tooooo old Req2
    if (*r2).ttl == 0 {
        return REORDER_COST_TTL;
    }
    // check if reorderable
    flags1 = (*r1).Flags;
    flags2 = (*r2).Flags;
    if (((flags2 & flags1) as u32) & REQ_FLAG_REORDERABLE_CMD) == 0 {
        return REORDER_COST_DENIED;
    }
    // if at least one Req is WRITE, target areas can not intersect
    write1 = ((flags1 as u32 & REQ_FLAG_RW_MASK) == REQ_FLAG_WRITE) as BOOLEAN;
    write2 = ((flags2 as u32 & REQ_FLAG_RW_MASK) == REQ_FLAG_WRITE) as BOOLEAN;
    cost = (*r1).lba + (*r1).bcount as LONGLONG - (*r2).lba;
    if write1 != 0 || write2 != 0 {
        // check for intersection
        if ((*r1).lba < (*r2).lba + (*r2).bcount as LONGLONG)
            && ((*r1).lba + (*r1).bcount as LONGLONG > (*r2).lba)
        {
            // Intersection...
            return REORDER_COST_INTERSECT;
        }
    }
    if cost < 0 {
        cost *= 1 - (*LunExt).SeekBackMCost;
    } else {
        cost *= (*LunExt).SeekBackMCost - 1;
    }
    if write1 == write2 {
        return cost;
    }
    (cost * (*LunExt).RwSwitchMCost) + (*LunExt).RwSwitchCost
} // end UniataGetCost()

/// Insert command into the proper place of the command queue; reorder if needed.
pub unsafe extern "system" fn UniataQueueRequest(chan: PHW_CHANNEL, Srb: PSCSI_REQUEST_BLOCK) {
    let AtaReq: PATA_REQ = (*Srb).SrbExtension as PATA_REQ;
    let mut AtaReq1: PATA_REQ;
    let mut AtaReq2: PATA_REQ;

    let mut best_cost: LONGLONG = 0;
    let mut new_cost: LONGLONG;
    let mut new_cost1: LONGLONG;
    let mut new_cost2: LONGLONG;
    let mut BestAtaReq1: PATA_REQ;

    let mut use_reorder: BOOLEAN = (*chan).UseReorder;

    // GET_CDEV(Srb) -> (*Srb).TargetId
    let LunExt: PHW_LU_EXTENSION = (*chan).lun[(*Srb).TargetId as usize];
    (*ata_req_core(AtaReq)).Srb = Srb;

    // check if queue is empty
    if (*LunExt).queue_depth != 0 {
        (*ata_req_core(AtaReq)).ttl =
            max_i64((*LunExt).queue_depth as LONGLONG, MIN_REQ_TTL as LONGLONG) as UCHAR;

        // init loop
        AtaReq2 = (*LunExt).last_req;
        BestAtaReq1 = AtaReq2;

        if use_reorder != 0 && ((*Srb).SrbFlags & SRB_FLAGS_QUEUE_ACTION_ENABLE) != 0 {
            match (*Srb).QueueAction as u32 {
                SRB_ORDERED_QUEUE_TAG_REQUEST => {
                    use_reorder = FALSE as BOOLEAN;
                }
                SRB_HEAD_OF_QUEUE_TAG_REQUEST => {
                    BestAtaReq1 = (*LunExt).first_req;
                    best_cost = -REORDER_COST_RESELECT;
                }
                // SRB_SIMPLE_TAG_REQUEST and default
                _ => {
                    best_cost = UniataGetCost(LunExt, BestAtaReq1, AtaReq);
                    use_reorder |= TRUE as BOOLEAN;
                }
            }
        } else if use_reorder != 0 {
            best_cost = UniataGetCost(LunExt, BestAtaReq1, AtaReq);
        }

        if use_reorder != 0 {
            // walk through command queue and find the best place for insertion
            loop {
                AtaReq1 = (*ata_req_core(AtaReq2)).prev_req;
                if AtaReq1.is_null() {
                    break;
                }
                new_cost1 = UniataGetCost(LunExt, AtaReq1, AtaReq);
                new_cost2 = UniataGetCost(LunExt, AtaReq, AtaReq2);

                if new_cost2 > REORDER_COST_RESELECT {
                    break;
                }

                new_cost = UniataGetCost(LunExt, AtaReq1, AtaReq2);
                new_cost = new_cost1 + new_cost2 - new_cost;

                // check for Stop Reordering
                if new_cost > REORDER_COST_RESELECT * 3 {
                    break;
                }

                if new_cost < best_cost {
                    best_cost = new_cost;
                    BestAtaReq1 = AtaReq1;
                }
                AtaReq2 = AtaReq1;
            }
        }
        // Insert Req between Req2 & Req1
        AtaReq2 = (*ata_req_core(BestAtaReq1)).next_req;
        if !AtaReq2.is_null() {
            (*ata_req_core(AtaReq2)).prev_req = AtaReq;
            (*ata_req_core(AtaReq)).next_req = AtaReq2;
        } else {
            (*ata_req_core(AtaReq)).next_req = core::ptr::null_mut();
            (*LunExt).last_req = AtaReq;
        }
        (*ata_req_core(BestAtaReq1)).next_req = AtaReq;
        (*ata_req_core(AtaReq)).prev_req = BestAtaReq1;

        AtaReq1 = AtaReq;
        loop {
            AtaReq1 = (*ata_req_core(AtaReq1)).next_req;
            if AtaReq1.is_null() {
                break;
            }
            (*ata_req_core(AtaReq1)).ttl -= 1;
        }
    } else {
        // empty queue
        (*ata_req_core(AtaReq)).ttl = 0;
        (*ata_req_core(AtaReq)).next_req = core::ptr::null_mut();
        (*ata_req_core(AtaReq)).prev_req = core::ptr::null_mut();
        (*LunExt).first_req = AtaReq;
        (*LunExt).last_req = AtaReq;
        // __REACTOS__: do nothing here (workaround for CORE-12441 and CORE-17371)
    }
    (*LunExt).queue_depth += 1;
    (*chan).queue_depth += 1;
    (*(*chan).DeviceExtension).queue_depth += 1;
    // check if this is the 1st request in queue
    if (*chan).queue_depth == 1 {
        (*chan).cur_req = (*LunExt).first_req;
    }
} // end UniataQueueRequest()

/// Remove request from queue and get next request.
pub unsafe extern "system" fn UniataRemoveRequest(chan: PHW_CHANNEL, Srb: PSCSI_REQUEST_BLOCK) {
    if Srb.is_null() {
        return;
    }
    if chan.is_null() {
        return;
    }

    let AtaReq: PATA_REQ = (*Srb).SrbExtension as PATA_REQ;

    // GET_CDEV(Srb) -> (*Srb).TargetId
    let mut cdev: ULONG = (*Srb).TargetId as ULONG;
    let LunExt: PHW_LU_EXTENSION = (*chan).lun[cdev as usize];

    if LunExt.is_null() {
        return;
    }

    // check if queue for the device is empty
    if (*LunExt).queue_depth == 0 {
        return;
    }

    // check if request is under processing (busy)
    if (*ata_req_core(AtaReq)).ReqState == 0 {
        return;
    }

    // remove request from queue
    if !(*ata_req_core(AtaReq)).prev_req.is_null() {
        (*ata_req_core((*ata_req_core(AtaReq)).prev_req)).next_req =
            (*ata_req_core(AtaReq)).next_req;
    } else {
        (*LunExt).first_req = (*ata_req_core(AtaReq)).next_req;
    }
    if !(*ata_req_core(AtaReq)).next_req.is_null() {
        (*ata_req_core((*ata_req_core(AtaReq)).next_req)).prev_req =
            (*ata_req_core(AtaReq)).prev_req;
    } else {
        (*LunExt).last_req = (*ata_req_core(AtaReq)).prev_req;
    }
    (*LunExt).queue_depth -= 1;
    (*chan).queue_depth -= 1;
    (*(*chan).DeviceExtension).queue_depth -= 1;
    // set LastWrite flag for Lun
    (*LunExt).last_write =
        (((*ata_req_core(AtaReq)).Flags as u32 & REQ_FLAG_RW_MASK) == REQ_FLAG_WRITE) as ULONG;

    // get request from longest queue to balance load
    if (*chan).NumberLuns > 1 {
        if (*(*chan).lun[0]).queue_depth * ((*(*chan).lun[0]).LunSelectWaitCount + 1)
            > (*(*chan).lun[1]).queue_depth * ((*(*chan).lun[1]).LunSelectWaitCount + 1)
        {
            cdev = 0;
        } else {
            cdev = 1;
        }
    }
    // get next request for processing
    (*chan).cur_req = (*(*chan).lun[cdev as usize]).first_req;
    (*chan).cur_cdev = cdev;
    if (*chan).NumberLuns > 1 {
        if (*(*chan).lun[(cdev ^ 1) as usize]).queue_depth == 0 {
            (*(*chan).lun[(cdev ^ 1) as usize]).LunSelectWaitCount = 0;
        } else {
            (*(*chan).lun[(cdev ^ 1) as usize]).LunSelectWaitCount += 1;
        }
    }
    (*(*chan).lun[cdev as usize]).LunSelectWaitCount = 0;

    (*ata_req_core(AtaReq)).ReqState = REQ_STATE_NONE as UCHAR;
} // end UniataRemoveRequest()

/// Get currently processed request (from head of the queue).
pub unsafe extern "system" fn UniataGetCurRequest(chan: PHW_CHANNEL) -> PSCSI_REQUEST_BLOCK {
    if chan.is_null() || (*chan).cur_req.is_null() {
        return core::ptr::null_mut();
    }
    (*ata_req_core((*chan).cur_req)).Srb
} // end UniataGetCurRequest()

/// Get next channel to be serviced (used in simplex mode only).
pub unsafe extern "system" fn UniataGetNextChannel(mut chan: PHW_CHANNEL) -> PHW_CHANNEL {
    let mut c: ULONG = 0;
    let mut _c: ULONG;
    let deviceExtension: PHW_DEVICE_EXTENSION;
    let mut best_c: ULONG;
    let mut cost_c: ULONG;

    deviceExtension = (*chan).DeviceExtension;

    if (*deviceExtension).simplexOnly == 0 {
        return chan;
    }
    best_c = CHAN_NOT_SPECIFIED; // -1
    cost_c = 0;
    _c = 0;
    while _c < (*deviceExtension).NumberChannels {
        c = (_c + (*deviceExtension).FirstChannelToCheck) % (*deviceExtension).NumberChannels;
        chan = (*deviceExtension).chan.add(c as usize);
        if (*chan).queue_depth != 0
            && (*chan).queue_depth * ((*chan).ChannelSelectWaitCount + 1) > cost_c
        {
            best_c = c;
            cost_c = (*chan).queue_depth * ((*chan).ChannelSelectWaitCount + 1);
        }
        _c += 1;
    }
    if best_c == CHAN_NOT_SPECIFIED {
        return core::ptr::null_mut();
    }
    (*deviceExtension).FirstChannelToCheck = c;
    _c = 0;
    while _c < (*deviceExtension).NumberChannels {
        chan = (*deviceExtension).chan.add(_c as usize);
        if _c == best_c {
            (*chan).ChannelSelectWaitCount = 0;
            _c += 1;
            continue;
        }
        (*chan).ChannelSelectWaitCount += 1;
        if (*chan).queue_depth == 0 {
            (*chan).ChannelSelectWaitCount = 0;
        } else {
            (*chan).ChannelSelectWaitCount += 1;
        }
        _c += 1;
    }
    (*deviceExtension).chan.add(best_c as usize)
} // end UniataGetNextChannel()

// ===========================================================================
//  id_badblock.cpp
// ===========================================================================

/// RtlQueryRegistryValues callback. Keep `extern "system"`; the table stores a
/// `Option<extern "C" fn(...)>` ptr, so InitBadBlocks transmutes when wiring it.
pub unsafe extern "system" fn BadBlockQueryRoutine(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    _Context: PVOID,
    _EntryContext: PVOID,
) -> NTSTATUS {
    let mut cur: PSBadBlockListItem;
    let mut link: PLIST_ENTRY;
    let mut i: ULONG;

    // The ValueType should be REG_SZ; ValueData is a UNICODE string:
    //  "badblocks_start_from_lba  badblocks_end_at_lba"

    if BBListInited == 0 {
        return STATUS_SUCCESS;
    }

    let bblist_head: PLIST_ENTRY = core::ptr::addr_of_mut!(BBList);

    if (ValueType == REG_BINARY) // STRING
        && ValueLength != 0 // At least "0 0 0"
        && (ValueLength % core::mem::size_of::<SBadBlockRange>() as ULONG) == 0
    // There is free space for the record
    {
        cur = core::ptr::null_mut();
        link = BBList.Flink;
        while link != bblist_head {
            cur = list_to_item(link);
            link = (*link).Flink;
            if wcscmp((*cur).SerNumStr.as_ptr(), ValueName) == 0 {
                if !(*cur).LunExt.is_null() {
                    (*(*cur).LunExt).nBadBlocks = 0;
                    (*(*cur).LunExt).arrBadBlocks = core::ptr::null_mut();
                    (*(*cur).LunExt).bbListDescr = core::ptr::null_mut();
                    (*cur).LunExt = core::ptr::null_mut();
                }
                break;
            }
        }

        if cur.is_null() {
            cur = ExAllocatePool(NonPagedPool, core::mem::size_of::<SBadBlockListItem>() as SIZE_T)
                as PSBadBlockListItem;
            if cur.is_null() {
                return STATUS_SUCCESS;
            }
        } else {
            if !(*cur).arrBadBlocks.is_null() {
                ExFreePool((*cur).arrBadBlocks as PVOID);
                (*cur).arrBadBlocks = core::ptr::null_mut();
            }
        }
        (*cur).arrBadBlocks =
            ExAllocatePool(NonPagedPool, ValueLength as SIZE_T) as *mut SBadBlockRange;
        if (*cur).arrBadBlocks.is_null() {
            ExFreePool(cur as PVOID);
            return STATUS_SUCCESS;
        }
        RtlCopyMemory((*cur).arrBadBlocks as PVOID, ValueData as PCVOID, ValueLength as usize);
        wcsncpy((*cur).SerNumStr.as_mut_ptr(), ValueName, 127);
        (*cur).SerNumStr[127] = 0;
        (*cur).nBadBlocks = ValueLength / core::mem::size_of::<SBadBlockRange>() as ULONG;
        (*cur).LunExt = core::ptr::null_mut();
        init_list_head(core::ptr::addr_of_mut!((*cur).List));
        insert_tail_list(bblist_head, core::ptr::addr_of_mut!((*cur).List));
        i = 0;
        while i < (*cur).nBadBlocks {
            // KdPrint dropped
            i += 1;
        }
    }
    STATUS_SUCCESS
} // end BadBlockQueryRoutine()

// CONTAINING_RECORD(link, SBadBlockListItem, List): List is the first field
// (offset 0), so this is a plain cast, but use offset_of! to stay honest.
#[inline(always)]
unsafe fn list_to_item(link: PLIST_ENTRY) -> PSBadBlockListItem {
    (link as *mut u8).sub(core::mem::offset_of!(SBadBlockListItem, List)) as PSBadBlockListItem
}

#[allow(unused_assignments)]
pub unsafe extern "system" fn InitBadBlocks(LunExt: PHW_LU_EXTENSION) {
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    let mut DevSerial: [WCHAR; 128] = [0; 128];
    let mut Length: ULONG;
    let mut link: PLIST_ENTRY;
    let mut cur: PSBadBlockListItem;

    // Read badblock pairs from the registry and fill arrBadBlocks.
    // HKLM\System\CurrentControlSet\Services\UniATA\Parameters\BadBlocks

    let bblist_head: PLIST_ENTRY = core::ptr::addr_of_mut!(BBList);

    if LunExt.is_null() {
        // init
        if BBListInited == 0 {
            init_list_head(bblist_head);
            BBListInited = TRUE as BOOLEAN;
        }

        // BadBlockQueryRoutine is extern "system"; the table field is
        // Option<extern "C" fn(...)> — transmute the fn pointer (rule 6).
        let qr: unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS =
            BadBlockQueryRoutine;
        QueryTable[0].QueryRoutine = core::mem::transmute::<
            unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS,
            PRTL_QUERY_REGISTRY_ROUTINE,
        >(qr);
        QueryTable[0].Flags = RTL_QUERY_REGISTRY_REQUIRED;
        QueryTable[0].Name = core::ptr::null(); // call QueryRoutine for all values
        QueryTable[0].EntryContext = core::ptr::null_mut();
        QueryTable[0].DefaultType = REG_NONE;
        QueryTable[0].DefaultData = core::ptr::null_mut();
        QueryTable[0].DefaultLength = 0;

        // QueryTable[1] is already zeroed (EOF marker).

        // L"UniATA\\Parameters\\BadBlocks"
        const PATH: &[u16] = &[
            b'U' as u16, b'n' as u16, b'i' as u16, b'A' as u16, b'T' as u16, b'A' as u16,
            b'\\' as u16, b'P' as u16, b'a' as u16, b'r' as u16, b'a' as u16, b'm' as u16,
            b'e' as u16, b't' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'\\' as u16,
            b'B' as u16, b'a' as u16, b'd' as u16, b'B' as u16, b'l' as u16, b'o' as u16,
            b'c' as u16, b'k' as u16, b's' as u16, 0,
        ];

        let _status: NTSTATUS = RtlQueryRegistryValues(
            RTL_REGISTRY_SERVICES,
            PATH.as_ptr() as PCWSTR,
            QueryTable.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        // KdPrint dropped
    } else {
        Length = EncodeVendorStr(
            DevSerial.as_mut_ptr(),
            core::ptr::addr_of!((*LunExt).IdentifyData.ModelNumber) as PUCHAR,
            core::mem::size_of::<[USHORT; 20]>() as ULONG,
        );
        DevSerial[Length as usize] = b'-' as WCHAR;
        Length += 1;
        Length += EncodeVendorStr(
            DevSerial.as_mut_ptr().add(Length as usize),
            (*LunExt).IdentifyData.SerialNumber.as_ptr() as PUCHAR,
            core::mem::size_of_val(&(*LunExt).IdentifyData.SerialNumber) as ULONG,
        );

        (*LunExt).nBadBlocks = 0;
        (*LunExt).arrBadBlocks = core::ptr::null_mut();

        link = BBList.Flink;
        while link != bblist_head {
            cur = list_to_item(link);
            link = (*link).Flink;
            if (*cur).LunExt == LunExt {
                // deassociate BB list (by LunExt)
                (*(*cur).LunExt).nBadBlocks = 0;
                (*(*cur).LunExt).arrBadBlocks = core::ptr::null_mut();
                (*(*cur).LunExt).bbListDescr = core::ptr::null_mut();
                (*cur).LunExt = core::ptr::null_mut();
            } else if wcscmp((*cur).SerNumStr.as_ptr(), DevSerial.as_ptr()) == 0 {
                // deassociate BB list (by Serial)
                if !(*cur).LunExt.is_null() {
                    (*(*cur).LunExt).nBadBlocks = 0;
                    (*(*cur).LunExt).arrBadBlocks = core::ptr::null_mut();
                    (*(*cur).LunExt).bbListDescr = core::ptr::null_mut();
                    (*cur).LunExt = core::ptr::null_mut();
                }
            }
        }

        if ((*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE) == 0 {
            link = BBList.Flink;
            while link != bblist_head {
                cur = list_to_item(link);
                link = (*link).Flink;
                if wcscmp((*cur).SerNumStr.as_ptr(), DevSerial.as_ptr()) == 0 {
                    // found BB:List
                    (*cur).LunExt = LunExt;
                    (*LunExt).arrBadBlocks = (*cur).arrBadBlocks;
                    (*LunExt).nBadBlocks = (*cur).nBadBlocks;
                    (*LunExt).bbListDescr = cur;
                    return;
                }
            }
        }
    }
} // end InitBadBlocks()

pub unsafe extern "system" fn ForgetBadBlocks(LunExt: PHW_LU_EXTENSION) {
    if !(*LunExt).bbListDescr.is_null() {
        (*(*LunExt).bbListDescr).LunExt = core::ptr::null_mut();
        (*LunExt).nBadBlocks = 0;
        (*LunExt).arrBadBlocks = core::ptr::null_mut();
        (*LunExt).bbListDescr = core::ptr::null_mut();
    }
} // end ForgetBadBlocks()

/// C++ `bool` → BOOLEAN (returns 1/0).
pub unsafe extern "system" fn CheckIfBadBlock(
    LunExt: PHW_LU_EXTENSION,
    mut lba: ULONGLONG,
    count: ULONG,
) -> BOOLEAN {
    if (*LunExt).nBadBlocks == 0 {
        return FALSE as BOOLEAN;
    }

    let nBadBlocks: ULONG = (*LunExt).nBadBlocks;
    let mut arrBadBlocks: *mut SBadBlockRange = (*LunExt).arrBadBlocks;

    // back transform for possibly CHS'ed LBA
    lba = UniAtaCalculateLBARegsBack(LunExt, lba);

    let mut i: ULONG = 0;
    while i < nBadBlocks {
        // _SBadBlockRange is packed(4); read fields through the pointer
        // (deref of a packed field needs a copy, so use addr_of! + read).
        let lba_start = core::ptr::addr_of!((*arrBadBlocks).m_lbaStart).read_unaligned();
        let lba_end = core::ptr::addr_of!((*arrBadBlocks).m_lbaEnd).read_unaligned();
        if lba + count as ULONGLONG > lba_start && lba < lba_end {
            return TRUE as BOOLEAN;
        }
        arrBadBlocks = arrBadBlocks.add(1);
        i += 1;
    }

    FALSE as BOOLEAN
} // end CheckIfBadBlock()
