use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite, ExDeleteResourceLite, ExIsResourceAcquiredExclusiveLite, ExIsResourceAcquiredSharedLite, ExReleaseResourceLite, KeEnterCriticalRegion, KeLeaveCriticalRegion, KeBugCheckEx, NtAddAtom, PsGetCurrentThreadWin32Thread, ZwClose};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;

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


// win32k ERR() is a debug-channel macro; route it to a real fn (call sites use
// ERR(...) call syntax — see cliprgn.rs). Not a no-op: these two report USER
// subsystem init failing — a handle table that would not create, and a session
// impl that would not start. A silent one of those is a desktop that never
// appears with nothing on the wire to say why.
extern "C" { fn DbgPrint(fmt: *const i8, ...) -> u32; }
#[inline(always)] unsafe fn ERR(msg: *const i8) { DbgPrint(msg); }

// Bugcheck codes (bugcodes.mc) + the USER subsystem version (ntuser.h:1149).
const USER_VERSION: u32 = 0x0005_0000;            // MAKELONG(0x0000, 0x0005)
const WIN32K_INIT_OR_RIT_FAILURE: u32 = 0x91;
const VIDEO_DRIVER_INIT_FAILURE: u32 = 0xB4;

// win32k C functions not re-exported by the win32k_sys glob — declare with their
// real ABI (header oracle): FASTCALL on i386, cdecl elsewhere for the FASTCALL set;
// NTAPI (extern "system") for the Reg* helpers; plain cdecl for the gdi/init bodies.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RegisterControlAtoms() -> BOOL;
    fn InitSessionImpl() -> NTSTATUS;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RegisterControlAtoms() -> BOOL;
    fn InitSessionImpl() -> NTSTATUS;
}
extern "system" {
    fn RegOpenKey(pwszKeyName: *mut u16, phkResult: *mut HKEY) -> NTSTATUS;
    fn RegReadDWORD(hkey: HKEY, pwszValue: *mut u16, pdwData: *mut DWORD) -> BOOL;
    fn InitVideo() -> NTSTATUS;     // display.c: NTSTATUS NTAPI InitVideo(VOID)
}
extern "C" {
    fn InitSysParams() -> BOOL;
    fn InitCsrProcess();
    fn InitializeGreCSRSS() -> BOOL;
}

// hbrGray lives in the anonymous PERUSERSERVERINFO tail of tagSERVERINFO, which
// bindgen made opaque (__bindgen_padding_0). Reach it via the same offset cast
// used by metric.rs/stockobj.rs.
#[inline(always)]
unsafe fn peruser(psi: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    core::ptr::addr_of_mut!((*psi).__bindgen_padding_0) as *mut _PERUSERSERVERINFO
}

// ---- file-scope preamble (translated) ----
// These globals are extern-declared in ntuser.h / window.h / win32.h and read by
// many still-C files, so each must export its C symbol (#[no_mangle]).
#[no_mangle] pub static mut gptiCurrent: PTHREADINFO = core::ptr::null_mut();
#[no_mangle] pub static mut gppiInputProvider: PPROCESSINFO = core::ptr::null_mut();
#[no_mangle] pub static mut g_AlwaysDisplayVersion: BOOL = 0;
// ERESOURCE: declare with the ntkrnl_sys::ddk type so addr_of_mut!(UserLock)
// feeds the Ex*ResourceLite fns directly (win32k_sys's _ERESOURCE is the same
// layout but a distinct cross-crate type). The exported C symbol is identical.
#[no_mangle] pub static mut UserLock: ntkrnl_sys::ddk::_ERESOURCE = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut AtomMessage: ATOM = 0;
#[no_mangle] pub static mut AtomWndObj: ATOM = 0;
#[no_mangle] pub static mut AtomLayer: ATOM = 0;
#[no_mangle] pub static mut AtomFlashWndState: ATOM = 0;
#[no_mangle] pub static mut AtomDDETrack: ATOM = 0;
#[no_mangle] pub static mut AtomQOS: ATOM = 0;
#[no_mangle] pub static mut hModClient: HINSTANCE = core::ptr::null_mut();
#[no_mangle] pub static mut ClientPfnInit: BOOL = 0;
#[no_mangle] pub static mut gaGuiConsoleWndClass: ATOM = 0;
#[no_mangle] pub static mut AtomImeLevel: ATOM = 0;

