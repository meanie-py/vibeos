//! win32ss/user/ntuser/rawinput.c — raw input: device enumeration (upstream #9498) and the
//! WM_INPUT pipeline (upstream #8819, "Introduce raw input processing"). Before #8819 the
//! registration and data syscalls were ntstubs.rs bodies returning 0.
//!
//! The pipeline: an application registers for the generic mouse and/or keyboard usage;
//! while either is registered the RIT hands every device packet to `UserCreateRawInput`,
//! which parks a RAWINPUT on the target thread's queue (`RawInputListHead`) and posts a
//! hardware WM_INPUT whose lParam is that packet. The packet lives until the thread removes
//! the NEXT message (`UserUpdatePrevRawInput`), a `GetRawInputBuffer` consumes it, or the
//! queue dies.
//!
//! User memory: the C wraps each probe AND its access in one `_SEH2_TRY`. Here the probes are
//! validate-only (`safe_user_io.rs`), so every access to a user buffer goes through the
//! fault-tolerant `crate::copy` primitives, and a fault takes the C's except path:
//! `SetLastNtError(STATUS_ACCESS_VIOLATION)` with the return value left as it was.
//! #8819's GetRawInputBuffer, GetRawInputData and RegisterRawInputDevices touch their user
//! pointers with no probe and no SEH at all — a bad pointer from user mode is a kernel
//! fault there, and an unprobed kernel address is a write-anywhere. They get the same
//! treatment as the two older syscalls; on a fault they return their failure value.
#![allow(non_snake_case)]
use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use core::ffi::c_void;

use ntkrnl_sys::base::{STATUS_NO_MEMORY, STATUS_SUCCESS};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExInitializePagedLookasideList, PsGetCurrentThreadWin32Thread};

use crate::copy::{capture_from_user, publish_to_user};
use crate::desktop::IntGetFocusMessageQueue;
use crate::error::{EngSetLastError, SetLastNtError};
use crate::input::{gpInputDeviceInfo, AcquireDeviceInfoListMutex, ReleaseDeviceInfoListMutex};
use crate::mem::EngAllocMem;
use crate::msgqueue::{ClearMsgBitsMask, MsqDestroyMessage};
use crate::ntuser::{UserEnterExclusive, UserEnterShared, UserLeave};
use crate::safe_user_io::{SafeProbeForRead, SafeProbeForWrite};
use crate::window::UserGetWindowObject;

// Real NTAPI ntoskrnl exports (see msgqueue.rs): the exfuncs.h inline is this code.
extern "system" {
    fn ExAllocateFromPagedLookasideList(Lookaside: PPAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToPagedLookasideList(Lookaside: PPAGED_LOOKASIDE_LIST, Entry: PVOID);
}

// rawinput.c defines these itself, under #ifndef, because hidusage.h is not in win32k's
// include closure.
const HID_USAGE_PAGE_GENERIC: USHORT = 0x01;
const HID_USAGE_GENERIC_MOUSE: USHORT = 0x02;
const HID_USAGE_GENERIC_KEYBOARD: USHORT = 0x06;

/// `'iwar'`, the C multi-char literal (stored little-endian, so "rawi" in a pool dump).
const TAG_RAWINPUT: ULONG = u32::from_be_bytes(*b"iwar");

#[repr(C)]
struct USER_RAWINPUT {
    ListEntry: LIST_ENTRY,
    RawInput: RAWINPUT,
}
type PUSER_RAWINPUT = *mut USER_RAWINPUT;

// rawinput.c's C_ASSERTs.
const _: () = assert!(core::mem::offset_of!(RAWINPUT, data) == core::mem::size_of::<RAWINPUTHEADER>());
const _: () = assert!(core::mem::size_of::<RAWINPUTHEADER>() + core::mem::size_of::<RAWMOUSE>() <= core::mem::size_of::<RAWINPUT>());
const _: () = assert!(core::mem::size_of::<RAWINPUTHEADER>() + core::mem::size_of::<RAWKEYBOARD>() <= core::mem::size_of::<RAWINPUT>());

const MAX_RAWINPUT_PER_QUEUE: ULONG = 256;

// The ntdef.h list inlines, over this island's LIST_ENTRY (as msgqueue.rs does).
#[inline(always)]
unsafe fn IsListEmpty(h: PLIST_ENTRY) -> bool {
    (*h).Flink == h
}
#[inline(always)]
unsafe fn InsertTailList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let b = (*h).Blink;
    (*e).Flink = h;
    (*e).Blink = b;
    (*b).Flink = e;
    (*h).Blink = e;
}
#[inline(always)]
unsafe fn RemoveEntryList(e: PLIST_ENTRY) {
    let f = (*e).Flink;
    let b = (*e).Blink;
    (*b).Flink = f;
    (*f).Blink = b;
}

