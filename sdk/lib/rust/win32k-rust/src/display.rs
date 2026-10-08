#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, IoGetDeviceProperty, RtlCompareMemory, RtlInitUnicodeString, ZwClose};
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::{STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER_1, STATUS_INVALID_PARAMETER_2, STATUS_UNSUCCESSFUL};
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

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


// tags.h: USERTAG_DISPLAYINFO 'idsU', GDITAG_DEVMODE 'vedG' (C multichar
// big-endian — reversed bytes for from_le_bytes).
const USERTAG_DISPLAYINFO: u32 = u32::from_le_bytes(*b"Udsi");
const GDITAG_DEVMODE: u32 = u32::from_le_bytes(*b"Gdev");

// winuser.h: #define HWND_BROADCAST ((HWND)0xffff)
const HWND_BROADCAST: HWND = 0xffff as HWND;

// winuser.h: #define ENUM_CURRENT_SETTINGS ((DWORD)-1), ENUM_REGISTRY_SETTINGS ((DWORD)-2)
// (not bindgen'd -- they're #define, not enum/const decls the header scanner sees).
const ENUM_CURRENT_SETTINGS: DWORD = 0xFFFFFFFFu32;
const ENUM_REGISTRY_SETTINGS: DWORD = 0xFFFFFFFEu32;

// Registry helpers — now Rust (misc/registry.c flipped to registry.rs);
// exported `extern "system"` (NTAPI = stdcall i386 / cdecl else). Engp*/Init*
// helpers are NTAPI/cdecl C in win32ss; SetLastNtError is fastcall.
extern "system" {
    fn RegWriteDWORD(hkey: HKEY, pwszValue: *const u16, dwValue: u32);
    fn RegOpenKey(pwszKeyName: *const u16, phkey: *mut HKEY) -> NTSTATUS;
    fn RegReadDWORD(hkey: HKEY, pwszValue: *const u16, pdwData: *mut DWORD) -> BOOL;
    fn RegQueryValue(hkey: HKEY, pwszValueName: *const u16, ulType: u32, pvData: *mut core::ffi::c_void, pcbValue: *mut u32) -> NTSTATUS;
    fn EngpGetPDEV(pustrDevice: PUNICODE_STRING) -> PPDEVOBJ;
    fn EngpRegisterGraphicsDevice(pustrDeviceName: PUNICODE_STRING, pustrDisplayDrivers: PUNICODE_STRING, pustrDescription: PUNICODE_STRING) -> PGRAPHICS_DEVICE;
    fn EngpFindGraphicsDevice(pustrDevice: PUNICODE_STRING, iDevNum: ULONG) -> PGRAPHICS_DEVICE;
    fn EngpUpdateMonitorDevices(pGraphicsDevice: PGRAPHICS_DEVICE) -> u32;
    fn SafeProbeForRead(Address: *mut core::ffi::c_void, Length: usize, Alignment: usize) -> i32;
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: usize, Alignment: usize) -> i32;
}

extern "system" {
    fn InitMetrics() -> u32;
}

// FASTCALL win32k export (win32k already imports it elsewhere per
// accelerator.rs's convention: stdcall-name-mangled fastcall on i386, C ABI
// otherwise).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

extern "C" {
    fn EngpUpdateGraphicsDeviceList() -> NTSTATUS;
    fn LDEVOBJ_bProbeAndCaptureDevmode(pGraphicsDevice: *mut core::ffi::c_void, pdm: PDEVMODEW, ppdmOut: *mut PDEVMODEW, bSearchClosest: u32) -> u32;
    fn InitSysParams() -> BOOL;
}

// RtlInitEmptyUnicodeString is a macro (no NTAPI kernel symbol) — set fields
// directly (matches registry.rs's rtl_init_empty_unicode_string).
#[inline(always)]
unsafe fn rtl_init_empty_unicode_string(s: *mut _UNICODE_STRING, buf: *mut u16, sz: USHORT) {
    (*s).Buffer = buf;
    (*s).Length = 0;
    (*s).MaximumLength = sz;
}

// Fault-protected replacement for the shape repeated at three _SEH2_TRY sites
// in display.c (NtUserEnumDisplayDevices ~400-414, NtUserEnumDisplaySettings
// ~636-655, NtUserChangeDisplaySettings ~988-1003): probe the UNICODE_STRING
// header, capture it, (optionally) reject an empty/NULL buffer the way
// NtUserEnumDisplaySettings's `if (!ustrDeviceUser.Length || !ustrDeviceUser.Buffer)
// ExRaiseStatus(STATUS_NO_MEMORY)` does, probe the string buffer, then do what
// RtlCopyUnicodeString does (copy min(dst.MaximumLength, src.Length) bytes,
// set dst.Length, NUL-terminate if room) -- but with the byte copy itself
// fault-protected via safe_copy_raw instead of calling RtlCopyUnicodeString
// directly on the raw (unprotected between check and read) user pointer.
//
// SafeProbeForRead/safe_copy_raw, not ntkrnl_sys::probe::probe_user_buffer or
// write_user: those read MmUserProbeAddress, a DATA export this driver can't
// link against (see the accelerator.rs comment on the same trap).
//
// Returns the real NTSTATUS on failure so a caller that needs it (site 5,
// NtUserChangeDisplaySettings, which does `SetLastNtError(_SEH2_GetExceptionCode())`)
// can use it; callers that always return a fixed status/bool on any failure
// (sites 1 and 3) just check `.is_err()`.
unsafe fn seh_probe_and_copy_unicode_string(
    dst: &mut _UNICODE_STRING,
    src: PUNICODE_STRING,
    require_nonempty: bool,
) -> Result<(), NTSTATUS> {
    let st = SafeProbeForRead(
        src as *mut core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
        1,
    );
    if st < 0 {
        return Err(st);
    }

    let mut hdr: _UNICODE_STRING = core::mem::zeroed();
    ntkrnl_sys::probe::safe_copy_raw(
        core::ptr::addr_of_mut!(hdr) as *mut core::ffi::c_void,
        src as *const core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
    )?;

    if require_nonempty && (hdr.Length == 0 || hdr.Buffer.is_null()) {
        // display.c's ExRaiseStatus(STATUS_NO_MEMORY), caught by the same
        // _SEH2_EXCEPT as a real probe fault.
        return Err(STATUS_NO_MEMORY);
    }

    let st2 = SafeProbeForRead(hdr.Buffer as *mut core::ffi::c_void, hdr.Length as usize, 1);
    if st2 < 0 {
        return Err(st2);
    }

    let copy_len = core::cmp::min(dst.MaximumLength, hdr.Length) as usize;
    ntkrnl_sys::probe::safe_copy_raw(
        dst.Buffer as *mut core::ffi::c_void,
        hdr.Buffer as *const core::ffi::c_void,
        copy_len,
    )?;
    dst.Length = copy_len as USHORT;
    if (dst.Length as usize) < dst.MaximumLength as usize {
        *dst.Buffer.add(copy_len / 2) = 0;
    }

    Ok(())
}

