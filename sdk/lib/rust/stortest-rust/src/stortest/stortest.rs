use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static MOUNTMGR_DOS_DEVICE_NAME: [::core::ffi::c_ushort; 22] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_ushort; 22]>(
        *b"\\\0\\\0.\0\\\0M\0o\0u\0n\0t\0P\0o\0i\0n\0t\0M\0a\0n\0a\0g\0e\0r\0\0\0",
    )
};

pub static MARKER_SUBSTR: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"vibeos usb e2e marker\0")
};

pub static WRITE_CONTENT: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"vibeos wrote this through fastfat+usbstor\r\n\0",
    )
};

static mut found_pi_card: ::core::ffi::c_int = 0;

unsafe extern "C" fn mark(msg: *const i8) {
    OutputDebugStringA(msg);
}

#[no_mangle]
unsafe extern "C" fn markf_v(fmt: *const ::core::ffi::c_char, ap_list: *mut ::core::ffi::c_void) {
    let mut buf: [::core::ffi::c_char; 256] = [0; 256];
    // The trampoline in rust_varargs_shim.c did the va_start/va_end; `ap_list` is its `&ap`.
    let ap: va_list = *(ap_list as *mut va_list);
    _vsnprintf(buf.as_mut_ptr(), buf.len() - 1, fmt, ap as *mut ::core::ffi::c_char);
    buf[buf.len() - 1] = 0;
    OutputDebugStringA(buf.as_ptr());
}

