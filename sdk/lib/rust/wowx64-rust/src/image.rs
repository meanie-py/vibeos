//! The guest's own amd64 ntdll: mapped from `<SystemRoot>\sysamd64`, rebased by
//! hand, and searched for the loader entry points wowx64.dll starts the guest
//! through. PE32+ is the same shape on both architectures, so this is plain
//! header arithmetic — but on an image that is NOT a loader entry (the native
//! loader never saw it), so nothing here may go through `Ldr*`.

use core::ptr::null_mut;

use ntdll_sys::nt::*;

use crate::host::{die, CURRENT_PROCESS};

const STATUS_IMAGE_NOT_AT_BASE: NTSTATUS = 0x4000_0003;
const SEC_IMAGE: ULONG = 0x0100_0000;
const PAGE_EXECUTE: ULONG = 0x10;
const PAGE_READWRITE: ULONG = 0x04;
const VIEW_SHARE: u32 = 1;
const OBJ_CASE_INSENSITIVE: ULONG = 0x40;
const FILE_SHARE_READ: ULONG = 1;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x20;
const FILE_NON_DIRECTORY_FILE: ULONG = 0x40;
const GENERIC_READ: ULONG = 0x8000_0000;
const GENERIC_EXECUTE: ULONG = 0x2000_0000;
const SYNCHRONIZE: ULONG = 0x0010_0000;
const SECTION_QUERY: ULONG = 0x1;
const SECTION_MAP_READ: ULONG = 0x4;
const SECTION_MAP_EXECUTE: ULONG = 0x8;

const IMAGE_DIRECTORY_ENTRY_EXPORT: usize = 0;
const IMAGE_DIRECTORY_ENTRY_BASERELOC: usize = 5;
const IMAGE_REL_BASED_ABSOLUTE: u16 = 0;
const IMAGE_REL_BASED_DIR64: u16 = 10;

/// The guest system directory, under `<SystemRoot>`. Ten characters with its
/// slashes, the same as `\system32\`, so the path redirect in `syscall.rs` is
/// an in-place same-length rewrite. Named after the machine, as Windows on ARM
/// names its 32-bit ARM directory `SysArm32`.
pub const GUEST_SYSDIR: &[u8; 10] = b"\\sysamd64\\";

pub struct GuestImage {
    pub Base: *mut u8,
    pub Size: usize,
}

unsafe fn rd16(p: *const u8) -> u16 {
    core::ptr::read_unaligned(p as *const u16)
}
unsafe fn rd32(p: *const u8) -> u32 {
    core::ptr::read_unaligned(p as *const u32)
}

/// Offset of the optional header's data directory `Index` (RVA, size), or None
/// past `NumberOfRvaAndSizes`.
unsafe fn DataDirectory(Base: *const u8, Index: usize) -> Option<(u32, u32)> {
    let Nt = Base.add(rd32(Base.add(0x3C)) as usize);
    let Opt = Nt.add(24);
    if rd16(Opt) != 0x20B {
        return None; // not PE32+
    }
    if Index as u32 >= rd32(Opt.add(108)) {
        return None;
    }
    let Dir = Opt.add(112 + Index * 8);
    Some((rd32(Dir), rd32(Dir.add(4))))
}

/// The image's preferred base (`OptionalHeader.ImageBase`, PE32+).
unsafe fn PreferredBase(Base: *const u8) -> u64 {
    let Nt = Base.add(rd32(Base.add(0x3C)) as usize);
    core::ptr::read_unaligned(Nt.add(24 + 24) as *const u64)
}

/// Map `<SystemRoot>\sysamd64\ntdll.dll` as an image. Dies on any failure: no
/// guest process can start without it, and the stage that should have put it
/// there is the thing to go fix.
pub unsafe fn MapGuestNtdll() -> GuestImage {
    // "\??\" + SystemRoot (KUSER_SHARED_DATA+0x30) + "\sysamd64\ntdll.dll"
    let mut Path = [0u16; 300];
    let mut n = 0usize;
    for &c in b"\\??\\" {
        Path[n] = c as u16;
        n += 1;
    }
    let SysRoot = (0x7FFE_0030usize) as *const u16;
    let mut i = 0;
    while *SysRoot.add(i) != 0 && n < 260 {
        Path[n] = *SysRoot.add(i);
        n += 1;
        i += 1;
    }
    for &c in GUEST_SYSDIR.iter().chain(b"ntdll.dll".iter()) {
        Path[n] = c as u16;
        n += 1;
    }
    Path[n] = 0;

    let mut Name: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut Name, Path.as_ptr());
    let mut Oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    Oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    Oa.ObjectName = &mut Name;
    Oa.Attributes = OBJ_CASE_INSENSITIVE;
    let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut File: HANDLE = null_mut();
    let Status = NtOpenFile(
        &mut File,
        GENERIC_READ | GENERIC_EXECUTE | SYNCHRONIZE,
        &mut Oa,
        &mut Iosb,
        FILE_SHARE_READ,
        FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if Status < 0 {
        crate::trace!("open %wZ: %08lx — is the amd64 guest DLL set staged?", &Name as *const UNICODE_STRING, Status);
        die(b"guest ntdll.dll open\0");
    }

    let mut Section: HANDLE = null_mut();
    let Status = NtCreateSection(
        &mut Section,
        SECTION_MAP_READ | SECTION_MAP_EXECUTE | SECTION_QUERY,
        null_mut(),
        null_mut(),
        PAGE_EXECUTE,
        SEC_IMAGE,
        File,
    );
    NtClose(File);
    if Status < 0 {
        crate::trace!("NtCreateSection(guest ntdll) %08lx", Status);
        die(b"guest ntdll.dll section\0");
    }

    let mut Base: PVOID = null_mut();
    let mut ViewSize: SIZE_T = 0;
    let Status = NtMapViewOfSection(
        Section,
        CURRENT_PROCESS,
        &mut Base,
        0,
        0,
        null_mut(),
        &mut ViewSize,
        VIEW_SHARE as _,
        0,
        PAGE_EXECUTE,
    );
    NtClose(Section);
    if Status < 0 {
        crate::trace!("NtMapViewOfSection(guest ntdll) %08lx", Status);
        die(b"guest ntdll.dll map\0");
    }

    let Image = GuestImage { Base: Base as *mut u8, Size: ViewSize as usize };
    // The guest ntdll's preferred base is the native one's (both are linked at
    // the same address), so it essentially never lands there, and nothing else
    // will rebase it: its own loader assumes the kernel put it at its base, as
    // the kernel always does for the real ntdll.
    let Delta = (Base as u64).wrapping_sub(PreferredBase(Image.Base));
    if Status == STATUS_IMAGE_NOT_AT_BASE || Delta != 0 {
        Relocate(&Image, Delta);
    }
    crate::trace!("guest ntdll.dll mapped at %p size %p (rebased by %p)", Base, ViewSize as PVOID, Delta as PVOID);
    Image
}

