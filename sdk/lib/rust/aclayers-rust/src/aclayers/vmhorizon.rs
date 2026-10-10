use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_2833F7F0D6B8: [u16; 4] = [0x006Du16, 0x0073, 0x0069, 0x0000];
pub static RUSTCONV_C2R_WSTR_DE31C7E52129: [u16; 5] = [0x002Eu16, 0x0074, 0x006D, 0x0070, 0x0000];

static mut DllPrefix: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 4]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 4]>() as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_2833F7F0D6B8.as_ptr() as PWSTR,
    };

static mut DllPostfix: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 5]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 5]>() as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_DE31C7E52129.as_ptr() as PWSTR,
    };

// WIN32_NO_STATUS: feature-guard flag with no value, nothing to emit.

// FixupDll's function-local `#define OFFSET_n` (the C defines them inside the function body;
// they are the file offsets of the three patch sites, see the comment table in FixupDll).
const OFFSET_1: usize = 0x21A6E;
const OFFSET_2: usize = 0x21B04;
const OFFSET_3: usize = 0x21C3C;

// SHIM_NS VMHorizonSetup: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(VMHorizonSetup);

unsafe extern "C" fn Write(Address: PBYTE, Data: PBYTE, Size: SIZE_T) -> BOOL {
    let mut BaseAddress: PVOID = Address as PVOID;
    let mut RegionSize: SIZE_T = Size;
    let mut OldProtection: ULONG = 0;
    let mut Status: NTSTATUS = NtProtectVirtualMemory(
        NtCurrentProcess(),
        core::ptr::addr_of_mut!(BaseAddress),
        core::ptr::addr_of_mut!(RegionSize),
        PAGE_EXECUTE_READWRITE as ULONG,
        core::ptr::addr_of_mut!(OldProtection),
    );
    if NT_SUCCESS!(Status) {
        let mut Bytes: SIZE_T = 0;
        Status = NtWriteVirtualMemory(
            NtCurrentProcess(),
            Address as PVOID,
            Data as PVOID,
            Size,
            core::ptr::addr_of_mut!(Bytes),
        );
        if NT_SUCCESS!(Status) && Bytes != Size {
            Status = STATUS_MEMORY_NOT_ALLOCATED;
        }
        NtProtectVirtualMemory(
            NtCurrentProcess(),
            core::ptr::addr_of_mut!(BaseAddress),
            core::ptr::addr_of_mut!(RegionSize),
            OldProtection,
            core::ptr::addr_of_mut!(OldProtection),
        );
    }
    return NT_SUCCESS!(Status) as BOOL;
}

unsafe extern "C" fn FixupDll(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    static Match1: [UCHAR; 5] = [0x0C, 0x8B, 0xFC, 0xF3, 0xA5];
    static Match2: [UCHAR; 5] = [0x0C, 0x8B, 0xFC, 0xF3, 0xA5];
    static Match3: [UCHAR; 5] = [0xB0, 0x8B, 0xFC, 0xF3, 0xA5];
    let mut Replacement1: [UCHAR; 5] = [0x10, 0x89, 0x34, 0x24, 0x90];
    let mut Replacement2: [UCHAR; 5] = [0x10, 0x89, 0x34, 0x24, 0x90];
    let mut Replacement3: [UCHAR; 5] = [0xB4, 0x89, 0x34, 0x24, 0x90];
    let mut Buffer: [UCHAR; 5] = [0; 5];
    let mut Base: PBYTE = (*LdrEntry).DllBase as PBYTE;
    let mut Bytes: SIZE_T = 0;

    loop {
        'body: {
            DbgPrint(
                b"Module %wZ Loaded at 0x%p, we should patch!\n\0".as_ptr() as PCSTR,
                &raw mut (*LdrEntry).BaseDllName,
                (*LdrEntry).DllBase,
            );
            if !NT_SUCCESS!(NtReadVirtualMemory(
                NtCurrentProcess(),
                Base.offset(OFFSET_1 as isize) as PVOID,
                (&raw mut Buffer) as *mut UCHAR as PVOID,
                5 as SIZE_T,
                &raw mut Bytes,
            )) || Bytes != 5 as SIZE_T {
                break 'body;
            }
            if memcmp(
                (&raw mut Buffer) as *mut UCHAR as *const core::ffi::c_void,
                (&raw const Match1) as *const UCHAR as *const core::ffi::c_void,
                core::mem::size_of::<[UCHAR; 5]>() as usize,
            ) != 0 {
                break 'body;
            }

            if !NT_SUCCESS!(NtReadVirtualMemory(
                NtCurrentProcess(),
                Base.offset(OFFSET_2 as isize) as PVOID,
                (&raw mut Buffer) as *mut UCHAR as PVOID,
                5 as SIZE_T,
                &raw mut Bytes,
            )) || Bytes != 5 as SIZE_T {
                break 'body;
            }
            if memcmp(
                (&raw mut Buffer) as *mut UCHAR as *const core::ffi::c_void,
                (&raw const Match2) as *const UCHAR as *const core::ffi::c_void,
                core::mem::size_of::<[UCHAR; 5]>() as usize,
            ) != 0 {
                break 'body;
            }

            if !NT_SUCCESS!(NtReadVirtualMemory(
                NtCurrentProcess(),
                Base.offset(OFFSET_3 as isize) as PVOID,
                (&raw mut Buffer) as *mut UCHAR as PVOID,
                5 as SIZE_T,
                &raw mut Bytes,
            )) || Bytes != 5 as SIZE_T {
                break 'body;
            }
            if memcmp(
                (&raw mut Buffer) as *mut UCHAR as *const core::ffi::c_void,
                (&raw const Match3) as *const UCHAR as *const core::ffi::c_void,
                core::mem::size_of::<[UCHAR; 5]>() as usize,
            ) != 0 {
                break 'body;
            }

            DbgPrint(
                b"Module %wZ Loaded at 0x%p, OK to patch!\n\0".as_ptr() as PCSTR,
                &raw mut (*LdrEntry).BaseDllName,
                (*LdrEntry).DllBase,
            );
            if Write(
                Base.offset(OFFSET_1 as isize),
                (&raw mut Replacement1) as PBYTE,
                core::mem::size_of::<[UCHAR; 5]>() as SIZE_T,
            ) == 0 {
                break 'body;
            }
            if Write(
                Base.offset(OFFSET_2 as isize),
                (&raw mut Replacement2) as PBYTE,
                core::mem::size_of::<[UCHAR; 5]>() as SIZE_T,
            ) == 0 {
                break 'body;
            }
            if Write(
                Base.offset(OFFSET_3 as isize),
                (&raw mut Replacement3) as PBYTE,
                core::mem::size_of::<[UCHAR; 5]>() as SIZE_T,
            ) == 0 {
                break 'body;
            }

            NtFlushInstructionCache(
                NtCurrentProcess(),
                Base as PVOID,
                0x22000 as SIZE_T,
            );

            DbgPrint(
                b"Module %wZ Loaded at 0x%p, patched!\n\0".as_ptr() as PCSTR,
                &raw mut (*LdrEntry).BaseDllName,
                (*LdrEntry).DllBase,
            );
        }
        if !(0 != 0) {
            break;
        }
    }
}

