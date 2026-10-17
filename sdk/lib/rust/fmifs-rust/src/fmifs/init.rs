use crate::fmifs::chkdsk::Chkdsk;
use crate::fmifs::extend::Extend;
use crate::fmifs::format::Format;
use crate::fmifs::format::FormatEx;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8AEFC07D888B: [u16; 62] = [
    0x005Cu16, 0x0052, 0x0045, 0x0047, 0x0049, 0x0053, 0x0054, 0x0052, 0x0059, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x004F,
    0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0052, 0x0065, 0x0061,
    0x0063, 0x0074, 0x004F, 0x0053, 0x005C, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074,
    0x004F, 0x0053, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074,
    0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0049, 0x0046,
    0x0053, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut FmIfsAttached: ULONG = 0 as ULONG;

#[no_mangle]
pub static mut ProviderListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

pub const NTOS_MODE_USER: () = ();

pub const NDEBUG: () = ();
#[no_mangle]
pub unsafe extern "C" fn LoadProvider(mut FileSystem: PWCHAR) -> PIFS_PROVIDER {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut::<LIST_ENTRY>();
    let mut Provider: PIFS_PROVIDER = core::ptr::null_mut::<IFS_PROVIDER>();
    DPRINT!("LoadProvider(%S)\n", FileSystem);
    ListEntry = (*core::ptr::addr_of_mut!(ProviderListHead)).Flink as PLIST_ENTRY;
    while ListEntry != core::ptr::addr_of_mut!(ProviderListHead) as PLIST_ENTRY {
        Provider = (ListEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(IFS_PROVIDER, ListEntry) as ULONG_PTR,
        ) as *mut IFS_PROVIDER as PIFS_PROVIDER;
        if _wcsicmp(
            (*Provider).Name as *const wchar_t,
            FileSystem as *const wchar_t,
        ) == 0
        {
            DPRINT!("Found it!\n");
            if (*Provider).hModule.is_null() {
                (*Provider).hModule = LoadLibraryW((*Provider).DllFile as LPCWSTR) as HMODULE;
                if (*Provider).hModule.is_null() {
                    DPRINT!("Dll Loading failed!\n");
                    return core::ptr::null_mut::<IFS_PROVIDER>();
                }

                /* Get function pointers */
                (*Provider).Chkdsk = core::mem::transmute::<FARPROC, PULIB_CHKDSK>(
                    GetProcAddress(
                        (*Provider).hModule as HINSTANCE,
                        b"Chkdsk\0" as *const u8 as LPCSTR,
                    ),
                );
                //Provider->ChkdskEx = (PULIB_CHKDSKEX)GetProcAddress(Provider->hModule, "ChkdskEx");
                //Provider->Extend = (PULIB_EXTEND)GetProcAddress(Provider->hModule, "Extend");
                (*Provider).Format = core::mem::transmute::<FARPROC, PULIB_FORMAT>(
                    GetProcAddress(
                        (*Provider).hModule as HINSTANCE,
                        b"Format\0" as *const u8 as LPCSTR,
                    ),
                );
                //Provider->FormatEx = (PULIB_FORMATEX)GetProcAddress(Provider->hModule, "FormatEx");
            }
            return Provider;
        }
        ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
    }

    /* Provider not found */
    return core::ptr::null_mut::<IFS_PROVIDER>();
}
#[no_mangle]
pub unsafe extern "C" fn UnloadProvider(Provider: PIFS_PROVIDER) -> BOOLEAN {
    DPRINT!("UnloadProvider(%S)\n", (*Provider).Name);

    if !(*Provider).hModule.is_null() {
        FreeLibrary((*Provider).hModule);
        (*Provider).hModule = core::ptr::null_mut();

        (*Provider).Chkdsk = None;
        (*Provider).ChkdskEx = core::ptr::null_mut();
        (*Provider).Extend = core::ptr::null_mut();
        (*Provider).Format = None;
        (*Provider).FormatEx = core::ptr::null_mut();
    }

    return (TRUE) as BOOLEAN;
}

unsafe extern "C" fn AddProvider(FileSystem: PCUNICODE_STRING, DllFile: PWCHAR) -> BOOLEAN {
    let mut Provider: PIFS_PROVIDER = core::ptr::null_mut();
    let mut ProcCount: ULONG = 0;
    let mut hMod: HMODULE = core::ptr::null_mut();
    let mut ret: BOOLEAN = (FALSE as u8);

    DPRINT!("AddProvider(%wZ %S)\n", FileSystem, DllFile);

    'quit: {
        hMod = LoadLibraryW(DllFile as *const wchar_t) as HMODULE;
        if hMod.is_null() {
            break 'quit;
        }

        if GetProcAddress(hMod as HINSTANCE, b"Chkdsk\0".as_ptr() as *const i8).is_some() {
            ProcCount = ProcCount.wrapping_add(1);
        }
        if GetProcAddress(hMod as HINSTANCE, b"ChkdskEx\0".as_ptr() as *const i8).is_some() {
            ProcCount = ProcCount.wrapping_add(1);
        }
        if GetProcAddress(hMod as HINSTANCE, b"Extend\0".as_ptr() as *const i8).is_some() {
            ProcCount = ProcCount.wrapping_add(1);
        }
        if GetProcAddress(hMod as HINSTANCE, b"Format\0".as_ptr() as *const i8).is_some() {
            ProcCount = ProcCount.wrapping_add(1);
        }
        if GetProcAddress(hMod as HINSTANCE, b"FormatEx\0".as_ptr() as *const i8).is_some() {
            ProcCount = ProcCount.wrapping_add(1);
        }

        DPRINT!("ProcCount %lu\n", ProcCount);
        if ProcCount == 0 {
            break 'quit;
        }

        Provider = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<IFS_PROVIDER>() as SIZE_T,
        ) as PIFS_PROVIDER;
        if Provider.is_null() {
            break 'quit;
        }

        (*Provider).Name = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            ((*FileSystem).Length as SIZE_T)
                .wrapping_add(core::mem::size_of_val(&UNICODE_NULL) as SIZE_T),
        ) as PWSTR;
        if (*Provider).Name.is_null() {
            break 'quit;
        }

        RtlCopyMemory!(
            (*Provider).Name as *mut core::ffi::c_void,
            (*FileSystem).Buffer as *const core::ffi::c_void,
            (*FileSystem).Length as SIZE_T,
        );

        (*Provider).DllFile = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            HEAP_ZERO_MEMORY,
            (wcslen(DllFile as *const wchar_t) as SIZE_T)
                .wrapping_add(1 as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if (*Provider).DllFile.is_null() {
            break 'quit;
        }

        wcscpy(
            (*Provider).DllFile as *mut wchar_t,
            DllFile as *const wchar_t,
        );

        InsertTailList(&raw mut ProviderListHead, &raw mut (*Provider).ListEntry);
        ret = (TRUE as u8);

        DPRINT!("AddProvider success\n");
    }

    // cleanup:
    if !hMod.is_null() {
        FreeLibrary(hMod);
    }

    if ret == 0 {
        if !Provider.is_null() {
            if !(*Provider).Name.is_null() {
                RtlFreeHeap(
                    RtlGetProcessHeap!(),
                    0,
                    (*Provider).Name as PVOID,
                );
            }
            if !(*Provider).DllFile.is_null() {
                RtlFreeHeap(
                    RtlGetProcessHeap!(),
                    0,
                    (*Provider).DllFile as PVOID,
                );
            }
            RtlFreeHeap(
                RtlGetProcessHeap!(),
                0,
                Provider as PVOID,
            );
        }
    }
    return ret;
}

