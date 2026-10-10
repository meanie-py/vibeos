#[allow(unused_imports)] use crate::{ERR, TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WINDOWS; // winldr.c: DBG_DEFAULT_CHANNEL(WINDOWS)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{
    LARGE_INTEGER, PLOADER_PARAMETER_BLOCK, PULONG64,
    HARDWARE_PTE, PHARDWARE_PTE,
};
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderMemoryData as LoaderMemoryData;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderStartupPcrPage as LoaderStartupPcrPage;

// ---------------------------------------------------------------------------
// amd64 file-scope types restored faithfully from ndk/amd64/ketypes.h:
//   #define KGDTENTRY  KGDTENTRY64
//   #define PKGDTENTRY PKGDTENTRY64
//   #define KIDTENTRY  KIDTENTRY64
//   #define KTSS       KTSS64
// (rustconv dropped these macro-aliases; restore as file-local type aliases.)
use ntkrnl_sys::ddk::{
    _KGDTENTRY64, _KIDTENTRY64, _KTSS64, _KDESCRIPTOR,
};
#[allow(non_camel_case_types)]
type KGDTENTRY = _KGDTENTRY64;
#[allow(non_camel_case_types)]
type PKGDTENTRY = *mut _KGDTENTRY64;
#[allow(non_camel_case_types)]
type KIDTENTRY = _KIDTENTRY64;
#[allow(non_camel_case_types)]
type KTSS = _KTSS64;
#[allow(non_camel_case_types)]
type KDESCRIPTOR = _KDESCRIPTOR;
// rustconv emitted bindgen-underscore spellings at some use sites; alias them.
#[allow(non_camel_case_types)]
type _KGDTENTRY = _KGDTENTRY64;
// MMPTE_HARDWARE — amd64 winldr.c uses PHARDWARE_PTE throughout; the lone
// rustconv `as *mut MMPTE_HARDWARE` cast site is the PML4 page and is a
// PHARDWARE_PTE in C. Alias MMPTE_HARDWARE to the HARDWARE_PTE bitfield
// layout so the cast keeps the .Valid()/.set_Valid() accessor surface.
#[allow(non_camel_case_types)]
type MMPTE_HARDWARE = HARDWARE_PTE;

// ndk/amd64/ketypes.h GDT selector constants
const KGDT64_NULL: u16 = 0x0000;
const KGDT64_R0_CODE: u16 = 0x0010;
const KGDT64_R0_DATA: u16 = 0x0018;
const KGDT64_R3_CMCODE: u16 = 0x0020;
const KGDT64_R3_DATA: u16 = 0x0028;
const KGDT64_R3_CODE: u16 = 0x0030;
const KGDT64_SYS_TSS: u16 = 0x0040;
const KGDT64_R3_CMTEB: u16 = 0x0050;
// ndk/amd64/ketypes.h
const I386_TSS: u32 = 0x9;
const MACHINE_TYPE_ISA: u32 = 0x0000;
const APIC_BASE: u64 = 0xFFFFFFFFFFFE0000;
const MM_HAL_VA_START: u64 = 0xFFFFFFFFFFC00000;
// boot/freeldr/freeldr/ntldr/winldr.h
const NUM_GDT: u32 = 128;
const NUM_IDT: u32 = 0x100;
// sdk/include/xdk/amd64/mm.h
const PXE_BASE: u64 = 0xFFFFF6FB7DBED000;

// amd64 winldr.c file-scope globals (rustconv dropped these; owned here).
static mut PxeBase: PHARDWARE_PTE = core::ptr::null_mut();
static mut GdtIdt: PVOID = core::ptr::null_mut();
static mut SharedUserDataPfn: PFN_NUMBER = 0;
static mut TssBasePage: ULONG_PTR = 0;

