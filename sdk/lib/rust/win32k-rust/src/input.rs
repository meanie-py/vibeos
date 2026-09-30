#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeInitializeTimer, KeSetPriorityThread, KeWaitForMultipleObjects, ObCloseHandle, ObOpenObjectByPointer, ObReferenceObjectByHandle, PsGetCurrentThread, PsGetCurrentThreadWin32Thread, RtlInitUnicodeString, RtlNtStatusToDosError, ZwCancelIoFile, ZwOpenFile, ZwReadFile};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::{STATUS_UNSUCCESSFUL, STATUS_INVALID_PARAMETER, STATUS_ACCESS_DENIED, STATUS_PENDING};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExDeleteResourceLite, ExInitializeResourceLite, ExReleaseResourceLite, KeEnterCriticalRegion, KeLeaveCriticalRegion};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};
use core::ffi::c_void;

// KeWaitForMultipleObjects returns STATUS_WAIT_0 (== 0) as the base of the
// signaled-object index range (ntdef.h: STATUS_WAIT_0 = 0x00000000).
const STATUS_WAIT_0: NTSTATUS = 0;

// USER pool tags (tags.h). C 4-char constants are big-endian; from_le_bytes
// reverses to byte order.
const USERTAG_SYSTEM: u32 = u32::from_le_bytes(*b"Ussy");      // 'yssU'
const USERTAG_ATTACHINFO: u32 = u32::from_le_bytes(*b"Usai");  // 'iasU'
const USERTAG_Q: u32 = u32::from_le_bytes(*b"Usqu");           // 'uqsU'

// PWND_BOTTOM = ((PWND)1) — macro, no symbol (ntuser.h). Matches hotkey.rs.
const PWND_BOTTOM: PWND = 1usize as PWND;

// UlongToHandle(ul) = (HANDLE)(ULONG_PTR)(ul) — basetsd.h macro, no symbol.
#[inline(always)]
unsafe fn UlongToHandle(ul: DWORD) -> HANDLE {
    ul as usize as HANDLE
}

// ExWindowStationObjectType is an ntoskrnl DATA export (ntoskrnl.spec `@ extern`).
// Per-arch cross-DLL raw-dylib `__imp_` pattern (same shape as winsta.rs/desktop.rs).
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut ExWindowStationObjectType: *mut _OBJECT_TYPE;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_ExWindowStationObjectType"]
    static mut ExWindowStationObjectType_imp: *mut *mut _OBJECT_TYPE;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_windowstation_object_type() -> *mut _OBJECT_TYPE { ExWindowStationObjectType }
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_windowstation_object_type() -> *mut _OBJECT_TYPE { *ExWindowStationObjectType_imp }