unsafe extern "C" fn InitUserAtoms() -> NTSTATUS {
    RegisterControlAtoms();

    (*gpsi).atomSysClass[ICLS_MENU as usize] = 32768;
    (*gpsi).atomSysClass[ICLS_DESKTOP as usize] = 32769;
    (*gpsi).atomSysClass[ICLS_DIALOG as usize] = 32770;
    (*gpsi).atomSysClass[ICLS_SWITCH as usize] = 32771;
    (*gpsi).atomSysClass[ICLS_ICONTITLE as usize] = 32772;
    (*gpsi).atomSysClass[ICLS_TOOLTIPS as usize] = 32774;

    AtomMessage = IntAddGlobalAtom(
        w!("Message").as_ptr() as *mut u16,
        1,
    );
    (*gpsi).atomSysClass[ICLS_HWNDMESSAGE as usize] = AtomMessage;

    (*gpsi).atomContextHelpIdProp = IntAddGlobalAtom(
        w!("SysCH").as_ptr() as *mut u16,
        1,
    );

    (*gpsi).atomIconSmProp = IntAddGlobalAtom(
        w!("SysICS").as_ptr() as *mut u16,
        1,
    );
    (*gpsi).atomIconProp = IntAddGlobalAtom(
        w!("SysIC").as_ptr() as *mut u16,
        1,
    );

    (*gpsi).atomFrostedWindowProp = IntAddGlobalAtom(
        w!("SysFrostedWindow").as_ptr() as *mut u16,
        1,
    );

    AtomDDETrack = IntAddGlobalAtom(
        w!("SysDT").as_ptr() as *mut u16,
        1,
    );
    AtomQOS = IntAddGlobalAtom(
        w!("SysQOS").as_ptr() as *mut u16,
        1,
    );
    AtomImeLevel = IntAddGlobalAtom(
        w!("SysIMEL").as_ptr() as *mut u16,
        1,
    );

    NtAddAtom(
        w!("SysWNDO").as_ptr() as *mut u16,
        14,
        core::ptr::addr_of_mut!(AtomWndObj),
    );

    AtomLayer = IntAddGlobalAtom(
        w!("SysLayer").as_ptr() as *mut u16,
        1,
    );
    AtomFlashWndState = IntAddGlobalAtom(
        w!("FlashWState").as_ptr() as *mut u16,
        1,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn InitUserImpl() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hKey: HKEY = core::ptr::null_mut();

    if UserCreateHandleTable() == 0 {
        ERR(b"Failed creating handle table\n\0".as_ptr() as *mut i8);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = InitSessionImpl();
    if !NT_SUCCESS(Status) {
        ERR(b"Error init session impl.\n\0".as_ptr() as *mut i8);
        return Status;
    }

    InitUserAtoms();

    Status = RegOpenKey(
        w!("\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Windows").as_ptr() as *mut u16,
        core::ptr::addr_of_mut!(hKey),
    );
    if NT_SUCCESS(Status) {
        let mut dwValue: DWORD = 0;
        RegReadDWORD(
            hKey,
            w!("DisplayVersion").as_ptr() as *mut u16,
            core::ptr::addr_of_mut!(dwValue),
        );
        g_AlwaysDisplayVersion = (dwValue != 0) as BOOL;
        ZwClose(hKey as _);
    }

    InitSysParams();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn UserInitialize() -> NTSTATUS {
    static wPattern55AA: [DWORD; 8] = [
        0x55555555, 0xaaaaaaaa, 0x55555555, 0xaaaaaaaa,
        0x55555555, 0xaaaaaaaa, 0x55555555, 0xaaaaaaaa,
    ];
    let mut hPattern55AABitmap: HBITMAP = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = UserCreateWinstaDirectory();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = InitVideo();
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(VIDEO_DRIVER_INIT_FAILURE, Status as ULONG_PTR, 0, 0, USER_VERSION as ULONG_PTR);
    }

    {
        let mut pmdev: PMDEVOBJ = core::ptr::null_mut();
        let lRet = PDEVOBJ_lChangeDisplaySettings(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut pmdev,
            1,
        );
        if lRet == DISP_CHANGE_SUCCESSFUL as LONG {
            gpmdev = pmdev;
        }
    }

    NtUserUpdatePerUserSystemParameters(0, 1);

    if (*peruser(gpsi)).hbrGray.is_null() {
        hPattern55AABitmap = GreCreateBitmap(8, 8, 1, 1, wPattern55AA.as_ptr() as _);
        if hPattern55AABitmap.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*peruser(gpsi)).hbrGray = IntGdiCreatePatternBrush(hPattern55AABitmap);

        if (*peruser(gpsi)).hbrGray.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn NtUserInitialize(
    dwWinVersion: DWORD,
    hPowerRequestEvent: HANDLE,
    hMediaRequestEvent: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if !gpepCSRSS.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    if dwWinVersion != USER_VERSION {
        KeBugCheckEx(
            WIN32K_INIT_OR_RIT_FAILURE,
            0,
            0,
            dwWinVersion as ULONG_PTR,
            USER_VERSION as ULONG_PTR,
        );
    }

    UserEnterExclusive();

    InitCsrProcess();

    Status = IntInitWin32PowerManagement(hPowerRequestEvent);
    if !NT_SUCCESS(Status) {
        UserLeave();
        return Status;
    }

    if InitializeGreCSRSS() == 0 {
        UserLeave();
        return STATUS_UNSUCCESSFUL;
    }

    Status = UserInitialize();

    UserLeave();
    Status
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserIsEntered() -> BOOL {
    (ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!(UserLock)) != 0
        || ExIsResourceAcquiredSharedLite(core::ptr::addr_of_mut!(UserLock)) != 0) as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserIsEntered() -> BOOL {
    (ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!(UserLock)) != 0
        || ExIsResourceAcquiredSharedLite(core::ptr::addr_of_mut!(UserLock)) != 0) as BOOL
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserIsEnteredExclusive() -> BOOL {
    ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!(UserLock)) as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserIsEnteredExclusive() -> BOOL {
    ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!(UserLock)) as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CleanupUserImpl() {
    ExDeleteResourceLite(core::ptr::addr_of_mut!(UserLock));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CleanupUserImpl() {
    ExDeleteResourceLite(core::ptr::addr_of_mut!(UserLock));
}


// Who holds the user lock for long. Every GUI thread queues on this one
// ERESOURCE, so one long hold stalls all of them: taskmgr's close waited
// minutes (receiver wait reason WrResource) once x86 DOOM was running.
// First exclusive acquire notes the time, last release reports a hold over
// USER_LOCK_SLOW_MS with the holder. Both fields change only while held
// exclusively.
const USER_LOCK_SLOW_MS: u32 = 2000;
static mut USER_LOCK_DEPTH: u32 = 0;
static mut USER_LOCK_SINCE: u32 = 0;

unsafe fn user_lock_tick() -> u32 {
    // The interrupt-time tick in SharedUserData, as EngGetTickCount32 reads it.
    extern "system" { fn EngGetTickCount() -> u64; }
    EngGetTickCount() as u32
}

#[inline(never)]
unsafe fn user_lock_entered() {
    if USER_LOCK_DEPTH == 0 {
        USER_LOCK_SINCE = user_lock_tick();
    }
    USER_LOCK_DEPTH += 1;
}

#[inline(never)]
unsafe fn user_lock_leaving() {
    if ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!(UserLock)) == 0 || USER_LOCK_DEPTH == 0 {
        return;
    }
    USER_LOCK_DEPTH -= 1;
    if USER_LOCK_DEPTH != 0 {
        return;
    }
    let Held = user_lock_tick().wrapping_sub(USER_LOCK_SINCE);
    if Held >= USER_LOCK_SLOW_MS {
        let Process = ntkrnl_sys::ddk::IoGetCurrentProcess() as *const u8;
        let Name = Process.add(core::mem::offset_of!(ntkrnl_sys::ddk::_EPROCESS, ImageFileName));
        DbgPrint(
            b"[WIN32K] user lock held %u ms by tid %u (%.15s)\n\0".as_ptr() as *const i8,
            Held,
            ntkrnl_sys::ddk::PsGetCurrentThreadId() as usize as u32,
            Name,
        );
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserEnterShared() {
    KeEnterCriticalRegion();
    ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(UserLock), 1u8);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserEnterShared() {
    KeEnterCriticalRegion();
    ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(UserLock), 1u8);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserEnterExclusive() {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(UserLock), 1u8);
    user_lock_entered();
    gptiCurrent = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserEnterExclusive() {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(UserLock), 1u8);
    user_lock_entered();
    gptiCurrent = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserLeave() {
    user_lock_leaving();
    ExReleaseResourceLite(core::ptr::addr_of_mut!(UserLock));
    KeLeaveCriticalRegion();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserLeave() {
    user_lock_leaving();
    ExReleaseResourceLite(core::ptr::addr_of_mut!(UserLock));
    KeLeaveCriticalRegion();
}
