//! win32ss/user/ntuser/rawinput.c — NtUserGetRawInputDeviceInfo / NtUserGetRawInputDeviceList
//! (upstream #9498). The upstream merge brought rawinput.c in after ntuser had gone Rust-only,
//! so until this port both syscalls were ntstubs.rs bodies returning 0: "no devices", with no
//! error set.
//!
//! User memory: the C wraps each probe AND its access in one `_SEH2_TRY`. Here the probes are
//! validate-only (`safe_user_io.rs`), so every access to a user buffer goes through the
//! fault-tolerant `crate::copy` primitives, and a fault takes the C's except path:
//! `SetLastNtError(STATUS_ACCESS_VIOLATION)` with the return value left as it was.
#![allow(non_snake_case)]
use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use core::ffi::c_void;

use crate::copy::{capture_from_user, publish_to_user};
use crate::error::{EngSetLastError, SetLastNtError};
use crate::input::{gpInputDeviceInfo, AcquireDeviceInfoListMutex, ReleaseDeviceInfoListMutex};
use crate::ntuser::{UserEnterShared, UserLeave};
use crate::safe_user_io::{SafeProbeForRead, SafeProbeForWrite};

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