/// The C's file-static `PAGED_LOOKASIDE_LIST`. Pool-allocated like msgqueue.rs's lists, which
/// guarantees the 16-byte SLIST_HEADER alignment amd64 needs without trusting a static's.
static mut gRawInputLookasideList: PPAGED_LOOKASIDE_LIST = core::ptr::null_mut();
/// Two slots: [0] the generic-mouse registration, [1] the generic-keyboard one. A zeroed slot
/// (usUsage == 0) is "not registered".
pub static mut global_pRawInputDevices: PRAWINPUTDEVICE = core::ptr::null_mut();
pub static mut RawInputEnabled: BOOLEAN = 0;

const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as NTSTATUS;
const DWORD_ALIGN: ULONG = core::mem::size_of::<DWORD>() as ULONG;

/// The body of one C `_SEH2_TRY`: `Err(status)` is the exception the C would raise.
type Seh = Result<(), NTSTATUS>;

unsafe fn probe_read(p: *const c_void, len: usize, align: ULONG) -> Seh {
    let st = SafeProbeForRead(p as *mut c_void, len as SIZE_T, align);
    if NT_SUCCESS(st) { Ok(()) } else { Err(st) }
}

unsafe fn probe_write(p: *mut c_void, len: usize, align: ULONG) -> Seh {
    let st = SafeProbeForWrite(p, len as SIZE_T, align);
    if NT_SUCCESS(st) { Ok(()) } else { Err(st) }
}

unsafe fn read<T: Copy>(src: *const T) -> Result<T, NTSTATUS> {
    let mut v = core::mem::MaybeUninit::<T>::uninit();
    if capture_from_user(v.as_mut_ptr() as *mut c_void, src as *const c_void, core::mem::size_of::<T>()) {
        Ok(v.assume_init())
    } else {
        Err(STATUS_ACCESS_VIOLATION)
    }
}

unsafe fn copy_out(dst: *mut c_void, src: *const c_void, len: usize) -> Seh {
    if publish_to_user(dst, src, len) { Ok(()) } else { Err(STATUS_ACCESS_VIOLATION) }
}

unsafe fn write<T: Copy>(dst: *mut T, val: T) -> Seh {
    copy_out(dst as *mut c_void, &val as *const T as *const c_void, core::mem::size_of::<T>())
}

/// `*pcbSize = cb` after its ProbeForWrite — the "tell the caller the size" step both paths share.
unsafe fn store_size(pcbSize: PUINT, cb: UINT) -> Seh {
    probe_write(pcbSize as *mut c_void, core::mem::size_of::<UINT>(), DWORD_ALIGN)?;
    write(pcbSize, cb)
}

