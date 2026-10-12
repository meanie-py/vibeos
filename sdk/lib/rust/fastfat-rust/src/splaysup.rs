use ntkrnl_sys::globals::*;
use crate::externs::DbgPrint;
// One declaration per external symbol: externs.rs.
use crate::externs::RtlAssert;
// fastfat's own functions: the definition is the declaration.
use crate::verfysup::FatMarkFcbCondition;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PRTL_SPLAY_LINKS, PSTRING, RtlCompareMemory, RtlDelete, RtlFreeOemString, RtlFreeUnicodeString, RtlSplay};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FAT_BUG_CHECK_SPLAYSUP, FCB, FILE_NAME_NODE, PDCB, PFCB, PFILE_NAME_NODE, PIRP_CONTEXT};
use ntkrnl_sys::ddk::{RTL_SPLAY_LINKS, _RTL_SPLAY_LINKS};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_SPLAYSUP};

extern "system" {
    fn KeGetCurrentIrql() -> u8;
    } extern "system" {
    fn CompareNames(name1: PSTRING, name2: PSTRING) -> COMPARISON;
    fn KeBugCheckEx( bugcheck_code: u32, param1: usize, param2: usize, param3: usize, param4: usize, ) -> !;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_SPLAYSUP;
pub const Dbg: u32 = DEBUG_TRACE_SPLAYSUP;

const APC_LEVEL: u32 = 1;

const IsLessThan: u32 = 0;
const IsGreaterThan: u32 = 1;
type COMPARISON = u32;


const IsEqual: COMPARISON = 2;

const FAT_FILE_SYSTEM: u32 = 0x23;
const FcbGood: u32 = 1; // placeholder
const FcbBad: u32 = 2;  // placeholder
#[no_mangle]
pub unsafe extern "system" fn FatInsertName(
    IrpContext: PIRP_CONTEXT,
    RootNode: *mut PRTL_SPLAY_LINKS,
    Name: PFILE_NAME_NODE,
) {
    let mut Comparison: i32 = (IsLessThan) as i32;
    let mut Node: PFILE_NAME_NODE = core::ptr::null_mut();

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 as core::ffi::c_int == 0 {
            // always true
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/filesystems/fastfat/splaysup.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                70u32,
                core::ptr::null_mut(),
            );
        }
    }

    // RtlInitializeSplayLinks equivalent (c2rust expansion)
    let _SplayLinks: PRTL_SPLAY_LINKS = &raw mut (*Name).Links as PRTL_SPLAY_LINKS;
    (*_SplayLinks).Parent = _SplayLinks as *mut _RTL_SPLAY_LINKS;
    (*_SplayLinks).LeftChild = core::ptr::null_mut();
    (*_SplayLinks).RightChild = core::ptr::null_mut();

    'restart: loop {
        if (*RootNode).is_null() {
            *RootNode = &raw mut (*Name).Links as PRTL_SPLAY_LINKS;
            return;
        }

        // CONTAINING_RECORD(*RootNode, FILE_NAME_NODE, Links)
        Node = {
            let offset = core::mem::offset_of!(FILE_NAME_NODE, Links);
            ((*RootNode as *mut u8).byte_offset(-(offset as isize))) as PFILE_NAME_NODE
        };

        loop {
            // Inlined CompareNames logic
            Comparison = {
                let node_first = *(((*Node).Name.Oem.as_ref().Buffer) as *const u8) as core::ffi::c_int;
                let name_first = *(((*Name).Name.Oem.as_ref().Buffer) as *const u8) as core::ffi::c_int;
                if node_first != name_first {
                    if node_first < name_first {
                        (IsLessThan) as i32
                    } else {
                        (IsGreaterThan) as i32
                    }
                } else {
                    FatCompareNames(
                        &raw mut (*Node).Name.Oem as PSTRING,
                        &raw mut (*Name).Name.Oem as PSTRING,
                    ) as core::ffi::c_int
                }
            };

            if Comparison as u32 == IsEqual as u32 {
                if (*(*Node).Fcb).FcbState == FcbGood {
                    KeBugCheckEx(
                        FAT_FILE_SYSTEM,
                        (BugCheckFileId | 125u32) as usize,
                        *RootNode as usize,
                        Name as usize,
                        Node as usize,
                    );
                }
                FatMarkFcbCondition(
                    IrpContext,
                    (*Node).Fcb as PFCB,
                    (FcbBad) as i32,
                    (0u8) as _, // FALSE as BOOLEAN
                );
                FatRemoveNames(IrpContext, (*Node).Fcb as PFCB);
                break; // restarts the 'restart loop
            } else if Comparison as u32 == IsGreaterThan as u32 {
                if (*(&raw mut (*Node).Links as PRTL_SPLAY_LINKS)).LeftChild.is_null() {
                    // RtlInsertAsLeftChild expansion
                    let _SplayParent: PRTL_SPLAY_LINKS =
                        &raw mut (*Node).Links as PRTL_SPLAY_LINKS;
                    let _SplayChild: PRTL_SPLAY_LINKS =
                        &raw mut (*Name).Links as PRTL_SPLAY_LINKS;
                    (*_SplayParent).LeftChild = _SplayChild as *mut _RTL_SPLAY_LINKS;
                    (*_SplayChild).Parent = _SplayParent as *mut _RTL_SPLAY_LINKS;
                    break 'restart;
                } else {
                    let left_child = (*(&raw mut (*Node).Links as PRTL_SPLAY_LINKS)).LeftChild;
                    let offset = core::mem::offset_of!(FILE_NAME_NODE, Links);
                    Node = ((left_child as *mut u8).byte_offset(-(offset as isize)))
                        as PFILE_NAME_NODE;
                }
            } else {
                if (*(&raw mut (*Node).Links as PRTL_SPLAY_LINKS)).RightChild.is_null() {
                    // RtlInsertAsRightChild expansion
                    let _SplayParent: PRTL_SPLAY_LINKS =
                        &raw mut (*Node).Links as PRTL_SPLAY_LINKS;
                    let _SplayChild: PRTL_SPLAY_LINKS =
                        &raw mut (*Name).Links as PRTL_SPLAY_LINKS;
                    (*_SplayParent).RightChild = _SplayChild as *mut _RTL_SPLAY_LINKS;
                    (*_SplayChild).Parent = _SplayParent as *mut _RTL_SPLAY_LINKS;
                    break 'restart;
                } else {
                    let right_child = (*(&raw mut (*Node).Links as PRTL_SPLAY_LINKS)).RightChild;
                    let offset = core::mem::offset_of!(FILE_NAME_NODE, Links);
                    Node = ((right_child as *mut u8).byte_offset(-(offset as isize)))
                        as PFILE_NAME_NODE;
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatRemoveNames(IrpContext: PIRP_CONTEXT, Fcb: PFCB) {
    let _ = IrpContext;

    if (KeGetCurrentIrql() as u32) > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8 as *const i8,
            KeGetCurrentIrql() as u32,
        );
        if 0i32 == 0 {
            RtlAssert(
                0 as *const core::ffi::c_void,
                b"drivers/filesystems/fastfat/splaysup.c\0".as_ptr() as *const core::ffi::c_void,
                248u32,
                core::ptr::null(),
            );
        }
    }

    let Parent: PDCB = (*Fcb).ParentDcb;
    let mut NewRoot: PRTL_SPLAY_LINKS = core::ptr::null_mut();

    if (*Fcb).FcbState & 0x100u32 != 0 {
        NewRoot = RtlDelete(core::ptr::addr_of_mut!((*Fcb).ShortName.Links));
        (*Parent).Specific.Dcb.as_mut().RootOemNode = NewRoot;
        if (*Fcb).FcbState & 0x200u32 != 0 {
            NewRoot = RtlDelete(core::ptr::addr_of_mut!((*Fcb).LongName.Oem.as_mut().Links));
            (*Parent).Specific.Dcb.as_mut().RootOemNode = NewRoot;
            RtlFreeOemString((core::ptr::addr_of_mut!((*Fcb).LongName.Oem.as_mut().Name.Oem)).cast());
            (*Fcb).FcbState &= !0x200u32;
        }
        if (*Fcb).FcbState & 0x400u32 != 0 {
            NewRoot = RtlDelete(core::ptr::addr_of_mut!((*Fcb).LongName.Unicode.as_mut().Links));
            (*Parent).Specific.Dcb.as_mut().RootUnicodeNode = NewRoot;
            RtlFreeUnicodeString((core::ptr::addr_of_mut!((*Fcb).LongName.Unicode.as_mut().Name.Unicode)).cast());
            (*Fcb).FcbState &= !0x400u32;
        }
        (*Fcb).FcbState &= !0x100u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatFindFcb(
    mut IrpContext: PIRP_CONTEXT,
    mut RootNode: *mut PRTL_SPLAY_LINKS,
    mut Name: PSTRING,
    mut FileNameDos: PBOOLEAN,
) -> PFCB {
    let mut Comparison: COMPARISON = IsLessThan;
    let mut Node: PFILE_NAME_NODE = core::ptr::null_mut::<FILE_NAME_NODE>();
    let mut Links: PRTL_SPLAY_LINKS = core::ptr::null_mut::<RTL_SPLAY_LINKS>();

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const core::ffi::c_void,
                b"drivers/filesystems/fastfat/splaysup.c\0"
                    as *const u8 as *const core::ffi::c_void,
                340,
                core::ptr::null(),
            );
        }
    }

    Links = *RootNode;

    while !Links.is_null() {
        Node = (Links as usize)
            .wrapping_sub(core::mem::offset_of!(FILE_NAME_NODE, Links) as usize)
            as PFILE_NAME_NODE;

        // Inline character comparison as per the mechanical anchor
        let node_first_char = *((*Node).Name.Oem.as_ref().Buffer as *const u8) as i32;
        let name_first_char = *((*Name).Buffer as *const u8) as i32;
        Comparison = if node_first_char != name_first_char {
            if node_first_char < name_first_char {
                IsLessThan
            } else {
                IsGreaterThan
            }
        } else {
            CompareNames(
                &raw mut (*Node).Name.Oem as PSTRING,
                Name,
            ) as u32
        } as COMPARISON;

        if Comparison == IsGreaterThan {
            Links = (*Links).LeftChild as PRTL_SPLAY_LINKS;
        } else if Comparison == IsLessThan {
            Links = (*Links).RightChild as PRTL_SPLAY_LINKS;
        } else {
            *RootNode = RtlSplay(Links);
            if !FileNameDos.is_null() {
                *FileNameDos = (*Node).FileNameDos as u8;
            }
            return (*Node).Fcb as PFCB;
        }
    }

    return core::ptr::null_mut::<FCB>();
}
#[no_mangle]
pub unsafe extern "system" fn FatCompareNames(
    NameA: ntkrnl_sys::ddk::PSTRING,
    NameB: ntkrnl_sys::ddk::PSTRING,
) -> COMPARISON {
    let mut i: u32;
    let mut min_length: u32;

    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const core::ffi::c_void,
                b"drivers/filesystems/fastfat/splaysup.c\0"
                    as *const u8 as *const core::ffi::c_void,
                453 as u32,
                core::ptr::null(),
            );
        }
    }

    min_length = (if ((*NameA).Length as i32) < (*NameB).Length as i32 {
        (*NameA).Length as i32
    } else {
        (*NameB).Length as i32
    }) as u32;

    i = ntkrnl_sys::ddk::RtlCompareMemory(
        (*NameA).Buffer as *const core::ffi::c_void,
        (*NameB).Buffer as *const core::ffi::c_void,
        min_length as _,
    ) as u32;

    if i < min_length {
        return if (*(*NameA).Buffer.add(i as usize) as i32) < (*(*NameB).Buffer.add(i as usize) as i32) {
            IsLessThan
        } else {
            IsGreaterThan
        };
    }

    if ((*NameA).Length as i32) < (*NameB).Length as i32 {
        return IsLessThan;
    }

    if (*NameA).Length as i32 > (*NameB).Length as i32 {
        return IsGreaterThan;
    }

    return IsEqual;
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
//     This module implements the Fat Name lookup Suport routines
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (FAT_BUG_CHECK_SPLAYSUP)
//
// //
// //  The debug trace level for this module
// //
//
// #define Dbg                              (DEBUG_TRACE_SPLAYSUP)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatInsertName)
// #pragma alloc_text(PAGE, FatRemoveNames)
// #pragma alloc_text(PAGE, FatFindFcb)
// #pragma alloc_text(PAGE, FatCompareNames)
// #endif
//
//
// VOID
// FatInsertName (
//     IN PIRP_CONTEXT IrpContext,
//     IN PRTL_SPLAY_LINKS *RootNode,
//     IN PFILE_NAME_NODE Name
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will insert a name in the splay tree pointed to
//     by RootNode.
//
//     The name must not already exist in the splay tree.
//
// Arguments:
//
//     RootNode - Supplies a pointer to the table.
//
//     Name - Contains the New name to enter.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     COMPARISON Comparison;
//     PFILE_NAME_NODE Node;
//
//     PAGED_CODE();
//
//     RtlInitializeSplayLinks(&Name->Links);
//
// Restart:
//
//     //
//     //  If we are the first entry in the tree, just become the root.
//     //
//
//     if (*RootNode == NULL) {
//
//         *RootNode = &Name->Links;
//
//         return;
//     }
//
//     Node = CONTAINING_RECORD( *RootNode, FILE_NAME_NODE, Links );
//
//     while (TRUE) {
//
//         //
//         //  Compare the prefix in the tree with the prefix we want
//         //  to insert.  Note that Oem here doesn't mean anything.
//         //
//
//         Comparison = CompareNames(&Node->Name.Oem, &Name->Name.Oem);
//
//         //
//         //  We should never find the name in the table already.
//         //
//
//         if (Comparison == IsEqual) {
//
//             //
//             //  Almost. If the removable media was taken to another machine and
//             //  back, and we have something like:
//             //
//             //  Old: abcdef~1  /  abcdefxyz
//             //  New: abcdef~1  /  abcdefxyzxyz
//             //
//             //  but a handle was kept open to abcdefxyz so we couldn't purge
//             //  away the Fcb in the verify path ... opening abcdefxyzxyz will
//             //  try to insert a duplicate shortname. Bang!
//             //
//             //  Invalidate it and the horse it came in on.  This new one wins.
//             //  The old one is gone.  Only if the old one is in normal state
//             //  do we really have a problem.
//             //
//
//             if (Node->Fcb->FcbState == FcbGood) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//                 FatBugCheck( (ULONG_PTR)*RootNode, (ULONG_PTR)Name, (ULONG_PTR)Node );
//             }
//
//             //
//             //  Note, once we zap the prefix links we need to restart our walk
//             //  of the tree. Note that we aren't properly synchronized to
//             //  recursively mark bad.
//             //
//
//             FatMarkFcbCondition( IrpContext, Node->Fcb, FcbBad, FALSE );
//             FatRemoveNames( IrpContext, Node->Fcb );
//
//             goto Restart;
//         }
//
//         //
//         //  If the tree prefix is greater than the new prefix then
//         //  we go down the left subtree
//         //
//
//         if (Comparison == IsGreaterThan) {
//
//             //
//             //  We want to go down the left subtree, first check to see
//             //  if we have a left subtree
//             //
//
//             if (RtlLeftChild(&Node->Links) == NULL) {
//
//                 //
//                 //  there isn't a left child so we insert ourselves as the
//                 //  new left child
//                 //
//
//                 RtlInsertAsLeftChild(&Node->Links, &Name->Links);
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
//                 Node = CONTAINING_RECORD( RtlLeftChild(&Node->Links),
//                                           FILE_NAME_NODE,
//                                           Links );
//             }
//
//         } else {
//
//             //
//             //  The tree prefix is either less than or a proper prefix
//             //  of the new string.  We treat both cases a less than when
//             //  we do insert.  So we want to go down the right subtree,
//             //  first check to see if we have a right subtree
//             //
//
//             if (RtlRightChild(&Node->Links) == NULL) {
//
//                 //
//                 //  These isn't a right child so we insert ourselves as the
//                 //  new right child
//                 //
//
//                 RtlInsertAsRightChild(&Node->Links, &Name->Links);
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
//                 Node = CONTAINING_RECORD( RtlRightChild(&Node->Links),
//                                           FILE_NAME_NODE,
//                                           Links );
//             }
//
//         }
//     }
//
//     return;
// }
//
// VOID
// FatRemoveNames (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will remove the short name and any long names associated
//     with the files from their repsective splay tree.
//
// Arguments:
//
//     Name - Supplies the Fcb to process.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PDCB Parent;
//     PRTL_SPLAY_LINKS NewRoot;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     Parent = Fcb->ParentDcb;
//
//     //
//     //  We used to assert this condition, but it really isn't good.  If
//     //  someone rapidly renames a directory multiple times and we can't
//     //  flush the lower fcbs fast enough (that didn't go away synch.)
//     //  well, well hit some of them again.
//     //
//     //  NT_ASSERT( FlagOn( Fcb->FcbState, FCB_STATE_NAMES_IN_SPLAY_TREE ));
//     //
//
//     if (FlagOn( Fcb->FcbState, FCB_STATE_NAMES_IN_SPLAY_TREE )) {
//
//         //
//         //  Delete the node short name.
//         //
//
//         NewRoot = RtlDelete(&Fcb->ShortName.Links);
//
//         Parent->Specific.Dcb.RootOemNode = NewRoot;
//
//         //
//         //  Now check for the presence of long name and delete it.
//         //
//
//         if (FlagOn( Fcb->FcbState, FCB_STATE_HAS_OEM_LONG_NAME )) {
//
//             NewRoot = RtlDelete(&Fcb->LongName.Oem.Links);
//
//             Parent->Specific.Dcb.RootOemNode = NewRoot;
//
//             RtlFreeOemString( &Fcb->LongName.Oem.Name.Oem );
//
//             ClearFlag( Fcb->FcbState, FCB_STATE_HAS_OEM_LONG_NAME );
//         }
//
//         if (FlagOn( Fcb->FcbState, FCB_STATE_HAS_UNICODE_LONG_NAME )) {
//
//             NewRoot = RtlDelete(&Fcb->LongName.Unicode.Links);
//
//             Parent->Specific.Dcb.RootUnicodeNode = NewRoot;
//
//             RtlFreeUnicodeString( &Fcb->LongName.Unicode.Name.Unicode );
//
//             ClearFlag( Fcb->FcbState, FCB_STATE_HAS_UNICODE_LONG_NAME );
//         }
//
//         ClearFlag( Fcb->FcbState, FCB_STATE_NAMES_IN_SPLAY_TREE );
//     }
//
//     return;
// }
//
//
// PFCB
// FatFindFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PRTL_SPLAY_LINKS *RootNode,
//     IN PSTRING Name,
//     OUT PBOOLEAN FileNameDos OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine searches either the Oem or Unicode splay tree looking
//     for an Fcb with the specified name.  In the case the Fcb is found,
//     rebalance the tree.
//
// Arguments:
//
//     RootNode - Supplies the parent to search.
//
//     Name - If present, search the Oem tree.
//
//     UnicodeName - If present, search the Unicode tree.
//
// Return Value:
//
//     PFCB - The Fcb, or NULL if none was found.
//
// --*/
//
// {
//     COMPARISON Comparison;
//     PFILE_NAME_NODE Node;
//     PRTL_SPLAY_LINKS Links;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     Links = *RootNode;
//
//     while (Links != NULL) {
//
//         Node = CONTAINING_RECORD(Links, FILE_NAME_NODE, Links);
//
//         //
//         //  Compare the prefix in the tree with the full name
//         //
//
//         Comparison = CompareNames(&Node->Name.Oem, Name);
//
//         //
//         //  See if they don't match
//         //
//
//         if (Comparison == IsGreaterThan) {
//
//             //
//             //  The prefix is greater than the full name
//             //  so we go down the left child
//             //
//
//             Links = RtlLeftChild(Links);
//
//             //
//             //  And continue searching down this tree
//             //
//
//         } else if (Comparison == IsLessThan) {
//
//             //
//             //  The prefix is less than the full name
//             //  so we go down the right child
//             //
//
//             Links = RtlRightChild(Links);
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
//             *RootNode = RtlSplay(Links);
//
//             //
//             //  Tell the caller what kind of name we hit
//             //
//
//             if (FileNameDos) {
//
//                 *FileNameDos = Node->FileNameDos;
//             }
//
//             return Node->Fcb;
//         }
//     }
//
//     //
//     //  We didn't find the Fcb.
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
// COMPARISON
// FatCompareNames (
//     IN PSTRING NameA,
//     IN PSTRING NameB
//     )
//
// /*++
//
// Routine Description:
//
//     This function compares two names as fast as possible.  Note that since
//     this comparison is case sensitive, I neither know nor case if the names
//     are UNICODE or OEM.  All that is important is that the result is
//     deterministic.
//
// Arguments:
//
//     NameA & NameB - The names to compare.
//
// Return Value:
//
//     COMPARISON - returns
//
//         IsLessThan    if NameA < NameB lexicalgraphically,
//         IsGreaterThan if NameA > NameB lexicalgraphically,
//         IsEqual       if NameA is equal to NameB
//
// --*/
//
// {
//     ULONG i;
//     ULONG MinLength;
//
//     PAGED_CODE();
//
//     //
//     //  Figure out the minimum of the two lengths
//     //
//
//     MinLength = NameA->Length < NameB->Length ? NameA->Length :
//                                                 NameB->Length;
//
//     //
//     //  Loop through looking at all of the characters in both strings
//     //  testing for equalilty, less than, and greater than
//     //
//
//     i = (ULONG)RtlCompareMemory( NameA->Buffer, NameB->Buffer, MinLength );
//
//
//     if (i < MinLength) {
//
//         return NameA->Buffer[i] < NameB->Buffer[i] ? IsLessThan :
//                                                      IsGreaterThan;
//     }
//
//     if (NameA->Length < NameB->Length) {
//
//         return IsLessThan;
//     }
//
//     if (NameA->Length > NameB->Length) {
//
//         return IsGreaterThan;
//     }
//
//     return IsEqual;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