unsafe extern "C" fn InitializeFmIfsOnce() -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut::<core::ffi::c_void>(),
        ObjectName: core::ptr::null_mut::<_UNICODE_STRING>(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
        SecurityQualityOfService: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut RegistryPath: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_8AEFC07D888B.as_ptr());
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut Buffer: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut BufferSize: ULONG = (core::mem::size_of::<KEY_VALUE_FULL_INFORMATION>() as ULONG)
        .wrapping_add(MAX_PATH);
    let mut RequiredSize: ULONG = 0;
    let mut i: ULONG = 0;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut Data: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut Status: NTSTATUS = 0;

    InitializeListHead(&raw mut ProviderListHead);

    /* Read IFS providers from HKLM\SOFTWARE\ReactOS\ReactOS\CurrentVersion\IFS */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut RegistryPath;
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenKey(&raw mut hKey, KEY_QUERY_VALUE, &raw mut ObjectAttributes);
    if Status == STATUS_OBJECT_NAME_NOT_FOUND {
        return (TRUE) as BOOLEAN;
    } else if !NT_SUCCESS!(Status) {
        return (FALSE) as BOOLEAN;
    }

    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        (BufferSize) as SIZE_T,
    ) as PKEY_VALUE_FULL_INFORMATION;
    if Buffer.is_null() {
        NtClose(hKey);
        return (FALSE) as BOOLEAN;
    }

    loop {
        let fresh15 = i;
        i = i.wrapping_add(1);
        Status = NtEnumerateValueKey(
            hKey,
            fresh15,
            KeyValueFullInformation,
            Buffer as PVOID,
            BufferSize,
            &raw mut RequiredSize,
        );
        if Status == STATUS_BUFFER_OVERFLOW {
            continue;
        } else if !NT_SUCCESS!(Status) {
            break;
        } else if (*Buffer).Type != REG_SZ {
            continue;
        }

        Name.MaximumLength = (*Buffer).NameLength as USHORT;
        Name.Length = Name.MaximumLength;
        Name.Buffer = &raw mut (*Buffer).Name as *mut WCHAR as PWSTR;
        Data.MaximumLength = (*Buffer).DataLength as USHORT;
        Data.Length = Data.MaximumLength;
        Data.Buffer = (Buffer as ULONG_PTR).wrapping_add((*Buffer).DataOffset as ULONG_PTR)
            as PWCHAR as PWSTR;
        if Data.Length as usize > core::mem::size_of::<WCHAR>() as usize
            && *Data.Buffer.offset(
                (Data.Length as usize)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) == UNICODE_NULL
        {
            Data.Length = (Data.Length as usize)
                .wrapping_sub(core::mem::size_of::<WCHAR>() as usize) as USHORT;
        }

        AddProvider(&raw mut Name as PCUNICODE_STRING, Data.Buffer as PWCHAR);
    }

    NtClose(hKey);
    RtlFreeHeap(RtlGetProcessHeap!(), 0, Buffer as PVOID);
    return (TRUE) as BOOLEAN;
}

