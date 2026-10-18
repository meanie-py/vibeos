use crate::cmconfig::CmpInitializeRegistryNode;
// ke globals referenced across the config crate boundary as link symbols (all
// #[no_mangle] in ntoskrnl-rust). KeI386NpxPresent has the same symbol on both x86
// arches (the old cfg was only for the Rust path). KiProcessorBlock is externed as
// its element 0 — the code only takes its address to index the PKPRCB array.
extern "C" {
    static mut KeI386NpxPresent: ULONG;
    static mut KiProcessorBlock: PKPRCB;
    static mut KeNumberProcessors: core::ffi::c_char; // CCHAR
}

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::macros::NtCurrentProcess;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::opaque_ptr;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    KeRevertToUserAffinityThread, KeSetSystemAffinityThread, NtClose, NtCreateKey,
    NtOpenKey, NtSetValueKey, RtlAnsiStringToUnicodeString, RtlCompareMemory,
    RtlCreateUnicodeString, RtlFreeUnicodeString, RtlInitAnsiString, RtlInitUnicodeString, ZwClose,
    ZwMapViewOfSection, ZwOpenSection, ZwUnmapViewOfSection, _CONFIGURATION_CLASS_ProcessorClass,
    _CONFIGURATION_TYPE_CentralProcessor, _CONFIGURATION_TYPE_FloatingPointProcessor,
    _INTERFACE_TYPE_InterfaceTypeUndefined, _SECTION_INHERIT_ViewUnmap, ANSI_STRING,
    CONFIGURATION_COMPONENT_DATA, CPU_INFO, HANDLE, INTERFACE_TYPE, KAFFINITY, LARGE_INTEGER,
    PCM_FULL_RESOURCE_DESCRIPTOR, PKPRCB, PLOADER_PARAMETER_BLOCK, _OBJECT_ATTRIBUTES,
    _UNICODE_STRING, PCPU_INFO,
};

// KiCpuId is defined as stdcall (`_KiCpuId@8`) in the i386 kernel — bindgen has it as
// extern "C" which would call cdecl `_KiCpuId`. Override with extern "system" so the
// stdcall decoration matches what the asm definition exports.
extern "system" {
    fn KiCpuId(CpuInfo: PCPU_INFO, Function: ULONG);
}

macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

const ANSI_NULL: CHAR = 0;
const UNICODE_NULL: WCHAR = 0;
const PF_PAE_ENABLED: usize = 9;
const MEM_DOS_LIM: ULONG = 0x40000000;
// C: cmlib.h `#define TAG_CM '  MC'`. A multi-char literal is BIG-endian.
const TAG_CM: ULONG = u32::from_be_bytes(*b"  MC");
const CM_MAXIMUM_TYPE_COUNT: usize = 42;
const REG_OPTION_NON_VOLATILE: ULONG = 0;
const REG_CREATED_NEW_KEY: ULONG = 1;
const KEY_WRITE: ULONG = STANDARD_RIGHTS_WRITE | KEY_SET_VALUE | KEY_CREATE_SUB_KEY;
// cmtypes.h masks SYNCHRONIZE back out of STANDARD_RIGHTS_ALL (0xF003F).
const KEY_ALL_ACCESS: ULONG = (STANDARD_RIGHTS_ALL
    | KEY_QUERY_VALUE
    | KEY_SET_VALUE
    | KEY_CREATE_SUB_KEY
    | KEY_ENUMERATE_SUB_KEYS
    | KEY_NOTIFY
    | KEY_CREATE_LINK)
    & !SYNCHRONIZE;

const ProcessorClass: u32 = _CONFIGURATION_CLASS_ProcessorClass;
const CentralProcessor: u32 = _CONFIGURATION_TYPE_CentralProcessor;
const FloatingPointProcessor: u32 = _CONFIGURATION_TYPE_FloatingPointProcessor;
const InterfaceTypeUndefined: INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined;
const ViewUnmap: ntkrnl_sys::ddk::SECTION_INHERIT = _SECTION_INHERIT_ViewUnmap;

#[cfg(target_arch = "x86_64")]
const CPU_VIA: UCHAR = 5;
#[cfg(target_arch = "x86_64")]
const CPU_AMD: UCHAR = 2;

static CmpFullCpuID: &[u8] = b"%s Family %u Model %u Stepping %u\0";
static CmpBiosStrings: [&[u8]; 23] = [
    b"Ver", b"Rev", b"Rel", b"v0", b"v1", b"v2", b"v3", b"v4", b"v5", b"v6", b"v7", b"v8", b"v9",
    b"v 0", b"v 1", b"v 2", b"v 3", b"v 4", b"v 5", b"v 6", b"v 7", b"v 8", b"v 9",
];

pub static mut CmpBiosBegin: PCHAR = core::ptr::null_mut();
pub static mut CmpBiosSearchStart: PCHAR = core::ptr::null_mut();
pub static mut CmpBiosSearchEnd: PCHAR = core::ptr::null_mut();