#[no_mangle]
pub unsafe extern "system" fn NtUserGetRawInputDeviceInfo(
    hDevice: HANDLE,
    uiCommand: UINT,
    pData: LPVOID,
    pcbSize: PUINT,
) -> DWORD {
    let mut Ret: UINT = UINT::MAX;

    let cbSize: UINT = match probe_read(pcbSize as *const c_void, core::mem::size_of::<UINT>(), DWORD_ALIGN)
        .and_then(|()| read(pcbSize))
    {
        Ok(v) => v,
        Err(st) => {
            SetLastNtError(st);
            return Ret;
        }
    };

    AcquireDeviceInfoListMutex();
    let mut DeviceInfo = gpInputDeviceInfo;
    while !DeviceInfo.is_null() && DeviceInfo as HANDLE != hDevice {
        DeviceInfo = (*DeviceInfo).pNextDeviceInfo;
    }
    if DeviceInfo.is_null() {
        ReleaseDeviceInfoListMutex();
        EngSetLastError(ERROR_INVALID_HANDLE as ULONG);
        return Ret;
    }

    UserEnterShared();

    let dev = &*DeviceInfo;
    let cbRequiredSize: UINT = match uiCommand as u32 {
        RIDI_PREPARSEDDATA => {
            if dev.DeviceType == RIM_TYPEHID as ULONG {
                dev.__bindgen_anon_1.Hid.CollectionInformation.DescriptorSize as UINT
            } else {
                0
            }
        }
        RIDI_DEVICENAME => {
            dev.DeviceName.Length as UINT / core::mem::size_of::<WCHAR>() as UINT
                + core::mem::size_of::<CHAR>() as UINT // sizeof(ANSI_NULL)
        }
        RIDI_DEVICEINFO => core::mem::size_of::<RID_DEVICE_INFO>() as UINT,
        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
            UserLeave();
            ReleaseDeviceInfoListMutex();
            return Ret;
        }
    };

    let body = (|| -> Seh {
        if pData.is_null() {
            store_size(pcbSize, cbRequiredSize)?;
            Ret = 0;
            return Ok(());
        }

        if cbSize < cbRequiredSize {
            store_size(pcbSize, cbRequiredSize)?;
            EngSetLastError(ERROR_INSUFFICIENT_BUFFER as ULONG);
            return Ok(());
        }

        probe_write(pData, cbRequiredSize as usize, DWORD_ALIGN)?;
        match uiCommand as u32 {
            RIDI_PREPARSEDDATA => {
                if dev.DeviceType == RIM_TYPEHID as ULONG {
                    copy_out(
                        pData,
                        dev.__bindgen_anon_1.Hid.PreparsedData as *const c_void,
                        cbRequiredSize as usize,
                    )?;
                }
            }
            RIDI_DEVICENAME => {
                // cbSize/cbRequiredSize count chars, so the probe above covered half the
                // buffer: probe the real byte length again, as the C does.
                let bytes = dev.DeviceName.Length as usize;
                probe_write(pData, bytes + core::mem::size_of::<WCHAR>(), DWORD_ALIGN)?;
                copy_out(pData, dev.DeviceName.Buffer as *const c_void, bytes)?;
                write((pData as *mut WCHAR).add(cbRequiredSize as usize - 1), 0)?;
            }
            RIDI_DEVICEINFO => {
                let prdi = pData as PRID_DEVICE_INFO;
                probe_read(
                    core::ptr::addr_of!((*prdi).cbSize) as *const c_void,
                    core::mem::size_of::<DWORD>(),
                    DWORD_ALIGN,
                )?;
                if read(core::ptr::addr_of!((*prdi).cbSize))? != cbRequiredSize {
                    // Divergence from rawinput.c: its `_SEH2_YIELD(break)` leaves only the
                    // inner switch, so the C falls through to `Ret = cbRequiredSize` and
                    // reports success for a buffer it never filled, with ERROR_INVALID_PARAMETER
                    // set beside it. Keep Ret at (UINT)-1, the error the code already chose.
                    EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
                    return Ok(());
                }

                probe_write(prdi as *mut c_void, core::mem::size_of::<RID_DEVICE_INFO>(), DWORD_ALIGN)?;
                // RtlZeroMemory + field stores, built in kernel memory and published once.
                let mut info: RID_DEVICE_INFO = core::mem::zeroed();
                info.cbSize = core::mem::size_of::<RID_DEVICE_INFO>() as DWORD;
                info.dwType = dev.DeviceType as DWORD;
                match dev.DeviceType as u32 {
                    RIM_TYPEMOUSE => {
                        let a = &dev.__bindgen_anon_1.Mouse.Attributes;
                        let m = &mut info.__bindgen_anon_1.mouse;
                        m.dwId = (a.MouseIdentifier as u32 & !HORIZONTAL_WHEEL_PRESENT) as DWORD;
                        m.dwNumberOfButtons = a.NumberOfButtons as DWORD;
                        m.dwSampleRate = a.SampleRate as DWORD;
                        m.fHasHorizontalWheel =
                            (a.MouseIdentifier as u32 & HORIZONTAL_WHEEL_PRESENT != 0) as BOOL;
                    }
                    RIM_TYPEKEYBOARD => {
                        let a = &dev.__bindgen_anon_1.Keyboard.Attributes;
                        let k = &mut info.__bindgen_anon_1.keyboard;
                        k.dwType = a.KeyboardIdentifier.Type as DWORD;
                        k.dwSubType = a.KeyboardIdentifier.Subtype as DWORD;
                        k.dwKeyboardMode = a.KeyboardMode as DWORD;
                        k.dwNumberOfFunctionKeys = a.NumberOfFunctionKeys as DWORD;
                        k.dwNumberOfIndicators = a.NumberOfIndicators as DWORD;
                        k.dwNumberOfKeysTotal = a.NumberOfKeysTotal as DWORD;
                    }
                    RIM_TYPEHID => {
                        let h = &dev.__bindgen_anon_1.Hid;
                        let d = &mut info.__bindgen_anon_1.hid;
                        d.dwVendorId = h.CollectionInformation.VendorID as DWORD;
                        d.dwProductId = h.CollectionInformation.ProductID as DWORD;
                        d.dwVersionNumber = h.CollectionInformation.VersionNumber as DWORD;
                        d.usUsagePage = h.Caps.UsagePage;
                        d.usUsage = h.Caps.Usage;
                    }
                    _ => {}
                }
                write(prdi, info)?;
            }
            _ => ntkrnl_sys::ASSERT!(false),
        }
        Ret = cbRequiredSize;
        Ok(())
    })();
    if let Err(st) = body {
        SetLastNtError(st);
    }

    UserLeave();
    ReleaseDeviceInfoListMutex();
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn NtUserGetRawInputDeviceList(
    pRawInputDeviceList: PRAWINPUTDEVICELIST,
    puiNumDevices: PUINT,
    cbSize: UINT,
) -> DWORD {
    let mut Ret: UINT = UINT::MAX;

    if cbSize as usize != core::mem::size_of::<RAWINPUTDEVICELIST>() {
        EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
        return Ret;
    }

    AcquireDeviceInfoListMutex();
    let mut cDevices: UINT = 0;
    let mut DeviceInfo = gpInputDeviceInfo;
    while !DeviceInfo.is_null() {
        cDevices += 1;
        DeviceInfo = (*DeviceInfo).pNextDeviceInfo;
    }

    UserEnterShared();

    let body = (|| -> Seh {
        if pRawInputDeviceList.is_null() {
            store_size(puiNumDevices, cDevices)?;
            Ret = 0;
            return Ok(());
        }

        probe_read(puiNumDevices as *const c_void, core::mem::size_of::<UINT>(), DWORD_ALIGN)?;
        if read(puiNumDevices)? < cDevices {
            store_size(puiNumDevices, cDevices)?;
            EngSetLastError(ERROR_INSUFFICIENT_BUFFER as ULONG);
            return Ok(());
        }

        probe_write(
            pRawInputDeviceList as *mut c_void,
            cDevices as usize * core::mem::size_of::<RAWINPUTDEVICELIST>(),
            core::mem::size_of::<PVOID>() as ULONG,
        )?;
        let mut DeviceInfo = gpInputDeviceInfo;
        let mut i: UINT = 0;
        while !DeviceInfo.is_null() && i < cDevices {
            let entry = RAWINPUTDEVICELIST {
                hDevice: DeviceInfo as HANDLE,
                dwType: (*DeviceInfo).DeviceType as DWORD,
            };
            write(pRawInputDeviceList.add(i as usize), entry)?;
            DeviceInfo = (*DeviceInfo).pNextDeviceInfo;
            i += 1;
        }
        Ret = cDevices;
        Ok(())
    })();
    if let Err(st) = body {
        SetLastNtError(st);
    }

    UserLeave();
    ReleaseDeviceInfoListMutex();
    Ret
}