/// Apply the image's base relocations for `Delta`. amd64 images carry only
/// DIR64 (and ABSOLUTE padding); any other type is a malformed or foreign image
/// and fails loud rather than half-relocating.
unsafe fn Relocate(Image: &GuestImage, Delta: u64) {
    let (Rva, Size) = match DataDirectory(Image.Base, IMAGE_DIRECTORY_ENTRY_BASERELOC) {
        Some(d) if d.0 != 0 && d.1 != 0 => d,
        _ => die(b"guest ntdll.dll is not at its base and has no relocations\0"),
    };
    let mut Block = Image.Base.add(Rva as usize);
    let End = Block.add(Size as usize);
    while Block < End {
        let PageRva = rd32(Block);
        let BlockSize = rd32(Block.add(4)) as usize;
        if BlockSize < 8 || Block.add(BlockSize) > End || PageRva as usize >= Image.Size {
            die(b"guest ntdll.dll relocation block out of range\0");
        }

        // Pages are mapped with their section's protection; open the page for
        // the write and put the protection back after.
        let mut ProtBase = Image.Base.add(PageRva as usize) as PVOID;
        let mut ProtSize: SIZE_T = 0x2000; // a DIR64 can straddle into the next page
        let mut Old: ULONG = 0;
        if NtProtectVirtualMemory(CURRENT_PROCESS, &mut ProtBase, &mut ProtSize, PAGE_READWRITE, &mut Old) < 0 {
            die(b"guest ntdll.dll relocation: unprotect\0");
        }

        let mut Entry = Block.add(8);
        while Entry < Block.add(BlockSize) {
            let E = rd16(Entry);
            let Kind = E >> 12;
            let Off = (E & 0xFFF) as usize;
            match Kind {
                IMAGE_REL_BASED_ABSOLUTE => {}
                IMAGE_REL_BASED_DIR64 => {
                    let P = Image.Base.add(PageRva as usize + Off) as *mut u64;
                    core::ptr::write_unaligned(P, core::ptr::read_unaligned(P).wrapping_add(Delta));
                }
                _ => {
                    crate::trace!("relocation type %u at rva %x", Kind as u32, PageRva + Off as u32);
                    die(b"guest ntdll.dll has a non-amd64 relocation type\0");
                }
            }
            Entry = Entry.add(2);
        }

        let mut Ignored: ULONG = 0;
        NtProtectVirtualMemory(CURRENT_PROCESS, &mut ProtBase, &mut ProtSize, Old, &mut Ignored);
        Block = Block.add(BlockSize);
    }
}

/// Address of the named export of a mapped image, bounds-checked against the
/// view (the view is a file someone staged, not trusted data).
pub unsafe fn FindExport(Image: &GuestImage, Name: &[u8]) -> Option<u64> {
    let (Rva, _) = DataDirectory(Image.Base, IMAGE_DIRECTORY_ENTRY_EXPORT)?;
    let Size = Image.Size;
    if Rva == 0 || Rva as usize + 40 > Size {
        return None;
    }
    let Exp = Image.Base.add(Rva as usize);
    let NumberOfFunctions = rd32(Exp.add(20)) as usize;
    let NumberOfNames = rd32(Exp.add(24)) as usize;
    let Functions = rd32(Exp.add(28)) as usize;
    let Names = rd32(Exp.add(32)) as usize;
    let Ordinals = rd32(Exp.add(36)) as usize;
    if Names + NumberOfNames * 4 > Size || Ordinals + NumberOfNames * 2 > Size || Functions + NumberOfFunctions * 4 > Size {
        return None;
    }
    let Want = &Name[..Name.len() - (Name.last() == Some(&0)) as usize];
    for i in 0..NumberOfNames {
        let NameRva = rd32(Image.Base.add(Names + i * 4)) as usize;
        if NameRva + Want.len() >= Size {
            continue;
        }
        let Candidate = core::slice::from_raw_parts(Image.Base.add(NameRva), Want.len() + 1);
        if &Candidate[..Want.len()] == Want && Candidate[Want.len()] == 0 {
            let Ord = rd16(Image.Base.add(Ordinals + i * 2)) as usize;
            if Ord >= NumberOfFunctions {
                return None;
            }
            let FnRva = rd32(Image.Base.add(Functions + Ord * 4)) as usize;
            if FnRva == 0 || FnRva >= Size {
                return None;
            }
            return Some(Image.Base as u64 + FnRva as u64);
        }
    }
    None
}