// ---- Cross-file win32k functions (real symbols; ABI per win32ss headers) ----
//
// FASTCALL on i386 (`@Name@N`), cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn StartTheTimers();
    fn ProcessTimers();
    fn SetDebugHotKeys();
    fn UserRegisterHotKey(pWnd: PWND, id: INT, fsModifiers: UINT, vk: UINT) -> BOOL;
    fn UserGetMouseButtonsState() -> WORD;
    fn UserSetProcessWindowStation(hWindowStation: HWINSTA) -> BOOL;
    fn UserPostMessage(Wnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn MsqCreateMessageQueue(pti: PTHREADINFO) -> PUSER_MESSAGE_QUEUE;
    fn MsqDestroyMessageQueue(pti: PTHREADINFO);
    fn co_MsqInsertMouseMessage(Msg: *mut MSG, flags: DWORD, dwExtraInfo: ULONG_PTR, Hook: BOOL,);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn UserObjectInDestroy(h: HANDLE) -> BOOL;
    fn IntSetFocusMessageQueue(NewQueue: PUSER_MESSAGE_QUEUE);
    fn IntGetFocusMessageQueue() -> PUSER_MESSAGE_QUEUE;
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn StartTheTimers();
    fn ProcessTimers();
    fn SetDebugHotKeys();
    fn UserRegisterHotKey(pWnd: PWND, id: INT, fsModifiers: UINT, vk: UINT) -> BOOL;
    fn UserGetMouseButtonsState() -> WORD;
    fn UserSetProcessWindowStation(hWindowStation: HWINSTA) -> BOOL;
    fn UserPostMessage(Wnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn MsqCreateMessageQueue(pti: PTHREADINFO) -> PUSER_MESSAGE_QUEUE;
    fn MsqDestroyMessageQueue(pti: PTHREADINFO);
    fn co_MsqInsertMouseMessage(Msg: *mut MSG, flags: DWORD, dwExtraInfo: ULONG_PTR, Hook: BOOL,);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn UserObjectInDestroy(h: HANDLE) -> BOOL;
    fn IntSetFocusMessageQueue(NewQueue: PUSER_MESSAGE_QUEUE);
    fn IntGetFocusMessageQueue() -> PUSER_MESSAGE_QUEUE;
    fn SetLastNtError(Status: NTSTATUS);
}

// NTAPI (stdcall on i386, cdecl elsewhere) win32k functions → extern "system".
extern "system" {
    fn UserInitKeyboard(hKeyboardDevice: HANDLE);
    fn UserProcessKeyboardInput(pKeyInput: *mut KEYBOARD_INPUT_DATA);
    fn UserProcessMouseInput(pMouseInputData: *mut MOUSE_INPUT_DATA);
    fn UserSendKeyboardInput(pKbdInput: *mut KEYBDINPUT, bInjected: BOOL) -> BOOL;
    fn UserSendMouseInput(pMouseInput: *mut MOUSEINPUT, bInjected: BOOL) -> BOOL;
    // EngSetLastError — APIENTRY (stdcall) per eng.h.
    fn EngSetLastError(iError: ULONG);
    // PoRequestShutdownEvent — NTAPI (ntpoapi.h).
    fn PoRequestShutdownEvent(Event: *mut PVOID) -> NTSTATUS;
}

// EngGetTickCount() — APIENTRY (stdcall) per eng.h; EngGetTickCount32 truncates.
extern "system" {
    fn EngGetTickCount() -> ULONGLONG;
}
#[inline(always)]
unsafe fn EngGetTickCount32() -> ULONG {
    EngGetTickCount() as ULONG
}

// _MmCopyFromCaller — the C `MmCopyFromCaller(x,y,z)` macro forwards to this
// (mmcopy.h); cdecl, provided by copy.rs (matches misc.rs).
// GetW32ThreadInfo — misc.rs sibling, cdecl-exported (PTHREADINFO GetW32ThreadInfo(VOID)).
extern "C" {
    fn _MmCopyFromCaller(Target: *mut c_void, Source: *const c_void, Bytes: UINT) -> NTSTATUS;
    fn GetW32ThreadInfo() -> PTHREADINFO;
}
#[inline(always)]
unsafe fn MmCopyFromCaller(dst: *mut c_void, src: *const c_void, len: usize) -> NTSTATUS {
    _MmCopyFromCaller(dst, src, len as UINT)
}

// MAKELPARAM(low, high) — windef.h macro. (matches misc.rs/focus.rs)
#[inline(always)]
fn MAKELPARAM(low: i32, high: i32) -> LPARAM {
    (((low as u16 as u32) | ((high as u16 as u32) << 16)) as i32) as LPARAM
}

// ThreadHasInputAccess(W32Thread) — (TRUE) macro (input.h). Always succeeds.
#[inline(always)]
unsafe fn ThreadHasInputAccess(_pti: PTHREADINFO) -> BOOL {
    1
}

// IntIsActiveDesktop(Desktop) = Desktop->rpwinstaParent->ActiveDesktop == Desktop (desktop.h).
#[inline(always)]
unsafe fn IntIsActiveDesktop(Desktop: PDESKTOP) -> BOOL {
    ((*(*Desktop).rpwinstaParent).ActiveDesktop == Desktop) as BOOL
}

// IntReferenceMessageQueue / IntDereferenceMessageQueue (msgqueue.h macros).
#[inline(always)]
unsafe fn IntReferenceMessageQueue(MsgQueue: PUSER_MESSAGE_QUEUE) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*MsgQueue).References) as *mut i32);
}
#[inline(always)]
unsafe fn IntDereferenceMessageQueue(MsgQueue: PUSER_MESSAGE_QUEUE) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*MsgQueue).References) as *mut i32) == 0 {
        ExFreePoolWithTag(MsgQueue as *mut c_void, USERTAG_Q);
    }
}

// UserHMGetHandle(obj) = obj->head.h (object.h macro).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    (*(obj as *const _HEAD)).h
}

// gpsi per-user fields (ptCursor, dwLastRITEventTickCount): tagSERVERINFO ends with
// __bindgen_padding_0 representing the anonymous PERUSERSERVERINFO member; cast that
// tail (matches timer.rs/mouse.rs gpsi_per_user).
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// THRDESKHEAD.pti: bindgen flattens HEAD{h, cLockObj} + pdesk + rpdesk... — pti
// follows HEAD{HANDLE, DWORD}, so it sits at 2*usize (8 i386 / 16 amd64-arm64),
// NOT HANDLE+DWORD (=12 on 64-bit → garbage). Matches timer.rs/event.rs head_pti.
#[inline(always)]
unsafe fn head_pti(head: *const _THRDESKHEAD) -> PTHREADINFO {
    let off = 2 * core::mem::size_of::<usize>();
    *((head as *const u8).add(off) as *const PTHREADINFO)
}

// PROCESSINFO->W32PF_flags lives in the flattened W32PROCESS prefix
// (peProcess ptr + RefCount DWORD), so W32PF_flags (DWORD) sits at
// size_of::<usize>()+size_of::<u32>() = 8 (i386) / 12 (amd64/arm64). Matches misc.rs.
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> *mut DWORD {
    let off = core::mem::size_of::<usize>() + core::mem::size_of::<u32>();
    (ppi as *mut u8).add(off) as *mut DWORD
}

// wide_str!("ascii"): const-encode an ASCII literal to a NUL-terminated static
// [u16; N], yielding *const u16 (PCWSTR). Replaces the alloc::vec the translator
// emitted (this crate is no_std, no alloc). Matches freetype.rs/ldevobj.rs.
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

