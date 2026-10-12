use ntkrnl_sys::base::*;
use crate::externs::DbgPrint;
// One declaration per external symbol: externs.rs.
use ntkrnl_sys::driver_ddk::*;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, FAT_DATA, FCB, FatData, VCB};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_RTL_BITMAP_DBG, _VCB__bindgen_ty_1};
use ntkrnl_sys::ddk::{_LARGE_MCB};

extern "system" {
    } extern "system" {
}

// ---- file-scope preamble (translated) ----
pub static mut FatDumpCurrentColumn: u32 = 0;
#[no_mangle]
pub unsafe extern "system" fn FatDump(Ptr: *mut core::ffi::c_void) {
    // TestForNull("FatDump")
    if Ptr.is_null() {
        DbgPrint(
            b"%s - Cannot dump a NULL pointer\n\0".as_ptr() as *const i8,
            b"FatDump\0".as_ptr() as *const i8,
        );
        return;
    }

    match NodeType!(Ptr) {
        // case FAT_NTC_DATA_HEADER:
        0x0500 => { // FAT_NTC_DATA_HEADER
            FatDumpDataHeader();
        }
        // case FAT_NTC_VCB:
        0x0501 => { // FAT_NTC_VCB
            FatDumpVcb((Ptr).cast());
        }
        // case FAT_NTC_FCB: case FAT_NTC_DCB: case FAT_NTC_ROOT_DCB:
        0x0502 | 0x0503 | 0x0504 => { // FAT_NTC_FCB, FAT_NTC_DCB, FAT_NTC_ROOT_DCB
            FatDumpFcb(Ptr as *const FCB);
        }
        // case FAT_NTC_CCB:
        0x0507 => { // FAT_NTC_CCB
            FatDumpCcb((Ptr).cast());
        }
        // default:
        _ => {
            DbgPrint(
                b"FatDump - Unknown Node type code %p\n\0".as_ptr() as *const i8,
                *(Ptr as *const u32),
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatDumpDataHeader() {
    let Ptr: *mut FAT_DATA = core::ptr::addr_of_mut!(FatData);
    let mut Links: *mut LIST_ENTRY;

    // TestForNull("FatDumpDataHeader")
    if Ptr.is_null() {
        DbgPrint(
            b"%s - Cannot dump a NULL pointer\n\0".as_ptr() as *const i8,
            b"FatDumpDataHeader\0".as_ptr() as *const i8,
        );
        return;
    }

    // DumpNewLine()
    {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }

    // DbgPrint("FatData@ %lx", (Ptr))
    {
        DbgPrint(
            b"FatData@ %lx\0".as_ptr() as *const i8,
            Ptr as u32, // %lx expects unsigned long, which is u32 on Windows
        );
    }

    // DumpNewLine()
    {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }

    // DumpField(NodeTypeCode)
    {
        if (FatDumpCurrentColumn + 36) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 36;

        // DumpLabel(NodeTypeCode, 18)
        {
            let name = b"NodeTypeCode";
            let mut last = 0usize;
            for i in 0..name.len() {
                if name[i] == b'.' {
                    last = i;
                }
            }
            let start = last;
            let remaining = name.len() - start;
            let copy_len = remaining.min(16);
            let mut buf: [u8; 19] = [0u8; 19];
            buf[0] = b' ';
            buf[1] = b' ';
            buf[2..2 + copy_len].copy_from_slice(&name[start..start + copy_len]);
            for i in (2 + copy_len)..18 {
                buf[i] = b' ';
            }
            buf[18] = 0;
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr() as *const i8);
        }

        {
            let val: *const core::ffi::c_void =
                ((*Ptr).NodeTypeCode as usize) as *const core::ffi::c_void;
            DbgPrint(b":%p\0".as_ptr() as *const i8, val);
            DbgPrint(b"         \0".as_ptr() as *const i8);
        }
    }

    // DumpField(NodeByteSize)
    {
        if (FatDumpCurrentColumn + 36) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 36;

        // DumpLabel(NodeByteSize, 18)
        {
            let name = b"NodeByteSize";
            let mut last = 0usize;
            for i in 0..name.len() {
                if name[i] == b'.' {
                    last = i;
                }
            }
            let start = last;
            let remaining = name.len() - start;
            let copy_len = remaining.min(16);
            let mut buf: [u8; 19] = [0u8; 19];
            buf[0] = b' ';
            buf[1] = b' ';
            buf[2..2 + copy_len].copy_from_slice(&name[start..start + copy_len]);
            for i in (2 + copy_len)..18 {
                buf[i] = b' ';
            }
            buf[18] = 0;
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr() as *const i8);
        }

        {
            let val: *const core::ffi::c_void =
                ((*Ptr).NodeByteSize as usize) as *const core::ffi::c_void;
            DbgPrint(b":%p\0".as_ptr() as *const i8, val);
            DbgPrint(b"         \0".as_ptr() as *const i8);
        }
    }

    // DumpListEntry(VcbQueue)
    {
        if (FatDumpCurrentColumn + 36) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 36;

        // DumpLabel(VcbQueue, 18)
        {
            let name = b"VcbQueue";
            let mut last = 0usize;
            for i in 0..name.len() {
                if name[i] == b'.' {
                    last = i;
                }
            }
            let start = last;
            let remaining = name.len() - start;
            let copy_len = remaining.min(16);
            let mut buf: [u8; 19] = [0u8; 19];
            buf[0] = b' ';
            buf[1] = b' ';
            buf[2..2 + copy_len].copy_from_slice(&name[start..start + copy_len]);
            for i in (2 + copy_len)..18 {
                buf[i] = b' ';
            }
            buf[18] = 0;
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr() as *const i8);
        }

        DbgPrint(
            b":%p\0".as_ptr() as *const i8,
            (*Ptr).VcbQueue.Flink as *const core::ffi::c_void,
        );
        DbgPrint(
            b":%p\0".as_ptr() as *const i8,
            (*Ptr).VcbQueue.Blink as *const core::ffi::c_void,
        );
    }

    // DumpField(DriverObject)
    {
        if (FatDumpCurrentColumn + 36) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 36;

        // DumpLabel(DriverObject, 18)
        {
            let name = b"DriverObject";
            let mut last = 0usize;
            for i in 0..name.len() {
                if name[i] == b'.' {
                    last = i;
                }
            }
            let start = last;
            let remaining = name.len() - start;
            let copy_len = remaining.min(16);
            let mut buf: [u8; 19] = [0u8; 19];
            buf[0] = b' ';
            buf[1] = b' ';
            buf[2..2 + copy_len].copy_from_slice(&name[start..start + copy_len]);
            for i in (2 + copy_len)..18 {
                buf[i] = b' ';
            }
            buf[18] = 0;
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr() as *const i8);
        }

        {
            let val: *const core::ffi::c_void =
                ((*Ptr).DriverObject as usize) as *const core::ffi::c_void;
            DbgPrint(b":%p\0".as_ptr() as *const i8, val);
            DbgPrint(b"         \0".as_ptr() as *const i8);
        }
    }

    // DumpField(OurProcess)
    {
        if (FatDumpCurrentColumn + 36) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 36;

        // DumpLabel(OurProcess, 18)
        {
            let name = b"OurProcess";
            let mut last = 0usize;
            for i in 0..name.len() {
                if name[i] == b'.' {
                    last = i;
                }
            }
            let start = last;
            let remaining = name.len() - start;
            let copy_len = remaining.min(16);
            let mut buf: [u8; 19] = [0u8; 19];
            buf[0] = b' ';
            buf[1] = b' ';
            buf[2..2 + copy_len].copy_from_slice(&name[start..start + copy_len]);
            for i in (2 + copy_len)..18 {
                buf[i] = b' ';
            }
            buf[18] = 0;
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr() as *const i8);
        }

        {
            let val: *const core::ffi::c_void =
                ((*Ptr).OurProcess as usize) as *const core::ffi::c_void;
            DbgPrint(b":%p\0".as_ptr() as *const i8, val);
            DbgPrint(b"         \0".as_ptr() as *const i8);
        }
    }

    // DumpNewLine()
    {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }

    // Loop over VcbQueue
    Links = (*Ptr).VcbQueue.Flink;
    while Links != core::ptr::addr_of_mut!((*Ptr).VcbQueue) {
        // CONTAINING_RECORD(Links, VCB, VcbLinks)
        let vcb_ptr: *mut VCB =
            (Links as *mut u8).sub(core::mem::offset_of!(VCB, VcbLinks)) as *mut VCB;
        FatDumpVcb(vcb_ptr);
        Links = (*Links).Flink;
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatDumpVcb(Ptr: *mut VCB) {
    // TestForNull("FatDumpVcb")
    if Ptr.is_null() {
        DbgPrint(
            b"%s - Cannot dump a NULL pointer\n\0".as_ptr() as *const i8,
            b"FatDumpVcb\0".as_ptr() as *const i8,
        );
        return;
    }

    // DumpNewLine
    DbgPrint(b"\n\0".as_ptr() as *const i8);
    FatDumpCurrentColumn = 1;

    // DbgPrint("Vcb@ %lx", (Ptr))
    DbgPrint(b"Vcb@ %lx\0".as_ptr() as *const i8, Ptr as usize as u32);

    // DumpNewLine
    DbgPrint(b"\n\0".as_ptr() as *const i8);
    FatDumpCurrentColumn = 1;

    // DumpField VolumeFileHeader.NodeTypeCode
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"VolumeFileHeader.NodeTypeCode";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).VolumeFileHeader.NodeTypeCode as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField VolumeFileHeader.NodeByteSize
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"VolumeFileHeader.NodeByteSize";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).VolumeFileHeader.NodeByteSize as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpListEntry VcbLinks
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"VcbLinks";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        // For token without dot, last_period remains 0, so we copy from beginning
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).VcbLinks.Flink as *const core::ffi::c_void);
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).VcbLinks.Blink as *const core::ffi::c_void);

    // DumpField TargetDeviceObject
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"TargetDeviceObject";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).TargetDeviceObject as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField Vpb
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"Vpb";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).Vpb as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField VcbState
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"VcbState";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).VcbState as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField VcbCondition
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"VcbCondition";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).VcbCondition as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField RootDcb
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"RootDcb";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).RootDcb as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField DirectAccessOpenCount
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"DirectAccessOpenCount";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).DirectAccessOpenCount as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField OpenFileCount
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"OpenFileCount";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).OpenFileCount as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField ReadOnlyCount
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"ReadOnlyCount";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).ReadOnlyCount as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, &(*Ptr).AllocationSupport as *const _VCB__bindgen_ty_1 as *const core::ffi::c_void); // address of struct
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport.RootDirectoryLbo
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport.RootDirectoryLbo";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).AllocationSupport.RootDirectoryLbo as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport.RootDirectorySize
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport.RootDirectorySize";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).AllocationSupport.RootDirectorySize as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport.FileAreaLbo
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport.FileAreaLbo";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).AllocationSupport.FileAreaLbo as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport.NumberOfClusters
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport.NumberOfClusters";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).AllocationSupport.NumberOfClusters as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport.NumberOfFreeClusters
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport.NumberOfFreeClusters";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).AllocationSupport.NumberOfFreeClusters as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport.FatIndexBitSize
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport.FatIndexBitSize";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).AllocationSupport.FatIndexBitSize as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport.LogOfBytesPerSector
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport.LogOfBytesPerSector";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).AllocationSupport.LogOfBytesPerSector as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField AllocationSupport.LogOfBytesPerCluster
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"AllocationSupport.LogOfBytesPerCluster";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).AllocationSupport.LogOfBytesPerCluster as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField DirtyFatMcb
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"DirtyFatMcb";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, &(*Ptr).DirtyFatMcb as *const _LARGE_MCB as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField FreeClusterBitMap
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"FreeClusterBitMap";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, &(*Ptr).FreeClusterBitMap as *const _RTL_BITMAP_DBG as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField VirtualVolumeFile
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"VirtualVolumeFile";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).VirtualVolumeFile as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField SectionObjectPointers.DataSectionObject
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"SectionObjectPointers.DataSectionObject";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).SectionObjectPointers.DataSectionObject as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField SectionObjectPointers.SharedCacheMap
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"SectionObjectPointers.SharedCacheMap";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).SectionObjectPointers.SharedCacheMap as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField SectionObjectPointers.ImageSectionObject
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"SectionObjectPointers.ImageSectionObject";
        let mut last_period = 0usize;
        let mut i = 0;
        while i < label.len() {
            if label[i] == b'.' {
                last_period = i;
            }
            i += 1;
        }
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = last_period;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).SectionObjectPointers.ImageSectionObject as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpField ClusterHint
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    {
        let label = b"ClusterHint";
        let mut _str = [b' '; 20];
        _str[0] = b' ';
        _str[1] = b' ';
        let mut j = 0usize;
        let mut src_idx = 0usize;
        while j < 18 && src_idx < label.len() {
            _str[2 + j] = label[src_idx];
            j += 1;
            src_idx += 1;
        }
        let mut cur_len = 2 + j;
        while cur_len < 18 {
            _str[cur_len] = b' ';
            cur_len += 1;
        }
        _str[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, _str.as_ptr());
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).ClusterHint as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // DumpNewLine
    DbgPrint(b"\n\0".as_ptr() as *const i8);
    FatDumpCurrentColumn = 1;

    // FatDumpFcb(Ptr->RootDcb)
    FatDumpFcb((*Ptr).RootDcb);
}