fn RawInputDeviceIndexFromType(dwType: DWORD) -> INT {
    match dwType {
        RIM_TYPEMOUSE => 0,
        RIM_TYPEKEYBOARD => 1,
        _ => -1,
    }
}

unsafe fn RawInputRegistrationFromType(dwType: DWORD) -> PRAWINPUTDEVICE {
    let Index = RawInputDeviceIndexFromType(dwType);

    if global_pRawInputDevices.is_null() || Index < 0 {
        return core::ptr::null_mut();
    }

    let Registration = global_pRawInputDevices.add(Index as usize);
    if (*Registration).usUsagePage != HID_USAGE_PAGE_GENERIC {
        return core::ptr::null_mut();
    }

    if (dwType == RIM_TYPEMOUSE && (*Registration).usUsage != HID_USAGE_GENERIC_MOUSE)
        || (dwType == RIM_TYPEKEYBOARD && (*Registration).usUsage != HID_USAGE_GENERIC_KEYBOARD)
    {
        return core::ptr::null_mut();
    }

    Registration
}

unsafe fn RawInputWarnUnimplementedFlags(Device: &RAWINPUTDEVICE) {
    let IsMouse = Device.usUsagePage == HID_USAGE_PAGE_GENERIC && Device.usUsage == HID_USAGE_GENERIC_MOUSE;
    let IsKeyboard = Device.usUsagePage == HID_USAGE_PAGE_GENERIC && Device.usUsage == HID_USAGE_GENERIC_KEYBOARD;
    let PageFlags = Device.dwFlags & (RIDEV_PAGEONLY | RIDEV_EXCLUDE);

    if PageFlags == RIDEV_NOLEGACY && (IsMouse || IsKeyboard) {
        DPRINT1!(
            "RIDEV_NOLEGACY for raw %s is accepted but not implemented yet.\n",
            if IsMouse { b"mouse\0".as_ptr() } else { b"keyboard\0".as_ptr() },
        );
    } else {
        if Device.dwFlags & RIDEV_PAGEONLY != 0 {
            DPRINT1!("RIDEV_PAGEONLY is accepted but not implemented yet.\n");
        }
        if Device.dwFlags & RIDEV_EXCLUDE != 0 {
            DPRINT1!("RIDEV_EXCLUDE is accepted but not implemented yet.\n");
        }
    }

    if IsMouse && Device.dwFlags & RIDEV_CAPTUREMOUSE != 0 {
        DPRINT1!("RIDEV_CAPTUREMOUSE for raw mouse is accepted but not implemented yet.\n");
    }
    if IsKeyboard && Device.dwFlags & RIDEV_NOHOTKEYS != 0 {
        DPRINT1!("RIDEV_NOHOTKEYS for raw keyboard is accepted but not implemented yet.\n");
    }
    if IsKeyboard && Device.dwFlags & RIDEV_APPKEYS != 0 {
        DPRINT1!("RIDEV_APPKEYS for raw keyboard is accepted but not implemented yet.\n");
    }
    if Device.dwFlags & RIDEV_EXINPUTSINK != 0 {
        DPRINT1!("RIDEV_EXINPUTSINK is only partially implemented.\n");
    }
    if Device.dwFlags & RIDEV_DEVNOTIFY != 0 {
        DPRINT1!("RIDEV_DEVNOTIFY is accepted but not implemented yet.\n");
    }
}

/// CONTAINING_RECORD(Entry, USER_RAWINPUT, ListEntry). ListEntry is the first field.
#[inline(always)]
unsafe fn raw_entry(Entry: PLIST_ENTRY) -> PUSER_RAWINPUT {
    (Entry as *mut u8).sub(core::mem::offset_of!(USER_RAWINPUT, ListEntry)) as PUSER_RAWINPUT
}

#[inline(always)]
unsafe fn user_message(Entry: PLIST_ENTRY) -> PUSER_MESSAGE {
    (Entry as *mut u8).sub(core::mem::offset_of!(USER_MESSAGE, ListEntry)) as PUSER_MESSAGE
}

