use win32k_sys::win32k::*;
// §5: cross-crate kernel types — these Rtl/Zw imports use ntkrnl_sys::ddk's
// _UNICODE_STRING type. The win32k_sys::win32k::_UNICODE_STRING in the
// `pGdiDriverInfo->DriverName` field is layout-identical, so we cast pointers
// at the call site rather than re-bindgen the rtl APIs here.
use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag as DdkExAllocatePoolWithTag,
    ExFreePoolWithTag as DdkExFreePoolWithTag,
    RtlAppendUnicodeToString as DdkRtlAppendUnicodeToString,
    RtlCopyUnicodeString as DdkRtlCopyUnicodeString,
    RtlEqualUnicodeString as DdkRtlEqualUnicodeString,
    RtlImageDirectoryEntryToData as DdkRtlImageDirectoryEntryToData,
    RtlInitUnicodeString as DdkRtlInitUnicodeString,
    ZwSetSystemInformation as DdkZwSetSystemInformation,
    _POOL_TYPE_PagedPool,
    _SYSTEM_INFORMATION_CLASS_SystemLoadGdiDriverInformation,
    _UNICODE_STRING as DdkUNICODE_STRING,
};
use ntkrnl_sys::rtl::{
    RtlZeroMemory as DdkRtlZeroMemory,
    RtlCopyMemory as DdkRtlCopyMemory,
};
use ntkrnl_sys::base::{
    NT_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY, STATUS_NOT_IMPLEMENTED,
    STATUS_SUCCESS,
};

// GDITAG_LDEV ('Lvde'), GDITAG_DEVMODE ('Dmde')  — win32k pool tags. C uses
// the GDITAG_* macros; rustconv didn't pull them in, so define inline (the
// big-endian 4-char literals translate via from_le_bytes per AGENTS.md).
const GDITAG_LDEV: u32 = u32::from_le_bytes(*b"Ldev");
const GDITAG_DEVMODE: u32 = u32::from_le_bytes(*b"Dmde");

// The original translation built every wide-string arg as a BYTE literal cast
// to `*const u16` (`b"...".as_ptr() as *const u16`) — 1 byte/char reinterpreted
// as 2-byte WCHARs yields garbage (e.g. `\S` → 0x535C instead of L'\'). That
// corrupted the driver path so the display driver never loaded → NtUserInitialize
// 0xc0000001. This macro materializes a proper NUL-terminated UTF-16 static
// (ASCII source) and returns its `*const u16`. (Same shape as freetype.rs.)
macro_rules! wide_str {
    ($s:literal) => {{
        const N: usize = $s.len() + 1;
        static W: [u16; N] = {
            let b = $s.as_bytes();
            let mut w = [0u16; N];
            let mut i = 0;
            while i < b.len() { w[i] = b[i] as u16; i += 1; }
            w
        };
        W.as_ptr()
    }};
}

// ntoskrnl provides __ImageBase as a linker symbol pointing at this DLL's
// PE base. Rust sees it as an extern static — `&__ImageBase` then yields
// the base address, mirroring the C `&__ImageBase` idiom.
extern "C" {
    static __ImageBase: u8;
}
// DriverEntry is win32k's own entry point — defined in win32ss/user/ntuser/main.c
// as APIENTRY (= stdcall on i386, cdecl elsewhere). Takes (PDRIVER_OBJECT, PUNICODE_STRING)
// → `_DriverEntry@8` on i386.
extern "system" {
    fn DriverEntry(DriverObject: PVOID, RegistryPath: PUNICODE_STRING) -> NTSTATUS;
}

extern "C" {
    fn _stricmp(s1: LPSTR, s2: LPSTR) -> i32;
    fn _strnicmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn wcslen(s: LPWSTR) -> usize;
    fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32;
}

// EngpGetDisplayDriverParameters lives in win32ss/gdi/eng/multidisp.c
// (plain cdecl in C). Declare as extern "C".
extern "C" {
    fn EngpGetDisplayDriverParameters( pGraphicsDevice: PGRAPHICS_DEVICE, pdm: PDEVMODEW, ) -> i32;
}

// C debug macros — silenced via macro_rules! (variadic no-op).
#[allow(unused_macros)] macro_rules! TRACE { ($($t:tt)*) => { () }; }
#[allow(unused_macros)] macro_rules! ERR { ($($t:tt)*) => { () }; }
#[allow(unused_macros)] macro_rules! WARN { ($($t:tt)*) => { () }; }

// FORCEINLINE NTSTRSAFEAPI in ddk/ntstrsafe.h:174 — inline-expand.
// Returns STATUS_SUCCESS (0) on success, STATUS_BUFFER_OVERFLOW (0xC0000023u32 as i32) on overflow.
#[inline]
unsafe fn RtlStringCbCopyW(dest: PWSTR, cb_dest: usize, src: PWSTR) -> i32 {
    let cch_dest = cb_dest / core::mem::size_of::<u16>();
    if cch_dest == 0 {
        return 0x80000005u32 as i32; // STATUS_BUFFER_OVERFLOW
    }
    let mut i = 0usize;
    while i < cch_dest - 1 {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 { return 0; }
        i += 1;
    }
    // No NUL fit — terminate and signal overflow.
    *dest.add(cch_dest - 1) = 0;
    if *src.add(i) == 0 { 0 } else { 0x80000005u32 as i32 } // STATUS_BUFFER_OVERFLOW (truncated success)
}

// Local InitializeListHead / InsertHeadList / RemoveEntryList specialised to
// the win32k_sys::win32k::LIST_ENTRY type. ntkrnl_sys::base provides these
// for its own LIST_ENTRY but the type identities differ (§5).
#[inline]
unsafe fn InitializeListHead(head: *mut LIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}
#[inline]
unsafe fn InsertHeadList(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    let flink = (*head).Flink;
    (*entry).Flink = flink;
    (*entry).Blink = head;
    (*flink).Blink = entry;
    (*head).Flink = entry;
}
#[inline]
unsafe fn RemoveEntryList(entry: *mut LIST_ENTRY) -> bool {
    let flink = (*entry).Flink;
    let blink = (*entry).Blink;
    (*blink).Flink = flink;
    (*flink).Blink = blink;
    flink == blink
}

// Small adapters around the ntkrnl_sys::ddk APIs that take win32k_sys
// pointer types and cast to the ddk types at the boundary. This keeps the
// translated bodies readable.
#[inline]
unsafe fn ExAllocatePoolWithTag(pool: u32, n: SIZE_T, tag: u32) -> PVOID {
    DdkExAllocatePoolWithTag(pool, n as _, tag) as PVOID
}
// FORCEINLINE in xdk/exfuncs.h:589 — inline-expand here (DdkExAllocatePoolZero
// is a bindgen extern with no linker symbol).
#[inline]
unsafe fn ExAllocatePoolZero(pool: u32, n: SIZE_T, tag: u32) -> PVOID {
    let p = DdkExAllocatePoolWithTag(pool, n as _, tag) as PVOID;
    if !p.is_null() {
        DdkRtlZeroMemory(p as _, n as _);
    }
    p
}
#[inline]
unsafe fn ExFreePoolWithTag(p: PVOID, tag: u32) {
    DdkExFreePoolWithTag(p as _, tag)
}
#[inline]
unsafe fn RtlZeroMemory(dst: PVOID, len: SIZE_T) {
    DdkRtlZeroMemory(dst as _, len as _)
}
#[inline]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, len: SIZE_T) {
    DdkRtlCopyMemory(dst as _, src as _, len as _)
}
#[inline]
unsafe fn RtlInitUnicodeString(s: *mut UNICODE_STRING, src: *const u16) {
    DdkRtlInitUnicodeString(s as *mut DdkUNICODE_STRING, src)
}
// FORCEINLINE in ndk/rtlfuncs.h:181 — inline-expand here.
#[inline]
unsafe fn RtlInitEmptyUnicodeString(s: *mut UNICODE_STRING, buf: *mut u16, sz: u16) {
    (*s).Length = 0;
    (*s).MaximumLength = sz;
    (*s).Buffer = buf;
}
#[inline]
unsafe fn RtlAppendUnicodeToString(s: *mut UNICODE_STRING, src: *const u16) -> NTSTATUS {
    DdkRtlAppendUnicodeToString(s as *mut DdkUNICODE_STRING, src)
}
#[inline]
unsafe fn RtlCopyUnicodeString(dst: *mut UNICODE_STRING, src: *const UNICODE_STRING) {
    DdkRtlCopyUnicodeString(dst as *mut DdkUNICODE_STRING, src as *const DdkUNICODE_STRING)
}
#[inline]
unsafe fn RtlEqualUnicodeString(
    s1: *const UNICODE_STRING,
    s2: *const UNICODE_STRING,
    ci: u8,
) -> u8 {
    DdkRtlEqualUnicodeString(
        s1 as *const DdkUNICODE_STRING,
        s2 as *const DdkUNICODE_STRING,
        ci,
    )
}
#[inline]
unsafe fn RtlImageDirectoryEntryToData(
    base: PVOID,
    mapped: u8,
    dir_entry: u16,
    size: *mut u32,
) -> PVOID {
    DdkRtlImageDirectoryEntryToData(base as _, mapped, dir_entry, size) as PVOID
}
#[inline]
unsafe fn ZwSetSystemInformation(class: u32, info: PVOID, len: u32) -> NTSTATUS {
    DdkZwSetSystemInformation(class, info as _, len)
}

const PagedPool: u32 = _POOL_TYPE_PagedPool as u32;
const SystemLoadGdiDriverInformation: u32 =
    _SYSTEM_INFORMATION_CLASS_SystemLoadGdiDriverInformation as u32;
const LDEV_DEVICE_DISPLAY: u32 = LDEVTYPE_LDEV_DEVICE_DISPLAY;
const LDEV_IMAGE: u32 = LDEVTYPE_LDEV_IMAGE;
const IMAGE_DIRECTORY_ENTRY_EXPORT_U16: u16 = IMAGE_DIRECTORY_ENTRY_EXPORT as u16;

