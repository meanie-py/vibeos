use ntkrnl_sys::driver_ddk::*;
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, ObReferenceObjectByHandle, PsGetCurrentThreadWin32Thread, PsGetThreadId, RtlAppendUnicodeToString, RtlCopyUnicodeString, RtlIntegerToUnicodeString, RtlUnicodeStringToInteger, ZwClose};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use ntkrnl_sys::interlocked::{InterlockedDecrement, InterlockedIncrement};
use win32k_sys::win32k::MAX_PATH;   // disambiguate the two glob-imported MAX_PATH
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
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


extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

// gspklBaseLayout — kbdlayout.c-owned global (win32k-sys only declares it extern).
// kbdlayout.rs is the defining site now; #[no_mangle] shadows the glob extern import.
#[no_mangle]
pub static mut gspklBaseLayout: PKL = core::ptr::null_mut();
// gSystemFS — kbdlayout.c-owned DWORD global (win32k-sys declares it extern).
#[no_mangle]
pub static mut gSystemFS: DWORD = 0;
// gSystemCPCharSet — kbdlayout.c-owned UINT global.
#[no_mangle]
pub static mut gSystemCPCharSet: UINT = 0;

// USERTAG_IME 'misU' (tags.h) — little-endian multichar.
const USERTAG_IME: u32 = u32::from_le_bytes(*b"Usim");
// INVALID_HANDLE_VALUE = (HANDLE)-1.
const INVALID_HANDLE_VALUE: HANDLE = !0usize as HANDLE;

// RegOpenKey / RegQueryValue — win32k registry helpers (reg.c exports).
extern "C" {
    // W32kGetDefaultKeyLayout — same file (defined below); forward ref not needed but
    // declared inline. (Defined in this module at fn W32kGetDefaultKeyLayout.)
}

// RtlStringCbCatW — strsafe.h inline (no kernel symbol). Appends a wide string.
#[inline(always)]
unsafe fn RtlStringCbCatW(pszDest: *mut u16, cbDest: usize, pszSrc: *const u16) -> NTSTATUS {
    let dst_max = cbDest / core::mem::size_of::<u16>();
    let mut d = 0usize;
    while d < dst_max && *pszDest.add(d) != 0 { d += 1; }
    let mut s = 0usize;
    while d < dst_max.saturating_sub(1) && *pszSrc.add(s) != 0 {
        *pszDest.add(d) = *pszSrc.add(s);
        d += 1; s += 1;
    }
    if d < dst_max { *pszDest.add(d) = 0; }
    STATUS_SUCCESS
}

// GetW32ThreadInfo — win32.h: (PTHREADINFO)PsGetCurrentThreadWin32Thread().
#[inline(always)]
unsafe fn GetW32ThreadInfo() -> PTHREADINFO {
    PsGetCurrentThreadWin32Thread() as PTHREADINFO
}

// UlongToHandle(ul) = (HANDLE)(ULONG_PTR)(ul).
#[inline(always)]
fn UlongToHandle(ul: u32) -> HANDLE { ul as usize as HANDLE }

// IS_IMM_MODE() = gpsi && (gpsi->dwSRVIFlags & SRVINFO_IMM32). Matches ime.rs.
#[inline(always)]
unsafe fn IS_IMM_MODE() -> BOOL {
    (!gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_IMM32) != 0) as BOOL
}

// IS_16BIT_MODE() = current thread TIF_flags & TIF_16BIT (ntuser.h, kernel view).
#[inline(always)]
unsafe fn IS_16BIT_MODE() -> BOOL {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (!pti.is_null() && ((*pti).TIF_flags & TIF_16BIT) != 0) as BOOL
}

// ISITHOOKED(HookId) — fsHooks & HOOKID_TO_FLAG(HookId), HOOKID_TO_FLAG = 1<<(id+1).
#[inline(always)]
unsafe fn ISITHOOKED(hook: core::ffi::c_int) -> BOOL {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if pti.is_null() { return FALSE as BOOL; }
    (((*pti).fsHooks & (1u32 << (hook + 1))) != 0) as BOOL
}

// IntReferenceThreadInfo — win32.h macro (InterlockedIncrement on RefCount).
// THREADINFO embeds W32THREAD: RefCount (LONG) at size_of::<usize>() (after pEThread).
#[inline(always)]
unsafe fn IntReferenceThreadInfo(pti: PTHREADINFO) {
    InterlockedIncrement((pti as *mut u8).add(core::mem::size_of::<usize>()) as *mut LONG);
}
// IntDereferenceThreadInfo(pti) — win32.h macro.
#[inline(always)]
unsafe fn IntDereferenceThreadInfo(pti: PTHREADINFO) {
    if InterlockedDecrement((pti as *mut u8).add(core::mem::size_of::<usize>()) as *mut LONG) == 0 {
        UserDeleteW32Thread(pti);
    }
}

// UserHMGetHandle(obj) = obj->head.h (object.h macro). Generic over object type.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    (*(obj as *const _HEAD)).h
}

// RtlInitEmptyUnicodeString (rtlfuncs.h FORCEINLINE) — no kernel symbol.
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(UnicodeString: PUNICODE_STRING, Buffer: PWSTR, BufferSize: USHORT) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

// IS_IME_HKL(hKL) = ((ULONG_PTR)hKL & 0xF0000000) == IME_MASK (ime.h macro).
#[inline(always)]
unsafe fn IS_IME_HKL(hKL: HKL) -> u8 {
    (((hKL as ULONG_PTR) & 0xF0000000) == IME_MASK as ULONG_PTR) as u8
}
// IS_CICERO_MODE() = gpsi && (gpsi->dwSRVIFlags & SRVINFO_CTFIME_ENABLED) (ntuser.h macro).
#[inline(always)]
unsafe fn IS_CICERO_MODE() -> u8 {
    (!gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_CTFIME_ENABLED) != 0) as u8
}
// HandleToUlong(h) = (ULONG)(ULONG_PTR)(h) (basetsd.h macro).
#[inline(always)]
fn HandleToUlong(h: *mut core::ffi::c_void) -> u32 {
    h as usize as u32
}
extern "system" {
    fn SafeProbeForWrite( Address: *mut core::ffi::c_void, Length: usize, Alignment: ULONG, ) -> NTSTATUS;
    fn SafeProbeForRead( Address: *mut core::ffi::c_void, Length: usize, Alignment: ULONG, ) -> NTSTATUS;
}
// co_IntSendMessage — real FASTCALL export (message.rs). i386 fastcall, cdecl else.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn co_IntSendMessage(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn co_IntSendMessage(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
// UserRefObjectCo / UserDerefObjectCo (object.h static __inline; mirrors focus.rs).
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PVOID, RefEntry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*RefEntry).obj = obj;
    UserReferenceObject(obj);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*RefEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PVOID) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    UserDereferenceObject(obj);
}

// ---- file-scope preamble (translated) ----
pub const CP_ACP: u32 = 0;

pub type PFN_KBDLAYERDESCRIPTOR = Option<unsafe extern "system" fn() -> PVOID>;