// ---- file-scope preamble (translated) ----
// ptiRawInput / MasterTimer / gpai / ghKeyboardDevice are non-static in C and
// declared `extern` in win32ss headers — other .c TUs (keyboard.c, msgqueue.c,
// hotkey.c) reference them, so export the undecorated C symbol via #[no_mangle].
#[no_mangle]
pub static mut ptiRawInput: PTHREADINFO = core::ptr::null_mut();
#[no_mangle]
pub static mut MasterTimer: PKTIMER = core::ptr::null_mut();
#[no_mangle]
pub static mut gpai: PATTACHINFO = core::ptr::null_mut();
#[no_mangle]
pub static mut paiCount: INT = 0;
#[no_mangle]
pub static mut ghKeyboardDevice: HANDLE = core::ptr::null_mut();
// The raw input device list (upstream #9498). Its nodes are the RIT's own stack
// locals, so the list is only valid while RawInputThreadMain runs; rawinput.rs
// walks it under gpDeviceInfoListMutex.
#[no_mangle]
pub static mut gpInputDeviceInfo: PINPUT_DEVICE_INFO = core::ptr::null_mut();
#[no_mangle]
pub static mut gpDeviceInfoListMutex: PERESOURCE = core::ptr::null_mut();

// input.h FORCEINLINE AcquireDeviceInfoListMutex / ReleaseDeviceInfoListMutex.
pub(crate) unsafe fn AcquireDeviceInfoListMutex() {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(gpDeviceInfoListMutex as _, 1);
}