unsafe extern "C" fn UninitializeFmIfsOnce() {
    let mut ListEntry: PLIST_ENTRY;
    let mut Provider: PIFS_PROVIDER;
    while IsListEmpty(core::ptr::addr_of_mut!(ProviderListHead)) == false {
        ListEntry = RemoveTailList(core::ptr::addr_of_mut!(ProviderListHead));
        Provider = CONTAINING_RECORD!(ListEntry, IFS_PROVIDER, ListEntry);
        if !(*Provider).Name.is_null() {
            RtlFreeHeap(RtlGetProcessHeap!(), 0, (*Provider).Name as PVOID);
        }
        if !(*Provider).DllFile.is_null() {
            RtlFreeHeap(RtlGetProcessHeap!(), 0, (*Provider).DllFile as PVOID);
        }
        if !(*Provider).hModule.is_null() {
            FreeLibrary((*Provider).hModule);
        }
        RtlFreeHeap(RtlGetProcessHeap!(), 0, Provider as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn InitializeFmIfs(
    hinstDll: PVOID,
    dwReason: DWORD,
    reserved: PVOID,
) -> BOOLEAN {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            if FmIfsAttached == 0 {
                if InitializeFmIfsOnce() == (FALSE as u8) {
                    return (FALSE) as BOOLEAN;
                }

                FmIfsAttached = FmIfsAttached.wrapping_add(1);
            }
        }

        DLL_THREAD_ATTACH => {}

        DLL_THREAD_DETACH => {}

        DLL_PROCESS_DETACH => {
            FmIfsAttached = FmIfsAttached.wrapping_sub(1);
            if FmIfsAttached == 0 {
                UninitializeFmIfsOnce();
            }
        }

        _ => {}
    }

    return (TRUE) as BOOLEAN;
}
