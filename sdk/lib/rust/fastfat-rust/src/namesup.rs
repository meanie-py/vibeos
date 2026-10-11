#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
// One declaration per external symbol: externs.rs.
use crate::externs::FatIsNameShortOemValid;
use crate::externs::FatUnpinBcb;
// fastfat's own functions: the definition is the declaration.
use crate::dirsup::FatLocateDirent;
use crate::dirsup::FatLocateSimpleOemDirent;
use crate::seh::FatNormalizeAndRaiseStatus;
use crate::seh::FatRaiseStatus;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, GENERATE_NAME_CONTEXT, OEM_STRING, POEM_STRING, PUNICODE_STRING, RtlDowncaseUnicodeString, RtlGenerate8dot3Name, RtlOemStringToCountedUnicodeString, RtlUnicodeStringToCountedOemString, RtlUpcaseUnicodeStringToCountedOemString, _UNICODE_STRING};
use ntkrnl_sys::ddk::_PUBLIC_BCB as BCB;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, DCB, DIRENT, FCB, FatData, IRP_CONTEXT, PFCB, PIRP_CONTEXT};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_NAMESUP, STATUS_UNMAPPABLE_CHARACTER, TAG_FILENAME_BUFFER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag};

extern "system" {
}
extern "system" {
    fn FsRtlIsDbcsInExpression(Expression: *const OEM_STRING, Name: *const OEM_STRING) -> BOOLEAN;
    } extern "system" {
    fn FsRtlIsLeadDbcsCharacter(character: u8) -> u8;
}

// ---- file-scope preamble (translated) ----
pub const Dbg: u32 = DEBUG_TRACE_NAMESUP;

// Forward declarations for unknown externs

// Constants
const MAX_LFN_CHARACTERS: usize = 0x0104;
const STATUS_FILE_INVALID: i32 = 0xC0000098_u32 as i32; // NTSTATUS

// FsRtlLegalAnsiCharacterArray is a kernel DATA export — raw-dylib routes the
// reference through the __imp_ thunk (the glob `globals::*` decl lacks the link).
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    #[link_name = "FsRtlLegalAnsiCharacterArray"]
    static FAT_LEGAL_ANSI_CHARACTER_ARRAY: *const u8;
}

//
//  Translated from the INLINE FatIsNameLongUnicodeValid in fatprocs.h.
//  Validates that every ASCII (<0x80) char of a long Unicode name is a legal
//  HPFS character (per FsRtlLegalAnsiCharacterArray); non-ASCII chars are
//  always permitted.  FSRTL_HPFS_LEGAL=0x02, FSRTL_WILD_CHARACTER=0x08.
//
#[no_mangle]
pub unsafe extern "system" fn FatIsNameLongUnicodeValid(
    _IrpContext: *mut IRP_CONTEXT,
    Name: *mut _UNICODE_STRING,
    CanContainWildcards: BOOLEAN,
    _PathNameOk: BOOLEAN,
    _LeadingBackslashOk: BOOLEAN,
) -> BOOLEAN {
    let mask: u8 = if CanContainWildcards != 0 { 0x02 | 0x08 } else { 0x02 };
    let arr: *const u8 = FAT_LEGAL_ANSI_CHARACTER_ARRAY;
    let len = (*Name).Length as usize / core::mem::size_of::<u16>();
    let buf = (*Name).Buffer;
    let mut i = 0usize;
    while i < len {
        let c = *buf.add(i);
        if c < 0x80 && (*arr.add(c as usize)) & mask == 0 {
            return 0 as BOOLEAN; // FALSE
        }
        i += 1;
    }
    1 as BOOLEAN // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn FatIsNameInExpression(
    IrpContext: PIRP_CONTEXT,
    Expression: OEM_STRING,
    Name: OEM_STRING,
) -> BOOLEAN {
    // PAGED_CODE();
    let _ = &IrpContext;
    FsRtlIsDbcsInExpression(
        core::ptr::addr_of!(Expression),
        core::ptr::addr_of!(Name),
    )
}