pub static mut gpkfList: PKBDFILE = core::ptr::null_mut();
pub static mut ghKLSentToShell: HKL = core::ptr::null_mut();

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntHKLtoPKL(pti: PTHREADINFO, hKL: HKL) -> PKL {
    IntHKLtoPKL_impl(pti, hKL)
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn IntHKLtoPKL(pti: PTHREADINFO, hKL: HKL) -> PKL {
    IntHKLtoPKL_impl(pti, hKL)
}

unsafe fn IntHKLtoPKL_impl(pti: PTHREADINFO, hKL: HKL) -> PKL {
    let pFirstKL: PKL;
    let mut pKL: PKL;

    pFirstKL = (*pti).KeyboardLayout;
    if pFirstKL.is_null() {
        return core::ptr::null_mut();
    }

    pKL = pFirstKL;

    let hkl_next = HKL_NEXT as usize as HKL;
    let hkl_prev = HKL_PREV as usize as HKL;

    if hKL == hkl_next {
        loop {
            pKL = (*pKL).pklNext;
            if (*pKL).dwKL_Flags & KL_UNLOAD == 0 {
                return pKL;
            }
            if pKL == pFirstKL {
                break;
            }
        }
    } else if hKL == hkl_prev {
        loop {
            pKL = (*pKL).pklPrev;
            if (*pKL).dwKL_Flags & KL_UNLOAD == 0 {
                return pKL;
            }
            if pKL == pFirstKL {
                break;
            }
        }
    } else if ((hKL as usize >> 16) & 0xFFFF) != 0 {
        loop {
            if (*pKL).hkl == hKL {
                return pKL;
            }
            pKL = (*pKL).pklNext;
            if pKL == pFirstKL {
                break;
            }
        }
    } else {
        loop {
            if ((*pKL).hkl as usize & 0xFFFF) == (hKL as usize & 0xFFFF) {
                return pKL;
            }
            pKL = (*pKL).pklNext;
            if pKL == pFirstKL {
                break;
            }
        }
    }

    core::ptr::null_mut()
}

// Not `extern "system"` any more: `Result<UINT, NTSTATUS>` (added below to
// propagate a write fault out of the *pHklBuff store loop) is not FFI-safe,
// and nothing outside this file calls IntGetKeyboardLayoutList.
unsafe fn IntGetKeyboardLayoutList(
    pWinSta: PWINSTATION_OBJECT,
    mut nBuff: ULONG,
    mut pHklBuff: *mut HKL,
) -> Result<UINT, NTSTATUS> {
    let mut ret: UINT = 0;
    let pFirstKL: PKL = gspklBaseLayout;

    if pWinSta.is_null() || pFirstKL.is_null() {
        return Ok(0);
    }

    let mut pKL: PKL = pFirstKL;

    if nBuff == 0 {
        loop {
            if (*pKL).dwKL_Flags & KL_UNLOAD == 0 {
                ret += 1;
            }
            pKL = (*pKL).pklNext;
            if pKL == pFirstKL {
                break;
            }
        }
    } else {
        loop {
            if (*pKL).dwKL_Flags & KL_UNLOAD == 0 {
                // Fault-protected store — this call sits inside the C's
                // NtUserGetKeyboardLayoutList _SEH2_TRY (see the dropped-_SEH2
                // recovery at that call site), and the raw `*pHklBuff = ...`
                // here is what that _SEH2_TRY was actually backstopping. The
                // range/alignment was already probed once by the caller via
                // SafeProbeForWrite before calling in, matching the C's earlier
                // SafeProbeForWrite; this only has to survive a page going away
                // mid-loop.
                let hkl: HKL = (*pKL).hkl;
                ntkrnl_sys::probe::safe_copy_raw(
                    pHklBuff as *mut core::ffi::c_void,
                    core::ptr::addr_of!(hkl) as *const core::ffi::c_void,
                    core::mem::size_of::<HKL>(),
                )?;
                pHklBuff = pHklBuff.add(1);
                ret += 1;
                nBuff -= 1;

                if nBuff == 0 {
                    break;
                }
            }
            pKL = (*pKL).pklNext;
            if pKL == pFirstKL {
                break;
            }
        }
    }

    Ok(ret)
}

unsafe fn DumpKbdLayout(pKbdTbl: PKBDTABLES) {
    let mut pVkToBit: PVK_TO_BIT;
    let mut pVkToWchTbl: PVK_TO_WCHAR_TABLE;
    let mut pVscVk: PVSC_VK;
    let mut i: ULONG;

    DbgPrint(
        b"Kbd layout: fLocaleFlags %x bMaxVSCtoVK %x\n\0".as_ptr() as *const i8,
        (*pKbdTbl).fLocaleFlags,
        (*pKbdTbl).bMaxVSCtoVK as u32,
    );
    DbgPrint(
        b"wMaxModBits %x\n\0".as_ptr() as *const i8,
        if !(*pKbdTbl).pCharModifiers.is_null() {
            (*(*pKbdTbl).pCharModifiers).wMaxModBits as u32
        } else {
            0
        },
    );

    if !(*pKbdTbl).pCharModifiers.is_null() {
        pVkToBit = (*(*pKbdTbl).pCharModifiers).pVkToBit;
        if !pVkToBit.is_null() {
            while (*pVkToBit).Vk != 0 {
                DbgPrint(
                    b"VkToBit %x -> %x\n\0".as_ptr() as *const i8,
                    (*pVkToBit).Vk as u32,
                    (*pVkToBit).ModBits as u32,
                );
                pVkToBit = pVkToBit.add(1);
            }
        }

        i = 0;
        while i <= (*(*pKbdTbl).pCharModifiers).wMaxModBits as u32 {
            DbgPrint(
                b"ModNumber %x -> %x\n\0".as_ptr() as *const i8,
                i,
                *(*(*pKbdTbl).pCharModifiers).ModNumber.as_ptr().add(i as usize) as u32,
            );
            i += 1;
        }
    }

    pVkToWchTbl = (*pKbdTbl).pVkToWcharTable;
    if !pVkToWchTbl.is_null() {
        while !(*pVkToWchTbl).pVkToWchars.is_null() {
            let mut pVkToWch: PVK_TO_WCHARS1 = (*pVkToWchTbl).pVkToWchars;

            DbgPrint(
                b"pVkToWchTbl nModifications %x cbSize %x\n\0".as_ptr() as *const i8,
                (*pVkToWchTbl).nModifications as u32,
                (*pVkToWchTbl).cbSize as u32,
            );
            if !pVkToWch.is_null() {
                while (*pVkToWch).VirtualKey != 0 {
                    DbgPrint(
                        b"pVkToWch VirtualKey %x Attributes %x wc { \0".as_ptr() as *const i8,
                        (*pVkToWch).VirtualKey as u32,
                        (*pVkToWch).Attributes as u32,
                    );
                    i = 0;
                    while i < (*pVkToWchTbl).nModifications as u32 {
                        DbgPrint(
                            b"%x \0".as_ptr() as *const i8,
                            *(*pVkToWch).wch.as_ptr().add(i as usize) as u32,
                        );
                        i += 1;
                    }
                    DbgPrint(b"}\n\0".as_ptr() as *const i8);
                    pVkToWch = ((pVkToWch as PBYTE).add((*pVkToWchTbl).cbSize as usize))
                        as PVK_TO_WCHARS1;
                }
            }
            pVkToWchTbl = pVkToWchTbl.add(1);
        }
    }

    DbgPrint(b"pusVSCtoVK: { \0".as_ptr() as *const i8);
    if !(*pKbdTbl).pusVSCtoVK.is_null() {
        i = 0;
        while i < (*pKbdTbl).bMaxVSCtoVK as u32 {
            DbgPrint(
                b"%x -> %x, \0".as_ptr() as *const i8,
                i,
                *(*pKbdTbl).pusVSCtoVK.add(i as usize) as u32,
            );
            i += 1;
        }
    }
    DbgPrint(b"}\n\0".as_ptr() as *const i8);

    DbgPrint(b"pVSCtoVK_E0: { \0".as_ptr() as *const i8);
    pVscVk = (*pKbdTbl).pVSCtoVK_E0;
    if !pVscVk.is_null() {
        while (*pVscVk).Vsc != 0 {
            DbgPrint(
                b"%x -> %x, \0".as_ptr() as *const i8,
                (*pVscVk).Vsc as u32,
                (*pVscVk).Vk as u32,
            );
            pVscVk = pVscVk.add(1);
        }
    }
    DbgPrint(b"}\n\0".as_ptr() as *const i8);

    DbgPrint(b"pVSCtoVK_E1: { \0".as_ptr() as *const i8);
    pVscVk = (*pKbdTbl).pVSCtoVK_E1;
    if !pVscVk.is_null() {
        while (*pVscVk).Vsc != 0 {
            DbgPrint(
                b"%x -> %x, \0".as_ptr() as *const i8,
                (*pVscVk).Vsc as u32,
                (*pVscVk).Vk as u32,
            );
            pVscVk = pVscVk.add(1);
        }
    }
    DbgPrint(b"}\n\0".as_ptr() as *const i8);
}

unsafe fn UserLoadKbdDll(
    pwszLayoutPath: *mut WCHAR,
    phModule: *mut HANDLE,
    pKbdTables: *mut PKBDTABLES,
) -> BOOL {
    let pfnKbdLayerDescriptor: PFN_KBDLAYERDESCRIPTOR;

    /* Load keyboard layout DLL */
    *phModule = EngLoadImage(pwszLayoutPath);
    if (*phModule).is_null() {
        return 0;
    }

    /* Find KbdLayerDescriptor function and get layout tables */
    let pfn = EngFindImageProcAddress(*phModule, b"KbdLayerDescriptor\0".as_ptr() as *mut _);
    pfnKbdLayerDescriptor = core::mem::transmute::<_, PFN_KBDLAYERDESCRIPTOR>(pfn);

    if let Some(pfn) = pfnKbdLayerDescriptor {
        *pKbdTables = pfn() as PKBDTABLES;
    }

    if pfnKbdLayerDescriptor.is_none() || (*pKbdTables).is_null() {
        EngUnloadImage(*phModule);
        return 0;
    }

    1
}

unsafe fn UserLoadKbdFile(pwszKLID: PUNICODE_STRING) -> PKBDFILE {
    let mut pRet: PKBDFILE = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut cbSize: ULONG;
    let mut hKey: HKEY = core::ptr::null_mut();

    let mut wszLayoutPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    {
        let init = b"\\SystemRoot\\System32\\";
        let s: &[u16] = &[
            0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x52, 0x6F, 0x6F, 0x74,
            0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x33, 0x32, 0x5C,
        ];
        let _ = init;
        for (i, &c) in s.iter().enumerate() {
            wszLayoutPath[i] = c;
        }
    }

    let mut wszLayoutRegKey: [WCHAR; 256] = [0; 256];
    {
        let s = "\\REGISTRY\\Machine\\SYSTEM\\CurrentControlSet\\Control\\Keyboard Layouts\\";
        for (i, c) in s.encode_utf16().enumerate() {
            wszLayoutRegKey[i] = c;
        }
    }

    let pkf: PKBDFILE = UserCreateObject(
        gHandleTable,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        TYPE_KBDFILE,
        core::mem::size_of::<KBDFILE>() as _,
    ) as PKBDFILE;

    if pkf.is_null() {
        ERR!("Failed to create object!\n");
        return core::ptr::null_mut();
    }

    swprintf(
        (*pkf).awchKF.as_mut_ptr(),
        w!("%wZ").as_ptr() as *const u16 as *const _,
        pwszKLID,
    );

    'cleanup: {
        RtlStringCbCatW(
            wszLayoutRegKey.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; 256]>(),
            (*pkf).awchKF.as_mut_ptr(),
        );

        Status = RegOpenKey(wszLayoutRegKey.as_mut_ptr(), &mut hKey);
        if !NT_SUCCESS(Status) {
            ERR!(
                "Failed to open keyboard layouts registry key %ws (%lx)\n",
                wszLayoutRegKey.as_ptr(),
                Status,
            );
            break 'cleanup;
        }

        cbSize = (core::mem::size_of::<[WCHAR; MAX_PATH as usize]>()
            - (wcslen(wszLayoutPath.as_ptr()) as usize) * core::mem::size_of::<WCHAR>())
            as ULONG;

        let Status2 = RegQueryValue(
            hKey,
            w!("Layout File").as_ptr() as *const u16 as *const _,
            REG_SZ,
            wszLayoutPath.as_mut_ptr().add(wcslen(wszLayoutPath.as_ptr()) as usize) as *mut _,
            &mut cbSize,
        );

        if !NT_SUCCESS(Status2) {
            ERR!(
                "Can't get layout filename for %wZ (%lx)\n",
                pwszKLID,
                Status2,
            );
            break 'cleanup;
        }

        if UserLoadKbdDll(
            wszLayoutPath.as_mut_ptr(),
            core::ptr::addr_of_mut!((*pkf).hBase),
            core::ptr::addr_of_mut!((*pkf).pKbdTbl),
        ) == 0
        {
            ERR!(
                "Failed to load %ws dll!\n",
                wszLayoutPath.as_ptr(),
            );
            break 'cleanup;
        }

        (*pkf).pkfNext = gpkfList;
        gpkfList = pkf;

        pRet = pkf;
    }

    if !hKey.is_null() {
        ZwClose(hKey as *mut c_void);
    }
    if !pkf.is_null() {
        UserDereferenceObject(pkf as *mut _);
    }
    if pRet.is_null() {
        if !pkf.is_null() {
            UserDeleteObject(UserHMGetHandle(pkf as *mut _), TYPE_KBDFILE);
        }
    }

    pRet
}