// ---- file-scope preamble (translated) ----
static mut ghsemLDEVList: HSEMAPHORE = core::ptr::null_mut();
static mut gleLdevListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
static mut gpldevWin32k: *mut LDEVOBJ = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn InitLDEVImpl() -> NTSTATUS {
    let mut cbSize: ULONG = 0;

    InitializeListHead(&raw mut gleLdevListHead);

    ghsemLDEVList = EngCreateSemaphore();
    if ghsemLDEVList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    gpldevWin32k = ExAllocatePoolWithTag(
        PagedPool,
        (core::mem::size_of::<LDEVOBJ>() + core::mem::size_of::<SYSTEM_GDI_DRIVER_INFORMATION>()) as SIZE_T,
        GDITAG_LDEV,
    ) as *mut LDEVOBJ;
    if gpldevWin32k.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*gpldevWin32k).leLink.Flink = core::ptr::null_mut();
    (*gpldevWin32k).leLink.Blink = core::ptr::null_mut();
    (*gpldevWin32k).ldevtype = LDEV_DEVICE_DISPLAY;
    (*gpldevWin32k).cRefs = 1;
    (*gpldevWin32k).ulDriverVersion = GDI_ENGINE_VERSION;
    (*gpldevWin32k).pGdiDriverInfo = gpldevWin32k.add(1) as PSYSTEM_GDI_DRIVER_INFORMATION;
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!((*(*gpldevWin32k).pGdiDriverInfo).DriverName),
        wide_str!("\\SystemRoot\\System32\\win32k.sys"),
    );
    (*(*gpldevWin32k).pGdiDriverInfo).ImageAddress = &__ImageBase as *const _ as PVOID;
    (*(*gpldevWin32k).pGdiDriverInfo).SectionPointer = core::ptr::null_mut();
    (*(*gpldevWin32k).pGdiDriverInfo).EntryPoint = DriverEntry as PVOID;
    (*(*gpldevWin32k).pGdiDriverInfo).ExportSectionPointer = RtlImageDirectoryEntryToData(
        &__ImageBase as *const _ as PVOID,
        1u8,
        IMAGE_DIRECTORY_ENTRY_EXPORT_U16,
        core::ptr::addr_of_mut!(cbSize),
    ) as PIMAGE_EXPORT_DIRECTORY;
    (*(*gpldevWin32k).pGdiDriverInfo).ImageLength = 0;

    STATUS_SUCCESS
}

unsafe fn LDEVOBJ_AllocLDEV(
    ldevtype: LDEVTYPE,
) -> PLDEVOBJ {
    let pldev: PLDEVOBJ;

    pldev = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<LDEVOBJ>() as SIZE_T, GDITAG_LDEV) as PLDEVOBJ;
    if pldev.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(pldev as PVOID, core::mem::size_of::<LDEVOBJ>() as SIZE_T);

    (*pldev).ldevtype = ldevtype;

    pldev
}

unsafe fn LDEVOBJ_vFreeLDEV(pldev: PLDEVOBJ) {
    ExFreePoolWithTag(pldev as PVOID, GDITAG_LDEV);
}

unsafe fn LDEVOBJ_bLoadImage(
    pldev: PLDEVOBJ,
    pustrPathName: PUNICODE_STRING,
) -> u8 {
    let pDriverInfo: PSYSTEM_GDI_DRIVER_INFORMATION;
    let Status: i32;
    let cbSize: u32;

    cbSize = (core::mem::size_of::<SYSTEM_GDI_DRIVER_INFORMATION>() + (*pustrPathName).Length as usize) as u32;
    pDriverInfo = ExAllocatePoolWithTag(PagedPool, cbSize as SIZE_T, GDITAG_LDEV) as PSYSTEM_GDI_DRIVER_INFORMATION;
    if pDriverInfo.is_null() {
        return 0u8;
    }

    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!((*pDriverInfo).DriverName),
        pDriverInfo.offset(1) as PWSTR,
        (*pustrPathName).Length,
    );
    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!((*pDriverInfo).DriverName),
        pustrPathName,
    );

    Status = ZwSetSystemInformation(
        SystemLoadGdiDriverInformation,
        pDriverInfo as PVOID,
        core::mem::size_of::<SYSTEM_GDI_DRIVER_INFORMATION>() as u32,
    );
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(pDriverInfo as PVOID, GDITAG_LDEV);
        return 0u8;
    }

    (*pldev).pGdiDriverInfo = pDriverInfo;

    1u8
}

unsafe fn LDEVOBJ_bEnableDriver(
    pldev: PLDEVOBJ,
    pfnEnableDriver: PFN_DrvEnableDriver,
) -> u8 {
    let mut ded: DRVENABLEDATA = core::mem::zeroed();
    let mut i: u32;

    if (*pldev).ldevtype == LDEV_IMAGE {
        return 1u8;
    }

    core::ptr::write_bytes(&mut ded as *mut DRVENABLEDATA, 0, 1);
    if pfnEnableDriver.unwrap()(GDI_ENGINE_VERSION, core::mem::size_of::<DRVENABLEDATA>() as u32, &mut ded) == 0 {
        return 0u8;
    }

    (*pldev).ulDriverVersion = ded.iDriverVersion;

    // apfn lives in the unnamed union (`__bindgen_anon_1`) — access via the
    // union accessor; the pdrvfn entries are `DRVFN` values, so one deref.
    // PFN is `Option<unsafe extern "C" fn() -> LONG_PTR>` which is layout-
    // identical to a raw pointer (None == NULL); transmute to PVOID for
    // apfn[] storage.
    i = 0;
    while i < ded.c {
        let entry = &*ded.pdrvfn.offset(i as isize);
        (*pldev).__bindgen_anon_1.apfn[entry.iFunc as usize] =
            core::mem::transmute::<PFN, PVOID>(entry.pfn);
        i += 1;
    }

    return 1u8;
}

unsafe fn LDEVOBJ_vDisableDriver(pldev: PLDEVOBJ) {
    if (*pldev).ldevtype == LDEV_IMAGE {
        return;
    }

    if (*pldev).__bindgen_anon_1.pfn.DisableDriver.is_some() {
        (*pldev).__bindgen_anon_1.pfn.DisableDriver.unwrap()();
    }
}

unsafe fn LDEVOBJ_pvFindImageProcAddress(
    pldev: PLDEVOBJ,
    pszProcName: LPSTR,
) -> PVOID {
    let pvImageBase: PVOID;
    let pExportDir: PIMAGE_EXPORT_DIRECTORY;
    let mut pvProcAddress: PVOID = core::ptr::null_mut();
    let pOrdinals: PUSHORT;
    let pNames: PULONG;
    let pAddresses: PULONG;
    let mut i: ULONG;

    pvImageBase = (*(*pldev).pGdiDriverInfo).ImageAddress;
    pExportDir = (*(*pldev).pGdiDriverInfo).ExportSectionPointer;
    if pExportDir.is_null() {
        return core::ptr::null_mut();
    }

    pNames = ((pvImageBase as ULONG_PTR) + (*pExportDir).AddressOfNames as ULONG_PTR) as PULONG;
    pOrdinals = ((pvImageBase as ULONG_PTR) + (*pExportDir).AddressOfNameOrdinals as ULONG_PTR) as PUSHORT;
    pAddresses = ((pvImageBase as ULONG_PTR) + (*pExportDir).AddressOfFunctions as ULONG_PTR) as PULONG;

    i = 0;
    while i < (*pExportDir).NumberOfNames {
        let pName = ((pvImageBase as ULONG_PTR) + (*pNames.add(i as usize)) as ULONG_PTR) as LPSTR;
        if _stricmp(pszProcName, pName) == 0 {
            let ordinal = *pOrdinals.add(i as usize);
            pvProcAddress = ((pvImageBase as ULONG_PTR) + (*pAddresses.add(ordinal as usize)) as ULONG_PTR) as PVOID;
            break;
        }
        i += 1;
    }

    pvProcAddress
}

unsafe fn LDEVOBJ_bUnloadImage(pldev: PLDEVOBJ) -> BOOLEAN {
    let mut Status: NTSTATUS;

    TRACE!(
        b"LDEVOBJ_bUnloadImage('%wZ')\n\0".as_ptr() as *const i8,
        core::ptr::addr_of!((*(*pldev).pGdiDriverInfo).DriverName),
    );

    // UNIMPLEMENTED macro is a DPRINT debug-only side effect; safe to elide.
    Status = STATUS_NOT_IMPLEMENTED;

    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    ExFreePoolWithTag((*pldev).pGdiDriverInfo as PVOID, GDITAG_LDEV);
    (*pldev).pGdiDriverInfo = core::ptr::null_mut();

    return 1u8;
}

// Plain cdecl in C (ldevobj.h:50 — no NTAPI/APIENTRY).
#[no_mangle]
pub unsafe extern "C" fn LDEVOBJ_pLoadInternal(
    pfnEnableDriver: PFN_DrvEnableDriver,
    ldevtype: u32,
) -> PLDEVOBJ {
    let mut pldev: PLDEVOBJ;

    EngAcquireSemaphore(ghsemLDEVList);

    pldev = LDEVOBJ_AllocLDEV(ldevtype);
    if pldev.is_null() {
        EngReleaseSemaphore(ghsemLDEVList);
        return core::ptr::null_mut();
    }

    if LDEVOBJ_bEnableDriver(pldev, pfnEnableDriver) == 0 {
        LDEVOBJ_vFreeLDEV(pldev);
        pldev = core::ptr::null_mut();
        EngReleaseSemaphore(ghsemLDEVList);
        return pldev;
    }

    InsertHeadList(core::ptr::addr_of_mut!(gleLdevListHead), core::ptr::addr_of_mut!((*pldev).leLink));

    (*pldev).cRefs += 1;

    EngReleaseSemaphore(ghsemLDEVList);

    pldev
}

