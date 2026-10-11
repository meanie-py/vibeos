use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn isspace(c: i32) -> i32;
    fn SECUR32_initNTLMSP();
    fn SECUR32_initKerberosSP();
    fn SECUR32_initNegotiateSP();
}
extern "system" {
    // advapi32 registry + kernel32 lstrlenW are WINAPI (stdcall) — i386 needs the
    // `@N` decoration (extern "C" here produced undefined `_RegOpenKeyExW` etc).
    fn RegOpenKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: u32, samDesired: u32, phkResult: *mut HKEY) -> LSTATUS;
    fn RegQueryValueExW(hKey: HKEY, lpValueName: LPCWSTR, lpReserved: *mut u32, lpType: *mut u32, lpData: PBYTE, lpcbData: *mut u32) -> LSTATUS;
    fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    fn lstrlenW(lpString: LPCWSTR) -> i32;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn lstrcpyW(lpString1: PWSTR, lpString2: PCWSTR) -> PWSTR;
    fn MultiByteToWideChar( CodePage: u32, dwFlags: u32, lpMultiByteStr: PCSTR, cbMultiByte: i32, lpWideCharStr: PWSTR, cchWideChar: i32, ) -> i32;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    fn SetLastError(dwErrCode: u32);
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: LPCWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut i32, ) -> i32;
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
}

// The SecurityFunctionTable members below resolve to the per-package SSPI
// dispatchers translated in wrapper.rs (AcquireCredentialsHandleA, …) plus the
// two locally-defined ones (EnumerateSecurityPackages*/FreeContextBuffer).
use crate::wrapper::*;

// ---- file-scope preamble (translated) ----
pub const UNLEN: u32 = 256;

#[repr(C)]
pub struct _SecurePackageTable {
    pub numPackages: DWORD,
    pub numAllocated: DWORD,
    pub table: list,
}
pub type SecurePackageTable = _SecurePackageTable;
pub type PSecurePackageTable = *mut _SecurePackageTable;

#[repr(C)]
pub struct _SecureProviderTable {
    pub numProviders: DWORD,
    pub numAllocated: DWORD,
    pub table: list,
}
pub type SecureProviderTable = _SecureProviderTable;
pub type PSecureProviderTable = *mut _SecureProviderTable;

pub static securityProvidersKeyW: [WCHAR; 51] = [
    'S' as WCHAR, 'Y' as WCHAR, 'S' as WCHAR, 'T' as WCHAR, 'E' as WCHAR, 'M' as WCHAR, '\\' as WCHAR,
    'C' as WCHAR, 'u' as WCHAR, 'r' as WCHAR, 'r' as WCHAR, 'e' as WCHAR, 'n' as WCHAR, 't' as WCHAR,
    'C' as WCHAR, 'o' as WCHAR, 'n' as WCHAR, 't' as WCHAR, 'r' as WCHAR, 'o' as WCHAR, 'l' as WCHAR,
    'S' as WCHAR, 'e' as WCHAR, 't' as WCHAR, '\\' as WCHAR,
    'C' as WCHAR, 'o' as WCHAR, 'n' as WCHAR, 't' as WCHAR, 'r' as WCHAR, 'o' as WCHAR, 'l' as WCHAR, '\\' as WCHAR,
    'S' as WCHAR, 'e' as WCHAR, 'c' as WCHAR, 'u' as WCHAR, 'r' as WCHAR, 'i' as WCHAR, 't' as WCHAR, 'y' as WCHAR,
    'P' as WCHAR, 'r' as WCHAR, 'o' as WCHAR, 'v' as WCHAR, 'i' as WCHAR, 'd' as WCHAR, 'e' as WCHAR, 'r' as WCHAR, 's' as WCHAR,
    0,
];

pub static securityProvidersW: [WCHAR; 18] = [
    'S' as WCHAR, 'e' as WCHAR, 'c' as WCHAR, 'u' as WCHAR, 'r' as WCHAR, 'i' as WCHAR, 't' as WCHAR, 'y' as WCHAR,
    'P' as WCHAR, 'r' as WCHAR, 'o' as WCHAR, 'v' as WCHAR, 'i' as WCHAR, 'd' as WCHAR, 'e' as WCHAR, 'r' as WCHAR, 's' as WCHAR,
    0,
];

#[no_mangle]
pub unsafe extern "system" fn InitSecurityInterfaceA() -> PSecurityFunctionTableA {
    core::ptr::addr_of_mut!(securityFunctionTableA)
}

#[no_mangle]
pub unsafe extern "system" fn InitSecurityInterfaceW() -> PSecurityFunctionTableW {
    core::ptr::addr_of_mut!(securityFunctionTableW)
}

// ---- file-scope statics dropped by the translation (recovered from sspi.c) ----
// C: static CRITICAL_SECTION cs = { &cs_debug, -1, 0, 0, 0, 0 };  The DebugInfo
// self-referential cs_debug record is diagnostic-only (lock list / source tag);
// RTL critical sections operate correctly with a NULL DebugInfo, so we drop the
// debug record and keep the unlocked init (LockCount = -1).
pub static mut cs: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: core::ptr::null_mut(),
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};
pub static mut packageTable: PSecurePackageTable = core::ptr::null_mut();
pub static mut providerTable: PSecureProviderTable = core::ptr::null_mut();

// C: static SecurityFunctionTableA securityFunctionTableA = { … };  Reserved3 and
// Reserved4 are repurposed as the legacy Encrypt/Decrypt message slots (wine), so
// they hold the (raw-pointer-cast) wrapper EncryptMessage/DecryptMessage.
pub static mut securityFunctionTableA: SecurityFunctionTableA = SecurityFunctionTableA {
    dwVersion: SECURITY_SUPPORT_PROVIDER_INTERFACE_VERSION as ULONG,
    EnumerateSecurityPackagesA: sec_entry!(EnumerateSecurityPackagesA),
    QueryCredentialsAttributesA: sec_entry!(QueryCredentialsAttributesA),
    AcquireCredentialsHandleA: sec_entry!(AcquireCredentialsHandleA),
    FreeCredentialsHandle: sec_entry!(FreeCredentialsHandle),
    Reserved2: core::ptr::null_mut(),
    InitializeSecurityContextA: sec_entry!(InitializeSecurityContextA),
    AcceptSecurityContext: sec_entry!(AcceptSecurityContext),
    CompleteAuthToken: sec_entry!(CompleteAuthToken),
    DeleteSecurityContext: sec_entry!(DeleteSecurityContext),
    ApplyControlToken: sec_entry!(ApplyControlToken),
    QueryContextAttributesA: sec_entry!(QueryContextAttributesA),
    ImpersonateSecurityContext: sec_entry!(ImpersonateSecurityContext),
    RevertSecurityContext: sec_entry!(RevertSecurityContext),
    MakeSignature: sec_entry!(MakeSignature),
    VerifySignature: sec_entry!(VerifySignature),
    FreeContextBuffer: sec_entry!(FreeContextBuffer),
    QuerySecurityPackageInfoA: sec_entry!(QuerySecurityPackageInfoA),
    Reserved3: EncryptMessage as *const () as *mut c_void,
    Reserved4: DecryptMessage as *const () as *mut c_void,
    ExportSecurityContext: sec_entry!(ExportSecurityContext),
    ImportSecurityContextA: sec_entry!(ImportSecurityContextA),
    AddCredentialsA: sec_entry!(AddCredentialsA),
    Reserved8: core::ptr::null_mut(),
    QuerySecurityContextToken: sec_entry!(QuerySecurityContextToken),
    EncryptMessage: sec_entry!(EncryptMessage),
    DecryptMessage: sec_entry!(DecryptMessage),
    SetContextAttributesA: None,
};

pub static mut securityFunctionTableW: SecurityFunctionTableW = SecurityFunctionTableW {
    dwVersion: SECURITY_SUPPORT_PROVIDER_INTERFACE_VERSION as ULONG,
    EnumerateSecurityPackagesW: sec_entry!(EnumerateSecurityPackagesW),
    QueryCredentialsAttributesW: sec_entry!(QueryCredentialsAttributesW),
    AcquireCredentialsHandleW: sec_entry!(AcquireCredentialsHandleW),
    FreeCredentialsHandle: sec_entry!(FreeCredentialsHandle),
    Reserved2: core::ptr::null_mut(),
    InitializeSecurityContextW: sec_entry!(InitializeSecurityContextW),
    AcceptSecurityContext: sec_entry!(AcceptSecurityContext),
    CompleteAuthToken: sec_entry!(CompleteAuthToken),
    DeleteSecurityContext: sec_entry!(DeleteSecurityContext),
    ApplyControlToken: sec_entry!(ApplyControlToken),
    QueryContextAttributesW: sec_entry!(QueryContextAttributesW),
    ImpersonateSecurityContext: sec_entry!(ImpersonateSecurityContext),
    RevertSecurityContext: sec_entry!(RevertSecurityContext),
    MakeSignature: sec_entry!(MakeSignature),
    VerifySignature: sec_entry!(VerifySignature),
    FreeContextBuffer: sec_entry!(FreeContextBuffer),
    QuerySecurityPackageInfoW: sec_entry!(QuerySecurityPackageInfoW),
    Reserved3: EncryptMessage as *const () as *mut c_void,
    Reserved4: DecryptMessage as *const () as *mut c_void,
    ExportSecurityContext: sec_entry!(ExportSecurityContext),
    ImportSecurityContextW: sec_entry!(ImportSecurityContextW),
    AddCredentialsW: sec_entry!(AddCredentialsW),
    Reserved8: core::ptr::null_mut(),
    QuerySecurityContextToken: sec_entry!(QuerySecurityContextToken),
    EncryptMessage: sec_entry!(EncryptMessage),
    DecryptMessage: sec_entry!(DecryptMessage),
    SetContextAttributesW: None,
};