/// Only a handle this queue issued resolves: an HRAWINPUT is the address of the packet, so
/// the walk is what keeps a forged lParam from being dereferenced.
unsafe fn RawInputEntryFromHandle(MessageQueue: PUSER_MESSAGE_QUEUE, hRawInput: HRAWINPUT) -> PUSER_RAWINPUT {
    if MessageQueue.is_null() || hRawInput.is_null() {
        return core::ptr::null_mut();
    }

    let Head = core::ptr::addr_of_mut!((*MessageQueue).RawInputListHead);
    let mut Entry = (*Head).Flink;
    while Entry != Head {
        let RawEntry = raw_entry(Entry);
        if core::ptr::addr_of_mut!((*RawEntry).RawInput) as HRAWINPUT == hRawInput {
            return RawEntry;
        }
        Entry = (*Entry).Flink;
    }

    core::ptr::null_mut()
}

unsafe fn RawInputFreeEntry(MessageQueue: PUSER_MESSAGE_QUEUE, RawEntry: PUSER_RAWINPUT) {
    ntkrnl_sys::ASSERT!((*MessageQueue).cRawInput > 0);
    RemoveEntryList(core::ptr::addr_of_mut!((*RawEntry).ListEntry));
    (*MessageQueue).cRawInput -= 1;
    ExFreeToPagedLookasideList(gRawInputLookasideList, RawEntry as PVOID);
}

pub unsafe extern "system" fn InitRawInputImpl() -> NTSTATUS {
    gRawInputLookasideList = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<PAGED_LOOKASIDE_LIST>() as SIZE_T,
        TAG_RAWINPUT,
    ) as PPAGED_LOOKASIDE_LIST;
    if gRawInputLookasideList.is_null() {
        return STATUS_NO_MEMORY;
    }
    ExInitializePagedLookasideList(
        gRawInputLookasideList as _,
        None,
        None,
        0,
        core::mem::size_of::<USER_RAWINPUT>() as _,
        TAG_RAWINPUT,
        256,
    );
    STATUS_SUCCESS
}

pub unsafe fn UserCreateRawInput(
    pti: PTHREADINFO,
    dwType: DWORD,
    hDevice: HANDLE,
    wParam: WPARAM,
    pData: *const c_void,
    cbData: UINT,
) -> HRAWINPUT {
    if pti.is_null() || (*pti).MessageQueue.is_null() || pData.is_null() {
        return core::ptr::null_mut();
    }

    let cbMax = core::mem::size_of::<RAWINPUT>() - core::mem::offset_of!(RAWINPUT, data);
    ntkrnl_sys::ASSERT!(cbData as usize <= cbMax);
    if cbData as usize > cbMax {
        return core::ptr::null_mut();
    }

    let MessageQueue = (*pti).MessageQueue;
    if (*MessageQueue).cRawInput >= MAX_RAWINPUT_PER_QUEUE {
        DPRINT!("Dropping raw input, queue %p is not being pumped\n", MessageQueue);
        return core::ptr::null_mut();
    }

    let RawEntry = ExAllocateFromPagedLookasideList(gRawInputLookasideList) as PUSER_RAWINPUT;
    if RawEntry.is_null() {
        return core::ptr::null_mut();
    }

    core::ptr::write_bytes(RawEntry, 0, 1);
    let Raw = &mut (*RawEntry).RawInput;
    Raw.header.dwType = dwType;
    Raw.header.dwSize = (core::mem::size_of::<RAWINPUTHEADER>() + cbData as usize) as DWORD;
    Raw.header.hDevice = hDevice;
    Raw.header.wParam = wParam;
    core::ptr::copy_nonoverlapping(
        pData as *const u8,
        core::ptr::addr_of_mut!(Raw.data) as *mut u8,
        cbData as usize,
    );

    InsertTailList(
        core::ptr::addr_of_mut!((*MessageQueue).RawInputListHead),
        core::ptr::addr_of_mut!((*RawEntry).ListEntry),
    );
    (*MessageQueue).cRawInput += 1;
    core::ptr::addr_of_mut!((*RawEntry).RawInput) as HRAWINPUT
}

pub unsafe fn UserFreeRawInput(MessageQueue: PUSER_MESSAGE_QUEUE, hRawInput: HRAWINPUT) -> BOOL {
    let RawEntry = RawInputEntryFromHandle(MessageQueue, hRawInput);
    if RawEntry.is_null() {
        return FALSE as BOOL;
    }
    RawInputFreeEntry(MessageQueue, RawEntry);
    TRUE as BOOL
}

pub unsafe fn UserUpdatePrevRawInput(pti: PTHREADINFO, pMsg: *const MSG) {
    if !(*pti).hPrevRawInput.is_null() {
        UserFreeRawInput((*pti).MessageQueue, (*pti).hPrevRawInput);
        (*pti).hPrevRawInput = core::ptr::null_mut();
    }

    if (*pMsg).message == WM_INPUT
        && ((*pMsg).wParam == RIM_INPUT as WPARAM || (*pMsg).wParam == RIM_INPUTSINK as WPARAM)
    {
        (*pti).hPrevRawInput = (*pMsg).lParam as HRAWINPUT;
    }
}