#[no_mangle]
pub unsafe extern "system" fn LDEVOBJ_pLoadDriver(
    pwszDriverName: LPWSTR,
    ldevtype: u32,
) -> PLDEVOBJ {
    let mut acwBuffer: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut pleLink: PLIST_ENTRY;
    let mut pldev: PLDEVOBJ = core::ptr::null_mut();
    let mut strDriverName: _UNICODE_STRING = core::mem::zeroed();
    let mut cwcLength: usize;
    let mut pwsz: LPWSTR;

    TRACE!(b"LDEVOBJ_pLoadDriver(%ls, %lu)\n\0".as_ptr() as *const i8, pwszDriverName, ldevtype);

    // The acwBuffer occupies MAX_PATH (=260) u16s so the byte size (520)
    // would wrap u16 (max 65535) — clamp to u16::MAX defensively (mirrors
    // the C `sizeof(acwBuffer)` truncation; in practice 520 fits u16 fine).
    let acw_bytes = core::mem::size_of_val(&acwBuffer);
    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(strDriverName),
        acwBuffer.as_mut_ptr(),
        if acw_bytes > u16::MAX as usize { u16::MAX } else { acw_bytes as u16 },
    );

    RtlAppendUnicodeToString(core::ptr::addr_of_mut!(strDriverName), wide_str!("\\SystemRoot\\System32\\"));

    cwcLength = wcslen(pwszDriverName);

    pwsz = pwszDriverName.add(cwcLength);
    while pwsz > pwszDriverName {
        if (*pwsz == '\\' as u16) && (_wcsnicmp(pwsz, wide_str!("\\system32\\"), 10) == 0) {
            pwsz = pwsz.add(10);
            break;
        }
        pwsz = pwsz.sub(1);
    }

    RtlAppendUnicodeToString(core::ptr::addr_of_mut!(strDriverName), pwsz);

    if (cwcLength < 4) ||
       ((_wcsnicmp(pwszDriverName.add(cwcLength - 4), wide_str!(".dll"), 4) != 0) &&
        (_wcsnicmp(pwszDriverName.add(cwcLength - 4), wide_str!(".sys"), 4) != 0)) {
        RtlAppendUnicodeToString(core::ptr::addr_of_mut!(strDriverName), wide_str!(".dll"));
    }

    EngAcquireSemaphore(ghsemLDEVList);

    pleLink = gleLdevListHead.Flink;
    'search: loop {
        if pleLink == core::ptr::addr_of_mut!(gleLdevListHead) as PLIST_ENTRY {
            break 'search;
        }

        pldev = (pleLink as *mut u8).offset(-(core::mem::offset_of!(LDEVOBJ, leLink) as isize)) as PLDEVOBJ;

        if !(*pldev).pGdiDriverInfo.is_null() {
            if RtlEqualUnicodeString(
                core::ptr::addr_of!((*(*pldev).pGdiDriverInfo).DriverName),
                core::ptr::addr_of!(strDriverName),
                1u8,
            ) != 0 {
                break 'search;
            }
        }

        pleLink = (*pleLink).Flink;
    }

    if pleLink == core::ptr::addr_of_mut!(gleLdevListHead) as PLIST_ENTRY {
        pldev = LDEVOBJ_AllocLDEV(ldevtype);
        if pldev.is_null() {
            ERR!(b"Could not allocate LDEV\n\0".as_ptr() as *const i8);
            EngReleaseSemaphore(ghsemLDEVList);
            TRACE!(b"LDEVOBJ_pLoadDriver returning %p\n\0".as_ptr() as *const i8, pldev);
            return pldev;
        }

        if LDEVOBJ_bLoadImage(pldev, core::ptr::addr_of_mut!(strDriverName)) == 0 {
            LDEVOBJ_vFreeLDEV(pldev);
            pldev = core::ptr::null_mut();
            ERR!(b"LDEVOBJ_bLoadImage failed\n\0".as_ptr() as *const i8);
            EngReleaseSemaphore(ghsemLDEVList);
            TRACE!(b"LDEVOBJ_pLoadDriver returning %p\n\0".as_ptr() as *const i8, pldev);
            return pldev;
        }

        // EntryPoint is PVOID in SYSTEM_GDI_DRIVER_INFORMATION but the C
        // calls it as a `PFN_DrvEnableDriver` (Option<fn>). Transmute pointer
        // ↔ Option<fn> (Option<extern fn> is layout-identical to a pointer).
        let entry_point: PFN_DrvEnableDriver =
            core::mem::transmute::<PVOID, PFN_DrvEnableDriver>(
                (*(*pldev).pGdiDriverInfo).EntryPoint,
            );
        if LDEVOBJ_bEnableDriver(pldev, entry_point) == 0 {
            ERR!(b"LDEVOBJ_bEnableDriver failed\n\0".as_ptr() as *const i8);

            if LDEVOBJ_bUnloadImage(pldev) != 0 {
                LDEVOBJ_vFreeLDEV(pldev);
            } else {
                ERR!(b"Could not unload driver. Leaking memory\n\0".as_ptr() as *const i8);
            }
            pldev = core::ptr::null_mut();
            EngReleaseSemaphore(ghsemLDEVList);
            TRACE!(b"LDEVOBJ_pLoadDriver returning %p\n\0".as_ptr() as *const i8, pldev);
            return pldev;
        }

        InsertHeadList(core::ptr::addr_of_mut!(gleLdevListHead), core::ptr::addr_of_mut!((*pldev).leLink));
    }

    (*pldev).cRefs += 1;

    EngReleaseSemaphore(ghsemLDEVList);

    TRACE!(b"LDEVOBJ_pLoadDriver returning %p\n\0".as_ptr() as *const i8, pldev);
    pldev
}

unsafe fn LDEVOBJ_vDereference(pldev: PLDEVOBJ) {
    EngAcquireSemaphore(ghsemLDEVList);

    (*pldev).cRefs -= 1;

    if (*pldev).cRefs > 0 {
        EngReleaseSemaphore(ghsemLDEVList);
        return;
    }

    LDEVOBJ_vDisableDriver(pldev);

    if LDEVOBJ_bUnloadImage(pldev) != 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*pldev).leLink));
        LDEVOBJ_vFreeLDEV(pldev);
    } else {
        WARN!(
            b"Failed to unload driver '%wZ', trying to re-enable it.\n\0".as_ptr() as *const i8,
            core::ptr::addr_of!((*(*pldev).pGdiDriverInfo).DriverName),
        );
        let entry_point: PFN_DrvEnableDriver =
            core::mem::transmute::<PVOID, PFN_DrvEnableDriver>(
                (*(*pldev).pGdiDriverInfo).EntryPoint,
            );
        LDEVOBJ_bEnableDriver(pldev, entry_point);
        (*pldev).cRefs += 1;
    }

    EngReleaseSemaphore(ghsemLDEVList);
}

#[no_mangle]
pub unsafe extern "system" fn LDEVOBJ_ulGetDriverModes(
    pwszDriverName: LPWSTR,
    hDriver: HANDLE,
    ppdm: *mut PDEVMODEW,
) -> u32 {

    let mut pldev: PLDEVOBJ = core::ptr::null_mut();
    let mut cbSize: u32 = 0;
    let mut pdm: PDEVMODEW = core::ptr::null_mut();

    TRACE!(b"LDEVOBJ_ulGetDriverModes('%ls', %p)\n\0".as_ptr() as *const i8, pwszDriverName, hDriver);

    pldev = LDEVOBJ_pLoadDriver(pwszDriverName, LDEV_DEVICE_DISPLAY);
    if pldev.is_null() {
        *ppdm = pdm;
        return cbSize;
    }

    if (*pldev).__bindgen_anon_1.pfn.GetModes.is_none() {
        LDEVOBJ_vDereference(pldev);
        *ppdm = pdm;
        return cbSize;
    }

    cbSize = (*pldev).__bindgen_anon_1.pfn.GetModes.unwrap()(hDriver, 0, core::ptr::null_mut());
    if cbSize == 0 {
        ERR!(b"DrvGetModes returned 0\n\0".as_ptr() as *const i8);
        LDEVOBJ_vDereference(pldev);
        *ppdm = pdm;
        return cbSize;
    }

    pdm = ExAllocatePoolWithTag(PagedPool, cbSize as SIZE_T, GDITAG_DEVMODE) as PDEVMODEW;
    if pdm.is_null() {
        ERR!(b"Could not allocate devmodeinfo\n\0".as_ptr() as *const i8);
        LDEVOBJ_vDereference(pldev);
        *ppdm = pdm;
        return cbSize;
    }

    cbSize = (*pldev).__bindgen_anon_1.pfn.GetModes.unwrap()(hDriver, cbSize, pdm);
    if cbSize == 0 {
        ERR!(b"DrvrGetModes returned 0 on second call\n\0".as_ptr() as *const i8);
        ExFreePoolWithTag(pdm as PVOID, GDITAG_DEVMODE);
        pdm = core::ptr::null_mut();
    }

    LDEVOBJ_vDereference(pldev);

    *ppdm = pdm;
    cbSize
}

// C decl in ldevobj.h has no calling-convention attribute = cdecl.
// On i386 cdecl = `_LDEVOBJ_bBuildDevmodeList` (undecorated). Use extern "C".
#[no_mangle]
pub unsafe extern "C" fn LDEVOBJ_bBuildDevmodeList(
    pGraphicsDevice: PGRAPHICS_DEVICE,
) -> u8 {

    let mut pwsz: PWSTR;
    let mut pdminfo: PDEVMODEINFO;
    let mut pdm: PDEVMODEW = core::ptr::null_mut();
    let mut pdmEnd: PDEVMODEW;
    let mut i: u32;
    let mut cModes: u32 = 0;
    let mut cbSize: u32;
    let mut cbFull: u32;

    if !(*pGraphicsDevice).pdevmodeInfo.is_null() {
        return 1u8;
    }

    pwsz = (*pGraphicsDevice).pDiplayDrivers;

    while *pwsz != 0 {
        TRACE!(b"Trying driver: %ls\n\0".as_ptr() as *const i8, pwsz);
        cbSize = LDEVOBJ_ulGetDriverModes(pwsz, (*pGraphicsDevice).DeviceObject as HANDLE, core::ptr::addr_of_mut!(pdm));
        if cbSize == 0 {
            WARN!(b"Driver %ls returned no valid mode\n\0".as_ptr() as *const i8, pwsz);
            pwsz = pwsz.add(wcslen(pwsz) + 1);
            continue;
        }

        cbFull = cbSize + core::mem::offset_of!(DEVMODEINFO, adevmode) as u32;

        pdminfo = ExAllocatePoolWithTag(PagedPool, cbFull as SIZE_T, 0x444D4447u32) as PDEVMODEINFO;
        if pdminfo.is_null() {
            ERR!(b"Could not allocate devmodeinfo\n\0".as_ptr() as *const i8);
            ExFreePoolWithTag(pdm as PVOID, 0x444D4447u32);
            pwsz = pwsz.add(wcslen(pwsz) + 1);
            continue;
        }

        (*pdminfo).cbdevmode = cbSize;
        core::ptr::copy_nonoverlapping(pdm as *const u8, core::ptr::addr_of_mut!((*pdminfo).adevmode) as *mut u8, cbSize as usize);
        ExFreePoolWithTag(pdm as PVOID, 0x444D4447u32);

        (*pdminfo).pdmiNext = (*pGraphicsDevice).pdevmodeInfo;
        (*pGraphicsDevice).pdevmodeInfo = pdminfo;

        pdmEnd = (core::ptr::addr_of_mut!((*pdminfo).adevmode) as *mut u8).add((*pdminfo).cbdevmode as usize) as PDEVMODEW;
        pdm = core::ptr::addr_of_mut!((*pdminfo).adevmode) as PDEVMODEW;
        while (pdm.add(1) <= pdmEnd) && ((*pdm).dmSize != 0) {
            cModes += 1;

            RtlStringCbCopyW(
                core::ptr::addr_of_mut!((*pdm).dmDeviceName) as PWSTR,
                core::mem::size_of_val(&(*pdm).dmDeviceName),
                pwsz
            );

            pdm = (pdm as *mut u8).add(((*pdm).dmSize + (*pdm).dmDriverExtra) as usize) as PDEVMODEW;
        }

        pwsz = pwsz.add(wcslen(pwsz) + 1);
    }

    if (*pGraphicsDevice).pdevmodeInfo.is_null() || cModes == 0 {
        ERR!(b"No devmodes\n\0".as_ptr() as *const i8);
        return 0u8;
    }

    (*pGraphicsDevice).cDevModes = cModes;
    (*pGraphicsDevice).pDevModeList = ExAllocatePoolWithTag(
        PagedPool,
        ((cModes as usize) * core::mem::size_of::<DEVMODEENTRY>()) as SIZE_T,
        0x47444447u32
    ) as *mut DEVMODEENTRY;
    if (*pGraphicsDevice).pDevModeList.is_null() {
        ERR!(b"No devmode list\n\0".as_ptr() as *const i8);
        return 0u8;
    }

    pdminfo = (*pGraphicsDevice).pdevmodeInfo;
    i = 0;
    while !pdminfo.is_null() {
        pdmEnd = (core::ptr::addr_of_mut!((*pdminfo).adevmode) as *mut u8).add((*pdminfo).cbdevmode as usize) as PDEVMODEW;

        pdm = core::ptr::addr_of_mut!((*pdminfo).adevmode) as PDEVMODEW;
        while (pdm.add(1) <= pdmEnd) && ((*pdm).dmSize != 0) {
            TRACE!(
                b"    %S has mode %lux%lux%lu(%lu Hz)\n\0".as_ptr() as *const i8,
                core::ptr::addr_of!((*pdm).dmDeviceName),
                (*pdm).dmPelsWidth,
                (*pdm).dmPelsHeight,
                (*pdm).dmBitsPerPel,
                (*pdm).dmDisplayFrequency
            );

            (*(*pGraphicsDevice).pDevModeList.add(i as usize)).dwFlags = 0;
            (*(*pGraphicsDevice).pDevModeList.add(i as usize)).pdm = pdm;
            i += 1;

            pdm = (pdm as *mut u8).add(((*pdm).dmSize + (*pdm).dmDriverExtra) as usize) as PDEVMODEW;
        }

        pdminfo = (*pdminfo).pdmiNext;
    }
    TRACE!(b"Prepared %lu modes for %ls\n\0".as_ptr() as *const i8, (*pGraphicsDevice).cDevModes, (*pGraphicsDevice).pwszDescription);
    return 1u8;
}