// strsafe.h RtlStringCbCopyW/RtlStringCbCatW are inline (no kernel symbol).
// cb is a *byte* count; bounded wide copy/cat keeping NUL termination.
#[inline(always)]
unsafe fn rtl_string_cb_copy_w(dst: *mut u16, cb: usize, src: *const u16) {
    let cch = cb / core::mem::size_of::<u16>();
    if cch == 0 { return; }
    let mut i = 0usize;
    while i + 1 < cch && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
}

#[inline(always)]
unsafe fn rtl_string_cb_cat_w(dst: *mut u16, cb: usize, src: *const u16) {
    let cch = cb / core::mem::size_of::<u16>();
    let mut d = 0usize;
    while d < cch && *dst.add(d) != 0 { d += 1; }
    let mut s = 0usize;
    while d + 1 < cch && *src.add(s) != 0 {
        *dst.add(d) = *src.add(s);
        d += 1; s += 1;
    }
    if d < cch { *dst.add(d) = 0; }
}

// libcnt/UCRT string helpers (undecorated cdecl).
extern "C" {
    fn swprintf(buf: *mut u16, fmt: *const u16, ...) -> i32;
    fn wcslen(s: *const u16) -> usize;
}

// minwindef.h MAKELONG.
#[inline(always)]
fn MAKELONG(low: u32, high: u32) -> LONG {
    (((low & 0xffff) | ((high & 0xffff) << 16)) as i32) as LONG
}

// gpsi->{aiSysMet,BitCount,Planes,...}: tagSERVERINFO ends with the anonymous
// PERUSERSERVERINFO member, made opaque by bindgen (`__bindgen_padding_0`).
// Reach it via a cast at that field's offset (matches winsta.rs/metric.rs).
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut gbBaseVideo: BOOL = FALSE as BOOL;
static mut gpFullscreen: PPROCESSINFO = core::ptr::null_mut();
static KEY_VIDEO: &[u16] = &[0x5C,0x52,0x65,0x67,0x69,0x73,0x74,0x72,0x79,0x5C,0x4D,0x61,0x63,0x68,0x69,0x6E,0x65,0x5C,0x48,0x41,0x52,0x44,0x57,0x41,0x52,0x45,0x5C,0x44,0x45,0x56,0x49,0x43,0x45,0x4D,0x41,0x50,0x5C,0x56,0x49,0x44,0x45,0x4F,0x00];

// NUL-terminated wide value-name literals (replace alloc-based encode_utf16).
macro_rules! wstr {
    ($s:literal) => {{
        const N: usize = $s.len() + 1;
        const W: [u16; N] = {
            let b = $s.as_bytes();
            let mut a = [0u16; N];
            let mut i = 0;
            while i < b.len() { a[i] = b[i] as u16; i += 1; }
            a
        };
        W.as_ptr()
    }};
}

#[no_mangle]
pub unsafe extern "C" fn RegWriteDisplaySettings(hkey: HKEY, pdm: PDEVMODEW) {

    RegWriteDWORD(hkey, wstr!("DefaultSettings.BitsPerPel"), (*pdm).dmBitsPerPel);
    RegWriteDWORD(hkey, wstr!("DefaultSettings.XResolution"), (*pdm).dmPelsWidth);
    RegWriteDWORD(hkey, wstr!("DefaultSettings.YResolution"), (*pdm).dmPelsHeight);
    RegWriteDWORD(hkey, wstr!("DefaultSettings.Flags"), (*pdm).__bindgen_anon_2.dmDisplayFlags);
    RegWriteDWORD(hkey, wstr!("DefaultSettings.VRefresh"), (*pdm).dmDisplayFrequency);
    RegWriteDWORD(hkey, wstr!("DefaultSettings.XPanning"), (*pdm).dmPanningWidth);
    RegWriteDWORD(hkey, wstr!("DefaultSettings.YPanning"), (*pdm).dmPanningHeight);
    RegWriteDWORD(hkey, wstr!("DefaultSettings.Orientation"), (*pdm).__bindgen_anon_1.__bindgen_anon_2.dmDisplayOrientation);
    RegWriteDWORD(hkey, wstr!("DefaultSettings.FixedOutput"), (*pdm).__bindgen_anon_1.__bindgen_anon_2.dmDisplayFixedOutput);
    RegWriteDWORD(hkey, wstr!("Attach.RelativeX"), (*pdm).__bindgen_anon_1.__bindgen_anon_2.dmPosition.x as u32);
    RegWriteDWORD(hkey, wstr!("Attach.RelativeY"), (*pdm).__bindgen_anon_1.__bindgen_anon_2.dmPosition.y as u32);
}