unsafe fn co_UserLoadKbdLayout(pustrKLID: PUNICODE_STRING, hKL: HKL) -> PKL {
    let mut lCid: LCID = 0;
    let mut cs: CHARSETINFO = core::mem::zeroed();

    /* Create keyboard layout object */
    let pKl = UserCreateObject(
        gHandleTable,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        TYPE_KBDLAYOUT,
        core::mem::size_of::<KL>() as _,
    ) as PKL;
    if pKl.is_null() {
        ERR!("Failed to create object!\n");
        return core::ptr::null_mut();
    }

    (*pKl).hkl = hKL;
    (*pKl).spkf = UserLoadKbdFile(pustrKLID);

    /* Dereference keyboard layout */
    UserDereferenceObject(pKl as _);

    /* If we failed, remove KL object */
    if (*pKl).spkf.is_null() {
        ERR!("UserLoadKbdFile failed!\n", pustrKLID);
        UserDeleteObject(UserHMGetHandle(pKl), TYPE_KBDLAYOUT);
        return core::ptr::null_mut();
    }

    // Up to Language Identifiers..
    if !NT_SUCCESS(RtlUnicodeStringToInteger(
        pustrKLID as *mut _UNICODE_STRING as *mut _,
        16,
        &mut lCid as *mut LCID as *mut u32,
    )) {
        ERR!("RtlUnicodeStringToInteger failed\n", pustrKLID);
        UserDeleteObject(UserHMGetHandle(pKl), TYPE_KBDLAYOUT);
        return core::ptr::null_mut();
    }

    TRACE!(
        b"Language Identifiers LCID 0x%x\n\0".as_ptr() as _,
        pustrKLID,
        lCid,
    );
    if co_IntGetCharsetInfo(lCid, &mut cs) != 0 {
        (*pKl).iBaseCharset = cs.ciCharset;
        (*pKl).dwFontSigs = cs.fs.fsCsb[0];
        (*pKl).CodePage = cs.ciACP as USHORT;
        TRACE!(
            b"Charset %u Font Sig %lu CodePage %u\n\0".as_ptr() as _,
            (*pKl).iBaseCharset,
            (*pKl).dwFontSigs,
            (*pKl).CodePage,
        );
    } else {
        (*pKl).iBaseCharset = ANSI_CHARSET;
        (*pKl).dwFontSigs = FS_LATIN1;
        (*pKl).CodePage = 0;
    }

    // Set initial system character set and font signature.
    if gSystemFS == 0 {
        gSystemCPCharSet = (*pKl).iBaseCharset;
        gSystemFS = (*pKl).dwFontSigs;
    }

    pKl
}