#[no_mangle]
pub unsafe extern "system" fn FatDumpFcb(Ptr: *const FCB) {
    // Forward-declare external functions we depend on

    // FAT_NTC_* constants from fastfat.h
    const FAT_NTC_FCB: u32      = 0x0502;
    const FAT_NTC_DCB: u32      = 0x0503;
    const FAT_NTC_ROOT_DCB: u32 = 0x0504;

    let mut Links: *mut LIST_ENTRY;

    // TestForNull("FatDumpFcb")
    if Ptr.is_null() {
        DbgPrint(b"FatDumpFcb - Cannot dump a NULL pointer\n\0".as_ptr() as *const i8);
        return;
    }

    // DumpNewLine() expansion
    {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }

    // if-else chain for node type
    let header_ptr = core::ptr::addr_of!((*Ptr).Header) as *const core::ffi::c_void;
    if NodeType!(header_ptr) == (FAT_NTC_FCB as u16) {
        DbgPrint(b"Fcb@ %lx\0".as_ptr() as *const i8, Ptr as usize);
    } else if NodeType!(header_ptr) == (FAT_NTC_DCB as u16) {
        DbgPrint(b"Dcb@ %lx\0".as_ptr() as *const i8, Ptr as usize);
    } else if NodeType!(header_ptr) == (FAT_NTC_ROOT_DCB as u16) {
        DbgPrint(b"RootDcb@ %lx\0".as_ptr() as *const i8, Ptr as usize);
    } else {
        DbgPrint(b"NonFcb NodeType @ %lx\0".as_ptr() as *const i8, Ptr as usize);
    }

    // DumpNewLine() after the if-else
    {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }

    // DumpField(Header.NodeTypeCode)
    {
        // DumpField macro expansion: label width 18
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        // DumpLabel("Header.NodeTypeCode", 18)
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"Header.NodeTypeCode"; // stringified #Field
            // pad 2 spaces
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0usize;
            for i in 0..label.len() {
                if label[i] == b'.' {
                    last_period = i;
                }
            }
            let src = &label[last_period..]; // skip the dot itself? #Label[LastPeriod] means character including dot? The code does: `strncpy(&_Str[2],&#Label[LastPeriod],Width);` where `&Label[LastPeriod]` is the substring starting at the dot. So we copy the dot plus the rest. Example: `Header.NodeTypeCode` -> last_period=6, we copy from index 6 onward = `.NodeTypeCode`. Then width 18 truncates it. So we copy from that position.
            let copy_start = last_period; // index of the dot
            let copy_len = if src.len() > 18 { 18 } else { src.len() };
            // copy into buf starting at offset 2
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            // pad remaining of Width (18) with spaces
            let filled = 2 + copy_len;
            for i in filled..18 {
                buf[i] = b' ';
            }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        // print value: DbgPrint(":%p", Ptr->Header.NodeTypeCode)
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).Header.NodeTypeCode) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(Header.NodeByteSize)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        // DumpLabel("Header.NodeByteSize", 18)
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"Header.NodeByteSize";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).Header.NodeByteSize) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpListEntry(ParentDcbLinks)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        // DumpLabel("ParentDcbLinks", 18)
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"ParentDcbLinks";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        // print Links.Flink and Links.Blink
        let links_val = core::ptr::addr_of!((*Ptr).ParentDcbLinks) as *const LIST_ENTRY;
        DbgPrint(b":%p\0".as_ptr() as *const i8, (*links_val).Flink as *const core::ffi::c_void);
        DbgPrint(b":%p\0".as_ptr() as *const i8, (*links_val).Blink as *const core::ffi::c_void);
    }

    // DumpField(ParentDcb)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        // DumpLabel("ParentDcb", 18)
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"ParentDcb";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).ParentDcb) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(Vcb)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        // DumpLabel("Vcb", 18)
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"Vcb";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).Vcb) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(FcbState)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"FcbState";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).FcbState) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(FcbCondition)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"FcbCondition";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).FcbCondition) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(UncleanCount)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"UncleanCount";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).UncleanCount) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(OpenCount)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"OpenCount";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).OpenCount) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(DirentOffsetWithinDirectory)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"DirentOffsetWithinDirectory";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).DirentOffsetWithinDirectory) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(DirentFatFlags)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"DirentFatFlags";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).DirentFatFlags) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(FullFileName.Length)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"FullFileName.Length";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).FullFileName.Length) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(FullFileName.Buffer)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"FullFileName.Buffer";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).FullFileName.Buffer as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpName(FullFileName.Buffer, 32)
    {
        if (FatDumpCurrentColumn + 18 + 32) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 32;
        // DumpLabel("FullFileName.Buffer", 18)
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"FullFileName.Buffer";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        // dump the string: for i < 32, copy from Ptr->FullFileName.Buffer[i] to _String, then print
        {
            let mut string_buf: [u8; 256] = [0u8; 256];
            let buffer_ptr = (*Ptr).FullFileName.Buffer;
            let limit = 32;
            for i in 0..limit {
                string_buf[i] = *buffer_ptr.add(i) as u8; // assumes CHAR is i8, but we want u8
            }
            string_buf[limit] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, string_buf.as_ptr());
        }
    }

    // DumpField(ShortName.Name.Oem.Length)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"ShortName.Name.Oem.Length";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).ShortName.Name.Oem.as_ref().Length) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(ShortName.Name.Oem.Buffer)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"ShortName.Name.Oem.Buffer";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).ShortName.Name.Oem.as_ref().Buffer as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(NonPaged)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"NonPaged";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).NonPaged as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(Header.AllocationSize.LowPart)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"Header.AllocationSize.__bindgen_anon_1.LowPart";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).Header.AllocationSize.__bindgen_anon_1.LowPart) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(NonPaged->SectionObjectPointers.DataSectionObject) – need to deref NonPaged
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"NonPaged->SectionObjectPointers.DataSectionObject";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        let np = (*Ptr).NonPaged;
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*np).SectionObjectPointers.DataSectionObject) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(NonPaged->SectionObjectPointers.SharedCacheMap)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"NonPaged->SectionObjectPointers.SharedCacheMap";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        let np = (*Ptr).NonPaged;
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*np).SectionObjectPointers.SharedCacheMap) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // DumpField(NonPaged->SectionObjectPointers.ImageSectionObject)
    {
        if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            FatDumpCurrentColumn = 1;
        }
        FatDumpCurrentColumn += 18 + 9 + 9;
        {
            let mut buf: [u8; 20] = [0u8; 20];
            let label = b"NonPaged->SectionObjectPointers.ImageSectionObject";
            buf[0] = b' ';
            buf[1] = b' ';
            let mut last_period = 0;
            for i in 0..label.len() {
                if label[i] == b'.' { last_period = i; }
            }
            let copy_start = last_period;
            let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
            for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                buf[2+i] = c;
            }
            let filled = 2 + copy_len;
            for i in filled..18 { buf[i] = b' '; }
            buf[18] = b'\0';
            DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
        }
        let np = (*Ptr).NonPaged;
        DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*np).SectionObjectPointers.ImageSectionObject) as *const core::ffi::c_void);
        DbgPrint(b"         \0".as_ptr() as *const i8);
    }

    // if node type is DCB or ROOT_DCB
    if ((*Ptr).Header.NodeTypeCode == (FAT_NTC_DCB) as i16) || ((*Ptr).Header.NodeTypeCode == (FAT_NTC_ROOT_DCB) as i16) {
        // DumpListEntry(Specific.Dcb.ParentDcbQueue)
        {
            if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
                DbgPrint(b"\n\0".as_ptr() as *const i8);
                FatDumpCurrentColumn = 1;
            }
            FatDumpCurrentColumn += 18 + 9 + 9;
            {
                let mut buf: [u8; 20] = [0u8; 20];
                let label = b"Specific.Dcb.ParentDcbQueue";
                buf[0] = b' ';
                buf[1] = b' ';
                let mut last_period = 0;
                for i in 0..label.len() {
                    if label[i] == b'.' { last_period = i; }
                }
                let copy_start = last_period;
                let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
                for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                    buf[2+i] = c;
                }
                let filled = 2 + copy_len;
                for i in filled..18 { buf[i] = b' '; }
                buf[18] = b'\0';
                DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
            }
            let dcb_queue = core::ptr::addr_of!((*Ptr).Specific.Dcb.as_ref().ParentDcbQueue);
            DbgPrint(b":%p\0".as_ptr() as *const i8, (*dcb_queue).Flink as *const core::ffi::c_void);
            DbgPrint(b":%p\0".as_ptr() as *const i8, (*dcb_queue).Blink as *const core::ffi::c_void);
        }

        // DumpField(Specific.Dcb.DirectoryFileOpenCount)
        {
            if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
                DbgPrint(b"\n\0".as_ptr() as *const i8);
                FatDumpCurrentColumn = 1;
            }
            FatDumpCurrentColumn += 18 + 9 + 9;
            {
                let mut buf: [u8; 20] = [0u8; 20];
                let label = b"Specific.Dcb.DirectoryFileOpenCount";
                buf[0] = b' ';
                buf[1] = b' ';
                let mut last_period = 0;
                for i in 0..label.len() {
                    if label[i] == b'.' { last_period = i; }
                }
                let copy_start = last_period;
                let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
                for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                    buf[2+i] = c;
                }
                let filled = 2 + copy_len;
                for i in filled..18 { buf[i] = b' '; }
                buf[18] = b'\0';
                DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
            }
            DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).Specific.Dcb.as_ref().DirectoryFileOpenCount) as *const core::ffi::c_void);
            DbgPrint(b"         \0".as_ptr() as *const i8);
        }

        // DumpField(Specific.Dcb.DirectoryFile)
        {
            if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
                DbgPrint(b"\n\0".as_ptr() as *const i8);
                FatDumpCurrentColumn = 1;
            }
            FatDumpCurrentColumn += 18 + 9 + 9;
            {
                let mut buf: [u8; 20] = [0u8; 20];
                let label = b"Specific.Dcb.DirectoryFile";
                buf[0] = b' ';
                buf[1] = b' ';
                let mut last_period = 0;
                for i in 0..label.len() {
                    if label[i] == b'.' { last_period = i; }
                }
                let copy_start = last_period;
                let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
                for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                    buf[2+i] = c;
                }
                let filled = 2 + copy_len;
                for i in filled..18 { buf[i] = b' '; }
                buf[18] = b'\0';
                DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
            }
            DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).Specific.Dcb.as_ref().DirectoryFile) as *const core::ffi::c_void);
            DbgPrint(b"         \0".as_ptr() as *const i8);
        }

    } else if (*Ptr).Header.NodeTypeCode == (FAT_NTC_FCB) as i16 {
        // DumpField(Header.FileSize.LowPart)
        {
            if (FatDumpCurrentColumn + 18 + 9 + 9) > 80 {
                DbgPrint(b"\n\0".as_ptr() as *const i8);
                FatDumpCurrentColumn = 1;
            }
            FatDumpCurrentColumn += 18 + 9 + 9;
            {
                let mut buf: [u8; 20] = [0u8; 20];
                let label = b"Header.FileSize.__bindgen_anon_1.LowPart";
                buf[0] = b' ';
                buf[1] = b' ';
                let mut last_period = 0;
                for i in 0..label.len() {
                    if label[i] == b'.' { last_period = i; }
                }
                let copy_start = last_period;
                let copy_len = if (label.len() - last_period) > 18 { 18 } else { label.len() - last_period };
                for (i, &c) in label[copy_start..copy_start+copy_len].iter().enumerate() {
                    buf[2+i] = c;
                }
                let filled = 2 + copy_len;
                for i in filled..18 { buf[i] = b' '; }
                buf[18] = b'\0';
                DbgPrint(b"%s\0".as_ptr() as *const i8, buf.as_ptr());
            }
            DbgPrint(b":%p\0".as_ptr() as *const i8, core::ptr::addr_of!((*Ptr).Header.FileSize.__bindgen_anon_1.LowPart) as *const core::ffi::c_void);
            DbgPrint(b"         \0".as_ptr() as *const i8);
        }

    } else {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
        DbgPrint(b"Illegal Node type code\0".as_ptr() as *const i8);
    }

    // final DumpNewLine
    {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }

    // for loop over ParentDcbQueue if node type DCB or ROOT_DCB
    if ((*Ptr).Header.NodeTypeCode == (FAT_NTC_DCB) as i16) || ((*Ptr).Header.NodeTypeCode == (FAT_NTC_ROOT_DCB) as i16) {
        let queue_head: *mut LIST_ENTRY = {
            let dcb_ref = &mut (*(Ptr as *mut FCB)).Specific.Dcb.as_mut();
            &mut dcb_ref.ParentDcbQueue as *mut LIST_ENTRY
        };
        Links = (*queue_head).Flink;
        while Links != queue_head {
            // CONTAINING_RECORD(Links, FCB, ParentDcbLinks)
            let containing_record = (Links as *mut u8)
                .sub(core::mem::offset_of!(FCB, ParentDcbLinks)) as *mut FCB;
            FatDumpFcb(containing_record as *const FCB);
            Links = (*Links).Flink;
        }
    }

    // return; (implicit)
}

