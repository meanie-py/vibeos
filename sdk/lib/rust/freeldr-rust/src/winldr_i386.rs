#[allow(unused_imports)] use crate::{TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WINDOWS; // winldr.c: DBG_DEFAULT_CHANNEL(WINDOWS)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{LARGE_INTEGER, PLOADER_PARAMETER_BLOCK};
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderFirmwareTemporary as LoaderFirmwareTemporary;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderMemoryData as LoaderMemoryData;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderStartupPcrPage as LoaderStartupPcrPage;

// ndk i386 mmtypes.h: #define PHARDWARE_PTE PHARDWARE_PTE_X86
pub type PHARDWARE_PTE = ntkrnl_sys::ddk::PHARDWARE_PTE_X86;
pub use ntkrnl_sys::ddk::{KGDTENTRY, KIDTENTRY, KTSS, PKGDTENTRY};

// arch/i386/winldr.c #defines
pub const KIP0PCRADDRESS: usize = 0xffdff000;
pub const SELFMAP_ENTRY: usize = 0x300;
// ndk i386 ketypes.h: APIC_BASE; ntldr/winldr.h: NUM_GDT / NUM_IDT
pub const APIC_BASE: usize = 0xFFFE0000;
pub const NUM_GDT: u32 = 128;
pub const NUM_IDT: u32 = 0x100;

// MSVC __readmsr intrinsic — GNU target: emit rdmsr (i386).
#[inline]
unsafe fn __readmsr(msr: u32) -> u64 {
    let (lo, hi): (u32, u32);
    core::arch::asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi,
                     options(nomem, nostack, preserves_flags));
    ((hi as u64) << 32) | (lo as u64)
}
// ExtendedBIOSDataArea/Size/RomFontPointers are fixed low-memory addrs
// (#define ExtendedBIOSDataArea ((PULONG)0x740) etc.)
const ExtendedBIOSDataArea: *mut u32 = 0x740 as *mut u32;
const ExtendedBIOSDataSize: *mut u32 = 0x744 as *mut u32;
const RomFontPointers: *mut u32 = 0x700 as *mut u32;

// arch/i386/winldr.c file-scope globals (TU-internal — no other freeldr
// TU references them; private static mut).
static mut PDE: PHARDWARE_PTE = core::ptr::null_mut();
static mut HalPageTable: PHARDWARE_PTE = core::ptr::null_mut();
static mut PhysicalPageTablesBuffer: *mut u8 = core::ptr::null_mut();
static mut KernelPageTablesBuffer: *mut u8 = core::ptr::null_mut();
static mut PhysicalPageTables: ULONG = 0;
static mut KernelPageTables: ULONG = 0;
static mut PcrBasePage: ULONG = 0;
static mut TssBasePage: ULONG = 0;
static mut GdtIdt: PVOID = core::ptr::null_mut();

extern "C" {
    fn __cpuid(info: *mut i32, function: i32);
    fn MachVideoGetFontsFromFirmware(fonts: *mut u32);
    fn MachGetExtendedBIOSData(area: *mut u32, size: *mut u32);
    fn MmAllocateMemoryWithType(size: usize, memory_type: u32) -> *mut core::ffi::c_void;
    fn MmSetMemoryType(BaseAddress: *mut core::ffi::c_void, Size: SIZE_T, MemoryType: u32);
    fn MmGetTotalPagesInLookupTable() -> PFN_NUMBER;
    fn UiMessageBox(message: *const i8);
    fn BugCheck(message: *const i8);
}

unsafe fn DumpGDTEntry(Base: ULONG_PTR, Selector: ULONG) {
    let pGdt = (Base + Selector as ULONG_PTR) as PKGDTENTRY;

    TRACE!("\nSelector 0x%04x\n===============\nLimitLow               = 0x%04x\nBaseLow                = 0x%04x\nHighWord.Bytes.BaseMid = 0x%02x\nHighWord.Bytes.Flags1  = 0x%02x\nHighWord.Bytes.Flags2  = 0x%02x\nHighWord.Bytes.BaseHi  = 0x%02x\n\n", (Selector) as usize, ((*pGdt).LimitLow) as usize, ((*pGdt).BaseLow) as usize, ((*pGdt).HighWord.Bytes.BaseMid) as usize, ((*pGdt).HighWord.Bytes.Flags1) as usize, ((*pGdt).HighWord.Bytes.Flags2) as usize, ((*pGdt).HighWord.Bytes.BaseHi) as usize);
}

unsafe fn MempAllocatePageTables() -> u8 {
    let mut NumPageTables: u32;
    let mut TotalSize: u32;
    let mut Buffer: *mut u8;

    NumPageTables = ((MmGetTotalPagesInLookupTable() >> 10) as u32);

    TRACE!("NumPageTables = %d\n", (NumPageTables) as usize);

    TotalSize = (1 + 1 + NumPageTables * 2) * ((MM_PAGE_SIZE) as u32);

    Buffer = MmAllocateMemoryWithType(TotalSize as usize, LoaderMemoryData) as *mut u8;

    PhysicalPageTablesBuffer = Buffer.add((TotalSize - NumPageTables * ((MM_PAGE_SIZE) as u32)) as usize);
    MmSetMemoryType(
        PhysicalPageTablesBuffer as *mut core::ffi::c_void,
        (NumPageTables * ((MM_PAGE_SIZE) as u32)) as SIZE_T,
        LoaderFirmwareTemporary,
    );

    if Buffer.add((TotalSize - NumPageTables * ((MM_PAGE_SIZE) as u32)) as usize) != PhysicalPageTablesBuffer {
        TRACE!("There was a problem allocating two adjacent blocks of memory!\n");
    }

    if Buffer.is_null() || PhysicalPageTablesBuffer.is_null() {
        UiMessageBox(b"Impossible to allocate memory block for page tables!\0".as_ptr() as *const i8);
        return 0u8;
    }

    RtlZeroMemory(Buffer as *mut core::ffi::c_void, TotalSize as usize);

    PDE = Buffer as PHARDWARE_PTE;

    (*PDE.add(SELFMAP_ENTRY as usize)).set_PageFrameNumber((PDE as u32) >> MM_PAGE_SHIFT as ULONG);
    (*PDE.add(SELFMAP_ENTRY as usize)).set_Valid(1);
    (*PDE.add(SELFMAP_ENTRY as usize)).set_Write(1);

    HalPageTable = Buffer.add((MM_PAGE_SIZE * 1) as usize) as PHARDWARE_PTE;

    (*PDE.add(1023)).set_PageFrameNumber((HalPageTable as u32) >> MM_PAGE_SHIFT as ULONG);
    (*PDE.add(1023)).set_Valid(1);
    (*PDE.add(1023)).set_Write(1);

    KernelPageTablesBuffer = Buffer.add((MM_PAGE_SIZE * 2) as usize);

    PhysicalPageTables = 0;
    KernelPageTables = 0;

    1u8
}

unsafe fn MempAllocatePTE(Entry: u32, PhysicalPT: *mut PHARDWARE_PTE, KernelPT: *mut PHARDWARE_PTE) {
    *PhysicalPT = PhysicalPageTablesBuffer.add((PhysicalPageTables * MM_PAGE_SIZE as ULONG) as usize) as PHARDWARE_PTE;
    PhysicalPageTables = PhysicalPageTables.wrapping_add(1);

    (*PDE.add(Entry as usize)).set_PageFrameNumber((*PhysicalPT as u32) >> MM_PAGE_SHIFT as ULONG);
    (*PDE.add(Entry as usize)).set_Valid(1 as ULONG);
    (*PDE.add(Entry as usize)).set_Write(1 as ULONG);

    if Entry.wrapping_add((KSEG0_BASE >> 22) as u32) > 1023 {
    }

    *KernelPT = KernelPageTablesBuffer.add((KernelPageTables * MM_PAGE_SIZE as ULONG) as usize) as PHARDWARE_PTE;
    KernelPageTables = KernelPageTables.wrapping_add(1);

    (*PDE.add((Entry.wrapping_add((KSEG0_BASE >> 22) as u32)) as usize)).set_PageFrameNumber((*KernelPT as u32) >> MM_PAGE_SHIFT as ULONG);
    (*PDE.add((Entry.wrapping_add((KSEG0_BASE >> 22) as u32)) as usize)).set_Valid(1 as ULONG);
    (*PDE.add((Entry.wrapping_add((KSEG0_BASE >> 22) as u32)) as usize)).set_Write(1 as ULONG);
}

