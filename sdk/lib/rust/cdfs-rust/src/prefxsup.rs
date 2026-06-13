use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{EqualTo, GreaterThan, LessThan, PRTL_SPLAY_LINKS, PUNICODE_STRING, RtlDelete, RtlSplay};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CD_NAME, FCB, FSRTL_COMPARISON_RESULT, IRP_CONTEXT, NAME_LINK, PREFIX_ENTRY, VCB};
use ntkrnl_sys::ddk::{RTL_SPLAY_LINKS, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CDFS_BUG_CHECK_PREFXSUP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{RtlInitializeSplayLinks, RtlInsertAsLeftChild, RtlInsertAsRightChild};

extern "system" {
    fn PAGED_CODE();
    fn UNREFERENCED_PARAMETER(p: *mut core::ffi::c_void);
    fn FlagOn(Flags: u32, Mask: u32) -> i32;
    fn ClearFlag(Flags: *mut u32, Mask: u32);
    fn CdFreePool(Pool: *mut *mut u16);
    fn SafeNodeType(fcb: *mut FCB) -> u32;
    fn CdDissectName(irp_context: *mut IRP_CONTEXT, local_remaining_name: *mut _UNICODE_STRING, final_name: *mut _UNICODE_STRING);
    fn CdLockVcb(irp_context: *mut IRP_CONTEXT, vcb: *mut VCB);
    fn CdUnlockVcb(irp_context: *mut IRP_CONTEXT, vcb: *mut VCB);
    fn CdReleaseFcb(irp_context: *mut IRP_CONTEXT, fcb: *mut FCB);
    fn CdAcquireFcbExclusive(irp_context: *mut IRP_CONTEXT, fcb: *mut FCB, wait: i32) -> i32; // returns BOOL #[no_mangle] fn CdAcquireFcbExclusive(irp_context: *mut IRP_CONTEXT, fcb: *mut FCB, wait: i32) -> i32; // returns BOOL #[no_mangle] fn CdAcquireFcbExclusive(irp_context: *mut IRP_CONTEXT, fcb: *mut FCB, wait: i32) -> i32; // returns BOOL #[no_mangle] fn CdAcquireFcbExclusive(irp_context: *mut IRP_CONTEXT, fcb: *mut FCB, wait: i32) -> i32; // returns BOOL pub unsafe extern "C" fn CdFindNameLink(irp_context: *mut IRP_CONTEXT, root_node: *mut *mut RTL_SPLAY_LINKS, name: *const _UNICODE_STRING) -> *mut NAME_LINK;
    fn Add2Ptr(p: *mut core::ffi::c_void, offset: u32) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_PREFXSUP;

unsafe extern "system" fn RtlLeftChild(Links: PRTL_SPLAY_LINKS) -> PRTL_SPLAY_LINKS;
unsafe extern "system" fn RtlRightChild(Links: PRTL_SPLAY_LINKS) -> PRTL_SPLAY_LINKS;
unsafe extern "system" fn CdFullCompareNames(IrpContext: *mut IRP_CONTEXT, Name1: *const _UNICODE_STRING, Name2: *const _UNICODE_STRING) -> i32;

// Constants (placeholders – actual values may need to be defined elsewhere)
const CDFS_NTC_FCB_INDEX: u32 = 0x206;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000001;
const STATUS_CANT_WAIT: i32 = 0xC00000098u32 as i32;

// Extern declarations for unknown functions

// Constants approximated from original source
const CdPagedPool: u32 = 1; // PagedPool
const TAG_PREFIX_ENTRY: u32 = 0; // original value unspecified
const TAG_PREFIX_NAME: u32 = 0; // original value unspecified
const PREFIX_FLAG_EXACT_CASE_IN_TREE: u32 = 0x0002;
const PREFIX_FLAG_IGNORE_CASE_IN_TREE: u32 = 0x0004;
const BYTE_COUNT_EMBEDDED_NAME: u32 = 8; // approximate size of embedded array
#[no_mangle]
pub unsafe extern "C" fn CdInsertPrefix(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Name: *mut CD_NAME,
    IgnoreCase: u8, // BOOLEAN
    ShortNameMatch: u8, // BOOLEAN
    ParentFcb: *mut FCB,
) {
    unsafe {
        PAGED_CODE();

        let mut PrefixEntry: *mut PREFIX_ENTRY = core::ptr::addr_of_mut!((*Fcb).FileNamePrefix);
        
        if ShortNameMatch != 0 {
            if (*Fcb).ShortNamePrefix.is_null() {
                let alloc = ExAllocatePoolWithTag(
                    CdPagedPool,
                    core::mem::size_of::<PREFIX_ENTRY>() as u32,
                    TAG_PREFIX_ENTRY,
                ) as *mut PREFIX_ENTRY;
                if alloc.is_null() {
                    return;
                }
                (*Fcb).ShortNamePrefix = alloc;
                RtlZeroMemory(alloc as *mut core::ffi::c_void, core::mem::size_of::<PREFIX_ENTRY>());
            }
            PrefixEntry = (*Fcb).ShortNamePrefix;
        }

        let PrefixFlags: u32;
        let mut NameLink: *mut NAME_LINK;
        let mut TreeRoot: *mut *mut RTL_SPLAY_LINKS;

        if IgnoreCase != 0 {
            PrefixFlags = PREFIX_FLAG_IGNORE_CASE_IN_TREE;
            NameLink = core::ptr::addr_of_mut!((*PrefixEntry).IgnoreCaseName);
            TreeRoot = core::ptr::addr_of_mut!((*ParentFcb).IgnoreCaseRoot);
        } else {
            PrefixFlags = PREFIX_FLAG_EXACT_CASE_IN_TREE;
            NameLink = core::ptr::addr_of_mut!((*PrefixEntry).ExactCaseName);
            TreeRoot = core::ptr::addr_of_mut!((*ParentFcb).ExactCaseRoot);
        }

        // Check if neither name is in the tree
        if ((*PrefixEntry).PrefixFlags & (PREFIX_FLAG_EXACT_CASE_IN_TREE | PREFIX_FLAG_IGNORE_CASE_IN_TREE)) == 0 {
            let mut NameBuffer = core::ptr::addr_of_mut!((*PrefixEntry).FileNameBuffer) as *mut u16;

            if (*Name).FileName.Length > BYTE_COUNT_EMBEDDED_NAME as u16 {
                NameBuffer = ExAllocatePoolWithTag(
                    CdPagedPool,
                    ((*Name).FileName.Length as u32) * 2,
                    TAG_PREFIX_NAME,
                ) as *mut u16;
                if NameBuffer.is_null() {
                    return;
                }
            }

            // Split the buffer for exact and ignore case entries
            (*PrefixEntry).ExactCaseName.FileName.Buffer = NameBuffer;
            let ign_buf = Add2Ptr(
                NameBuffer as *mut core::ffi::c_void,
                (*Name).FileName.Length as u32,
            ) as *mut u16;
            (*PrefixEntry).IgnoreCaseName.FileName.Buffer = ign_buf;

            let len = (*Name).FileName.Length;
            (*PrefixEntry).ExactCaseName.FileName.MaximumLength = len;
            (*PrefixEntry).ExactCaseName.FileName.Length = len;
            (*PrefixEntry).IgnoreCaseName.FileName.MaximumLength = len;
            (*PrefixEntry).IgnoreCaseName.FileName.Length = len;
        }

        // Only insert if not already present
        if ((*PrefixEntry).PrefixFlags & PrefixFlags) == 0 {
            // Copy the name into the prefix entry
            core::ptr::copy_nonoverlapping(
                (*Name).FileName.Buffer as *const u8,
                (*NameLink).FileName.Buffer as *mut u8,
                (*Name).FileName.Length as usize,
            );

            CdInsertNameLink(IrpContext, TreeRoot, NameLink);

            (*PrefixEntry).Fcb = Fcb;
            (*PrefixEntry).PrefixFlags |= PrefixFlags;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdRemovePrefix(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) {
    PAGED_CODE();

    UNREFERENCED_PARAMETER(IrpContext as *mut core::ffi::c_void);

    let short_name_prefix = (*Fcb).ShortNamePrefix;

    if !short_name_prefix.is_null() {
        if FlagOn(
            (*short_name_prefix).PrefixFlags,
            PREFIX_FLAG_IGNORE_CASE_IN_TREE,
        ) != 0
        {
            (*(*Fcb).ParentFcb).IgnoreCaseRoot = ntkrnl_sys::ddk::RtlDelete(core::ptr::addr_of_mut!(
                (*short_name_prefix).IgnoreCaseName.Links
            ));
        }

        if FlagOn(
            (*short_name_prefix).PrefixFlags,
            PREFIX_FLAG_EXACT_CASE_IN_TREE,
        ) != 0
        {
            (*(*Fcb).ParentFcb).ExactCaseRoot = ntkrnl_sys::ddk::RtlDelete(core::ptr::addr_of_mut!(
                (*short_name_prefix).ExactCaseName.Links
            ));
        }

        ClearFlag(
            core::ptr::addr_of_mut!((*short_name_prefix).PrefixFlags),
            PREFIX_FLAG_IGNORE_CASE_IN_TREE | PREFIX_FLAG_EXACT_CASE_IN_TREE,
        );
    }

    let file_name_prefix = &(*Fcb).FileNamePrefix;

    if FlagOn(
        file_name_prefix.PrefixFlags,
        PREFIX_FLAG_IGNORE_CASE_IN_TREE,
    ) != 0
    {
        (*(*Fcb).ParentFcb).IgnoreCaseRoot = ntkrnl_sys::ddk::RtlDelete(core::ptr::addr_of_mut!(
            file_name_prefix.IgnoreCaseName.Links
        ));
    }

    if FlagOn(
        file_name_prefix.PrefixFlags,
        PREFIX_FLAG_EXACT_CASE_IN_TREE,
    ) != 0
    {
        (*(*Fcb).ParentFcb).ExactCaseRoot = ntkrnl_sys::ddk::RtlDelete(core::ptr::addr_of_mut!(
            file_name_prefix.ExactCaseName.Links
        ));
    }

    ClearFlag(
        core::ptr::addr_of_mut!(file_name_prefix.PrefixFlags) as *mut u32,
        PREFIX_FLAG_IGNORE_CASE_IN_TREE | PREFIX_FLAG_EXACT_CASE_IN_TREE,
    );

    if (*Fcb).FileNamePrefix.ExactCaseName.FileName.Buffer
        != (core::ptr::addr_of!((*Fcb).FileNamePrefix.FileNameBuffer) as *mut u16)
        && !(*Fcb).FileNamePrefix.ExactCaseName.FileName.Buffer.is_null()
    {
        CdFreePool(core::ptr::addr_of_mut!(
            (*Fcb).FileNamePrefix.ExactCaseName.FileName.Buffer
        ));
        (*Fcb).FileNamePrefix.ExactCaseName.FileName.Buffer = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdFindPrefix(
    irp_context: *mut IRP_CONTEXT,
    current_fcb: *mut *mut FCB,
    remaining_name: *mut _UNICODE_STRING,
    ignore_case: i32,
) {
    // PAGED_CODE();

    // Make a local copy of the input strings.
    let mut local_remaining_name: _UNICODE_STRING = core::ptr::read(remaining_name);

    let mut final_name: _UNICODE_STRING = core::mem::zeroed();
    let mut name_link: *mut NAME_LINK;
    let mut prefix_entry: *mut PREFIX_ENTRY;

    loop {
        if local_remaining_name.Length == 0
            || SafeNodeType(*current_fcb) != CDFS_NTC_FCB_INDEX
        {
            return;
        }

        CdDissectName(
            irp_context,
            &mut local_remaining_name as *mut _UNICODE_STRING,
            &mut final_name as *mut _UNICODE_STRING,
        );

        if ignore_case != 0 {
            name_link = CdFindNameLink(
                irp_context,
                core::ptr::addr_of_mut!((*(*current_fcb)).IgnoreCaseRoot),
                &final_name as *const _UNICODE_STRING,
            );
            prefix_entry = if !name_link.is_null() {
                let offset = core::mem::offset_of!(PREFIX_ENTRY, IgnoreCaseName);
                (name_link as *const u8).sub(offset) as *mut PREFIX_ENTRY
            } else {
                core::ptr::null_mut()
            };
        } else {
            name_link = CdFindNameLink(
                irp_context,
                core::ptr::addr_of_mut!((*(*current_fcb)).ExactCaseRoot),
                &final_name as *const _UNICODE_STRING,
            );
            prefix_entry = if !name_link.is_null() {
                let offset = core::mem::offset_of!(PREFIX_ENTRY, ExactCaseName);
                (name_link as *const u8).sub(offset) as *mut PREFIX_ENTRY
            } else {
                core::ptr::null_mut()
            };
        }

        if name_link.is_null() {
            return;
        }

        if ignore_case != 0 {
            core::ptr::copy_nonoverlapping(
                (*prefix_entry).ExactCaseName.FileName.Buffer,
                final_name.Buffer,
                (*prefix_entry).ExactCaseName.FileName.Length as usize,
            );
        }

        core::ptr::write(remaining_name, local_remaining_name);

        if CdAcquireFcbExclusive(irp_context, (*prefix_entry).Fcb, 1) == 0 {
            if FlagOn((*irp_context).Flags, IRP_CONTEXT_FLAG_WAIT) == 0 {
                CdRaiseStatus(irp_context, STATUS_CANT_WAIT);
            }
            CdLockVcb(irp_context, (*irp_context).Vcb);
            (*(*prefix_entry).Fcb).FcbReference += 1;
            CdUnlockVcb(irp_context, (*irp_context).Vcb);

            CdReleaseFcb(irp_context, *(*current_fcb));
            CdAcquireFcbExclusive(irp_context, (*prefix_entry).Fcb, 0);

            CdLockVcb(irp_context, (*irp_context).Vcb);
            (*(*prefix_entry).Fcb).FcbReference -= 1;
            CdUnlockVcb(irp_context, (*irp_context).Vcb);
        } else {
            CdReleaseFcb(irp_context, *(*current_fcb));
        }

        *current_fcb = (*prefix_entry).Fcb;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdInsertNameLink(
    IrpContext: *mut IRP_CONTEXT,
    RootNode: *mut PRTL_SPLAY_LINKS,
    NameLink: *mut NAME_LINK,
) -> i32 {
    let mut Comparison: FSRTL_COMPARISON_RESULT;
    let mut Node: *mut NAME_LINK;

    PAGED_CODE();

    core::ptr::addr_of_mut!((*NameLink).Links).as_mut().unwrap(); // constraining MIR type, but effectively just getting a pointer
    RtlInitializeSplayLinks(core::ptr::addr_of_mut!((*NameLink).Links));

    // If we are the first entry in the tree, just become the root.
    if (*RootNode).is_null() {
        *RootNode = core::ptr::addr_of!((*NameLink).Links) as PRTL_SPLAY_LINKS;
        return 1; // TRUE
    }

    // CONTAINING_RECORD(*RootNode, NAME_LINK, Links)
    Node = (((*RootNode) as *const u8)
        .sub(core::mem::offset_of!(NAME_LINK, Links)))
        as *mut NAME_LINK;

    loop {
        // Compare the prefix in the tree with the prefix we want to insert.
        Comparison = CdFullCompareNames(
            IrpContext,
            core::ptr::addr_of!((*Node).FileName) as PUNICODE_STRING,
            core::ptr::addr_of!((*NameLink).FileName) as PUNICODE_STRING,
        );

        // If we found the entry, return immediately.
        if Comparison == EqualTo {
            return 0; // FALSE
        }

        // If the tree prefix is greater than the new prefix then
        // we go down the left subtree
        if Comparison == GreaterThan {
            if RtlLeftChild(core::ptr::addr_of!((*Node).Links)).is_null() {
                // There isn't a left child so we insert ourselves as the new left child
                RtlInsertAsLeftChild(
                    core::ptr::addr_mut!((*Node).Links),
                    core::ptr::addr_mut!((*NameLink).Links),
                );
                break;
            } else {
                // There is a left child so simply go down that path
                Node = ((RtlLeftChild(core::ptr::addr_of!((*Node).Links)) as *const u8)
                    .sub(core::mem::offset_of!(NAME_LINK, Links)))
                    as *mut NAME_LINK;
            }
        } else {
            // The tree prefix is either less than or a proper prefix of the new string.
            // We treat both cases as less than when we do insert. So we want to go down
            // the right subtree, first check to see if we have a right subtree.
            if RtlRightChild(core::ptr::addr_of!((*Node).Links)).is_null() {
                // These isn't a right child so we insert ourselves as the new right child
                RtlInsertAsRightChild(
                    core::ptr::addr_mut!((*Node).Links),
                    core::ptr::addr_mut!((*NameLink).Links),
                );
                break;
            } else {
                // There is a right child so simply go down that path
                Node = ((RtlRightChild(core::ptr::addr_of!((*Node).Links)) as *const u8)
                    .sub(core::mem::offset_of!(NAME_LINK, Links)))
                    as *mut NAME_LINK;
            }
        }
    }

    return 1; // TRUE
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
//     PrefxSup.c
//
// Abstract:
//
//     This module implements the Cdfs Prefix support routines
//
//
// --*/
//
// #include "cdprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (CDFS_BUG_CHECK_PREFXSUP)
//
// //
// //  Local support routines.
// //
//
// PNAME_LINK
// CdFindNameLink (
//      PIRP_CONTEXT IrpContext,
//      PRTL_SPLAY_LINKS *RootNode,
//      PUNICODE_STRING Name
//     );
//
// BOOLEAN
// CdInsertNameLink (
//      PIRP_CONTEXT IrpContext,
//      PRTL_SPLAY_LINKS *RootNode,
//      PNAME_LINK NameLink
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdFindNameLink)
// #pragma alloc_text(PAGE, CdFindPrefix)
// #pragma alloc_text(PAGE, CdInsertNameLink)
// #pragma alloc_text(PAGE, CdInsertPrefix)
// #pragma alloc_text(PAGE, CdRemovePrefix)
// #endif
//
//
// VOID
// CdInsertPrefix (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PCD_NAME Name,
//      BOOLEAN IgnoreCase,
//      BOOLEAN ShortNameMatch,
//      PFCB ParentFcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine inserts the names in the given Lcb into the links for the
//     parent.
//
// Arguments:
//
//     Fcb - This is the Fcb whose name is being inserted into the tree.
//
//     Name - This is the name for the component.  The IgnoreCase flag tells
//         us which entry this belongs to.
//
//     IgnoreCase - Indicates if we should insert the case-insensitive name.
//
//     ShortNameMatch - Indicates if this is the short name.
//
//     ParentFcb - This is the ParentFcb.  The prefix tree is attached to this.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG PrefixFlags;
//     PNAME_LINK NameLink;
//     PPREFIX_ENTRY PrefixEntry;
//     PRTL_SPLAY_LINKS *TreeRoot;
//
//     PWCHAR NameBuffer;
//
//     PAGED_CODE();
//
//     //
//     //  Check if we need to allocate a prefix entry for the short name.
//     //  If we can't allocate one then fail quietly.  We don't have to
//     //  insert the name.
//     //
//
//     PrefixEntry = &Fcb->FileNamePrefix;
//
//     if (ShortNameMatch) {
//
//         if (Fcb->ShortNamePrefix == NULL) {
//
//             Fcb->ShortNamePrefix = ExAllocatePoolWithTag( CdPagedPool,
//                                                           sizeof( PREFIX_ENTRY ),
//                                                           TAG_PREFIX_ENTRY );
//
//             if (Fcb->ShortNamePrefix == NULL) { return; }
//
//             RtlZeroMemory( Fcb->ShortNamePrefix, sizeof( PREFIX_ENTRY ));
//         }
//
//         PrefixEntry = Fcb->ShortNamePrefix;
//     }
//
//     //
//     //  Capture the local variables for the separate cases.
//     //
//
//     if (IgnoreCase) {
//
//         PrefixFlags = PREFIX_FLAG_IGNORE_CASE_IN_TREE;
//         NameLink = &PrefixEntry->IgnoreCaseName;
//         TreeRoot = &ParentFcb->IgnoreCaseRoot;
//
//     } else {
//
//         PrefixFlags = PREFIX_FLAG_EXACT_CASE_IN_TREE;
//         NameLink = &PrefixEntry->ExactCaseName;
//         TreeRoot = &ParentFcb->ExactCaseRoot;
//     }
//
//     //
//     //  If neither name is in the tree then check whether we have a buffer for this
//     //  name
//     //
//
//     if (!FlagOn( PrefixEntry->PrefixFlags,
//                  PREFIX_FLAG_EXACT_CASE_IN_TREE | PREFIX_FLAG_IGNORE_CASE_IN_TREE )) {
//
//         //
//         //  Allocate a new buffer if the embedded buffer is too small.
//         //
//
//         NameBuffer = PrefixEntry->FileNameBuffer;
//
//         if (Name->FileName.Length > BYTE_COUNT_EMBEDDED_NAME) {
//
//             NameBuffer = ExAllocatePoolWithTag( CdPagedPool,
//                                                 Name->FileName.Length * 2,
//                                                 TAG_PREFIX_NAME );
//
//             //
//             //  Exit if no name buffer.
//             //
//
//             if (NameBuffer == NULL) { return; }
//         }
//
//         //
//         //  Split the buffer and fill in the separate components.
//         //
//
//         PrefixEntry->ExactCaseName.FileName.Buffer = NameBuffer;
//         PrefixEntry->IgnoreCaseName.FileName.Buffer = Add2Ptr( NameBuffer,
//                                                                Name->FileName.Length,
//                                                                PWCHAR );
//
//         PrefixEntry->IgnoreCaseName.FileName.MaximumLength =
//         PrefixEntry->IgnoreCaseName.FileName.Length =
//         PrefixEntry->ExactCaseName.FileName.MaximumLength =
//         PrefixEntry->ExactCaseName.FileName.Length = Name->FileName.Length;
//     }
//
//     //
//     //  Only insert the name if not already present.
//     //
//
//     if (!FlagOn( PrefixEntry->PrefixFlags, PrefixFlags )) {
//
//         //
//         //  Initialize the name in the prefix entry.
//         //
//
//         RtlCopyMemory( NameLink->FileName.Buffer,
//                        Name->FileName.Buffer,
//                        Name->FileName.Length );
//
//         CdInsertNameLink( IrpContext,
//                           TreeRoot,
//                           NameLink );
//
//         PrefixEntry->Fcb = Fcb;
//         SetFlag( PrefixEntry->PrefixFlags, PrefixFlags );
//     }
//
//     return;
// }
//
//
// VOID
// CdRemovePrefix (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to remove all of the previx entries of a
//     given Fcb from its parent Fcb.
//
// Arguments:
//
//     Fcb - Fcb whose entries are to be removed.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Start with the short name prefix entry.
//     //
//
//     if (Fcb->ShortNamePrefix != NULL) {
//
//         if (FlagOn( Fcb->ShortNamePrefix->PrefixFlags, PREFIX_FLAG_IGNORE_CASE_IN_TREE )) {
//
//             Fcb->ParentFcb->IgnoreCaseRoot = RtlDelete( &Fcb->ShortNamePrefix->IgnoreCaseName.Links );
//         }
//
//         if (FlagOn( Fcb->ShortNamePrefix->PrefixFlags, PREFIX_FLAG_EXACT_CASE_IN_TREE )) {
//
//             Fcb->ParentFcb->ExactCaseRoot = RtlDelete( &Fcb->ShortNamePrefix->ExactCaseName.Links );
//         }
//
//         ClearFlag( Fcb->ShortNamePrefix->PrefixFlags,
//                    PREFIX_FLAG_IGNORE_CASE_IN_TREE | PREFIX_FLAG_EXACT_CASE_IN_TREE );
//     }
//
//     //
//     //  Now do the long name prefix entries.
//     //
//
//     if (FlagOn( Fcb->FileNamePrefix.PrefixFlags, PREFIX_FLAG_IGNORE_CASE_IN_TREE )) {
//
//         Fcb->ParentFcb->IgnoreCaseRoot = RtlDelete( &Fcb->FileNamePrefix.IgnoreCaseName.Links );
//     }
//
//     if (FlagOn( Fcb->FileNamePrefix.PrefixFlags, PREFIX_FLAG_EXACT_CASE_IN_TREE )) {
//
//         Fcb->ParentFcb->ExactCaseRoot = RtlDelete( &Fcb->FileNamePrefix.ExactCaseName.Links );
//     }
//
//     ClearFlag( Fcb->FileNamePrefix.PrefixFlags,
//                PREFIX_FLAG_IGNORE_CASE_IN_TREE | PREFIX_FLAG_EXACT_CASE_IN_TREE );
//
//     //
//     //  Deallocate any buffer we may have allocated.
//     //
//
//     if ((Fcb->FileNamePrefix.ExactCaseName.FileName.Buffer != (PWCHAR) &Fcb->FileNamePrefix.FileNameBuffer) &&
//         (Fcb->FileNamePrefix.ExactCaseName.FileName.Buffer != NULL)) {
//
//         CdFreePool( &Fcb->FileNamePrefix.ExactCaseName.FileName.Buffer );
//         Fcb->FileNamePrefix.ExactCaseName.FileName.Buffer = NULL;
//     }
//
//     return;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdFindPrefix (
//      PIRP_CONTEXT IrpContext,
//      PFCB *CurrentFcb,
//      PUNICODE_STRING RemainingName,
//      BOOLEAN IgnoreCase
//     )
//
// /*++
//
// Routine Description:
//
//     This routine begins from the given CurrentFcb and walks through all of
//     components of the name looking for the longest match in the prefix
//     splay trees.  The search is relative to the starting Fcb so the
//     full name may not begin with a '\'.  On return this routine will
//     update Current Fcb with the lowest point it has travelled in the
//     tree.  It will also hold only that resource on return and it must
//     hold that resource.
//
// Arguments:
//
//     CurrentFcb - Address to store the lowest Fcb we find on this search.
//         On return we will have acquired this Fcb.  On entry this is the
//         Fcb to examine.
//
//     RemainingName - Supplies a buffer to store the exact case of the name being
//         searched for.  Initially will contain the upcase name based on the
//         IgnoreCase flag.
//
//     IgnoreCase - Indicates if we are doing a case-insensitive compare.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     UNICODE_STRING LocalRemainingName;
//
//     UNICODE_STRING FinalName;
//
//     PNAME_LINK NameLink;
//     PPREFIX_ENTRY PrefixEntry;
//
//     PAGED_CODE();
//
//     //
//     //  Make a local copy of the input strings.
//     //
//
//     LocalRemainingName = *RemainingName;
//
//     //
//     //  Loop until we find the longest matching prefix.
//     //
//
//     while (TRUE) {
//
//         //
//         //  If there are no characters left or we are not at an IndexFcb then
//         //  return immediately.
//         //
//
//         if ((LocalRemainingName.Length == 0) ||
//             (SafeNodeType( *CurrentFcb ) != CDFS_NTC_FCB_INDEX)) {
//
//             return;
//         }
//
//         //
//         //  Split off the next component from the name.
//         //
//
//         CdDissectName( IrpContext,
//                        &LocalRemainingName,
//                        &FinalName );
//
//         //
//         //  Check if this name is in the splay tree for this Scb.
//         //
//
//         if (IgnoreCase) {
//
//             NameLink = CdFindNameLink( IrpContext,
//                                        &(*CurrentFcb)->IgnoreCaseRoot,
//                                        &FinalName );
//
//             //
//             //  Get the prefix entry from this NameLink.  Don't access any
//             //  fields within it until we verify we have a name link.
//             //
//
//             PrefixEntry = (PPREFIX_ENTRY) CONTAINING_RECORD( NameLink,
//                                                              PREFIX_ENTRY,
//                                                              IgnoreCaseName );
//
//         } else {
//
//             NameLink = CdFindNameLink( IrpContext,
//                                        &(*CurrentFcb)->ExactCaseRoot,
//                                        &FinalName );
//
//             PrefixEntry = (PPREFIX_ENTRY) CONTAINING_RECORD( NameLink,
//                                                              PREFIX_ENTRY,
//                                                              ExactCaseName );
//         }
//
//         //
//         //  If we didn't find a match then exit.
//         //
//
//         if (NameLink == NULL) { return; }
//
//         //
//         //  If this is a case-insensitive match then copy the exact case of the name into
//         //  the input buffer.
//         //
//
//         if (IgnoreCase) {
//
//             RtlCopyMemory( FinalName.Buffer,
//                            PrefixEntry->ExactCaseName.FileName.Buffer,
//                            PrefixEntry->ExactCaseName.FileName.Length );
//         }
//
//         //
//         //  Update the caller's remaining name string to reflect the fact that we found
//         //  a match.
//         //
//
//         *RemainingName = LocalRemainingName;
//
//         //
//         //  Move down to the next component in the tree.  Acquire without waiting.
//         //  If this fails then lock the Fcb to reference this Fcb and then drop
//         //  the parent and acquire the child.
//         //
//
//         if (!CdAcquireFcbExclusive( IrpContext, PrefixEntry->Fcb, TRUE )) {
//
//             //
//             //  If we can't wait then raise CANT_WAIT.
//             //
//
//             if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//                 CdRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//             }
//
//             CdLockVcb( IrpContext, IrpContext->Vcb );
//             PrefixEntry->Fcb->FcbReference += 1;
//             CdUnlockVcb( IrpContext, IrpContext->Vcb );
//
//             CdReleaseFcb( IrpContext, *CurrentFcb );
//             CdAcquireFcbExclusive( IrpContext, PrefixEntry->Fcb, FALSE );
//
//             CdLockVcb( IrpContext, IrpContext->Vcb );
//             PrefixEntry->Fcb->FcbReference -= 1;
//             CdUnlockVcb( IrpContext, IrpContext->Vcb );
//
//         } else {
//
//             CdReleaseFcb( IrpContext, *CurrentFcb );
//         }
//
//         *CurrentFcb = PrefixEntry->Fcb;
//     }
// }
//
//
// //
// //  Local support routine
// //
//
// PNAME_LINK
// CdFindNameLink (
//      PIRP_CONTEXT IrpContext,
//      PRTL_SPLAY_LINKS *RootNode,
//      PUNICODE_STRING Name
//     )
//
// /*++
//
// Routine Description:
//
//     This routine searches through a splay link tree looking for a match for the
//     input name.  If we find the corresponding name we will rebalance the
//     tree.
//
// Arguments:
//
//     RootNode - Supplies the parent to search.
//
//     Name - This is the name to search for.  Note if we are doing a case
//         insensitive search the name would have been upcased already.
//
// Return Value:
//
//     PNAME_LINK - The name link found or NULL if there is no match.
//
// --*/
//
// {
//     FSRTL_COMPARISON_RESULT Comparison;
//     PNAME_LINK Node;
//     PRTL_SPLAY_LINKS Links;
//
//     PAGED_CODE();
//
//     Links = *RootNode;
//
//     while (Links != NULL) {
//
//         Node = CONTAINING_RECORD( Links, NAME_LINK, Links );
//
//         //
//         //  Compare the prefix in the tree with the full name
//         //
//
//         Comparison = CdFullCompareNames( IrpContext, &Node->FileName, Name );
//
//         //
//         //  See if they don't match
//         //
//
//         if (Comparison == GreaterThan) {
//
//             //
//             //  The prefix is greater than the full name
//             //  so we go down the left child
//             //
//
//             Links = RtlLeftChild( Links );
//
//             //
//             //  And continue searching down this tree
//             //
//
//         } else if (Comparison == LessThan) {
//
//             //
//             //  The prefix is less than the full name
//             //  so we go down the right child
//             //
//
//             Links = RtlRightChild( Links );
//
//             //
//             //  And continue searching down this tree
//             //
//
//         } else {
//
//             //
//             //  We found it.
//             //
//             //  Splay the tree and save the new root.
//             //
//
//             *RootNode = RtlSplay( Links );
//
//             return Node;
//         }
//     }
//
//     //
//     //  We didn't find the Link.
//     //
//
//     return NULL;
// }
//
//
// //
// //  Local support routine
// //
//
// BOOLEAN
// CdInsertNameLink (
//      PIRP_CONTEXT IrpContext,
//      PRTL_SPLAY_LINKS *RootNode,
//      PNAME_LINK NameLink
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will insert a name in the splay tree pointed to
//     by RootNode.
//
//     The name could already exist in this tree for a case-insensitive tree.
//     In that case we simply return FALSE and do nothing.
//
// Arguments:
//
//     RootNode - Supplies a pointer to the table.
//
//     NameLink - Contains the new link to enter.
//
// Return Value:
//
//     BOOLEAN - TRUE if the name is inserted, FALSE otherwise.
//
// --*/
//
// {
//     FSRTL_COMPARISON_RESULT Comparison;
//     PNAME_LINK Node;
//
//     PAGED_CODE();
//
//     RtlInitializeSplayLinks( &NameLink->Links );
//
//     //
//     //  If we are the first entry in the tree, just become the root.
//     //
//
//     if (*RootNode == NULL) {
//
//         *RootNode = &NameLink->Links;
//
//         return TRUE;
//     }
//
//     Node = CONTAINING_RECORD( *RootNode, NAME_LINK, Links );
//
//     while (TRUE) {
//
//         //
//         //  Compare the prefix in the tree with the prefix we want
//         //  to insert.
//         //
//
//         Comparison = CdFullCompareNames( IrpContext, &Node->FileName, &NameLink->FileName );
//
//         //
//         //  If we found the entry, return immediately.
//         //
//
//         if (Comparison == EqualTo) { return FALSE; }
//
//         //
//         //  If the tree prefix is greater than the new prefix then
//         //  we go down the left subtree
//         //
//
//         if (Comparison == GreaterThan) {
//
//             //
//             //  We want to go down the left subtree, first check to see
//             //  if we have a left subtree
//             //
//
//             if (RtlLeftChild( &Node->Links ) == NULL) {
//
//                 //
//                 //  there isn't a left child so we insert ourselves as the
//                 //  new left child
//                 //
//
//                 RtlInsertAsLeftChild( &Node->Links, &NameLink->Links );
//
//                 //
//                 //  and exit the while loop
//                 //
//
//                 break;
//
//             } else {
//
//                 //
//                 //  there is a left child so simply go down that path, and
//                 //  go back to the top of the loop
//                 //
//
//                 Node = CONTAINING_RECORD( RtlLeftChild( &Node->Links ),
//                                           NAME_LINK,
//                                           Links );
//             }
//
//         } else {
//
//             //
//             //  The tree prefix is either less than or a proper prefix
//             //  of the new string.  We treat both cases as less than when
//             //  we do insert.  So we want to go down the right subtree,
//             //  first check to see if we have a right subtree
//             //
//
//             if (RtlRightChild( &Node->Links ) == NULL) {
//
//                 //
//                 //  These isn't a right child so we insert ourselves as the
//                 //  new right child
//                 //
//
//                 RtlInsertAsRightChild( &Node->Links, &NameLink->Links );
//
//                 //
//                 //  and exit the while loop
//                 //
//
//                 break;
//
//             } else {
//
//                 //
//                 //  there is a right child so simply go down that path, and
//                 //  go back to the top of the loop
//                 //
//
//                 Node = CONTAINING_RECORD( RtlRightChild( &Node->Links ),
//                                           NAME_LINK,
//                                           Links );
//             }
//         }
//     }
//
//     return TRUE;
// }
//
//
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