#[no_mangle]
pub unsafe extern "C" fn RegReadDisplaySettings(hkey: HKEY, pdm: PDEVMODEW) {
    let mut dwValue: DWORD = 0;

    RtlZeroMemory(pdm as *mut core::ffi::c_void, core::mem::size_of::<DEVMODEW>() as _);

    if RegReadDWORD(hkey, wstr!("DefaultSettings.BitsPerPel"), &mut dwValue) != 0 {
        (*pdm).dmBitsPerPel = dwValue;
        (*pdm).dmFields |= DM_BITSPERPEL;
    }
    if RegReadDWORD(hkey, wstr!("DefaultSettings.XResolution"), &mut dwValue) != 0 {
        (*pdm).dmPelsWidth = dwValue;
        (*pdm).dmFields |= DM_PELSWIDTH;
    }
    if RegReadDWORD(hkey, wstr!("DefaultSettings.YResolution"), &mut dwValue) != 0 {
        (*pdm).dmPelsHeight = dwValue;
        (*pdm).dmFields |= DM_PELSHEIGHT;
    }
    if RegReadDWORD(hkey, wstr!("DefaultSettings.Flags"), &mut dwValue) != 0 {
        (*pdm).__bindgen_anon_2.dmDisplayFlags = dwValue;
        (*pdm).dmFields |= DM_DISPLAYFLAGS;
    }
    if RegReadDWORD(hkey, wstr!("DefaultSettings.VRefresh"), &mut dwValue) != 0 {
        (*pdm).dmDisplayFrequency = dwValue;
        (*pdm).dmFields |= DM_DISPLAYFREQUENCY;
    }
    if RegReadDWORD(hkey, wstr!("DefaultSettings.XPanning"), &mut dwValue) != 0 {
        (*pdm).dmPanningWidth = dwValue;
        (*pdm).dmFields |= DM_PANNINGWIDTH;
    }
    if RegReadDWORD(hkey, wstr!("DefaultSettings.YPanning"), &mut dwValue) != 0 {
        (*pdm).dmPanningHeight = dwValue;
        (*pdm).dmFields |= DM_PANNINGHEIGHT;
    }
    if RegReadDWORD(hkey, wstr!("DefaultSettings.Orientation"), &mut dwValue) != 0 {
        (*pdm).__bindgen_anon_1.__bindgen_anon_2.dmDisplayOrientation = dwValue;
        (*pdm).dmFields |= DM_DISPLAYORIENTATION;
    }
    if RegReadDWORD(hkey, wstr!("DefaultSettings.FixedOutput"), &mut dwValue) != 0 {
        (*pdm).__bindgen_anon_1.__bindgen_anon_2.dmDisplayFixedOutput = dwValue;
        (*pdm).dmFields |= DM_DISPLAYFIXEDOUTPUT;
    }
    if RegReadDWORD(hkey, wstr!("Attach.RelativeX"), &mut dwValue) != 0 {
        (*pdm).__bindgen_anon_1.__bindgen_anon_2.dmPosition.x = dwValue as LONG;
        (*pdm).dmFields |= DM_POSITION;
    }
    if RegReadDWORD(hkey, wstr!("Attach.RelativeY"), &mut dwValue) != 0 {
        (*pdm).__bindgen_anon_1.__bindgen_anon_2.dmPosition.y = dwValue as LONG;
        (*pdm).dmFields |= DM_POSITION;
    }
}

#[no_mangle]
pub unsafe extern "system" fn InitDisplayDriver(
    pwszDeviceName: PWSTR,
    pwszRegKey: PWSTR,
) -> PGRAPHICS_DEVICE {
    let pGraphicsDevice: PGRAPHICS_DEVICE;
    let mut ustrDeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut ustrDisplayDrivers: _UNICODE_STRING = core::mem::zeroed();
    let mut ustrDescription: _UNICODE_STRING = core::mem::zeroed();
    let mut Status;
    let mut awcBuffer: [u16; 128] = [0; 128];
    let mut cbSize: u32;
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut dwVga: u32 = 0;

    TRACE!();

    /* Open the driver's registry key */
    Status = RegOpenKey(pwszRegKey, &mut hkey);
    if !NT_SUCCESS(Status) {
        ERR!("Failed to open registry key: %ls\n", pwszRegKey);
        return core::ptr::null_mut();
    }

    /* Query the diplay drivers */
    cbSize = (core::mem::size_of_val(&awcBuffer) - 10) as u32;
    Status = RegQueryValue(
        hkey,
        wstr!("InstalledDisplayDrivers"),
        REG_MULTI_SZ,
        awcBuffer.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbSize,
    );
    if !NT_SUCCESS(Status) {
        ERR!("Didn't find 'InstalledDisplayDrivers', status = 0x%lx\n", Status);
        ZwClose(hkey as _);
        return core::ptr::null_mut();
    }

    /* Initialize the _UNICODE_STRING */
    ustrDisplayDrivers.Buffer = awcBuffer.as_mut_ptr();
    ustrDisplayDrivers.MaximumLength = cbSize as u16;
    ustrDisplayDrivers.Length = cbSize as u16;

    /* Set Buffer for description and size of remaining buffer */
    ustrDescription.Buffer =
        awcBuffer.as_mut_ptr().add((cbSize as usize) / core::mem::size_of::<u16>());
    cbSize = (core::mem::size_of_val(&awcBuffer) as u32) - cbSize;

    /* Query the device string */
    Status = RegQueryValue(
        hkey,
        wstr!("Device Description"),
        REG_SZ,
        ustrDescription.Buffer as *mut core::ffi::c_void,
        &mut cbSize,
    );
    if NT_SUCCESS(Status) {
        ustrDescription.MaximumLength = cbSize as u16;
        ustrDescription.Length = cbSize as u16;
    } else {
        RtlInitUnicodeString(&mut ustrDescription as *mut _ as _, wstr!("<unknown>"));
    }

    /* Query if this is a VGA compatible driver */
    cbSize = core::mem::size_of::<u32>() as u32;
    Status = RegQueryValue(
        hkey,
        wstr!("VgaCompatible"),
        REG_DWORD,
        &mut dwVga as *mut u32 as *mut core::ffi::c_void,
        &mut cbSize,
    );
    if !NT_SUCCESS(Status) {
        dwVga = 0;
    }

    /* Close the registry key */
    ZwClose(hkey as _);

    /* Register the device with GDI */
    RtlInitUnicodeString(&mut ustrDeviceName as *mut _ as _, pwszDeviceName);
    pGraphicsDevice = EngpRegisterGraphicsDevice(
        &mut ustrDeviceName,
        &mut ustrDisplayDrivers,
        &mut ustrDescription,
    );
    if !pGraphicsDevice.is_null() && dwVga != 0 {
        (*pGraphicsDevice).StateFlags |= DISPLAY_DEVICE_VGA_COMPATIBLE;
    }

    pGraphicsDevice
}