const UCHAR_SP: u8 = 0x20;
const FAT_DIRENT_REALLY_0E5: u8 = 0x05;
#[no_mangle]
pub unsafe extern "system" fn FatStringTo8dot3(
    IrpContext: *mut core::ffi::c_void,
    InputString: OEM_STRING,
    Output8dot3: *mut [u8; 11],
) {
    PAGED_CODE!();

    DebugTrace!(1, Dbg, b"FatStringTo8dot3\n\0".as_ptr() as *const i8, 0);
    DebugTrace!(0, Dbg, b"InputString = %Z\n\0".as_ptr() as *const i8, &InputString as *const OEM_STRING);

    NT_ASSERT!((InputString.Length <= 12) as i32);

    // Make the output name all blanks
    core::ptr::write_bytes(Output8dot3 as *mut u8, UCHAR_SP, 11);

    // Copy over the first part of the file name.  Stop when we get to
    // the end of the input string or a dot.
    let mut i: u32 = 0;
    while i < InputString.Length as u32 && *InputString.Buffer.add(i as usize) != (b'.') as i8 && i < 11 {
        (*Output8dot3)[i as usize] = (*InputString.Buffer.add(i as usize)) as u8;
        i += 1;
    }

    // Check if we need to process an extension
    if i < InputString.Length as u32 {
        // Make sure we have a dot and then skip over it.
        NT_ASSERT!(((InputString.Length as u32) - i <= 4) as i32);
        NT_ASSERT!((*InputString.Buffer.add(i as usize) == (b'.') as i8) as i32);

        i += 1;

        // Copy over the extension.  Stop when we get to the
        // end of the input string.
        let mut j: u32 = 8;
        while i < InputString.Length as u32 {
            (*Output8dot3)[j as usize] = (*InputString.Buffer.add(i as usize)) as u8;
            j += 1;
            i += 1;
        }
    }

    // Before we return check if we should translate the first character
    // from 0xe5 to 0x5.
    if (*Output8dot3)[0] == 0xe5 {
        (*Output8dot3)[0] = FAT_DIRENT_REALLY_0E5;
    }

    DebugTrace!(-1, Dbg, b"FatStringTo8dot3 -> (VOID)\n\0".as_ptr() as *const i8, 0);

    let _ = IrpContext; // UNREFERENCED_PARAMETER
}
#[no_mangle]
pub unsafe extern "system" fn Fat8dot3ToString(
    IrpContext: *mut IRP_CONTEXT,
    Dirent: *mut DIRENT,
    RestoreCase: BOOLEAN,
    OutputString: *mut OEM_STRING,
) {
    unsafe {
        const FAT_DIRENT_REALLY_0E5: u8 = 0x05;
        const FAT_DIRENT_NT_BYTE_8_LOWER_CASE: u8 = 0x08;
        const FAT_DIRENT_NT_BYTE_3_LOWER_CASE: u8 = 0x10;
        const UCHAR_SP: u8 = b' ';

        let mut BaseLength: u32;
        let mut ExtensionLength: u32;
        let mut StringIndex: u32;
        let mut o: *mut u8;
        let mut d: *const u8;

        PAGED_CODE!();

        DebugTrace!(1, 0u32, b"Fat8dot3ToString\n\0".as_ptr() as *const i8, 0);

        BaseLength = 8;
        while BaseLength > 0 {
            if (*Dirent).FileName[(BaseLength - 1) as usize] != UCHAR_SP {
                break;
            }
            BaseLength -= 1;
        }

        ExtensionLength = 3;
        while ExtensionLength > 0 {
            if (*Dirent).FileName[(8 + ExtensionLength - 1) as usize] != UCHAR_SP {
                break;
            }
            ExtensionLength -= 1;
        }

        if BaseLength != 0 {
            core::ptr::copy_nonoverlapping(
                &(*Dirent).FileName as *const u8,
                ((*OutputString).Buffer).cast(),
                BaseLength as usize,
            );

            if *(*OutputString).Buffer == (FAT_DIRENT_REALLY_0E5) as i8 {
                *(*OutputString).Buffer = 0xe5_u8 as i8;
            }

            let chicago = (*core::ptr::addr_of!(FatData)).ChicagoMode();
            if chicago != 0 && RestoreCase != 0 && ((*Dirent).NtByte & FAT_DIRENT_NT_BYTE_8_LOWER_CASE) != 0 {
                StringIndex = 0;
                while StringIndex < BaseLength {
                    let p = (*OutputString).Buffer.offset(StringIndex as isize);
                    if FsRtlIsLeadDbcsCharacter((*p) as u8) != 0 {
                        StringIndex += 1;
                        continue;
                    }
                    if *p >= (b'A') as i8 && *p <= (b'Z') as i8 {
                        *p += (b'a' - b'A') as i8;
                    }
                    StringIndex += 1;
                }
            }
        }

        if ExtensionLength != 0 {
            *(*OutputString).Buffer.offset(BaseLength as isize) = (b'.') as i8;
            BaseLength += 1;

            o = ((*OutputString).Buffer.offset(BaseLength as isize)).cast();
            d = &(*Dirent).FileName[8] as *const u8;

            // replica of fallthrough switch
            if ExtensionLength >= 1 {
                *o = *d;
                if ExtensionLength >= 2 {
                    o = o.offset(1);
                    d = d.offset(1);
                    *o = *d;
                    if ExtensionLength >= 3 {
                        o = o.offset(1);
                        d = d.offset(1);
                        *o = *d;
                    }
                }
            }

            (*OutputString).Length = (BaseLength + ExtensionLength) as u16;

            let chicago = (*core::ptr::addr_of!(FatData)).ChicagoMode();
            if chicago != 0 && RestoreCase != 0 && ((*Dirent).NtByte & FAT_DIRENT_NT_BYTE_3_LOWER_CASE) != 0 {
                StringIndex = BaseLength;
                while StringIndex < (*OutputString).Length as u32 {
                    let p = (*OutputString).Buffer.offset(StringIndex as isize);
                    if FsRtlIsLeadDbcsCharacter((*p) as u8) != 0 {
                        StringIndex += 1;
                        continue;
                    }
                    if *p >= (b'A') as i8 && *p <= (b'Z') as i8 {
                        *p += (b'a' - b'A') as i8;
                    }
                    StringIndex += 1;
                }
            }
        } else {
            (*OutputString).Length = BaseLength as u16;
        }

        DebugTrace!(-1, 0u32, b"Fat8dot3ToString, OutputString = \"%Z\" -> VOID\n\0".as_ptr() as *const i8, OutputString as *const OEM_STRING);

        UNREFERENCED_PARAMETER!(IrpContext);
    }
    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatGetUnicodeNameFromFcb(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Lfn: *mut _UNICODE_STRING,
) {
    let mut dirent: *mut DIRENT = core::ptr::null_mut();
    let mut dirent_bcb: *mut BCB = core::ptr::null_mut();
    let mut dirent_byte_offset: u32 = 0;
    let mut local_ccb: CCB = core::mem::zeroed();

    PAGED_CODE!();

    NT_ASSERT!((((MAX_LFN_CHARACTERS as u16) * (core::mem::size_of::<u16>() as u16))
            == (*Lfn).MaximumLength) as i32);

    // We'll start by locating the dirent for the name.
    FatStringTo8dot3(
        IrpContext as *mut c_void,
        *(*Fcb).ShortName.Name.Oem.as_ref(),
        core::ptr::addr_of_mut!(local_ccb.__bindgen_anon_1.__bindgen_anon_1.as_mut().OemQueryTemplate) as *mut [u8; 11],
    );

    local_ccb.Flags = 0;
    local_ccb.__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate.Length = 0;
    local_ccb.ContainsWildCards = 0;

    FatLocateDirent(
        IrpContext,
        (*Fcb).ParentDcb,
        &raw mut local_ccb,
        (*Fcb).LfnOffsetWithinDirectory,
        core::ptr::null_mut(),
        &raw mut dirent,
        &raw mut dirent_bcb,
        (&raw mut dirent_byte_offset as *mut core::ffi::c_void).cast(),
        core::ptr::null_mut(),
        Lfn,
        core::ptr::null_mut(),
    );

    // _SEH2_TRY / _SEH2_FINALLY: the finally unpins on the raise paths too.
    let p_dirent_bcb = core::ptr::addr_of_mut!(dirent_bcb);
    crate::seh::try_finally(&mut (), |_| {
        // If we didn't find the Dirent, something is terribly wrong.
        if (*p_dirent_bcb).is_null() || dirent_byte_offset != (*Fcb).DirentOffsetWithinDirectory {
            FatRaiseStatus(IrpContext, STATUS_FILE_INVALID);
        }

        // Check for the easy case.
        if (*Lfn).Length == 0 {
            let mut status: NTSTATUS;
            let mut short_name: OEM_STRING = core::mem::zeroed();
            let mut short_name_buffer: [u8; 12] = [0u8; 12];

            // If we thought that there was an LFN here and didn't find one,
            // we're as dead.
            NT_ASSERT!(((*Fcb).LfnOffsetWithinDirectory == (*Fcb).DirentOffsetWithinDirectory) as i32);

            if (*Fcb).LfnOffsetWithinDirectory != (*Fcb).DirentOffsetWithinDirectory {
                FatRaiseStatus(IrpContext, STATUS_FILE_INVALID);
            }

            // There is no LFN, so manufacture a UNICODE name.
            short_name.Length = 0;
            short_name.MaximumLength = 12;
            short_name.Buffer = short_name_buffer.as_mut_ptr() as *mut i8;

            Fat8dot3ToString(
                IrpContext,
                dirent,
                1, // TRUE
                &raw mut short_name as *mut OEM_STRING,
            );

            // OK, now convert this string to UNICODE
            status = RtlOemStringToCountedUnicodeString(
                Lfn,
                &raw mut short_name as *mut OEM_STRING,
                0, // FALSE
            );

            NT_ASSERT!((status == STATUS_SUCCESS) as i32);
        }
    }, |_, _abnormal| {
        // _SEH2_FINALLY
        FatUnpinBcb(IrpContext, p_dirent_bcb as *mut *mut core::ffi::c_void);
    });
}
#[no_mangle]
pub unsafe extern "system" fn FatSetFullFileNameInFcb(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
) {
    // PAGED_CODE();  (no-op)

    if (*Fcb).FullFileName.Buffer.is_null() {
        let mut Lfn: _UNICODE_STRING = core::mem::zeroed();
        let mut TmpFcb = Fcb;
        let mut StopFcb: PFCB = core::ptr::null_mut();
        let mut TmpBuffer: *mut u16 = core::ptr::null_mut();
        let mut PathLength: u32 = 0;

        // Initialize Lfn
        Lfn.Length = 0;
        Lfn.MaximumLength = (MAX_LFN_CHARACTERS * core::mem::size_of::<u16>()) as u16;
        Lfn.Buffer = FsRtlAllocatePoolWithTag(
            ntkrnl_sys::mm::PagedPool,
            (MAX_LFN_CHARACTERS * core::mem::size_of::<u16>()) as u32,
            TAG_FILENAME_BUFFER,
        ) as *mut u16;

        // _SEH2_TRY / _SEH2_FINALLY
        let p_lfn = core::ptr::addr_of_mut!(Lfn);
        crate::seh::try_finally(&mut (), |_| {
            // First pass: accumulate PathLength, possibly break
            while TmpFcb != (*(*Fcb).Vcb).RootDcb {
                if (TmpFcb != Fcb) && !(*TmpFcb).FullFileName.Buffer.is_null() {
                    PathLength += (*TmpFcb).FullFileName.Length as u32;

                    (*Fcb).FullFileName.Buffer = FsRtlAllocatePoolWithTag(
                        ntkrnl_sys::mm::PagedPool,
                        PathLength,
                        TAG_FILENAME_BUFFER,
                    ) as *mut u16;

                    core::ptr::copy_nonoverlapping(
                        (*TmpFcb).FullFileName.Buffer as *const u8,
                        (*Fcb).FullFileName.Buffer as *mut u8,
                        (*TmpFcb).FullFileName.Length as usize,
                    );
                    break;
                }

                PathLength += (*TmpFcb).FinalNameLength as u32
                    + core::mem::size_of::<u16>() as u32;
                TmpFcb = (*TmpFcb).ParentDcb;
            }

            // Allocate if still NULL
            if (*Fcb).FullFileName.Buffer.is_null() {
                (*Fcb).FullFileName.Buffer = FsRtlAllocatePoolWithTag(
                    ntkrnl_sys::mm::PagedPool,
                    PathLength,
                    TAG_FILENAME_BUFFER,
                ) as *mut u16;
            }

            StopFcb = TmpFcb;
            TmpFcb = Fcb;
            TmpBuffer = (*Fcb).FullFileName.Buffer
                .offset((PathLength / core::mem::size_of::<u16>() as u32) as isize);

            (*Fcb).FullFileName.Length = PathLength as u16;
            (*Fcb).FullFileName.MaximumLength = PathLength as u16;

            // Second pass: copy names backwards
            while TmpFcb != StopFcb {
                FatGetUnicodeNameFromFcb(
                    IrpContext,
                    TmpFcb,
                    p_lfn,
                );

                TmpBuffer = TmpBuffer.offset(
                    -((*p_lfn).Length as isize / core::mem::size_of::<u16>() as isize),
                );

                core::ptr::copy_nonoverlapping(
                    (*p_lfn).Buffer as *const u8,
                    TmpBuffer as *mut u8,
                    (*p_lfn).Length as usize,
                );

                TmpBuffer = TmpBuffer.offset(-1);
                *TmpBuffer = b'\\' as u16;

                TmpFcb = (*TmpFcb).ParentDcb;
            }
        }, |_, abnormal| {
            if abnormal {
                if !(*Fcb).FullFileName.Buffer.is_null() {
                    ExFreePool((*Fcb).FullFileName.Buffer as *mut c_void);
                    (*Fcb).FullFileName.Buffer = core::ptr::null_mut();
                }
            }
            ExFreePool((*p_lfn).Buffer as *mut c_void);
        });
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatUnicodeToUpcaseOem(IrpContext: *mut IRP_CONTEXT, OemString: *mut OEM_STRING, UnicodeString: *mut _UNICODE_STRING) {
    // PAGED_CODE();
    let mut Status: NTSTATUS = RtlUpcaseUnicodeStringToCountedOemString(OemString, UnicodeString, 0);
    if Status == STATUS_BUFFER_OVERFLOW {
        (*OemString).Buffer = core::ptr::null_mut();
        (*OemString).Length = 0;
        (*OemString).MaximumLength = 0;
        Status = RtlUpcaseUnicodeStringToCountedOemString(OemString, UnicodeString, 1);
    }
    if !NT_SUCCESS(Status) {
        if Status == STATUS_UNMAPPABLE_CHARACTER {
            (*OemString).Length = 0;
        } else {
            FatNormalizeAndRaiseStatus(IrpContext, Status);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatSelectNames(
    IrpContext: *mut core::ffi::c_void,
    Parent: *mut core::ffi::c_void,
    OemName: *mut OEM_STRING,
    UnicodeName: *mut _UNICODE_STRING,
    ShortName: *mut OEM_STRING,
    SuggestedShortName: *mut _UNICODE_STRING,
    AllLowerComponent: *mut u8,
    AllLowerExtension: *mut u8,
    CreateLfn: *mut u8,
) {
    // PAGED_CODE(); – pageable code assertion
    let mut GenerateShortName: u8;

    if (*OemName).Length == 0
        || FatIsNameShortOemValid(
            (IrpContext).cast(),
            *OemName,
            0u8,
            0u8,
            0u8,
        ) == 0
        || FatSpaceInName(
            IrpContext,
            UnicodeName as *mut _UNICODE_STRING,
        ) != 0
    {
        let mut ShortNameBuffer: [u16; 12] = [0u16; 12];
        let mut ShortUnicodeName: _UNICODE_STRING = core::mem::zeroed();
        let mut Context: GENERATE_NAME_CONTEXT = core::mem::zeroed();
        let mut TrySuggestedShortName: u8;
        let mut Dirent: *mut DIRENT = core::ptr::null_mut();
        let mut Bcb: *mut BCB = core::ptr::null_mut();
        let mut ByteOffset: u32 = 0;
        let mut Status: i32 = 0;

        GenerateShortName = 1;

        TrySuggestedShortName = if SuggestedShortName.is_null() { 0 } else { 1 };

        ShortUnicodeName.Length = 0;
        ShortUnicodeName.MaximumLength = (12 * core::mem::size_of::<u16>()) as u16;
        ShortUnicodeName.Buffer = ShortNameBuffer.as_mut_ptr();

        ntkrnl_sys::rtl::RtlZeroMemory(
            &mut Context as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<GENERATE_NAME_CONTEXT>(),
        );

        // _SEH2_TRY / _SEH2_FINALLY
        let p_bcb = core::ptr::addr_of_mut!(Bcb);
        crate::seh::try_finally(&mut (), |_| {
            loop {
                FatUnpinBcb(
                    IrpContext as *mut IRP_CONTEXT, p_bcb as *mut *mut core::ffi::c_void);

                if TrySuggestedShortName != 0 {
                    ShortUnicodeName.Length = (*SuggestedShortName).Length;
                    ShortUnicodeName.MaximumLength = (*SuggestedShortName).MaximumLength;
                    ShortUnicodeName.Buffer = (*SuggestedShortName).Buffer;
                    TrySuggestedShortName = 0;
                } else {
                    RtlGenerate8dot3Name(
                        UnicodeName as *const _UNICODE_STRING,
                        1u8,
                        &mut Context,
                        &mut ShortUnicodeName,
                    );
                }

                Status = RtlUnicodeStringToCountedOemString(
                    ShortName,
                    &ShortUnicodeName as *const _UNICODE_STRING,
                    0u8,
                );

                NT_ASSERT!(((Status == ntkrnl_sys::base::STATUS_SUCCESS) as u8) as i32);

                FatLocateSimpleOemDirent(
                    (IrpContext).cast(),
                    (Parent).cast(),
                    ShortName,
                    (&mut Dirent as *mut *mut DIRENT as *mut *mut c_void).cast(),
                    (p_bcb as *mut *mut c_void).cast(),
                    (&mut ByteOffset as *mut u32 as *mut c_void).cast(),
                );

                if (*p_bcb).is_null() {
                    // _SEH2_LEAVE
                    break;
                }
            }
        }, |_, _abnormal| {
            // _SEH2_FINALLY
            FatUnpinBcb(
                IrpContext as *mut IRP_CONTEXT, p_bcb as *mut *mut core::ffi::c_void);
        });
        // _SEH2_END
    } else {
        if ShortName != OemName {
            (*ShortName).Length = (*OemName).Length;

            // _Analysis_assume_(ShortName->MaximumLength <= OemName->Length);

            ntkrnl_sys::rtl::RtlCopyMemory(
                (*ShortName).Buffer as *mut _ as *mut core::ffi::c_void,
                (*OemName).Buffer as *const _ as *const core::ffi::c_void,
                (*OemName).Length as usize,
            );
        }

        GenerateShortName = 0;
    }

    if GenerateShortName != 0 {
        *CreateLfn = 1;
        *AllLowerComponent = 0;
        *AllLowerExtension = 0;
    } else {
        FatEvaluateNameCase(
            IrpContext as *mut IRP_CONTEXT,
            UnicodeName,
            AllLowerComponent,
            AllLowerExtension,
            CreateLfn,
        );
    }
}


#[no_mangle]
pub unsafe extern "system" fn FatEvaluateNameCase(
    IrpContext: PIRP_CONTEXT,
    UnicodeName: PUNICODE_STRING,
    AllLowerComponent: *mut u8,
    AllLowerExtension: *mut u8,
    CreateLfn: *mut u8,
) {
    let mut i: u32;
    let mut Uppers: u8 = 0;
    let mut Lowers: u8 = 0;
    let mut ExtensionPresent: u8 = 0;

    PAGED_CODE!();
    let _ = IrpContext;

    *CreateLfn = 0;

    i = 0;
    while (i as u32) < (((*UnicodeName).Length / (core::mem::size_of::<u16>() as u16)) as u32) {
        let c: u16;
        c = *((*UnicodeName).Buffer.add(i as usize));

        if (c >= 'A' as u16) && (c <= 'Z' as u16) {
            Uppers += 1;
        } else if (c >= 'a' as u16) && (c <= 'z' as u16) {
            Lowers += 1;
        } else if (c >= 0x0080) && (*&raw const FatData).CodePageInvariant() != 0 {
            break;
        }

        if c == '.' as u16 {
            *CreateLfn = ((Lowers != 0) && (Uppers != 0)) as u8;
            *AllLowerComponent = ((!(*CreateLfn != 0)) && (Lowers != 0)) as u8;
            ExtensionPresent = 1;

            Uppers = 0;
            Lowers = 0;
        }

        i += 1;
    }

    *CreateLfn = ((*CreateLfn != 0)
                  || ((i as u32) != (((*UnicodeName).Length / (core::mem::size_of::<u16>() as u16)) as u32))
                  || ((Lowers != 0) && (Uppers != 0))) as u8;

    if ExtensionPresent != 0 {
        *AllLowerComponent = ((!(*CreateLfn != 0)) && *AllLowerComponent != 0) as u8;
        *AllLowerExtension = ((!(*CreateLfn != 0)) && (Lowers != 0)) as u8;
    } else {
        *AllLowerComponent = ((!(*CreateLfn != 0)) && (Lowers != 0)) as u8;
        *AllLowerExtension = 0;
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatSpaceInName(
    IrpContext: *mut core::ffi::c_void,
    UnicodeName: *mut _UNICODE_STRING,
) -> i32 {
    // PAGED_CODE();
    let _ = IrpContext;

    let char_count = (*UnicodeName).Length as usize / core::mem::size_of::<u16>();

    for i in 0..char_count {
        if *((*UnicodeName).Buffer).add(i) == ' ' as u16 {
            return 1;
        }
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn FatUnicodeRestoreShortNameCase(
    ShortNameWithCase: *mut _UNICODE_STRING,
    LowerCase8: u8,
    LowerCase3: u8,
) {
    // PAGED_CODE(); omitted
    unsafe {
        let length = (*ShortNameWithCase).Length;
        NT_ASSERT!((length <= 24) as i32);

        let buffer = (*ShortNameWithCase).Buffer;
        let length_bytes = length as usize;
        let wchar_size = core::mem::size_of::<u16>();
        let length_chars = length_bytes / wchar_size;
        let mut i: u16 = 0;
        while i < length_chars as u16 && *buffer.add(i as usize) != 0x002Eu16 {
            i = i.wrapping_add(1);
        }

        if LowerCase8 != 0 {
            let mut DownCaseSeg = _UNICODE_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut(),
            };
            DownCaseSeg.Buffer = buffer;
            let seg_len = (i as usize * wchar_size) as u16;
            DownCaseSeg.Length = seg_len;
            DownCaseSeg.MaximumLength = seg_len;
            RtlDowncaseUnicodeString(core::ptr::addr_of_mut!(DownCaseSeg), core::ptr::addr_of_mut!(DownCaseSeg), 0);
        }

        i = i.wrapping_add(1);

        if LowerCase3 != 0 {
            if (i as usize * wchar_size) < length_bytes {
                let mut DownCaseSeg = _UNICODE_STRING {
                    Length: 0,
                    MaximumLength: 0,
                    Buffer: core::ptr::null_mut(),
                };
                DownCaseSeg.Buffer = buffer.add(i as usize);
                let ext_len = length_bytes - (i as usize * wchar_size);
                DownCaseSeg.Length = ext_len as u16;
                DownCaseSeg.MaximumLength = ext_len as u16;
                RtlDowncaseUnicodeString(core::ptr::addr_of_mut!(DownCaseSeg), core::ptr::addr_of_mut!(DownCaseSeg), 0);
            }
        }
    }
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
//     NameSup.c
//
// Abstract:
//
//     This module implements the Fat Name support routines
//
//
// --*/
//
// #include "fatprocs.h"
//
// #define Dbg                              (DEBUG_TRACE_NAMESUP)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, Fat8dot3ToString)
// #pragma alloc_text(PAGE, FatIsNameInExpression)
// #pragma alloc_text(PAGE, FatStringTo8dot3)
// #pragma alloc_text(PAGE, FatSetFullFileNameInFcb)
// #pragma alloc_text(PAGE, FatGetUnicodeNameFromFcb)
// #pragma alloc_text(PAGE, FatUnicodeToUpcaseOem)
// #pragma alloc_text(PAGE, FatSelectNames)
// #pragma alloc_text(PAGE, FatEvaluateNameCase)
// #pragma alloc_text(PAGE, FatSpaceInName)
// #pragma alloc_text(PAGE, FatUnicodeRestoreShortNameCase)
// #endif
//
//
// BOOLEAN
// FatIsNameInExpression (
//     IN PIRP_CONTEXT IrpContext,
//     IN OEM_STRING Expression,
//     IN OEM_STRING Name
//     )
//
// /*++
//
// Routine Description:
//
//     This routine compare a name and an expression and tells the caller if
//     the name is equal to or not equal to the expression.  The input name
//     cannot contain wildcards, while the expression may contain wildcards.
//
// Arguments:
//
//     Expression - Supplies the input expression to check against
//                  The caller must have already upcased the Expression.
//
//     Name - Supplies the input name to check for.  The caller must have
//            already upcased the name.
//
// Return Value:
//
//     BOOLEAN - TRUE if Name is an element in the set of strings denoted
//         by the input Expression and FALSE otherwise.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     //
//     //  Call the appropriate FsRtl routine do to the real work
//     //
//
//     return FsRtlIsDbcsInExpression( &Expression,
//                                     &Name );
//
//     UNREFERENCED_PARAMETER( IrpContext );
// }
//
//
// VOID
// FatStringTo8dot3 (
//      PIRP_CONTEXT IrpContext,
//      OEM_STRING InputString,
//      PFAT8DOT3 Output8dot3
//     )
//
// /*++
//
// Routine Description:
//
//     Convert a string into fat 8.3 format.  The string must not contain
//     any wildcards.
//
// Arguments:
//
//     InputString - Supplies the input string to convert
//
//     Output8dot3 - Receives the converted string, the memory must be supplied
//         by the caller.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG i;
//     ULONG j;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatStringTo8dot3\n", 0);
//     DebugTrace( 0, Dbg, "InputString = %Z\n", &InputString);
//
//     NT_ASSERT( InputString.Length <= 12 );
//
//     //
//     //  Make the output name all blanks
//     //
//
//     RtlFillMemory( Output8dot3, 11, UCHAR_SP );
//
//     //
//     //  Copy over the first part of the file name.  Stop when we get to
//     //  the end of the input string or a dot.
//     //
//
//     for (i = 0;
//          (i < (ULONG)InputString.Length) && (InputString.Buffer[i] != '.') && (i < 11);
//          i += 1) {
//
//         (*Output8dot3)[i] = InputString.Buffer[i];
//     }
//
//     //
//     //  Check if we need to process an extension
//     //
//
//     if (i < (ULONG)InputString.Length) {
//
//         //
//         //  Make sure we have a dot and then skip over it.
//         //
//
//         NT_ASSERT( (InputString.Length - i) <= 4 );
//         NT_ASSERT( InputString.Buffer[i] == '.' );
//
//         i += 1;
//
//         //
//         //  Copy over the extension.  Stop when we get to the
//         //  end of the input string.
//         //
//
//         for (j = 8; (i < (ULONG)InputString.Length); j += 1, i += 1) {
//
//             (*Output8dot3)[j] = InputString.Buffer[i];
//         }
//     }
//
//     //
//     //  Before we return check if we should translate the first character
//     //  from 0xe5 to 0x5.
//     //
//
//     if ((*Output8dot3)[0] == 0xe5) {
//
//         (*Output8dot3)[0] = FAT_DIRENT_REALLY_0E5;
//     }
//
//     DebugTrace(-1, Dbg, "FatStringTo8dot3 -> (VOID)\n", 0);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
//
// VOID
// Fat8dot3ToString (
//      PIRP_CONTEXT IrpContext,
//      PDIRENT Dirent,
//      BOOLEAN RestoreCase,
//      POEM_STRING OutputString
//     )
//
// /*++
//
// Routine Description:
//
//     Convert fat 8.3 format into a string.  The 8.3 name must be well formed.
//
// Arguments:
//
//     Dirent - Supplies the input 8.3 name to convert
//
//     RestoreCase - If TRUE, then the magic reserved bits are used to restore
//         the original case.
//
//     OutputString - Receives the converted name, the memory must be supplied
//         by the caller.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     ULONG StringIndex;
//     ULONG BaseLength, ExtensionLength;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "Fat8dot3ToString\n", 0);
//
//     //
//     //  First, find the length of the base component.
//     //
//
//     for (BaseLength = 8; BaseLength > 0; BaseLength -= 1) {
//
//         if (Dirent->FileName[BaseLength - 1] != UCHAR_SP) {
//
//             break;
//         }
//     }
//
//     //
//     //  Now find the length of the extension.
//     //
//
//     for (ExtensionLength = 3; ExtensionLength > 0; ExtensionLength -= 1) {
//
//         if (Dirent->FileName[8 + ExtensionLength - 1] != UCHAR_SP) {
//
//             break;
//         }
//     }
//
//     //
//     //  If there was a base part, copy it and check the case.  Don't forget
//     //  if the first character needs to be changed from 0x05 to 0xe5.
//     //
//
//     if (BaseLength != 0) {
//
//         RtlCopyMemory( OutputString->Buffer, Dirent->FileName, BaseLength );
//
//         if (OutputString->Buffer[0] == FAT_DIRENT_REALLY_0E5) {
//
//             OutputString->Buffer[0] = 0xe5;
//         }
//
//         //
//         //  Now if we are to restore case, look for A-Z
//         //
//
//         if (FatData.ChicagoMode &&
//             RestoreCase &&
//             FlagOn(Dirent->NtByte, FAT_DIRENT_NT_BYTE_8_LOWER_CASE)) {
//
//             for (StringIndex = 0; StringIndex < BaseLength; StringIndex += 1) {
//
//                 //
//                 //  Depending on whether the media was built in a system that was
//                 //  running with "code page invariance" (see FatEvaluateNameCase),
//                 //  there could be double-byte OEM characters lying in wait here.
//                 //  Gotta skip them.
//                 //
//
//                 if (FsRtlIsLeadDbcsCharacter(OutputString->Buffer[StringIndex])) {
//
//                     StringIndex += 1;
//                     continue;
//                 }
//
//                 if ((OutputString->Buffer[StringIndex] >= 'A') &&
//                     (OutputString->Buffer[StringIndex] <= 'Z')) {
//
//                     OutputString->Buffer[StringIndex] += 'a' - 'A';
//                 }
//             }
//         }
//     }
//
//     //
//     //  If there was an extension, copy that over.  Else we now know the
//     //  size of the string.
//     //
//
//     if (ExtensionLength != 0) {
//
//         PUCHAR o, d;
//
//         //
//         //  Now add the dot
//         //
//
//         OutputString->Buffer[BaseLength++] = '.';
//
//         //
//         //  Copy over the extension into the output buffer.
//         //
//
//         o = (PUCHAR)&OutputString->Buffer[BaseLength];
//         d = &Dirent->FileName[8];
//
//         switch (ExtensionLength) {
//         case 3:
//             *o++ = *d++;
//         case 2:
//             *o++ = *d++;
//         case 1:
//             *o++ = *d++;
//         }
//
//         //
//         //  Set the output string length
//         //
//
//         OutputString->Length = (USHORT)(BaseLength + ExtensionLength);
//
//         //
//         //  Now if we are to restore case, look for A-Z
//         //
//
//         if (FatData.ChicagoMode &&
//             RestoreCase &&
//             FlagOn(Dirent->NtByte, FAT_DIRENT_NT_BYTE_3_LOWER_CASE)) {
//
//             for (StringIndex = BaseLength;
//                  StringIndex < OutputString->Length;
//                  StringIndex++ ) {
//
//                 //
//                 //  Depending on whether the media was built in a system that was
//                 //  running with "code page invariance" (see FatEvaluateNameCase),
//                 //  there could be double-byte OEM characters lying in wait here.
//                 //  Gotta skip them.
//                 //
//
//                 if (FsRtlIsLeadDbcsCharacter(OutputString->Buffer[StringIndex])) {
//
//                     StringIndex += 1;
//                     continue;
//                 }
//
//                 if ((OutputString->Buffer[StringIndex] >= 'A') &&
//                     (OutputString->Buffer[StringIndex] <= 'Z')) {
//
//                     OutputString->Buffer[StringIndex] += 'a' - 'A';
//                 }
//             }
//         }
//
//     } else {
//
//         //
//         //  Set the output string length
//         //
//
//         OutputString->Length = (USHORT)BaseLength;
//     }
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "Fat8dot3ToString, OutputString = \"%Z\" -> VOID\n", OutputString);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatGetUnicodeNameFromFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN OUT PUNICODE_STRING Lfn
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will return the unicode name for a given Fcb.  If the
//     file has an LFN, it will return this.  Otherwise it will return
//     the UNICODE conversion of the Oem name, properly cased.
//
// Arguments:
//
//     Fcb - Supplies the Fcb to query.
//
//     Lfn - Supplies a string that already has enough storage for the
//         full unicode name.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PDIRENT Dirent;
//     PBCB DirentBcb = NULL;
//     ULONG DirentByteOffset;
//
//     CCB LocalCcb;
//
//     PAGED_CODE();
//
//     NT_ASSERT((MAX_LFN_CHARACTERS * sizeof( WCHAR)) == Lfn->MaximumLength);
//
//     //
//     //  We'll start by locating the dirent for the name.
//     //
//
//     FatStringTo8dot3( IrpContext,
//                       Fcb->ShortName.Name.Oem,
//                       &LocalCcb.OemQueryTemplate.Constant );
//
//     LocalCcb.Flags = 0;
//     LocalCcb.UnicodeQueryTemplate.Length = 0;
//     LocalCcb.ContainsWildCards = FALSE;
//
//     FatLocateDirent( IrpContext,
//                      Fcb->ParentDcb,
//                      &LocalCcb,
//                      Fcb->LfnOffsetWithinDirectory,
//                      NULL,
//                      &Dirent,
//                      &DirentBcb,
//                      (PVBO)&DirentByteOffset,
//                      NULL,
//                      Lfn,
//                      NULL );
//     _SEH2_TRY {
//
//         //
//         //  If we didn't find the Dirent, something is terribly wrong.
//         //
//
//         if ((DirentBcb == NULL) ||
//             (DirentByteOffset != Fcb->DirentOffsetWithinDirectory)) {
//
//             FatRaiseStatus( IrpContext, STATUS_FILE_INVALID );
//         }
//
//         //
//         //  Check for the easy case.
//         //
//
//         if (Lfn->Length == 0) {
//
//             NTSTATUS Status;
//             OEM_STRING ShortName;
//             UCHAR ShortNameBuffer[12];
//
//             //
//             //  If we thought that there was an LFN here and didn't find one,
//             //  we're as dead.  This shouldn't happen in normal operation, but
//             //  if someone scrambles a directory by hand ...
//             //
//
//             NT_ASSERT( Fcb->LfnOffsetWithinDirectory == Fcb->DirentOffsetWithinDirectory );
//
//             if (Fcb->LfnOffsetWithinDirectory != Fcb->DirentOffsetWithinDirectory) {
//
//                 FatRaiseStatus( IrpContext, STATUS_FILE_INVALID );
//             }
//
//             //
//             //  There is no LFN, so manufacture a UNICODE name.
//             //
//
//             ShortName.Length = 0;
//             ShortName.MaximumLength = 12;
//             ShortName.Buffer = (PCHAR)ShortNameBuffer;
//
//             Fat8dot3ToString( IrpContext, Dirent, TRUE, &ShortName );
//
//             //
//             //  OK, now convert this string to UNICODE
//             //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//             Status = RtlOemStringToCountedUnicodeString( Lfn,
//                                                          &ShortName,
//                                                          FALSE );
//
//             NT_ASSERT( Status == STATUS_SUCCESS );
//         }
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//     } _SEH2_END;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatSetFullFileNameInFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     If the FullFileName field in the Fcb has not yet been filled in, we
//     proceed to do so.
//
// Arguments:
//
//     Fcb - Supplies the file.
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
//     if (Fcb->FullFileName.Buffer == NULL) {
//
//         UNICODE_STRING Lfn;
//         PFCB TmpFcb = Fcb;
//         PFCB StopFcb;
//         PWCHAR TmpBuffer;
//         ULONG PathLength = 0;
//
//         //
//         //  We will assume we do this infrequently enough, that it's OK to
//         //  to a pool allocation here.
//         //
//
//         Lfn.Length = 0;
//         Lfn.MaximumLength = MAX_LFN_CHARACTERS * sizeof(WCHAR);
//         Lfn.Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                MAX_LFN_CHARACTERS * sizeof(WCHAR),
//                                                TAG_FILENAME_BUFFER );
//
//         _SEH2_TRY {
//
//             //
//             //  First determine how big the name will be.  If we find an
//             //  ancestor with a FullFileName, our work is easier.
//             //
//
//             while (TmpFcb != Fcb->Vcb->RootDcb) {
//
//                 if ((TmpFcb != Fcb) && (TmpFcb->FullFileName.Buffer != NULL)) {
//
//                     PathLength += TmpFcb->FullFileName.Length;
//
//                     Fcb->FullFileName.Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                                          PathLength,
//                                                                          TAG_FILENAME_BUFFER );
//
//                     RtlCopyMemory( Fcb->FullFileName.Buffer,
//                                    TmpFcb->FullFileName.Buffer,
//                                    TmpFcb->FullFileName.Length );
//
//                     break;
//                 }
//
//                 PathLength += TmpFcb->FinalNameLength + sizeof(WCHAR);
//
//                 TmpFcb = TmpFcb->ParentDcb;
//             }
//
//             //
//             //  If FullFileName.Buffer is still NULL, allocate it.
//             //
//
//             if (Fcb->FullFileName.Buffer == NULL) {
//
//                 Fcb->FullFileName.Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                                      PathLength,
//                                                                      TAG_FILENAME_BUFFER );
//             }
//
//             StopFcb = TmpFcb;
//
//             TmpFcb = Fcb;
//             TmpBuffer =  Fcb->FullFileName.Buffer + PathLength / sizeof(WCHAR);
//
//             Fcb->FullFileName.Length =
//             Fcb->FullFileName.MaximumLength = (USHORT)PathLength;
//
//             while (TmpFcb != StopFcb) {
//
//                 FatGetUnicodeNameFromFcb( IrpContext,
//                                           TmpFcb,
//                                           &Lfn );
//
//                 TmpBuffer -= Lfn.Length / sizeof(WCHAR);
//
//                 RtlCopyMemory( TmpBuffer, Lfn.Buffer, Lfn.Length );
//
//                 TmpBuffer -= 1;
//
//                 *TmpBuffer = L'\\';
//
//                 TmpFcb = TmpFcb->ParentDcb;
//             }
//
//         } _SEH2_FINALLY {
//
//             if (_SEH2_AbnormalTermination()) {
//
//                 if (Fcb->FullFileName.Buffer) {
//
//                     ExFreePool( Fcb->FullFileName.Buffer );
//                     Fcb->FullFileName.Buffer = NULL;
//                 }
//             }
//
//             ExFreePool( Lfn.Buffer );
//         } _SEH2_END;
//     }
// }
//
// VOID
// FatUnicodeToUpcaseOem (
//     IN PIRP_CONTEXT IrpContext,
//     IN POEM_STRING OemString,
//     IN PUNICODE_STRING UnicodeString
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is our standard routine for trying to use stack space
//     if possible when calling RtlUpcaseUnicodeStringToCountedOemString().
//
//     If an unmappable character is encountered, we set the destination
//     length to 0.
//
// Arguments:
//
//     OemString - Specifies the destination string.  Space is already assumed to
//         be allocated.  If there is not enough, then we allocate enough
//         space.
//
//     UnicodeString - Specifies the source string.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     Status = RtlUpcaseUnicodeStringToCountedOemString( OemString,
//                                                        UnicodeString,
//                                                        FALSE );
//
//     if (Status == STATUS_BUFFER_OVERFLOW) {
//
//         OemString->Buffer = NULL;
//         OemString->Length = 0;
//         OemString->MaximumLength = 0;
//
//         Status = RtlUpcaseUnicodeStringToCountedOemString( OemString,
//                                                            UnicodeString,
//                                                            TRUE );
//     }
//
//     if (!NT_SUCCESS(Status)) {
//
//         if (Status == STATUS_UNMAPPABLE_CHARACTER) {
//
//             OemString->Length = 0;
//
//         } else {
//
//             FatNormalizeAndRaiseStatus( IrpContext, Status );
//         }
//     }
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatSelectNames (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Parent,
//     IN POEM_STRING OemName,
//     IN PUNICODE_STRING UnicodeName,
//     IN OUT POEM_STRING ShortName,
//     IN PUNICODE_STRING SuggestedShortName OPTIONAL,
//     IN OUT BOOLEAN *AllLowerComponent,
//     IN OUT BOOLEAN *AllLowerExtension,
//     IN OUT BOOLEAN *CreateLfn
//     )
//
// /*++
//
// Routine Description:
//
//     This routine takes the original UNICODE string that the user specified,
//     and the upcased Oem equivalent.  This routine then decides if the OemName
//     is acceptable for dirent, or whether a short name has to be manufactured.
//
//     Two values are returned to the caller.  One tells the caller if the name
//     happens to be all lower case < 0x80.  In this special case we don't
//     have to create an Lfn.  Also we tell the caller if it must create an LFN.
//
// Arguments:
//
//     OemName -  Supplies the proposed short Oem name.
//
//     ShortName - If this OemName is OK for storeage in a dirent it is copied to
//         this string, otherwise this string is filled with a name that is OK.
//         If OemName and ShortName are the same string, no copy is done.
//
//     UnicodeName - Provides the original final name.
//
//     SuggestedShortName - a first-try short name to try before auto-generation
//         is used
//
//     AllLowerComponent - Returns whether this component was all lower case.
//
//     AllLowerExtension - Returns wheather the extension was all lower case.
//
//     CreateLfn - Tells the caller if we must create an LFN for the UnicodeName or
//         SuggestedLongName
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     BOOLEAN GenerateShortName;
//
//     PAGED_CODE();
//
//     //
//     //  First see if we must generate a short name.
//     //
//
//     if ((OemName->Length == 0) ||
//         !FatIsNameShortOemValid( IrpContext, *OemName, FALSE, FALSE, FALSE ) ||
//         FatSpaceInName( IrpContext, UnicodeName )) {
//
//         WCHAR ShortNameBuffer[12];
//         UNICODE_STRING ShortUnicodeName;
//         GENERATE_NAME_CONTEXT Context;
//         BOOLEAN TrySuggestedShortName;
//
//         PDIRENT Dirent;
//         PBCB Bcb = NULL;
//         ULONG ByteOffset;
//         NTSTATUS Status;
//
//         GenerateShortName = TRUE;
//
//         TrySuggestedShortName = (SuggestedShortName != NULL);
//
//         //
//         //  Now generate a short name.
//         //
//
//         ShortUnicodeName.Length = 0;
//         ShortUnicodeName.MaximumLength = 12 * sizeof(WCHAR);
//         ShortUnicodeName.Buffer = ShortNameBuffer;
//
//         RtlZeroMemory( &Context, sizeof( GENERATE_NAME_CONTEXT ) );
//
//         _SEH2_TRY {
//
//             while ( TRUE ) {
//
//                 FatUnpinBcb( IrpContext, Bcb );
//
//                 if (TrySuggestedShortName) {
//
//                     //
//                     //  Try our caller's candidate first. Note that this must have
//                     //  been uppercased previously.
//                     //
//
//                     ShortUnicodeName.Length = SuggestedShortName->Length;
//                     ShortUnicodeName.MaximumLength = SuggestedShortName->MaximumLength;
//                     ShortUnicodeName.Buffer = SuggestedShortName->Buffer;
//
//                     TrySuggestedShortName = FALSE;
//
//                 } else {
//
//                     RtlGenerate8dot3Name( UnicodeName, TRUE, &Context, &ShortUnicodeName );
//                 }
//
//                 //
//                 //  We have a candidate, make sure it doesn't exist.
//                 //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//                 Status = RtlUnicodeStringToCountedOemString( ShortName,
//                                                              &ShortUnicodeName,
//                                                              FALSE );
//
//                 NT_ASSERT( Status == STATUS_SUCCESS );
//
//                 FatLocateSimpleOemDirent( IrpContext,
//                                           Parent,
//                                           ShortName,
//                                           &Dirent,
//                                           &Bcb,
//                                           (PVBO)&ByteOffset );
//
//                 if (Bcb == NULL) {
//
//                     _SEH2_LEAVE;
//
//                 }
//             }
//
//         } _SEH2_FINALLY {
//
//             FatUnpinBcb( IrpContext, Bcb );
//         } _SEH2_END;
//
//     } else {
//
//         //
//         //  Only do this copy if the two string are indeed different.
//         //
//
//         if (ShortName != OemName) {
//             ShortName->Length = OemName->Length;
//
//             //
//             // If FsRtlIsFatDbcsLegal() on OemName fails, we will not
//             // be in this code path, so we infer that ShortName's
//             // buffer is big enough to hold the full FAT file name in
//             // OemName.
//             //
//
//             _Analysis_assume_(ShortName->MaximumLength <= OemName->Length);
//
//             RtlCopyMemory( ShortName->Buffer, OemName->Buffer, OemName->Length );
//         }
//
//         GenerateShortName = FALSE;
//     }
//
//     //
//     //  Now see if the caller will have to use unicode string as an LFN
//     //
//
//     if (GenerateShortName) {
//
//         *CreateLfn = TRUE;
//         *AllLowerComponent = FALSE;
//         *AllLowerExtension = FALSE;
//
//     } else {
//
//         FatEvaluateNameCase( IrpContext,
//                              UnicodeName,
//                              AllLowerComponent,
//                              AllLowerExtension,
//                              CreateLfn );
//     }
//
//     return;
// }
//
//
// VOID
// FatEvaluateNameCase (
//     IN PIRP_CONTEXT IrpContext,
//     IN PUNICODE_STRING UnicodeName,
//     IN OUT BOOLEAN *AllLowerComponent,
//     IN OUT BOOLEAN *AllLowerExtension,
//     IN OUT BOOLEAN *CreateLfn
//     )
//
// /*++
//
// Routine Description:
//
//     This routine takes a UNICODE string and sees if it is eligible for
//     the special case optimization.
//
// Arguments:
//
//     UnicodeName - Provides the original final name.
//
//     AllLowerComponent - Returns whether this compoent was all lower case.
//
//     AllLowerExtension - Returns wheather the extension was all lower case.
//
//     CreateLfn - Tells the call if we must create an LFN for the UnicodeName.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG i;
//     UCHAR Uppers = 0;
//     UCHAR Lowers = 0;
//
//     BOOLEAN ExtensionPresent = FALSE;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     *CreateLfn = FALSE;
//
//     for (i = 0; i < UnicodeName->Length / sizeof(WCHAR); i++) {
//
//         WCHAR c;
//
//         c = UnicodeName->Buffer[i];
//
//         if ((c >= 'A') && (c <= 'Z')) {
//
//             Uppers += 1;
//
//         } else if ((c >= 'a') && (c <= 'z')) {
//
//             Lowers += 1;
//
//         } else if ((c >= 0x0080) && FatData.CodePageInvariant) {
//
//             break;
//         }
//
//         //
//         //  If we come to a period, figure out if the extension was
//         //  all one case.
//         //
//
//         if (c == L'.') {
//
//             *CreateLfn = (Lowers != 0) && (Uppers != 0);
//
//             *AllLowerComponent = !(*CreateLfn) && (Lowers != 0);
//
//             ExtensionPresent = TRUE;
//
//             //
//             //  Now reset the uppers and lowers count.
//             //
//
//             Uppers = Lowers = 0;
//         }
//     }
//
//     //
//     //  Now check again for creating an LFN.
//     //
//
//     *CreateLfn = (*CreateLfn ||
//                   (i != UnicodeName->Length / sizeof(WCHAR)) ||
//                   ((Lowers != 0) && (Uppers != 0)));
//
//     //
//     //  Now we know the final state of CreateLfn, update the two
//     //  "AllLower" booleans.
//     //
//
//     if (ExtensionPresent) {
//
//         *AllLowerComponent = !(*CreateLfn) && *AllLowerComponent;
//         *AllLowerExtension = !(*CreateLfn) && (Lowers != 0);
//
//     } else {
//
//         *AllLowerComponent = !(*CreateLfn) && (Lowers != 0);
//         *AllLowerExtension = FALSE;
//     }
//
//     return;
// }
//
//
// BOOLEAN
// FatSpaceInName (
//     IN PIRP_CONTEXT IrpContext,
//     IN PUNICODE_STRING UnicodeName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine takes a UNICODE string and sees if it contains any spaces.
//
// Arguments:
//
//     UnicodeName - Provides the final name.
//
// Return Value:
//
//     BOOLEAN - TRUE if it does, FALSE if it doesn't.
//
// --*/
//
// {
//     ULONG i;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     for (i=0; i < UnicodeName->Length/sizeof(WCHAR); i++) {
//
//         if (UnicodeName->Buffer[i] == L' ') {
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// VOID
// FatUnicodeRestoreShortNameCase(
//     IN PUNICODE_STRING ShortNameWithCase,
//     IN BOOLEAN LowerCase8,
//     IN BOOLEAN LowerCase3
//     )
//
// /*++
//
// Routine Description:
//
//     Given an 8.3 filename in a UNICODE_STRING, fix the case of the
//     name given the two 8do3 case flags.
//
// Arguments:
//
//     ShortNameWithCase - the UNICODE_STRING containing the short name.
//     LowerCase8, LowerCase3 - the flag indicating whether to downcase the 8dot3 name component.
//
// Return Value:
//
//     None.
//
// --*/
// {
//     USHORT i;
//     UNICODE_STRING DownCaseSeg;
//
//     PAGED_CODE();
//
//     NT_ASSERT( ShortNameWithCase->Length <= 24 );
//
//     //
//     //  Have to repair the case of the short name
//     //
//
//     for (i = 0; i < (ShortNameWithCase->Length/sizeof(WCHAR)) &&
//                 ShortNameWithCase->Buffer[i] != L'.'; i++);
//
//     //
//     //  Now pointing at the '.', or otherwise the end of name component
//     //
//
//     if (LowerCase8) {
//
//         DownCaseSeg.Buffer = ShortNameWithCase->Buffer;
//         DownCaseSeg.MaximumLength = DownCaseSeg.Length = i*sizeof(WCHAR);
//
//         RtlDowncaseUnicodeString(&DownCaseSeg, &DownCaseSeg, FALSE);
//     }
//
//     i++;
//
//     //
//     //  Now pointing at first wchar of the extension.
//     //
//
//     if (LowerCase3) {
//
//         //
//         //  It is not neccesarily the case that we can rely on the flag
//         //  indicating that we really have an extension.
//         //
//
//         if ((i*sizeof(WCHAR)) < ShortNameWithCase->Length) {
//             DownCaseSeg.Buffer = &ShortNameWithCase->Buffer[i];
//             DownCaseSeg.MaximumLength = DownCaseSeg.Length = ShortNameWithCase->Length - i*sizeof(WCHAR);
//
//             RtlDowncaseUnicodeString(&DownCaseSeg, &DownCaseSeg, FALSE);
//         }
//     }
//
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