#[no_mangle]
pub unsafe extern "C" fn MempSetupPaging(
    StartPage: PFN_NUMBER,
    NumberOfPages: PFN_COUNT,
    KernelMapping: u8,
) -> u8 {
    let mut PhysicalPT: PHARDWARE_PTE = core::ptr::null_mut();
    let mut KernelPT: PHARDWARE_PTE = core::ptr::null_mut();
    let mut Entry: PFN_COUNT;
    let mut Page: PFN_COUNT;

    TRACE!("MempSetupPaging: SP 0x%X, Number: 0x%X, Kernel: %s\n", (StartPage) as usize, (NumberOfPages) as usize, (if KernelMapping != 0 {
            b"yes\0".as_ptr()
        } else {
            b"no\0".as_ptr()
        }) as usize);

    if StartPage.wrapping_add(((NumberOfPages) as usize)) >= 0x80000 {
        return 1u8;
    }

    Page = ((StartPage) as u32);
    while Page < ((StartPage.wrapping_add(((NumberOfPages) as usize))) as u32) {
        Entry = Page >> 10;

        if (*(PDE as *mut u32).add(Entry as usize)) == 0 {
            MempAllocatePTE(Entry as u32, &mut PhysicalPT, &mut KernelPT);
        } else {
            PhysicalPT = ((*PDE.add(Entry as usize)).PageFrameNumber() << MM_PAGE_SHIFT) as PHARDWARE_PTE;
            KernelPT = ((*PDE.add(Entry.wrapping_add((KSEG0_BASE >> 22) as u32) as usize)).PageFrameNumber() << MM_PAGE_SHIFT) as PHARDWARE_PTE;
        }

        (*PhysicalPT.add((Page & 0x3ff) as usize)).set_PageFrameNumber(Page as ULONG);
        (*PhysicalPT.add((Page & 0x3ff) as usize)).set_Valid(1 as ULONG);
        (*PhysicalPT.add((Page & 0x3ff) as usize)).set_Write(if Page != 0 { 1 } else { 0 } as ULONG);

        if KernelMapping != 0 {
            if (*KernelPT.add((Page & 0x3ff) as usize)).Valid() != 0 {
                WARN!("KernelPT already mapped\n");
            }
            (*KernelPT.add((Page & 0x3ff) as usize)).set_PageFrameNumber(Page as ULONG);
            (*KernelPT.add((Page & 0x3ff) as usize)).set_Valid(if Page != 0 { 1 } else { 0 } as ULONG);
            (*KernelPT.add((Page & 0x3ff) as usize)).set_Write(if Page != 0 { 1 } else { 0 } as ULONG);
        }

        Page = Page.wrapping_add(1);
    }

    1u8
}

unsafe fn MempUnmapPage(Page: PFN_NUMBER) {
    let mut KernelPT: PHARDWARE_PTE;
    let Entry: PFN_NUMBER = ((Page >> 10) as usize) + (KSEG0_BASE >> 22);

    if Entry == SELFMAP_ENTRY || Entry == 1023 {
        return;
    }

    if (*PDE.offset(Entry as isize)).Valid() != 0 {
        KernelPT = ((*PDE.offset(Entry as isize)).PageFrameNumber() << MM_PAGE_SHIFT) as PHARDWARE_PTE;

        if !KernelPT.is_null() {
            (*KernelPT.offset((Page & 0x3ff) as isize)).set_PageFrameNumber(0);
            (*KernelPT.offset((Page & 0x3ff) as isize)).set_Valid(0);
            (*KernelPT.offset((Page & 0x3ff) as isize)).set_Write(0);
        }
    }
}

unsafe fn WinLdrpMapApic() {
    let mut LocalAPIC: BOOLEAN;
    let mut MsrValue: LARGE_INTEGER = core::mem::zeroed();
    let mut APICAddress: ULONG;
    let mut CpuInfo: [ULONG; 4] = [0; 4];

    __cpuid(CpuInfo.as_mut_ptr() as *mut i32, 1);
    LocalAPIC = if ((CpuInfo[3] >> 9) & 1) != 0 { 1 } else { 0 };

    if LocalAPIC == 0 {
        return;
    }

    MsrValue.QuadPart = __readmsr(0x1B) as i64;
    APICAddress = MsrValue.u.LowPart & 0xFFFFF000;

    TRACE!("Local APIC detected at address 0x%x\n", (APICAddress) as usize);

    let index = ((APIC_BASE - 0xFFC00000) >> MM_PAGE_SHIFT as usize) as usize;
    (*HalPageTable.add(index)).set_PageFrameNumber((APICAddress >> MM_PAGE_SHIFT) as ULONG);
    (*HalPageTable.add(index)).set_Valid(1);
    (*HalPageTable.add(index)).set_Write(1);
    (*HalPageTable.add(index)).set_WriteThrough(1);
    (*HalPageTable.add(index)).set_CacheDisable(1);
}

unsafe fn WinLdrMapSpecialPages() -> BOOLEAN {

    TRACE!("HalPageTable: 0x%X\n", (HalPageTable) as usize);

    (*HalPageTable.offset(((KI_USER_SHARED_DATA as ULONG).wrapping_sub(0xFFC00000) >> MM_PAGE_SHIFT) as isize)).set_PageFrameNumber(PcrBasePage.wrapping_add(1) as ULONG);
    (*HalPageTable.offset(((KI_USER_SHARED_DATA as ULONG).wrapping_sub(0xFFC00000) >> MM_PAGE_SHIFT) as isize)).set_Valid(1 as ULONG);
    (*HalPageTable.offset(((KI_USER_SHARED_DATA as ULONG).wrapping_sub(0xFFC00000) >> MM_PAGE_SHIFT) as isize)).set_Write(1 as ULONG);

    (*HalPageTable.offset(((KIP0PCRADDRESS as ULONG).wrapping_sub(0xFFC00000) >> MM_PAGE_SHIFT) as isize)).set_PageFrameNumber(PcrBasePage as ULONG);
    (*HalPageTable.offset(((KIP0PCRADDRESS as ULONG).wrapping_sub(0xFFC00000) >> MM_PAGE_SHIFT) as isize)).set_Valid(1 as ULONG);
    (*HalPageTable.offset(((KIP0PCRADDRESS as ULONG).wrapping_sub(0xFFC00000) >> MM_PAGE_SHIFT) as isize)).set_Write(1 as ULONG);

    WinLdrpMapApic();

    1u8
}

