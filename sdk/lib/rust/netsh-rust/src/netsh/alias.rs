// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use ntdll_sys::nt::*;
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_7CDC108434B2: [u16; 10] = [0x0025u16, 0x0073, 0x0020, 0x003A, 0x0020, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut AliasListHead: PALIAS_ENTRY =
    ::core::ptr::null::<_ALIAS_ENTRY>() as *mut _ALIAS_ENTRY;

#[no_mangle]
pub static mut AliasListTail: PALIAS_ENTRY =
    ::core::ptr::null::<_ALIAS_ENTRY>() as *mut _ALIAS_ENTRY;

#[repr(C)]
pub struct _ALIAS_ENTRY {
    pub pPrev: *mut _ALIAS_ENTRY,
    pub pNext: *mut _ALIAS_ENTRY,
    pub pszAliasName: PWSTR,
    pub pszAlias: PWSTR,
}

pub type ALIAS_ENTRY = _ALIAS_ENTRY;
pub type PALIAS_ENTRY = *mut _ALIAS_ENTRY;

unsafe extern "C" fn ShowAliases() {
    let mut pAliasEntry: PALIAS_ENTRY = core::ptr::null_mut();

    DPRINT1!("ShowAliases()\n");

    pAliasEntry = AliasListHead;
    while !pAliasEntry.is_null() {
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_7CDC108434B2.as_ptr(),
            (*pAliasEntry).pszAliasName,
            (*pAliasEntry).pszAlias,
        );
        pAliasEntry = (*pAliasEntry).pNext;
    }
}

unsafe extern "C" fn ShowAlias(pszAliasName: PWSTR) {
    let mut pAliasEntry: PALIAS_ENTRY = core::ptr::null_mut();

    DPRINT1!("ShowAlias(%S)\n", pszAliasName);

    pAliasEntry = AliasListHead;
    while !pAliasEntry.is_null() {
        if wcscmp((*pAliasEntry).pszAliasName, pszAliasName) == 0 {
            ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), (*pAliasEntry).pszAlias);
            return;
        }

        pAliasEntry = (*pAliasEntry).pNext;
    }

    ConResPrintf(StdOut, IDS_ALIAS_NOT_FOUND, pszAliasName);
}

unsafe extern "C" fn GetAliasEntry(mut pszAliasName: PWSTR) -> PALIAS_ENTRY {
    let mut pAliasEntry: PALIAS_ENTRY = core::ptr::null_mut::<ALIAS_ENTRY>();

    pAliasEntry = AliasListHead;
    while !pAliasEntry.is_null() {
        if wcscmp(
            (*pAliasEntry).pszAliasName as *const u16,
            pszAliasName as *const u16,
        ) == 0
        {
            return pAliasEntry;
        }

        pAliasEntry = (*pAliasEntry).pNext;
    }

    return core::ptr::null_mut::<ALIAS_ENTRY>();
}
#[no_mangle]
pub unsafe extern "C" fn InitAliases() {
    AliasListHead = core::ptr::null_mut::<ALIAS_ENTRY>();
    AliasListTail = core::ptr::null_mut::<ALIAS_ENTRY>();
}
#[no_mangle]
pub unsafe extern "C" fn DestroyAliases() {
    let mut pAliasEntry: PALIAS_ENTRY = core::ptr::null_mut::<_ALIAS_ENTRY>();
    while !AliasListHead.is_null() {
        pAliasEntry = AliasListHead;
        AliasListHead = (*AliasListHead).pNext;

        HeapFree(GetProcessHeap(), 0, (*pAliasEntry).pszAliasName as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, (*pAliasEntry).pszAlias as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, pAliasEntry as *mut core::ffi::c_void);
    }

    AliasListTail = core::ptr::null_mut::<_ALIAS_ENTRY>();
}