unsafe fn LDEVOBJ_bGetClosestMode(
    pGraphicsDevice: PGRAPHICS_DEVICE,
    RequestedMode: PDEVMODEW,
    pSelectedMode: *mut PDEVMODEW,
) -> BOOL {
    let mut dmDiff: DEVMODEW = core::mem::zeroed();
    let mut pdmCurrent: PDEVMODEW;
    let mut pdmBest: PDEVMODEW = core::ptr::null_mut();
    let mut i: ULONG;

    dmDiff.dmPelsWidth = 0xffffffff;
    dmDiff.dmPelsHeight = 0xffffffff;
    dmDiff.dmBitsPerPel = 0xffffffff;
    dmDiff.dmDisplayFrequency = 0xffffffff;

    i = 0;
    while i < (*pGraphicsDevice).cDevModes {
        pdmCurrent = (*(*pGraphicsDevice).pDevModeList.offset(i as isize)).pdm;

        if (*RequestedMode).dmPelsWidth != 0 {
            let diff = if (*RequestedMode).dmPelsWidth > (*pdmCurrent).dmPelsWidth {
                (*RequestedMode).dmPelsWidth - (*pdmCurrent).dmPelsWidth
            } else {
                (*pdmCurrent).dmPelsWidth - (*RequestedMode).dmPelsWidth
            };
            if diff > dmDiff.dmPelsWidth {
                i += 1;
                continue;
            }
        }

        if (*RequestedMode).dmPelsHeight != 0 {
            let diff = if (*RequestedMode).dmPelsHeight > (*pdmCurrent).dmPelsHeight {
                (*RequestedMode).dmPelsHeight - (*pdmCurrent).dmPelsHeight
            } else {
                (*pdmCurrent).dmPelsHeight - (*RequestedMode).dmPelsHeight
            };
            if diff > dmDiff.dmPelsHeight {
                i += 1;
                continue;
            }
        }

        if (*RequestedMode).dmBitsPerPel != 0 {
            let diff = if (*RequestedMode).dmBitsPerPel > (*pdmCurrent).dmBitsPerPel {
                (*RequestedMode).dmBitsPerPel - (*pdmCurrent).dmBitsPerPel
            } else {
                (*pdmCurrent).dmBitsPerPel - (*RequestedMode).dmBitsPerPel
            };
            if diff > dmDiff.dmBitsPerPel {
                i += 1;
                continue;
            }
        }

        if (*RequestedMode).dmDisplayFrequency != 0 {
            let diff = if (*RequestedMode).dmDisplayFrequency > (*pdmCurrent).dmDisplayFrequency {
                (*RequestedMode).dmDisplayFrequency - (*pdmCurrent).dmDisplayFrequency
            } else {
                (*pdmCurrent).dmDisplayFrequency - (*RequestedMode).dmDisplayFrequency
            };
            if diff > dmDiff.dmDisplayFrequency {
                i += 1;
                continue;
            }
        }

        dmDiff.dmPelsWidth = if (*RequestedMode).dmPelsWidth > (*pdmCurrent).dmPelsWidth {
            (*RequestedMode).dmPelsWidth - (*pdmCurrent).dmPelsWidth
        } else {
            (*pdmCurrent).dmPelsWidth - (*RequestedMode).dmPelsWidth
        };
        dmDiff.dmPelsHeight = if (*RequestedMode).dmPelsHeight > (*pdmCurrent).dmPelsHeight {
            (*RequestedMode).dmPelsHeight - (*pdmCurrent).dmPelsHeight
        } else {
            (*pdmCurrent).dmPelsHeight - (*RequestedMode).dmPelsHeight
        };
        dmDiff.dmBitsPerPel = if (*RequestedMode).dmBitsPerPel > (*pdmCurrent).dmBitsPerPel {
            (*RequestedMode).dmBitsPerPel - (*pdmCurrent).dmBitsPerPel
        } else {
            (*pdmCurrent).dmBitsPerPel - (*RequestedMode).dmBitsPerPel
        };
        dmDiff.dmDisplayFrequency = if (*RequestedMode).dmDisplayFrequency > (*pdmCurrent).dmDisplayFrequency {
            (*RequestedMode).dmDisplayFrequency - (*pdmCurrent).dmDisplayFrequency
        } else {
            (*pdmCurrent).dmDisplayFrequency - (*RequestedMode).dmDisplayFrequency
        };
        pdmBest = pdmCurrent;

        i += 1;
    }

    *pSelectedMode = pdmBest;
    if !pdmBest.is_null() { 1 as BOOL } else { 0 as BOOL }
}

