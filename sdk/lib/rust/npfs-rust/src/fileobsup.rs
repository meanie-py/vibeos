//! fileobsup.rs — fileobsup.c: Pipes File Object Support (FsContext/FsContext2 encode + decode).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ke::KeBugCheckEx;

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// File ID number for NPFS bugchecking support
// npfs.h: #define NPFS_BUGCHECK_FILE_ID (NPFS_BUGCHECK_FILEOBSUP)
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_FILEOBSUP;

// xdk/iotypes.h: #define FO_NAMED_PIPE 0x00000080
const FO_NAMED_PIPE: ULONG = 0x0000_0080;

#[no_mangle]
pub unsafe extern "system" fn NpDecodeFileObject(
    FileObject: PFILE_OBJECT,
    PrimaryContext: *mut PVOID, // OPTIONAL
    Ccb: *mut PNP_CCB,
    NamedPipeEnd: *mut ULONG,   // OPTIONAL
) -> NODE_TYPE_CODE {
    let mut Context: ULONG_PTR;
    let mut Node: PNP_CCB;
    // PAGED_CODE();

    Context = (*FileObject).FsContext as ULONG_PTR;
    if (Context != 0) && (Context != 1) {
        if !NamedPipeEnd.is_null() {
            *NamedPipeEnd = (Context & 1) as ULONG;
        }

        Node = (Context & !1) as PVOID as PNP_CCB;

        match (*Node).NodeType {
            NPFS_NTC_VCB => {
                *Ccb = null_mut();
                if !PrimaryContext.is_null() {
                    *PrimaryContext = Node as PVOID;
                }
                return NPFS_NTC_VCB;
            }

            NPFS_NTC_ROOT_DCB => {
                *Ccb = (*FileObject).FsContext2 as PNP_CCB;
                if !PrimaryContext.is_null() {
                    *PrimaryContext = Node as PVOID;
                }
                return NPFS_NTC_ROOT_DCB;
            }

            NPFS_NTC_CCB => {
                *Ccb = Node;
                if !PrimaryContext.is_null() {
                    *PrimaryContext = (*Node).Fcb as PVOID;
                }
                return NPFS_NTC_CCB;
            }

            _ => {
                // NpBugCheck(Node->NodeType, 0, 0); — fileobsup.c:54
                KeBugCheckEx(
                    NPFS_FILE_SYSTEM,
                    ((NPFS_BUGCHECK_FILE_ID << 16) | 54u32) as _,
                    (*Node).NodeType as _,
                    0 as _,
                    0 as _,
                );
            }
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn NpSetFileObject(
    FileObject: PFILE_OBJECT,
    mut PrimaryContext: PVOID,
    Ccb: PVOID,
    NamedPipeEnd: ULONG,
) {
    let mut FileIsPipe: BOOLEAN;
    // PAGED_CODE();

    if FileObject.is_null() {
        return;
    }

    if (!PrimaryContext.is_null()) && ((*(PrimaryContext as PNP_CCB)).NodeType == NPFS_NTC_CCB) {
        FileIsPipe = TRUE;
        if NamedPipeEnd == FILE_PIPE_SERVER_END {
            PrimaryContext = ((PrimaryContext as ULONG_PTR) | 1) as PVOID;
        }
    } else {
        FileIsPipe = FALSE;
    }

    (*FileObject).FsContext = PrimaryContext;
    (*FileObject).FsContext2 = Ccb;
    (*FileObject).PrivateCacheMap = 1 as PVOID;
    if FileIsPipe != 0 {
        (*FileObject).Flags |= FO_NAMED_PIPE;
    }
}

/* EOF */