// ---------------------------------------------------------------------------
// amd64 arch intrinsics (intrin.h / internal/amd64/intrin_i.h). rustconv
// dropped these to `extern "C"`; restore as file-local core::arch::asm!
// implementations (precedent: winldr_i386.rs __lgdt_i386/__sidt_i386/...).
#[inline]
unsafe fn _disable() {
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn __writeeflags(flags: ULONG_PTR) {
    core::arch::asm!("push {0}; popfq", in(reg) flags, options(nomem));
}
#[inline]
unsafe fn __writecr3(val: ULONG64) {
    core::arch::asm!("mov cr3, {0}", in(reg) val, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn __readmsr(msr: u32) -> u64 {
    let (lo, hi): (u32, u32);
    core::arch::asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi,
                     options(nomem, nostack, preserves_flags));
    ((hi as u64) << 32) | (lo as u64)
}
#[inline]
unsafe fn __lgdt(descriptor: PVOID) {
    core::arch::asm!("lgdt [{0}]", in(reg) descriptor, options(nostack));
}
#[inline]
unsafe fn __lidt(descriptor: PVOID) {
    core::arch::asm!("lidt [{0}]", in(reg) descriptor, options(nostack));
}
#[inline]
unsafe fn __sidt(descriptor: PVOID) {
    core::arch::asm!("sidt [{0}]", in(reg) descriptor, options(nostack));
}
#[inline]
unsafe fn __ltr(selector: USHORT) {
    core::arch::asm!("ltr {0:x}", in(reg) selector, options(nomem, nostack, preserves_flags));
}

// internal/amd64/intrin_i.h FORCEINLINE GDT helpers (precedent:
// winldr_i386.rs KiGetGdtEntry / KiSetGdtEntry, adapted to KGDTENTRY64).
//
//   KiGetGdtEntry(Base, Selector) -> (PKGDTENTRY)((ULONG_PTR)Base + (Selector & ~RPL_MASK))
const RPL_MASK: u16 = 0x0003;
#[inline]
unsafe fn KiGetGdtEntry(Base: PVOID, Selector: u16) -> PKGDTENTRY {
    ((Base as usize) + ((Selector & !RPL_MASK) as usize)) as PKGDTENTRY
}
//   KiInitGdtEntry(Entry, Base, Size, Type, Dpl) — amd64 64-bit system
//   descriptor (TSS). Faithful to internal/amd64/intrin_i.h.
#[inline]
unsafe fn KiInitGdtEntry(Entry: PKGDTENTRY, Base: u64, Size: ULONG, Type_: u32, Dpl: u32) {
    let e = &mut (*Entry).__bindgen_anon_1;
    e.BaseLow = (Base & 0xffff) as USHORT;
    e.__bindgen_anon_1.Bytes.BaseMiddle = ((Base >> 16) & 0xff) as UCHAR;
    e.__bindgen_anon_1.Bytes.BaseHigh = ((Base >> 24) & 0xff) as UCHAR;
    e.BaseUpper = (Base >> 32) as ULONG;
    e.LimitLow = ((Size - 1) & 0xffff) as USHORT;
    // Flags1: Type (bits 0-4), Dpl (bits 5-6), Present (bit 7)
    e.__bindgen_anon_1.Bytes.Flags1 =
        ((Type_ & 0x1f) | ((Dpl & 0x3) << 5) | 0x80) as UCHAR;
    // Flags2: limit high nibble (bits 0-3), no granularity for TSS
    e.__bindgen_anon_1.Bytes.Flags2 = (((Size - 1) >> 16) & 0x0f) as UCHAR;
    e.MustBeZero = 0;
}

// boot/freeldr/freeldr/include/arch/amd64/amd64.h macros (rustconv dropped
// these to extern "C"; they are C macros with no symbol — restore file-local).
const PXI_SHIFT: u32 = 39;
const PPI_SHIFT: u32 = 30;
const PDI_SHIFT: u32 = 21;
const PTI_SHIFT: u32 = 12;
const PAGE_SHIFT_AMD64: u32 = 12;
#[inline]
unsafe fn PtrToPfn(p: *mut core::ffi::c_void) -> u64 {
    (((p as u64) >> PAGE_SHIFT_AMD64) & 0xfffffff) as u64
}
#[inline]
unsafe fn VAtoPXI(va: u64) -> u32 {
    (((va) >> PXI_SHIFT) & 0x1FF) as u32
}
#[inline]
unsafe fn VAtoPPI(va: u64) -> ULONG {
    (((va) >> PPI_SHIFT) & 0x1FF) as ULONG
}
#[inline]
unsafe fn VAtoPDI(va: u64) -> ULONG {
    (((va) >> PDI_SHIFT) & 0x1FF) as ULONG
}
#[inline]
unsafe fn VAtoPTI(va: u64) -> ULONG {
    (((va) >> PTI_SHIFT) & 0x1FF) as ULONG
}

extern "C" {
    fn MmAllocateMemoryWithType(size: usize, type_: u32) -> *mut core::ffi::c_void;
    fn __cpuid(info: *mut i32, function: i32);
    fn DbgPrint(fmt: *const i8, ...);
    fn UiMessageBox(msg: *const u8);
}

unsafe fn MempAllocatePageTables() -> u8 {

    TRACE!(">>> MempAllocatePageTables\n");

    PxeBase = MmAllocateMemoryWithType(PAGE_SIZE as usize, LoaderMemoryData) as *mut MMPTE_HARDWARE;
    if PxeBase.is_null() {
        ERR!("failed to allocate PML4\n");
        return 0u8;
    }

    RtlZeroMemory(PxeBase as *mut core::ffi::c_void, PAGE_SIZE as usize);

    let idx = VAtoPXI(PXE_BASE) as usize;
    (*PxeBase.add(idx)).set_Valid(1);
    (*PxeBase.add(idx)).set_Write(1);
    (*PxeBase.add(idx)).set_PageFrameNumber(PtrToPfn(PxeBase as *mut core::ffi::c_void));

    TRACE!(">>> leave MempAllocatePageTables\n");

    return 1u8;
}

unsafe fn MempGetOrCreatePageDir(PdeBase: PHARDWARE_PTE, Index: u32) -> PHARDWARE_PTE {
    let SubDir: PHARDWARE_PTE;

    if PdeBase.is_null() {
        return core::ptr::null_mut();
    }

    if (*PdeBase.offset(Index as isize)).Valid() == 0 {
        SubDir = MmAllocateMemoryWithType(PAGE_SIZE as usize, LoaderMemoryData) as PHARDWARE_PTE;
        if SubDir.is_null() {
            return core::ptr::null_mut();
        }
        RtlZeroMemory(SubDir as *mut core::ffi::c_void, PAGE_SIZE as usize);
        (*PdeBase.offset(Index as isize)).set_PageFrameNumber(PtrToPfn(SubDir as *mut core::ffi::c_void));
        (*PdeBase.offset(Index as isize)).set_Valid(1);
        (*PdeBase.offset(Index as isize)).set_Write(1);
    } else {
        SubDir = ((*PdeBase.offset(Index as isize)).PageFrameNumber() * PAGE_SIZE as u64) as *mut core::ffi::c_void as PHARDWARE_PTE;
    }
    SubDir
}

unsafe fn MempMapSinglePage(VirtualAddress: u64, PhysicalAddress: u64) -> u8 {
    let PpeBase: PHARDWARE_PTE;
    let PdeBase: PHARDWARE_PTE;
    let PteBase: PHARDWARE_PTE;
    let Index: u32;

    PpeBase = MempGetOrCreatePageDir(PxeBase, VAtoPXI(VirtualAddress));
    PdeBase = MempGetOrCreatePageDir(PpeBase, VAtoPPI(VirtualAddress));
    PteBase = MempGetOrCreatePageDir(PdeBase, VAtoPDI(VirtualAddress));

    if PteBase.is_null() {
        ERR!("!!!No Dir %p, %p, %p, %p\n", (PxeBase) as usize, (PpeBase) as usize, (PdeBase) as usize, (PteBase) as usize);
        return 0u8;
    }

    Index = VAtoPTI(VirtualAddress);
    if (*PteBase.offset(Index as isize)).Valid() != 0 {
        ERR!("!!!Already mapped %ld\n", (Index) as usize);
        return 0u8;
    }

    (*PteBase.offset(Index as isize)).set_Valid(1);
    (*PteBase.offset(Index as isize)).set_Write(1);
    (*PteBase.offset(Index as isize)).set_PageFrameNumber(PhysicalAddress / PAGE_SIZE as u64);

    return 1u8;
}

unsafe fn MempIsPageMapped(VirtualAddress: PVOID) -> BOOLEAN {
    let PpeBase: PHARDWARE_PTE;
    let PdeBase: PHARDWARE_PTE;
    let PteBase: PHARDWARE_PTE;
    let mut Index: ULONG;

    Index = VAtoPXI(VirtualAddress as u64);
    if (*PxeBase.offset(Index as isize)).Valid() == 0 {
        return 0u8;
    }

    PpeBase = ((*PxeBase.offset(Index as isize)).PageFrameNumber() as u64 * PAGE_SIZE as u64) as usize as PVOID as PHARDWARE_PTE;
    Index = VAtoPPI(VirtualAddress as u64);
    if (*PpeBase.offset(Index as isize)).Valid() == 0 {
        return 0u8;
    }

    PdeBase = ((*PpeBase.offset(Index as isize)).PageFrameNumber() as u64 * PAGE_SIZE as u64) as usize as PVOID as PHARDWARE_PTE;
    Index = VAtoPDI(VirtualAddress as u64);
    if (*PdeBase.offset(Index as isize)).Valid() == 0 {
        return 0u8;
    }

    PteBase = ((*PdeBase.offset(Index as isize)).PageFrameNumber() as u64 * PAGE_SIZE as u64) as usize as PVOID as PHARDWARE_PTE;
    Index = VAtoPTI(VirtualAddress as u64);
    if (*PteBase.offset(Index as isize)).Valid() == 0 {
        return 0u8;
    }

    1u8
}

unsafe fn MempMapRangeOfPages(mut VirtualAddress: u64, mut PhysicalAddress: u64, cPages: PFN_NUMBER) -> PFN_NUMBER {
    let mut i: PFN_NUMBER;

    i = 0;
    while i < cPages {
        if MempMapSinglePage(VirtualAddress, PhysicalAddress) == 0 {
            ERR!("Failed to map page %ld from %p to %p\n", (i) as usize, (VirtualAddress as *mut core::ffi::c_void) as usize, (PhysicalAddress as *mut core::ffi::c_void) as usize);
            return i;
        }
        VirtualAddress += PAGE_SIZE as u64;
        PhysicalAddress += PAGE_SIZE as u64;
        i += 1;
    }
    return i;
}

#[no_mangle]
pub unsafe extern "C" fn MempSetupPaging(StartPage: PFN_NUMBER, NumberOfPages: PFN_NUMBER, KernelMapping: u8) -> u8 {

    TRACE!(">>> MempSetupPaging(0x%lx, %ld, %p)\n", (StartPage) as usize, (NumberOfPages) as usize, ((StartPage * PAGE_SIZE + KSEG0_BASE) as *mut core::ffi::c_void) as usize);

    if MempMapRangeOfPages((StartPage * PAGE_SIZE) as u64,
                          (StartPage * PAGE_SIZE) as u64,
                          NumberOfPages) != NumberOfPages
    {
        ERR!("Failed to map pages %ld, %ld\n", (StartPage) as usize, (NumberOfPages) as usize);
        return 0u8;
    }

    if KernelMapping != 0 {
        if MempMapRangeOfPages((StartPage * PAGE_SIZE + KSEG0_BASE) as u64,
                              (StartPage * PAGE_SIZE) as u64,
                              NumberOfPages) != NumberOfPages
        {
            ERR!("Failed to map pages %ld, %ld\n", (StartPage) as usize, (NumberOfPages) as usize);
            return 0u8;
        }
    }

    return 1u8;
}

unsafe fn MempUnmapPage(_Page: PFN_NUMBER) {
}

unsafe fn WinLdrpMapApic() {

    let LocalAPIC: u8;
    let mut MsrValue: LARGE_INTEGER = core::mem::zeroed();
    let mut CpuInfo: [u32; 4] = [0; 4];
    let APICAddress: u64;

    TRACE!(">>> WinLdrpMapApic\n");

    __cpuid(CpuInfo.as_mut_ptr() as *mut i32, 1);
    LocalAPIC = if ((CpuInfo[3] >> 9) & 1) != 0 { 1 } else { 0 };

    if LocalAPIC == 0 {
        WARN!("No APIC found.\n");
        return;
    }

    MsrValue.QuadPart = __readmsr(0x1B) as LONGLONG;
    APICAddress = (MsrValue.u.LowPart & 0xFFFFF000) as u64;

    TRACE!("Local APIC detected at address 0x%x\n", (APICAddress) as usize);

    MempMapSinglePage(APIC_BASE, APICAddress);
}

unsafe fn WinLdrMapSpecialPages() -> u8 {

    let PpeBase: PHARDWARE_PTE;
    let PdeBase: PHARDWARE_PTE;

    if MempMapSinglePage(KI_USER_SHARED_DATA as u64, (SharedUserDataPfn * PAGE_SIZE) as u64) == 0 {
        ERR!("Could not map KI_USER_SHARED_DATA\n");
        return 0u8;
    }

    WinLdrpMapApic();

    PpeBase = MempGetOrCreatePageDir(PxeBase, VAtoPXI(MM_HAL_VA_START));
    PdeBase = MempGetOrCreatePageDir(PpeBase, VAtoPPI(MM_HAL_VA_START));
    MempGetOrCreatePageDir(PdeBase, VAtoPDI(MM_HAL_VA_START));
    MempGetOrCreatePageDir(PdeBase, VAtoPDI(MM_HAL_VA_START + 2 * 1024 * 1024));

    1u8
}

unsafe fn Amd64SetupGdt(GdtBase: ntkrnl_sys::base::PVOID, TssBase: u64) {
    let mut Entry: *mut _KGDTENTRY64;
    let mut GdtDesc: _KDESCRIPTOR = core::mem::zeroed();

    Entry = KiGetGdtEntry(GdtBase, KGDT64_NULL);
    *(Entry as PULONG64) = 0x0000000000000000u64;

    Entry = KiGetGdtEntry(GdtBase, KGDT64_R0_CODE);
    *(Entry as PULONG64) = 0x00209b0000000000u64;

    Entry = KiGetGdtEntry(GdtBase, KGDT64_R0_DATA);
    *(Entry as PULONG64) = 0x00cf93000000ffffu64;

    Entry = KiGetGdtEntry(GdtBase, KGDT64_R3_CMCODE);
    *(Entry as PULONG64) = 0x00cffb000000ffffu64;

    Entry = KiGetGdtEntry(GdtBase, KGDT64_R3_DATA);
    *(Entry as PULONG64) = 0x00cff3000000ffffu64;

    Entry = KiGetGdtEntry(GdtBase, KGDT64_R3_CODE);
    *(Entry as PULONG64) = 0x0020fb0000000000u64;

    Entry = KiGetGdtEntry(GdtBase, KGDT64_R3_CMTEB);
    *(Entry as PULONG64) = 0xff40f3fd50003c00u64;

    Entry = KiGetGdtEntry(GdtBase, KGDT64_SYS_TSS);
    KiInitGdtEntry(Entry, TssBase, core::mem::size_of::<KTSS>() as ULONG, I386_TSS, 0);

    GdtDesc.Base = GdtBase;
    GdtDesc.Limit = (NUM_GDT as usize * core::mem::size_of::<KGDTENTRY>() - 1) as u16;

    __lgdt(core::ptr::addr_of_mut!(GdtDesc.Limit) as ntkrnl_sys::base::PVOID);
}

unsafe fn Amd64SetupIdt(IdtBase: PVOID)
{
    let mut IdtDesc: KDESCRIPTOR = core::mem::zeroed();
    let mut OldIdt: KDESCRIPTOR = core::mem::zeroed();
    
    __sidt(core::ptr::addr_of_mut!(OldIdt.Limit) as PVOID);
    
    IdtDesc.Base = IdtBase;
    IdtDesc.Limit = (NUM_IDT as usize * core::mem::size_of::<KIDTENTRY>() - 1) as u16;
    
    __lidt(core::ptr::addr_of_mut!(IdtDesc.Limit) as PVOID);
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrSetProcessorContext(
    _OperatingSystemVersion: USHORT)
{

    TRACE!("WinLdrSetProcessorContext\n");

    _disable();

    __writeeflags(0);

    __writecr3(PxeBase as usize as u64);

    GdtIdt = (GdtIdt as usize + KSEG0_BASE) as PVOID;

    Amd64SetupGdt(GdtIdt, (KSEG0_BASE as u64) | ((TssBasePage as u64) << MM_PAGE_SHIFT));

    Amd64SetupIdt((GdtIdt as usize + NUM_GDT as usize * core::mem::size_of::<KGDTENTRY>()) as PVOID);

    __ltr(KGDT64_SYS_TSS);

    TRACE!("leave WinLdrSetProcessorContext\n");
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrSetupMachineDependent(LoaderBlock: PLOADER_PARAMETER_BLOCK) {

    let mut SharedUserDataAddress: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Tss: usize = 0;
    let mut BlockSize: u32;
    let NumPages: u32;

    (*LoaderBlock).u.I386.CommonDataArea = DbgPrint as *mut core::ffi::c_void;
    (*LoaderBlock).u.I386.MachineType = MACHINE_TYPE_ISA;

    SharedUserDataAddress = MmAllocateMemoryWithType(MM_PAGE_SIZE as usize, LoaderStartupPcrPage);
    SharedUserDataPfn = (SharedUserDataAddress as usize) >> MM_PAGE_SHIFT;
    if SharedUserDataAddress.is_null() {
        UiMessageBox(b"Can't allocate SharedUserData page.\0".as_ptr());
        return;
    }
    RtlZeroMemory(SharedUserDataAddress, MM_PAGE_SIZE as usize);

    BlockSize = ((core::mem::size_of::<KTSS>() + MM_PAGE_SIZE as usize) & !(MM_PAGE_SIZE as usize - 1)) as u32;
    Tss = MmAllocateMemoryWithType(BlockSize as usize, LoaderMemoryData) as usize;
    TssBasePage = Tss >> MM_PAGE_SHIFT;

    BlockSize = NUM_GDT * core::mem::size_of::<KGDTENTRY>() as u32 + NUM_IDT * core::mem::size_of::<KIDTENTRY>() as u32;
    NumPages = (BlockSize + MM_PAGE_SIZE as u32 - 1) >> MM_PAGE_SHIFT;
    GdtIdt = MmAllocateMemoryWithType(NumPages as usize * MM_PAGE_SIZE as usize, LoaderMemoryData) as PVOID;
    if GdtIdt.is_null() {
        UiMessageBox(b"Can't allocate pages for GDT+IDT!\0".as_ptr());
        return;
    }

    RtlZeroMemory(GdtIdt as *mut core::ffi::c_void, (NumPages << MM_PAGE_SHIFT) as usize);

    if MempAllocatePageTables() == 0u8 {
    }

    WinLdrMapSpecialPages();
}

unsafe fn MempDump() {
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         EFI Windows Loader
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            boot/freeldr/freeldr/arch/amd64/winldr.c
//  * PURPOSE:         Memory related routines
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES ***************************************************************/
//
// #include <freeldr.h>
// #include <ndk/asm.h>
// #include <internal/amd64/intrin_i.h>
// #include "../../winldr.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WINDOWS);
//
// //extern ULONG LoaderPagesSpanned;
//
// /* GLOBALS ***************************************************************/
//
// PHARDWARE_PTE PxeBase;
// //PHARDWARE_PTE HalPageTable;
//
// PVOID GdtIdt;
// PFN_NUMBER SharedUserDataPfn;
// ULONG_PTR TssBasePage;
//
// /* FUNCTIONS **************************************************************/
//
// static
// BOOLEAN
// MempAllocatePageTables(VOID)
// {
//     TRACE(">>> MempAllocatePageTables\n");
//
//     /* Allocate a page for the PML4 */
//     PxeBase = MmAllocateMemoryWithType(PAGE_SIZE, LoaderMemoryData);
//     if (!PxeBase)
//     {
//         ERR("failed to allocate PML4\n");
//         return FALSE;
//     }
//
//     // FIXME: Physical PTEs = FirmwareTemporary ?
//
//     /* Zero the PML4 */
//     RtlZeroMemory(PxeBase, PAGE_SIZE);
//
//     /* The page tables are located at 0xfffff68000000000
//      * We create a recursive self mapping through all 4 levels at
//      * virtual address 0xfffff6fb7dbedf68 */
//     PxeBase[VAtoPXI(PXE_BASE)].Valid = 1;
//     PxeBase[VAtoPXI(PXE_BASE)].Write = 1;
//     PxeBase[VAtoPXI(PXE_BASE)].PageFrameNumber = PtrToPfn(PxeBase);
//
//     // FIXME: map PDE's for hals memory mapping
//
//     TRACE(">>> leave MempAllocatePageTables\n");
//
//     return TRUE;
// }
//
// static
// PHARDWARE_PTE
// MempGetOrCreatePageDir(PHARDWARE_PTE PdeBase, ULONG Index)
// {
//     PHARDWARE_PTE SubDir;
//
//     if (!PdeBase)
//         return NULL;
//
//     if (!PdeBase[Index].Valid)
//     {
//         SubDir = MmAllocateMemoryWithType(PAGE_SIZE, LoaderMemoryData);
//         if (!SubDir)
//             return NULL;
//         RtlZeroMemory(SubDir, PAGE_SIZE);
//         PdeBase[Index].PageFrameNumber = PtrToPfn(SubDir);
//         PdeBase[Index].Valid = 1;
//         PdeBase[Index].Write = 1;
//     }
//     else
//     {
//         SubDir = (PVOID)((ULONG64)(PdeBase[Index].PageFrameNumber) * PAGE_SIZE);
//     }
//     return SubDir;
// }
//
// static
// BOOLEAN
// MempMapSinglePage(ULONG64 VirtualAddress, ULONG64 PhysicalAddress)
// {
//     PHARDWARE_PTE PpeBase, PdeBase, PteBase;
//     ULONG Index;
//
//     PpeBase = MempGetOrCreatePageDir(PxeBase, VAtoPXI(VirtualAddress));
//     PdeBase = MempGetOrCreatePageDir(PpeBase, VAtoPPI(VirtualAddress));
//     PteBase = MempGetOrCreatePageDir(PdeBase, VAtoPDI(VirtualAddress));
//
//     if (!PteBase)
//     {
//         ERR("!!!No Dir %p, %p, %p, %p\n", (PxeBase) as usize, (PpeBase) as usize, (PdeBase) as usize, (PteBase) as usize);
//         return FALSE;
//     }
//
//     Index = VAtoPTI(VirtualAddress);
//     if (PteBase[Index].Valid)
//     {
//         ERR("!!!Already mapped %ld\n", (Index) as usize);
//         return FALSE;
//     }
//
//     PteBase[Index].Valid = 1;
//     PteBase[Index].Write = 1;
//     PteBase[Index].PageFrameNumber = PhysicalAddress / PAGE_SIZE;
//
//     return TRUE;
// }
//
// BOOLEAN
// MempIsPageMapped(PVOID VirtualAddress)
// {
//     PHARDWARE_PTE PpeBase, PdeBase, PteBase;
//     ULONG Index;
//
//     Index = VAtoPXI(VirtualAddress);
//     if (!PxeBase[Index].Valid)
//         return FALSE;
//
//     PpeBase = (PVOID)((ULONG64)(PxeBase[Index].PageFrameNumber) * PAGE_SIZE);
//     Index = VAtoPPI(VirtualAddress);
//     if (!PpeBase[Index].Valid)
//         return FALSE;
//
//     PdeBase = (PVOID)((ULONG64)(PpeBase[Index].PageFrameNumber) * PAGE_SIZE);
//     Index = VAtoPDI(VirtualAddress);
//     if (!PdeBase[Index].Valid)
//         return FALSE;
//
//     PteBase = (PVOID)((ULONG64)(PdeBase[Index].PageFrameNumber) * PAGE_SIZE);
//     Index = VAtoPTI(VirtualAddress);
//     if (!PteBase[Index].Valid)
//         return FALSE;
//
//     return TRUE;
// }
//
// static
// PFN_NUMBER
// MempMapRangeOfPages(ULONG64 VirtualAddress, ULONG64 PhysicalAddress, PFN_NUMBER cPages)
// {
//     PFN_NUMBER i;
//
//     for (i = 0; i < cPages; i++)
//     {
//         if (!MempMapSinglePage(VirtualAddress, PhysicalAddress))
//         {
//             ERR("Failed to map page %ld from %p to %p\n", (//                     i) as usize, ((PVOID)VirtualAddress) as usize, ((PVOID)PhysicalAddress) as usize);
//             return i;
//         }
//         VirtualAddress += PAGE_SIZE;
//         PhysicalAddress += PAGE_SIZE;
//     }
//     return i;
// }
//
// BOOLEAN
// MempSetupPaging(IN PFN_NUMBER StartPage,
//                 IN PFN_NUMBER NumberOfPages,
//                 IN BOOLEAN KernelMapping)
// {
//     TRACE(">>> MempSetupPaging(0x%lx, %ld, %p)\n", (//             StartPage) as usize, (NumberOfPages) as usize, (StartPage * PAGE_SIZE + KSEG0_BASE) as usize);
//
//     /* Identity mapping */
//     if (MempMapRangeOfPages(StartPage * PAGE_SIZE,
//                             StartPage * PAGE_SIZE,
//                             NumberOfPages) != NumberOfPages)
//     {
//         ERR("Failed to map pages %ld, %ld\n", (//                 StartPage) as usize, (NumberOfPages) as usize);
//         return FALSE;
//     }
//
//     /* Kernel mapping */
//     if (KernelMapping)
//     {
//         if (MempMapRangeOfPages(StartPage * PAGE_SIZE + KSEG0_BASE,
//                                 StartPage * PAGE_SIZE,
//                                 NumberOfPages) != NumberOfPages)
//         {
//             ERR("Failed to map pages %ld, %ld\n", (//                     StartPage) as usize, (NumberOfPages) as usize);
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// VOID
// MempUnmapPage(PFN_NUMBER Page)
// {
//    // TRACE(">>> MempUnmapPage\n");
// }
//
// static
// VOID
// WinLdrpMapApic(VOID)
// {
//     BOOLEAN LocalAPIC;
//     LARGE_INTEGER MsrValue;
//     ULONG CpuInfo[4];
//     ULONG64 APICAddress;
//
//     TRACE(">>> WinLdrpMapApic\n");
//
//     /* Check if we have a local APIC */
//     __cpuid((int*)CpuInfo, 1);
//     LocalAPIC = (((CpuInfo[3] >> 9) & 1) != 0);
//
//     /* If there is no APIC, just return */
//     if (!LocalAPIC)
//     {
//         WARN("No APIC found.\n");
//         return;
//     }
//
//     /* Read the APIC Address */
//     MsrValue.QuadPart = __readmsr(0x1B);
//     APICAddress = (MsrValue.LowPart & 0xFFFFF000);
//
//     TRACE("Local APIC detected at address 0x%x\n", (//         APICAddress) as usize);
//
//     /* Map it */
//     MempMapSinglePage(APIC_BASE, APICAddress);
// }
//
// static
// BOOLEAN
// WinLdrMapSpecialPages(VOID)
// {
//     PHARDWARE_PTE PpeBase, PdeBase;
//
//     /* Map KI_USER_SHARED_DATA */
//     if (!MempMapSinglePage(KI_USER_SHARED_DATA, SharedUserDataPfn * PAGE_SIZE))
//     {
//         ERR("Could not map KI_USER_SHARED_DATA\n");
//         return FALSE;
//     }
//
//     /* Map the APIC page */
//     WinLdrpMapApic();
//
//     /* Map the page tables for 4 MB HAL address space. */
//     PpeBase = MempGetOrCreatePageDir(PxeBase, VAtoPXI(MM_HAL_VA_START));
//     PdeBase = MempGetOrCreatePageDir(PpeBase, VAtoPPI(MM_HAL_VA_START));
//     MempGetOrCreatePageDir(PdeBase, VAtoPDI(MM_HAL_VA_START));
//     MempGetOrCreatePageDir(PdeBase, VAtoPDI(MM_HAL_VA_START + 2 * 1024 * 1024));
//
//     return TRUE;
// }
//
// static
// VOID
// Amd64SetupGdt(PVOID GdtBase, ULONG64 TssBase)
// {
//     PKGDTENTRY64 Entry;
//     KDESCRIPTOR GdtDesc;
//     TRACE("Amd64SetupGdt(GdtBase = %p, TssBase = %p)\n", (GdtBase) as usize, (TssBase) as usize);
//
//     /* Setup KGDT64_NULL */
//     Entry = KiGetGdtEntry(GdtBase, KGDT64_NULL);
//     *(PULONG64)Entry = 0x0000000000000000ULL;
//
//     /* Setup KGDT64_R0_CODE */
//     Entry = KiGetGdtEntry(GdtBase, KGDT64_R0_CODE);
//     *(PULONG64)Entry = 0x00209b0000000000ULL;
//
//     /* Setup KGDT64_R0_DATA */
//     Entry = KiGetGdtEntry(GdtBase, KGDT64_R0_DATA);
//     *(PULONG64)Entry = 0x00cf93000000ffffULL;
//
//     /* Setup KGDT64_R3_CMCODE */
//     Entry = KiGetGdtEntry(GdtBase, KGDT64_R3_CMCODE);
//     *(PULONG64)Entry = 0x00cffb000000ffffULL;
//
//     /* Setup KGDT64_R3_DATA */
//     Entry = KiGetGdtEntry(GdtBase, KGDT64_R3_DATA);
//     *(PULONG64)Entry = 0x00cff3000000ffffULL;
//
//     /* Setup KGDT64_R3_CODE */
//     Entry = KiGetGdtEntry(GdtBase, KGDT64_R3_CODE);
//     *(PULONG64)Entry = 0x0020fb0000000000ULL;
//
//     /* Setup KGDT64_R3_CMTEB */
//     Entry = KiGetGdtEntry(GdtBase, KGDT64_R3_CMTEB);
//     *(PULONG64)Entry = 0xff40f3fd50003c00ULL;
//
//     /* Setup TSS entry */
//     Entry = KiGetGdtEntry(GdtBase, KGDT64_SYS_TSS);
//     KiInitGdtEntry(Entry, TssBase, sizeof(KTSS), I386_TSS, 0);
//
//     /* Setup GDT descriptor */
//     GdtDesc.Base  = GdtBase;
//     GdtDesc.Limit = NUM_GDT * sizeof(KGDTENTRY) - 1;
//
//     /* Set the new Gdt */
//     __lgdt(&GdtDesc.Limit);
//     TRACE("Leave Amd64SetupGdt()\n");
// }
//
// static
// VOID
// Amd64SetupIdt(PVOID IdtBase)
// {
//     KDESCRIPTOR IdtDesc, OldIdt;
//     //ULONG Size;
//     TRACE("Amd64SetupIdt(IdtBase = %p)\n", (IdtBase) as usize);
//
//     /* Get old IDT */
//     __sidt(&OldIdt.Limit);
//
//     /* Copy the old IDT */
//     //Size =  min(OldIdt.Limit + 1, NUM_IDT * sizeof(KIDTENTRY));
//     //RtlCopyMemory(IdtBase, (PVOID)OldIdt.Base, Size);
//
//     /* Setup the new IDT descriptor */
//     IdtDesc.Base = IdtBase;
//     IdtDesc.Limit = NUM_IDT * sizeof(KIDTENTRY) - 1;
//
//     /* Set the new IDT */
//     __lidt(&IdtDesc.Limit);
//     TRACE("Leave Amd64SetupIdt()\n");
// }
//
// VOID
// WinLdrSetProcessorContext(
//     _In_ USHORT OperatingSystemVersion)
// {
//     TRACE("WinLdrSetProcessorContext\n");
//
//     /* Disable Interrupts */
//     _disable();
//
//     /* Re-initialize EFLAGS */
//     __writeeflags(0);
//
//     /* Set the new PML4 */
//     __writecr3((ULONG64)PxeBase);
//
//     /* Get kernel mode address of gdt / idt */
//     GdtIdt = (PVOID)((ULONG64)GdtIdt + KSEG0_BASE);
//
//     /* Create gdt entries and load gdtr */
//     Amd64SetupGdt(GdtIdt, KSEG0_BASE | (TssBasePage << MM_PAGE_SHIFT));
//
//     /* Copy old Idt and set idtr */
//     Amd64SetupIdt((PVOID)((ULONG64)GdtIdt + NUM_GDT * sizeof(KGDTENTRY)));
//
//     /* LDT is unused */
// //    __lldt(0);
//
//     /* Load TSR */
//     __ltr(KGDT64_SYS_TSS);
//
//     TRACE("leave WinLdrSetProcessorContext\n");
// }
//
// void WinLdrSetupMachineDependent(PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PVOID SharedUserDataAddress = NULL;
//     ULONG_PTR Tss = 0;
//     ULONG BlockSize, NumPages;
//
//     LoaderBlock->u.I386.CommonDataArea = (PVOID)DbgPrint; // HACK
//     LoaderBlock->u.I386.MachineType = MACHINE_TYPE_ISA;
//
//     /* Allocate 1 page for SharedUserData */
//     SharedUserDataAddress = MmAllocateMemoryWithType(MM_PAGE_SIZE, LoaderStartupPcrPage);
//     SharedUserDataPfn = (ULONG_PTR)SharedUserDataAddress >> MM_PAGE_SHIFT;
//     if (SharedUserDataAddress == NULL)
//     {
//         UiMessageBox("Can't allocate SharedUserData page.");
//         return;
//     }
//     RtlZeroMemory(SharedUserDataAddress, MM_PAGE_SIZE);
//
//     /* Allocate TSS */
//     BlockSize = (sizeof(KTSS) + MM_PAGE_SIZE) & ~(MM_PAGE_SIZE - 1);
//     Tss = (ULONG_PTR)MmAllocateMemoryWithType(BlockSize, LoaderMemoryData);
//     TssBasePage = Tss >> MM_PAGE_SHIFT;
//
//     /* Allocate space for new GDT + IDT */
//     BlockSize = NUM_GDT * sizeof(KGDTENTRY) + NUM_IDT * sizeof(KIDTENTRY);
//     NumPages = (BlockSize + MM_PAGE_SIZE - 1) >> MM_PAGE_SHIFT;
//     GdtIdt = (PKGDTENTRY)MmAllocateMemoryWithType(NumPages * MM_PAGE_SIZE, LoaderMemoryData);
//     if (GdtIdt == NULL)
//     {
//         UiMessageBox("Can't allocate pages for GDT+IDT!");
//         return;
//     }
//
//     /* Zero newly prepared GDT+IDT */
//     RtlZeroMemory(GdtIdt, NumPages << MM_PAGE_SHIFT);
//
//     // Before we start mapping pages, create a block of memory, which will contain
//     // PDE and PTEs
//     if (MempAllocatePageTables() == FALSE)
//     {
//         // FIXME: bugcheck
//     }
//
//     /* Map KI_USER_SHARED_DATA, Apic and HAL space */
//     WinLdrMapSpecialPages();
// }
//
//
// VOID
// MempDump(VOID)
// {
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