// Plain cdecl in C (no NTAPI/APIENTRY) — `extern "C"` on all arches.
#[no_mangle]
pub unsafe extern "C" fn LDEVOBJ_bProbeAndCaptureDevmode(
    pGraphicsDevice: PGRAPHICS_DEVICE,
    RequestedMode: PDEVMODEW,
    pSelectedMode: *mut PDEVMODEW,
    bSearchClosestMode: BOOL,
) -> BOOL {

    let mut dmSearch: DEVMODEW = core::mem::zeroed();
    let mut pdmCurrent: PDEVMODEW;
    let mut pdm: PDEVMODEW;
    let mut pdmSelected: PDEVMODEW = core::ptr::null_mut();
    let mut i: ULONG;
    let mut ulVirtualWidth: ULONG = 0;
    let mut ulVirtualHeight: ULONG = 0;
    let mut bResult: BOOL = 1;
    let Status: i32;

    if LDEVOBJ_bBuildDevmodeList(pGraphicsDevice) == 0 {
        return 0;
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!(dmSearch) as PVOID,
        core::mem::size_of::<DEVMODEW>() as SIZE_T,
    );
    Status = EngpGetDisplayDriverParameters(pGraphicsDevice, core::ptr::addr_of_mut!(dmSearch));
    if !NT_SUCCESS(Status) {
        ERR!(
            b"EngpGetDisplayDriverParameters() failed with status 0x%08x\n\0".as_ptr() as *const i8,
            Status,
        );
        return 0;
    }

    let mut bSearchClosestMode = bSearchClosestMode;
    bSearchClosestMode |= if (*RequestedMode).dmFields == 0 { 1 } else { 0 };

    if (*RequestedMode).dmFields & DM_BITSPERPEL != 0 && (*RequestedMode).dmBitsPerPel != 0 {
        dmSearch.dmBitsPerPel = (*RequestedMode).dmBitsPerPel;
    }
    if (*RequestedMode).dmFields & DM_PELSWIDTH != 0 && (*RequestedMode).dmPelsWidth != 0 {
        dmSearch.dmPelsWidth = (*RequestedMode).dmPelsWidth;
    }
    if (*RequestedMode).dmFields & DM_PELSHEIGHT != 0 && (*RequestedMode).dmPelsHeight != 0 {
        dmSearch.dmPelsHeight = (*RequestedMode).dmPelsHeight;
    }
    if (*RequestedMode).dmFields & DM_DISPLAYFREQUENCY != 0 && (*RequestedMode).dmDisplayFrequency != 0 {
        dmSearch.dmDisplayFrequency = (*RequestedMode).dmDisplayFrequency;
    }

    if (*RequestedMode).dmFields & (DM_PANNINGWIDTH | DM_PANNINGHEIGHT) == (DM_PANNINGWIDTH | DM_PANNINGHEIGHT)
        && (*RequestedMode).dmPanningWidth != 0
        && (*RequestedMode).dmPanningHeight != 0
        && (*RequestedMode).dmPanningWidth < dmSearch.dmPelsWidth
        && (*RequestedMode).dmPanningHeight < dmSearch.dmPelsHeight
    {
        ulVirtualWidth = (*RequestedMode).dmPelsWidth;
        ulVirtualHeight = (*RequestedMode).dmPelsHeight;
        dmSearch.dmPelsWidth = (*RequestedMode).dmPanningWidth;
        dmSearch.dmPelsHeight = (*RequestedMode).dmPanningHeight;
    } else if dmSearch.dmPanningWidth != 0
        && dmSearch.dmPanningHeight != 0
        && dmSearch.dmPanningWidth < dmSearch.dmPelsWidth
        && dmSearch.dmPanningHeight < dmSearch.dmPelsHeight
    {
        ulVirtualWidth = dmSearch.dmPelsWidth;
        ulVirtualHeight = dmSearch.dmPelsHeight;
        dmSearch.dmPelsWidth = dmSearch.dmPanningWidth;
        dmSearch.dmPelsHeight = dmSearch.dmPanningHeight;
    }

    if LDEVOBJ_bGetClosestMode(
        pGraphicsDevice,
        core::ptr::addr_of_mut!(dmSearch),
        core::ptr::addr_of_mut!(pdmSelected),
    ) != 0
    {
        if bSearchClosestMode != 0 {
            dmSearch.dmBitsPerPel = (*pdmSelected).dmBitsPerPel;
            dmSearch.dmPelsWidth = (*pdmSelected).dmPelsWidth;
            dmSearch.dmPelsHeight = (*pdmSelected).dmPelsHeight;
            dmSearch.dmDisplayFrequency = (*pdmSelected).dmDisplayFrequency;
        } else {
            if (*RequestedMode).dmFields & DM_BITSPERPEL == 0 || (*RequestedMode).dmBitsPerPel == 0 {
                dmSearch.dmBitsPerPel = (*pdmSelected).dmBitsPerPel;
            }
            if (*RequestedMode).dmFields & DM_PELSWIDTH == 0 || (*RequestedMode).dmPelsWidth == 0 {
                dmSearch.dmPelsWidth = (*pdmSelected).dmPelsWidth;
            }
            if (*RequestedMode).dmFields & DM_PELSHEIGHT == 0 || (*RequestedMode).dmPelsHeight == 0 {
                dmSearch.dmPelsHeight = (*pdmSelected).dmPelsHeight;
            }
            if (*RequestedMode).dmFields & DM_DISPLAYFREQUENCY == 0 || (*RequestedMode).dmDisplayFrequency == 0 {
                dmSearch.dmDisplayFrequency = (*pdmSelected).dmDisplayFrequency;
            }
        }
    }

    i = 0;
    while i < (*pGraphicsDevice).cDevModes {
        pdmCurrent = (*(*pGraphicsDevice).pDevModeList.offset(i as isize)).pdm;

        if (*pdmCurrent).dmBitsPerPel != dmSearch.dmBitsPerPel {
            i += 1;
            continue;
        }
        if (*pdmCurrent).dmPelsWidth != dmSearch.dmPelsWidth {
            i += 1;
            continue;
        }
        if (*pdmCurrent).dmPelsHeight != dmSearch.dmPelsHeight {
            i += 1;
            continue;
        }
        if (*pdmCurrent).dmDisplayFrequency != dmSearch.dmDisplayFrequency {
            i += 1;
            continue;
        }

        pdmSelected = pdmCurrent;
        break;
    }

    if pdmSelected.is_null() {
        ERR!(
            b"Requested mode not found (%dx%dx%d %d Hz)\n\0".as_ptr() as *const i8,
            dmSearch.dmPelsWidth,
            dmSearch.dmPelsHeight,
            dmSearch.dmBitsPerPel,
            dmSearch.dmDisplayFrequency,
        );
        return 0;
    }

    pdm = ExAllocatePoolZero(
        PagedPool,
        ((*pdmSelected).dmSize as usize + (*pdmSelected).dmDriverExtra as usize) as SIZE_T,
        0x65646d44,
    ) as PDEVMODEW;
    if pdm.is_null() {
        return 0;
    }

    RtlCopyMemory(
        pdm as PVOID,
        pdmSelected as *const core::ffi::c_void,
        (*pdmSelected).dmSize as SIZE_T,
    );
    RtlCopyMemory(
        (pdm as usize + (*pdm).dmSize as usize) as PVOID,
        (pdmSelected as usize + (*pdmSelected).dmSize as usize) as *const core::ffi::c_void,
        (*pdmSelected).dmDriverExtra as SIZE_T,
    );

    if ulVirtualWidth != 0
        && ulVirtualHeight != 0
        && (*pdm).dmPelsWidth < ulVirtualWidth
        && (*pdm).dmPelsHeight < ulVirtualHeight
    {
        (*pdm).dmFields |= DM_PANNINGWIDTH | DM_PANNINGHEIGHT;
        (*pdm).dmPanningWidth = (*pdm).dmPelsWidth;
        (*pdm).dmPanningHeight = (*pdm).dmPelsHeight;
        (*pdm).dmPelsWidth = ulVirtualWidth;
        (*pdm).dmPelsHeight = ulVirtualHeight;
    }

    *pSelectedMode = pdm;
    1
}
#[no_mangle]
pub unsafe extern "system" fn EngLoadImage(
    pwszDriverName: LPWSTR,
) -> HANDLE {
    LDEVOBJ_pLoadDriver(pwszDriverName, LDEV_IMAGE) as HANDLE
}
#[no_mangle]
pub unsafe extern "system" fn EngUnloadImage(
    hModule: HANDLE,
) {
    let pldev: PLDEVOBJ = hModule as PLDEVOBJ;

    LDEVOBJ_vDereference(pldev);
}
#[no_mangle]
pub unsafe extern "system" fn EngFindImageProcAddress(
    hModule: HANDLE,
    lpProcName: LPSTR,
) -> PVOID {
    let mut pldev: PLDEVOBJ = hModule as PLDEVOBJ;

    if pldev.is_null() {
        pldev = gpldevWin32k;
    }

    if _strnicmp(lpProcName, b"DrvEnableDriver\0".as_ptr() as LPSTR, 15) == 0 {
        return (*(*pldev).pGdiDriverInfo).EntryPoint;
    }

    return LDEVOBJ_pvFindImageProcAddress(pldev, lpProcName);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Support for logical devices
//  * FILE:             win32ss/gdi/eng/ldevobj.c
//  * PROGRAMER:        Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// #include <win32k.h>
// #define NDEBUG
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(EngLDev);
//
// #ifndef RVA_TO_ADDR
// #define RVA_TO_ADDR(Base,Rva) ((PVOID)(((ULONG_PTR)(Base)) + (Rva)))
// #endif
//
// /** Globals *******************************************************************/
//
// static HSEMAPHORE ghsemLDEVList;
// static LIST_ENTRY gleLdevListHead;
// static LDEVOBJ *gpldevWin32k = NULL;
//
//
// /** Private functions *********************************************************/
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// InitLDEVImpl(VOID)
// {
//     ULONG cbSize;
//
//     /* Initialize the LDEV list head */
//     InitializeListHead(&gleLdevListHead);
//
//     /* Initialize the loader lock */
//     ghsemLDEVList = EngCreateSemaphore();
//     if (!ghsemLDEVList)
//     {
//         ERR("Failed to create ghsemLDEVList\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Allocate a LDEVOBJ for win32k */
//     gpldevWin32k = ExAllocatePoolWithTag(PagedPool,
//                                          sizeof(LDEVOBJ) +
//                                          sizeof(SYSTEM_GDI_DRIVER_INFORMATION),
//                                          GDITAG_LDEV);
//     if (!gpldevWin32k)
//     {
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Initialize the LDEVOBJ for win32k */
//     gpldevWin32k->leLink.Flink = NULL;
//     gpldevWin32k->leLink.Blink = NULL;
//     gpldevWin32k->ldevtype = LDEV_DEVICE_DISPLAY;
//     gpldevWin32k->cRefs = 1;
//     gpldevWin32k->ulDriverVersion = GDI_ENGINE_VERSION;
//     gpldevWin32k->pGdiDriverInfo = (PVOID)(gpldevWin32k + 1);
//     RtlInitUnicodeString(&gpldevWin32k->pGdiDriverInfo->DriverName,
//                          L"\\SystemRoot\\System32\\win32k.sys");
//     gpldevWin32k->pGdiDriverInfo->ImageAddress = &__ImageBase;
//     gpldevWin32k->pGdiDriverInfo->SectionPointer = NULL;
//     gpldevWin32k->pGdiDriverInfo->EntryPoint = (PVOID)DriverEntry;
//     gpldevWin32k->pGdiDriverInfo->ExportSectionPointer =
//         RtlImageDirectoryEntryToData(&__ImageBase,
//                                      TRUE,
//                                      IMAGE_DIRECTORY_ENTRY_EXPORT,
//                                      &cbSize);
//     gpldevWin32k->pGdiDriverInfo->ImageLength = 0; // FIXME
//
//     return STATUS_SUCCESS;
// }
//
// static
// PLDEVOBJ
// LDEVOBJ_AllocLDEV(
//     _In_ LDEVTYPE ldevtype)
// {
//     PLDEVOBJ pldev;
//
//     /* Allocate the structure from paged pool */
//     pldev = ExAllocatePoolWithTag(PagedPool, sizeof(LDEVOBJ), GDITAG_LDEV);
//     if (!pldev)
//     {
//         ERR("Failed to allocate LDEVOBJ.\n");
//         return NULL;
//     }
//
//     /* Zero out the structure */
//     RtlZeroMemory(pldev, sizeof(LDEVOBJ));
//
//     /* Set the ldevtype */
//     pldev->ldevtype = ldevtype;
//
//     return pldev;
// }
//
// static
// VOID
// LDEVOBJ_vFreeLDEV(
//     _In_ _Post_ptr_invalid_ PLDEVOBJ pldev)
// {
//     /* Make sure we don't have a driver loaded */
//     ASSERT(pldev && pldev->pGdiDriverInfo == NULL);
//     ASSERT(pldev->cRefs == 0);
//
//     /* Free the memory */
//     ExFreePoolWithTag(pldev, GDITAG_LDEV);
// }
//
// static
// BOOL
// LDEVOBJ_bLoadImage(
//     _Inout_ PLDEVOBJ pldev,
//     _In_ PUNICODE_STRING pustrPathName)
// {
//     PSYSTEM_GDI_DRIVER_INFORMATION pDriverInfo;
//     NTSTATUS Status;
//     ULONG cbSize;
//
//     /* Make sure no image is loaded yet */
//     ASSERT(pldev && pldev->pGdiDriverInfo == NULL);
//
//     /* Allocate a SYSTEM_GDI_DRIVER_INFORMATION structure */
//     cbSize = sizeof(SYSTEM_GDI_DRIVER_INFORMATION) + pustrPathName->Length;
//     pDriverInfo = ExAllocatePoolWithTag(PagedPool, cbSize, GDITAG_LDEV);
//     if (!pDriverInfo)
//     {
//         ERR("Failed to allocate SYSTEM_GDI_DRIVER_INFORMATION\n");
//         return FALSE;
//     }
//
//     /* Initialize the UNICODE_STRING and copy the driver name */
//     RtlInitEmptyUnicodeString(&pDriverInfo->DriverName,
//                               (PWSTR)(pDriverInfo + 1),
//                               pustrPathName->Length);
//     RtlCopyUnicodeString(&pDriverInfo->DriverName, pustrPathName);
//
//     /* Try to load the driver */
//     Status = ZwSetSystemInformation(SystemLoadGdiDriverInformation,
//                                     pDriverInfo,
//                                     sizeof(SYSTEM_GDI_DRIVER_INFORMATION));
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to load a GDI driver: '%wZ', Status = 0x%lx\n",
//             pustrPathName, Status);
//
//         /* Free the allocated memory */
//         ExFreePoolWithTag(pDriverInfo, GDITAG_LDEV);
//         return FALSE;
//     }
//
//     /* Set the driver info */
//     pldev->pGdiDriverInfo = pDriverInfo;
//
//     /* Return success. */
//     return TRUE;
// }
//
// static
// BOOL
// LDEVOBJ_bEnableDriver(
//     _Inout_ PLDEVOBJ pldev,
//     _In_ PFN_DrvEnableDriver pfnEnableDriver)
// {
//     DRVENABLEDATA ded;
//     ULONG i;
//
//     TRACE("LDEVOBJ_bEnableDriver('%wZ')\n", &pldev->pGdiDriverInfo->DriverName);
//
//     ASSERT(pldev);
//     ASSERT(pldev->cRefs == 0);
//
//     if (pldev->ldevtype == LDEV_IMAGE)
//         return TRUE;
//
//     /* Call the drivers DrvEnableDriver function */
//     RtlZeroMemory(&ded, sizeof(ded));
//     if (!pfnEnableDriver(GDI_ENGINE_VERSION, sizeof(ded), &ded))
//     {
//         ERR("DrvEnableDriver failed\n");
//         return FALSE;
//     }
//
//     /* Copy the returned driver version */
//     pldev->ulDriverVersion = ded.iDriverVersion;
//
//     /* Fill the driver function array */
//     for (i = 0; i < ded.c; i++)
//     {
//         pldev->apfn[ded.pdrvfn[i].iFunc] = ded.pdrvfn[i].pfn;
//     }
//
//     /* Return success. */
//     return TRUE;
// }
//
// static
// VOID
// LDEVOBJ_vDisableDriver(
//     _Inout_ PLDEVOBJ pldev)
// {
//     ASSERT(pldev);
//     ASSERT(pldev->cRefs == 0);
//
//     TRACE("LDEVOBJ_vDisableDriver('%wZ')\n", &pldev->pGdiDriverInfo->DriverName);
//
//     if (pldev->ldevtype == LDEV_IMAGE)
//         return;
//
//     if (pldev->pfn.DisableDriver)
//     {
//         /* Call the unload function */
//         pldev->pfn.DisableDriver();
//     }
// }
//
// static
// PVOID
// LDEVOBJ_pvFindImageProcAddress(
//     _In_ PLDEVOBJ pldev,
//     _In_z_ LPSTR pszProcName)
// {
//     PVOID pvImageBase;
//     PIMAGE_EXPORT_DIRECTORY pExportDir;
//     PVOID pvProcAddress = NULL;
//     PUSHORT pOrdinals;
//     PULONG pNames, pAddresses;
//     ULONG i;
//
//     /* Make sure we have a driver info */
//     ASSERT(pldev && pldev->pGdiDriverInfo != NULL);
//
//     /* Get the pointer to the export directory */
//     pvImageBase = pldev->pGdiDriverInfo->ImageAddress;
//     pExportDir = pldev->pGdiDriverInfo->ExportSectionPointer;
//     if (!pExportDir)
//     {
//         ERR("LDEVOBJ_pvFindImageProcAddress: no export section found\n");
//         return NULL;
//     }
//
//     /* Get pointers to some tables */
//     pNames = RVA_TO_ADDR(pvImageBase, pExportDir->AddressOfNames);
//     pOrdinals = RVA_TO_ADDR(pvImageBase, pExportDir->AddressOfNameOrdinals);
//     pAddresses = RVA_TO_ADDR(pvImageBase, pExportDir->AddressOfFunctions);
//
//     /* Loop the export table */
//     for (i = 0; i < pExportDir->NumberOfNames; i++)
//     {
//         /* Compare the name */
//         if (_stricmp(pszProcName, RVA_TO_ADDR(pvImageBase, pNames[i])) == 0)
//         {
//             /* Found! Calculate the procedure address */
//             pvProcAddress = RVA_TO_ADDR(pvImageBase, pAddresses[pOrdinals[i]]);
//             break;
//         }
//     }
//
//     /* Return the address */
//     return pvProcAddress;
// }
//
// static
// BOOL
// LDEVOBJ_bUnloadImage(
//     _Inout_ PLDEVOBJ pldev)
// {
//     NTSTATUS Status;
//
//     /* Make sure we have a driver info */
//     ASSERT(pldev && pldev->pGdiDriverInfo != NULL);
//     ASSERT(pldev->cRefs == 0);
//
//     TRACE("LDEVOBJ_bUnloadImage('%wZ')\n", &pldev->pGdiDriverInfo->DriverName);
//
//     /* Unload the driver */
// #if 0
//     Status = ZwSetSystemInformation(SystemUnloadGdiDriverInformation,
//                                     &pldev->pGdiDriverInfo->SectionPointer,
//                                     sizeof(HANDLE));
// #else
//     /* Unfortunately, ntoskrnl allows unloading a driver, but fails loading
//      * it again with STATUS_IMAGE_ALREADY_LOADED. Prevent this problem by
//      * never unloading any driver.
//      */
//     UNIMPLEMENTED;
//     Status = STATUS_NOT_IMPLEMENTED;
// #endif
//     if (!NT_SUCCESS(Status))
//         return FALSE;
//
//     ExFreePoolWithTag(pldev->pGdiDriverInfo, GDITAG_LDEV);
//     pldev->pGdiDriverInfo = NULL;
//
//     return TRUE;
// }
//
// PLDEVOBJ
// LDEVOBJ_pLoadInternal(
//     _In_ PFN_DrvEnableDriver pfnEnableDriver,
//     _In_ ULONG ldevtype)
// {
//     PLDEVOBJ pldev;
//
//     TRACE("LDEVOBJ_pLoadInternal(%lu)\n", ldevtype);
//
//     /* Lock loader */
//     EngAcquireSemaphore(ghsemLDEVList);
//
//     /* Allocate a new LDEVOBJ */
//     pldev = LDEVOBJ_AllocLDEV(ldevtype);
//     if (!pldev)
//     {
//         ERR("Could not allocate LDEV\n");
//         goto leave;
//     }
//
//     /* Load the driver */
//     if (!LDEVOBJ_bEnableDriver(pldev, pfnEnableDriver))
//     {
//         ERR("LDEVOBJ_bEnableDriver failed\n");
//         LDEVOBJ_vFreeLDEV(pldev);
//         pldev = NULL;
//         goto leave;
//     }
//
//     /* Insert the LDEV into the global list */
//     InsertHeadList(&gleLdevListHead, &pldev->leLink);
//
//     /* Increase ref count */
//     pldev->cRefs++;
//
// leave:
//     /* Unlock loader */
//     EngReleaseSemaphore(ghsemLDEVList);
//
//     TRACE("LDEVOBJ_pLoadInternal returning %p\n", pldev);
//     return pldev;
// }
//
// PLDEVOBJ
// NTAPI
// LDEVOBJ_pLoadDriver(
//     _In_z_ LPWSTR pwszDriverName,
//     _In_ ULONG ldevtype)
// {
//     WCHAR acwBuffer[MAX_PATH];
//     PLIST_ENTRY pleLink;
//     PLDEVOBJ pldev;
//     UNICODE_STRING strDriverName;
//     SIZE_T cwcLength;
//     LPWSTR pwsz;
//
//     TRACE("LDEVOBJ_pLoadDriver(%ls, %lu)\n", pwszDriverName, ldevtype);
//     ASSERT(pwszDriverName);
//
//     /* Initialize buffer for the the driver name */
//     RtlInitEmptyUnicodeString(&strDriverName, acwBuffer, sizeof(acwBuffer));
//
//     /* Start path with systemroot */
//     RtlAppendUnicodeToString(&strDriverName, L"\\SystemRoot\\System32\\");
//
//     /* Get Length of given string */
//     cwcLength = wcslen(pwszDriverName);
//
//     /* Check if we have a system32 path given */
//     pwsz = pwszDriverName + cwcLength;
//     while (pwsz > pwszDriverName)
//     {
//         if ((*pwsz == L'\\') && (_wcsnicmp(pwsz, L"\\system32\\", 10) == 0))
//         {
//             /* Driver name starts after system32 */
//             pwsz += 10;
//             break;
//         }
//         pwsz--;
//     }
//
//     /* Append the driver name */
//     RtlAppendUnicodeToString(&strDriverName, pwsz);
//
//     /* MSDN says "The driver must include this suffix in the pwszDriver string."
//        But in fact it's optional. The function can also load .sys files without
//        appending the .dll extension. */
//     if ((cwcLength < 4) ||
//         ((_wcsnicmp(pwszDriverName + cwcLength - 4, L".dll", 4) != 0) &&
//          (_wcsnicmp(pwszDriverName + cwcLength - 4, L".sys", 4) != 0)) )
//     {
//         /* Append the .dll suffix */
//         RtlAppendUnicodeToString(&strDriverName, L".dll");
//     }
//
//     /* Lock loader */
//     EngAcquireSemaphore(ghsemLDEVList);
//
//     /* Search the List of LDEVS for the driver name */
//     for (pleLink = gleLdevListHead.Flink;
//          pleLink != &gleLdevListHead;
//          pleLink = pleLink->Flink)
//     {
//         pldev = CONTAINING_RECORD(pleLink, LDEVOBJ, leLink);
//
//         /* Check if the ldev is associated with a file */
//         if (pldev->pGdiDriverInfo)
//         {
//             /* Check for match (case insensitive) */
//             if (RtlEqualUnicodeString(&pldev->pGdiDriverInfo->DriverName, &strDriverName, TRUE))
//             {
//                 /* Image found in LDEV list */
//                 break;
//             }
//         }
//     }
//
//     /* Did we find one? */
//     if (pleLink == &gleLdevListHead)
//     {
//         /* No, allocate a new LDEVOBJ */
//         pldev = LDEVOBJ_AllocLDEV(ldevtype);
//         if (!pldev)
//         {
//             ERR("Could not allocate LDEV\n");
//             goto leave;
//         }
//
//         /* Load the image */
//         if (!LDEVOBJ_bLoadImage(pldev, &strDriverName))
//         {
//             LDEVOBJ_vFreeLDEV(pldev);
//             pldev = NULL;
//             ERR("LDEVOBJ_bLoadImage failed\n");
//             goto leave;
//         }
//
//         /* Load the driver */
//         if (!LDEVOBJ_bEnableDriver(pldev, pldev->pGdiDriverInfo->EntryPoint))
//         {
//             ERR("LDEVOBJ_bEnableDriver failed\n");
//
//             /* Unload the image. */
//             if (LDEVOBJ_bUnloadImage(pldev))
//                 LDEVOBJ_vFreeLDEV(pldev);
//             else
//                 ERR("Could not unload driver. Leaking memory\n");
//             pldev = NULL;
//             goto leave;
//         }
//
//         /* Insert the LDEV into the global list */
//         InsertHeadList(&gleLdevListHead, &pldev->leLink);
//     }
//
//     /* Increase ref count */
//     pldev->cRefs++;
//
// leave:
//     /* Unlock loader */
//     EngReleaseSemaphore(ghsemLDEVList);
//
//     TRACE("LDEVOBJ_pLoadDriver returning %p\n", pldev);
//     return pldev;
// }
//
// static
// VOID
// LDEVOBJ_vDereference(
//     _Inout_ PLDEVOBJ pldev)
// {
//     /* Lock loader */
//     EngAcquireSemaphore(ghsemLDEVList);
//
//     /* Decrement reference count */
//     ASSERT(pldev->cRefs > 0);
//     pldev->cRefs--;
//
//     /* More references left? */
//     if (pldev->cRefs > 0)
//     {
//         EngReleaseSemaphore(ghsemLDEVList);
//         return;
//     }
//
//     LDEVOBJ_vDisableDriver(pldev);
//
//     if (LDEVOBJ_bUnloadImage(pldev))
//     {
//         /* Remove ldev from the list */
//         RemoveEntryList(&pldev->leLink);
//
//         /* Free the driver info structure */
//         LDEVOBJ_vFreeLDEV(pldev);
//     }
//     else
//     {
//         WARN("Failed to unload driver '%wZ', trying to re-enable it.\n", &pldev->pGdiDriverInfo->DriverName);
//         LDEVOBJ_bEnableDriver(pldev, pldev->pGdiDriverInfo->EntryPoint);
//
//         /* Increment again reference count */
//         pldev->cRefs++;
//     }
//
//     /* Unlock loader */
//     EngReleaseSemaphore(ghsemLDEVList);
// }
//
// ULONG
// LDEVOBJ_ulGetDriverModes(
//     _In_ LPWSTR pwszDriverName,
//     _In_ HANDLE hDriver,
//     _Out_ PDEVMODEW *ppdm)
// {
//     PLDEVOBJ pldev = NULL;
//     ULONG cbSize = 0;
//     PDEVMODEW pdm = NULL;
//
//     TRACE("LDEVOBJ_ulGetDriverModes('%ls', %p)\n", pwszDriverName, hDriver);
//
//     pldev = LDEVOBJ_pLoadDriver(pwszDriverName, LDEV_DEVICE_DISPLAY);
//     if (!pldev)
//         goto cleanup;
//
//     /* Mirror drivers may omit this function */
//     if (!pldev->pfn.GetModes)
//         goto cleanup;
//
//     /* Call the driver to get the required size */
//     cbSize = pldev->pfn.GetModes(hDriver, 0, NULL);
//     if (!cbSize)
//     {
//         ERR("DrvGetModes returned 0\n");
//         goto cleanup;
//     }
//
//     /* Allocate a buffer for the DEVMODE array */
//     pdm = ExAllocatePoolWithTag(PagedPool, cbSize, GDITAG_DEVMODE);
//     if (!pdm)
//     {
//         ERR("Could not allocate devmodeinfo\n");
//         goto cleanup;
//     }
//
//     /* Call the driver again to fill the buffer */
//     cbSize = pldev->pfn.GetModes(hDriver, cbSize, pdm);
//     if (!cbSize)
//     {
//         /* Could not get modes */
//         ERR("DrvrGetModes returned 0 on second call\n");
//         ExFreePoolWithTag(pdm, GDITAG_DEVMODE);
//         pdm = NULL;
//     }
//
// cleanup:
//     if (pldev)
//         LDEVOBJ_vDereference(pldev);
//
//     *ppdm = pdm;
//     return cbSize;
// }
//
// BOOL
// LDEVOBJ_bBuildDevmodeList(
//     _Inout_ PGRAPHICS_DEVICE pGraphicsDevice)
// {
//     PWSTR pwsz;
//     PDEVMODEINFO pdminfo;
//     PDEVMODEW pdm, pdmEnd;
//     ULONG i, cModes = 0;
//     ULONG cbSize, cbFull;
//
//     if (pGraphicsDevice->pdevmodeInfo)
//         return TRUE;
//     ASSERT(pGraphicsDevice->pDevModeList == NULL);
//
//     pwsz = pGraphicsDevice->pDiplayDrivers;
//
//     /* Loop through the driver names
//      * This is a REG_MULTI_SZ string */
//     for (; *pwsz; pwsz += wcslen(pwsz) + 1)
//     {
//         /* Get the mode list from the driver */
//         TRACE("Trying driver: %ls\n", pwsz);
//         cbSize = LDEVOBJ_ulGetDriverModes(pwsz, pGraphicsDevice->DeviceObject, &pdm);
//         if (!cbSize)
//         {
//             WARN("Driver %ls returned no valid mode\n", pwsz);
//             continue;
//         }
//
//         /* Add space for the header */
//         cbFull = cbSize + FIELD_OFFSET(DEVMODEINFO, adevmode);
//
//         /* Allocate a buffer for the DEVMODE array */
//         pdminfo = ExAllocatePoolWithTag(PagedPool, cbFull, GDITAG_DEVMODE);
//         if (!pdminfo)
//         {
//             ERR("Could not allocate devmodeinfo\n");
//             ExFreePoolWithTag(pdm, GDITAG_DEVMODE);
//             continue;
//         }
//
//         pdminfo->cbdevmode = cbSize;
//         RtlCopyMemory(pdminfo->adevmode, pdm, cbSize);
//         ExFreePoolWithTag(pdm, GDITAG_DEVMODE);
//
//         /* Attach the mode info to the device */
//         pdminfo->pdmiNext = pGraphicsDevice->pdevmodeInfo;
//         pGraphicsDevice->pdevmodeInfo = pdminfo;
//
//         /* Loop all DEVMODEs */
//         pdmEnd = (DEVMODEW*)((PCHAR)pdminfo->adevmode + pdminfo->cbdevmode);
//         for (pdm = pdminfo->adevmode;
//              (pdm + 1 <= pdmEnd) && (pdm->dmSize != 0);
//              pdm = (DEVMODEW*)((PCHAR)pdm + pdm->dmSize + pdm->dmDriverExtra))
//         {
//             /* Count this DEVMODE */
//             cModes++;
//
//             /* Some drivers like the VBox driver don't fill the dmDeviceName
//                with the name of the display driver. So fix that here. */
//             RtlStringCbCopyW(pdm->dmDeviceName, sizeof(pdm->dmDeviceName), pwsz);
//         }
//     }
//
//     if (!pGraphicsDevice->pdevmodeInfo || cModes == 0)
//     {
//         ERR("No devmodes\n");
//         return FALSE;
//     }
//
//     /* Allocate an index buffer */
//     pGraphicsDevice->cDevModes = cModes;
//     pGraphicsDevice->pDevModeList = ExAllocatePoolWithTag(PagedPool,
//                                                           cModes * sizeof(DEVMODEENTRY),
//                                                           GDITAG_GDEVICE);
//     if (!pGraphicsDevice->pDevModeList)
//     {
//         ERR("No devmode list\n");
//         return FALSE;
//     }
//
//     /* Loop through all DEVMODEINFOs */
//     for (pdminfo = pGraphicsDevice->pdevmodeInfo, i = 0;
//          pdminfo;
//          pdminfo = pdminfo->pdmiNext)
//     {
//         /* Calculate End of the DEVMODEs */
//         pdmEnd = (DEVMODEW*)((PCHAR)pdminfo->adevmode + pdminfo->cbdevmode);
//
//         /* Loop through the DEVMODEs */
//         for (pdm = pdminfo->adevmode;
//              (pdm + 1 <= pdmEnd) && (pdm->dmSize != 0);
//              pdm = (PDEVMODEW)((PCHAR)pdm + pdm->dmSize + pdm->dmDriverExtra))
//         {
//             TRACE("    %S has mode %lux%lux%lu(%lu Hz)\n",
//                   pdm->dmDeviceName,
//                   pdm->dmPelsWidth,
//                   pdm->dmPelsHeight,
//                   pdm->dmBitsPerPel,
//                   pdm->dmDisplayFrequency);
//
//             /* Initialize the entry */
//             pGraphicsDevice->pDevModeList[i].dwFlags = 0;
//             pGraphicsDevice->pDevModeList[i].pdm = pdm;
//             i++;
//         }
//     }
//     TRACE("Prepared %lu modes for %ls\n", pGraphicsDevice->cDevModes, pGraphicsDevice->pwszDescription);
//     return TRUE;
// }
//
// /* Search the closest display mode according to some settings.
//  * Note that we don't care about the DM_* flags in dmFields, but check if value != 0 instead */
// static
// BOOL
// LDEVOBJ_bGetClosestMode(
//     _Inout_ PGRAPHICS_DEVICE pGraphicsDevice,
//     _In_ PDEVMODEW RequestedMode,
//     _Out_ PDEVMODEW *pSelectedMode)
// {
//     DEVMODEW dmDiff;
//     PDEVMODEW pdmCurrent, pdmBest = NULL;
//     ULONG i;
//
//     /* Use a DEVMODE to keep the differences between best mode found and expected mode.
//      * Initialize fields to max value so we can find better modes. */
//     dmDiff.dmPelsWidth = 0xffffffff;
//     dmDiff.dmPelsHeight = 0xffffffff;
//     dmDiff.dmBitsPerPel = 0xffffffff;
//     dmDiff.dmDisplayFrequency = 0xffffffff;
//
//     /* Search the closest mode */
// #define DM_DIFF(field) (RequestedMode->field > pdmCurrent->field ? (RequestedMode->field - pdmCurrent->field) : (pdmCurrent->field - RequestedMode->field))
//     for (i = 0; i < pGraphicsDevice->cDevModes; i++)
//     {
//         pdmCurrent = pGraphicsDevice->pDevModeList[i].pdm;
//
//         /* Skip current mode if it is worse than best mode found */
//         if (RequestedMode->dmPelsWidth != 0 && DM_DIFF(dmPelsWidth) > dmDiff.dmPelsWidth)
//             continue;
//         if (RequestedMode->dmPelsHeight != 0 && DM_DIFF(dmPelsHeight) > dmDiff.dmPelsHeight)
//             continue;
//         if (RequestedMode->dmBitsPerPel != 0 && DM_DIFF(dmBitsPerPel) > dmDiff.dmBitsPerPel)
//             continue;
//         if (RequestedMode->dmDisplayFrequency != 0 && DM_DIFF(dmDisplayFrequency) > dmDiff.dmDisplayFrequency)
//             continue;
//
//         /* Better (or equivalent) mode found. Update differences */
//         dmDiff.dmPelsWidth = DM_DIFF(dmPelsWidth);
//         dmDiff.dmPelsHeight = DM_DIFF(dmPelsHeight);
//         dmDiff.dmBitsPerPel = DM_DIFF(dmBitsPerPel);
//         dmDiff.dmDisplayFrequency = DM_DIFF(dmDisplayFrequency);
//         pdmBest = pdmCurrent;
//     }
// #undef DM_DIFF
//
//     if (pdmBest)
//     {
//         TRACE("Closest display mode to '%dx%dx%d %d Hz' is '%dx%dx%d %d Hz'\n",
//               RequestedMode->dmPelsWidth,
//               RequestedMode->dmPelsHeight,
//               RequestedMode->dmBitsPerPel,
//               RequestedMode->dmDisplayFrequency,
//               pdmBest->dmPelsWidth,
//               pdmBest->dmPelsHeight,
//               pdmBest->dmBitsPerPel,
//               pdmBest->dmDisplayFrequency);
//     }
//
//     *pSelectedMode = pdmBest;
//     return pdmBest != NULL;
// }
//
// BOOL
// LDEVOBJ_bProbeAndCaptureDevmode(
//     _Inout_ PGRAPHICS_DEVICE pGraphicsDevice,
//     _In_ PDEVMODEW RequestedMode,
//     _Out_ PDEVMODEW *pSelectedMode,
//     _In_ BOOL bSearchClosestMode)
// {
//     DEVMODEW dmSearch;
//     PDEVMODEW pdmCurrent, pdm, pdmSelected = NULL;
//     ULONG i;
//     ULONG ulVirtualWidth = 0, ulVirtualHeight = 0;
//     BOOL bResult = TRUE;
//     NTSTATUS Status;
//
//     if (!LDEVOBJ_bBuildDevmodeList(pGraphicsDevice))
//         return FALSE;
//
//     /* At first, load information from registry */
//     RtlZeroMemory(&dmSearch, sizeof(dmSearch));
//     Status = EngpGetDisplayDriverParameters(pGraphicsDevice, &dmSearch);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("EngpGetDisplayDriverParameters() failed with status 0x%08x\n", Status);
//         return FALSE;
//     }
//
//     /* Override values with the new ones provided */
//
//     _SEH2_TRY
//     {
//         bSearchClosestMode |= RequestedMode->dmFields == 0;
//
//         /* Copy standard fields (if provided) */
//         if (RequestedMode->dmFields & DM_BITSPERPEL && RequestedMode->dmBitsPerPel != 0)
//             dmSearch.dmBitsPerPel = RequestedMode->dmBitsPerPel;
//         if (RequestedMode->dmFields & DM_PELSWIDTH && RequestedMode->dmPelsWidth != 0)
//             dmSearch.dmPelsWidth = RequestedMode->dmPelsWidth;
//         if (RequestedMode->dmFields & DM_PELSHEIGHT && RequestedMode->dmPelsHeight != 0)
//             dmSearch.dmPelsHeight = RequestedMode->dmPelsHeight;
//         if (RequestedMode->dmFields & DM_DISPLAYFREQUENCY && RequestedMode->dmDisplayFrequency != 0)
//             dmSearch.dmDisplayFrequency = RequestedMode->dmDisplayFrequency;
//
//         if ((RequestedMode->dmFields & (DM_PANNINGWIDTH | DM_PANNINGHEIGHT)) == (DM_PANNINGWIDTH | DM_PANNINGHEIGHT) &&
//             RequestedMode->dmPanningWidth != 0 && RequestedMode->dmPanningHeight != 0 &&
//             RequestedMode->dmPanningWidth < dmSearch.dmPelsWidth &&
//             RequestedMode->dmPanningHeight < dmSearch.dmPelsHeight)
//         {
//             /* Get new panning values */
//             ulVirtualWidth = RequestedMode->dmPelsWidth;
//             ulVirtualHeight = RequestedMode->dmPelsHeight;
//             dmSearch.dmPelsWidth = RequestedMode->dmPanningWidth;
//             dmSearch.dmPelsHeight = RequestedMode->dmPanningHeight;
//         }
//         else if (dmSearch.dmPanningWidth != 0 && dmSearch.dmPanningHeight != 0 &&
//                  dmSearch.dmPanningWidth < dmSearch.dmPelsWidth &&
//                  dmSearch.dmPanningHeight < dmSearch.dmPelsHeight)
//         {
//             /* Keep existing panning values */
//             ulVirtualWidth = dmSearch.dmPelsWidth;
//             ulVirtualHeight = dmSearch.dmPelsHeight;
//             dmSearch.dmPelsWidth = dmSearch.dmPanningWidth;
//             dmSearch.dmPelsHeight = dmSearch.dmPanningHeight;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         bResult = FALSE;
//     }
//     _SEH2_END;
//
//     if (!bResult)
//         return FALSE;
//
//     if (LDEVOBJ_bGetClosestMode(pGraphicsDevice, &dmSearch, &pdmSelected))
//     {
//         if (bSearchClosestMode)
//         {
//             /* Ok, found a closest mode. Update search */
//             dmSearch.dmBitsPerPel = pdmSelected->dmBitsPerPel;
//             dmSearch.dmPelsWidth = pdmSelected->dmPelsWidth;
//             dmSearch.dmPelsHeight = pdmSelected->dmPelsHeight;
//             dmSearch.dmDisplayFrequency = pdmSelected->dmDisplayFrequency;
//         }
//         else
//         {
//             /* Only update not provided fields */
//             _SEH2_TRY
//             {
//                 if (!(RequestedMode->dmFields & DM_BITSPERPEL) || RequestedMode->dmBitsPerPel == 0)
//                     dmSearch.dmBitsPerPel = pdmSelected->dmBitsPerPel;
//                 if (!(RequestedMode->dmFields & DM_PELSWIDTH) || RequestedMode->dmPelsWidth == 0)
//                     dmSearch.dmPelsWidth = pdmSelected->dmPelsWidth;
//                 if (!(RequestedMode->dmFields & DM_PELSHEIGHT) || RequestedMode->dmPelsHeight == 0)
//                     dmSearch.dmPelsHeight = pdmSelected->dmPelsHeight;
//                 if (!(RequestedMode->dmFields & DM_DISPLAYFREQUENCY) || RequestedMode->dmDisplayFrequency == 0)
//                     dmSearch.dmDisplayFrequency = pdmSelected->dmDisplayFrequency;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 bResult = FALSE;
//             }
//             _SEH2_END;
//
//             if (!bResult)
//                 return FALSE;
//         }
//     }
//
//     /* Now, search the exact mode to return to caller */
//     for (i = 0; i < pGraphicsDevice->cDevModes; i++)
//     {
//         pdmCurrent = pGraphicsDevice->pDevModeList[i].pdm;
//
//         /* For now, we only need those */
//         if (pdmCurrent->dmBitsPerPel != dmSearch.dmBitsPerPel)
//             continue;
//         if (pdmCurrent->dmPelsWidth != dmSearch.dmPelsWidth)
//             continue;
//         if (pdmCurrent->dmPelsHeight != dmSearch.dmPelsHeight)
//             continue;
//         if (pdmCurrent->dmDisplayFrequency != dmSearch.dmDisplayFrequency)
//             continue;
//
//         pdmSelected = pdmCurrent;
//         break;
//     }
//
//     if (!pdmSelected)
//     {
//         ERR("Requested mode not found (%dx%dx%d %d Hz)\n",
//             dmSearch.dmPelsWidth,
//             dmSearch.dmPelsHeight,
//             dmSearch.dmBitsPerPel,
//             dmSearch.dmDisplayFrequency);
//         return FALSE;
//     }
//
//     /* Allocate memory for output */
//     pdm = ExAllocatePoolZero(PagedPool, pdmSelected->dmSize + pdmSelected->dmDriverExtra, GDITAG_DEVMODE);
//     if (!pdm)
//         return FALSE;
//
//     /* Copy selected mode */
//     RtlCopyMemory(pdm, pdmSelected, pdmSelected->dmSize);
//     RtlCopyMemory((PVOID)((ULONG_PTR)pdm + pdm->dmSize),
//                   (PVOID)((ULONG_PTR)pdmSelected + pdmSelected->dmSize),
//                   pdmSelected->dmDriverExtra);
//
//     /* Add back panning */
//     if (ulVirtualWidth != 0 && ulVirtualHeight != 0 &&
//         pdm->dmPelsWidth < ulVirtualWidth &&
//         pdm->dmPelsHeight < ulVirtualHeight)
//     {
//         pdm->dmFields |= DM_PANNINGWIDTH | DM_PANNINGHEIGHT;
//         pdm->dmPanningWidth = pdm->dmPelsWidth;
//         pdm->dmPanningHeight = pdm->dmPelsHeight;
//         pdm->dmPelsWidth = ulVirtualWidth;
//         pdm->dmPelsHeight = ulVirtualHeight;
//     }
//
//     *pSelectedMode = pdm;
//     return TRUE;
// }
//
// /** Exported functions ********************************************************/
//
// HANDLE
// APIENTRY
// EngLoadImage(
//     _In_ LPWSTR pwszDriverName)
// {
//     return (HANDLE)LDEVOBJ_pLoadDriver(pwszDriverName, LDEV_IMAGE);
// }
//
//
// VOID
// APIENTRY
// EngUnloadImage(
//     _In_ HANDLE hModule)
// {
//     PLDEVOBJ pldev = (PLDEVOBJ)hModule;
//
//     /* Make sure the LDEV is in the list */
//     ASSERT((pldev->leLink.Flink != NULL) &&  (pldev->leLink.Blink != NULL));
//
//     LDEVOBJ_vDereference(pldev);
// }
//
//
// PVOID
// APIENTRY
// EngFindImageProcAddress(
//     _In_ HANDLE hModule,
//     _In_ LPSTR  lpProcName)
// {
//     PLDEVOBJ pldev = (PLDEVOBJ)hModule;
//
//     ASSERT(gpldevWin32k != NULL);
//
//     /* Check if win32k is requested */
//     if (!pldev)
//     {
//         pldev = gpldevWin32k;
//     }
//
//     /* Check if the drivers entry point is requested */
//     if (_strnicmp(lpProcName, "DrvEnableDriver", 15) == 0)
//     {
//         return pldev->pGdiDriverInfo->EntryPoint;
//     }
//
//     /* Try to find the address */
//     return LDEVOBJ_pvFindImageProcAddress(pldev, lpProcName);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