#[no_mangle]
pub unsafe extern "system" fn InitVideo() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hkey: HKEY = core::ptr::null_mut();

    ERR!("InitVideo() called\n");

    Status = RegOpenKey(
        w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\GraphicsDrivers\\BaseVideo").as_ptr() as *const u16,
        &mut hkey,
    );
    if NT_SUCCESS(Status) {
        ZwClose(hkey as *mut core::ffi::c_void);
    }
    gbBaseVideo = if NT_SUCCESS(Status) { 1 } else { 0 };
    if gbBaseVideo != 0 {
        ERR!("InitVideo: VGA mode requested.\n");
    }

    Status = EngpUpdateGraphicsDeviceList();
    ERR!("InitVideo: EngpUpdateGraphicsDeviceList returned 0x%lx\n", Status);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    InitSysParams();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn UserRefreshDisplay(ppdev: PPDEVOBJ) {
    let ulResult: ULONG_PTR;

    if ppdev.is_null() {
        return;
    }

    PDEVOBJ_vReference(ppdev);

    ulResult = PDEVOBJ_bSwitchMode(ppdev, (*ppdev).pdmwDev) as ULONG_PTR;
    let _ = ulResult;

    InitMetrics();

    UserRedrawDesktop();

    PDEVOBJ_vRelease(ppdev);
}