unsafe extern "C" fn find_vibeusb() -> ::core::ffi::c_char {
    let mut root: [::core::ffi::c_char; 8] = [0; 8];
    let mut label: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut letter: ::core::ffi::c_char = 0;
    letter = 'C' as i32 as ::core::ffi::c_char;
    while letter as ::core::ffi::c_int <= 'Z' as i32 {
        sprintf(
            &raw mut root as *mut ::core::ffi::c_char,
            b"%c:\\\0" as *const u8 as *const ::core::ffi::c_char,
            letter as ::core::ffi::c_int,
        );
        label[0 as usize] = 0;
        if !(GetVolumeInformationA(
            &raw mut root as *mut ::core::ffi::c_char as LPCSTR,
            &raw mut label as LPSTR,
            ::core::mem::size_of::<[::core::ffi::c_char; (MAX_PATH) as _]>() as DWORD,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<CHAR>(),
            0 as DWORD,
        ) == 0)
        {
            found_pi_card = (_stricmp(
                &raw mut label as *mut ::core::ffi::c_char,
                b"VIBEOS_RPI4\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0) as ::core::ffi::c_int;
            if _stricmp(
                &raw mut label as *mut ::core::ffi::c_char,
                b"VIBEUSB\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0
                || found_pi_card != 0
            {
                return letter;
            }
        }
        letter += 1;
    }
    return 0;
}

unsafe extern "C" fn test_read_marker(letter: core::ffi::c_char) -> core::ffi::c_int {
    let mut path: [core::ffi::c_char; 32] = [0; 32];
    let mut buf: [core::ffi::c_char; 256] = [0; 256];
    let mut h: HANDLE = core::ptr::null_mut();
    let mut n: DWORD = 0;

    sprintf(
        core::ptr::addr_of_mut!(path) as *mut core::ffi::c_char,
        b"%c:\\MARKER.TXT\0".as_ptr() as *const core::ffi::c_char,
        letter as core::ffi::c_int,
    );
    h = CreateFileA(
        core::ptr::addr_of_mut!(path) as *mut core::ffi::c_char as LPCSTR,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if h == INVALID_HANDLE_VALUE {
        markf(
            b"[STORTEST] FAIL: open %s (error %lu)\n\0".as_ptr() as *const core::ffi::c_char,
            core::ptr::addr_of_mut!(path) as *mut core::ffi::c_char,
            GetLastError(),
        );
        return 0;
    }
    if ReadFile(
        h,
        core::ptr::addr_of_mut!(buf) as *mut core::ffi::c_char as PVOID,
        (core::mem::size_of::<[core::ffi::c_char; 256]>() - 1 as usize) as DWORD,
        core::ptr::addr_of_mut!(n),
        core::ptr::null_mut(),
    ) == 0
    {
        markf(
            b"[STORTEST] FAIL: read %s (error %lu)\n\0".as_ptr() as *const core::ffi::c_char,
            core::ptr::addr_of_mut!(path) as *mut core::ffi::c_char,
            GetLastError(),
        );
        CloseHandle(h);
        return 0;
    }
    CloseHandle(h);
    buf[n as usize] = 0 as core::ffi::c_char;
    if strstr(
        core::ptr::addr_of_mut!(buf) as *mut core::ffi::c_char,
        (MARKER_SUBSTR).as_ptr(),
    )
    .is_null()
    {
        markf(
            b"[STORTEST] FAIL: MARKER.TXT content mismatch (%lu bytes)\n\0".as_ptr() as *const core::ffi::c_char,
            n,
        );
        return 0;
    }
    markf(
        b"[STORTEST] read MARKER.TXT OK (%lu bytes)\n\0".as_ptr() as *const core::ffi::c_char,
        n,
    );
    return 1;
}

unsafe extern "C" fn test_write_readback(letter: ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut path: [::core::ffi::c_char; 32] = [0; 32];
    let mut buf: [::core::ffi::c_char; 128] = [0; 128];
    let mut h: HANDLE = core::ptr::null_mut::<::core::ffi::c_void>();
    let mut n: DWORD = 0 as DWORD;

    sprintf(
        core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char,
        b"%c:\\WRITETST.TXT\0".as_ptr() as *const ::core::ffi::c_char,
        letter as ::core::ffi::c_int,
    );
    h = CreateFileA(
        core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char as LPCSTR,
        GENERIC_WRITE as DWORD,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        CREATE_ALWAYS as DWORD,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        NULL,
    );
    if h == INVALID_HANDLE_VALUE {
        markf(
            b"[STORTEST] FAIL: create %s (error %lu)\n\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char,
            GetLastError(),
        );
        return 0 as ::core::ffi::c_int;
    }
    markf(
        b"[STORTEST] created %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char,
    );
    if WriteFile(
        h,
        (WRITE_CONTENT).as_ptr() as LPCVOID,
        strlen((WRITE_CONTENT).as_ptr()) as DWORD,
        core::ptr::addr_of_mut!(n),
        core::ptr::null_mut::<_OVERLAPPED>(),
    ) == 0
        || n as usize != strlen((WRITE_CONTENT).as_ptr())
    {
        markf(
            b"[STORTEST] FAIL: write %s (error %lu, wrote %lu)\n\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char,
            GetLastError(),
            n,
        );
        CloseHandle(h);
        return 0 as ::core::ffi::c_int;
    }
    markf(
        b"[STORTEST] wrote %lu bytes\n\0".as_ptr() as *const ::core::ffi::c_char,
        n,
    );
    if FlushFileBuffers(h) == 0 {
        markf(
            b"[STORTEST] FAIL: flush %s (error %lu)\n\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char,
            GetLastError(),
        );
        CloseHandle(h);
        return 0 as ::core::ffi::c_int;
    }
    markf(b"[STORTEST] flushed\n\0".as_ptr() as *const ::core::ffi::c_char);
    CloseHandle(h);
    markf(b"[STORTEST] closed\n\0".as_ptr() as *const ::core::ffi::c_char);

    /* The deferred-flush worker may hold a transient reference for ~1s after
     * the first cached write on a removable volume; retry briefly. */
    {
        let mut tries: ::core::ffi::c_int;
        h = INVALID_HANDLE_VALUE;
        tries = 0 as ::core::ffi::c_int;
        while tries < 10 as ::core::ffi::c_int && h == INVALID_HANDLE_VALUE {
            h = CreateFileA(
                core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char as LPCSTR,
                GENERIC_READ as DWORD,
                (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD,
                core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
                OPEN_EXISTING as DWORD,
                0 as DWORD,
                NULL,
            );
            if h == INVALID_HANDLE_VALUE {
                Sleep(500 as DWORD);
            }
            tries += 1;
        }
    }
    if h == INVALID_HANDLE_VALUE {
        markf(
            b"[STORTEST] FAIL: reopen %s (error %lu)\n\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char,
            GetLastError(),
        );
        return 0 as ::core::ffi::c_int;
    }
    n = 0 as DWORD;
    if ReadFile(
        h,
        core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char as PVOID,
        (core::mem::size_of::<[::core::ffi::c_char; 128]>() - 1 as usize) as DWORD,
        core::ptr::addr_of_mut!(n),
        core::ptr::null_mut::<_OVERLAPPED>(),
    ) == 0
    {
        markf(
            b"[STORTEST] FAIL: readback %s (error %lu)\n\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(path) as *mut ::core::ffi::c_char,
            GetLastError(),
        );
        CloseHandle(h);
        return 0 as ::core::ffi::c_int;
    }
    CloseHandle(h);
    buf[n as usize] = 0 as ::core::ffi::c_char;
    if n as usize != strlen((WRITE_CONTENT).as_ptr())
        || memcmp(
            core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            (WRITE_CONTENT).as_ptr() as *const ::core::ffi::c_void,
            n as usize,
        ) != 0 as ::core::ffi::c_int
    {
        markf(
            b"[STORTEST] FAIL: readback mismatch (%lu bytes)\n\0".as_ptr() as *const ::core::ffi::c_char,
            n,
        );
        return 0 as ::core::ffi::c_int;
    }
    markf(
        b"[STORTEST] write+readback OK (%lu bytes)\n\0".as_ptr() as *const ::core::ffi::c_char,
        n,
    );
    return 1 as ::core::ffi::c_int;
}

unsafe extern "C" fn request_drive_letters() {
    let mut mgr: HANDLE = core::ptr::null_mut();
    let mut in_: MOUNTMGR_MOUNT_POINT = core::mem::zeroed();
    let mut outbuf: [BYTE; 8192] = [0; 8192];
    let mut points: PMOUNTMGR_MOUNT_POINTS = core::ptr::null_mut();
    let mut bytes: DWORD = 0;
    let mut i: ULONG = 0;

    mgr = CreateFileW(
        (MOUNTMGR_DOS_DEVICE_NAME).as_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if mgr == INVALID_HANDLE_VALUE {
        markf(
            b"[STORTEST] open MountPointManager failed (error %lu)\n\0".as_ptr() as *const core::ffi::c_char,
            GetLastError(),
        );
        return;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(in_) as *mut core::ffi::c_void,
        core::mem::size_of::<MOUNTMGR_MOUNT_POINT>(),
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(outbuf) as *mut core::ffi::c_void,
        core::mem::size_of::<[BYTE; 8192]>(),
    );
    if DeviceIoControl(
        mgr,
        IOCTL_MOUNTMGR_QUERY_POINTS,
        core::ptr::addr_of_mut!(in_) as LPVOID,
        core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as DWORD,
        core::ptr::addr_of_mut!(outbuf) as *mut BYTE as LPVOID,
        core::mem::size_of::<[BYTE; 8192]>() as DWORD,
        core::ptr::addr_of_mut!(bytes),
        core::ptr::null_mut(),
    ) == 0
    {
        markf(
            b"[STORTEST] QUERY_POINTS failed (error %lu)\n\0".as_ptr() as *const core::ffi::c_char,
            GetLastError(),
        );
        CloseHandle(mgr);
        return;
    }

    points = core::ptr::addr_of_mut!(outbuf) as *mut BYTE as PMOUNTMGR_MOUNT_POINTS;
    markf(
        b"[STORTEST] mountmgr knows %lu mount points\n\0".as_ptr() as *const core::ffi::c_char,
        (*points).NumberOfMountPoints,
    );

    i = 0;
    while i < (*points).NumberOfMountPoints {
        let pt: PMOUNTMGR_MOUNT_POINT =
            (core::ptr::addr_of_mut!((*points).MountPoints) as *mut MOUNTMGR_MOUNT_POINT)
                .offset(i as isize);
        let mut devA: [core::ffi::c_char; 128] = [0; 128];
        let mut lnkA: [core::ffi::c_char; 128] = [0; 128];
        let dn: core::ffi::c_int = WideCharToMultiByte(
            CP_ACP,
            0,
            (core::ptr::addr_of_mut!(outbuf) as *mut BYTE).offset((*pt).DeviceNameOffset as isize)
                as LPCWSTR,
            (*pt).DeviceNameLength as core::ffi::c_int / 2,
            core::ptr::addr_of_mut!(devA) as LPSTR,
            (core::mem::size_of::<[core::ffi::c_char; 128]>()
                - 1 as usize) as core::ffi::c_int,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        let ln: core::ffi::c_int = WideCharToMultiByte(
            CP_ACP,
            0,
            (core::ptr::addr_of_mut!(outbuf) as *mut BYTE)
                .offset((*pt).SymbolicLinkNameOffset as isize) as LPCWSTR,
            (*pt).SymbolicLinkNameLength as core::ffi::c_int / 2,
            core::ptr::addr_of_mut!(lnkA) as LPSTR,
            (core::mem::size_of::<[core::ffi::c_char; 128]>()
                - 1 as usize) as core::ffi::c_int,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        devA[(if dn > 0 { dn } else { 0 }) as usize] =
            0;
        lnkA[(if ln > 0 { ln } else { 0 }) as usize] =
            0;
        markf(
            b"[STORTEST] point %lu: dev='%s' link='%s'\n\0".as_ptr() as *const core::ffi::c_char,
            i,
            core::ptr::addr_of_mut!(devA) as *mut core::ffi::c_char,
            core::ptr::addr_of_mut!(lnkA) as *mut core::ffi::c_char,
        );
        i = i.wrapping_add(1);
    }

    {
        let mut d: core::ffi::c_int = 0;
        while d < 4 {
            let mut name: [WCHAR; 64] = [
                '\\' as u16, 'D' as u16, 'e' as u16, 'v' as u16, 'i' as u16, 'c' as u16,
                'e' as u16, '\\' as u16, 'H' as u16, 'a' as u16, 'r' as u16, 'd' as u16,
                'd' as u16, 'i' as u16, 's' as u16, 'k' as u16, '0' as u16, '\\' as u16,
                'P' as u16, 'a' as u16, 'r' as u16, 't' as u16, 'i' as u16, 't' as u16,
                'i' as u16, 'o' as u16, 'n' as u16, '1' as u16,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let mut reqbuf: [BYTE; 256] = [0; 256];
            let target: PMOUNTMGR_DRIVE_LETTER_TARGET =
                core::ptr::addr_of_mut!(reqbuf) as *mut BYTE as PMOUNTMGR_DRIVE_LETTER_TARGET;
            let mut letterInfo: MOUNTMGR_DRIVE_LETTER_INFORMATION = core::mem::zeroed();
            let mut outBytes: DWORD = 0;
            let chars: core::ffi::c_int = 28;
            name[16 as usize] =
                (0x0030u16 + d as u16);

            (*target).DeviceNameLength =
                ((chars as usize) * core::mem::size_of::<WCHAR>()) as USHORT;
            memcpy(
                core::ptr::addr_of_mut!((*target).DeviceName) as *mut WCHAR as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(name) as *mut WCHAR as *const core::ffi::c_void,
                (*target).DeviceNameLength as usize,
            );

            ZeroMemory!(
                core::ptr::addr_of_mut!(letterInfo) as *mut core::ffi::c_void,
                core::mem::size_of::<MOUNTMGR_DRIVE_LETTER_INFORMATION>(),
            );
            if DeviceIoControl(
                mgr,
                IOCTL_MOUNTMGR_NEXT_DRIVE_LETTER,
                target as LPVOID,
                (core::mem::size_of::<USHORT>() + (*target).DeviceNameLength as usize) as DWORD,
                core::ptr::addr_of_mut!(letterInfo) as LPVOID,
                core::mem::size_of::<MOUNTMGR_DRIVE_LETTER_INFORMATION>() as DWORD,
                core::ptr::addr_of_mut!(outBytes),
                core::ptr::null_mut(),
            ) != 0
            {
                markf(
                    b"[STORTEST] next-letter Harddisk%d: assigned=%u letter=%c\n\0".as_ptr()
                        as *const core::ffi::c_char,
                    d,
                    letterInfo.DriveLetterWasAssigned as core::ffi::c_int,
                    if letterInfo.CurrentDriveLetter as core::ffi::c_int != 0 {
                        letterInfo.CurrentDriveLetter as core::ffi::c_int
                    } else {
                        b'?' as core::ffi::c_int
                    },
                );
            } else {
                markf(
                    b"[STORTEST] next-letter Harddisk%d failed (error %lu)\n\0".as_ptr()
                        as *const core::ffi::c_char,
                    d,
                    GetLastError(),
                );
            }
            d += 1;
        }
    }
    CloseHandle(mgr);
}
#[no_mangle]
pub unsafe extern "C" fn main() -> i32 {
    let mut letter: i8 = 0 as i8;
    let mut attempt: i32;
    let mut ok: i32 = 1;

    mark(b"[STORTEST] start\n\0".as_ptr() as *const i8);

    attempt = 0;
    while attempt < 60 && letter as i32 == 0 {
        if (attempt % 5) == 0 {
            request_drive_letters();
        }
        letter = find_vibeusb();
        if letter as i32 == 0 {
            Sleep(2000);
        }
        attempt += 1;
    }

    if letter as i32 == 0 {
        mark(b"[STORTEST] FAIL: no VIBEUSB volume found after 120s\n\0".as_ptr() as *const i8);
        return 1;
    }
    markf(
        b"[STORTEST] found %s at %c:\n\0".as_ptr() as *const i8,
        if found_pi_card != 0 {
            b"VIBEOS_RPI4 (Pi 4 boot card)\0".as_ptr() as *const i8
        } else {
            b"VIBEUSB\0".as_ptr() as *const i8
        },
        letter as i32,
    );

    if found_pi_card == 0 {
        ok &= test_read_marker(letter);
    }
    ok &= test_write_readback(letter);

    if ok != 0 {
        markf(b"[STORTEST] ALL OK drive %c:\n\0".as_ptr() as *const i8, letter as i32);
    } else {
        mark(b"[STORTEST] FAILURES (see above)\n\0".as_ptr() as *const i8);
    }
    return if ok != 0 { 0 } else { 1 };
}