unsafe fn WinLdrSetupSpecialDataPointers() {

    MachVideoGetFontsFromFirmware(RomFontPointers);

    MachGetExtendedBIOSData(ExtendedBIOSDataArea, ExtendedBIOSDataSize);

    if *ExtendedBIOSDataArea == 0 && *ExtendedBIOSDataSize == 0 {
        WARN!("Couldn't get address of extended BIOS data area\n");
    } else {
        TRACE!("*ExtendedBIOSDataArea = 0x%lx\n", (*ExtendedBIOSDataArea) as usize);
    }
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrSetupMachineDependent(LoaderBlock: PLOADER_PARAMETER_BLOCK) {

    let TssSize: u32;
    let mut Pcr: usize = 0;
    let mut Tss: usize = 0;
    let BlockSize: u32;
    let NumPages: u32;

    (*LoaderBlock).u.I386.CommonDataArea = core::ptr::null_mut();
    (*LoaderBlock).u.I386.MachineType = 0;

    Pcr = MmAllocateMemoryWithType((2 * MM_PAGE_SIZE) as usize, LoaderStartupPcrPage) as usize;
    PcrBasePage = ((Pcr >> MM_PAGE_SHIFT) as u32);
    if Pcr == 0 {
        UiMessageBox(b"Could not allocate PCR.\0".as_ptr() as *const i8);
        return;
    }

    TssSize = (core::mem::size_of::<KTSS>() as u32 + ((MM_PAGE_SIZE) as u32)) & ((!(MM_PAGE_SIZE - 1)) as u32);

    Tss = MmAllocateMemoryWithType(TssSize as usize, LoaderMemoryData) as usize;
    TssBasePage = ((Tss >> MM_PAGE_SHIFT) as u32);
    if Tss == 0 {
        UiMessageBox(b"Could not allocate TSS.\0".as_ptr() as *const i8);
        return;
    }

    BlockSize = NUM_GDT * core::mem::size_of::<KGDTENTRY>() as u32 + NUM_IDT * core::mem::size_of::<KIDTENTRY>() as u32;
    NumPages = (BlockSize + ((MM_PAGE_SIZE) as u32) - 1) >> MM_PAGE_SHIFT;
    GdtIdt = MmAllocateMemoryWithType((NumPages * ((MM_PAGE_SIZE) as u32)) as usize, LoaderMemoryData) as PVOID;
    if GdtIdt.is_null() {
        UiMessageBox(b"Could not allocate pages for GDT+IDT!\0".as_ptr() as *const i8);
        return;
    }

    RtlZeroMemory(GdtIdt as *mut core::ffi::c_void, ((NumPages << MM_PAGE_SHIFT) as usize));

    if MempAllocatePageTables() == 0 {
        BugCheck(b"MempAllocatePageTables failed!\n\0".as_ptr() as *const i8);
    }

    WinLdrMapSpecialPages();

    WinLdrSetupSpecialDataPointers();
}

pub(crate) unsafe fn MempDump() {
    let PDE_Addr: *mut u32 = PDE as *mut u32;
    let mut i: i32;
    let mut j: i32;

    TRACE!("\nPDE\n");

    i = 0;
    while i < 128 {
        TRACE!("0x%04X | ", (i * 8) as usize);

        j = 0;
        while j < 8 {
            TRACE!("0x%08X ", (*PDE_Addr.offset((i * 8 + j) as isize)) as usize);
            j += 1;
        }

        TRACE!("\n");
        i += 1;
    }
}

// ============================================================================
// WinLdrSetProcessorContext + GDT/IDT helpers
//
// rustconv left WinLdrSetProcessorContext untranslated (raw C in the
// commented ORIGINAL C SOURCE block below). uefildr.efi pulls the winldr
// OS-handoff path (winldr.rs -> WinLdrSetProcessorContext) so it needs a
// real symbol. Faithful translation of arch/i386/winldr.c's
// WinLdrSetProcessorContext + the FORCEINLINE helpers it uses from
// ntoskrnl/include/internal/i386/intrin_i.h (KiGetGdtEntry / KiSetGdtEntry
// / KiSetGdtEntryEx / __lgdt / __lidt / __sidt / Ke386Set* segment macros)
// plus the GDTIDT struct and DPL_*/TYPE_* descriptor constants from
// winldr.c's file scope.
// ============================================================================

// arch/i386/winldr.c file-scope #defines
const DPL_SYSTEM: u8 = 0;
const DPL_USER: u8 = 3;
const TYPE_TSS32A: u8 = 0x09;
const DESCRIPTOR_ACCESSED: u8 = 0x1;
const DESCRIPTOR_READ_WRITE: u8 = 0x2;
const DESCRIPTOR_EXECUTE_READ: u8 = 0x2;
const DESCRIPTOR_CODE: u8 = 0x8;
const TYPE_CODE: u8 = 0x10 | DESCRIPTOR_CODE | DESCRIPTOR_EXECUTE_READ;
const TYPE_DATA: u8 = 0x10 | DESCRIPTOR_READ_WRITE;

// ndk/i386/ketypes.h
const RPL_MASK: u16 = 0x0003;
const KGDT_NULL: u16 = 0x00;
const KGDT_R0_CODE: u16 = 0x08;
const KGDT_R0_DATA: u16 = 0x10;
const KGDT_R3_CODE: u16 = 0x18;
const KGDT_R3_DATA: u16 = 0x20;
const KGDT_TSS: u16 = 0x28;
const KGDT_R0_PCR: u16 = 0x30;
const KGDT_R3_TEB: u16 = 0x38;
const KGDT_VDM_TILE: u16 = 0x40;
const KGDT_LDT: u16 = 0x48;
const KGDT_DF_TSS: u16 = 0x50;
const KGDT_NMI_TSS: u16 = 0x58;
// psdk/sdkddkver.h
const _WIN32_WINNT_VISTA: u16 = 0x0600;
// i386 CR0 paging bit
const CR0_PG: u32 = 0x80000000;
const CR4_PAE: u32 = 0x00000020;

// winldr.c `#pragma pack(2) typedef struct { USHORT Limit; ULONG Base; } GDTIDT;`
#[repr(C, packed(2))]
#[derive(Copy, Clone, Default)]
struct GDTIDT {
    Limit: u16,
    Base: u32,
}

// intrin_i.h: KiGetGdtEntry(pGdt, Selector) -> pGdt + (Selector & ~RPL_MASK)
#[inline]
unsafe fn KiGetGdtEntry(pGdt: PVOID, Selector: u16) -> *mut KGDTENTRY {
    ((pGdt as usize) + ((Selector & !RPL_MASK) as usize)) as *mut KGDTENTRY
}

// intrin_i.h: KiSetGdtDescriptorBase
#[inline]
unsafe fn KiSetGdtDescriptorBase(Entry: *mut KGDTENTRY, Base: u32) {
    (*Entry).BaseLow = (Base & 0xffff) as u16;
    (*Entry).HighWord.Bytes.BaseMid = ((Base >> 16) & 0xff) as u8;
    (*Entry).HighWord.Bytes.BaseHi = ((Base >> 24) & 0xff) as u8;
}

// intrin_i.h: KiSetGdtDescriptorLimit
#[inline]
unsafe fn KiSetGdtDescriptorLimit(Entry: *mut KGDTENTRY, mut Limit: u32) {
    if Limit < 0x100000 {
        (*Entry).HighWord.Bits.set_Granularity(0);
    } else {
        Limit >>= 12;
        (*Entry).HighWord.Bits.set_Granularity(1);
    }
    (*Entry).LimitLow = (Limit & 0xffff) as u16;
    (*Entry).HighWord.Bits.set_LimitHi((Limit >> 16) & 0x0f);
}

// intrin_i.h: KiSetGdtEntryEx
#[inline]
unsafe fn KiSetGdtEntryEx(
    Entry: *mut KGDTENTRY,
    Base: u32,
    Limit: u32,
    Type_: u8,
    Dpl: u8,
    Granularity: BOOLEAN,
    SegMode: u8,
) {
    KiSetGdtDescriptorBase(Entry, Base);
    KiSetGdtDescriptorLimit(Entry, Limit);
    (*Entry).HighWord.Bits.set_Type((Type_ & 0x1f) as u32);
    (*Entry).HighWord.Bits.set_Dpl((Dpl & 0x3) as u32);
    (*Entry).HighWord.Bits.set_Pres((Type_ != 0) as u32);
    (*Entry).HighWord.Bits.set_Sys(0);
    (*Entry).HighWord.Bits.set_Reserved_0(0);
    (*Entry).HighWord.Bits.set_Default_Big(((SegMode & 2) != 0) as u32);
    let g = (*Entry).HighWord.Bits.Granularity();
    (*Entry).HighWord.Bits.set_Granularity(g | ((Granularity != 0) as u32));
}

// intrin_i.h: KiSetGdtEntry
#[inline]
unsafe fn KiSetGdtEntry(Entry: *mut KGDTENTRY, Base: u32, Limit: u32, Type_: u8, Dpl: u8, SegMode: u8) {
    KiSetGdtEntryEx(Entry, Base, Limit, Type_, Dpl, 0 /* FALSE */, SegMode);
}

// intrin_i.h __lgdt / __sgdt / __lidt / __sidt (GNU inline asm)
#[inline]
unsafe fn __lgdt_i386(Descriptor: *const GDTIDT) {
    core::arch::asm!("lgdt [{0}]", in(reg) Descriptor, options(nostack));
}
#[inline]
unsafe fn __sgdt_i386(Descriptor: *mut GDTIDT) {
    core::arch::asm!("sgdt [{0}]", in(reg) Descriptor, options(nostack));
}
#[inline]
unsafe fn __lidt_i386(Descriptor: *const GDTIDT) {
    core::arch::asm!("lidt [{0}]", in(reg) Descriptor, options(nostack));
}
#[inline]
unsafe fn __sidt_i386(Descriptor: *mut GDTIDT) {
    core::arch::asm!("sidt [{0}]", in(reg) Descriptor, options(nostack));
}

// i386 CR / EFLAGS intrinsics (intrin.h equivalents)
#[inline]
unsafe fn __i386_disable() {
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn __i386_writeeflags(flags: u32) {
    core::arch::asm!("push {0}; popfd", in(reg) flags, options(nomem));
}
#[inline]
unsafe fn __i386_writecr3(v: u32) {
    core::arch::asm!("mov cr3, {0}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn __i386_readcr0() -> u32 {
    let v: u32;
    core::arch::asm!("mov {0}, cr0", out(reg) v, options(nomem, nostack, preserves_flags));
    v
}
#[inline]
unsafe fn __i386_writecr0(v: u32) {
    core::arch::asm!("mov cr0, {0}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn __i386_readcr4() -> u32 {
    let v: u32;
    core::arch::asm!("mov {0}, cr4", out(reg) v, options(nomem, nostack, preserves_flags));
    v
}
#[inline]
unsafe fn __i386_writecr4(v: u32) {
    core::arch::asm!("mov cr4, {0}", in(reg) v, options(nomem, nostack, preserves_flags));
}

// intrin_i.h Ke386Set* segment / table loaders
#[inline]
unsafe fn Ke386SetSs(v: u16) {
    core::arch::asm!("mov ss, {0:x}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn Ke386SetDs(v: u16) {
    core::arch::asm!("mov ds, {0:x}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn Ke386SetEs(v: u16) {
    core::arch::asm!("mov es, {0:x}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn Ke386SetGs(v: u16) {
    core::arch::asm!("mov gs, {0:x}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn Ke386SetFs(v: u16) {
    core::arch::asm!("mov fs, {0:x}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn Ke386SetLocalDescriptorTable(v: u16) {
    core::arch::asm!("lldt {0:x}", in(reg) v, options(nomem, nostack, preserves_flags));
}
#[inline]
unsafe fn Ke386SetTr(v: u16) {
    core::arch::asm!("ltr ax", in("ax") v, options(nomem, nostack, preserves_flags));
}

// arch/i386/winldr.c WinLdrSetProcessorContext
#[no_mangle]
pub unsafe extern "C" fn WinLdrSetProcessorContext(OperatingSystemVersion: u16) {
    let mut GdtDesc: GDTIDT = GDTIDT::default();
    let mut IdtDesc: GDTIDT = GDTIDT::default();
    let mut OldIdt: GDTIDT = GDTIDT::default();
    let pGdt: *mut KGDTENTRY;
    let pIdt: *mut KIDTENTRY;

    let Pcr: u32 = KIP0PCRADDRESS as u32;
    let Tss: u32 = (KSEG0_BASE as u32) | (TssBasePage << MM_PAGE_SHIFT);

    TRACE!("GdtIdt %p, Pcr %p, Tss 0x%08x\n", GdtIdt as usize, Pcr as usize, Tss as usize);

    /* Disable Interrupts */
    __i386_disable();

    /* Re-initialize EFLAGS */
    __i386_writeeflags(0);

    /*
     * UEFI hands control over with paging already enabled, and on x86 the
     * firmware (OVMF/EDK2) runs in PAE mode (CR4.PAE = 1). The i386 kernel
     * uses classic 2-level *non-PAE* paging, and PDE here is a non-PAE page
     * directory. Simply loading CR3=PDE while CR4.PAE=1 makes the MMU read
     * PDE as a PAE PDPT — garbage translation, immediate triple fault.
     *
     * Transition cleanly to the non-PAE directory: disable paging, clear
     * CR4.PAE, load CR3, then re-enable paging. While paging is off the CPU
     * runs identity (linear == physical), which holds because the loader
     * image and its stack live in identity-mapped low memory under the
     * firmware. On a legacy BIOS boot paging is already off here, so the
     * CR0_PG test makes this a no-op. (i386 UEFI boot bring-up.)
     */
    let cr0_now = __i386_readcr0();
    if (cr0_now & CR0_PG) != 0 {
        /* Disable paging first (still on the firmware's PAE tables). */
        __i386_writecr0(cr0_now & !CR0_PG);
        /* Now clear CR4.PAE so the upcoming CR3 is read as a 32-bit PDE. */
        __i386_writecr4(__i386_readcr4() & !CR4_PAE);
    }

    /* Set the PDBR */
    __i386_writecr3(PDE as u32);

    /* Enable paging by modifying CR0 */
    __i386_writecr0(__i386_readcr0() | CR0_PG);

    /* The Kernel expects the boot processor PCR to be zero-filled on startup */
    RtlZeroMemory(Pcr as *mut core::ffi::c_void, MM_PAGE_SIZE as usize);

    /* Get old values of GDT and IDT */
    __sgdt_i386(&mut GdtDesc);
    __sidt_i386(&mut IdtDesc);

    /* Save old IDT */
    OldIdt.Base = IdtDesc.Base;
    OldIdt.Limit = IdtDesc.Limit;

    /* Prepare new IDT+GDT */
    GdtDesc.Base = (KSEG0_BASE as u32) | (GdtIdt as u32);
    GdtDesc.Limit = (NUM_GDT * core::mem::size_of::<KGDTENTRY>() as u32 - 1) as u16;
    IdtDesc.Base = GdtDesc.Base + GdtDesc.Limit as u32 + 1;
    IdtDesc.Limit = (NUM_IDT * core::mem::size_of::<KIDTENTRY>() as u32 - 1) as u16;

    pGdt = GdtDesc.Base as *mut KGDTENTRY;
    pIdt = IdtDesc.Base as *mut KIDTENTRY;

    /* KGDT_NULL (0x00) Null Selector - Unused */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, KGDT_NULL), 0x0000, 0, 0, DPL_SYSTEM, 0);

    /* KGDT_R0_CODE (0x08) Ring 0 Code selector - Flat 4Gb */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, KGDT_R0_CODE), 0x0000, 0xFFFFFFFF, TYPE_CODE, DPL_SYSTEM, 2);

    /* KGDT_R0_DATA (0x10) Ring 0 Data selector - Flat 4Gb */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, KGDT_R0_DATA), 0x0000, 0xFFFFFFFF, TYPE_DATA, DPL_SYSTEM, 2);

    /* KGDT_R3_CODE (0x18) Ring 3 Code Selector - Flat 2Gb */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, KGDT_R3_CODE), 0x0000, 0xFFFFFFFF, TYPE_CODE, DPL_USER, 2);

    /* KGDT_R3_DATA (0x20) Ring 3 Data Selector - Flat 2Gb */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, KGDT_R3_DATA), 0x0000, 0xFFFFFFFF, TYPE_DATA, DPL_USER, 2);

    /* KGDT_TSS (0x28) TSS Selector (Ring 0) */
    KiSetGdtEntry(
        KiGetGdtEntry(pGdt as PVOID, KGDT_TSS),
        Tss,
        0x78 - 1, /* FIXME: Check this; would be sizeof(KTSS)-1 */
        TYPE_TSS32A,
        DPL_SYSTEM,
        0,
    );

    /*
     * KGDT_R0_PCR (0x30) PCR Selector (Ring 0)
     */
    if OperatingSystemVersion < _WIN32_WINNT_VISTA {
        /* Server 2003 way */
        KiSetGdtEntryEx(
            KiGetGdtEntry(pGdt as PVOID, KGDT_R0_PCR),
            Pcr,
            0x1,
            TYPE_DATA,
            DPL_SYSTEM,
            1, /* TRUE */
            2,
        );
    } else {
        /* Vista+ way */
        KiSetGdtEntry(
            KiGetGdtEntry(pGdt as PVOID, KGDT_R0_PCR),
            Pcr,
            MM_PAGE_SIZE as u32 - 1,
            TYPE_DATA,
            DPL_SYSTEM,
            2,
        );
    }

    /* KGDT_R3_TEB (0x38) Thread Environment Block Selector (Ring 3) */
    KiSetGdtEntry(
        KiGetGdtEntry(pGdt as PVOID, KGDT_R3_TEB),
        0x0000,
        0x0FFF,
        TYPE_DATA | DESCRIPTOR_ACCESSED,
        DPL_USER,
        2,
    );

    /* KGDT_VDM_TILE (0x40) VDM BIOS Data Area Selector (Ring 3) */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, KGDT_VDM_TILE), 0x0400, 0xFFFF, TYPE_DATA, DPL_USER, 0);

    /* KGDT_LDT (0x48) LDT Selector (Reserved) */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, KGDT_LDT), 0x0000, 0, 0, DPL_SYSTEM, 0);

    /* KGDT_DF_TSS (0x50) Double-Fault Task Switch Selector (Ring 0) */
    KiSetGdtEntry(
        KiGetGdtEntry(pGdt as PVOID, KGDT_DF_TSS),
        0x20000, /* FIXME: Not correct! */
        0xFFFF,  /* FIXME: Not correct! */
        TYPE_TSS32A,
        DPL_SYSTEM,
        0,
    );

    /* KGDT_NMI_TSS (0x58) NMI Task Switch Selector (Ring 0) */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, KGDT_NMI_TSS), 0x20000, 0xFFFF, TYPE_CODE, DPL_SYSTEM, 0);

    /* Selector (0x60) - Unused (Reserved) on Windows Longhorn/Vista+ */
    KiSetGdtEntry(
        KiGetGdtEntry(pGdt as PVOID, 0x60),
        0x20000, /* FIXME: Maybe not correct, but noone cares */
        0xFFFF,
        TYPE_DATA,
        DPL_SYSTEM,
        0,
    );

    /* Video Display Buffer Selector (0x68) - Unused on Vista+ */
    KiSetGdtEntry(KiGetGdtEntry(pGdt as PVOID, 0x68), 0xB8000, 0x3FFF, TYPE_DATA, DPL_SYSTEM, 0);

    /* GDT Alias Selector (points to GDT) (0x70) */
    KiSetGdtEntry(
        KiGetGdtEntry(pGdt as PVOID, 0x70),
        0xFFFF7000, /* GdtDesc.Base ?? */
        GdtDesc.Limit as u32,
        TYPE_DATA,
        DPL_SYSTEM,
        0,
    );

    /* Copy the old IDT */
    RtlCopyMemory(
        pIdt as *mut core::ffi::c_void,
        OldIdt.Base as *const core::ffi::c_void,
        (OldIdt.Limit as usize) + 1,
    );

    /* Load GDT+IDT */
    __lgdt_i386(&GdtDesc);
    __lidt_i386(&IdtDesc);

    /* Jump to proper CS and clear prefetch queue */
    core::arch::asm!("ljmp $0x08, $1f", "1:", options(att_syntax));

    /* Set SS selector */
    Ke386SetSs(KGDT_R0_DATA);

    /* Set DS and ES selectors */
    Ke386SetDs(KGDT_R0_DATA);
    Ke386SetEs(KGDT_R0_DATA); // This is vital for rep stosd

    /* LDT = not used ever, thus set to 0 */
    Ke386SetLocalDescriptorTable(0);

    /* Load TSR */
    Ke386SetTr(KGDT_TSS);

    /* Clear GS */
    Ke386SetGs(0);

    /* Set FS to PCR */
    Ke386SetFs(KGDT_R0_PCR);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         EFI Windows Loader
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            boot/freeldr/freeldr/arch/i386/winldr.c
//  * PURPOSE:         Memory related routines
//  * PROGRAMMERS:     Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <freeldr.h>
// #include <ndk/asm.h>
// #include <internal/i386/intrin_i.h>
// #include "../../winldr.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WINDOWS);
//
// // This is needed because headers define wrong one for ReactOS
// #undef KIP0PCRADDRESS
// #define KIP0PCRADDRESS                      0xffdff000
//
// #define SELFMAP_ENTRY       0x300
//
// // This is needed only for SetProcessorContext routine
// #pragma pack(2)
// typedef struct
// {
//     USHORT Limit;
//     ULONG Base;
// } GDTIDT;
// #pragma pack(4)
//
// /*
//  * Consider adding these definitions into
//  * ntoskrnl/include/internal/i386/intrin_i.h and/or the NDK.
//  */
//
// #define DPL_SYSTEM  0
// #define DPL_USER    3
//
// #define TYPE_TSS16A 0x01    // 16-bit Task State Segment (Available)
// #define TYPE_LDT    0x02    // Local Descriptor Table
// #define TYPE_TSS16B 0x03    // 16-bit Task State Segment (Busy)
// #define TYPE_CALL16 0x04    // 16-bit Call Gate
// #define TYPE_TASK   0x05    // Task Gate (I386_TASK_GATE)
// #define TYPE_INT16  0x06    // 16-bit Interrupt Gate
// #define TYPE_TRAP16 0x07    // 16-bit Trap Gate
// // #define TYPE_RESERVED_1 0x08
// #define TYPE_TSS32A 0x09    // 32-bit Task State Segment (Available) (I386_TSS)
// // #define TYPE_RESERVED_2 0x0A
// #define TYPE_TSS32B 0x0B    // 32-bit Task State Segment (Busy) (I386_ACTIVE_TSS)
// #define TYPE_CALL32 0x0C    // 32-bit Call Gate (I386_CALL_GATE)
// // #define TYPE_RESERVED_3 0x0D
// #define TYPE_INT32  0x0E    // 32-bit Interrupt Gate (I386_INTERRUPT_GATE)
// #define TYPE_TRAP32 0x0F    // 32-bit Trap Gate (I386_TRAP_GATE)
//
// #define DESCRIPTOR_ACCESSED     0x1
// #define DESCRIPTOR_READ_WRITE   0x2
// #define DESCRIPTOR_EXECUTE_READ 0x2
// #define DESCRIPTOR_EXPAND_DOWN  0x4
// #define DESCRIPTOR_CONFORMING   0x4
// #define DESCRIPTOR_CODE         0x8
//
// #define TYPE_CODE   (0x10 | DESCRIPTOR_CODE | DESCRIPTOR_EXECUTE_READ)
// #define TYPE_DATA   (0x10 | DESCRIPTOR_READ_WRITE)
//
// #if 0
// VOID
// DumpGDTEntry(ULONG_PTR Base, ULONG Selector)
// {
//     PKGDTENTRY pGdt = (PKGDTENTRY)((ULONG_PTR)Base + Selector);
//
//     TRACE("\n"
//           "Selector 0x%04x\n"
//           "===============\n"
//           "LimitLow               = 0x%04x\n"
//           "BaseLow                = 0x%04x\n"
//           "HighWord.Bytes.BaseMid = 0x%02x\n"
//           "HighWord.Bytes.Flags1  = 0x%02x\n"
//           "HighWord.Bytes.Flags2  = 0x%02x\n"
//           "HighWord.Bytes.BaseHi  = 0x%02x\n"
//           "\n", (//           Selector) as usize, (//           pGdt->LimitLow) as usize, (pGdt->BaseLow) as usize, (//           pGdt->HighWord.Bytes.BaseMid) as usize, (//           pGdt->HighWord.Bytes.Flags1) as usize, (//           pGdt->HighWord.Bytes.Flags2) as usize, (//           pGdt->HighWord.Bytes.BaseHi) as usize);
// }
// #endif
//
// /* GLOBALS *******************************************************************/
//
// PHARDWARE_PTE PDE;
// PHARDWARE_PTE HalPageTable;
//
// PUCHAR PhysicalPageTablesBuffer;
// PUCHAR KernelPageTablesBuffer;
// ULONG PhysicalPageTables;
// ULONG KernelPageTables;
//
// ULONG PcrBasePage;
// ULONG TssBasePage;
// PVOID GdtIdt;
//
// /* FUNCTIONS *****************************************************************/
//
// static
// BOOLEAN
// MempAllocatePageTables(VOID)
// {
//     ULONG NumPageTables, TotalSize;
//     PUCHAR Buffer;
//     // It's better to allocate PDE + PTEs contiguous
//
//     // Max number of entries = MaxPageNum >> 10
//     // FIXME: This is a number to describe ALL physical memory
//     // and windows doesn't expect ALL memory mapped...
//     NumPageTables = MmGetTotalPagesInLookupTable() >> 10;
//
//     TRACE("NumPageTables = %d\n", (NumPageTables) as usize);
//
//     // Allocate memory block for all these things:
//     // PDE, HAL mapping page table, physical mapping, kernel mapping
//     TotalSize = (1 + 1 + NumPageTables * 2) * MM_PAGE_SIZE;
//
//     // PDE+HAL+KernelPTEs == MemoryData
//     Buffer = MmAllocateMemoryWithType(TotalSize, LoaderMemoryData);
//
//     // Physical PTEs = FirmwareTemporary
//     PhysicalPageTablesBuffer = (PUCHAR)Buffer + TotalSize - NumPageTables*MM_PAGE_SIZE;
//     MmSetMemoryType(PhysicalPageTablesBuffer,
//                     NumPageTables*MM_PAGE_SIZE,
//                     LoaderFirmwareTemporary);
//
//     // This check is now redundant
//     if (Buffer + (TotalSize - NumPageTables*MM_PAGE_SIZE) !=
//         PhysicalPageTablesBuffer)
//     {
//         TRACE("There was a problem allocating two adjacent blocks of memory!\n");
//     }
//
//     if (Buffer == NULL || PhysicalPageTablesBuffer == NULL)
//     {
//         UiMessageBox("Impossible to allocate memory block for page tables!");
//         return FALSE;
//     }
//
//     // Zero all this memory block
//     RtlZeroMemory(Buffer, TotalSize);
//
//     // Set up pointers correctly now
//     PDE = (PHARDWARE_PTE)Buffer;
//
//     // Map the page directory at 0xC0000000 (maps itself)
//     PDE[SELFMAP_ENTRY].PageFrameNumber = (ULONG)PDE >> MM_PAGE_SHIFT;
//     PDE[SELFMAP_ENTRY].Valid = 1;
//     PDE[SELFMAP_ENTRY].Write = 1;
//
//     // The last PDE slot is allocated for HAL's memory mapping (Virtual Addresses 0xFFC00000 - 0xFFFFFFFF)
//     HalPageTable = (PHARDWARE_PTE)&Buffer[MM_PAGE_SIZE*1];
//
//     // Map it
//     PDE[1023].PageFrameNumber = (ULONG)HalPageTable >> MM_PAGE_SHIFT;
//     PDE[1023].Valid = 1;
//     PDE[1023].Write = 1;
//
//     // Store pointer to the table for easier access
//     KernelPageTablesBuffer = &Buffer[MM_PAGE_SIZE*2];
//
//     // Zero counters of page tables used
//     PhysicalPageTables = 0;
//     KernelPageTables = 0;
//
//     return TRUE;
// }
//
// static
// VOID
// MempAllocatePTE(ULONG Entry, PHARDWARE_PTE *PhysicalPT, PHARDWARE_PTE *KernelPT)
// {
//     //TRACE("Creating PDE Entry %X\n", (Entry) as usize);
//
//     // Identity mapping
//     *PhysicalPT = (PHARDWARE_PTE)&PhysicalPageTablesBuffer[PhysicalPageTables*MM_PAGE_SIZE];
//     PhysicalPageTables++;
//
//     PDE[Entry].PageFrameNumber = (ULONG)*PhysicalPT >> MM_PAGE_SHIFT;
//     PDE[Entry].Valid = 1;
//     PDE[Entry].Write = 1;
//
//     if (Entry+(KSEG0_BASE >> 22) as u32 > 1023)
//     {
//         TRACE("WARNING! Entry: %X > 1023\n", (Entry+(KSEG0_BASE >> 22) as u32) as usize);
//     }
//
//     // Kernel-mode mapping
//     *KernelPT = (PHARDWARE_PTE)&KernelPageTablesBuffer[KernelPageTables*MM_PAGE_SIZE];
//     KernelPageTables++;
//
//     PDE[Entry+(KSEG0_BASE >> 22) as u32].PageFrameNumber = ((ULONG)*KernelPT >> MM_PAGE_SHIFT);
//     PDE[Entry+(KSEG0_BASE >> 22) as u32].Valid = 1;
//     PDE[Entry+(KSEG0_BASE >> 22) as u32].Write = 1;
// }
//
// BOOLEAN
// MempSetupPaging(IN PFN_NUMBER StartPage,
//                 IN PFN_COUNT NumberOfPages,
//                 IN BOOLEAN KernelMapping)
// {
//     PHARDWARE_PTE PhysicalPT;
//     PHARDWARE_PTE KernelPT;
//     PFN_COUNT Entry, Page;
//
//     TRACE("MempSetupPaging: SP 0x%X, Number: 0x%X, Kernel: %s\n", (//        StartPage) as usize, (NumberOfPages) as usize, (KernelMapping ? "yes" : "no") as usize);
//
//     // HACK
//     if (StartPage+NumberOfPages >= 0x80000)
//     {
//         //
//         // We cannot map this as it requires more than 1 PDE
//         // and in fact it's not possible at all ;)
//         //
//         //TRACE("skipping...\n");
//         return TRUE;
//     }
//
//     //
//     // Now actually set up the page tables for identity mapping
//     //
//     for (Page = StartPage; Page < StartPage + NumberOfPages; Page++)
//     {
//         Entry = Page >> 10;
//
//         if (((PULONG)PDE)[Entry] == 0)
//         {
//             MempAllocatePTE(Entry, &PhysicalPT, &KernelPT);
//         }
//         else
//         {
//             PhysicalPT = (PHARDWARE_PTE)(PDE[Entry].PageFrameNumber << MM_PAGE_SHIFT);
//             KernelPT = (PHARDWARE_PTE)(PDE[Entry+(KSEG0_BASE >> 22) as u32].PageFrameNumber << MM_PAGE_SHIFT);
//         }
//
//         PhysicalPT[Page & 0x3ff].PageFrameNumber = Page;
//         PhysicalPT[Page & 0x3ff].Valid = 1;
//         PhysicalPT[Page & 0x3ff].Write = (Page != 0);
//
//         if (KernelMapping)
//         {
//             if (KernelPT[Page & 0x3ff].Valid) WARN("KernelPT already mapped\n");
//             KernelPT[Page & 0x3ff].PageFrameNumber = Page;
//             KernelPT[Page & 0x3ff].Valid = (Page != 0);
//             KernelPT[Page & 0x3ff].Write = (Page != 0);
//         }
//     }
//
//     return TRUE;
// }
//
// VOID
// MempUnmapPage(PFN_NUMBER Page)
// {
//     PHARDWARE_PTE KernelPT;
//     PFN_NUMBER Entry = (Page >> 10) + (KSEG0_BASE >> 22) as u32;
//
//     /* Don't unmap page directory or HAL entries */
//     if (Entry == SELFMAP_ENTRY || Entry == 1023)
//         return;
//
//     if (PDE[Entry].Valid)
//     {
//         KernelPT = (PHARDWARE_PTE)(PDE[Entry].PageFrameNumber << MM_PAGE_SHIFT);
//
//         if (KernelPT)
//         {
//             KernelPT[Page & 0x3ff].PageFrameNumber = 0;
//             KernelPT[Page & 0x3ff].Valid = 0;
//             KernelPT[Page & 0x3ff].Write = 0;
//         }
//     }
// }
//
// static
// VOID
// WinLdrpMapApic(VOID)
// {
//     BOOLEAN LocalAPIC;
//     LARGE_INTEGER MsrValue;
//     ULONG APICAddress, CpuInfo[4];
//
//     /* Check if we have a local APIC */
//     __cpuid((int*)CpuInfo, 1);
//     LocalAPIC = (((CpuInfo[3] >> 9) & 1) != 0);
//
//     /* If there is no APIC, just return */
//     if (!LocalAPIC)
//         return;
//
//     /* Read the APIC Address */
//     MsrValue.QuadPart = __readmsr(0x1B);
//     APICAddress = (MsrValue.LowPart & 0xFFFFF000);
//
//     TRACE("Local APIC detected at address 0x%x\n", (//         APICAddress) as usize);
//
//     /* Map it */
//     HalPageTable[(APIC_BASE - 0xFFC00000) >> MM_PAGE_SHIFT].PageFrameNumber
//         = APICAddress >> MM_PAGE_SHIFT;
//     HalPageTable[(APIC_BASE - 0xFFC00000) >> MM_PAGE_SHIFT].Valid = 1;
//     HalPageTable[(APIC_BASE - 0xFFC00000) >> MM_PAGE_SHIFT].Write = 1;
//     HalPageTable[(APIC_BASE - 0xFFC00000) >> MM_PAGE_SHIFT].WriteThrough = 1;
//     HalPageTable[(APIC_BASE - 0xFFC00000) >> MM_PAGE_SHIFT].CacheDisable = 1;
// }
//
// static
// BOOLEAN
// WinLdrMapSpecialPages(void)
// {
//     TRACE("HalPageTable: 0x%X\n", (HalPageTable) as usize);
//
//     /*
//      * The Page Tables have been setup, make special handling
//      * for the boot processor PCR and KI_USER_SHARED_DATA.
//      */
//     HalPageTable[(KI_USER_SHARED_DATA - 0xFFC00000) >> MM_PAGE_SHIFT].PageFrameNumber = PcrBasePage+1;
//     HalPageTable[(KI_USER_SHARED_DATA - 0xFFC00000) >> MM_PAGE_SHIFT].Valid = 1;
//     HalPageTable[(KI_USER_SHARED_DATA - 0xFFC00000) >> MM_PAGE_SHIFT].Write = 1;
//
//     HalPageTable[(KIP0PCRADDRESS - 0xFFC00000) >> MM_PAGE_SHIFT].PageFrameNumber = PcrBasePage;
//     HalPageTable[(KIP0PCRADDRESS - 0xFFC00000) >> MM_PAGE_SHIFT].Valid = 1;
//     HalPageTable[(KIP0PCRADDRESS - 0xFFC00000) >> MM_PAGE_SHIFT].Write = 1;
//
//     /* Map APIC */
//     WinLdrpMapApic();
//
//     /* Map VGA memory */
//     //VideoMemoryBase = MmMapIoSpace(0xb8000, 4000, MmNonCached);
//     //TRACE("VideoMemoryBase: 0x%X\n", (VideoMemoryBase) as usize);
//
//     return TRUE;
// }
//
// #define ExtendedBIOSDataArea ((PULONG)0x740)
// #define ExtendedBIOSDataSize ((PULONG)0x744)
// #define RomFontPointers ((PULONG)0x700)
//
// static
// void WinLdrSetupSpecialDataPointers(VOID)
// {
//     /* Get the address of the BIOS ROM fonts. Win 2003 videoprt reads these
//        values from address 0x700 .. 0x718 and store them in the registry
//        in HKLM\System\CurrentControlSet\Control\Wow\RomFontPointers */
//     MachVideoGetFontsFromFirmware(RomFontPointers);
//
//     /* Store address of the extended BIOS data area in 0x740 */
//     MachGetExtendedBIOSData(ExtendedBIOSDataArea, ExtendedBIOSDataSize);
//
//     if (*ExtendedBIOSDataArea == 0 && *ExtendedBIOSDataSize == 0)
//     {
//         WARN("Couldn't get address of extended BIOS data area\n");
//     }
//     else
//     {
//         TRACE("*ExtendedBIOSDataArea = 0x%lx\n", (*ExtendedBIOSDataArea) as usize);
//     }
// }
//
// void WinLdrSetupMachineDependent(PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     ULONG TssSize;
//     //ULONG TssPages;
//     ULONG_PTR Pcr = 0;
//     ULONG_PTR Tss = 0;
//     ULONG BlockSize, NumPages;
//
//     LoaderBlock->u.I386.CommonDataArea = NULL; // Force No ABIOS support
//     LoaderBlock->u.I386.MachineType = MACHINE_TYPE_ISA;
//
//     /* Allocate 2 pages for PCR: one for the boot processor PCR and one for KI_USER_SHARED_DATA */
//     Pcr = (ULONG_PTR)MmAllocateMemoryWithType(2 * MM_PAGE_SIZE, LoaderStartupPcrPage);
//     PcrBasePage = Pcr >> MM_PAGE_SHIFT;
//     if (Pcr == 0)
//     {
//         UiMessageBox("Could not allocate PCR.");
//         return;
//     }
//
//     /* Allocate TSS */
//     TssSize = (sizeof(KTSS) + MM_PAGE_SIZE) & ~(MM_PAGE_SIZE - 1);
//     //TssPages = TssSize / MM_PAGE_SIZE;
//
//     Tss = (ULONG_PTR)MmAllocateMemoryWithType(TssSize, LoaderMemoryData);
//     TssBasePage = Tss >> MM_PAGE_SHIFT;
//     if (Tss == 0)
//     {
//         UiMessageBox("Could not allocate TSS.");
//         return;
//     }
//
//     /* Allocate space for new GDT + IDT */
//     BlockSize = NUM_GDT*sizeof(KGDTENTRY) + NUM_IDT*sizeof(KIDTENTRY);//FIXME: Use GDT/IDT limits here?
//     NumPages = (BlockSize + MM_PAGE_SIZE - 1) >> MM_PAGE_SHIFT;
//     GdtIdt = (PKGDTENTRY)MmAllocateMemoryWithType(NumPages * MM_PAGE_SIZE, LoaderMemoryData);
//     if (GdtIdt == NULL)
//     {
//         UiMessageBox("Could not allocate pages for GDT+IDT!");
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
//         BugCheck("MempAllocatePageTables failed!\n");
//     }
//
//     /* Map stuff like PCR, KI_USER_SHARED_DATA and Apic */
//     WinLdrMapSpecialPages();
//
//     /* Set some special fields */
//     WinLdrSetupSpecialDataPointers();
// }
//
//
// VOID
// WinLdrSetProcessorContext(
//     _In_ USHORT OperatingSystemVersion)
// {
//     GDTIDT GdtDesc, IdtDesc, OldIdt;
//     PKGDTENTRY    pGdt;
//     PKIDTENTRY    pIdt;
//     USHORT Ldt = 0;
//     ULONG Pcr;
//     ULONG Tss;
//     //ULONG i;
//
//     Pcr = KIP0PCRADDRESS;
//     Tss = KSEG0_BASE | (TssBasePage << MM_PAGE_SHIFT);
//
//     TRACE("GdtIdt %p, Pcr %p, Tss 0x%08x\n", (//           GdtIdt) as usize, (Pcr) as usize, (Tss) as usize);
//
//     /* Enable paging */
//     //BS->ExitBootServices(ImageHandle,MapKey);
//
//     /* Disable Interrupts */
//     _disable();
//
//     /* Re-initialize EFLAGS */
//     __writeeflags(0);
//
//     /* Set the PDBR */
//     __writecr3((ULONG_PTR)PDE);
//
//     /* Enable paging by modifying CR0 */
//     __writecr0(__readcr0() | CR0_PG);
//
//     /* The Kernel expects the boot processor PCR to be zero-filled on startup */
//     RtlZeroMemory((PVOID)Pcr, MM_PAGE_SIZE);
//
//     /* Get old values of GDT and IDT */
//     Ke386GetGlobalDescriptorTable(&GdtDesc);
//     __sidt(&IdtDesc);
//
//     /* Save old IDT */
//     OldIdt.Base  = IdtDesc.Base;
//     OldIdt.Limit = IdtDesc.Limit;
//
//     /* Prepare new IDT+GDT */
//     GdtDesc.Base  = KSEG0_BASE | (ULONG_PTR)GdtIdt;
//     GdtDesc.Limit = NUM_GDT * sizeof(KGDTENTRY) - 1;
//     IdtDesc.Base  = (ULONG)((PUCHAR)GdtDesc.Base + GdtDesc.Limit + 1);
//     IdtDesc.Limit = NUM_IDT * sizeof(KIDTENTRY) - 1;
//
//     // ========================
//     // Fill all descriptors now
//     // ========================
//
//     pGdt = (PKGDTENTRY)GdtDesc.Base;
//     pIdt = (PKIDTENTRY)IdtDesc.Base;
//
//     /* KGDT_NULL (0x00) Null Selector - Unused */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_NULL), 0x0000, 0, 0, DPL_SYSTEM, 0);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_NULL);
//
//     /* KGDT_R0_CODE (0x08) Ring 0 Code selector - Flat 4Gb */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_R0_CODE), 0x0000, 0xFFFFFFFF,
//                   TYPE_CODE, DPL_SYSTEM, 2);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_R0_CODE);
//
//     /* KGDT_R0_DATA (0x10) Ring 0 Data selector - Flat 4Gb */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_R0_DATA), 0x0000, 0xFFFFFFFF,
//                   TYPE_DATA, DPL_SYSTEM, 2);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_R0_DATA);
//
//     /* KGDT_R3_CODE (0x18) Ring 3 Code Selector - Flat 2Gb */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_R3_CODE), 0x0000, 0xFFFFFFFF,
//                   TYPE_CODE, DPL_USER, 2);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_R3_CODE);
//
//     /* KGDT_R3_DATA (0x20) Ring 3 Data Selector - Flat 2Gb */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_R3_DATA), 0x0000, 0xFFFFFFFF,
//                   TYPE_DATA, DPL_USER, 2);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_R3_DATA);
//
//     /* KGDT_TSS (0x28) TSS Selector (Ring 0) */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_TSS), (ULONG32)Tss,
//                   0x78-1 /* FIXME: Check this; would be sizeof(KTSS)-1 */,
//                   TYPE_TSS32A, DPL_SYSTEM, 0);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_TSS);
//
//     /*
//      * KGDT_R0_PCR (0x30) PCR Selector (Ring 0)
//      *
//      * IMPORTANT COMPATIBILITY NOTE:
//      * Windows <= Server 2003 sets a Base == KIP0PCRADDRESS (0xffdff000 on x86)
//      * with a Limit of 1 page (LimitLow == 1, Granularity == 1, that is interpreted
//      * as a "Limit" == 0x1fff but is incorrect, of course).
//      * On Windows Longhorn/Vista+ however, the Base is not hardcoded to KIP0PCRADDRESS
//      * and the limit is set in bytes (Granularity == 0):
//      * Longhorn/Vista reports LimitLow == 0x0fff == MM_PAGE_SIZE - 1, whereas
//      * Windows 7+ uses larger sizes there (not aligned on a page boundary).
//      */
//     if (OperatingSystemVersion < _WIN32_WINNT_VISTA)
//     {
//         /* Server 2003 way */
//         KiSetGdtEntryEx(KiGetGdtEntry(pGdt, KGDT_R0_PCR), (ULONG32)Pcr, 0x1,
//                         TYPE_DATA, DPL_SYSTEM, TRUE, 2);
//     }
//     else
//     {
//         /* Vista+ way */
//         KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_R0_PCR), (ULONG32)Pcr, MM_PAGE_SIZE - 1,
//                       TYPE_DATA, DPL_SYSTEM, 2);
//     }
//
//     // DumpGDTEntry(GdtDesc.Base, KGDT_R0_PCR);
//
//     /* KGDT_R3_TEB (0x38) Thread Environment Block Selector (Ring 3) */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_R3_TEB), 0x0000, 0x0FFF,
//                   TYPE_DATA | DESCRIPTOR_ACCESSED, DPL_USER, 2);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_R3_TEB);
//
//     /* KGDT_VDM_TILE (0x40) VDM BIOS Data Area Selector (Ring 3) */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_VDM_TILE), 0x0400, 0xFFFF,
//                   TYPE_DATA, DPL_USER, 0);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_VDM_TILE);
//
//     /* KGDT_LDT (0x48) LDT Selector (Reserved) */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_LDT), 0x0000, 0, 0, DPL_SYSTEM, 0);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_LDT);
//
//     /* KGDT_DF_TSS (0x50) Double-Fault Task Switch Selector (Ring 0) */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_DF_TSS), 0x20000 /* FIXME: Not correct! */,
//                   0xFFFF /* FIXME: Not correct! */,
//                   TYPE_TSS32A, DPL_SYSTEM, 0);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_DF_TSS);
//
//     /* KGDT_NMI_TSS (0x58) NMI Task Switch Selector (Ring 0) */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, KGDT_NMI_TSS), 0x20000, 0xFFFF,
//                   TYPE_CODE, DPL_SYSTEM, 0);
//     // DumpGDTEntry(GdtDesc.Base, KGDT_NMI_TSS);
//
//     /* Selector (0x60) - Unused (Reserved) on Windows Longhorn/Vista+ */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, 0x60), 0x20000 /* FIXME: Maybe not correct, but noone cares */,
//                   0xFFFF, TYPE_DATA, DPL_SYSTEM, 0);
//     // DumpGDTEntry(GdtDesc.Base, 0x60);
//
//     /* Video Display Buffer Selector (0x68) - Unused (Reserved) on Windows Longhorn/Vista+ */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, 0x68), 0xB8000, 0x3FFF,
//                   TYPE_DATA, DPL_SYSTEM, 0);
//     // DumpGDTEntry(GdtDesc.Base, 0x68);
//
//     /*
//      * GDT Alias Selector (points to GDT) (0x70)
//      *
//      * IMPORTANT COMPATIBILITY NOTE:
//      * The Base is not fixed to 0xFFFF7000 on Windows Longhorn/Vista+.
//      */
//     KiSetGdtEntry(KiGetGdtEntry(pGdt, 0x70), 0xFFFF7000 /* GdtDesc.Base ?? */, GdtDesc.Limit,
//                   TYPE_DATA, DPL_SYSTEM, 0);
//     // DumpGDTEntry(GdtDesc.Base, 0x70);
//
//     /*
//      * Windows <= Server 2003 defines three additional unused but valid
//      * descriptors there, possibly for debugging purposes only.
//      * They are not present on Windows Longhorn/Vista+.
//      */
//     // KiSetGdtEntry(KiGetGdtEntry(pGdt, 0x78), 0x80400000, 0xffff, TYPE_CODE, DPL_SYSTEM, 0);
//     // KiSetGdtEntry(KiGetGdtEntry(pGdt, 0x80), 0x80400000, 0xffff, TYPE_DATA, DPL_SYSTEM, 0);
//     // KiSetGdtEntry(KiGetGdtEntry(pGdt, 0x88), 0x0000, 0, TYPE_DATA, DPL_SYSTEM, 0);
//
//     /* Copy the old IDT */
//     RtlCopyMemory(pIdt, (PVOID)OldIdt.Base, OldIdt.Limit + 1);
//
//     /* Mask interrupts */
//     //asm("cli\n"); // they are already masked before enabling paged mode
//
//     /* Load GDT+IDT */
//     Ke386SetGlobalDescriptorTable(&GdtDesc);
//     __lidt(&IdtDesc);
//
//     /* Jump to proper CS and clear prefetch queue */
// #if defined(__GNUC__) || defined(__clang__)
//     asm("ljmp    $0x08, $1f\n"
//         "1:\n");
// #elif defined(_MSC_VER)
//     /* We cannot express the above in MASM so we use this far return instead */
//     __asm
//     {
//         push 8
//         push offset resume
//         retf
//         resume:
//     };
// #else
// #error
// #endif
//
//     /* Set SS selector */
//     Ke386SetSs(KGDT_R0_DATA);
//
//     /* Set DS and ES selectors */
//     Ke386SetDs(KGDT_R0_DATA);
//     Ke386SetEs(KGDT_R0_DATA); // This is vital for rep stosd
//
//     /* LDT = not used ever, thus set to 0 */
//     Ke386SetLocalDescriptorTable(Ldt);
//
//     /* Load TSR */
//     Ke386SetTr(KGDT_TSS);
//
//     /* Clear GS */
//     Ke386SetGs(0);
//
//     /* Set FS to PCR */
//     Ke386SetFs(KGDT_R0_PCR);
//
//     // Real end of the function, just for information
//     /* do not uncomment!
//     pop edi;
//     pop esi;
//     pop ebx;
//     mov esp, ebp;
//     pop ebp;
//     ret
//     */
// }
//
// #if DBG
// VOID
// MempDump(VOID)
// {
//     PULONG PDE_Addr=(PULONG)PDE;//0xC0300000;
//     int i, j;
//
//     TRACE("\nPDE\n");
//
//     for (i=0; i<128; i++)
//     {
//         TRACE("0x%04X | ", (i*8) as usize);
//
//         for (j=0; j<8; j++)
//         {
//             TRACE("0x%08X ", (PDE_Addr[i*8+j]) as usize);
//         }
//
//         TRACE("\n");
//     }
// }
// #endif
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