unsafe fn SECUR32_strdupW(str: PCWSTR) -> PWSTR {

    let ret: PWSTR;

    if !str.is_null() {
        ret = HeapAlloc(GetProcessHeap(), 0, ((lstrlenW(str) + 1) as usize) * core::mem::size_of::<u16>()) as PWSTR;
        if !ret.is_null() {
            lstrcpyW(ret, str);
        }
    } else {
        ret = core::ptr::null_mut();
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_AllocWideFromMultiByte(str: PCSTR) -> PWSTR {

    const CP_ACP: u32 = 0;

    let ret: PWSTR;

    if !str.is_null() {
        let charsNeeded = MultiByteToWideChar(CP_ACP, 0, str, -1, core::ptr::null_mut(), 0);

        if charsNeeded != 0 {
            ret = HeapAlloc(
                GetProcessHeap(),
                0,
                charsNeeded as usize * core::mem::size_of::<u16>(),
            ) as PWSTR;
            if !ret.is_null() {
                MultiByteToWideChar(CP_ACP, 0, str, -1, ret, charsNeeded);
            }
        } else {
            ret = core::ptr::null_mut();
        }
    } else {
        ret = core::ptr::null_mut();
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_AllocMultiByteFromWide(str: PCWSTR) -> PSTR {
    let ret: PSTR;

    if !str.is_null() {
        let charsNeeded = WideCharToMultiByte(
            CP_ACP,
            0,
            str,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if charsNeeded != 0 {
            ret = HeapAlloc(GetProcessHeap(), 0, charsNeeded as usize) as PSTR;
            if !ret.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    str,
                    -1,
                    ret,
                    charsNeeded,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }
        } else {
            ret = core::ptr::null_mut();
        }
    } else {
        ret = core::ptr::null_mut();
    }
    ret
}

unsafe fn _makeFnTableA(
    fnTableA: PSecurityFunctionTableA,
    inFnTableA: *const SecurityFunctionTableA,
    inFnTableW: *const SecurityFunctionTableW,
) {
    if !fnTableA.is_null() {
        if !inFnTableA.is_null() {
            let tableSize: usize = if (*inFnTableA).dwVersion == 1 {
                (core::ptr::addr_of!((*inFnTableA).SetContextAttributesA) as *const u8)
                    .offset_from(inFnTableA as *const u8) as usize
            } else {
                core::mem::size_of::<SecurityFunctionTableA>()
            };

            core::ptr::copy_nonoverlapping(
                inFnTableA as *const u8,
                fnTableA as *mut u8,
                tableSize,
            );
            (*fnTableA).QuerySecurityPackageInfoA = sec_entry!((QuerySecurityPackageInfoA));
        } else if !inFnTableW.is_null() {
            if (*inFnTableW).AcquireCredentialsHandleW.is_some() {
                (*fnTableA).AcquireCredentialsHandleA = sec_entry!(thunk_AcquireCredentialsHandleA);
            }
            if (*inFnTableW).InitializeSecurityContextW.is_some() {
                (*fnTableA).InitializeSecurityContextA = sec_entry!(thunk_InitializeSecurityContextA);
            }
            if (*inFnTableW).ImportSecurityContextW.is_some() {
                (*fnTableA).ImportSecurityContextA = sec_entry!(thunk_ImportSecurityContextA);
            }
            if (*inFnTableW).AddCredentialsW.is_some() {
                (*fnTableA).AddCredentialsA = sec_entry!(thunk_AddCredentialsA);
            }
            if (*inFnTableW).QueryCredentialsAttributesW.is_some() {
                (*fnTableA).QueryCredentialsAttributesA = sec_entry!(thunk_QueryCredentialsAttributesA);
            }
            if (*inFnTableW).QueryContextAttributesW.is_some() {
                (*fnTableA).QueryContextAttributesA = sec_entry!(thunk_QueryContextAttributesA);
            }
            if (*inFnTableW).SetContextAttributesW.is_some() {
                (*fnTableA).SetContextAttributesA = sec_entry!(thunk_SetContextAttributesA);
            }
            (*fnTableA).EnumerateSecurityPackagesA = None;
            (*fnTableA).AcceptSecurityContext = (*inFnTableW).AcceptSecurityContext;
            (*fnTableA).CompleteAuthToken = (*inFnTableW).CompleteAuthToken;
            (*fnTableA).DeleteSecurityContext = (*inFnTableW).DeleteSecurityContext;
            (*fnTableA).ImpersonateSecurityContext = (*inFnTableW).ImpersonateSecurityContext;
            (*fnTableA).RevertSecurityContext = (*inFnTableW).RevertSecurityContext;
            (*fnTableA).MakeSignature = (*inFnTableW).MakeSignature;
            (*fnTableA).VerifySignature = (*inFnTableW).VerifySignature;
            (*fnTableA).FreeContextBuffer = (*inFnTableW).FreeContextBuffer;
            (*fnTableA).QuerySecurityPackageInfoA = sec_entry!((QuerySecurityPackageInfoA));
            (*fnTableA).ExportSecurityContext = (*inFnTableW).ExportSecurityContext;
            (*fnTableA).QuerySecurityContextToken = (*inFnTableW).QuerySecurityContextToken;
            (*fnTableA).EncryptMessage = (*inFnTableW).EncryptMessage;
            (*fnTableA).DecryptMessage = (*inFnTableW).DecryptMessage;
        }
    }
}

unsafe fn _makeFnTableW(
    fnTableW: PSecurityFunctionTableW,
    inFnTableA: *const SecurityFunctionTableA,
    inFnTableW: *const SecurityFunctionTableW,
) {
    if !fnTableW.is_null() {
        if !inFnTableW.is_null() {
            let tableSize: usize = if (*inFnTableW).dwVersion == 1 {
                (core::ptr::addr_of!((*inFnTableW).SetContextAttributesW) as *const BYTE)
                    .offset_from(inFnTableW as *const BYTE) as usize
            } else {
                core::mem::size_of::<SecurityFunctionTableW>()
            };

            core::ptr::copy_nonoverlapping(
                inFnTableW as *const u8,
                fnTableW as *mut u8,
                tableSize,
            );
            (*fnTableW).QuerySecurityPackageInfoW = sec_entry!((QuerySecurityPackageInfoW));
        } else if !inFnTableA.is_null() {
            if (*inFnTableA).AcquireCredentialsHandleA.is_some() {
                (*fnTableW).AcquireCredentialsHandleW = sec_entry!(thunk_AcquireCredentialsHandleW);
            }
            if (*inFnTableA).InitializeSecurityContextA.is_some() {
                (*fnTableW).InitializeSecurityContextW = sec_entry!(thunk_InitializeSecurityContextW);
            }
            if (*inFnTableA).ImportSecurityContextA.is_some() {
                (*fnTableW).ImportSecurityContextW = sec_entry!(thunk_ImportSecurityContextW);
            }
            if (*inFnTableA).AddCredentialsA.is_some() {
                (*fnTableW).AddCredentialsW = sec_entry!(thunk_AddCredentialsW);
            }
            if (*inFnTableA).QueryCredentialsAttributesA.is_some() {
                (*fnTableW).QueryCredentialsAttributesW = sec_entry!(thunk_QueryCredentialsAttributesW);
            }
            if (*inFnTableA).QueryContextAttributesA.is_some() {
                (*fnTableW).QueryContextAttributesW = sec_entry!(thunk_QueryContextAttributesW);
            }
            if (*inFnTableA).SetContextAttributesA.is_some() {
                (*fnTableW).SetContextAttributesW = sec_entry!(thunk_SetContextAttributesW);
            }
            (*fnTableW).EnumerateSecurityPackagesW = None;
            (*fnTableW).AcceptSecurityContext = (*inFnTableA).AcceptSecurityContext;
            (*fnTableW).CompleteAuthToken = (*inFnTableA).CompleteAuthToken;
            (*fnTableW).DeleteSecurityContext = (*inFnTableA).DeleteSecurityContext;
            (*fnTableW).ImpersonateSecurityContext = (*inFnTableA).ImpersonateSecurityContext;
            (*fnTableW).RevertSecurityContext = (*inFnTableA).RevertSecurityContext;
            (*fnTableW).MakeSignature = (*inFnTableA).MakeSignature;
            (*fnTableW).VerifySignature = (*inFnTableA).VerifySignature;
            (*fnTableW).FreeContextBuffer = (*inFnTableA).FreeContextBuffer;
            (*fnTableW).QuerySecurityPackageInfoW = sec_entry!((QuerySecurityPackageInfoW));
            (*fnTableW).ExportSecurityContext = (*inFnTableA).ExportSecurityContext;
            (*fnTableW).QuerySecurityContextToken = (*inFnTableA).QuerySecurityContextToken;
            (*fnTableW).EncryptMessage = (*inFnTableA).EncryptMessage;
            (*fnTableW).DecryptMessage = (*inFnTableA).DecryptMessage;
        }
    }
}

unsafe fn _copyPackageInfo(info: PSecPkgInfoW, inInfoA: *const SecPkgInfoA, inInfoW: *const SecPkgInfoW) {
    if !info.is_null() && (!inInfoA.is_null() || !inInfoW.is_null()) {
        let src = if !inInfoW.is_null() {
            inInfoW
        } else {
            inInfoA as *const SecPkgInfoW
        };
        memcpy(
            info as *mut core::ffi::c_void,
            src as *const core::ffi::c_void,
            core::mem::size_of::<SecPkgInfoW>(),
        );
        if !inInfoW.is_null() {
            (*info).Name = SECUR32_strdupW((*inInfoW).Name);
            (*info).Comment = SECUR32_strdupW((*inInfoW).Comment);
        } else {
            (*info).Name = SECUR32_AllocWideFromMultiByte((*inInfoA).Name);
            (*info).Comment = SECUR32_AllocWideFromMultiByte((*inInfoA).Comment);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_addProvider(
    fnTableA: *const SecurityFunctionTableA,
    fnTableW: *const SecurityFunctionTableW,
    moduleName: PCWSTR,
) -> *mut SecureProvider {
    let ret: *mut SecureProvider;

    EnterCriticalSection(core::ptr::addr_of_mut!(cs));

    if providerTable.is_null() {
        providerTable = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<SecureProviderTable>(),
        ) as *mut SecureProviderTable;
        if providerTable.is_null() {
            LeaveCriticalSection(core::ptr::addr_of_mut!(cs));
            return core::ptr::null_mut();
        }

        list_init(core::ptr::addr_of_mut!((*providerTable).table));
    }

    ret = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<SecureProvider>(),
    ) as *mut SecureProvider;
    if ret.is_null() {
        LeaveCriticalSection(core::ptr::addr_of_mut!(cs));
        return core::ptr::null_mut();
    }

    list_add_tail(
        core::ptr::addr_of_mut!((*providerTable).table),
        core::ptr::addr_of_mut!((*ret).entry),
    );
    (*ret).lib = core::ptr::null_mut();

    if !fnTableA.is_null() || !fnTableW.is_null() {
        (*ret).moduleName = if !moduleName.is_null() {
            SECUR32_strdupW(moduleName)
        } else {
            core::ptr::null_mut()
        };
        _makeFnTableA(core::ptr::addr_of_mut!((*ret).fnTableA), fnTableA, fnTableW);
        _makeFnTableW(core::ptr::addr_of_mut!((*ret).fnTableW), fnTableA, fnTableW);
        (*ret).loaded = if moduleName.is_null() { 1 } else { 0 };
    } else {
        (*ret).moduleName = SECUR32_strdupW(moduleName);
        (*ret).loaded = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(cs));
    ret
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_addPackages(
    provider: *mut SecureProvider,
    toAdd: ULONG,
    infoA: *const SecPkgInfoA,
    infoW: *const SecPkgInfoW,
) {

    EnterCriticalSection(core::ptr::addr_of_mut!(cs));

    if packageTable.is_null() {
        packageTable = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<SecurePackageTable>(),
        ) as *mut SecurePackageTable;
        if packageTable.is_null() {
            LeaveCriticalSection(core::ptr::addr_of_mut!(cs));
            return;
        }

        (*packageTable).numPackages = 0;
        list_init(core::ptr::addr_of_mut!((*packageTable).table));
    }

    let mut i: ULONG = 0;
    while i < toAdd {
        let package = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<SecurePackage>(),
        ) as *mut SecurePackage;
        if package.is_null() {
            i += 1;
            continue;
        }

        list_add_tail(
            core::ptr::addr_of_mut!((*packageTable).table),
            core::ptr::addr_of_mut!((*package).entry),
        );

        (*package).provider = provider;
        _copyPackageInfo(
            core::ptr::addr_of_mut!((*package).infoW),
            if infoA.is_null() {
                core::ptr::null()
            } else {
                infoA.add(i as usize)
            },
            if infoW.is_null() {
                core::ptr::null()
            } else {
                infoW.add(i as usize)
            },
        );

        i += 1;
    }
    (*packageTable).numPackages += toAdd;

    LeaveCriticalSection(core::ptr::addr_of_mut!(cs));
}

unsafe fn _tryLoadProvider(moduleName: PWSTR) {
    let lib = LoadLibraryW(moduleName);

    if !lib.is_null() {
        let pInitSecurityInterfaceW: INIT_SECURITY_INTERFACE_W =
            core::mem::transmute(GetProcAddress(lib, SECURITY_ENTRYPOINT_ANSIW.as_ptr() as LPCSTR));
        let pInitSecurityInterfaceA: INIT_SECURITY_INTERFACE_A =
            core::mem::transmute(GetProcAddress(lib, SECURITY_ENTRYPOINT_ANSIA.as_ptr() as LPCSTR));

        if pInitSecurityInterfaceW.is_some() || pInitSecurityInterfaceA.is_some() {
            let mut fnTableA: PSecurityFunctionTableA = core::ptr::null_mut();
            let mut fnTableW: PSecurityFunctionTableW = core::ptr::null_mut();
            let mut toAdd: ULONG = 0;
            let mut infoA: PSecPkgInfoA = core::ptr::null_mut();
            let mut infoW: PSecPkgInfoW = core::ptr::null_mut();
            let mut ret: SECURITY_STATUS = SEC_E_OK as SECURITY_STATUS;

            if let Some(f) = pInitSecurityInterfaceA {
                fnTableA = f();
            }
            if let Some(f) = pInitSecurityInterfaceW {
                fnTableW = f();
            }
            if !fnTableW.is_null() && (*fnTableW).EnumerateSecurityPackagesW.is_some() {
                if fnTableW != core::ptr::addr_of_mut!(securityFunctionTableW) {
                    ret = (*fnTableW).EnumerateSecurityPackagesW.unwrap()(
                        &mut toAdd,
                        &mut infoW,
                    );
                }
            } else if !fnTableA.is_null() && (*fnTableA).EnumerateSecurityPackagesA.is_some() {
                if fnTableA != core::ptr::addr_of_mut!(securityFunctionTableA) {
                    ret = (*fnTableA).EnumerateSecurityPackagesA.unwrap()(
                        &mut toAdd,
                        &mut infoA,
                    );
                }
            }
            if ret == SEC_E_OK as SECURITY_STATUS
                && toAdd > 0
                && (!infoW.is_null() || !infoA.is_null())
            {
                let provider = SECUR32_addProvider(
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    moduleName,
                );

                if !provider.is_null() {
                    SECUR32_addPackages(provider, toAdd, infoA, infoW);
                }
                if !infoW.is_null() {
                    (*fnTableW).FreeContextBuffer.unwrap()(infoW as *mut _);
                } else {
                    (*fnTableA).FreeContextBuffer.unwrap()(infoA as *mut _);
                }
            }
        }
        FreeLibrary(lib);
    }
}

unsafe fn SECUR32_initializeProviders() {
    let mut key: HKEY = core::ptr::null_mut();
    let mut apiRet: LSTATUS;

    const HKEY_LOCAL_MACHINE: HKEY = 0x80000002u32 as usize as HKEY;

    apiRet = RegOpenKeyExW(HKEY_LOCAL_MACHINE, securityProvidersKeyW.as_ptr(), 0,
        KEY_READ, &mut key);
    if apiRet == ERROR_SUCCESS as LSTATUS {
        let mut securityPkgNames: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut size: u32 = core::mem::size_of_val(&securityPkgNames) as u32;
        let mut type_: u32 = 0;

        apiRet = RegQueryValueExW(key, securityProvidersW.as_ptr(), core::ptr::null_mut(), &mut type_,
            securityPkgNames.as_mut_ptr() as PBYTE, &mut size);
        if apiRet == ERROR_SUCCESS as LSTATUS && type_ == REG_SZ {
            size = size / core::mem::size_of::<u16>() as u32;
            let base = securityPkgNames.as_mut_ptr();
            let end = base.add(size as usize);
            let mut ptr = base;
            while ptr < end {
                let mut comma = ptr;
                while *comma != 0 && *comma != b',' as u16 {
                    comma = comma.add(1);
                }
                if *comma == b',' as u16 {
                    *comma = 0;
                }
                while *ptr != 0 && isspace(*ptr as i32) != 0 && ptr < end {
                    ptr = ptr.add(1);
                }
                if *ptr != 0 {
                    _tryLoadProvider(ptr);
                }
                ptr = ptr.add(lstrlenW(ptr) as usize + 1);
            }
        }
        RegCloseKey(key);
    }

    SECUR32_initNTLMSP();
    SECUR32_initKerberosSP();
    SECUR32_initNegotiateSP();
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_findPackageW(packageName: PCWSTR) -> *mut SecurePackage {
    let mut ret: *mut SecurePackage = core::ptr::null_mut();
    let mut matched: BOOL = 0;

    if packageTable.is_null() {
        SECUR32_initializeProviders();
    }

    if !packageTable.is_null() && !packageName.is_null() {
        // C: LIST_FOR_EACH_ENTRY(ret, &packageTable->table, SecurePackage, entry)
        // is a for-loop header macro; the manual list walk below is its faithful
        // expansion (the translator emitted a phantom call for the macro name).
        let mut entry = (*packageTable).table.next;
        ret = core::ptr::null_mut();
        while entry != core::ptr::addr_of_mut!((*packageTable).table) {
            let cur = (entry as PBYTE).offset(
                -(core::mem::offset_of!(SecurePackage, entry) as isize),
            ) as *mut SecurePackage;
            matched = if lstrcmpiW((*cur).infoW.Name, packageName) == 0 { 1 } else { 0 };
            if matched != 0 {
                ret = cur;
                break;
            }
            entry = (*entry).next;
        }

        if matched == 0 {
            return core::ptr::null_mut();
        }

        if !(*ret).provider.is_null() && (*(*ret).provider).loaded == 0 {
            (*(*ret).provider).lib = LoadLibraryW((*(*ret).provider).moduleName);
            if !(*(*ret).provider).lib.is_null() {
                let pInitSecurityInterfaceW: INIT_SECURITY_INTERFACE_W =
                    core::mem::transmute(GetProcAddress(
                        (*(*ret).provider).lib,
                        SECURITY_ENTRYPOINT_ANSIW.as_ptr() as LPCSTR,
                    ));
                let pInitSecurityInterfaceA: INIT_SECURITY_INTERFACE_A =
                    core::mem::transmute(GetProcAddress(
                        (*(*ret).provider).lib,
                        SECURITY_ENTRYPOINT_ANSIA.as_ptr() as LPCSTR,
                    ));
                let mut fnTableA: PSecurityFunctionTableA = core::ptr::null_mut();
                let mut fnTableW: PSecurityFunctionTableW = core::ptr::null_mut();

                if let Some(f) = pInitSecurityInterfaceA {
                    fnTableA = f();
                }
                if let Some(f) = pInitSecurityInterfaceW {
                    fnTableW = f();
                }
                if fnTableA != core::ptr::addr_of!(securityFunctionTableA) as PSecurityFunctionTableA {
                    _makeFnTableA(
                        core::ptr::addr_of_mut!((*(*ret).provider).fnTableA),
                        fnTableA,
                        fnTableW,
                    );
                }
                if fnTableW != core::ptr::addr_of!(securityFunctionTableW) as PSecurityFunctionTableW {
                    _makeFnTableW(
                        core::ptr::addr_of_mut!((*(*ret).provider).fnTableW),
                        fnTableA,
                        fnTableW,
                    );
                }
                (*(*ret).provider).loaded = 1;
            } else {
                ret = core::ptr::null_mut();
            }
        }
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_findPackageA(packageName: PCSTR) -> *mut SecurePackage {
    let ret: *mut SecurePackage;

    if !packageName.is_null() {
        let mut package: _UNICODE_STRING = core::mem::zeroed();

        RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(package), packageName);
        ret = SECUR32_findPackageW(package.Buffer);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(package));
    } else {
        ret = core::ptr::null_mut();
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_freeProviders() {
    EnterCriticalSection(core::ptr::addr_of_mut!(cs));

    if !packageTable.is_null() {
        let mut package: *mut SecurePackage;
        let mut package_next: *mut SecurePackage;
        let head = core::ptr::addr_of_mut!((*packageTable).table);
        package = (*head).next as *mut SecurePackage;
        package_next = (*((*head).next)).next as *mut SecurePackage;
        while core::ptr::addr_of_mut!((*package).entry) != head {
            HeapFree(GetProcessHeap(), 0, (*package).infoW.Name as *mut _);
            HeapFree(GetProcessHeap(), 0, (*package).infoW.Comment as *mut _);
            HeapFree(GetProcessHeap(), 0, package as *mut _);
            package = package_next;
            package_next = (*package).entry.next as *mut SecurePackage;
        }

        HeapFree(GetProcessHeap(), 0, packageTable as *mut _);
        packageTable = core::ptr::null_mut();
    }

    if !providerTable.is_null() {
        let mut provider: *mut SecureProvider;
        let mut provider_next: *mut SecureProvider;
        let head = core::ptr::addr_of_mut!((*providerTable).table);
        provider = (*head).next as *mut SecureProvider;
        provider_next = (*((*head).next)).next as *mut SecureProvider;
        while core::ptr::addr_of_mut!((*provider).entry) != head {
            HeapFree(GetProcessHeap(), 0, (*provider).moduleName as *mut _);
            if !(*provider).lib.is_null() {
                FreeLibrary((*provider).lib);
            }
            HeapFree(GetProcessHeap(), 0, provider as *mut _);
            provider = provider_next;
            provider_next = (*provider).entry.next as *mut SecureProvider;
        }

        HeapFree(GetProcessHeap(), 0, providerTable as *mut _);
        providerTable = core::ptr::null_mut();
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(cs));
    DeleteCriticalSection(core::ptr::addr_of_mut!(cs));
}

#[no_mangle]
pub unsafe extern "system" fn FreeContextBuffer(pv: PVOID) -> SECURITY_STATUS {

    HeapFree(GetProcessHeap(), 0, pv);

    SEC_E_OK
}

#[no_mangle]
pub unsafe extern "system" fn EnumerateSecurityPackagesW(
    pcPackages: PULONG,
    ppPackageInfo: *mut PSecPkgInfoW,
) -> SECURITY_STATUS {
    let mut ret: SECURITY_STATUS = SEC_E_OK;

    if packageTable.is_null() {
        SECUR32_initializeProviders();
    }

    *pcPackages = 0;
    EnterCriticalSection(core::ptr::addr_of_mut!(cs));
    if !packageTable.is_null() {
        let mut bytesNeeded: usize =
            (*packageTable).numPackages as usize * core::mem::size_of::<SecPkgInfoW>();

        let mut package: *mut SecurePackage;
        package = ((*packageTable).table.next as *mut u8)
            .sub(core::mem::offset_of!(SecurePackage, entry))
            .cast::<SecurePackage>();
        while core::ptr::addr_of!((*package).entry) as *const _
            != core::ptr::addr_of!((*packageTable).table)
        {
            if !(*package).infoW.Name.is_null() {
                bytesNeeded += ((lstrlenW((*package).infoW.Name) + 1) as usize)
                    * core::mem::size_of::<WCHAR>();
            }
            if !(*package).infoW.Comment.is_null() {
                bytesNeeded += ((lstrlenW((*package).infoW.Comment) + 1) as usize)
                    * core::mem::size_of::<WCHAR>();
            }
            package = ((*package).entry.next as *mut u8)
            .sub(core::mem::offset_of!(SecurePackage, entry))
            .cast::<SecurePackage>();
        }

        if bytesNeeded != 0 {
            *ppPackageInfo =
                HeapAlloc(GetProcessHeap(), 0, bytesNeeded) as PSecPkgInfoW;
            if !(*ppPackageInfo).is_null() {
                let mut i: ULONG = 0;
                let mut nextString: PWSTR;

                *pcPackages = (*packageTable).numPackages;
                nextString = ((*ppPackageInfo as PBYTE).add(
                    (*packageTable).numPackages as usize
                        * core::mem::size_of::<SecPkgInfoW>(),
                )) as PWSTR;

                package = ((*packageTable).table.next as *mut u8)
            .sub(core::mem::offset_of!(SecurePackage, entry))
            .cast::<SecurePackage>();
                while core::ptr::addr_of!((*package).entry) as *const _
                    != core::ptr::addr_of!((*packageTable).table)
                {
                    let pkgInfo: PSecPkgInfoW = (*ppPackageInfo).add(i as usize);
                    i += 1;

                    *pkgInfo = (*package).infoW;
                    if !(*package).infoW.Name.is_null() {
                        (*pkgInfo).Name = nextString;
                        lstrcpyW(nextString, (*package).infoW.Name);
                        nextString = nextString.add((lstrlenW(nextString) + 1) as usize);
                    } else {
                        (*pkgInfo).Name = core::ptr::null_mut();
                    }
                    if !(*package).infoW.Comment.is_null() {
                        (*pkgInfo).Comment = nextString;
                        lstrcpyW(nextString, (*package).infoW.Comment);
                        nextString = nextString.add((lstrlenW(nextString) + 1) as usize);
                    } else {
                        (*pkgInfo).Comment = core::ptr::null_mut();
                    }

                    package = ((*package).entry.next as *mut u8)
            .sub(core::mem::offset_of!(SecurePackage, entry))
            .cast::<SecurePackage>();
                }
            } else {
                ret = SEC_E_INSUFFICIENT_MEMORY;
            }
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(cs));
    ret
}

unsafe fn thunk_PSecPkgInfoWToA(cPackages: ULONG, info: *const SecPkgInfoW) -> PSecPkgInfoA {
    let ret: PSecPkgInfoA;

    if !info.is_null() {
        let mut bytesNeeded: usize = cPackages as usize * core::mem::size_of::<SecPkgInfoA>();
        let mut i: ULONG;

        i = 0;
        while i < cPackages {
            if !(*info.add(i as usize)).Name.is_null() {
                bytesNeeded += WideCharToMultiByte(CP_ACP, 0, (*info.add(i as usize)).Name,
                    -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as usize;
            }
            if !(*info.add(i as usize)).Comment.is_null() {
                bytesNeeded += WideCharToMultiByte(CP_ACP, 0, (*info.add(i as usize)).Comment,
                    -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as usize;
            }
            i += 1;
        }
        ret = HeapAlloc(GetProcessHeap(), 0, bytesNeeded) as PSecPkgInfoA;
        if !ret.is_null() {
            let mut nextString: PSTR;

            nextString = (ret as PBYTE).add(cPackages as usize * core::mem::size_of::<SecPkgInfoA>()) as PSTR;
            i = 0;
            while i < cPackages {
                let pkgInfo: PSecPkgInfoA = ret.add(i as usize);
                let bytes: i32;

                memcpy(pkgInfo as *mut core::ffi::c_void, info.add(i as usize) as *const core::ffi::c_void,
                    core::mem::size_of::<SecPkgInfoA>());
                if !(*info.add(i as usize)).Name.is_null() {
                    (*pkgInfo).Name = nextString;
                    let bytes = WideCharToMultiByte(CP_ACP, 0, (*info.add(i as usize)).Name, -1,
                        core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());
                    WideCharToMultiByte(CP_ACP, 0, (*info.add(i as usize)).Name, -1,
                        (*pkgInfo).Name, bytes, core::ptr::null(), core::ptr::null_mut());
                    nextString = nextString.add(lstrlenA(nextString) as usize + 1);
                } else {
                    (*pkgInfo).Name = core::ptr::null_mut();
                }
                if !(*info.add(i as usize)).Comment.is_null() {
                    (*pkgInfo).Comment = nextString;
                    let bytes = WideCharToMultiByte(CP_ACP, 0, (*info.add(i as usize)).Comment, -1,
                        core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());
                    WideCharToMultiByte(CP_ACP, 0, (*info.add(i as usize)).Comment, -1,
                        (*pkgInfo).Comment, bytes, core::ptr::null(), core::ptr::null_mut());
                    nextString = nextString.add(lstrlenA(nextString) as usize + 1);
                } else {
                    (*pkgInfo).Comment = core::ptr::null_mut();
                }
                i += 1;
            }
        }
    } else {
        ret = core::ptr::null_mut();
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumerateSecurityPackagesA(
    pcPackages: PULONG,
    ppPackageInfo: *mut PSecPkgInfoA,
) -> SECURITY_STATUS {
    let mut info: PSecPkgInfoW = core::ptr::null_mut();

    let mut ret = EnumerateSecurityPackagesW(pcPackages, &mut info);
    if ret == SEC_E_OK && *pcPackages != 0 && !info.is_null() {
        *ppPackageInfo = thunk_PSecPkgInfoWToA(*pcPackages, info);
        if *pcPackages != 0 && (*ppPackageInfo).is_null() {
            *pcPackages = 0;
            ret = SEC_E_INSUFFICIENT_MEMORY;
        }
        FreeContextBuffer(info as PVOID);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetComputerObjectNameA(
    NameFormat: EXTENDED_NAME_FORMAT,
    lpNameBuffer: LPSTR,
    nSize: PULONG,
) -> BOOLEAN {

    let rc: BOOLEAN;
    let mut bufferW: LPWSTR = core::ptr::null_mut();
    let mut sizeW: ULONG = *nSize;

    if !lpNameBuffer.is_null() {
        bufferW = HeapAlloc(GetProcessHeap(), 0, sizeW as usize * core::mem::size_of::<u16>())
            as LPWSTR;
        if bufferW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0u8;
        }
    }

    rc = GetComputerObjectNameW(NameFormat, bufferW, &mut sizeW);

    if rc != 0 && !bufferW.is_null() {
        let len = WideCharToMultiByte(
            CP_ACP,
            0,
            bufferW,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as ULONG;
        WideCharToMultiByte(
            CP_ACP,
            0,
            bufferW,
            -1,
            lpNameBuffer,
            *nSize as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        *nSize = len;
    } else {
        *nSize = sizeW;
    }

    HeapFree(GetProcessHeap(), 0, bufferW as *mut core::ffi::c_void);
    rc
}

#[no_mangle]
pub unsafe extern "system" fn GetComputerObjectNameW(
    NameFormat: EXTENDED_NAME_FORMAT,
    lpNameBuffer: LPWSTR,
    nSize: PULONG,
) -> BOOLEAN {
    let mut policyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut objectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut domainInfo: PPOLICY_DNS_DOMAIN_INFO = core::ptr::null_mut();
    let mut ntStatus: NTSTATUS;
    let status: BOOLEAN;

    if NameFormat == NameUnknown {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0u8;
    }

    core::ptr::write_bytes(
        &mut objectAttributes as *mut _ as *mut u8,
        0,
        core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>(),
    );
    objectAttributes.Length = core::mem::size_of::<LSA_OBJECT_ATTRIBUTES>() as u32;

    ntStatus = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut objectAttributes,
        POLICY_VIEW_LOCAL_INFORMATION,
        &mut policyHandle,
    );
    if ntStatus != STATUS_SUCCESS {
        SetLastError(LsaNtStatusToWinError(ntStatus));
        return 0u8;
    }

    ntStatus = LsaQueryInformationPolicy(
        policyHandle,
        PolicyDnsDomainInformation,
        &mut domainInfo as *mut _ as *mut PVOID,
    );
    if ntStatus != STATUS_SUCCESS {
        SetLastError(LsaNtStatusToWinError(ntStatus));
        LsaClose(policyHandle);
        return 0u8;
    }

    if !(*domainInfo).Sid.is_null() {
        match NameFormat {
            NameSamCompatible => {
                let mut name: [u16; (MAX_COMPUTERNAME_LENGTH + 1) as usize] =
                    [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];
                let mut size: DWORD = name.len() as DWORD;
                if GetComputerNameW(name.as_mut_ptr(), &mut size) != 0 {
                    let len: DWORD = (*domainInfo).Name.Length as DWORD + size + 3;
                    if !lpNameBuffer.is_null() {
                        if *nSize < len {
                            *nSize = len;
                            SetLastError(ERROR_INSUFFICIENT_BUFFER);
                            status = 0u8;
                        } else {
                            let bs: [u16; 2] = [b'\\' as u16, 0];
                            let ds: [u16; 2] = [b'$' as u16, 0];
                            lstrcpyW(lpNameBuffer, (*domainInfo).Name.Buffer);
                            lstrcatW(lpNameBuffer, bs.as_ptr());
                            lstrcatW(lpNameBuffer, name.as_ptr());
                            lstrcatW(lpNameBuffer, ds.as_ptr());
                            status = 1u8;
                        }
                    } else {
                        *nSize = len;
                        status = 1u8;
                    }
                } else {
                    SetLastError(ERROR_INTERNAL_ERROR);
                    status = 0u8;
                }
            }
            NameFullyQualifiedDN
            | NameDisplay
            | NameUniqueId
            | NameCanonical
            | NameUserPrincipal
            | NameCanonicalEx
            | NameServicePrincipal
            | NameDnsDomain => {
                SetLastError(ERROR_CANT_ACCESS_DOMAIN_INFO);
                status = 0u8;
            }
            _ => {
                SetLastError(ERROR_INVALID_PARAMETER);
                status = 0u8;
            }
        }
    } else {
        SetLastError(ERROR_CANT_ACCESS_DOMAIN_INFO);
        status = 0u8;
    }

    LsaFreeMemory(domainInfo as PVOID);
    LsaClose(policyHandle);

    status
}

#[no_mangle]
pub unsafe extern "system" fn GetUserNameExA(
    NameFormat: EXTENDED_NAME_FORMAT,
    lpNameBuffer: LPSTR,
    nSize: PULONG,
) -> BOOLEAN {
    let mut rc: BOOLEAN;
    let mut bufferW: LPWSTR = core::ptr::null_mut();
    let mut sizeW: ULONG = *nSize;

    TRACE!(b"(%d %p %p)\n\0".as_ptr() as *const i8, NameFormat, lpNameBuffer, nSize);

    if !lpNameBuffer.is_null() {
        bufferW = HeapAlloc(
            GetProcessHeap(),
            0,
            (sizeW as usize) * core::mem::size_of::<u16>(),
        ) as LPWSTR;
        if bufferW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0u8;
        }
    }

    rc = GetUserNameExW(NameFormat, bufferW, &mut sizeW);
    if rc != 0 {
        let len: ULONG = WideCharToMultiByte(
            CP_ACP,
            0,
            bufferW,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as ULONG;
        if len <= *nSize {
            WideCharToMultiByte(
                CP_ACP,
                0,
                bufferW,
                -1,
                lpNameBuffer,
                *nSize as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            *nSize = len - 1;
        } else {
            *nSize = len;
            rc = 0u8;
            SetLastError(ERROR_MORE_DATA);
        }
    } else {
        *nSize = sizeW;
    }

    HeapFree(GetProcessHeap(), 0, bufferW as *mut core::ffi::c_void);
    rc
}

#[no_mangle]
pub unsafe extern "system" fn GetUserNameExW(
    NameFormat: EXTENDED_NAME_FORMAT,
    lpNameBuffer: LPWSTR,
    nSize: PULONG,
) -> BOOLEAN {
    match NameFormat {
        _EXTENDED_NAME_FORMAT_NameSamCompatible => {
            let mut samname: [u16; UNLEN as usize + 1 + MAX_COMPUTERNAME_LENGTH as usize + 1] =
                [0; UNLEN as usize + 1 + MAX_COMPUTERNAME_LENGTH as usize + 1];
            let mut len: DWORD;

            len = MAX_COMPUTERNAME_LENGTH + 1;
            if GetComputerNameW(samname.as_mut_ptr(), &mut len) != 0 {
                let mut out = samname.as_mut_ptr().add(lstrlenW(samname.as_ptr()) as usize);
                *out = '\\' as u16;
                out = out.add(1);
                len = UNLEN + 1;
                if GetUserNameW(out, &mut len) != 0 {
                    if (lstrlenW(samname.as_ptr()) as ULONG) < *nSize {
                        lstrcpyW(lpNameBuffer, samname.as_ptr());
                        *nSize = lstrlenW(samname.as_ptr()) as ULONG;
                        return 1u8;
                    }

                    SetLastError(ERROR_MORE_DATA);
                    *nSize = (lstrlenW(samname.as_ptr()) + 1) as ULONG;
                }
            }
            0u8
        }

        _EXTENDED_NAME_FORMAT_NameUnknown
        | _EXTENDED_NAME_FORMAT_NameFullyQualifiedDN
        | _EXTENDED_NAME_FORMAT_NameDisplay
        | _EXTENDED_NAME_FORMAT_NameUniqueId
        | _EXTENDED_NAME_FORMAT_NameCanonical
        | _EXTENDED_NAME_FORMAT_NameUserPrincipal
        | _EXTENDED_NAME_FORMAT_NameCanonicalEx
        | _EXTENDED_NAME_FORMAT_NameServicePrincipal
        | _EXTENDED_NAME_FORMAT_NameDnsDomain => {
            SetLastError(ERROR_NONE_MAPPED);
            0u8
        }

        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            0u8
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn TranslateNameA(
    lpAccountName: LPCSTR,
    AccountNameFormat: EXTENDED_NAME_FORMAT,
    DesiredNameFormat: EXTENDED_NAME_FORMAT,
    lpTranslatedName: LPSTR,
    nSize: PULONG,
) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn TranslateNameW(
    lpAccountName: LPCWSTR,
    AccountNameFormat: EXTENDED_NAME_FORMAT,
    DesiredNameFormat: EXTENDED_NAME_FORMAT,
    lpTranslatedName: LPWSTR,
    nSize: PULONG,
) -> BOOLEAN {
    0u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright (C) 2004 Juan Lang
//  *
//  * This file implements loading of SSP DLLs.
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "precomp.h"
//
// #include <assert.h>
// #include <lsass/lsass.h>
//
// #include <wine/debug.h>
// WINE_DEFAULT_DEBUG_CHANNEL(secur32);
//
// #define UNLEN 256
//
// typedef struct _SecurePackageTable
// {
//     DWORD numPackages;
//     DWORD numAllocated;
//     struct list table;
// } SecurePackageTable;
//
// typedef struct _SecureProviderTable
// {
//     DWORD numProviders;
//     DWORD numAllocated;
//     struct list table;
// } SecureProviderTable;
//
// static void SECUR32_initializeProviders(void);
// /* static */ void SECUR32_freeProviders(void);
//
// static CRITICAL_SECTION cs;
// static CRITICAL_SECTION_DEBUG cs_debug =
// {
//     0, 0, &cs,
//     { &cs_debug.ProcessLocksList, &cs_debug.ProcessLocksList },
//       0, 0, { (DWORD_PTR)(__FILE__ ": cs") }
// };
// static CRITICAL_SECTION cs = { &cs_debug, -1, 0, 0, 0, 0 };
// static SecurePackageTable *packageTable = NULL;
// static SecureProviderTable *providerTable = NULL;
//
// static SecurityFunctionTableA securityFunctionTableA = {
//     SECURITY_SUPPORT_PROVIDER_INTERFACE_VERSION,
//     EnumerateSecurityPackagesA,
//     QueryCredentialsAttributesA,
//     AcquireCredentialsHandleA,
//     FreeCredentialsHandle,
//     NULL, /* Reserved2 */
//     InitializeSecurityContextA,
//     AcceptSecurityContext,
//     CompleteAuthToken,
//     DeleteSecurityContext,
//     ApplyControlToken,
//     QueryContextAttributesA,
//     ImpersonateSecurityContext,
//     RevertSecurityContext,
//     MakeSignature,
//     VerifySignature,
//     FreeContextBuffer,
//     QuerySecurityPackageInfoA,
//     EncryptMessage, /* Reserved3 */
//     DecryptMessage, /* Reserved4 */
//     ExportSecurityContext,
//     ImportSecurityContextA,
//     AddCredentialsA,
//     NULL, /* Reserved8 */
//     QuerySecurityContextToken,
//     EncryptMessage,
//     DecryptMessage,
//     NULL
// };
//
// static SecurityFunctionTableW securityFunctionTableW = {
//     SECURITY_SUPPORT_PROVIDER_INTERFACE_VERSION,
//     EnumerateSecurityPackagesW,
//     QueryCredentialsAttributesW,
//     AcquireCredentialsHandleW,
//     FreeCredentialsHandle,
//     NULL, /* Reserved2 */
//     InitializeSecurityContextW,
//     AcceptSecurityContext,
//     CompleteAuthToken,
//     DeleteSecurityContext,
//     ApplyControlToken,
//     QueryContextAttributesW,
//     ImpersonateSecurityContext,
//     RevertSecurityContext,
//     MakeSignature,
//     VerifySignature,
//     FreeContextBuffer,
//     QuerySecurityPackageInfoW,
//     EncryptMessage, /* Reserved3 */
//     DecryptMessage, /* Reserved4 */
//     ExportSecurityContext,
//     ImportSecurityContextW,
//     AddCredentialsW,
//     NULL, /* Reserved8 */
//     QuerySecurityContextToken,
//     EncryptMessage,
//     DecryptMessage,
//     NULL
// };
//
// /***********************************************************************
//  *		InitSecurityInterfaceA (SECUR32.@)
//  */
// PSecurityFunctionTableA WINAPI InitSecurityInterfaceA(void)
// {
//     TRACE("InitSecurityInterfaceA() called\n");
//     return &securityFunctionTableA;
// }
//
// /***********************************************************************
//  *		InitSecurityInterfaceW (SECUR32.@)
//  */
// PSecurityFunctionTableW WINAPI InitSecurityInterfaceW(void)
// {
//     TRACE("InitSecurityInterfaceW() called\n");
//     return &securityFunctionTableW;
// }
//
// static PWSTR SECUR32_strdupW(PCWSTR str)
// {
//     PWSTR ret;
//
//     if (str)
//     {
//         ret = HeapAlloc(GetProcessHeap(), 0, (lstrlenW(str) + 1) * sizeof(WCHAR));
//         if (ret)
//             lstrcpyW(ret, str);
//     }
//     else
//         ret = NULL;
//     return ret;
// }
//
// PWSTR SECUR32_AllocWideFromMultiByte(PCSTR str)
// {
//     PWSTR ret;
//
//     if (str)
//     {
//         int charsNeeded = MultiByteToWideChar(CP_ACP, 0, str, -1, NULL, 0);
//
//         if (charsNeeded)
//         {
//             ret = HeapAlloc(GetProcessHeap(), 0, charsNeeded * sizeof(WCHAR));
//             if (ret)
//                 MultiByteToWideChar(CP_ACP, 0, str, -1, ret, charsNeeded);
//         }
//         else
//             ret = NULL;
//     }
//     else
//         ret = NULL;
//     return ret;
// }
//
// PSTR SECUR32_AllocMultiByteFromWide(PCWSTR str)
// {
//     PSTR ret;
//
//     if (str)
//     {
//         int charsNeeded = WideCharToMultiByte(CP_ACP, 0, str, -1, NULL, 0,
//          NULL, NULL);
//
//         if (charsNeeded)
//         {
//             ret = HeapAlloc(GetProcessHeap(), 0, charsNeeded);
//             if (ret)
//                 WideCharToMultiByte(CP_ACP, 0, str, -1, ret, charsNeeded,
//                  NULL, NULL);
//         }
//         else
//             ret = NULL;
//     }
//     else
//         ret = NULL;
//     return ret;
// }
//
// static void _makeFnTableA(PSecurityFunctionTableA fnTableA,
//  const SecurityFunctionTableA *inFnTableA,
//  const SecurityFunctionTableW *inFnTableW)
// {
//     if (fnTableA)
//     {
//         if (inFnTableA)
//         {
//             /* The size of the version 1 table is based on platform sdk's
//              * sspi.h, though the sample ssp also provided with platform sdk
//              * implies only functions through QuerySecurityPackageInfoA are
//              * implemented (yikes)
//              */
//             size_t tableSize = inFnTableA->dwVersion == 1 ?
//              (const BYTE *)&inFnTableA->SetContextAttributesA -
//              (const BYTE *)inFnTableA : sizeof(SecurityFunctionTableA);
//
//             memcpy(fnTableA, inFnTableA, tableSize);
//             /* override this, since we can do it internally anyway */
//             fnTableA->QuerySecurityPackageInfoA =
//              QuerySecurityPackageInfoA;
//         }
//         else if (inFnTableW)
//         {
//             /* functions with thunks */
//             if (inFnTableW->AcquireCredentialsHandleW)
//                 fnTableA->AcquireCredentialsHandleA =
//                  thunk_AcquireCredentialsHandleA;
//             if (inFnTableW->InitializeSecurityContextW)
//                 fnTableA->InitializeSecurityContextA =
//                  thunk_InitializeSecurityContextA;
//             if (inFnTableW->ImportSecurityContextW)
//                 fnTableA->ImportSecurityContextA =
//                  thunk_ImportSecurityContextA;
//             if (inFnTableW->AddCredentialsW)
//                 fnTableA->AddCredentialsA =
//                  thunk_AddCredentialsA;
//             if (inFnTableW->QueryCredentialsAttributesW)
//                 fnTableA->QueryCredentialsAttributesA =
//                  thunk_QueryCredentialsAttributesA;
//             if (inFnTableW->QueryContextAttributesW)
//                 fnTableA->QueryContextAttributesA =
//                  thunk_QueryContextAttributesA;
//             if (inFnTableW->SetContextAttributesW)
//                 fnTableA->SetContextAttributesA =
//                  thunk_SetContextAttributesA;
//             /* this can't be thunked, there's no extra param to know which
//              * package to forward to */
//             fnTableA->EnumerateSecurityPackagesA = NULL;
//             /* functions with no thunks needed */
//             fnTableA->AcceptSecurityContext = inFnTableW->AcceptSecurityContext;
//             fnTableA->CompleteAuthToken = inFnTableW->CompleteAuthToken;
//             fnTableA->DeleteSecurityContext = inFnTableW->DeleteSecurityContext;
//             fnTableA->ImpersonateSecurityContext =
//              inFnTableW->ImpersonateSecurityContext;
//             fnTableA->RevertSecurityContext = inFnTableW->RevertSecurityContext;
//             fnTableA->MakeSignature = inFnTableW->MakeSignature;
//             fnTableA->VerifySignature = inFnTableW->VerifySignature;
//             fnTableA->FreeContextBuffer = inFnTableW->FreeContextBuffer;
//             fnTableA->QuerySecurityPackageInfoA =
//              QuerySecurityPackageInfoA;
//             fnTableA->ExportSecurityContext =
//              inFnTableW->ExportSecurityContext;
//             fnTableA->QuerySecurityContextToken =
//              inFnTableW->QuerySecurityContextToken;
//             fnTableA->EncryptMessage = inFnTableW->EncryptMessage;
//             fnTableA->DecryptMessage = inFnTableW->DecryptMessage;
//         }
//     }
// }
//
// static void _makeFnTableW(PSecurityFunctionTableW fnTableW,
//  const SecurityFunctionTableA *inFnTableA,
//  const SecurityFunctionTableW *inFnTableW)
// {
//     if (fnTableW)
//     {
//         if (inFnTableW)
//         {
//             /* The size of the version 1 table is based on platform sdk's
//              * sspi.h, though the sample ssp also provided with platform sdk
//              * implies only functions through QuerySecurityPackageInfoA are
//              * implemented (yikes)
//              */
//             size_t tableSize = inFnTableW->dwVersion == 1 ?
//              (const BYTE *)&inFnTableW->SetContextAttributesW -
//              (const BYTE *)inFnTableW : sizeof(SecurityFunctionTableW);
//
//             memcpy(fnTableW, inFnTableW, tableSize);
//             /* override this, since we can do it internally anyway */
//             fnTableW->QuerySecurityPackageInfoW =
//              QuerySecurityPackageInfoW;
//         }
//         else if (inFnTableA)
//         {
//             /* functions with thunks */
//             if (inFnTableA->AcquireCredentialsHandleA)
//                 fnTableW->AcquireCredentialsHandleW =
//                  thunk_AcquireCredentialsHandleW;
//             if (inFnTableA->InitializeSecurityContextA)
//                 fnTableW->InitializeSecurityContextW =
//                  thunk_InitializeSecurityContextW;
//             if (inFnTableA->ImportSecurityContextA)
//                 fnTableW->ImportSecurityContextW =
//                  thunk_ImportSecurityContextW;
//             if (inFnTableA->AddCredentialsA)
//                 fnTableW->AddCredentialsW =
//                  thunk_AddCredentialsW;
//             if (inFnTableA->QueryCredentialsAttributesA)
//                 fnTableW->QueryCredentialsAttributesW =
//                  thunk_QueryCredentialsAttributesW;
//             if (inFnTableA->QueryContextAttributesA)
//                 fnTableW->QueryContextAttributesW =
//                  thunk_QueryContextAttributesW;
//             if (inFnTableA->SetContextAttributesA)
//                 fnTableW->SetContextAttributesW =
//                  thunk_SetContextAttributesW;
//             /* this can't be thunked, there's no extra param to know which
//              * package to forward to */
//             fnTableW->EnumerateSecurityPackagesW = NULL;
//             /* functions with no thunks needed */
//             fnTableW->AcceptSecurityContext = inFnTableA->AcceptSecurityContext;
//             fnTableW->CompleteAuthToken = inFnTableA->CompleteAuthToken;
//             fnTableW->DeleteSecurityContext = inFnTableA->DeleteSecurityContext;
//             fnTableW->ImpersonateSecurityContext =
//              inFnTableA->ImpersonateSecurityContext;
//             fnTableW->RevertSecurityContext = inFnTableA->RevertSecurityContext;
//             fnTableW->MakeSignature = inFnTableA->MakeSignature;
//             fnTableW->VerifySignature = inFnTableA->VerifySignature;
//             fnTableW->FreeContextBuffer = inFnTableA->FreeContextBuffer;
//             fnTableW->QuerySecurityPackageInfoW =
//              QuerySecurityPackageInfoW;
//             fnTableW->ExportSecurityContext =
//              inFnTableA->ExportSecurityContext;
//             fnTableW->QuerySecurityContextToken =
//              inFnTableA->QuerySecurityContextToken;
//             fnTableW->EncryptMessage = inFnTableA->EncryptMessage;
//             fnTableW->DecryptMessage = inFnTableA->DecryptMessage;
//         }
//     }
// }
//
// static void _copyPackageInfo(PSecPkgInfoW info, const SecPkgInfoA *inInfoA,
//  const SecPkgInfoW *inInfoW)
// {
//     if (info && (inInfoA || inInfoW))
//     {
//         /* odd, I know, but up until Name and Comment the structures are
//          * identical
//          */
//         memcpy(info, inInfoW ? inInfoW : (const SecPkgInfoW *)inInfoA, sizeof(*info));
//         if (inInfoW)
//         {
//             info->Name = SECUR32_strdupW(inInfoW->Name);
//             info->Comment = SECUR32_strdupW(inInfoW->Comment);
//         }
//         else
//         {
//             info->Name = SECUR32_AllocWideFromMultiByte(inInfoA->Name);
//             info->Comment = SECUR32_AllocWideFromMultiByte(inInfoA->Comment);
//         }
//     }
// }
//
// // static
// SecureProvider *SECUR32_addProvider(const SecurityFunctionTableA *fnTableA,
//  const SecurityFunctionTableW *fnTableW, PCWSTR moduleName)
// {
//     SecureProvider *ret;
//
//     EnterCriticalSection(&cs);
//
//     if (!providerTable)
//     {
//         providerTable = HeapAlloc(GetProcessHeap(), 0, sizeof(SecureProviderTable));
//         if (!providerTable)
//         {
//             LeaveCriticalSection(&cs);
//             return NULL;
//         }
//
//         list_init(&providerTable->table);
//     }
//
//     ret = HeapAlloc(GetProcessHeap(), 0, sizeof(SecureProvider));
//     if (!ret)
//     {
//         LeaveCriticalSection(&cs);
//         return NULL;
//     }
//
//     list_add_tail(&providerTable->table, &ret->entry);
//     ret->lib = NULL;
//
//     if (fnTableA || fnTableW)
//     {
//         ret->moduleName = moduleName ? SECUR32_strdupW(moduleName) : NULL;
//         _makeFnTableA(&ret->fnTableA, fnTableA, fnTableW);
//         _makeFnTableW(&ret->fnTableW, fnTableA, fnTableW);
//         ret->loaded = !moduleName;
//     }
//     else
//     {
//         ret->moduleName = SECUR32_strdupW(moduleName);
//         ret->loaded = FALSE;
//     }
//     
//     LeaveCriticalSection(&cs);
//     return ret;
// }
//
// // static
// void SECUR32_addPackages(SecureProvider *provider, ULONG toAdd,
//  const SecPkgInfoA *infoA, const SecPkgInfoW *infoW)
// {
//     ULONG i;
//
//     assert(provider);
//     assert(infoA || infoW);
//
//     EnterCriticalSection(&cs);
//
//     if (!packageTable)
//     {
//         packageTable = HeapAlloc(GetProcessHeap(), 0, sizeof(SecurePackageTable));
//         if (!packageTable)
//         {
//             LeaveCriticalSection(&cs);
//             return;
//         }
//
//         packageTable->numPackages = 0;
//         list_init(&packageTable->table);
//     }
//         
//     for (i = 0; i < toAdd; i++)
//     {
//         SecurePackage *package = HeapAlloc(GetProcessHeap(), 0, sizeof(SecurePackage));
//         if (!package)
//             continue;
//
//         list_add_tail(&packageTable->table, &package->entry);
//
//         package->provider = provider;
//         _copyPackageInfo(&package->infoW,
//             infoA ? &infoA[i] : NULL,
//             infoW ? &infoW[i] : NULL);
//     }
//     packageTable->numPackages += toAdd;
//
//     LeaveCriticalSection(&cs);
// }
//
// static void _tryLoadProvider(PWSTR moduleName)
// {
//     HMODULE lib = LoadLibraryW(moduleName);
//
//     if (lib)
//     {
//         INIT_SECURITY_INTERFACE_W pInitSecurityInterfaceW =
//          (INIT_SECURITY_INTERFACE_W)GetProcAddress(lib,
//          SECURITY_ENTRYPOINT_ANSIW.as_ptr() as LPCSTR);
//         INIT_SECURITY_INTERFACE_A pInitSecurityInterfaceA =
//          (INIT_SECURITY_INTERFACE_A)GetProcAddress(lib,
//          SECURITY_ENTRYPOINT_ANSIA.as_ptr() as LPCSTR);
//
//         TRACE("loaded %s, InitSecurityInterfaceA is %p, InitSecurityInterfaceW is %p\n",
//          debugstr_w(moduleName), pInitSecurityInterfaceA,
//          pInitSecurityInterfaceW);
//         if (pInitSecurityInterfaceW || pInitSecurityInterfaceA)
//         {
//             PSecurityFunctionTableA fnTableA = NULL;
//             PSecurityFunctionTableW fnTableW = NULL;
//             ULONG toAdd = 0;
//             PSecPkgInfoA infoA = NULL;
//             PSecPkgInfoW infoW = NULL;
//             SECURITY_STATUS ret = SEC_E_OK;
//
//             if (pInitSecurityInterfaceA)
//                 fnTableA = pInitSecurityInterfaceA();
//             if (pInitSecurityInterfaceW)
//                 fnTableW = pInitSecurityInterfaceW();
//             if (fnTableW && fnTableW->EnumerateSecurityPackagesW)
//             {
//                 if (fnTableW != &securityFunctionTableW)
//                     ret = fnTableW->EnumerateSecurityPackagesW(&toAdd, &infoW);
//                 else
//                     TRACE("%s has built-in providers, skip adding\n", debugstr_w(moduleName));
//             }
//             else if (fnTableA && fnTableA->EnumerateSecurityPackagesA)
//             {
//                 if (fnTableA != &securityFunctionTableA)
//                     ret = fnTableA->EnumerateSecurityPackagesA(&toAdd, &infoA);
//                 else
//                     TRACE("%s has built-in providers, skip adding\n", debugstr_w(moduleName));
//             }
//             if (ret == SEC_E_OK && toAdd > 0 && (infoW || infoA))
//             {
//                 SecureProvider *provider = SECUR32_addProvider(NULL, NULL,
//                  moduleName);
//
//                 if (provider)
//                     SECUR32_addPackages(provider, toAdd, infoA, infoW);
//                 if (infoW)
//                     fnTableW->FreeContextBuffer(infoW);
//                 else
//                     fnTableA->FreeContextBuffer(infoA);
//             }
//         }
//         FreeLibrary(lib);
//     }
//     else
//         WARN("failed to load %s\n", debugstr_w(moduleName));
// }
//
// static const WCHAR securityProvidersKeyW[] = {
//  'S','Y','S','T','E','M','\\','C','u','r','r','e','n','t','C','o','n','t','r',
//  'o','l','S','e','t','\\','C','o','n','t','r','o','l','\\','S','e','c','u','r',
//  'i','t','y','P','r','o','v','i','d','e','r','s','\0'
//  };
// static const WCHAR securityProvidersW[] = {
//  'S','e','c','u','r','i','t','y','P','r','o','v','i','d','e','r','s',0
//  };
//
// static void SECUR32_initializeProviders(void)
// {
//     HKEY key;
//     LSTATUS apiRet;
//
//     /* Now load providers from registry */
//     apiRet = RegOpenKeyExW(HKEY_LOCAL_MACHINE, securityProvidersKeyW, 0,
//      KEY_READ, &key);
//     if (apiRet == ERROR_SUCCESS)
//     {
//         WCHAR securityPkgNames[MAX_PATH]; /* arbitrary len */
//         DWORD size = sizeof(securityPkgNames), type;
//
//         apiRet = RegQueryValueExW(key, securityProvidersW, NULL, &type,
//          (PBYTE)securityPkgNames, &size);
//         if (apiRet == ERROR_SUCCESS && type == REG_SZ)
//         {
//             WCHAR *ptr;
//
//             size = size / sizeof(WCHAR);
//             for (ptr = securityPkgNames;
//               ptr < securityPkgNames + size; )
//             {
//                 WCHAR *comma;
//
//                 for (comma = ptr; *comma && *comma != ','; comma++)
//                     ;
//                 if (*comma == ',')
//                     *comma = '\0';
//                 for (; *ptr && isspace(*ptr) && ptr < securityPkgNames + size;
//                  ptr++)
//                     ;
//                 if (*ptr)
//                     _tryLoadProvider(ptr);
//                 ptr += lstrlenW(ptr) + 1;
//             }
//         }
//         RegCloseKey(key);
//     }
//
//     /* Now load the built-in providers (in Wine, this is done before the registry loading) */
// #ifdef __REACTOS__
// /// FIXME: Interim Wine code until we get Samuel's rewrite!
//     /* First load built-in providers */
//     SECUR32_initNTLMSP();
//     SECUR32_initKerberosSP();
//     /* Load the Negotiate provider last so apps stumble over the working NTLM
//      * provider first. Attempting to fix bug #16905 while keeping the
//      * application reported on wine-users on 2006-09-12 working. */
//     SECUR32_initNegotiateSP();
// #endif
//
// }
//
// SecurePackage *SECUR32_findPackageW(PCWSTR packageName)
// {
//     SecurePackage *ret = NULL;
//     BOOL matched = FALSE;
//
// #ifdef __REACTOS__
//     if (!packageTable)
//         SECUR32_initializeProviders();
// #endif
//
//     if (packageTable && packageName)
//     {
//         LIST_FOR_EACH_ENTRY(ret, &packageTable->table, SecurePackage, entry)
//         {
//             matched = !lstrcmpiW(ret->infoW.Name, packageName);
// 	    if (matched)
// 		break;
//         }
//         
// 	if (!matched)
// 		return NULL;
//
// 	if (ret->provider && !ret->provider->loaded)
//         {
//             ret->provider->lib = LoadLibraryW(ret->provider->moduleName);
//             if (ret->provider->lib)
//             {
//                 INIT_SECURITY_INTERFACE_W pInitSecurityInterfaceW =
//                  (INIT_SECURITY_INTERFACE_W)GetProcAddress(ret->provider->lib,
//                  SECURITY_ENTRYPOINT_ANSIW.as_ptr() as LPCSTR);
//                 INIT_SECURITY_INTERFACE_A pInitSecurityInterfaceA =
//                  (INIT_SECURITY_INTERFACE_A)GetProcAddress(ret->provider->lib,
//                  SECURITY_ENTRYPOINT_ANSIA.as_ptr() as LPCSTR);
//                 PSecurityFunctionTableA fnTableA = NULL;
//                 PSecurityFunctionTableW fnTableW = NULL;
//
//                 if (pInitSecurityInterfaceA)
//                     fnTableA = pInitSecurityInterfaceA();
//                 if (pInitSecurityInterfaceW)
//                     fnTableW = pInitSecurityInterfaceW();
//                 /* don't update built-in SecurityFunctionTable */
//                 if (fnTableA != &securityFunctionTableA)
//                     _makeFnTableA(&ret->provider->fnTableA, fnTableA, fnTableW);
//                 if (fnTableW != &securityFunctionTableW)
//                     _makeFnTableW(&ret->provider->fnTableW, fnTableA, fnTableW);
//                 ret->provider->loaded = TRUE;
//             }
//             else
//                 ret = NULL;
//         }
//     }
//     return ret;
// }
//
// SecurePackage *SECUR32_findPackageA(PCSTR packageName)
// {
//     SecurePackage *ret;
//
//     if (packageName)
//     {
//         UNICODE_STRING package;
//
//         RtlCreateUnicodeStringFromAsciiz(&package, packageName);
//         ret = SECUR32_findPackageW(package.Buffer);
//         RtlFreeUnicodeString(&package);
//     }
//     else
//         ret = NULL;
//     return ret;
// }
//
// /* static */ void SECUR32_freeProviders(void)
// {
//     TRACE("\n");
//     EnterCriticalSection(&cs);
//
// #ifndef __REACTOS__
//     SECUR32_deinitSchannelSP();
// #endif
//
//     if (packageTable)
//     {
//         SecurePackage *package, *package_next;
//         LIST_FOR_EACH_ENTRY_SAFE(package, package_next, &packageTable->table,
//                                  SecurePackage, entry)
//         {
//             HeapFree(GetProcessHeap(), 0, package->infoW.Name);
//             HeapFree(GetProcessHeap(), 0, package->infoW.Comment);
//             HeapFree(GetProcessHeap(), 0, package);
//         }
//
//         HeapFree(GetProcessHeap(), 0, packageTable);
//         packageTable = NULL;
//     }
//
//     if (providerTable)
//     {
//         SecureProvider *provider, *provider_next;
//         LIST_FOR_EACH_ENTRY_SAFE(provider, provider_next, &providerTable->table,
//                                  SecureProvider, entry)
//         {
//             HeapFree(GetProcessHeap(), 0, provider->moduleName);
//             if (provider->lib)
//                 FreeLibrary(provider->lib);
//             HeapFree(GetProcessHeap(), 0, provider);
//         }
//
//         HeapFree(GetProcessHeap(), 0, providerTable);
//         providerTable = NULL;
//     }
//
//     LeaveCriticalSection(&cs);
//     DeleteCriticalSection(&cs);
// }
//
// /***********************************************************************
//  *		FreeContextBuffer (SECUR32.@)
//  *
//  * Doh--if pv was allocated by a crypto package, this may not be correct.
//  * The sample ssp seems to use LocalAlloc/LocalFee, but there doesn't seem to
//  * be any guarantee, nor is there an alloc function in secur32.
//  */
// SECURITY_STATUS WINAPI FreeContextBuffer(PVOID pv)
// {
//     HeapFree(GetProcessHeap(), 0, pv);
//
//     return SEC_E_OK;
// }
//
// /***********************************************************************
//  *		EnumerateSecurityPackagesW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI EnumerateSecurityPackagesW(PULONG pcPackages,
//  PSecPkgInfoW *ppPackageInfo)
// {
//     SECURITY_STATUS ret = SEC_E_OK;
//
//     TRACE("(%p, %p)\n", pcPackages, ppPackageInfo);
//
// #ifdef __REACTOS__
//     if (!packageTable)
//         SECUR32_initializeProviders();
// #endif
//
//     /* windows just crashes if pcPackages or ppPackageInfo is NULL, so will I */
//     *pcPackages = 0;
//     EnterCriticalSection(&cs);
//     if (packageTable)
//     {
//         SecurePackage *package;
//         size_t bytesNeeded;
//
//         bytesNeeded = packageTable->numPackages * sizeof(SecPkgInfoW);
//         LIST_FOR_EACH_ENTRY(package, &packageTable->table, SecurePackage, entry)
//         {
//             if (package->infoW.Name)
//                 bytesNeeded += (lstrlenW(package->infoW.Name) + 1) * sizeof(WCHAR);
//             if (package->infoW.Comment)
//                 bytesNeeded += (lstrlenW(package->infoW.Comment) + 1) * sizeof(WCHAR);
//         }
//         if (bytesNeeded)
//         {
//             *ppPackageInfo = HeapAlloc(GetProcessHeap(), 0, bytesNeeded);
//             if (*ppPackageInfo)
//             {
//                 ULONG i = 0;
//                 PWSTR nextString;
//
//                 *pcPackages = packageTable->numPackages;
//                 nextString = (PWSTR)((PBYTE)*ppPackageInfo +
//                  packageTable->numPackages * sizeof(SecPkgInfoW));
//                 LIST_FOR_EACH_ENTRY(package, &packageTable->table, SecurePackage, entry)
//                 {
//                     PSecPkgInfoW pkgInfo = *ppPackageInfo + i++;
//
//                     *pkgInfo = package->infoW;
//                     if (package->infoW.Name)
//                     {
//                         TRACE("Name[%d] = %s\n", i - 1, debugstr_w(package->infoW.Name));
//                         pkgInfo->Name = nextString;
//                         lstrcpyW(nextString, package->infoW.Name);
//                         nextString += lstrlenW(nextString) + 1;
//                     }
//                     else
//                         pkgInfo->Name = NULL;
//                     if (package->infoW.Comment)
//                     {
//                         TRACE("Comment[%d] = %s\n", i - 1, debugstr_w(package->infoW.Comment));
//                         pkgInfo->Comment = nextString;
//                         lstrcpyW(nextString, package->infoW.Comment);
//                         nextString += lstrlenW(nextString) + 1;
//                     }
//                     else
//                         pkgInfo->Comment = NULL;
//                 }
//             }
//             else
//                 ret = SEC_E_INSUFFICIENT_MEMORY;
//         }
//     }
//     LeaveCriticalSection(&cs);
//     TRACE("<-- 0x%08x\n", ret);
//     return ret;
// }
//
// /* Converts info (which is assumed to be an array of cPackages SecPkgInfoW
//  * structures) into an array of SecPkgInfoA structures, which it returns.
//  */
// static PSecPkgInfoA thunk_PSecPkgInfoWToA(ULONG cPackages,
//  const SecPkgInfoW *info)
// {
//     PSecPkgInfoA ret;
//
//     if (info)
//     {
//         size_t bytesNeeded = cPackages * sizeof(SecPkgInfoA);
//         ULONG i;
//
//         for (i = 0; i < cPackages; i++)
//         {
//             if (info[i].Name)
//                 bytesNeeded += WideCharToMultiByte(CP_ACP, 0, info[i].Name,
//                  -1, NULL, 0, NULL, NULL);
//             if (info[i].Comment)
//                 bytesNeeded += WideCharToMultiByte(CP_ACP, 0, info[i].Comment,
//                  -1, NULL, 0, NULL, NULL);
//         }
//         ret = HeapAlloc(GetProcessHeap(), 0, bytesNeeded);
//         if (ret)
//         {
//             PSTR nextString;
//
//             nextString = (PSTR)((PBYTE)ret + cPackages * sizeof(SecPkgInfoA));
//             for (i = 0; i < cPackages; i++)
//             {
//                 PSecPkgInfoA pkgInfo = ret + i;
//                 int bytes;
//
//                 memcpy(pkgInfo, &info[i], sizeof(SecPkgInfoA));
//                 if (info[i].Name)
//                 {
//                     pkgInfo->Name = nextString;
//                     /* just repeat back to WideCharToMultiByte how many bytes
//                      * it requires, since we asked it earlier
//                      */
//                     bytes = WideCharToMultiByte(CP_ACP, 0, info[i].Name, -1,
//                      NULL, 0, NULL, NULL);
//                     WideCharToMultiByte(CP_ACP, 0, info[i].Name, -1,
//                      pkgInfo->Name, bytes, NULL, NULL);
//                     nextString += lstrlenA(nextString) + 1;
//                 }
//                 else
//                     pkgInfo->Name = NULL;
//                 if (info[i].Comment)
//                 {
//                     pkgInfo->Comment = nextString;
//                     /* just repeat back to WideCharToMultiByte how many bytes
//                      * it requires, since we asked it earlier
//                      */
//                     bytes = WideCharToMultiByte(CP_ACP, 0, info[i].Comment, -1,
//                      NULL, 0, NULL, NULL);
//                     WideCharToMultiByte(CP_ACP, 0, info[i].Comment, -1,
//                      pkgInfo->Comment, bytes, NULL, NULL);
//                     nextString += lstrlenA(nextString) + 1;
//                 }
//                 else
//                     pkgInfo->Comment = NULL;
//             }
//         }
//     }
//     else
//         ret = NULL;
//     return ret;
// }
//
// /***********************************************************************
//  *		EnumerateSecurityPackagesA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI EnumerateSecurityPackagesA(PULONG pcPackages,
//  PSecPkgInfoA *ppPackageInfo)
// {
//     SECURITY_STATUS ret;
//     PSecPkgInfoW info;
//
//     ret = EnumerateSecurityPackagesW(pcPackages, &info);
//     if (ret == SEC_E_OK && *pcPackages && info)
//     {
//         *ppPackageInfo = thunk_PSecPkgInfoWToA(*pcPackages, info);
//         if (*pcPackages && !*ppPackageInfo)
//         {
//             *pcPackages = 0;
//             ret = SEC_E_INSUFFICIENT_MEMORY;
//         }
//         FreeContextBuffer(info);
//     }
//     return ret;
// }
//
// /***********************************************************************
//  *		GetComputerObjectNameA (SECUR32.@)
//  *
//  * Get the local computer's name using the format specified.
//  *
//  * PARAMS
//  *  NameFormat   [I] The format for the name.
//  *  lpNameBuffer [O] Pointer to buffer to receive the name.
//  *  nSize        [I/O] Size in characters of buffer.
//  *
//  * RETURNS
//  *  TRUE  If the name was written to lpNameBuffer.
//  *  FALSE If the name couldn't be written.
//  *
//  * NOTES
//  *  If lpNameBuffer is NULL, then the size of the buffer needed to hold the
//  *  name will be returned in *nSize.
//  *
//  *  nSize returns the number of characters written when lpNameBuffer is not
//  *  NULL or the size of the buffer needed to hold the name when the buffer
//  *  is too short or lpNameBuffer is NULL.
//  * 
//  *  It the buffer is too short, ERROR_INSUFFICIENT_BUFFER error will be set.
//  */
// BOOLEAN WINAPI GetComputerObjectNameA(
//   EXTENDED_NAME_FORMAT NameFormat, LPSTR lpNameBuffer, PULONG nSize)
// {
//     BOOLEAN rc;
//     LPWSTR bufferW = NULL;
//     ULONG sizeW = *nSize;
//     TRACE("(%d %p %p)\n", NameFormat, lpNameBuffer, nSize);
//     if (lpNameBuffer) {
//         bufferW = HeapAlloc(GetProcessHeap(), 0, sizeW * sizeof(WCHAR));
//         if (bufferW == NULL) {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//     rc = GetComputerObjectNameW(NameFormat, bufferW, &sizeW);
//     if (rc && bufferW) {
//         ULONG len = WideCharToMultiByte(CP_ACP, 0, bufferW, -1, NULL, 0, NULL, NULL);
//         WideCharToMultiByte(CP_ACP, 0, bufferW, -1, lpNameBuffer, *nSize, NULL, NULL);
//         *nSize = len;
//     }
//     else
//         *nSize = sizeW;
//     HeapFree(GetProcessHeap(), 0, bufferW);
//     return rc;
// }
//
// /***********************************************************************
//  *		GetComputerObjectNameW (SECUR32.@)
//  */
// BOOLEAN WINAPI GetComputerObjectNameW(
//   EXTENDED_NAME_FORMAT NameFormat, LPWSTR lpNameBuffer, PULONG nSize)
// {
//     LSA_HANDLE policyHandle;
//     LSA_OBJECT_ATTRIBUTES objectAttributes;
//     PPOLICY_DNS_DOMAIN_INFO domainInfo;
//     NTSTATUS ntStatus;
//     BOOLEAN status;
//     TRACE("(%d %p %p)\n", NameFormat, lpNameBuffer, nSize);
//
//     if (NameFormat == NameUnknown)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     ZeroMemory(&objectAttributes, sizeof(objectAttributes));
//     objectAttributes.Length = sizeof(objectAttributes);
//
//     ntStatus = LsaOpenPolicy(NULL, &objectAttributes,
//                              POLICY_VIEW_LOCAL_INFORMATION,
//                              &policyHandle);
//     if (ntStatus != STATUS_SUCCESS)
//     {
//         SetLastError(LsaNtStatusToWinError(ntStatus));
//         WARN("LsaOpenPolicy failed with NT status %u\n", GetLastError());
//         return FALSE;
//     }
//
//     ntStatus = LsaQueryInformationPolicy(policyHandle,
//                                          PolicyDnsDomainInformation,
//                                          (PVOID *)&domainInfo);
//     if (ntStatus != STATUS_SUCCESS)
//     {
//         SetLastError(LsaNtStatusToWinError(ntStatus));
//         WARN("LsaQueryInformationPolicy failed with NT status %u\n",
//              GetLastError());
//         LsaClose(policyHandle);
//         return FALSE;
//     }
//
//     if (domainInfo->Sid)
//     {
//         switch (NameFormat)
//         {
//         case NameSamCompatible:
//             {
//                 WCHAR name[MAX_COMPUTERNAME_LENGTH + 1];
//                 DWORD size = sizeof(name)/sizeof(name[0]);
//                 if (GetComputerNameW(name, &size))
//                 {
//                     DWORD len = domainInfo->Name.Length + size + 3;
//                     if (lpNameBuffer)
//                     {
//                         if (*nSize < len)
//                         {
//                             *nSize = len;
//                             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                             status = FALSE;
//                         }
//                         else
//                         {
//                             WCHAR bs[] = { '\\', 0 };
//                             WCHAR ds[] = { '$', 0 };
//                             lstrcpyW(lpNameBuffer, domainInfo->Name.Buffer);
//                             lstrcatW(lpNameBuffer, bs);
//                             lstrcatW(lpNameBuffer, name);
//                             lstrcatW(lpNameBuffer, ds);
//                             status = TRUE;
//                         }
//                     }
//                     else	/* just requesting length required */
//                     {
//                         *nSize = len;
//                         status = TRUE;
//                     }
//                 }
//                 else
//                 {
//                     SetLastError(ERROR_INTERNAL_ERROR);
//                     status = FALSE;
//                 }
//             }
//             break;
//         case NameFullyQualifiedDN:
//         case NameDisplay:
//         case NameUniqueId:
//         case NameCanonical:
//         case NameUserPrincipal:
//         case NameCanonicalEx:
//         case NameServicePrincipal:
//         case NameDnsDomain:
//             FIXME("NameFormat %d not implemented\n", NameFormat);
//             SetLastError(ERROR_CANT_ACCESS_DOMAIN_INFO);
//             status = FALSE;
//             break;
//         default:
//             SetLastError(ERROR_INVALID_PARAMETER);
//             status = FALSE;
//         }
//     }
//     else
//     {
//         SetLastError(ERROR_CANT_ACCESS_DOMAIN_INFO);
//         status = FALSE;
//     }
//
//     LsaFreeMemory(domainInfo);
//     LsaClose(policyHandle);
//
//     return status;
// }
//
// /***********************************************************************
//  *		GetUserNameExA (SECUR32.@)
//  */
// BOOLEAN WINAPI GetUserNameExA(
//   EXTENDED_NAME_FORMAT NameFormat, LPSTR lpNameBuffer, PULONG nSize)
// {
//     BOOLEAN rc;
//     LPWSTR bufferW = NULL;
//     ULONG sizeW = *nSize;
//     TRACE("(%d %p %p)\n", NameFormat, lpNameBuffer, nSize);
//     if (lpNameBuffer) {
//         bufferW = HeapAlloc(GetProcessHeap(), 0, sizeW * sizeof(WCHAR));
//         if (bufferW == NULL) {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//     rc = GetUserNameExW(NameFormat, bufferW, &sizeW);
//     if (rc) {
//         ULONG len = WideCharToMultiByte(CP_ACP, 0, bufferW, -1, NULL, 0, NULL, NULL);
//         if (len <= *nSize)
//         {
//             WideCharToMultiByte(CP_ACP, 0, bufferW, -1, lpNameBuffer, *nSize, NULL, NULL);
//             *nSize = len - 1;
//         }
//         else
//         {
//             *nSize = len;
//             rc = FALSE;
//             SetLastError(ERROR_MORE_DATA);
//         }
//     }
//     else
//         *nSize = sizeW;
//     HeapFree(GetProcessHeap(), 0, bufferW);
//     return rc;
// }
//
// BOOLEAN WINAPI GetUserNameExW(
//   EXTENDED_NAME_FORMAT NameFormat, LPWSTR lpNameBuffer, PULONG nSize)
// {
//     TRACE("(%d %p %p)\n", NameFormat, lpNameBuffer, nSize);
//
//     switch (NameFormat)
//     {
//     case NameSamCompatible:
//         {
//             WCHAR samname[UNLEN + 1 + MAX_COMPUTERNAME_LENGTH + 1];
//             LPWSTR out;
//             DWORD len;
//
//             /* This assumes the current user is always a local account */
//             len = MAX_COMPUTERNAME_LENGTH + 1;
//             if (GetComputerNameW(samname, &len))
//             {
//                 out = samname + lstrlenW(samname);
//                 *out++ = '\\';
//                 len = UNLEN + 1;
//                 if (GetUserNameW(out, &len))
//                 {
//                     if (lstrlenW(samname) < *nSize)
//                     {
//                         lstrcpyW(lpNameBuffer, samname);
//                         *nSize = lstrlenW(samname);
//                         return TRUE;
//                     }
//
//                     SetLastError(ERROR_MORE_DATA);
//                     *nSize = lstrlenW(samname) + 1;
//                 }
//             }
//             return FALSE;
//         }
//
//     case NameUnknown:
//     case NameFullyQualifiedDN:
//     case NameDisplay:
//     case NameUniqueId:
//     case NameCanonical:
//     case NameUserPrincipal:
//     case NameCanonicalEx:
//     case NameServicePrincipal:
//     case NameDnsDomain:
//         SetLastError(ERROR_NONE_MAPPED);
//         return FALSE;
//
//     default:
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
// }
//
// BOOLEAN WINAPI TranslateNameA(
//   LPCSTR lpAccountName, EXTENDED_NAME_FORMAT AccountNameFormat,
//   EXTENDED_NAME_FORMAT DesiredNameFormat, LPSTR lpTranslatedName,
//   PULONG nSize)
// {
//     FIXME("%p %d %d %p %p\n", lpAccountName, AccountNameFormat,
//           DesiredNameFormat, lpTranslatedName, nSize);
//     return FALSE;
// }
//
// BOOLEAN WINAPI TranslateNameW(
//   LPCWSTR lpAccountName, EXTENDED_NAME_FORMAT AccountNameFormat,
//   EXTENDED_NAME_FORMAT DesiredNameFormat, LPWSTR lpTranslatedName,
//   PULONG nSize)
// {
//     FIXME("%p %d %d %p %p\n", lpAccountName, AccountNameFormat,
//           DesiredNameFormat, lpTranslatedName, nSize);
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