pub(crate) unsafe fn ReleaseDeviceInfoListMutex() {
    ExReleaseResourceLite(gpDeviceInfoListMutex as _);
    KeLeaveCriticalRegion();
}
// LastInputTick + ghMouseDevice are `static` (file-local) in C — keep private.
static mut LastInputTick: DWORD = 0;
static mut ghMouseDevice: HANDLE = core::ptr::null_mut();

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntLastInputTick(bUpdate: BOOL) -> DWORD {
    if bUpdate != 0 {
        LastInputTick = EngGetTickCount32();
        if !gpsi.is_null() {
            (*gpsi_per_user()).dwLastRITEventTickCount = LastInputTick;
        }
    }
    LastInputTick
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntLastInputTick(bUpdate: BOOL) -> DWORD {
    if bUpdate != 0 {
        LastInputTick = EngGetTickCount32();
        if !gpsi.is_null() {
            (*gpsi_per_user()).dwLastRITEventTickCount = LastInputTick;
        }
    }
    LastInputTick
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DoTheScreenSaver() {
    DoTheScreenSaver_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DoTheScreenSaver() {
    DoTheScreenSaver_impl()
}

unsafe fn DoTheScreenSaver_impl() {
    let mut Test: DWORD;
    let TO: DWORD;

    if gspv.iScrSaverTimeout > 0 {
        Test = EngGetTickCount32();
        Test = Test - LastInputTick;
        TO = 1000 * gspv.iScrSaverTimeout as DWORD;
        if Test > TO {
            TRACE!("Screensaver Message Start! Tick {} Timeout {}\n", Test, gspv.iScrSaverTimeout);

            if !ppiScrnSaver.is_null() {
                if (*ppi_w32pf_flags(ppiScrnSaver)) & W32PF_IDLESCREENSAVER == 0 {
                    (*ppi_w32pf_flags(ppiScrnSaver)) |= W32PF_IDLESCREENSAVER;
                    ERR!("Screensaver is Idle\n");
                }
            } else {
                let ForegroundQueue: PUSER_MESSAGE_QUEUE = IntGetFocusMessageQueue();
                if !ForegroundQueue.is_null() && !(*ForegroundQueue).spwndActive.is_null() {
                    UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_START_SCREENSAVE as WPARAM, 1);
                } else {
                    UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_START_SCREENSAVE as WPARAM, 0);
                }
            }
        }
    }
}

unsafe extern "system" fn OpenInputDevice(
    pHandle: PHANDLE,
    ppObject: *mut PFILE_OBJECT,
    pszDeviceName: *const u16,
) -> NTSTATUS {
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    RtlInitUnicodeString(&mut DeviceName as *mut _ as *mut _, pszDeviceName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE as u32;
    ObjectAttributes.ObjectName = &mut DeviceName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenFile(
        pHandle,
        FILE_ALL_ACCESS,
        &mut ObjectAttributes as *mut _ as *mut _,
        &mut Iosb as *mut _ as *mut _,
        0,
        0,
    );

    if NT_SUCCESS(Status) && !ppObject.is_null() {
        Status = ObReferenceObjectByHandle(
            *pHandle,
            SYNCHRONIZE,
            core::ptr::null_mut(),
            0i32 as i8,
            ppObject as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RawInputThreadMain() {
    let mut MouStatus: NTSTATUS = STATUS_UNSUCCESSFUL;
    let mut KbdStatus: NTSTATUS = STATUS_UNSUCCESSFUL;
    let mut Status: NTSTATUS;
    let mut MouIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut KbdIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut pKbdDevice: PFILE_OBJECT = core::ptr::null_mut();
    let mut pMouDevice: PFILE_OBJECT = core::ptr::null_mut();
    let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut WaitObjects: [PVOID; 4] = [core::ptr::null_mut(); 4];
    let mut pSignaledObject: PVOID;
    let mut WaitBlockArray: [KWAIT_BLOCK; 4] = core::mem::zeroed();
    let mut cWaitObjects: ULONG;
    let mut cMaxWaitObjects: ULONG = 2;
    let mut MouseInput: MOUSE_INPUT_DATA = core::mem::zeroed();
    let mut KeyInput: KEYBOARD_INPUT_DATA = core::mem::zeroed();
    let mut ShutdownEvent: PVOID = core::ptr::null_mut();
    let mut hWinSta: HWINSTA = core::ptr::null_mut();
    let mut Mouse: INPUT_DEVICE_INFO = core::mem::zeroed();
    let mut Keyboard: INPUT_DEVICE_INFO = core::mem::zeroed();

    Mouse.DeviceType = RIM_TYPEMOUSE as ULONG;
    RtlInitUnicodeString(&mut Mouse.DeviceName as *mut _ as *mut _, wide_str!("\\Device\\PointerClass0"));
    Mouse.__bindgen_anon_1.Mouse.Attributes.NumberOfButtons = 2;

    Keyboard.DeviceType = RIM_TYPEKEYBOARD as ULONG;
    RtlInitUnicodeString(&mut Keyboard.DeviceName as *mut _ as *mut _, wide_str!("\\Device\\KeyboardClass0"));

    ByteOffset.QuadPart = 0;

    ptiRawInput = GetW32ThreadInfo();
    (*ptiRawInput).TIF_flags |= TIF_SYSTEMTHREAD as u32;
    (*(*ptiRawInput).pClientInfo).dwTIFlags = (*ptiRawInput).TIF_flags;

    KeSetPriorityThread(
        core::ptr::addr_of_mut!((*PsGetCurrentThread()).Tcb) as *mut _,
        (LOW_REALTIME_PRIORITY + 3) as i32,
    );

    Status = ObOpenObjectByPointer(
        InputWindowStation as PVOID,
        0,
        core::ptr::null_mut(),
        MAXIMUM_ALLOWED,
        ex_windowstation_object_type() as _,
        1i32 as i8,
        &mut hWinSta as *mut HWINSTA as *mut _,
    );
    if NT_SUCCESS(Status) {
        UserSetProcessWindowStation(hWinSta);
    } else {
        // ASSERT(FALSE);
    }

    gpDeviceInfoListMutex = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<ERESOURCE>() as _,
        USERTAG_SYSTEM,
    ) as PERESOURCE;
    ntkrnl_sys::ASSERT!(!gpDeviceInfoListMutex.is_null());
    Status = ExInitializeResourceLite(gpDeviceInfoListMutex as _);
    ntkrnl_sys::ASSERT!(NT_SUCCESS(Status));

    UserEnterExclusive();
    StartTheTimers();
    UserLeave();

    PoRequestShutdownEvent(&mut ShutdownEvent);

    extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> u32;
    }
    // One line per device when it opens (or first fails to): which transport
    // feeds the desktop is otherwise invisible, and a class device nobody opens
    // drops every key without a trace.
    let mut mou_open_logged = false;
    let mut kbd_open_logged = false;
    let mut first_kbd_logged = false;
    let mut first_mou_logged = false;
    loop {
        if ghMouseDevice.is_null() {
            Status = OpenInputDevice(
                core::ptr::addr_of_mut!(ghMouseDevice),
                &mut pMouDevice,
                Mouse.DeviceName.Buffer,
            );
            if NT_SUCCESS(Status) {
                cMaxWaitObjects += 1;
                Mouse.pNextDeviceInfo = gpInputDeviceInfo;
                gpInputDeviceInfo = &mut Mouse;
                DbgPrint(b"[WIN32K] RIT opened \\Device\\PointerClass0 (driver %wZ)\n\0".as_ptr(),
                    &(*(*(*pMouDevice).DeviceObject).DriverObject).DriverName as *const _);
            } else if !mou_open_logged {
                DbgPrint(b"[WIN32K] RIT cannot open \\Device\\PointerClass0 yet: 0x%x\n\0".as_ptr(), Status as u32);
            }
            mou_open_logged = true;
        }
        if ghKeyboardDevice.is_null() {
            Status = OpenInputDevice(
                core::ptr::addr_of_mut!(ghKeyboardDevice),
                &mut pKbdDevice,
                Keyboard.DeviceName.Buffer,
            );
            if !NT_SUCCESS(Status) && !kbd_open_logged {
                DbgPrint(b"[WIN32K] RIT cannot open \\Device\\KeyboardClass0 yet: 0x%x\n\0".as_ptr(), Status as u32);
            }
            kbd_open_logged = true;
            if NT_SUCCESS(Status) {
                DbgPrint(b"[WIN32K] RIT opened \\Device\\KeyboardClass0 (driver %wZ)\n\0".as_ptr(),
                    &(*(*(*pKbdDevice).DeviceObject).DriverObject).DriverName as *const _);
                cMaxWaitObjects += 1;
                Keyboard.pNextDeviceInfo = gpInputDeviceInfo;
                gpInputDeviceInfo = &mut Keyboard;
                UserInitKeyboard(ghKeyboardDevice);
                Keyboard.__bindgen_anon_1.Keyboard.Attributes = gKeyboardInfo;
                UserEnterExclusive();
                UserRegisterHotKey(PWND_BOTTOM, IDHK_WINKEY as i32, MOD_WIN as u32, 0);
                UserRegisterHotKey(PWND_BOTTOM, IDHK_SNAP_LEFT as i32, MOD_WIN as u32, VK_LEFT as u32);
                UserRegisterHotKey(PWND_BOTTOM, IDHK_SNAP_RIGHT as i32, MOD_WIN as u32, VK_RIGHT as u32);
                UserRegisterHotKey(PWND_BOTTOM, IDHK_SNAP_UP as i32, MOD_WIN as u32, VK_UP as u32);
                UserRegisterHotKey(PWND_BOTTOM, IDHK_SNAP_DOWN as i32, MOD_WIN as u32, VK_DOWN as u32);
                SetDebugHotKeys();
                UserLeave();
            }
        }

        cWaitObjects = 0;
        WaitObjects[cWaitObjects as usize] = ShutdownEvent;
        cWaitObjects += 1;
        WaitObjects[cWaitObjects as usize] = MasterTimer as PVOID;
        cWaitObjects += 1;

        if !ghMouseDevice.is_null() {
            if MouStatus != STATUS_PENDING {
                MouStatus = ZwReadFile(
                    ghMouseDevice,
                    core::ptr::null_mut(),
                    None,
                    core::ptr::null_mut(),
                    &mut MouIosb as *mut _ as *mut _,
                    &mut MouseInput as *mut MOUSE_INPUT_DATA as PVOID,
                    core::mem::size_of::<MOUSE_INPUT_DATA>() as u32,
                    &mut ByteOffset as *mut _ as *mut _,
                    core::ptr::null_mut(),
                );
            }

            if MouStatus == STATUS_PENDING {
                WaitObjects[cWaitObjects as usize] =
                    core::ptr::addr_of_mut!((*pMouDevice).Event) as PVOID;
                cWaitObjects += 1;
            }
        }

        if !ghKeyboardDevice.is_null() {
            if KbdStatus != STATUS_PENDING {
                KbdStatus = ZwReadFile(
                    ghKeyboardDevice,
                    core::ptr::null_mut(),
                    None,
                    core::ptr::null_mut(),
                    &mut KbdIosb as *mut _ as *mut _,
                    &mut KeyInput as *mut KEYBOARD_INPUT_DATA as PVOID,
                    core::mem::size_of::<KEYBOARD_INPUT_DATA>() as u32,
                    &mut ByteOffset as *mut _ as *mut _,
                    core::ptr::null_mut(),
                );
            }
            if KbdStatus == STATUS_PENDING {
                WaitObjects[cWaitObjects as usize] =
                    core::ptr::addr_of_mut!((*pKbdDevice).Event) as PVOID;
                cWaitObjects += 1;
            }
        }

        if cWaitObjects == cMaxWaitObjects {
            Status = KeWaitForMultipleObjects(
                cWaitObjects,
                WaitObjects.as_mut_ptr(),
                WaitAny as _,
                _KWAIT_REASON_UserRequest as _,
                0i32 as i8,
                1u8,
                core::ptr::null_mut(),
                WaitBlockArray.as_mut_ptr() as *mut _,
            );

            if Status >= STATUS_WAIT_0 && Status < STATUS_WAIT_0 + cWaitObjects as i32 {
                pSignaledObject = WaitObjects[(Status - STATUS_WAIT_0) as usize];

                if MouStatus == STATUS_PENDING
                    && pSignaledObject == core::ptr::addr_of_mut!((*pMouDevice).Event) as PVOID
                {
                    MouStatus = MouIosb.__bindgen_anon_1.Status;
                } else if KbdStatus == STATUS_PENDING
                    && pSignaledObject == core::ptr::addr_of_mut!((*pKbdDevice).Event) as PVOID
                {
                    KbdStatus = KbdIosb.__bindgen_anon_1.Status;
                } else if pSignaledObject == MasterTimer as PVOID {
                    ProcessTimers();
                } else if pSignaledObject == ShutdownEvent {
                    break;
                } else {
                    // ASSERT(FALSE);
                }
            }
        }

        if NT_SUCCESS(MouStatus) && MouStatus != STATUS_PENDING {
            if !first_mou_logged {
                first_mou_logged = true;
                DbgPrint(b"[WIN32K] RIT first mouse input flags=0x%x x=%d y=%d\n\0".as_ptr(),
                    MouseInput.Flags as u32, MouseInput.LastX, MouseInput.LastY);
            }
            IntLastInputTick(1);

            UserEnterExclusive();
            UserProcessMouseInput(&mut MouseInput);
            UserLeave();
        } else if MouStatus != STATUS_PENDING {
            // ERR
        }

        if NT_SUCCESS(KbdStatus) && KbdStatus != STATUS_PENDING {
            if !first_kbd_logged {
                first_kbd_logged = true;
                DbgPrint(b"[WIN32K] RIT first keyboard input sc=0x%x flags=0x%x\n\0".as_ptr(),
                    KeyInput.MakeCode as u32, KeyInput.Flags as u32);
            }
            IntLastInputTick(1);

            UserEnterExclusive();
            UserProcessKeyboardInput(&mut KeyInput);
            UserLeave();
        } else if KbdStatus != STATUS_PENDING {
            // Read failed — without a print this drops keys silently forever
            // (the read is only re-issued next loop; a persistent error spins).
            static mut RIT_KBDERR_TRACE: u32 = 0;
            if RIT_KBDERR_TRACE < 8 {
                RIT_KBDERR_TRACE += 1;
                DbgPrint(
                    b"[WIN32K] RIT kbd read FAILED status=0x%x\n\0".as_ptr(),
                    KbdStatus as u32,
                );
            }
        }
    }

    if !ghMouseDevice.is_null() {
        let _ = ZwCancelIoFile(ghMouseDevice, &mut MouIosb as *mut _ as *mut _);
        ObCloseHandle(ghMouseDevice, 0i32 as i8);
        ObDereferenceObject(pMouDevice as PVOID);
        ghMouseDevice = core::ptr::null_mut();
    }

    if !ghKeyboardDevice.is_null() {
        let _ = ZwCancelIoFile(ghKeyboardDevice, &mut KbdIosb as *mut _ as *mut _);
        ObCloseHandle(ghKeyboardDevice, 0i32 as i8);
        ObDereferenceObject(pKbdDevice as PVOID);
        ghKeyboardDevice = core::ptr::null_mut();
    }

    // Divergence from input.c: the list's nodes are this frame's locals, and the C
    // leaves gpInputDeviceInfo pointing at them after the thread returns, so a later
    // GetRawInputDeviceList would walk a dead stack. Unlink under the lock first.
    AcquireDeviceInfoListMutex();
    gpInputDeviceInfo = core::ptr::null_mut();
    ReleaseDeviceInfoListMutex();

    ExDeleteResourceLite(gpDeviceInfoListMutex as _);
    ExFreePoolWithTag(gpDeviceInfoListMutex as _, USERTAG_SYSTEM);
}

#[no_mangle]
pub unsafe extern "system" fn InitInputImpl() -> NTSTATUS {
    MasterTimer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<KTIMER>() as _,
        USERTAG_SYSTEM,
    ) as PKTIMER;
    if MasterTimer.is_null() {
        return STATUS_UNSUCCESSFUL;
    }
    KeInitializeTimer(MasterTimer as *mut _);

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntBlockInput(pti: PTHREADINFO, BlockIt: BOOL) -> BOOL {
    IntBlockInputImpl(pti, BlockIt)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntBlockInput(pti: PTHREADINFO, BlockIt: BOOL) -> BOOL {
    IntBlockInputImpl(pti, BlockIt)
}

unsafe fn IntBlockInputImpl(pti: PTHREADINFO, BlockIt: BOOL) -> BOOL {
    let OldBlock: PTHREADINFO;

    if (*pti).rpdesk.is_null()
        || (((*pti).TIF_flags & TIF_INCLEANUP) != 0 && BlockIt != 0)
    {
        return 0;
    }

    if ThreadHasInputAccess(pti) == 0 || IntIsActiveDesktop((*pti).rpdesk) == 0 {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }

    OldBlock = (*(*pti).rpdesk).BlockInputThread as PTHREADINFO;
    if !OldBlock.is_null() {
        if OldBlock != pti {
            EngSetLastError(ERROR_ACCESS_DENIED);
            return 0;
        }
        (*(*pti).rpdesk).BlockInputThread = if BlockIt != 0 { pti as *mut c_void } else { core::ptr::null_mut() };
        return (OldBlock.is_null()) as BOOL;
    }

    (*(*pti).rpdesk).BlockInputThread = if BlockIt != 0 { pti as *mut c_void } else { core::ptr::null_mut() };
    (OldBlock.is_null()) as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserBlockInput(BlockIt: BOOL) -> BOOL {
    let ret: BOOL;

    UserEnterExclusive();

    ret = IntBlockInput(PsGetCurrentThreadWin32Thread() as PTHREADINFO, BlockIt);

    UserLeave();

    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IsRemoveAttachThread(pti: PTHREADINFO) -> BOOL {
    IsRemoveAttachThread_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IsRemoveAttachThread(pti: PTHREADINFO) -> BOOL {
    IsRemoveAttachThread_impl(pti)
}

unsafe fn IsRemoveAttachThread_impl(pti: PTHREADINFO) -> BOOL {
    let mut Status: NTSTATUS;
    let mut pai: PATTACHINFO;
    let mut Ret: BOOL = 1;
    let mut ptiFrom: PTHREADINFO = core::ptr::null_mut();
    let mut ptiTo: PTHREADINFO = core::ptr::null_mut();

    loop {
        if gpai.is_null() {
            return 1;
        }

        pai = gpai;

        loop {
            if (*pai).pti2 == pti {
                ptiFrom = (*pai).pti1;
                ptiTo = pti;
                break;
            }
            if (*pai).pti1 == pti {
                ptiFrom = pti;
                ptiTo = (*pai).pti2;
                break;
            }
            pai = (*pai).paiNext;

            if pai.is_null() {
                break;
            }
        }

        if pai.is_null() && ptiFrom.is_null() && ptiTo.is_null() {
            break;
        }

        Status = UserAttachThreadInput(ptiFrom, ptiTo, 0);
        if !NT_SUCCESS(Status) {
            Ret = 0;
        }

        if Ret == 0 {
            break;
        }
    }

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserAttachThreadInput(
    ptiFrom: PTHREADINFO,
    ptiTo: PTHREADINFO,
    fAttach: BOOL,
) -> NTSTATUS {
    UserAttachThreadInput_impl(ptiFrom, ptiTo, fAttach)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserAttachThreadInput(
    ptiFrom: PTHREADINFO,
    ptiTo: PTHREADINFO,
    fAttach: BOOL,
) -> NTSTATUS {
    UserAttachThreadInput_impl(ptiFrom, ptiTo, fAttach)
}

unsafe fn UserAttachThreadInput_impl(
    ptiFrom: PTHREADINFO,
    ptiTo: PTHREADINFO,
    fAttach: BOOL,
) -> NTSTATUS {
    let mut msg: MSG = core::mem::zeroed();
    let mut pai: PATTACHINFO;
    let mut CurIcon: PCURICON_OBJECT;

    if ptiFrom == ptiTo {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*ptiFrom).TIF_flags & TIF_DONTATTACHQUEUE) != 0
        || ((*ptiTo).TIF_flags & TIF_DONTATTACHQUEUE) != 0
        || (*ptiFrom).rpdesk != (*ptiTo).rpdesk
    {
        return STATUS_ACCESS_DENIED;
    }

    if fAttach != 0 {
        pai = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<ATTACHINFO>() as _, USERTAG_ATTACHINFO) as PATTACHINFO;
        if pai.is_null() {
            return STATUS_NO_MEMORY;
        }

        (*pai).paiNext = gpai;
        (*pai).pti1 = ptiFrom;
        (*pai).pti2 = ptiTo;
        gpai = pai;
        paiCount += 1;

        if (*ptiTo).MessageQueue != (*ptiFrom).MessageQueue {
            (*(*ptiTo).MessageQueue).iCursorLevel -= (*ptiFrom).iCursorLevel;

            if (*ptiFrom).MessageQueue == gpqForeground {
                (*(*ptiTo).MessageQueue).spwndActive = (*(*ptiFrom).MessageQueue).spwndActive;
                (*(*ptiTo).MessageQueue).spwndFocus = (*(*ptiFrom).MessageQueue).spwndFocus;
                (*(*ptiTo).MessageQueue).spwndCapture = (*(*ptiFrom).MessageQueue).spwndCapture;
                (*(*ptiTo).MessageQueue).QF_flags ^=
                    ((*(*ptiTo).MessageQueue).QF_flags ^ (*(*ptiFrom).MessageQueue).QF_flags) & QF_CAPTURELOCKED;
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*(*ptiTo).MessageQueue).CaretInfo) as *mut _,
                    core::ptr::addr_of_mut!((*(*ptiFrom).MessageQueue).CaretInfo) as *const _,
                    core::mem::size_of_val(&(*(*ptiTo).MessageQueue).CaretInfo),
                );
                IntSetFocusMessageQueue(core::ptr::null_mut());
                IntSetFocusMessageQueue((*ptiTo).MessageQueue);
                gptiForeground = ptiTo;
            } else {
                if (*(*ptiTo).MessageQueue).spwndActive.is_null() {
                    (*(*ptiTo).MessageQueue).spwndActive = (*(*ptiFrom).MessageQueue).spwndActive;
                }
                if (*(*ptiTo).MessageQueue).spwndFocus.is_null() {
                    (*(*ptiTo).MessageQueue).spwndFocus = (*(*ptiFrom).MessageQueue).spwndFocus;
                }
            }

            CurIcon = (*(*ptiFrom).MessageQueue).CursorObject;

            MsqDestroyMessageQueue(ptiFrom);

            if !CurIcon.is_null() {
                UserReferenceObject(CurIcon as *mut _);
            }

            if !CurIcon.is_null() && UserObjectInDestroy(UserHMGetHandle(CurIcon)) != 0 {
                UserDereferenceObject(CurIcon as *mut _);
                CurIcon = core::ptr::null_mut();
            }

            (*ptiFrom).MessageQueue = (*ptiTo).MessageQueue;

            if !CurIcon.is_null() && (*(*ptiTo).MessageQueue).CursorObject.is_null() {
                (*(*ptiTo).MessageQueue).CursorObject = CurIcon;
            }

            (*(*ptiFrom).MessageQueue).cThreads += 1;

            IntReferenceMessageQueue((*ptiTo).MessageQueue);
        }
    } else {
        let mut Hit: BOOL = 0;
        let mut ppai: *mut PATTACHINFO;

        if gpai.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        ppai = core::ptr::addr_of_mut!(gpai);
        while !(*ppai).is_null() {
            if (**ppai).pti2 == ptiTo && (**ppai).pti1 == ptiFrom {
                pai = *ppai;
                *ppai = (**ppai).paiNext;
                ExFreePoolWithTag(pai as *mut _, USERTAG_ATTACHINFO);
                paiCount -= 1;
                Hit = 1;
                break;
            }
            ppai = core::ptr::addr_of_mut!((**ppai).paiNext);
        }

        if Hit == 0 {
            return STATUS_INVALID_PARAMETER;
        }

        if (*ptiTo).MessageQueue == (*ptiFrom).MessageQueue {
            let spwndActive: PWND = (*(*ptiTo).MessageQueue).spwndActive;
            let spwndFocus: PWND = (*(*ptiTo).MessageQueue).spwndFocus;

            if gptiForeground == ptiFrom {
                gptiForeground = ptiTo;
            }
            (*(*ptiTo).MessageQueue).cThreads -= 1;

            IntDereferenceMessageQueue((*ptiTo).MessageQueue);

            (*ptiFrom).MessageQueue = MsqCreateMessageQueue(ptiFrom);

            if !spwndActive.is_null() {
                if head_pti(core::ptr::addr_of!((*spwndActive).head)) == ptiFrom {
                    (*(*ptiFrom).MessageQueue).spwndActive = spwndActive;
                    (*(*ptiTo).MessageQueue).spwndActive = core::ptr::null_mut();
                }
            }
            if !spwndFocus.is_null() {
                if head_pti(core::ptr::addr_of!((*spwndFocus).head)) == ptiFrom {
                    (*(*ptiFrom).MessageQueue).spwndFocus = spwndFocus;
                    (*(*ptiTo).MessageQueue).spwndFocus = core::ptr::null_mut();
                }
            }
            (*(*ptiTo).MessageQueue).iCursorLevel -= (*ptiFrom).iCursorLevel;
        }
    }

    RtlCopyMemory(
        (*(*ptiTo).MessageQueue).afKeyState.as_mut_ptr() as *mut _,
        (*(&raw const gafAsyncKeyState)).as_ptr() as *const _,
        core::mem::size_of_val(&*(&raw const gafAsyncKeyState)),
    );

    (*(*ptiTo).MessageQueue).msgDblClk.message = 0;

    msg.message = WM_MOUSEMOVE;
    msg.wParam = UserGetMouseButtonsState() as WPARAM;
    msg.lParam = MAKELPARAM((*gpsi_per_user()).ptCursor.x, (*gpsi_per_user()).ptCursor.y);
    msg.pt = core::mem::transmute_copy(&(*gpsi_per_user()).ptCursor);
    co_MsqInsertMouseMessage(core::ptr::addr_of_mut!(msg), 0, 0, 1);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn NtUserAttachThreadInput(
    idAttach: DWORD,
    idAttachTo: DWORD,
    fAttach: BOOL,
) -> BOOL {
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'block: {
        let pti = IntTID2PTI(UlongToHandle(idAttach));
        let ptiTo = IntTID2PTI(UlongToHandle(idAttachTo));

        if pti.is_null() || ptiTo.is_null() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'block;
        }

        let Status = UserAttachThreadInput(pti, ptiTo, fAttach);
        if !NT_SUCCESS(Status) {
            EngSetLastError(RtlNtStatusToDosError(Status));
        } else {
            Ret = 1;
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSendInput(
    mut nInputs: UINT,
    mut pInput: LPINPUT,
    cbSize: INT,
) -> UINT {
    let pti: PTHREADINFO;
    let mut uRet: UINT = 0;

    UserEnterExclusive();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    'cleanup: {
        if (*pti).rpdesk.is_null() {
            break 'cleanup;
        }

        if nInputs == 0
            || pInput.is_null()
            || cbSize as usize != core::mem::size_of::<INPUT>()
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        if ThreadHasInputAccess(pti) == 0 || IntIsActiveDesktop((*pti).rpdesk) == 0 {
            EngSetLastError(ERROR_ACCESS_DENIED);
            break 'cleanup;
        }

        while nInputs != 0 {
            nInputs -= 1;

            let mut SafeInput: INPUT = core::mem::zeroed();
            let Status: NTSTATUS;

            Status = MmCopyFromCaller(
                core::ptr::addr_of_mut!(SafeInput) as *mut _,
                pInput as *mut _,
                core::mem::size_of::<INPUT>(),
            );
            pInput = pInput.add(1);

            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                break 'cleanup;
            }

            match SafeInput.type_ {
                INPUT_MOUSE => {
                    if UserSendMouseInput(
                        core::ptr::addr_of_mut!(SafeInput.__bindgen_anon_1.mi),
                        1,
                    ) != 0
                    {
                        uRet += 1;
                    }
                }
                INPUT_KEYBOARD => {
                    if UserSendKeyboardInput(
                        core::ptr::addr_of_mut!(SafeInput.__bindgen_anon_1.ki),
                        1,
                    ) != 0
                    {
                        uRet += 1;
                    }
                }
                INPUT_HARDWARE => {
                    FIXME!("INPUT_HARDWARE not supported!\n");
                }
                _ => {
                    ERR!("SendInput(): Invalid input type: 0x%x\n", SafeInput.type_);
                }
            }
        }
    }

    UserLeave();
    uRet
}