pub unsafe fn UserCleanupRawInput(MessageQueue: PUSER_MESSAGE_QUEUE) {
    let Head = core::ptr::addr_of_mut!((*MessageQueue).RawInputListHead);
    while !IsListEmpty(Head) {
        RawInputFreeEntry(MessageQueue, raw_entry((*Head).Flink));
    }
}

/// UserHMGetHandle(pWnd) = pWnd->head.h, the object handle at offset 0.
#[inline(always)]
unsafe fn wnd_handle(pWnd: PWND) -> HWND {
    *(pWnd as *const HWND)
}

/// pWnd->head.pti — bindgen flattens THRDESKHEAD, so the pti is read at its offset
/// (after HANDLE h + DWORD cLockObj + 64-bit tail padding; matches keyboard.rs head_pti).
#[inline(always)]
unsafe fn wnd_pti(pWnd: PWND) -> PTHREADINFO {
    *((pWnd as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

/// Which thread and window a packet of `dwType` goes to, and whether as RIM_INPUT or
/// RIM_INPUTSINK. FALSE drops the packet (the legacy message is still generated).
pub unsafe fn UserGetRawInputTarget(
    dwType: DWORD,
    ppti: *mut PTHREADINFO,
    phwndTarget: *mut HWND,
    pwParam: *mut WPARAM,
) -> BOOL {
    let Registration = RawInputRegistrationFromType(dwType);
    if Registration.is_null() {
        return FALSE as BOOL;
    }

    let pFocusQueue = IntGetFocusMessageQueue();
    if pFocusQueue.is_null() {
        return FALSE as BOOL;
    }

    let pWnd: PWND;
    if !(*Registration).hwndTarget.is_null() {
        pWnd = UserGetWindowObject((*Registration).hwndTarget);
        if pWnd.is_null() {
            return FALSE as BOOL;
        }
    } else {
        let mut w = (*pFocusQueue).spwndFocus;
        if w.is_null() {
            w = (*pFocusQueue).spwndActive;
        }
        if w.is_null() {
            return FALSE as BOOL;
        }
        pWnd = w;
    }

    let pti = wnd_pti(pWnd);
    if pti.is_null() {
        return FALSE as BOOL;
    }

    if !(*pFocusQueue).ptiKeyboard.is_null() && (*pti).rpdesk != (*(*pFocusQueue).ptiKeyboard).rpdesk {
        return FALSE as BOOL;
    }

    if (*Registration).dwFlags & RIDEV_INPUTSINK != 0 {
        *pwParam = if (*pti).MessageQueue == pFocusQueue { RIM_INPUT } else { RIM_INPUTSINK } as WPARAM;
        *phwndTarget = wnd_handle(pWnd);
        *ppti = pti;
        return TRUE as BOOL;
    }

    if (*Registration).dwFlags & RIDEV_EXINPUTSINK != 0 {
        if (*pti).MessageQueue != pFocusQueue {
            return FALSE as BOOL; /* FIXME: Needs per-process arbitration like Windows */
        }
        *pwParam = RIM_INPUT as WPARAM;
        *phwndTarget = wnd_handle(pWnd);
        *ppti = pti;
        return TRUE as BOOL;
    }

    if (*pti).MessageQueue != pFocusQueue {
        return FALSE as BOOL;
    }

    *pwParam = RIM_INPUT as WPARAM;
    *phwndTarget = wnd_handle(pWnd);
    *ppti = pti;
    TRUE as BOOL
}

/// RawInputCopyData. `pcbSize` and `pData` are USER pointers here (the C reads and writes
/// them raw); `Err` is the fault the C would have taken as an unhandled kernel exception.
unsafe fn RawInputCopyData(
    RawInput: *const RAWINPUT,
    uiCommand: UINT,
    pData: LPVOID,
    pcbSize: PUINT,
) -> Result<DWORD, NTSTATUS> {
    let OutputDataSize: UINT = match uiCommand {
        RID_INPUT => (*RawInput).header.dwSize,
        RID_HEADER => core::mem::size_of::<RAWINPUTHEADER>() as UINT,
        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
            return Ok(!0u32);
        }
    };

    if pData.is_null() {
        store_size(pcbSize, OutputDataSize)?;
        EngSetLastError(ERROR_SUCCESS as ULONG);
        return Ok(0);
    }

    probe_read(pcbSize as *const c_void, core::mem::size_of::<UINT>(), DWORD_ALIGN)?;
    if read(pcbSize)? < OutputDataSize {
        store_size(pcbSize, OutputDataSize)?;
        EngSetLastError(ERROR_INSUFFICIENT_BUFFER as ULONG);
        return Ok(!0u32);
    }

    // RID_HEADER copies the header, RID_INPUT the whole packet; both start at the header.
    probe_write(pData, OutputDataSize as usize, DWORD_ALIGN)?;
    copy_out(pData, RawInput as *const c_void, OutputDataSize as usize)?;
    Ok(OutputDataSize)
}

/* This file is pretty deeply inspired by how wine seems to do it, but wine can't be DIRECTLY used. */
#[no_mangle]
pub unsafe extern "system" fn NtUserGetRawInputBuffer(
    pData: PRAWINPUT,
    pcbSize: PUINT,
    cbSizeHeader: UINT,
) -> DWORD {
    if cbSizeHeader as usize != core::mem::size_of::<RAWINPUTHEADER>() || pcbSize.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
        return !0u32;
    }

    // The C walks and edits the queue with no lock while the RIT inserts into it under the
    // exclusive one; take it here.
    UserEnterExclusive();
    let r = GetRawInputBuffer(pData, pcbSize);
    UserLeave();
    match r {
        Ok(v) => v,
        Err(st) => {
            SetLastNtError(st);
            !0u32
        }
    }
}

unsafe fn GetRawInputBuffer(pData: PRAWINPUT, pcbSize: PUINT) -> Result<DWORD, NTSTATUS> {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let MessageQueue = if pti.is_null() { core::ptr::null_mut() } else { (*pti).MessageQueue };
    if MessageQueue.is_null() {
        store_size(pcbSize, 0)?;
        return Ok(0);
    }

    let Head = core::ptr::addr_of_mut!((*MessageQueue).HardwareMessagesListHead);
    let mut Entry = (*Head).Flink;
    if pData.is_null() {
        while Entry != Head {
            let Message = user_message(Entry);
            if (*Message).Msg.message == WM_INPUT {
                let RawEntry = RawInputEntryFromHandle(MessageQueue, (*Message).Msg.lParam as HRAWINPUT);
                let cb = if RawEntry.is_null() { 0 } else { (*RawEntry).RawInput.header.dwSize };
                store_size(pcbSize, cb)?;
                EngSetLastError(ERROR_SUCCESS as ULONG);
                return Ok(0);
            }
            Entry = (*Entry).Flink;
        }

        store_size(pcbSize, 0)?;
        return Ok(0);
    }

    probe_read(pcbSize as *const c_void, core::mem::size_of::<UINT>(), DWORD_ALIGN)?;
    let BufferSize: UINT = read(pcbSize)?;
    let mut UsedSize: UINT = 0;
    let mut Count: UINT = 0;
    while Entry != Head {
        let NextEntry = (*Entry).Flink;
        let Message = user_message(Entry);
        Entry = NextEntry;

        if (*Message).Msg.message != WM_INPUT {
            continue;
        }

        let RawEntry = RawInputEntryFromHandle(MessageQueue, (*Message).Msg.lParam as HRAWINPUT);
        if RawEntry.is_null() {
            continue;
        }

        let RawSize = (*RawEntry).RawInput.header.dwSize;
        if UsedSize as u64 + RawSize as u64 > BufferSize as u64 {
            if Count == 0 {
                store_size(pcbSize, RawSize)?;
                EngSetLastError(ERROR_INSUFFICIENT_BUFFER as ULONG);
                return Ok(!0u32);
            }
            break;
        }

        let dst = (pData as *mut u8).add(UsedSize as usize) as *mut c_void;
        probe_write(dst, RawSize as usize, DWORD_ALIGN)?;
        copy_out(dst, core::ptr::addr_of!((*RawEntry).RawInput) as *const c_void, RawSize as usize)?;
        UsedSize += RawSize;
        Count += 1;

        ClearMsgBitsMask(pti, (*Message).QS_Flags);
        MsqDestroyMessage(Message);
        RawInputFreeEntry(MessageQueue, RawEntry);
    }

    Ok(Count)
}

#[no_mangle]
pub unsafe extern "system" fn NtUserGetRawInputData(
    hRawInput: HRAWINPUT,
    uiCommand: UINT,
    pData: LPVOID,
    pcbSize: PUINT,
    cbSizeHeader: UINT,
) -> DWORD {
    if cbSizeHeader as usize != core::mem::size_of::<RAWINPUTHEADER>() || pcbSize.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
        return !0u32;
    }

    if hRawInput.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE as ULONG);
        return !0u32;
    }

    // Shared, unlike the C's no lock at all: the RIT inserts into this list under the
    // exclusive lock.
    UserEnterShared();
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let MessageQueue = if pti.is_null() { core::ptr::null_mut() } else { (*pti).MessageQueue };
    let RawEntry = RawInputEntryFromHandle(MessageQueue, hRawInput);
    let Ret = if RawEntry.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE as ULONG);
        !0u32
    } else {
        match RawInputCopyData(core::ptr::addr_of!((*RawEntry).RawInput), uiCommand, pData, pcbSize) {
            Ok(v) => v,
            Err(st) => {
                SetLastNtError(st);
                !0u32
            }
        }
    };
    UserLeave();
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn NtUserGetRegisteredRawInputDevices(
    pRawInputDevices: PRAWINPUTDEVICE,
    puiNumDevices: PUINT,
    cbSize: UINT,
) -> DWORD {
    let mut Ret: UINT = UINT::MAX;

    if cbSize as usize != core::mem::size_of::<RAWINPUTDEVICE>() {
        EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
        return Ret;
    }

    UserEnterShared();

    /* Snapshot the registrations, ordered by usage like Windows does */
    let mut Registrations: [RAWINPUTDEVICE; 2] = core::mem::zeroed();
    let mut cDevices: UINT = 0;
    for dwType in [RIM_TYPEMOUSE, RIM_TYPEKEYBOARD] {
        let Registration = RawInputRegistrationFromType(dwType);
        if !Registration.is_null() {
            Registrations[cDevices as usize] = *Registration;
            cDevices += 1;
        }
    }

    let body = (|| -> Seh {
        if pRawInputDevices.is_null() {
            store_size(puiNumDevices, cDevices)?;
            Ret = 0;
            return Ok(());
        }

        probe_read(puiNumDevices as *const c_void, core::mem::size_of::<UINT>(), DWORD_ALIGN)?;
        let uiNumDevices: UINT = read(puiNumDevices)?;
        if uiNumDevices == 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
            return Ok(());
        }

        /* Windows raises an access violation for counts it cannot probe */
        if uiNumDevices as usize > MAXULONG as usize / core::mem::size_of::<RAWINPUTDEVICE>() {
            EngSetLastError(ERROR_NOACCESS as ULONG);
            return Ok(());
        }

        probe_write(
            pRawInputDevices as *mut c_void,
            uiNumDevices as usize * core::mem::size_of::<RAWINPUTDEVICE>(),
            core::mem::size_of::<PVOID>() as ULONG,
        )?;
        if uiNumDevices < cDevices {
            store_size(puiNumDevices, cDevices)?;
            EngSetLastError(ERROR_INSUFFICIENT_BUFFER as ULONG);
            return Ok(());
        }

        copy_out(
            pRawInputDevices as *mut c_void,
            Registrations.as_ptr() as *const c_void,
            cDevices as usize * core::mem::size_of::<RAWINPUTDEVICE>(),
        )?;
        Ret = cDevices;
        Ok(())
    })();
    if let Err(st) = body {
        SetLastNtError(st);
    }

    UserLeave();
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn NtUserRegisterRawInputDevices(
    pRawInputDevices: PCRAWINPUTDEVICE,
    uiNumDevices: UINT,
    cbSize: UINT,
) -> BOOL {
    if pRawInputDevices.is_null() || uiNumDevices == 0 || cbSize as usize != core::mem::size_of::<RAWINPUTDEVICE>() {
        EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
        return FALSE as BOOL;
    }

    // The C edits the registration table with no lock while the RIT reads it under the
    // user lock; take it exclusively here.
    UserEnterExclusive();
    let Ret = match RegisterRawInputDevices(pRawInputDevices, uiNumDevices) {
        Ok(b) => b,
        Err(st) => {
            SetLastNtError(st);
            FALSE as BOOL
        }
    };
    UserLeave();
    Ret
}