#[no_mangle]
pub unsafe extern "system" fn AliasCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    let mut pAliasEntry: PALIAS_ENTRY = core::ptr::null_mut();
    let mut pszAlias: PWSTR;

    DPRINT!(
        "AliasCommand(dwCurrentIndex %lu dwArgCount %lu)\n",
        dwCurrentIndex,
        dwArgCount
    );

    /* Show aliases */
    if dwArgCount.wrapping_sub(dwCurrentIndex) == 0 as DWORD {
        ShowAliases();
        return ERROR_SUCCESS as DWORD;
    }

    if dwArgCount.wrapping_sub(dwCurrentIndex) == 1 as DWORD {
        ShowAlias(*argv.offset(dwCurrentIndex as isize));
        return ERROR_SUCCESS as DWORD;
    }

    /* TODO: Check builtin commands */

    pAliasEntry = GetAliasEntry(*argv.offset(dwCurrentIndex as isize));
    if !pAliasEntry.is_null() {
        pszAlias = MergeStrings(
            argv.offset(dwCurrentIndex.wrapping_add(1 as DWORD) as isize),
            dwArgCount
                .wrapping_sub(dwCurrentIndex)
                .wrapping_sub(1 as DWORD) as UINT,
        ) as PWSTR;
        DPRINT!("Alias: %S\n", pszAlias);
        if pszAlias.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY as DWORD;
        }

        if !(*pAliasEntry).pszAlias.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*pAliasEntry).pszAlias as PVOID,
            );
        }

        (*pAliasEntry).pszAlias = pszAlias;
    } else {
        pAliasEntry = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            core::mem::size_of::<ALIAS_ENTRY>() as SIZE_T,
        ) as PALIAS_ENTRY;
        if pAliasEntry.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY as DWORD;
        }

        (*pAliasEntry).pszAliasName = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (wcslen(*argv.offset(dwCurrentIndex as isize) as LPCWSTR) as SIZE_T)
                .wrapping_add(1 as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if (*pAliasEntry).pszAliasName.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pAliasEntry as PVOID,
            );
            return ERROR_NOT_ENOUGH_MEMORY as DWORD;
        }

        wcscpy(
            (*pAliasEntry).pszAliasName,
            *argv.offset(dwCurrentIndex as isize) as LPCWSTR,
        );
        DPRINT!("AliasName: %S\n", (*pAliasEntry).pszAliasName);

        (*pAliasEntry).pszAlias = MergeStrings(
            argv.offset(dwCurrentIndex.wrapping_add(1 as DWORD) as isize),
            dwArgCount
                .wrapping_sub(dwCurrentIndex)
                .wrapping_sub(1 as DWORD) as UINT,
        ) as PWSTR;
        DPRINT!("Alias: %S\n", (*pAliasEntry).pszAlias);
        if (*pAliasEntry).pszAlias.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*pAliasEntry).pszAliasName as PVOID,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pAliasEntry as PVOID,
            );
            return ERROR_NOT_ENOUGH_MEMORY as DWORD;
        }

        if AliasListHead.is_null() {
            AliasListHead = pAliasEntry;
            AliasListTail = pAliasEntry;
        } else {
            (*pAliasEntry).pPrev = AliasListTail;
            (*AliasListTail).pNext = pAliasEntry;
            AliasListTail = pAliasEntry;
        }
    }

    DPRINT!("Done\n");

    return ERROR_OKAY as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn ShowAliasCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("ShowAliasCommand()\n");

    ShowAliases();

    return ERROR_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn UnaliasCommand(
    mut pwszMachine: LPCWSTR,
    mut argv: *mut LPWSTR,
    mut dwCurrentIndex: DWORD,
    mut dwArgCount: DWORD,
    mut dwFlags: DWORD,
    mut pvData: LPCVOID,
    mut pbDone: *mut BOOL,
) -> DWORD {
    let mut pAliasEntry: PALIAS_ENTRY = core::ptr::null_mut::<_ALIAS_ENTRY>();

    DPRINT!("UnaliasCommand()\n");

    if dwArgCount.wrapping_sub(dwCurrentIndex) != 1 as DWORD {
        return ERROR_INVALID_SYNTAX as DWORD;
    }

    DPRINT!("Alias %S\n", *argv.offset(dwCurrentIndex as isize));

    pAliasEntry = AliasListHead;
    while !pAliasEntry.is_null() {
        if wcscmp(
            (*pAliasEntry).pszAliasName as *const wchar_t,
            *argv.offset(dwCurrentIndex as isize) as *const wchar_t,
        ) == 0 as ::core::ffi::c_int
        {
            DPRINT!("Alias found %S\n", *argv.offset(dwCurrentIndex as isize));

            if !(*pAliasEntry).pNext.is_null() {
                (*(*pAliasEntry).pNext).pPrev = (*pAliasEntry).pPrev;
            } else {
                AliasListTail = (*pAliasEntry).pPrev as PALIAS_ENTRY;
            }

            if !(*pAliasEntry).pPrev.is_null() {
                (*(*pAliasEntry).pPrev).pNext = (*pAliasEntry).pNext;
            } else {
                AliasListHead = (*pAliasEntry).pNext as PALIAS_ENTRY;
            }

            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*pAliasEntry).pszAliasName as PVOID,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*pAliasEntry).pszAlias as PVOID,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pAliasEntry as PVOID,
            );

            return ERROR_SUCCESS as DWORD;
        }

        pAliasEntry = (*pAliasEntry).pNext as PALIAS_ENTRY;
    }

    ConResPrintf(
        &raw mut csStdOut,
        IDS_ALIAS_NOT_FOUND as UINT,
        *argv.offset(dwCurrentIndex as isize),
    );

    return ERROR_SUCCESS as DWORD;
}