unsafe fn UnloadKbdFile(pkf: PKBDFILE) {
    let mut ppkfLink: *mut PKBDFILE = core::ptr::addr_of_mut!(gpkfList);

    while !(*ppkfLink).is_null() {
        if *ppkfLink == pkf {
            break;
        }

        ppkfLink = core::ptr::addr_of_mut!((**ppkfLink).pkfNext);
    }

    if *ppkfLink == pkf {
        *ppkfLink = (*pkf).pkfNext;
    }

    EngUnloadImage((*pkf).hBase);
    UserDeleteObject(UserHMGetHandle(pkf), TYPE_KBDFILE);
}

unsafe fn UserUnloadKbl(pKl: PKL) -> BOOL {
    if pKl == gspklBaseLayout {
        if (*pKl).pklNext == (*pKl).pklPrev {
            return 0;
        }

        gspklBaseLayout = (*pKl).pklNext;
    }

    if (*pKl).head.cLockObj > 1 {
        (*pKl).dwKL_Flags |= KL_UNLOAD;
        return 0;
    }

    (*(*pKl).pklPrev).pklNext = (*pKl).pklNext;
    (*(*pKl).pklNext).pklPrev = (*pKl).pklPrev;
    UnloadKbdFile((*pKl).spkf);
    if !(*pKl).piiex.is_null() {
        ExFreePoolWithTag((*pKl).piiex as *mut _, USERTAG_IME);
    }
    UserDeleteObject(UserHMGetHandle(pKl), TYPE_KBDLAYOUT);
    1
}