extern "C" {
    static mut CmpConfigurationData: PCM_FULL_RESOURCE_DESCRIPTOR;
    static CmpConfigurationAreaSize: ULONG;
    static mut SharedUserData: *mut u8;

    static mut KeRosProcessorName: _UNICODE_STRING;
    static mut KeRosBiosDate: _UNICODE_STRING;
    static mut KeRosBiosVersion: _UNICODE_STRING;
    static mut KeRosVideoBiosDate: _UNICODE_STRING;
    static mut KeRosVideoBiosVersion: _UNICODE_STRING;
}

#[inline(always)]
unsafe fn InitializeObjectAttributes(
    ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
    ObjectName: *mut _UNICODE_STRING,
    Attributes: ULONG,
    RootDirectory: HANDLE,
    SecurityDescriptor: PVOID,
) {
    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = RootDirectory;
    (*ObjectAttributes).Attributes = Attributes;
    (*ObjectAttributes).ObjectName = ObjectName;
    (*ObjectAttributes).SecurityDescriptor = SecurityDescriptor;
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();
}

#[inline(always)]
fn AFFINITY_MASK(i: ULONG) -> ULONG {
    (1u64 << i) as ULONG
}

#[inline(always)]
fn isdigit(ch: CHAR) -> bool {
    ch >= b'0' as CHAR && ch <= b'9' as CHAR
}

#[inline(always)]
unsafe fn RtlMoveMemory(Destination: PVOID, Source: PCVOID, Length: SIZE_T) {
    core::ptr::copy(Source as *const u8, Destination as *mut u8, Length);
}

#[inline(always)]
unsafe fn RtlEqualMemory(Source1: PCVOID, Source2: PCVOID, Length: SIZE_T) -> BOOLEAN {
    if RtlCompareMemory(Source1 as PVOID, Source2 as PVOID, Length as _) == Length as _ {
        TRUE
    } else {
        FALSE
    }
}

unsafe fn c_strlen(String: *const CHAR) -> usize {
    let mut p = String;
    while *p != ANSI_NULL {
        p = p.add(1);
    }
    p as usize - String as usize
}

unsafe fn c_strtoul_10(String: *const CHAR) -> ULONG {
    let mut p = String;
    let mut value: ULONG = 0;
    while isdigit(*p) {
        value = value
            .wrapping_mul(10)
            .wrapping_add((*p as u8 - b'0') as ULONG);
        p = p.add(1);
    }
    value
}

unsafe fn c_memcmp(Left: *const CHAR, Right: *const CHAR, Length: usize) -> i32 {
    let mut i = 0;
    while i < Length {
        let l = *Left.add(i) as u8;
        let r = *Right.add(i) as u8;
        if l != r {
            return l as i32 - r as i32;
        }
        i += 1;
    }
    0
}

unsafe fn c_strstr_pattern(Haystack: *const CHAR, Needle: &[u8]) -> *const CHAR {
    let mut p = Haystack;
    while *p != ANSI_NULL {
        let mut i = 0;
        while i < Needle.len() && *p.add(i) != ANSI_NULL && *p.add(i) as u8 == Needle[i] {
            i += 1;
        }
        if i == Needle.len() {
            return p;
        }
        p = p.add(1);
    }
    core::ptr::null()
}

unsafe fn c_strcpy(Destination: *mut CHAR, Source: *const u8) {
    let mut i = 0;
    loop {
        let ch = *Source.add(i) as CHAR;
        *Destination.add(i) = ch;
        if ch == ANSI_NULL {
            break;
        }
        i += 1;
    }
}

#[inline(always)]
unsafe fn bios_write_char(Destination: PCHAR, Value: CHAR) {
    core::ptr::write_volatile(Destination, Value);
}