#[no_mangle]
pub unsafe extern "system" fn FatDumpCcb(Ptr: *mut CCB) {
    // TestForNull expansion
    if Ptr.is_null() {
        DbgPrint(
            b"%s - Cannot dump a NULL pointer\n\0".as_ptr() as *const i8,
            b"FatDumpCcb\0".as_ptr() as *const i8,
        );
        return;
    }

    // DumpNewLine() expansion
    DbgPrint(b"\n\0".as_ptr() as *const i8);
    FatDumpCurrentColumn = 1;

    // DbgPrint("Ccb@ %lx", (Ptr))
    DbgPrint(b"Ccb@ %lx\0".as_ptr() as *const i8, Ptr as u32);

    // DumpNewLine() expansion
    DbgPrint(b"\n\0".as_ptr() as *const i8);
    FatDumpCurrentColumn = 1;

    // --- DumpField(NodeTypeCode) ---
    // column check + inc
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;

    // DumpLabel("NodeTypeCode", 18) expansion
    {
        let label = b"NodeTypeCode";
        let width = 18;
        let mut out_buf = [0u8; 19]; // extra byte for null terminator
        // write 2 leading spaces (UCHAR_SP)
        out_buf[0] = b' ';
        out_buf[1] = b' ';
        // copy label starting at offset 2, width-2 chars
        let copy_end = core::cmp::min(width - 2, label.len());
        for j in 0..copy_end {
            out_buf[2 + j] = label[j];
        }
        // pad remaining with spaces
        for j in copy_end..(width - 2) {
            out_buf[2 + j] = b' ';
        }
        // null terminate at width offset (index 18)
        out_buf[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, out_buf.as_ptr() as *const i8);
    }

    // DbgPrint(":%p", Ptr->NodeTypeCode)
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).NodeTypeCode as *const core::ffi::c_void);
    // DbgPrint("         ")
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // --- DumpField(NodeByteSize) ---
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    // DumpLabel("NodeByteSize", 18)
    {
        let label = b"NodeByteSize";
        let width = 18;
        let mut out_buf = [0u8; 19];
        out_buf[0] = b' ';
        out_buf[1] = b' ';
        let copy_end = core::cmp::min(width - 2, label.len());
        for j in 0..copy_end {
            out_buf[2 + j] = label[j];
        }
        for j in copy_end..(width - 2) {
            out_buf[2 + j] = b' ';
        }
        out_buf[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, out_buf.as_ptr() as *const i8);
    }
    DbgPrint(b":%p\0".as_ptr() as *const i8, (*Ptr).NodeByteSize as *const core::ffi::c_void);
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // --- DumpField(UnicodeQueryTemplate.Length) ---
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    // DumpLabel("UnicodeQueryTemplate.Length", 18)
    {
        let label = b"UnicodeQueryTemplate.Length";
        let width = 18;
        let mut out_buf = [0u8; 19];
        out_buf[0] = b' ';
        out_buf[1] = b' ';
        // find last period
        let mut last_period = 0;
        for (i, &ch) in label.iter().enumerate() {
            if ch == b'.' {
                last_period = i;
            }
        }
        let start = last_period + 1; // after the dot
        let copy_slice = &label[start..];
        let copy_end = core::cmp::min(width - 2, copy_slice.len());
        for j in 0..copy_end {
            out_buf[2 + j] = copy_slice[j];
        }
        for j in copy_end..(width - 2) {
            out_buf[2 + j] = b' ';
        }
        out_buf[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, out_buf.as_ptr() as *const i8);
    }
    DbgPrint(
        b":%p\0".as_ptr() as *const i8,
        core::ptr::addr_of!((*Ptr).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate.Length) as *const core::ffi::c_void,
    );
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // --- DumpName(UnicodeQueryTemplate.Buffer, 32) ---
    if FatDumpCurrentColumn + 18 + 32 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 18 + 32;
    // DumpLabel("UnicodeQueryTemplate.Buffer", 18)
    {
        let label = b"UnicodeQueryTemplate.Buffer";
        let width = 18;
        let mut out_buf = [0u8; 19];
        out_buf[0] = b' ';
        out_buf[1] = b' ';
        let mut last_period = 0;
        for (i, &ch) in label.iter().enumerate() {
            if ch == b'.' {
                last_period = i;
            }
        }
        let start = last_period + 1;
        let copy_slice = &label[start..];
        let copy_end = core::cmp::min(width - 2, copy_slice.len());
        for j in 0..copy_end {
            out_buf[2 + j] = copy_slice[j];
        }
        for j in copy_end..(width - 2) {
            out_buf[2 + j] = b' ';
        }
        out_buf[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, out_buf.as_ptr() as *const i8);
    }
    // print the first 32 bytes of the buffer
    {
        let mut _string = [0u8; 257]; // CHAR _String[256] with extra null
        let buf_ptr = (*Ptr).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate.Buffer as *const u8;
        let width = 32;
        for i in 0..width {
            _string[i] = *buf_ptr.add(i);
        }
        _string[width] = 0; // '\0'
        DbgPrint(b"%s\0".as_ptr() as *const i8, _string.as_ptr() as *const i8);
    }

    // --- DumpField(OffsetToStartSearchFrom) ---
    if FatDumpCurrentColumn + 36 > 80 {
        DbgPrint(b"\n\0".as_ptr() as *const i8);
        FatDumpCurrentColumn = 1;
    }
    FatDumpCurrentColumn += 36;
    // DumpLabel("OffsetToStartSearchFrom", 18)
    {
        let label = b"OffsetToStartSearchFrom";
        let width = 18;
        let mut out_buf = [0u8; 19];
        out_buf[0] = b' ';
        out_buf[1] = b' ';
        let copy_end = core::cmp::min(width - 2, label.len());
        for j in 0..copy_end {
            out_buf[2 + j] = label[j];
        }
        for j in copy_end..(width - 2) {
            out_buf[2 + j] = b' ';
        }
        out_buf[18] = 0;
        DbgPrint(b"%s\0".as_ptr() as *const i8, out_buf.as_ptr() as *const i8);
    }
    DbgPrint(
        b":%p\0".as_ptr() as *const i8,
        (*Ptr).__bindgen_anon_1.__bindgen_anon_1.as_ref().OffsetToStartSearchFrom as *const core::ffi::c_void,
    );
    DbgPrint(b"         \0".as_ptr() as *const i8);

    // final DumpNewLine()
    DbgPrint(b"\n\0".as_ptr() as *const i8);
    FatDumpCurrentColumn = 1;

    return;
}



// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     DumpSup.c
//
// Abstract:
//
//     This module implements a collection of data structure dump routines
//     for debugging the Fat file system
//
//
// --*/
//
// #include "fatprocs.h"
//
// #ifdef FASTFATDBG
//
// VOID FatDump(IN PVOID Ptr);
//
// VOID FatDumpDataHeader();
// VOID FatDumpVcb(IN PVCB Ptr);
// VOID FatDumpFcb(IN PFCB Ptr);
// VOID FatDumpCcb(IN PCCB Ptr);
//
// ULONG FatDumpCurrentColumn;
//
// #define DumpNewLine() {       \
//     DbgPrint("\n");            \
//     FatDumpCurrentColumn = 1; \
// }
//
// #define DumpLabel(Label,Width) {                                          \
//     size_t i, LastPeriod=0;                                                \
//     CHAR _Str[20];                                                        \
//     for(i=0;i<2;i++) { _Str[i] = UCHAR_SP;}                               \
//     for(i=0;i<strlen(#Label);i++) {if (#Label[i] == '.') LastPeriod = i;} \
//     strncpy(&_Str[2],&#Label[LastPeriod],Width);                          \
//     for(i=strlen(_Str);i<Width;i++) {_Str[i] = UCHAR_SP;}                 \
//     _Str[Width] = '\0';                                                   \
//     DbgPrint("%s", _Str);                                                  \
// }
//
// #define DumpField(Field) {                                         \
//     if ((FatDumpCurrentColumn + 18 + 9 + 9) > 80) {DumpNewLine();} \
//     FatDumpCurrentColumn += 18 + 9 + 9;                            \
//     DumpLabel(Field,18);                                           \
//     DbgPrint(":%p", Ptr->Field);                                  \
//     DbgPrint("         ");                                          \
// }
//
// #define DumpListEntry(Links) {                                     \
//     if ((FatDumpCurrentColumn + 18 + 9 + 9) > 80) {DumpNewLine();} \
//     FatDumpCurrentColumn += 18 + 9 + 9;                            \
//     DumpLabel(Links,18);                                           \
//     DbgPrint(":%p", Ptr->Links.Flink);                            \
//     DbgPrint(":%p", Ptr->Links.Blink);                            \
// }
//
// #define DumpName(Field,Width) {                                    \
//     ULONG i;                                                       \
//     CHAR _String[256];                                             \
//     if ((FatDumpCurrentColumn + 18 + Width) > 80) {DumpNewLine();} \
//     FatDumpCurrentColumn += 18 + Width;                            \
//     DumpLabel(Field,18);                                           \
//     for(i=0;i<Width;i++) {_String[i] = (CHAR)Ptr->Field[i];}             \
//     _String[Width] = '\0';                                         \
//     DbgPrint("%s", _String);                                        \
// }
//
// #define TestForNull(Name) {                                 \
//     if (Ptr == NULL) {                                      \
//         DbgPrint("%s - Cannot dump a NULL pointer\n", Name); \
//         return;                                             \
//     }                                                       \
// }
//
//
// VOID
// FatDump (
//     IN PVOID Ptr
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines the type of internal record reference by ptr and
//     calls the appropriate dump routine.
//
// Arguments:
//
//     Ptr - Supplies the pointer to the record to be dumped
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     TestForNull("FatDump");
//
//     switch (NodeType(Ptr)) {
//
//     case FAT_NTC_DATA_HEADER:
//
//         FatDumpDataHeader();
//         break;
//
//     case FAT_NTC_VCB:
//
//         FatDumpVcb(Ptr);
//         break;
//
//     case FAT_NTC_FCB:
//     case FAT_NTC_DCB:
//     case FAT_NTC_ROOT_DCB:
//
//         FatDumpFcb(Ptr);
//         break;
//
//     case FAT_NTC_CCB:
//
//         FatDumpCcb(Ptr);
//         break;
//
//     default :
//
//         DbgPrint("FatDump - Unknown Node type code %p\n", *((PNODE_TYPE_CODE)(Ptr)));
//         break;
//     }
//
//     return;
// }
//
//
// VOID
// FatDumpDataHeader (
//     )
//
// /*++
//
// Routine Description:
//
//     Dump the top data structures and all Device structures
//
// Arguments:
//
//     None
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PFAT_DATA Ptr;
//     PLIST_ENTRY Links;
//
//     Ptr = &FatData;
//
//     TestForNull("FatDumpDataHeader");
//
//     DumpNewLine();
//     DbgPrint("FatData@ %lx", (Ptr));
//     DumpNewLine();
//
//     DumpField           (NodeTypeCode);
//     DumpField           (NodeByteSize);
//     DumpListEntry       (VcbQueue);
//     DumpField           (DriverObject);
//     DumpField           (OurProcess);
//     DumpNewLine();
//
//     for (Links = Ptr->VcbQueue.Flink;
//          Links != &Ptr->VcbQueue;
//          Links = Links->Flink) {
//
//         FatDumpVcb(CONTAINING_RECORD(Links, VCB, VcbLinks));
//     }
//
//     return;
// }
//
//
// VOID
// FatDumpVcb (
//     IN PVCB Ptr
//     )
//
// /*++
//
// Routine Description:
//
//     Dump an Device structure, its Fcb queue amd direct access queue.
//
// Arguments:
//
//     Ptr - Supplies the Device record to be dumped
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     TestForNull("FatDumpVcb");
//
//     DumpNewLine();
//     DbgPrint("Vcb@ %lx", (Ptr));
//     DumpNewLine();
//
//     DumpField           (VolumeFileHeader.NodeTypeCode);
//     DumpField           (VolumeFileHeader.NodeByteSize);
//     DumpListEntry       (VcbLinks);
//     DumpField           (TargetDeviceObject);
//     DumpField           (Vpb);
//     DumpField           (VcbState);
//     DumpField           (VcbCondition);
//     DumpField           (RootDcb);
//     DumpField           (DirectAccessOpenCount);
//     DumpField           (OpenFileCount);
//     DumpField           (ReadOnlyCount);
//     DumpField           (AllocationSupport);
//     DumpField           (AllocationSupport.RootDirectoryLbo);
//     DumpField           (AllocationSupport.RootDirectorySize);
//     DumpField           (AllocationSupport.FileAreaLbo);
//     DumpField           (AllocationSupport.NumberOfClusters);
//     DumpField           (AllocationSupport.NumberOfFreeClusters);
//     DumpField           (AllocationSupport.FatIndexBitSize);
//     DumpField           (AllocationSupport.LogOfBytesPerSector);
//     DumpField           (AllocationSupport.LogOfBytesPerCluster);
//     DumpField           (DirtyFatMcb);
//     DumpField           (FreeClusterBitMap);
//     DumpField           (VirtualVolumeFile);
//     DumpField           (SectionObjectPointers.DataSectionObject);
//     DumpField           (SectionObjectPointers.SharedCacheMap);
//     DumpField           (SectionObjectPointers.ImageSectionObject);
//     DumpField           (ClusterHint);
//     DumpNewLine();
//
//     FatDumpFcb(Ptr->RootDcb);
//
//     return;
// }
//
//
// VOID
// FatDumpFcb (
//     IN PFCB Ptr
//     )
//
// /*++
//
// Routine Description:
//
//     Dump an Fcb structure, its various queues
//
// Arguments:
//
//     Ptr - Supplies the Fcb record to be dumped
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PLIST_ENTRY Links;
//
//     TestForNull("FatDumpFcb");
//
//     DumpNewLine();
//     if      (NodeType(&Ptr->Header) == FAT_NTC_FCB)      {DbgPrint("Fcb@ %lx", (Ptr));}
//     else if (NodeType(&Ptr->Header) == FAT_NTC_DCB)      {DbgPrint("Dcb@ %lx", (Ptr));}
//     else if (NodeType(&Ptr->Header) == FAT_NTC_ROOT_DCB) {DbgPrint("RootDcb@ %lx", (Ptr));}
//     else {DbgPrint("NonFcb NodeType @ %lx", (Ptr));}
//     DumpNewLine();
//
//     DumpField           (Header.NodeTypeCode);
//     DumpField           (Header.NodeByteSize);
//     DumpListEntry       (ParentDcbLinks);
//     DumpField           (ParentDcb);
//     DumpField           (Vcb);
//     DumpField           (FcbState);
//     DumpField           (FcbCondition);
//     DumpField           (UncleanCount);
//     DumpField           (OpenCount);
//     DumpField           (DirentOffsetWithinDirectory);
//     DumpField           (DirentFatFlags);
//     DumpField           (FullFileName.Length);
//     DumpField           (FullFileName.Buffer);
//     DumpName            (FullFileName.Buffer, 32);
//     DumpField           (ShortName.Name.Oem.Length);
//     DumpField           (ShortName.Name.Oem.Buffer);
//     DumpField           (NonPaged);
//     DumpField           (Header.AllocationSize.LowPart);
//     DumpField           (NonPaged->SectionObjectPointers.DataSectionObject);
//     DumpField           (NonPaged->SectionObjectPointers.SharedCacheMap);
//     DumpField           (NonPaged->SectionObjectPointers.ImageSectionObject);
//
//     if ((Ptr->Header.NodeTypeCode == FAT_NTC_DCB) ||
//         (Ptr->Header.NodeTypeCode == FAT_NTC_ROOT_DCB)) {
//
//         DumpListEntry   (Specific.Dcb.ParentDcbQueue);
//         DumpField       (Specific.Dcb.DirectoryFileOpenCount);
//         DumpField       (Specific.Dcb.DirectoryFile);
//
//     } else if (Ptr->Header.NodeTypeCode == FAT_NTC_FCB) {
//
//         DumpField       (Header.FileSize.LowPart);
//
//     } else {
//
//         DumpNewLine();
//         DbgPrint("Illegal Node type code");
//
//     }
//     DumpNewLine();
//
//     if ((Ptr->Header.NodeTypeCode == FAT_NTC_DCB) ||
//         (Ptr->Header.NodeTypeCode == FAT_NTC_ROOT_DCB)) {
//
//         for (Links = Ptr->Specific.Dcb.ParentDcbQueue.Flink;
//              Links != &Ptr->Specific.Dcb.ParentDcbQueue;
//              Links = Links->Flink) {
//
//             FatDumpFcb(CONTAINING_RECORD(Links, FCB, ParentDcbLinks));
//         }
//     }
//
//     return;
// }
//
//
// VOID
// FatDumpCcb (
//     IN PCCB Ptr
//     )
//
// /*++
//
// Routine Description:
//
//     Dump a Ccb structure
//
// Arguments:
//
//     Ptr - Supplies the Ccb record to be dumped
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     TestForNull("FatDumpCcb");
//
//     DumpNewLine();
//     DbgPrint("Ccb@ %lx", (Ptr));
//     DumpNewLine();
//
//     DumpField           (NodeTypeCode);
//     DumpField           (NodeByteSize);
//     DumpField           (UnicodeQueryTemplate.Length);
//     DumpName            (UnicodeQueryTemplate.Buffer, 32);
//     DumpField           (OffsetToStartSearchFrom);
//     DumpNewLine();
//
//     return;
// }
//
// #endif // FASTFATDBG
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