#[no_mangle]
unsafe extern "C" fn W32kGetDefaultKeyLayout() -> PKL {
    let mut pKl: PKL = gspklBaseLayout;

    if pKl.is_null() {
        return core::ptr::null_mut();
    }

    loop {
        if (*pKl).dwKL_Flags & KL_UNLOAD == 0 {
            return pKl;
        }

        pKl = (*pKl).pklPrev;

        if pKl == gspklBaseLayout {
            break;
        }
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn UserHklToKbl(hKl: HKL) -> PKL {
    let mut pKl: PKL = gspklBaseLayout;

    if gspklBaseLayout.is_null() {
        return core::ptr::null_mut();
    }

    loop {
        if (*pKl).hkl == hKl {
            return pKl;
        }

        pKl = (*pKl).pklNext;

        if pKl == gspklBaseLayout {
            break;
        }
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntReorderKeyboardLayouts(
    pWinSta: PWINSTATION_OBJECT,
    pNewKL: PKL,
) {
    IntReorderKeyboardLayouts_impl(pWinSta, pNewKL)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntReorderKeyboardLayouts(
    pWinSta: PWINSTATION_OBJECT,
    pNewKL: PKL,
) {
    IntReorderKeyboardLayouts_impl(pWinSta, pNewKL)
}

unsafe fn IntReorderKeyboardLayouts_impl(
    pWinSta: PWINSTATION_OBJECT,
    pNewKL: PKL,
) {
    let pOldKL = gspklBaseLayout;

    if ((*pWinSta).Flags & WSS_NOIO) != 0 || pNewKL == pOldKL {
        return;
    }

    (*(*pNewKL).pklPrev).pklNext = (*pNewKL).pklNext;
    (*(*pNewKL).pklNext).pklPrev = (*pNewKL).pklPrev;
    (*pNewKL).pklNext = pOldKL;
    (*pNewKL).pklPrev = (*pOldKL).pklPrev;
    (*(*pOldKL).pklPrev).pklNext = pNewKL;
    (*pOldKL).pklPrev = pNewKL;
    gspklBaseLayout = pNewKL;
}
#[no_mangle]
pub unsafe extern "system" fn UserSetDefaultInputLang(hKl: HKL) -> BOOL {
    let pKl: PKL;

    pKl = UserHklToKbl(hKl);
    if pKl.is_null() {
        return 0;
    }

    IntReorderKeyboardLayouts(IntGetProcessWindowStation(core::ptr::null_mut()), pKl);
    1
}

unsafe extern "system" fn IntImmActivateLayout(pti: PTHREADINFO, pKL: PKL) {
    let pImeWnd: PWND;
    let hImeWnd: HWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if (*pti).KeyboardLayout == pKL {
        return;
    }

    pImeWnd = (*pti).spwndDefaultIme;
    if !pImeWnd.is_null() {
        UserRefObjectCo(pImeWnd as PVOID, &mut Ref);
        hImeWnd = UserHMGetHandle(pImeWnd) as HWND;
        co_IntSendMessage(
            hImeWnd,
            WM_IME_SYSTEM,
            IMS_ACTIVATELAYOUT as WPARAM,
            (*pKL).hkl as LPARAM,
        );
        UserDerefObjectCo(pImeWnd as PVOID);
    } else {
        (*pti).hklPrev = (*(*pti).KeyboardLayout).hkl;

        if !(*pti).spDefaultImc.is_null() {
            (*(*pti).pClientInfo).CI_flags |= CI_IMMACTIVATE as ULONG_PTR;
        }
    }

    UserAssignmentLock(
        core::ptr::addr_of_mut!((*pti).KeyboardLayout) as *mut PVOID,
        pKL as PVOID,
    );
    (*(*pti).pClientInfo).hKL = (*pKL).hkl;
    (*(*pti).pClientInfo).CodePage = (*pKL).CodePage;
}

unsafe fn co_IntSetKeyboardLayoutForProcess(ppi: PPROCESSINFO, pKL: PKL) {
    let mut ptiNext: PTHREADINFO;
    let pClientInfo: PCLIENTINFO;
    let bImmMode = IS_IMM_MODE();

    let mut ptiNode = (*ppi).ptiList;
    while !ptiNode.is_null() {
        IntReferenceThreadInfo(ptiNode);
        ptiNext = (*ptiNode).ptiSibling;

        if (*ptiNode).KeyboardLayout == pKL || ((*ptiNode).TIF_flags & TIF_INCLEANUP) != 0 {
            IntDereferenceThreadInfo(ptiNode);
            ptiNode = ptiNext;
            continue;
        }

        if bImmMode != 0 {
            IntImmActivateLayout(ptiNode, pKL);
        } else {
            UserAssignmentLock(core::ptr::addr_of_mut!((*ptiNode).KeyboardLayout) as *mut PVOID, pKL as PVOID);
            let pClientInfo = (*ptiNode).pClientInfo;
            (*pClientInfo).CodePage = (*pKL).CodePage;
            (*pClientInfo).hKL = (*pKL).hkl;
        }

        IntDereferenceThreadInfo(ptiNode);
        ptiNode = ptiNext;
    }
}

#[no_mangle]
pub unsafe extern "system" fn co_UserActivateKeyboardLayout(
    pKL: PKL,
    uFlags: ULONG,
    pWnd: PWND,
) -> HKL {
    let mut hOldKL: HKL = core::ptr::null_mut();
    let pOldKL: PKL;
    let pti: PTHREADINFO = GetW32ThreadInfo();
    let mut pTargetWnd: PWND;
    let pImeWnd: PWND;
    let hTargetWnd: HWND;
    let hImeWnd: HWND;
    let mut Ref1: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Ref2: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let ClientInfo: PCLIENTINFO;
    let mut bSetForProcess: BOOL = ((uFlags & KLF_SETFORPROCESS) != 0) as BOOL;

    IntReferenceThreadInfo(pti);
    ClientInfo = (*pti).pClientInfo;

    pOldKL = (*pti).KeyboardLayout;
    if !pOldKL.is_null() {
        hOldKL = (*pOldKL).hkl;
    }

    if uFlags & KLF_RESET != 0 {
        FIXME!(b"KLF_RESET\n\0".as_ptr() as *const i8);
    }

    if bSetForProcess == 0 && pKL == (*pti).KeyboardLayout {
        IntDereferenceThreadInfo(pti);
        return hOldKL;
    }

    (*pKL).wchDiacritic = 0;

    if !pOldKL.is_null() {
        UserRefObjectCo(pOldKL as *mut _, &mut Ref1);
    }

    if (*pti).TIF_flags & TIF_CSRSSTHREAD != 0 {
        UserAssignmentLock(
            core::ptr::addr_of_mut!((*pti).KeyboardLayout) as *mut *mut core::ffi::c_void,
            pKL as *mut core::ffi::c_void,
        );
        (*ClientInfo).CodePage = (*pKL).CodePage;
        (*ClientInfo).hKL = (*pKL).hkl;
    } else if bSetForProcess != 0 {
        co_IntSetKeyboardLayoutForProcess((*pti).ppi, pKL);
    } else {
        if IS_IMM_MODE() != 0 {
            IntImmActivateLayout(pti, pKL);
        } else {
            UserAssignmentLock(
                core::ptr::addr_of_mut!((*pti).KeyboardLayout) as *mut *mut core::ffi::c_void,
                pKL as *mut core::ffi::c_void,
            );
        }

        (*ClientInfo).CodePage = (*pKL).CodePage;
        (*ClientInfo).hKL = (*pKL).hkl;
    }

    if !gptiForeground.is_null()
        && (*gptiForeground).ppi == (*pti).ppi
        && ISITHOOKED(WH_SHELL as i32) != 0
    {
        if ghKLSentToShell != (*pKL).hkl {
            co_IntShellHookNotify(HSHELL_LANGUAGE as _, 0, (*pKL).hkl as LPARAM);
            ghKLSentToShell = (*pKL).hkl;
        }
    }

    if !(*pti).MessageQueue.is_null() {
        pTargetWnd = (*(*pti).MessageQueue).spwndFocus;
        if pTargetWnd.is_null() {
            pTargetWnd = (*(*pti).MessageQueue).spwndActive;
            if pTargetWnd.is_null() {
                pTargetWnd = pWnd;
            }
        }

        if !pTargetWnd.is_null() {
            UserRefObjectCo(pTargetWnd as *mut _, &mut Ref2);
            hTargetWnd = UserHMGetHandle(pTargetWnd) as HWND;
            co_IntSendMessage(
                hTargetWnd,
                WM_INPUTLANGCHANGE,
                (*pKL).iBaseCharset as _,
                (*pKL).hkl as LPARAM,
            );
            UserDerefObjectCo(pTargetWnd as *mut _);
        }
    }

    if (*pti).TIF_flags & TIF_CSRSSTHREAD == 0 {
        if IS_IME_HKL((*pKL).hkl) != 0 || (IS_CICERO_MODE() != 0 && IS_16BIT_MODE() == 0) {
            pImeWnd = (*pti).spwndDefaultIme;
            if !pImeWnd.is_null() {
                bSetForProcess &= (IS_16BIT_MODE() == 0) as BOOL;
                UserRefObjectCo(pImeWnd as *mut _, &mut Ref2);
                hImeWnd = UserHMGetHandle(pImeWnd) as HWND;
                co_IntSendMessage(
                    hImeWnd,
                    WM_IME_SYSTEM,
                    IMS_SENDNOTIFICATION as _,
                    bSetForProcess as LPARAM,
                );
                UserDerefObjectCo(pImeWnd as *mut _);
            }
        }
    }

    if !pOldKL.is_null() {
        UserDerefObjectCo(pOldKL as *mut _);
    }

    IntDereferenceThreadInfo(pti);
    hOldKL
}

#[no_mangle]
pub unsafe extern "system" fn co_IntActivateKeyboardLayout(
    pWinSta: PWINSTATION_OBJECT,
    hKL: HKL,
    uFlags: ULONG,
    pWnd: PWND,
) -> HKL {
    let pKL: PKL;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    pKL = IntHKLtoPKL(pti, hKL);
    if pKL.is_null() {
        return core::ptr::null_mut();
    }

    if uFlags & KLF_REORDER != 0 {
        IntReorderKeyboardLayouts(pWinSta, pKL);
    }

    co_UserActivateKeyboardLayout(pKL, uFlags, pWnd)
}

unsafe extern "system" fn co_IntUnloadKeyboardLayoutEx(
    pWinSta: PWINSTATION_OBJECT,
    pKL: PKL,
    dwFlags: DWORD,
) -> BOOL {
    let pNextKL: PKL;
    let mut Ref1: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Ref2: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let pti: PTHREADINFO = gptiCurrent;

    if pKL == gspklBaseLayout && (dwFlags & UKL_NOACTIVATENEXT) == 0 {
        return FALSE as BOOL;
    }

    UserRefObjectCo(pKL as PVOID, &mut Ref1); /* Add reference */

    /* Regard as unloaded */
    UserMarkObjectDestroy(pKL as PVOID);
    (*pKL).dwKL_Flags |= KL_UNLOAD;

    if (dwFlags & UKL_NOACTIVATENEXT) == 0 && (*pti).KeyboardLayout == pKL {
        pNextKL = IntHKLtoPKL(pti, UlongToHandle(HKL_NEXT) as HKL);
        if !pNextKL.is_null() {
            UserRefObjectCo(pNextKL as PVOID, &mut Ref2); /* Add reference */
            co_UserActivateKeyboardLayout(pNextKL, dwFlags, core::ptr::null_mut());
            UserDerefObjectCo(pNextKL as PVOID); /* Release reference */
        }
    }

    if gspklBaseLayout == pKL && pKL != (*pKL).pklNext {
        /* Set next layout as default */
        gspklBaseLayout = (*pKL).pklNext;
    }

    UserDerefObjectCo(pKL as PVOID); /* Release reference */

    if ISITHOOKED(WH_SHELL as i32) != 0 {
        co_IntShellHookNotify(HSHELL_LANGUAGE as WPARAM, 0, 0);
        ghKLSentToShell = core::ptr::null_mut();
    }

    TRUE as BOOL
}

unsafe extern "system" fn IntUnloadKeyboardLayout(pWinSta: PWINSTATION_OBJECT, hKL: HKL) -> BOOL {
    let pKL: PKL = IntHKLtoPKL(gptiCurrent, hKL);
    if pKL.is_null() {
        return 0;
    }
    co_IntUnloadKeyboardLayoutEx(pWinSta, pKL, 0)
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_UserImmLoadLayout(hKL: HKL) -> PIMEINFOEX {
    co_UserImmLoadLayout_impl(hKL)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_UserImmLoadLayout(hKL: HKL) -> PIMEINFOEX {
    co_UserImmLoadLayout_impl(hKL)
}

unsafe fn co_UserImmLoadLayout_impl(hKL: HKL) -> PIMEINFOEX {

    let piiex: PIMEINFOEX;

    if IS_IME_HKL(hKL) == 0 && IS_CICERO_MODE() == 0 {
        return core::ptr::null_mut();
    }

    piiex = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<IMEINFOEX>() as SIZE_T,
        USERTAG_IME,
    ) as PIMEINFOEX;
    if piiex.is_null() {
        return core::ptr::null_mut();
    }

    if co_ClientImmLoadLayout(hKL, piiex) == 0 {
        ExFreePoolWithTag(piiex as *mut core::ffi::c_void, USERTAG_IME);
        return core::ptr::null_mut();
    }

    piiex
}

#[no_mangle]
pub unsafe extern "system" fn co_IntLoadKeyboardLayoutEx(
    pWinSta: PWINSTATION_OBJECT,
    hSafeFile: HANDLE,
    hOldKL: HKL,
    puszSafeKLID: PUNICODE_STRING,
    hNewKL: HKL,
    Flags: UINT,
) -> HKL {
    let pOldKL: PKL;
    let pNewKL: PKL;

    let _ = hSafeFile;

    if hNewKL.is_null() || ((*pWinSta).Flags & WSS_NOIO) != 0 {
        return core::ptr::null_mut();
    }

    if !hOldKL.is_null() && hOldKL != hNewKL {
        pOldKL = UserHklToKbl(hOldKL);
        if !pOldKL.is_null() {
            UserUnloadKbl(pOldKL);
        }
    }

    pNewKL = UserHklToKbl(hNewKL);
    if pNewKL.is_null() {
        let pNewKL = co_UserLoadKbdLayout(puszSafeKLID, hNewKL);
        if pNewKL.is_null() {
            return core::ptr::null_mut();
        }

        if !gspklBaseLayout.is_null() {
            let mut pLastKL = (*gspklBaseLayout).pklPrev;
            while pLastKL != gspklBaseLayout && ((*pLastKL).dwKL_Flags & KL_UNLOAD) != 0 {
                pLastKL = (*pLastKL).pklPrev;
            }

            (*pNewKL).pklNext = (*pLastKL).pklNext;
            (*pNewKL).pklPrev = pLastKL;
            (*(*pNewKL).pklNext).pklPrev = pNewKL;
            (*(*pNewKL).pklPrev).pklNext = pNewKL;
        } else {
            (*pNewKL).pklNext = pNewKL;
            (*pNewKL).pklPrev = pNewKL;
            gspklBaseLayout = pNewKL;
        }

        (*pNewKL).piiex = co_UserImmLoadLayout(hNewKL);

        (*pNewKL).dwKL_Flags &= !KL_UNLOAD;

        if Flags & KLF_REORDER != 0 {
            IntReorderKeyboardLayouts(pWinSta, pNewKL);
        }

        if Flags & KLF_ACTIVATE != 0 {
            co_UserActivateKeyboardLayout(pNewKL, Flags, core::ptr::null_mut());
        }

        if Flags & KLF_NOTELLSHELL == 0 {
            co_IntShellHookNotify(HSHELL_LANGUAGE as _, 0, hNewKL as LPARAM);
        }

        return hNewKL;
    }

    (*pNewKL).dwKL_Flags &= !KL_UNLOAD;

    if Flags & KLF_REORDER != 0 {
        IntReorderKeyboardLayouts(pWinSta, pNewKL);
    }

    if Flags & KLF_ACTIVATE != 0 {
        co_UserActivateKeyboardLayout(pNewKL, Flags, core::ptr::null_mut());
    }

    if Flags & KLF_NOTELLSHELL == 0 {
        co_IntShellHookNotify(HSHELL_LANGUAGE as _, 0, hNewKL as LPARAM);
    }

    hNewKL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntVerifyKeyboardFileHandle(hFile: HANDLE) -> HANDLE {
    IntVerifyKeyboardFileHandleImpl(hFile)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntVerifyKeyboardFileHandle(hFile: HANDLE) -> HANDLE {
    IntVerifyKeyboardFileHandleImpl(hFile)
}

unsafe fn IntVerifyKeyboardFileHandleImpl(hFile: HANDLE) -> HANDLE {
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    if hFile == INVALID_HANDLE_VALUE as HANDLE {
        return core::ptr::null_mut();
    }

    Status = ObReferenceObjectByHandle(
        hFile,
        FILE_READ_DATA,
        core::ptr::null_mut(),
        1i32 as i8,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        DbgPrint(b"0x%08X\n\0".as_ptr() as *const i8, Status);
        return core::ptr::null_mut();
    }

    if !FileObject.is_null() {
        ObDereferenceObject(FileObject as PVOID);
    }

    hFile
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn UserGetKeyboardLayout(dwThreadId: DWORD) -> HKL {
    UserGetKeyboardLayout_impl(dwThreadId)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn UserGetKeyboardLayout(dwThreadId: DWORD) -> HKL {
    UserGetKeyboardLayout_impl(dwThreadId)
}

unsafe fn UserGetKeyboardLayout_impl(dwThreadId: DWORD) -> HKL {
    let mut pti: PTHREADINFO;
    let mut ListEntry: PLIST_ENTRY;
    let pKl: PKL;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if dwThreadId == 0 {
        pKl = (*pti).KeyboardLayout;
        return if !pKl.is_null() { (*pKl).hkl } else { core::ptr::null_mut() };
    }

    ListEntry = (*(*pti).rpdesk).PtiList.Flink;

    while ListEntry != core::ptr::addr_of_mut!((*(*pti).rpdesk).PtiList) {
        pti = ((ListEntry as PBYTE).offset(-(core::mem::offset_of!(THREADINFO, PtiLink) as isize))) as PTHREADINFO;

        // W32THREAD.pEThread is the first field of THREADINFO (offset 0); bindgen
        // flattened the W32THREAD prefix so reach it through a pointer reinterpret.
        if PsGetThreadId((*(pti as *const PETHREAD)).cast()) == UlongToHandle(dwThreadId) {
            let pKl2 = (*pti).KeyboardLayout;
            return if !pKl2.is_null() { (*pKl2).hkl } else { core::ptr::null_mut() };
        }

        ListEntry = (*ListEntry).Flink;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetKeyboardLayoutList(
    mut nBuff: ULONG,
    pHklBuff: *mut HKL,
) -> UINT {
    let mut ret: UINT = 0;
    let pWinSta: PWINSTATION_OBJECT;

    if pHklBuff.is_null() {
        nBuff = 0;
    }

    UserEnterShared();

    'quit: {
        if nBuff > MAXULONG / core::mem::size_of::<HKL>() as ULONG {
            SetLastNtError(ERROR_INVALID_PARAMETER as NTSTATUS);
            break 'quit;
        }

        let __SehStatus = SafeProbeForWrite(
            pHklBuff as *mut core::ffi::c_void,
            (nBuff as usize * core::mem::size_of::<HKL>()) as usize,
            1,
        );
        if !NT_SUCCESS(__SehStatus) {
            SetLastNtError(__SehStatus);
            break 'quit;
        }

        pWinSta = IntGetProcessWindowStation(core::ptr::null_mut());

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
        // C (win32ss/user/ntuser/kbdlayout.c, NtUserGetKeyboardLayoutList):
        //   _SEH2_TRY {
        //       ret = IntGetKeyboardLayoutList(pWinSta, nBuff, pHklBuff);
        //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
        //       SetLastNtError(_SEH2_GetExceptionCode());
        //       goto Quit;
        //   } _SEH2_END;
        //
        // Shape: SetLastNtError then `goto Quit` — sets the Win32 LAST ERROR
        // (not a returned status) and jumps to the same `Quit:` fall-through
        // that runs on success, so `ret` is simply left at its unassigned
        // initial value of 0 on a fault. Dropping the SetLastNtError would make
        // a faulting call silently report success (ret=0, no last-error set)
        // instead of surfacing the fault to GetLastError(). IntGetKeyboardLayoutList
        // now returns Result<UINT, NTSTATUS> because it does the raw store into
        // pHklBuff itself (see its dropped-_SEH2 recovery).
        match IntGetKeyboardLayoutList(pWinSta, nBuff, pHklBuff) {
            Ok(count) => ret = count,
            Err(status) => {
                SetLastNtError(status);
                break 'quit;
            }
        }
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetKeyboardLayoutName(
    pustrName: PUNICODE_STRING,
) -> BOOL {
    let mut bRet: BOOL = 0;
    let pKl: PKL;
    let pti: PTHREADINFO;
    let mut ustrNameSafe: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    UserEnterShared();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    pKl = (*pti).KeyboardLayout;

    'cleanup: {
        if pKl.is_null() {
            break 'cleanup;
        }

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
        // C (win32ss/user/ntuser/kbdlayout.c, NtUserGetKeyboardLayoutName):
        //   _SEH2_TRY {
        //       ProbeForWriteUnicodeString(pustrName);
        //       ustrNameSafe = *pustrName;
        //       ProbeForWrite(ustrNameSafe.Buffer, ustrNameSafe.MaximumLength, 1);
        //       ... IS_IME_HKL branch: RtlIntegerToUnicodeString ...
        //       ... else branch: MaximumLength check + RtlAppendUnicodeToString ...
        //       if (NT_SUCCESS(Status)) { *pustrName = ustrNameSafe; bRet = TRUE; }
        //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
        //       SetLastNtError(_SEH2_GetExceptionCode());
        //   } _SEH2_END;
        //
        // Shape: SetLastNtError then plain fall-through — the handler doesn't
        // goto/return, it just falls out of _SEH2_TRY/_SEH2_END straight into
        // the immediately-following `cleanup:` label, i.e. exactly the same
        // place normal completion lands. So every fault point below sets last
        // error and `break 'cleanup` with bRet left at its initial FALSE (0) —
        // identical observable effect to the C's fall-through.
        //
        // ProbeForWriteUnicodeString(Ptr) == ProbeForWrite(Ptr, sizeof(UNICODE_STRING))
        // per sdk/include/reactos/probe.h — "alignment of the pointers is not
        // verified", so alignment 1 here (not the stricter align_of::<_UNICODE_STRING>()
        // some other translated site in this crate uses for the same macro).
        let probe_status = SafeProbeForWrite(
            pustrName as *mut core::ffi::c_void,
            core::mem::size_of::<_UNICODE_STRING>(),
            1,
        );
        if !NT_SUCCESS(probe_status) {
            SetLastNtError(probe_status);
            break 'cleanup;
        }

        // `ustrNameSafe = *pustrName;` — a bare struct read of a user pointer,
        // still inside the C's _SEH2_TRY. Fault-tolerant copy, not a plain deref.
        if ntkrnl_sys::probe::safe_copy_raw(
            core::ptr::addr_of_mut!(ustrNameSafe) as *mut core::ffi::c_void,
            pustrName as *const core::ffi::c_void,
            core::mem::size_of::<_UNICODE_STRING>(),
        )
        .is_err()
        {
            SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
            break 'cleanup;
        }

        // ProbeForWrite(ustrNameSafe.Buffer, ustrNameSafe.MaximumLength, 1) — literal
        // alignment 1 from the C.
        let probe_status = SafeProbeForWrite(
            ustrNameSafe.Buffer as *mut core::ffi::c_void,
            ustrNameSafe.MaximumLength as usize,
            1,
        );
        if !NT_SUCCESS(probe_status) {
            SetLastNtError(probe_status);
            break 'cleanup;
        }

        // ustrNameSafe.Buffer is a USER pointer, and RtlIntegerToUnicodeString /
        // RtlAppendUnicodeToString write through it with a bare RtlCopyMemory and
        // no fault protection of their own -- so a probed-but-unmapped Buffer
        // faults INSIDE ntoskrnl, with our frame as the caller and nothing to
        // catch it. That is why this one does not look like the usual
        // probe-then-raw-deref: there is no raw deref in this function at all,
        // the unprotected access is one frame out. The C had the whole sequence
        // in one _SEH2_TRY. Build into a kernel buffer and publish, exactly as
        // NtUserLoadKeyboardLayoutEx below already does for the read direction.
        // Found by the wine user32:input subtest.
        let mut KernelName: [WCHAR; KL_NAMELENGTH as usize] = [0; KL_NAMELENGTH as usize];
        let mut ustrNameKernel: _UNICODE_STRING = core::mem::zeroed();
        RtlInitEmptyUnicodeString(
            (&mut ustrNameKernel as *mut _UNICODE_STRING).cast(),
            KernelName.as_mut_ptr(),
            // Keep the caller's MaximumLength as the limit so an undersized
            // buffer still yields the C's STATUS_BUFFER_OVERFLOW rather than
            // silently succeeding; clamp to the kernel buffer we actually own.
            core::cmp::min(
                ustrNameSafe.MaximumLength as usize,
                core::mem::size_of::<[WCHAR; KL_NAMELENGTH as usize]>(),
            ) as USHORT,
        );

        if IS_IME_HKL((*pKl).hkl) != 0 {
            Status = RtlIntegerToUnicodeString(
                HandleToUlong((*pKl).hkl as *mut core::ffi::c_void),
                16,
                (&mut ustrNameKernel as *mut _UNICODE_STRING).cast(),
            );
        } else {
            if (ustrNameSafe.MaximumLength as usize)
                < KL_NAMELENGTH as usize * core::mem::size_of::<u16>()
            {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                break 'cleanup;
            }

            ustrNameKernel.Length = 0;
            Status = RtlAppendUnicodeToString(
                (&mut ustrNameKernel as *mut _UNICODE_STRING).cast(),
                (*(*pKl).spkf).awchKF.as_ptr(),
            );
        }

        if NT_SUCCESS(Status) {
            // Both Rtl routines NUL-terminate when MaximumLength leaves room, and
            // every caller of GetKeyboardLayoutName scans for that NUL -- so
            // publish the terminator too, not just Length bytes.
            let cbPublish = core::cmp::min(
                ustrNameKernel.MaximumLength as usize,
                ustrNameKernel.Length as usize + core::mem::size_of::<WCHAR>(),
            );
            if !crate::copy::publish_to_user(
                ustrNameSafe.Buffer as *mut core::ffi::c_void,
                ustrNameKernel.Buffer as *const core::ffi::c_void,
                cbPublish,
            ) {
                SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
                break 'cleanup;
            }
            ustrNameSafe.Length = ustrNameKernel.Length;
            // `*pustrName = ustrNameSafe;` — the final user-pointer write, still
            // inside the C's _SEH2_TRY. Fault-tolerant copy, not a plain store.
            if ntkrnl_sys::probe::safe_copy_raw(
                pustrName as *mut core::ffi::c_void,
                core::ptr::addr_of!(ustrNameSafe) as *const core::ffi::c_void,
                core::mem::size_of::<_UNICODE_STRING>(),
            )
            .is_ok()
            {
                bRet = 1;
            } else {
                SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
            }
        }
    }

    UserLeave();
    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserLoadKeyboardLayoutEx(
    hFile: HANDLE,
    offTable: DWORD,
    pTables: PVOID,
    hOldKL: HKL,
    puszKLID: PUNICODE_STRING,
    dwNewKL: DWORD,
    Flags: UINT,
) -> HKL {
    let hRetKL: HKL;
    let mut Buffer: [WCHAR; KL_NAMELENGTH as usize] = [0; KL_NAMELENGTH as usize];
    let mut uszSafeKLID: _UNICODE_STRING = core::mem::zeroed();
    let pWinSta: PWINSTATION_OBJECT;
    let hSafeFile: HANDLE;

    let _ = offTable;
    let _ = pTables;

    if (Flags
        & !(KLF_ACTIVATE
            | KLF_NOTELLSHELL
            | KLF_REORDER
            | KLF_REPLACELANG
            | KLF_SUBSTITUTE_OK
            | KLF_SETFORPROCESS
            | KLF_UNLOADPREVIOUS
            | KLF_RESET
            | KLF_SHIFTLOCK))
        != 0
    {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return core::ptr::null_mut();
    }

    RtlInitEmptyUnicodeString(
        (&mut uszSafeKLID as *mut _UNICODE_STRING).cast(),
        Buffer.as_mut_ptr(),
        core::mem::size_of::<[WCHAR; KL_NAMELENGTH as usize]>() as USHORT,
    );

    // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
    // C (win32ss/user/ntuser/kbdlayout.c, NtUserLoadKeyboardLayoutEx):
    //   _SEH2_TRY {
    //       ProbeForRead(puszKLID, sizeof(*puszKLID), 1);
    //       ProbeForRead(puszKLID->Buffer, sizeof(puszKLID->Length), 1);
    //       RtlCopyUnicodeString(&uszSafeKLID, puszKLID);
    //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
    //       SetLastNtError(_SEH2_GetExceptionCode());
    //       _SEH2_YIELD(return NULL);
    //   } _SEH2_END;
    //
    // Shape: SetLastNtError then an immediate `return NULL` — unlike the other
    // two handlers in this file, this one aborts the whole syscall on the spot
    // rather than falling through to shared cleanup (there's no Quit/cleanup
    // label yet at this point — UserEnterExclusive() hasn't even run).
    //
    // Two literal ProbeForRead calls, not one: the second probes only
    // sizeof(puszKLID->Length) == sizeof(USHORT) == 2 bytes at Buffer's address
    // (not the whole string) — kept exactly as narrow as the C, per the
    // "literal alignment/extent, don't round up" rule.
    //
    // RtlCopyUnicodeString(&uszSafeKLID, puszKLID) itself dereferences
    // puszKLID's fields and then copies its Buffer contents — all of that sits
    // inside the same _SEH2_TRY in the C, so it's reimplemented here with
    // fault-tolerant primitives instead of calling the imported
    // RtlCopyUnicodeString (which does bare reads/copies with no fault
    // protection of its own).
    let probe_status = SafeProbeForRead(
        puszKLID as *mut core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
        1,
    );
    if !NT_SUCCESS(probe_status) {
        SetLastNtError(probe_status);
        return core::ptr::null_mut();
    }

    let mut capturedKLID: _UNICODE_STRING = core::mem::zeroed();
    if ntkrnl_sys::probe::safe_copy_raw(
        core::ptr::addr_of_mut!(capturedKLID) as *mut core::ffi::c_void,
        puszKLID as *const core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
    )
    .is_err()
    {
        SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
        return core::ptr::null_mut();
    }

    let probe_status = SafeProbeForRead(
        capturedKLID.Buffer as *mut core::ffi::c_void,
        core::mem::size_of::<USHORT>(),
        1,
    );
    if !NT_SUCCESS(probe_status) {
        SetLastNtError(probe_status);
        return core::ptr::null_mut();
    }

    // RtlCopyUnicodeString body: Length = min(Source->Length, Dest->MaximumLength);
    // RtlCopyMemory(Dest->Buffer, Source->Buffer, Length); Dest->Length = Length;
    // NUL-terminate if room remains.
    let copy_len = core::cmp::min(capturedKLID.Length, uszSafeKLID.MaximumLength) as usize;
    if copy_len > 0
        && ntkrnl_sys::probe::safe_copy_raw(
            uszSafeKLID.Buffer as *mut core::ffi::c_void,
            capturedKLID.Buffer as *const core::ffi::c_void,
            copy_len,
        )
        .is_err()
    {
        SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
        return core::ptr::null_mut();
    }
    uszSafeKLID.Length = copy_len as USHORT;
    if uszSafeKLID.Length < uszSafeKLID.MaximumLength {
        *uszSafeKLID
            .Buffer
            .add(copy_len / core::mem::size_of::<WCHAR>()) = 0;
    }

    UserEnterExclusive();

    hSafeFile = if !hFile.is_null() {
        IntVerifyKeyboardFileHandle(hFile)
    } else {
        core::ptr::null_mut()
    };
    pWinSta = IntGetProcessWindowStation(core::ptr::null_mut());
    hRetKL = co_IntLoadKeyboardLayoutEx(
        pWinSta,
        hSafeFile,
        hOldKL,
        &mut uszSafeKLID,
        dwNewKL as HKL,
        Flags,
    );
    if !hSafeFile.is_null() {
        ZwClose(hSafeFile);
    }

    UserLeave();
    hRetKL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserActivateKeyboardLayout(
    hKL: HKL,
    Flags: ULONG,
) -> HKL {
    let pWinSta: PWINSTATION_OBJECT;
    let hOldKL: HKL;

    UserEnterExclusive();

    pWinSta = IntGetProcessWindowStation(core::ptr::null_mut());
    hOldKL = co_IntActivateKeyboardLayout(pWinSta, hKL, Flags, core::ptr::null_mut());
    UserLeave();

    hOldKL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUnloadKeyboardLayout(hKl: HKL) -> BOOL {
    let ret: BOOL;
    let pWinSta: PWINSTATION_OBJECT;

    UserEnterExclusive();

    pWinSta = IntGetProcessWindowStation(core::ptr::null_mut());
    ret = IntUnloadKeyboardLayout(pWinSta, hKl);

    UserLeave();
    ret
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn RegOpenKey(pwszKeyName: *const u16, phkResult: *mut HKEY) -> NTSTATUS;
    fn RegQueryValue(hkey: HKEY, pwszValueName: *const u16, ulType: u32, pvData: *mut core::ffi::c_void, pcbValue: *mut u32) -> NTSTATUS;
    fn co_ClientImmLoadLayout(hKL: HKL, piiex: PIMEINFOEX) -> u8;
}
extern "C" {
    fn co_IntShellHookNotify(Message: WPARAM, wParam: WPARAM, lParam: LPARAM);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}