#[cfg(target_arch = "x86")]
unsafe fn build_processor_identifier(Buffer: *mut CHAR, Prcb: PKPRCB) {
    if (*Prcb).CpuID == 0 {
        crate::rtl_string_cb_printf_a!(
            Buffer,
            128,
            b"80%u86-%c%x",
            (*Prcb).CpuType as ULONG,
            (((*Prcb).CpuStep >> 8) as ULONG + b'A' as ULONG) as i32,
            ((*Prcb).CpuStep & 0xff) as ULONG,
        );
    } else {
        crate::rtl_string_cb_printf_a!(
            Buffer,
            128,
            CmpFullCpuID,
            b"x86\0".as_ptr() as *const i8,
            (*Prcb).CpuType as ULONG,
            ((*Prcb).CpuStep >> 8) as ULONG,
            ((*Prcb).CpuStep & 0xff) as ULONG,
        );
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn build_processor_identifier(Buffer: *mut CHAR, Prcb: PKPRCB) {
    let FamilyId: *const i8;

    if (*Prcb).CpuVendor == CPU_VIA {
        FamilyId = b"VIA64\0".as_ptr() as *const i8;
    } else if (*Prcb).CpuVendor == CPU_AMD {
        FamilyId = b"AMD64\0".as_ptr() as *const i8;
    } else {
        FamilyId = b"EM64T\0".as_ptr() as *const i8;
    }

    crate::rtl_string_cb_printf_a!(
        Buffer,
        128,
        CmpFullCpuID,
        FamilyId,
        (*Prcb).CpuType as ULONG,
        ((*Prcb).CpuStep >> 8) as ULONG,
        ((*Prcb).CpuStep & 0xff) as ULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CmpGetBiosDate(
    BiosStart: PCHAR,
    BiosLength: ULONG,
    BiosDate: PCHAR,
    FromBios: BOOLEAN,
) -> BOOLEAN {
    let BiosStart = opaque_ptr(BiosStart);
    let BiosDate = opaque_ptr(BiosDate);
    let mut LastDate: [CHAR; 11] = [0; 11];
    let mut CurrentDate: [CHAR; 11] = [0; 11];
    let mut p: PCHAR;
    let pp: PCHAR;

    p = BiosStart.add(2);
    pp = BiosStart.add(BiosLength as usize).sub(5);
    while (p as usize) < (pp as usize) {
        if (*p == b'/' as CHAR)
            && (*p.add(3) == b'/' as CHAR)
            && isdigit(*p.sub(1))
            && isdigit(*p.add(1))
            && isdigit(*p.add(2))
            && isdigit(*p.add(4))
            && isdigit(*p.add(5))
        {
            RtlMoveMemory(
                CurrentDate.as_mut_ptr().add(5) as PVOID,
                p.sub(2) as PCVOID,
                5,
            );

            if !isdigit(CurrentDate[5]) {
                CurrentDate[5] = b'0' as CHAR;
            }

            CurrentDate[2] = *p.add(4);
            CurrentDate[3] = *p.add(5);
            CurrentDate[4] = ANSI_NULL;
            CurrentDate[7] = ANSI_NULL;
            CurrentDate[10] = ANSI_NULL;

            if (FromBios != FALSE)
                && isdigit(*p.add(6))
                && isdigit(*p.add(7))
                && ((RtlEqualMemory(p.add(4) as PCVOID, b"19".as_ptr() as PCVOID, 2) != FALSE)
                    || (RtlEqualMemory(p.add(4) as PCVOID, b"20".as_ptr() as PCVOID, 2) != FALSE))
            {
                CurrentDate[0] = *p.add(4);
                CurrentDate[1] = *p.add(5);
                CurrentDate[2] = *p.add(6);
                CurrentDate[3] = *p.add(7);
            } else {
                if c_strtoul_10(CurrentDate.as_ptr().add(2)) < 80 {
                    CurrentDate[0] = b'2' as CHAR;
                    CurrentDate[1] = b'0' as CHAR;
                } else {
                    CurrentDate[0] = b'1' as CHAR;
                    CurrentDate[1] = b'9' as CHAR;
                }
            }

            CurrentDate[4] = b'/' as CHAR;
            CurrentDate[7] = b'/' as CHAR;

            if c_memcmp(LastDate.as_ptr(), CurrentDate.as_ptr(), 10) < 0 {
                RtlMoveMemory(
                    LastDate.as_mut_ptr() as PVOID,
                    CurrentDate.as_ptr() as PCVOID,
                    10,
                );
            }

            p = p.add(2);
        }

        p = p.add(1);
    }

    if LastDate[0] != ANSI_NULL {
        RtlMoveMemory(BiosDate as PVOID, LastDate.as_ptr().add(5) as PCVOID, 5);
        bios_write_char(BiosDate.add(5), b'/' as CHAR);
        bios_write_char(BiosDate.add(6), LastDate[2]);
        bios_write_char(BiosDate.add(7), LastDate[3]);
        bios_write_char(BiosDate.add(8), ANSI_NULL);
        return TRUE;
    }

    bios_write_char(BiosDate, ANSI_NULL);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn CmpGetBiosVersion(
    BiosStart: PCHAR,
    BiosLength: ULONG,
    BiosVersion: PCHAR,
) -> BOOLEAN {
    let BiosStart = opaque_ptr(BiosStart);
    let BiosVersion = opaque_ptr(BiosVersion);
    let mut Buffer: [CHAR; 128] = [0; 128];
    let mut p: PCHAR = core::ptr::null_mut();
    let mut pp: PCHAR = core::ptr::null_mut();
    let mut i: USHORT;

    if !BiosStart.is_null() {
        CmpBiosBegin = BiosStart;
        CmpBiosSearchStart = BiosStart.add(1);
        CmpBiosSearchEnd = BiosStart.add(BiosLength as usize).sub(2);
    }

    loop {
        pp = core::ptr::null_mut();
        while (CmpBiosSearchStart as usize) <= (CmpBiosSearchEnd as usize) {
            if (*CmpBiosSearchStart == b'.' as CHAR)
                && (*CmpBiosSearchStart.add(1) >= b'0' as CHAR)
                && (*CmpBiosSearchStart.add(1) <= b'9' as CHAR)
                && (*CmpBiosSearchStart.sub(1) >= b'0' as CHAR)
                && (*CmpBiosSearchStart.sub(1) <= b'9' as CHAR)
            {
                pp = CmpBiosSearchStart;
                break;
            } else {
                CmpBiosSearchStart = CmpBiosSearchStart.add(1);
            }
        }

        if (CmpBiosSearchStart as usize) > (CmpBiosSearchEnd as usize) {
            return FALSE;
        }

        CmpBiosSearchStart = CmpBiosSearchStart.add(2);

        Buffer[127] = ANSI_NULL;
        p = Buffer.as_mut_ptr().add(127);

        pp = pp.sub(1);

        i = 0;
        while (i < 127)
            && ((pp as usize) >= (CmpBiosBegin as usize))
            && (*pp >= b' ' as CHAR)
            && (*pp != b'$' as CHAR)
        {
            i = i.wrapping_add(1);
            p = p.sub(1);
            *p = *pp;
            pp = pp.sub(1);
        }

        pp = pp.add(1);

        i = 0;
        while (i as usize) < CmpBiosStrings.len() {
            if !c_strstr_pattern(p, CmpBiosStrings[i as usize]).is_null() {
                break;
            }
            i = i.wrapping_add(1);
        }

        if (i as usize) < CmpBiosStrings.len() {
            break;
        }
    }

    while *pp == b' ' as CHAR {
        pp = pp.add(1);
    }

    i = 0;
    loop {
        bios_write_char(BiosVersion.add(i as usize), *pp);
        pp = pp.add(1);
        i = i.wrapping_add(1);

        if !((i < 127)
            && ((pp as usize) <= (CmpBiosSearchEnd.add(1) as usize))
            && (*pp >= b' ' as CHAR)
            && (*pp != b'$' as CHAR))
        {
            break;
        }
    }

    bios_write_char(BiosVersion.add(i as usize), ANSI_NULL);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitializeMachineDependentConfiguration(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let _LoaderBlock = opaque_ptr(LoaderBlock);
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut Data: _UNICODE_STRING = core::mem::zeroed();
    let mut SectionName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut HavePae: ULONG;
    let mut Length: ULONG;
    let mut TotalLength: ULONG = 0;
    let mut i: ULONG;
    let mut Disposition: ULONG = 0;
    let mut ViewSize: ntkrnl_sys::ddk::SIZE_T;
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut BiosHandle: HANDLE = core::ptr::null_mut();
    let mut SystemHandle: HANDLE = core::ptr::null_mut();
    let mut FpuHandle: HANDLE = core::ptr::null_mut();
    let mut SectionHandle: HANDLE = core::ptr::null_mut();
    let mut ConfigData: CONFIGURATION_COMPONENT_DATA = core::mem::zeroed();
    let mut Buffer: [CHAR; 128] = [0; 128];
    let mut CpuInfo: CPU_INFO = core::mem::zeroed();
    let mut ExtendedId: ULONG;
    let mut Prcb: PKPRCB;
    let mut IndexTable: [USHORT; CM_MAXIMUM_TYPE_COUNT] = [0; CM_MAXIMUM_TYPE_COUNT];
    let mut TempString: ANSI_STRING = core::mem::zeroed();
    let mut PartialString: PCHAR = core::ptr::null_mut();
    let mut BiosVersion: PCHAR;
    let mut CpuString: [CHAR; 48] = [0; 48];
    let mut BaseAddress: PVOID = core::ptr::null_mut();
    let mut ViewBase: LARGE_INTEGER = core::mem::zeroed();
    let mut VideoRomBase: ULONG_PTR;
    let mut CurrentVersion: PCHAR;

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        w!("\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management")
            .as_ptr(),
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(
        core::ptr::addr_of_mut!(KeyHandle),
        KEY_READ | KEY_WRITE,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if NT_SUCCESS(Status) {
        HavePae = *SharedUserData.add(0x274 + PF_PAE_ENABLED) as ULONG;

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(ValueName),
            w!("PhysicalAddressExtension").as_ptr(),
        );
        NtSetValueKey(
            KeyHandle,
            core::ptr::addr_of_mut!(ValueName),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(HavePae) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );

        NtClose(KeyHandle);
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        w!("\\Registry\\Machine\\Hardware\\Description\\System").as_ptr(),
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(
        core::ptr::addr_of_mut!(SystemHandle),
        KEY_READ | KEY_WRITE,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        w!("\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\BIOSINFO").as_ptr(),
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtCreateKey(
        core::ptr::addr_of_mut!(BiosHandle),
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::addr_of_mut!(Disposition),
    );
    if !NT_SUCCESS(Status) {
        NtClose(SystemHandle);
        return Status;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        w!("CentralProcessor").as_ptr(),
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SystemHandle,
        core::ptr::null_mut(),
    );
    Status = NtCreateKey(
        core::ptr::addr_of_mut!(KeyHandle),
        KEY_READ | KEY_WRITE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(Disposition),
    );
    NtClose(KeyHandle);

    if Disposition == REG_CREATED_NEW_KEY {
        CmpConfigurationData =
            ExAllocatePoolWithTag(PagedPool, CmpConfigurationAreaSize as SIZE_T, TAG_CM)
                as PCM_FULL_RESOURCE_DESCRIPTOR;
        if CmpConfigurationData.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        i = 0;
        while i < KeNumberProcessors as ULONG {
            Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add(i as usize);

            RtlZeroMemory(
                core::ptr::addr_of_mut!(ConfigData) as PVOID,
                core::mem::size_of::<CONFIGURATION_COMPONENT_DATA>(),
            );
            ConfigData.ComponentEntry.Class = ProcessorClass;
            ConfigData.ComponentEntry.Type = CentralProcessor;
            ConfigData.ComponentEntry.Key = i;
            ConfigData.ComponentEntry.AffinityMask = AFFINITY_MASK(i);
            ConfigData.ComponentEntry.Identifier = Buffer.as_mut_ptr();

            build_processor_identifier(Buffer.as_mut_ptr(), Prcb);

            ConfigData.ComponentEntry.IdentifierLength = c_strlen(Buffer.as_ptr()) as ULONG + 1;

            Status = CmpInitializeRegistryNode(
                core::ptr::addr_of_mut!(ConfigData),
                SystemHandle,
                core::ptr::addr_of_mut!(KeyHandle),
                InterfaceTypeUndefined,
                0xFFFFFFFF,
                IndexTable.as_mut_ptr(),
            );
            if !NT_SUCCESS(Status) {
                NtClose(BiosHandle);
                NtClose(SystemHandle);
                return Status;
            }

            if KeI386NpxPresent != 0 {
                RtlZeroMemory(
                    core::ptr::addr_of_mut!(ConfigData) as PVOID,
                    core::mem::size_of::<CONFIGURATION_COMPONENT_DATA>(),
                );
                ConfigData.ComponentEntry.Class = ProcessorClass;
                ConfigData.ComponentEntry.Type = FloatingPointProcessor;
                ConfigData.ComponentEntry.Key = i;
                ConfigData.ComponentEntry.AffinityMask = AFFINITY_MASK(i);
                ConfigData.ComponentEntry.Identifier = Buffer.as_mut_ptr();

                if (*Prcb).CpuType == 3 {
                    c_strcpy(Buffer.as_mut_ptr(), b"80387\0".as_ptr());
                }

                ConfigData.ComponentEntry.IdentifierLength = c_strlen(Buffer.as_ptr()) as ULONG + 1;

                Status = CmpInitializeRegistryNode(
                    core::ptr::addr_of_mut!(ConfigData),
                    SystemHandle,
                    core::ptr::addr_of_mut!(FpuHandle),
                    InterfaceTypeUndefined,
                    0xFFFFFFFF,
                    IndexTable.as_mut_ptr(),
                );
                if !NT_SUCCESS(Status) {
                    NtClose(KeyHandle);
                    NtClose(BiosHandle);
                    NtClose(SystemHandle);
                    return Status;
                }

                NtClose(FpuHandle);

                KeSetSystemAffinityThread((*Prcb).SetMember as KAFFINITY);
                if (*Prcb).CpuID == 0 {
                    ASSERT!(false);
                } else {
                    KiCpuId(core::ptr::addr_of_mut!(CpuInfo), 0x80000000);
                    ExtendedId = CpuInfo.__bindgen_anon_1.Eax;
                    if ExtendedId >= 0x80000004 {
                        PartialString = CpuString.as_mut_ptr();
                        ExtendedId = 2;
                        while ExtendedId <= 4 {
                            KiCpuId(core::ptr::addr_of_mut!(CpuInfo), 0x80000000 | ExtendedId);
                            *(PartialString as PULONG).add(0) = CpuInfo.__bindgen_anon_1.Eax;
                            *(PartialString as PULONG).add(1) = CpuInfo.__bindgen_anon_1.Ebx;
                            *(PartialString as PULONG).add(2) = CpuInfo.__bindgen_anon_1.Ecx;
                            *(PartialString as PULONG).add(3) = CpuInfo.__bindgen_anon_1.Edx;

                            PartialString = PartialString.add(16);
                            ExtendedId += 1;
                        }

                        CpuString[47] = ANSI_NULL;
                    }
                }

                KeRevertToUserAffinityThread();

                if !PartialString.is_null() {
                    RtlInitAnsiString(core::ptr::addr_of_mut!(TempString), CpuString.as_ptr());
                    if NT_SUCCESS(RtlAnsiStringToUnicodeString(
                        core::ptr::addr_of_mut!(Data),
                        core::ptr::addr_of_mut!(TempString),
                        TRUE,
                    )) {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(ValueName),
                            w!("ProcessorNameString").as_ptr(),
                        );
                        Status = NtSetValueKey(
                            KeyHandle,
                            core::ptr::addr_of_mut!(ValueName),
                            0,
                            REG_SZ,
                            Data.Buffer as PVOID,
                            Data.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG,
                        );

                        if RtlCreateUnicodeString(
                            core::ptr::addr_of_mut!(KeRosProcessorName),
                            Data.Buffer as *const WCHAR,
                        ) == FALSE
                        {
                            KeRosProcessorName.Length = 0;
                        }

                        RtlFreeUnicodeString(core::ptr::addr_of_mut!(Data));
                    }
                }

                if (*Prcb).VendorString[0] != 0 {
                    RtlInitAnsiString(
                        core::ptr::addr_of_mut!(TempString),
                        (*Prcb).VendorString.as_ptr() as *const i8,
                    );
                    if NT_SUCCESS(RtlAnsiStringToUnicodeString(
                        core::ptr::addr_of_mut!(Data),
                        core::ptr::addr_of_mut!(TempString),
                        TRUE,
                    )) {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(ValueName),
                            w!("VendorIdentifier").as_ptr(),
                        );
                        Status = NtSetValueKey(
                            KeyHandle,
                            core::ptr::addr_of_mut!(ValueName),
                            0,
                            REG_SZ,
                            Data.Buffer as PVOID,
                            Data.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG,
                        );

                        RtlFreeUnicodeString(core::ptr::addr_of_mut!(Data));
                    }
                }

                if (*Prcb).FeatureBits != 0 {
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(ValueName),
                        w!("FeatureSet").as_ptr(),
                    );
                    Status = NtSetValueKey(
                        KeyHandle,
                        core::ptr::addr_of_mut!(ValueName),
                        0,
                        REG_DWORD,
                        core::ptr::addr_of!((*Prcb).FeatureBits) as PVOID,
                        core::mem::size_of::<ULONG>() as ULONG,
                    );
                }

                if (*Prcb).MHz != 0 {
                    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), w!("~MHz").as_ptr());
                    Status = NtSetValueKey(
                        KeyHandle,
                        core::ptr::addr_of_mut!(ValueName),
                        0,
                        REG_DWORD,
                        core::ptr::addr_of!((*Prcb).MHz) as PVOID,
                        core::mem::size_of::<ULONG>() as ULONG,
                    );
                }

                if (*Prcb).UpdateSignature.QuadPart != 0 {
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(ValueName),
                        w!("Update Signature").as_ptr(),
                    );
                    Status = NtSetValueKey(
                        KeyHandle,
                        core::ptr::addr_of_mut!(ValueName),
                        0,
                        REG_BINARY,
                        core::ptr::addr_of!((*Prcb).UpdateSignature) as PVOID,
                        core::mem::size_of::<LARGE_INTEGER>() as ULONG,
                    );
                }

                NtClose(KeyHandle);
            }

            i += 1;
        }

        ExFreePoolWithTag(CmpConfigurationData as PVOID, TAG_CM);
    }

    'Quickie: {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(SectionName),
            w!("\\Device\\PhysicalMemory").as_ptr(),
        );
        InitializeObjectAttributes(
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(SectionName),
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        Status = ZwOpenSection(
            core::ptr::addr_of_mut!(SectionHandle),
            SECTION_ALL_ACCESS,
            core::ptr::addr_of_mut!(ObjectAttributes),
        );
        if !NT_SUCCESS(Status) {
            NtClose(BiosHandle);
            NtClose(SystemHandle);
            break 'Quickie;
        }

        ViewSize = PAGE_SIZE as ntkrnl_sys::ddk::SIZE_T;
        Status = ZwMapViewOfSection(
            SectionHandle,
            NtCurrentProcess(),
            core::ptr::addr_of_mut!(BaseAddress),
            0,
            ViewSize,
            core::ptr::addr_of_mut!(ViewBase),
            core::ptr::addr_of_mut!(ViewSize),
            ViewUnmap,
            MEM_DOS_LIM,
            PAGE_READWRITE,
        );
        if !NT_SUCCESS(Status) {
            VideoRomBase = 0xC0000;
        } else {
            VideoRomBase = (*((BaseAddress as PULONG).add(0x10)) >> 12) as ULONG_PTR & 0xFFFF0;
            VideoRomBase += (*((BaseAddress as PULONG).add(0x10)) & 0xFFF0) as ULONG_PTR;

            VideoRomBase &= 0xFFFF8000;
            if VideoRomBase < 0xC0000 {
                VideoRomBase = 0xC0000;
            }

            ZwUnmapViewOfSection(NtCurrentProcess(), BaseAddress);
        }

        BiosVersion = ExAllocatePoolWithTag(PagedPool, PAGE_SIZE, TAG_CM) as PCHAR;

        BaseAddress = core::ptr::null_mut();
        ViewSize = (16 * PAGE_SIZE) as ntkrnl_sys::ddk::SIZE_T;
        ViewBase.__bindgen_anon_1.LowPart = 0xF0000;
        ViewBase.__bindgen_anon_1.HighPart = 0;

        Status = ZwMapViewOfSection(
            SectionHandle,
            NtCurrentProcess(),
            core::ptr::addr_of_mut!(BaseAddress),
            0,
            ViewSize,
            core::ptr::addr_of_mut!(ViewBase),
            core::ptr::addr_of_mut!(ViewSize),
            ViewUnmap,
            MEM_DOS_LIM,
            PAGE_READWRITE,
        );
        if NT_SUCCESS(Status) {
            if CmpGetBiosDate(
                BaseAddress as PCHAR,
                (16 * PAGE_SIZE) as ULONG,
                Buffer.as_mut_ptr(),
                TRUE,
            ) != FALSE
            {
                RtlInitAnsiString(core::ptr::addr_of_mut!(TempString), Buffer.as_ptr());
                if NT_SUCCESS(RtlAnsiStringToUnicodeString(
                    core::ptr::addr_of_mut!(Data),
                    core::ptr::addr_of_mut!(TempString),
                    TRUE,
                )) {
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(ValueName),
                        w!("SystemBiosDate").as_ptr(),
                    );
                    Status = NtSetValueKey(
                        SystemHandle,
                        core::ptr::addr_of_mut!(ValueName),
                        0,
                        REG_SZ,
                        Data.Buffer as PVOID,
                        Data.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG,
                    );

                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(Data));
                }

                if !BiosHandle.is_null() {
                    RtlCopyMemory(
                        Buffer.as_mut_ptr() as PVOID,
                        (BaseAddress as PCHAR).add(16 * PAGE_SIZE - 11) as PCVOID,
                        8,
                    );
                    Buffer[8] = ANSI_NULL;

                    RtlInitAnsiString(core::ptr::addr_of_mut!(TempString), Buffer.as_ptr());
                    Status = RtlAnsiStringToUnicodeString(
                        core::ptr::addr_of_mut!(Data),
                        core::ptr::addr_of_mut!(TempString),
                        TRUE,
                    );
                    if NT_SUCCESS(Status) {
                        Status = NtSetValueKey(
                            BiosHandle,
                            core::ptr::addr_of_mut!(ValueName),
                            0,
                            REG_SZ,
                            Data.Buffer as PVOID,
                            Data.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG,
                        );

                        if RtlCreateUnicodeString(
                            core::ptr::addr_of_mut!(KeRosBiosDate),
                            Data.Buffer as *const WCHAR,
                        ) == FALSE
                        {
                            KeRosBiosDate.Length = 0;
                        }

                        RtlFreeUnicodeString(core::ptr::addr_of_mut!(Data));
                    }

                    NtClose(BiosHandle);
                }
            }

            if CmpGetBiosVersion(
                BaseAddress as PCHAR,
                (16 * PAGE_SIZE) as ULONG,
                Buffer.as_mut_ptr(),
            ) != FALSE
            {
                CurrentVersion = BiosVersion;
                loop {
                    RtlInitAnsiString(core::ptr::addr_of_mut!(TempString), Buffer.as_ptr());
                    Status = RtlAnsiStringToUnicodeString(
                        core::ptr::addr_of_mut!(Data),
                        core::ptr::addr_of_mut!(TempString),
                        TRUE,
                    );
                    if !NT_SUCCESS(Status) {
                        break;
                    }

                    Length = Data.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG;
                    RtlMoveMemory(
                        CurrentVersion as PVOID,
                        Data.Buffer as PCVOID,
                        Length as SIZE_T,
                    );

                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(Data));

                    TotalLength += Length;
                    if TotalLength + 256 + core::mem::size_of::<WCHAR>() as ULONG
                        > PAGE_SIZE as ULONG
                    {
                        break;
                    }

                    CurrentVersion = CurrentVersion.add(Length as usize);

                    if CmpGetBiosVersion(core::ptr::null_mut(), 0, Buffer.as_mut_ptr()) == FALSE {
                        break;
                    }
                }

                if TotalLength != 0 {
                    *(CurrentVersion as *mut WCHAR) = UNICODE_NULL;
                    TotalLength += core::mem::size_of::<WCHAR>() as ULONG;

                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(ValueName),
                        w!("SystemBiosVersion").as_ptr(),
                    );
                    Status = NtSetValueKey(
                        SystemHandle,
                        core::ptr::addr_of_mut!(ValueName),
                        0,
                        REG_MULTI_SZ,
                        BiosVersion as PVOID,
                        TotalLength,
                    );

                    if RtlCreateUnicodeString(
                        core::ptr::addr_of_mut!(KeRosBiosVersion),
                        BiosVersion as *const WCHAR,
                    ) == FALSE
                    {
                        KeRosBiosVersion.Length = 0;
                    }
                }
            }

            ZwUnmapViewOfSection(NtCurrentProcess(), BaseAddress);
        }

        BaseAddress = core::ptr::null_mut();
        ViewSize = (8 * PAGE_SIZE) as ntkrnl_sys::ddk::SIZE_T;
        ViewBase.QuadPart = VideoRomBase as i64;

        Status = ZwMapViewOfSection(
            SectionHandle,
            NtCurrentProcess(),
            core::ptr::addr_of_mut!(BaseAddress),
            0,
            ViewSize,
            core::ptr::addr_of_mut!(ViewBase),
            core::ptr::addr_of_mut!(ViewSize),
            ViewUnmap,
            MEM_DOS_LIM,
            PAGE_READWRITE,
        );
        if NT_SUCCESS(Status) {
            if CmpGetBiosDate(
                BaseAddress as PCHAR,
                (8 * PAGE_SIZE) as ULONG,
                Buffer.as_mut_ptr(),
                FALSE,
            ) != FALSE
            {
                RtlInitAnsiString(core::ptr::addr_of_mut!(TempString), Buffer.as_ptr());
                if NT_SUCCESS(RtlAnsiStringToUnicodeString(
                    core::ptr::addr_of_mut!(Data),
                    core::ptr::addr_of_mut!(TempString),
                    TRUE,
                )) {
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(ValueName),
                        w!("VideoBiosDate").as_ptr(),
                    );
                    Status = NtSetValueKey(
                        SystemHandle,
                        core::ptr::addr_of_mut!(ValueName),
                        0,
                        REG_SZ,
                        Data.Buffer as PVOID,
                        Data.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG,
                    );

                    if RtlCreateUnicodeString(
                        core::ptr::addr_of_mut!(KeRosVideoBiosDate),
                        Data.Buffer as *const WCHAR,
                    ) == FALSE
                    {
                        KeRosVideoBiosDate.Length = 0;
                    }

                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(Data));
                }
            }

            if CmpGetBiosVersion(
                BaseAddress as PCHAR,
                (8 * PAGE_SIZE) as ULONG,
                Buffer.as_mut_ptr(),
            ) != FALSE
            {
                CurrentVersion = BiosVersion;
                loop {
                    RtlInitAnsiString(core::ptr::addr_of_mut!(TempString), Buffer.as_ptr());
                    if !NT_SUCCESS(RtlAnsiStringToUnicodeString(
                        core::ptr::addr_of_mut!(Data),
                        core::ptr::addr_of_mut!(TempString),
                        TRUE,
                    )) {
                        break;
                    }

                    Length = Data.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG;
                    RtlMoveMemory(
                        CurrentVersion as PVOID,
                        Data.Buffer as PCVOID,
                        Length as SIZE_T,
                    );

                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(Data));

                    TotalLength += Length;
                    if TotalLength + 256 + core::mem::size_of::<WCHAR>() as ULONG
                        > PAGE_SIZE as ULONG
                    {
                        break;
                    }

                    CurrentVersion = CurrentVersion.add(Length as usize);

                    if CmpGetBiosVersion(core::ptr::null_mut(), 0, Buffer.as_mut_ptr()) == FALSE {
                        break;
                    }
                }

                if TotalLength != 0 {
                    *(CurrentVersion as *mut WCHAR) = UNICODE_NULL;
                    TotalLength += core::mem::size_of::<WCHAR>() as ULONG;

                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(ValueName),
                        w!("VideoBiosVersion").as_ptr(),
                    );
                    Status = NtSetValueKey(
                        SystemHandle,
                        core::ptr::addr_of_mut!(ValueName),
                        0,
                        REG_MULTI_SZ,
                        BiosVersion as PVOID,
                        TotalLength,
                    );

                    if RtlCreateUnicodeString(
                        core::ptr::addr_of_mut!(KeRosVideoBiosVersion),
                        BiosVersion as *const WCHAR,
                    ) == FALSE
                    {
                        KeRosVideoBiosVersion.Length = 0;
                    }
                }
            }

            ZwUnmapViewOfSection(NtCurrentProcess(), BaseAddress);
        }

        ZwClose(SectionHandle);

        if !BiosVersion.is_null() {
            ExFreePoolWithTag(BiosVersion as PVOID, TAG_CM);
        }
    }

    NtClose(KeyHandle);
    STATUS_SUCCESS
}