unsafe extern "C" fn PostfixUnicodeString(
    mut String1: *const _UNICODE_STRING,
    mut String2: *const _UNICODE_STRING,
) -> BOOLEAN {
    let mut pc1: PWCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut pc2: PWCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut NumChars: ULONG = 0;
    if ((*String2).Length as ::core::ffi::c_int) < (*String1).Length as ::core::ffi::c_int {
        return FALSE as BOOLEAN;
    }
    if (*String1).Buffer.is_null() || (*String2).Buffer.is_null() {
        return FALSE as BOOLEAN;
    }
    NumChars = ((*String1).Length as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
        as ULONG;
    pc1 = (*String1).Buffer as PWCHAR;
    pc2 = (*String2)
        .Buffer
        .offset(
            ((*String2).Length as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                as isize,
        )
        .offset(-(NumChars as isize)) as PWCHAR;
    loop {
        let fresh15 = NumChars;
        NumChars = NumChars.wrapping_sub(1);
        if !(fresh15 != 0) {
            break;
        }
        let fresh16 = pc1;
        pc1 = pc1.offset(1);
        let fresh17 = pc2;
        pc2 = pc2.offset(1);
        if RtlUpcaseUnicodeChar(*fresh16) as ::core::ffi::c_int
            != RtlUpcaseUnicodeChar(*fresh17) as ::core::ffi::c_int
        {
            return FALSE as BOOLEAN;
        }
    }
    return TRUE as BOOLEAN;
}

// SHIM_NUM_HOOKS 0; SHIM_NOTIFY_FN = SHIM_OBJ_NAME(Notify)
pub unsafe extern "system" fn Notify(fdwReason: DWORD, ptr: PVOID) -> BOOL {
    if fdwReason == SHIM_REASON_DLL_LOAD {
        // DllPrefix / DllPostfix: the function-local static const UNICODE_STRINGs, file-scope above.
        let LdrEntry: PLDR_DATA_TABLE_ENTRY = ptr as PLDR_DATA_TABLE_ENTRY;

        let Prefix: BOOLEAN = RtlPrefixUnicodeString(
            core::ptr::addr_of!(DllPrefix) as PCUNICODE_STRING,
            &raw mut (*LdrEntry).BaseDllName,
            TRUE as BOOLEAN,
        );
        let Postfix: BOOLEAN = PostfixUnicodeString(
            core::ptr::addr_of!(DllPostfix),
            &raw const (*LdrEntry).BaseDllName,
        );
        // C: (USHORT - USHORT - USHORT) is an int; `/ sizeof(WCHAR)` converts it to size_t, so a
        // negative difference becomes huge (and fails `<= 2`) -- keep that conversion order.
        let ExtraChars: ULONG = ((((*LdrEntry).BaseDllName.Length as core::ffi::c_int
            - DllPrefix.Length as core::ffi::c_int
            - DllPostfix.Length as core::ffi::c_int) as usize)
            / core::mem::size_of::<WCHAR>()) as ULONG;

        /* msiN[N].tmp */
        if Prefix != 0 && Postfix != 0 && ExtraChars <= 2 {
            let ImageNtHeader: PIMAGE_NT_HEADERS = RtlImageNtHeader((*LdrEntry).DllBase);
            if !ImageNtHeader.is_null() && (*ImageNtHeader).OptionalHeader.CheckSum == 0x176241 {
                SHIM_MSG!(
                    "Module %wZ is a match, applying fixups\n",
                    &raw mut (*LdrEntry).BaseDllName
                );
                FixupDll(LdrEntry);
            }
        }
    }
    return TRUE as BOOL;
}

implement_shim!(VMHorizonSetup, 0usize, Some(Notify), []);