unsafe fn RegisterRawInputDevices(pRawInputDevices: PCRAWINPUTDEVICE, uiNumDevices: UINT) -> Result<BOOL, NTSTATUS> {
    if global_pRawInputDevices.is_null() {
        let cb = 2 * core::mem::size_of::<RAWINPUTDEVICE>();
        global_pRawInputDevices = EngAllocMem(NonPagedPool as ULONG, cb as ULONG, TAG_RAWINPUT) as PRAWINPUTDEVICE;
        if global_pRawInputDevices.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY as ULONG);
            return Ok(FALSE as BOOL);
        }
        core::ptr::write_bytes(global_pRawInputDevices, 0, 2);
    }

    probe_read(
        pRawInputDevices as *const c_void,
        uiNumDevices as usize * core::mem::size_of::<RAWINPUTDEVICE>(),
        DWORD_ALIGN,
    )?;
    for i in 0..uiNumDevices as usize {
        let Device: RAWINPUTDEVICE = read(pRawInputDevices.add(i))?;

        if Device.dwFlags & RIDEV_INPUTSINK != 0 && Device.hwndTarget.is_null() {
            EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
            return Ok(FALSE as BOOL);
        }

        if Device.dwFlags & RIDEV_REMOVE != 0 && !Device.hwndTarget.is_null() {
            EngSetLastError(ERROR_INVALID_PARAMETER as ULONG);
            return Ok(FALSE as BOOL);
        }

        RawInputWarnUnimplementedFlags(&Device);

        if Device.usUsagePage != HID_USAGE_PAGE_GENERIC {
            continue;
        }

        let Index: usize = if Device.usUsage == HID_USAGE_GENERIC_MOUSE {
            0
        } else if Device.usUsage == HID_USAGE_GENERIC_KEYBOARD {
            1
        } else {
            continue;
        };

        if Device.dwFlags & RIDEV_REMOVE != 0 {
            core::ptr::write_bytes(global_pRawInputDevices.add(Index), 0, 1);
        } else {
            *global_pRawInputDevices.add(Index) = Device;
        }
    }

    RawInputEnabled = ((*global_pRawInputDevices).usUsage != 0
        || (*global_pRawInputDevices.add(1)).usUsage != 0) as BOOLEAN;

    Ok(TRUE as BOOL)
}