#[no_mangle]
pub unsafe extern "system" fn UserEnumDisplayDevices(
    pustrDevice: PUNICODE_STRING,
    iDevNum: DWORD,
    pdispdev: PDISPLAY_DEVICEW,
    dwFlags: DWORD,
) -> NTSTATUS {
    let pGraphicsDevice: PGRAPHICS_DEVICE;
    let pdo: PDEVICE_OBJECT;
    let pHardwareId: PWCHAR;
    let mut cbSize: ULONG;
    let mut dwLength: ULONG = 0;
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if pustrDevice.is_null() {
        EngpUpdateGraphicsDeviceList();
    }

    pGraphicsDevice = EngpFindGraphicsDevice(pustrDevice, iDevNum);
    if pGraphicsDevice.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    if pustrDevice.is_null() {
        pdo = (*pGraphicsDevice).PhysDeviceHandle as _;
    } else {
        EngpUpdateMonitorDevices(pGraphicsDevice);
        if iDevNum >= (*pGraphicsDevice).dwMonCnt {
            return STATUS_UNSUCCESSFUL;
        }
        pdo = (*(*pGraphicsDevice).pvMonDev.offset(iDevNum as isize)).pdo as _;
    }

    Status = RegOpenKey(KEY_VIDEO.as_ptr(), &mut hkey);
    if !NT_SUCCESS(Status) {
        return STATUS_UNSUCCESSFUL;
    }

    cbSize = core::mem::size_of_val(&(*pdispdev).DeviceKey) as ULONG;
    RegQueryValue(
        hkey,
        core::ptr::addr_of_mut!((*pGraphicsDevice).szNtDeviceName) as PWCHAR,
        REG_SZ,
        (*pdispdev).DeviceKey.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbSize,
    );

    ZwClose(hkey as _);

    if pustrDevice.is_null() {
        rtl_string_cb_copy_w(
            (*pdispdev).DeviceName.as_mut_ptr(),
            core::mem::size_of_val(&(*pdispdev).DeviceName),
            core::ptr::addr_of!((*pGraphicsDevice).szWinDeviceName) as PWCHAR,
        );
        rtl_string_cb_copy_w(
            (*pdispdev).DeviceString.as_mut_ptr(),
            core::mem::size_of_val(&(*pdispdev).DeviceString),
            (*pGraphicsDevice).pwszDescription,
        );
        (*pdispdev).StateFlags = (*pGraphicsDevice).StateFlags;
    } else {
        swprintf(
            (*pdispdev).DeviceName.as_mut_ptr(),
            wstr!("%ws\\Monitor%u"),
            core::ptr::addr_of!((*pGraphicsDevice).szWinDeviceName) as PWCHAR,
            iDevNum,
        );
        if !pdo.is_null() {
            Status = IoGetDeviceProperty(
                pdo as _,
                DevicePropertyDeviceDescription,
                core::mem::size_of_val(&(*pdispdev).DeviceString) as ULONG,
                (*pdispdev).DeviceString.as_mut_ptr() as *mut core::ffi::c_void,
                &mut dwLength,
            );
            if !NT_SUCCESS(Status) {
                (*pdispdev).DeviceString[0] = 0;
            }
        }
        (*pdispdev).StateFlags = (*(*pGraphicsDevice).pvMonDev.offset(iDevNum as isize)).flag;
    }
    (*pdispdev).DeviceID[0] = 0;

    if !pdo.is_null() {
        Status = IoGetDeviceProperty(
            pdo as _,
            DevicePropertyHardwareID,
            0,
            core::ptr::null_mut(),
            &mut dwLength,
        );

        if Status == STATUS_BUFFER_TOO_SMALL {
            pHardwareId = ExAllocatePoolWithTag(PagedPool, dwLength as _, USERTAG_DISPLAYINFO) as PWCHAR;
            if pHardwareId.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            Status = IoGetDeviceProperty(
                pdo as _,
                DevicePropertyHardwareID,
                dwLength,
                pHardwareId as *mut core::ffi::c_void,
                &mut dwLength,
            );

            if NT_SUCCESS(Status) {
                rtl_string_cb_copy_w(
                    (*pdispdev).DeviceID.as_mut_ptr(),
                    core::mem::size_of_val(&(*pdispdev).DeviceID),
                    pHardwareId,
                );

                if !pustrDevice.is_null() {
                    rtl_string_cb_cat_w(
                        (*pdispdev).DeviceID.as_mut_ptr(),
                        core::mem::size_of_val(&(*pdispdev).DeviceID),
                        wstr!("\\"),
                    );

                    dwLength = (wcslen((*pdispdev).DeviceID.as_ptr()) + 1) as ULONG;
                    Status = IoGetDeviceProperty(
                        pdo as _,
                        DevicePropertyDriverKeyName,
                        ((*pdispdev).DeviceID.len() as ULONG - dwLength)
                            * core::mem::size_of::<u16>() as ULONG,
                        (*pdispdev).DeviceID.as_mut_ptr().offset(dwLength as isize - 1) as *mut core::ffi::c_void,
                        &mut dwLength,
                    );
                }
            }

            ExFreePoolWithTag(pHardwareId as *mut core::ffi::c_void, USERTAG_DISPLAYINFO);
        }
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEnumDisplayDevices(
    mut pustrDevice: PUNICODE_STRING,
    iDevNum: DWORD,
    pDisplayDevice: PDISPLAY_DEVICEW,
    dwFlags: DWORD,
) -> BOOL {
    let mut ustrDevice: _UNICODE_STRING = core::mem::zeroed();
    let mut awcDevice: [WCHAR; CCHDEVICENAME as usize] = [0; CCHDEVICENAME as usize];
    let mut dispdev: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut Status: NTSTATUS;

    dispdev.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;

    if !pustrDevice.is_null() {
        rtl_init_empty_unicode_string(
            &mut ustrDevice,
            awcDevice.as_mut_ptr(),
            core::mem::size_of_val(&awcDevice) as USHORT,
        );

        // display.c ~400-414 (_SEH2_TRY/_SEH2_EXCEPT, DROPPED by the cohab
        // flip): probes pustrDevice, pustrDevice->Buffer, then
        // RtlCopyUnicodeString, all under one guard. The handler is the
        // UNUSUAL shape flagged for this file: the plain
        // `return _SEH2_GetExceptionCode()` is commented out in the C in
        // favor of `_SEH2_YIELD(return NT_SUCCESS(_SEH2_GetExceptionCode()))`.
        // NtUserEnumDisplayDevices returns BOOL, and an exception code always
        // carries the error-facility bits, so NT_SUCCESS() of it is
        // unconditionally FALSE -- the handler is "return FALSE", just
        // spelled through NT_SUCCESS. Reproduced as a plain boolean
        // early-return (not "fixed" into propagating a status).
        if seh_probe_and_copy_unicode_string(&mut ustrDevice, pustrDevice, false).is_err() {
            return FALSE as BOOL;
        }

        if ustrDevice.Length > 0 {
            pustrDevice = &mut ustrDevice;
        } else {
            pustrDevice = core::ptr::null_mut();
        }
    }

    UserEnterShared();

    // nolint: raw-user-access (both arms above re-point pustrDevice: the captured copy, or NULL)
    Status = UserEnumDisplayDevices(pustrDevice, iDevNum, &mut dispdev, dwFlags); // nolint: raw-user-access (see above)

    UserLeave();

    if NT_SUCCESS(Status) {
        // display.c ~435-455 (_SEH2_TRY/_SEH2_EXCEPT, DROPPED): probe-write
        // &pDisplayDevice->cb, read it, clamp+probe-write the whole output
        // buffer, RtlCopyMemory dispdev into it -- all one guard. Ordinary
        // assign-and-fall-through shape: `Status = _SEH2_GetExceptionCode();`
        // then execution falls out of the SEH block to the TRACE + return
        // below, unlike sites 1/2 (this file's other two are early-returns).
        'seh_out: {
            let wst = SafeProbeForWrite(
                core::ptr::addr_of_mut!((*pDisplayDevice).cb) as *mut core::ffi::c_void,
                core::mem::size_of::<DWORD>(),
                1,
            );
            if wst < 0 {
                Status = wst;
                break 'seh_out;
            }

            // Bare read of pDisplayDevice->cb in the C -- it sits inside the
            // same _SEH2_TRY as the probe above, so a fault here (not just an
            // out-of-range pointer) has to be caught too.
            let mut cb: DWORD = 0;
            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                core::ptr::addr_of_mut!(cb) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*pDisplayDevice).cb) as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>(),
            ) {
                Status = st;
                break 'seh_out;
            }

            if cb != 0 {
                cb = core::cmp::min(cb, core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD);

                // pDisplayDevice->cb = min(...) in the C -- also a bare store
                // under the guard.
                if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                    core::ptr::addr_of_mut!((*pDisplayDevice).cb) as *mut core::ffi::c_void,
                    core::ptr::addr_of!(cb) as *const core::ffi::c_void,
                    core::mem::size_of::<DWORD>(),
                ) {
                    Status = st;
                    break 'seh_out;
                }

                let wst2 = SafeProbeForWrite(pDisplayDevice as *mut core::ffi::c_void, cb as usize, 1);
                if wst2 < 0 {
                    Status = wst2;
                    break 'seh_out;
                }

                if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                    pDisplayDevice as *mut core::ffi::c_void,
                    &dispdev as *const _ as *const core::ffi::c_void,
                    cb as usize,
                ) {
                    Status = st;
                    break 'seh_out;
                }
            }
        }
    }

    NT_SUCCESS(Status) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn UserEnumCurrentDisplaySettings(
    pustrDevice: PUNICODE_STRING,
    ppdm: *mut PDEVMODEW,
) -> NTSTATUS {
    let ppdev: PPDEVOBJ = EngpGetPDEV(pustrDevice);
    if ppdev.is_null() {
        ERR!("No PDEV found!\n");
        return STATUS_INVALID_PARAMETER_1;
    }

    core::ptr::write_volatile(ppdm, (*ppdev).pdmwDev);
    PDEVOBJ_vRelease(ppdev);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn UserEnumDisplaySettings(
    pustrDevice: PUNICODE_STRING,
    iModeNum: DWORD,
    ppdm: *mut LPDEVMODEW,
    dwFlags: DWORD,
) -> NTSTATUS {
    let pGraphicsDevice: PGRAPHICS_DEVICE;
    let mut pdmentry: PDEVMODEENTRY;
    let mut i: ULONG;
    let mut iFoundMode: ULONG;
    let ppdev: PPDEVOBJ;

    pGraphicsDevice = EngpFindGraphicsDevice(pustrDevice, 0);
    ppdev = EngpGetPDEV(pustrDevice);

    if pGraphicsDevice.is_null() || ppdev.is_null() {
        return STATUS_INVALID_PARAMETER_1;
    }

    PDEVOBJ_vRefreshModeList(ppdev);
    PDEVOBJ_vRelease(ppdev);

    iFoundMode = 0;
    i = 0;
    while i < (*pGraphicsDevice).cDevModes {
        pdmentry = core::ptr::addr_of_mut!(*(*pGraphicsDevice).pDevModeList.offset(i as isize));

        if iFoundMode == iModeNum {
            core::ptr::write_volatile(ppdm, (*pdmentry).pdm);
            return STATUS_SUCCESS;
        }

        iFoundMode += 1;
        i += 1;
    }

    STATUS_INVALID_PARAMETER_2
}

#[no_mangle]
pub unsafe extern "system" fn UserOpenDisplaySettingsKey(
    phkey: PHKEY,
    pustrDevice: PUNICODE_STRING,
    bGlobal: BOOL,
) -> NTSTATUS {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut dispdev: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = UserEnumDisplayDevices(pustrDevice, 0, &mut dispdev, 0);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if bGlobal != 0 {
        // FIXME: Need to fix the registry key somehow
    }

    Status = RegOpenKey(dispdev.DeviceKey.as_mut_ptr(), &mut hkey);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    core::ptr::write_volatile(phkey, hkey);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn UserEnumRegistryDisplaySettings(
    pustrDevice: PUNICODE_STRING,
    pdm: LPDEVMODEW,
) -> NTSTATUS {
    let mut hkey: HKEY = core::ptr::null_mut();
    let Status: NTSTATUS = UserOpenDisplaySettingsKey(&mut hkey, pustrDevice, 0);
    if NT_SUCCESS(Status) {
        RegReadDisplaySettings(hkey, pdm);
        ZwClose(hkey as _);
        return STATUS_SUCCESS;
    }
    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEnumDisplaySettings(
    pustrDevice: PUNICODE_STRING,
    iModeNum: u32,
    lpDevMode: LPDEVMODEW,
    dwFlags: u32,
) -> i32 {
    let mut ustrDevice: _UNICODE_STRING = core::mem::zeroed();
    let mut awcDevice: [u16; CCHDEVICENAME as usize] = [0; CCHDEVICENAME as usize];
    let mut Status: i32;
    let mut cbSize: u32 = 0;
    let mut cbExtra: u32 = 0;
    let mut pustrDevice = pustrDevice;
    let mut dmReg: DEVMODEW = core::mem::zeroed();
    let mut pdm: LPDEVMODEW = core::ptr::null_mut();

    Status = SafeProbeForRead(
        lpDevMode as *mut core::ffi::c_void,
        core::mem::size_of::<DEVMODEW>(),
        core::mem::size_of::<u8>(),
    );
    if NT_SUCCESS(Status) {
        Status = SafeProbeForWrite(
            lpDevMode as *mut core::ffi::c_void,
            (cbSize + cbExtra) as usize,
            core::mem::size_of::<u8>(),
        );
    }
    if !NT_SUCCESS(Status) {
        return Status;
    }
    if NT_SUCCESS(Status) {
        // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND both loads in
        // one _SEH2_TRY, so they come across through the fault-tolerant copy.
        let mut dmSize: u16 = 0;
        let mut dmDriverExtra: u16 = 0;
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(dmSize) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*lpDevMode).dmSize) as *const core::ffi::c_void,
            core::mem::size_of::<u16>(),
        ) || !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(dmDriverExtra) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*lpDevMode).dmDriverExtra) as *const core::ffi::c_void,
            core::mem::size_of::<u16>(),
        ) {
            return STATUS_ACCESS_VIOLATION;
        }
        cbSize = dmSize as u32;
        cbExtra = dmDriverExtra as u32;
    }

    if cbSize != core::mem::size_of::<DEVMODEW>() as u32 {
        return STATUS_BUFFER_TOO_SMALL;
    }

    if !pustrDevice.is_null() {
        rtl_init_empty_unicode_string(
            &mut ustrDevice,
            awcDevice.as_mut_ptr(),
            core::mem::size_of_val(&awcDevice) as u16,
        );

        // display.c ~636-655 (_SEH2_TRY/_SEH2_EXCEPT, DROPPED by the cohab
        // flip): ProbeForReadUnicodeString (probe + capture the header),
        // `if (!ustrDeviceUser.Length || !ustrDeviceUser.Buffer)
        // ExRaiseStatus(STATUS_NO_MEMORY)`, ProbeForRead the buffer, then
        // RtlCopyUnicodeString -- all one guard. Fixed status-return shape:
        // ANY failure in the guarded region (a real probe fault OR the
        // explicit ExRaiseStatus) returns the same STATUS_INVALID_PARAMETER_1,
        // not the actual raised/faulting code.
        if seh_probe_and_copy_unicode_string(&mut ustrDevice, pustrDevice, true).is_err() {
            return STATUS_INVALID_PARAMETER_1;
        }

        pustrDevice = &mut ustrDevice;
    }

    /* Acquire global USER lock */
    UserEnterShared();

    if iModeNum == ENUM_REGISTRY_SETTINGS {
        /* Get the registry settings */
        Status = UserEnumRegistryDisplaySettings(pustrDevice, &mut dmReg);
        pdm = &mut dmReg;
        (*pdm).dmSize = core::mem::size_of::<DEVMODEW>() as u16;
    } else if iModeNum == ENUM_CURRENT_SETTINGS {
        /* Get the current settings */
        Status = UserEnumCurrentDisplaySettings(pustrDevice, &mut pdm);
    } else {
        /* Get specified settings */
        Status = UserEnumDisplaySettings(pustrDevice, iModeNum, &mut pdm, dwFlags);
    }

    /* Release lock */
    UserLeave();

    /* Did we succeed? */
    if NT_SUCCESS(Status) {
        // display.c ~688-705 (_SEH2_TRY/_SEH2_EXCEPT, DROPPED): RtlCopyMemory
        // of the (kernel-owned) pdm into the caller's lpDevMode, plus the
        // driver-extra tail -- both bare stores with no ProbeFor* at all (the
        // top-of-function SafeProbeForRead/SafeProbeForWrite already vetted
        // lpDevMode/cbSize+cbExtra), so safe_copy_raw is the right tool, not
        // an extra probe upstream never had. Ordinary assign-and-fall-through
        // shape: `Status = _SEH2_GetExceptionCode();`, then execution falls
        // out of the SEH block to the `return Status;` below.
        'seh_out: {
            let copy_len = core::cmp::min(cbSize, (*pdm).dmSize as u32) as usize;
            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                lpDevMode as *mut core::ffi::c_void,
                pdm as *const core::ffi::c_void,
                copy_len,
            ) {
                Status = st;
                break 'seh_out;
            }

            if cbExtra > 0 && (*pdm).dmDriverExtra > 0 {
                let extra_len = core::cmp::min(cbExtra, (*pdm).dmDriverExtra as u32) as usize;
                if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                    (lpDevMode as *mut u8).add(cbSize as usize) as *mut core::ffi::c_void,
                    (pdm as *const u8).add((*pdm).dmSize as usize) as *const core::ffi::c_void,
                    extra_len,
                ) {
                    Status = st;
                    break 'seh_out;
                }
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn UserUpdateFullscreen(flags: DWORD) {
    if flags & CDS_FULLSCREEN != 0 {
        gpFullscreen = (*gptiCurrent).ppi;
    } else {
        gpFullscreen = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn UserChangeDisplaySettings(
    pustrDevice: PUNICODE_STRING,
    pdm: LPDEVMODEW,
    flags: DWORD,
    lParam: LPVOID,
) -> LONG {

    let mut dm: DEVMODEW = core::mem::zeroed();
    let mut lResult: LONG = DISP_CHANGE_SUCCESSFUL as LONG;
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let ppdev: PPDEVOBJ;
    let OrigBC: u16;
    let mut newDevMode: PDEVMODEW = core::ptr::null_mut();

    /* If no DEVMODE is given, use registry settings */
    if pdm.is_null() {
        Status = UserEnumRegistryDisplaySettings(pustrDevice, &mut dm);
        if !NT_SUCCESS(Status) {
            return DISP_CHANGE_BADPARAM as LONG;
        }
    } else if ((*pdm).dmSize as usize)
        < (core::mem::offset_of!(DEVMODEW, dmFields))
    {
        return DISP_CHANGE_BADMODE as LONG;
    } else {
        dm = *pdm;
    }

    /* Save original bit count */
    OrigBC = (*gpsi_per_user()).BitCount;

    /* Check params */
    if (dm.dmFields & (DM_PELSWIDTH | DM_PELSHEIGHT)) != (DM_PELSWIDTH | DM_PELSHEIGHT) {
        return DISP_CHANGE_BADMODE as LONG;
    }

    /* Get the PDEV */
    ppdev = EngpGetPDEV(pustrDevice);
    if ppdev.is_null() {
        return DISP_CHANGE_BADPARAM as LONG;
    }

    /* Fixup values */
    if dm.dmBitsPerPel == 0 || (dm.dmFields & DM_BITSPERPEL) == 0 {
        dm.dmBitsPerPel = (*(*ppdev).pdmwDev).dmBitsPerPel;
        dm.dmFields |= DM_BITSPERPEL;
    }

    if (dm.dmFields & DM_DISPLAYFREQUENCY) != 0 && dm.dmDisplayFrequency == 0 {
        dm.dmDisplayFrequency = (*(*ppdev).pdmwDev).dmDisplayFrequency;
    }

    'leave: {
        /* Look for the requested DEVMODE */
        if LDEVOBJ_bProbeAndCaptureDevmode(
            (*ppdev).pGraphicsDevice as *mut core::ffi::c_void,
            &mut dm,
            &mut newDevMode,
            (0u8) as _,
        ) == 0
        {
            lResult = DISP_CHANGE_BADMODE as LONG;
            break 'leave;
        } else if (flags & CDS_TEST) != 0 {
            lResult = DISP_CHANGE_SUCCESSFUL as LONG;
            break 'leave;
        }

        /* Shall we update the registry? */
        if (flags & CDS_UPDATEREGISTRY) != 0 {
            Status = UserOpenDisplaySettingsKey(&mut hkey, pustrDevice, (flags & CDS_GLOBAL) as BOOL);
            if NT_SUCCESS(Status) {
                RegWriteDisplaySettings(hkey, newDevMode);
                ZwClose(hkey as _);
            } else {
                lResult = DISP_CHANGE_NOTUPDATED as LONG;
            }
        }

        /* Check if DEVMODE matches the current mode */
        if (*newDevMode).dmSize == (*(*ppdev).pdmwDev).dmSize
            && RtlCompareMemory(
                newDevMode as *const core::ffi::c_void,
                (*ppdev).pdmwDev as *const core::ffi::c_void,
                (*newDevMode).dmSize as _,
            ) as usize == (*newDevMode).dmSize as usize
            && (flags & CDS_RESET) == 0
        {
            break 'leave;
        }

        /* Shall we apply the settings? */
        if (flags & CDS_NORESET) == 0 {
            let mut ulResult: ULONG_PTR;
            let mut pvOldCursor: *mut core::ffi::c_void;
            let mut tmw: TEXTMETRICW = core::mem::zeroed();

            /* Remove mouse pointer */
            pvOldCursor = UserSetCursor(core::ptr::null_mut(), TRUE as BOOL) as *mut core::ffi::c_void;

            /* Do the mode switch */
            ulResult = PDEVOBJ_bSwitchMode(ppdev, newDevMode) as ULONG_PTR;

            /* Restore mouse pointer, no hooks called */
            pvOldCursor = UserSetCursor(pvOldCursor as _, TRUE as BOOL) as *mut core::ffi::c_void;
            let _ = pvOldCursor;

            /* Check for success or failure */
            if ulResult == 0 {
                if (flags & CDS_UPDATEREGISTRY) != 0 && lResult != DISP_CHANGE_NOTUPDATED as LONG {
                    lResult = DISP_CHANGE_RESTART as LONG;
                } else {
                    lResult = DISP_CHANGE_FAILED as LONG;
                }
            } else {
                lResult = DISP_CHANGE_SUCCESSFUL as LONG;
                ExFreePoolWithTag((*ppdev).pdmwDev as LPVOID, GDITAG_DEVMODE);
                (*ppdev).pdmwDev = newDevMode;

                UserUpdateFullscreen(flags);

                InitMetrics();

                UserUpdateMonitorSize(ppdev as HDEV);

                let pusi = gpsi_per_user();
                (*pusi).dmLogPixels = (*ppdev).gdiinfo.ulLogPixelsY as _;
                (*pusi).Planes = (*ppdev).gdiinfo.cPlanes as _;
                (*pusi).BitsPixel = (*ppdev).gdiinfo.cBitsPixel as _;
                (*pusi).BitCount = ((*pusi).Planes * (*pusi).BitsPixel) as u16;
                (*pusi).aiSysMet[SM_CXSCREEN as usize] = (*ppdev).gdiinfo.ulHorzRes as i32;
                (*pusi).aiSysMet[SM_CYSCREEN as usize] = (*ppdev).gdiinfo.ulVertRes as i32;
                if (*ppdev).gdiinfo.flRaster & RC_PALETTE != 0 {
                    (*pusi).PUSIFlags |= PUSIF_PALETTEDISPLAY;
                } else {
                    (*pusi).PUSIFlags &= !PUSIF_PALETTEDISPLAY;
                }
                (*pusi).cxSysFontChar =
                    IntGetCharDimensions(hSystemBM, &mut tmw, core::ptr::addr_of_mut!((*pusi).cySysFontChar) as *mut DWORD) as _;
                (*pusi).tmSysFont = tmw;
            }

            /* Remove all cursor clipping */
            UserClipCursor(core::ptr::null_mut());

            /* Send WM_DISPLAYCHANGE to all toplevel windows */
            co_IntSendMessageTimeout(
                HWND_BROADCAST,
                WM_DISPLAYCHANGE,
                (*gpsi_per_user()).BitCount as WPARAM,
                MAKELONG(
                    (*gpsi_per_user()).aiSysMet[SM_CXSCREEN as usize] as u32,
                    (*gpsi_per_user()).aiSysMet[SM_CYSCREEN as usize] as u32,
                ) as LPARAM,
                SMTO_NORMAL,
                100,
                &mut ulResult,
            );

            /* Not full screen and different bit count, send messages */
            if (flags & CDS_FULLSCREEN) == 0 && (*gpsi_per_user()).BitCount != OrigBC {
                UserSendNotifyMessage(HWND_BROADCAST, WM_SETTINGCHANGE, 0, 0);
                UserSendNotifyMessage(HWND_BROADCAST, WM_SYSCOLORCHANGE, 0, 0);
            }

            UserRedrawDesktop();
        }
    }

    if !newDevMode.is_null() && newDevMode != (*ppdev).pdmwDev {
        ExFreePoolWithTag(newDevMode as LPVOID, GDITAG_DEVMODE);
    }

    /* Release the PDEV */
    PDEVOBJ_vRelease(ppdev);

    lResult
}

#[no_mangle]
pub unsafe extern "C" fn UserDisplayNotifyShutdown(ppiCurrent: PPROCESSINFO) {
    if ppiCurrent == gpFullscreen {
        UserChangeDisplaySettings(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if !gpFullscreen.is_null() {
            ERR!("Failed to restore display mode!\n");
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserChangeDisplaySettings(
    mut pustrDevice: PUNICODE_STRING,
    mut lpDevMode: LPDEVMODEW,
    dwflags: DWORD,
    lParam: LPVOID,
) -> LONG {

    let mut awcDevice: [u16; CCHDEVICENAME as usize] = [0; CCHDEVICENAME as usize];
    let mut ustrDevice: _UNICODE_STRING = core::mem::zeroed();
    let mut dmLocal: DEVMODEW = core::mem::zeroed();
    let lRet: LONG;

    /* Check arguments */
    if dwflags != CDS_VIDEOPARAMETERS && lParam != core::ptr::null_mut() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return DISP_CHANGE_BADPARAM;
    }

    /* Check flags */
    if (dwflags & (CDS_GLOBAL | CDS_NORESET)) != 0 && (dwflags & CDS_UPDATEREGISTRY) == 0 {
        return DISP_CHANGE_BADFLAGS;
    }

    if (dwflags & CDS_RESET) != 0 && (dwflags & CDS_NORESET) != 0 {
        return DISP_CHANGE_BADFLAGS;
    }

    /* Copy the device name */
    if pustrDevice != core::ptr::null_mut() {
        /* Initialize destination string */
        rtl_init_empty_unicode_string(
            &mut ustrDevice,
            awcDevice.as_mut_ptr(),
            core::mem::size_of_val(&awcDevice) as u16,
        );

        // display.c ~988-1003 (_SEH2_TRY/_SEH2_EXCEPT, DROPPED by the cohab
        // flip): probes pustrDevice + pustrDevice->Buffer, then
        // RtlCopyUnicodeString, all under one guard. Early-return-with-side-
        // effect shape: `SetLastNtError(_SEH2_GetExceptionCode());
        // _SEH2_YIELD(return DISP_CHANGE_BADPARAM);` -- unlike display.rs's
        // fixed-status sites, this one DOES propagate the real exception
        // code, just to SetLastNtError rather than as the return value.
        match seh_probe_and_copy_unicode_string(&mut ustrDevice, pustrDevice, false) {
            Ok(()) => {}
            Err(st) => {
                SetLastNtError(st);
                return DISP_CHANGE_BADPARAM;
            }
        }

        pustrDevice = &mut ustrDevice;
    }

    /* Copy devmode */
    if lpDevMode != core::ptr::null_mut() {
        // display.c ~1011-1029 (_SEH2_TRY/_SEH2_EXCEPT, DROPPED): probe
        // sizeof(dmLocal.dmSize), read lpDevMode->dmSize (bare, under the
        // same guard), clamp, probe the full clamped size, then
        // RtlCopyMemory -- all one guard. Same early-return-with-side-effect
        // shape as the pustrDevice site just above.
        let sizeOfLocal = core::mem::size_of::<DEVMODEW>() as u16;

        let st = SafeProbeForRead(
            lpDevMode as *mut core::ffi::c_void,
            core::mem::size_of_val(&dmLocal.dmSize),
            1,
        );
        if st < 0 {
            SetLastNtError(st);
            return DISP_CHANGE_BADPARAM;
        }

        let mut dmSizeField: u16 = 0;
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            core::ptr::addr_of_mut!(dmSizeField) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*lpDevMode).dmSize) as *const core::ffi::c_void,
            core::mem::size_of::<u16>(),
        ) {
            SetLastNtError(st);
            return DISP_CHANGE_BADPARAM;
        }

        dmLocal.dmSize = core::cmp::min(sizeOfLocal, dmSizeField);

        let st2 = SafeProbeForRead(
            lpDevMode as *mut core::ffi::c_void,
            dmLocal.dmSize as usize,
            1,
        );
        if st2 < 0 {
            SetLastNtError(st2);
            return DISP_CHANGE_BADPARAM;
        }

        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            &mut dmLocal as *mut DEVMODEW as *mut core::ffi::c_void,
            lpDevMode as *const core::ffi::c_void,
            dmLocal.dmSize as usize,
        ) {
            SetLastNtError(st);
            return DISP_CHANGE_BADPARAM;
        }

        /* Check for extra parameters */
        if dmLocal.dmDriverExtra > 0 {
            /* FIXME: TODO */
            dmLocal.dmDriverExtra = 0;
        }

        /* Use the local structure */
        lpDevMode = &mut dmLocal;
    }

    // FIXME: Copy videoparameters

    /* Acquire global USER lock */
    UserEnterExclusive();

    /* Call internal function */
    lRet = UserChangeDisplaySettings(pustrDevice, lpDevMode, dwflags, core::ptr::null_mut());

    /* Release lock */
    UserLeave();

    lRet
}
