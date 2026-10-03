use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn grab_profile(handle: HPROFILE) -> *mut profile;
    fn release_profile(profile: *mut profile);
    fn cmsGetTagCount(cmsprofile: *mut core::ffi::c_void) -> i32;
    fn cmsGetTagSignature(cmsprofile: *mut core::ffi::c_void, index: i32) -> u32;
    fn TRACE(fmt: *const i8, arg1: HPROFILE, arg2: u32, arg3: *mut u32);
    fn get_profile_header(profile: *mut profile, header: *mut PROFILEHEADER);
    fn set_tag_data( profile: *mut profile, type_: TAGTYPE, offset: u32, buffer: *mut core::ffi::c_void, size: *mut u32, ) -> i32; // BOOL fn WARN(format: *const i8, ...);
    fn debugstr_w(str: *const u16) -> *const i8;
    fn debugstr_a(s: *const i8) -> *const i8;
}
extern "C" {
    fn get_tag_data(profile: *mut profile, type_: TAGTYPE, offset: DWORD, buffer: PVOID, size: PDWORD) -> BOOL;
    fn cmsTagLinkedTo(profile: cmsHPROFILE, type_: TAGTYPE) -> i32;
    fn cmsIsTag(hProfile: *mut core::ffi::c_void, tag: u32) -> i32;
    fn set_profile_header(profile: *mut profile, header: PPROFILEHEADER);
    fn close_profile(profile: HPROFILE) -> BOOL;
    fn FIXME(format: *const i8, ...);
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn GetColorDirectoryW(machine: *const u16, buffer: *mut u16, size: *mut u32) -> i32;
    unsafe fn basename(path: PCWSTR, base: LPWSTR) ;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn GetStandardColorSpaceProfileW(machine: *const u16, id: u32, profile: *mut u16, size: *mut u32) -> i32;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn InstallColorProfileW(machine: *const u16, profile: *const u16) -> i32;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn OpenColorProfileW(profile: *mut PROFILE, access: u32, share: u32, creation: u32) -> *mut core::ffi::c_void;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn CloseColorProfile(handle: *mut core::ffi::c_void) -> i32;
    fn WARN(format: *const i8, ...);
    unsafe fn profile_AtoW(profile: *const PROFILE, out: *mut PROFILE) -> BOOL;
    unsafe fn set_profile_device_key(profile: *const u16, data: *const u8, size: u32) -> i32;
    // Assume these forward declarations exist elsewhere #[no_mangle]
    // Assume these forward declarations exist elsewhere #[no_mangle]
    // Assume these forward declarations exist elsewhere pub unsafe extern "system" fn WcsOpenColorProfileW( cdm: *mut PROFILE, camp: *mut PROFILE, gmmp: *mut PROFILE, access: u32, sharing: u32, creation: u32, flags: u32, ) -> *mut core::ffi::c_void;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn DisassociateColorProfileFromDeviceW(machine: *const u16, profile: *const u16, device: *const u16) -> i32;
}
unsafe extern "system" {
    fn lstrlenW(lpString: *const u16) -> i32;
    fn lstrcpyW(lpString1: *mut u16, lpString2: *const u16) -> *mut u16;
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: LPCSTR, cbMultiByte: i32, lpWideCharStr: LPWSTR, cchWideChar: i32) -> i32;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn GetSystemDirectoryW(lpBuffer: PWSTR, uSize: u32) -> u32;
    fn lstrcatW(lpString1: PWSTR, lpString2: PCWSTR) -> PWSTR;
    fn SetLastError(dwErrCode: u32);
    fn DeleteFileW(lpFileName: PCWSTR) -> i32;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut u8, cbMultiByte: i32, lpDefaultChar: *const u8, lpUsedDefaultChar: *mut i32, ) -> i32;
    fn CopyFileW(lpExistingFileName: PCWSTR, lpNewFileName: PCWSTR, bFailIfExists: i32) -> i32;
    fn lstrcmpW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> i32;
    fn FindClose(hFind: *mut core::ffi::c_void) -> i32;
    fn FindFirstFileW( lpFileName: *const u16, lpFindFileData: *mut WIN32_FIND_DATAW, ) -> *mut core::ffi::c_void;
    fn FindNextFileW( hFind: *mut core::ffi::c_void, lpFindFileData: *mut WIN32_FIND_DATAW, ) -> i32;
    fn HeapReAlloc( hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void, dwBytes: usize, ) -> *mut core::ffi::c_void;
}

const CP_ACP: u32 = 0;

unsafe extern "C" {
    fn wine_dbg_sprintf(format: *const i8, ...) -> *const i8;
}

const ERROR_MORE_DATA: u32 = 234;

// Forward declarations for unknown functions (caller will dedup)

const PROFILE_READWRITE: u32 = 0x0002; // Placeholder, correct value unknown

// forward declarations for unknown functions

const PROFILE_READWRITE: u32 = 0x2; // icm.h

const ERROR_CALL_NOT_IMPLEMENTED: u32 = ntdll_sys::nt::ERROR_CALL_NOT_IMPLEMENTED as u32; // winerror.h: 120 (was 0x78F)

unsafe extern "C" {
    fn FIXME(fmt: *const i8, ...);
    fn TRACE(fmt: *const i8, ...);
    fn debugstr_w(s: *const u16) -> *const i8;
}

const ET_DEVICENAME: u32 = 0x00000001;
const ET_MEDIATYPE: u32 = 0x00000002;
const ET_DITHERMODE: u32 = 0x00000004;
const ET_RESOLUTION: u32 = 0x00000008;
const ET_DEVICECLASS: u32 = 0x00000010;
const ET_CMMTYPE: u32 = 0x00000020;
const ET_CLASS: u32 = 0x00000040;
const ET_DATACOLORSPACE: u32 = 0x00000080;
const ET_CONNECTIONSPACE: u32 = 0x00000100;
const ET_SIGNATURE: u32 = 0x00000200;
const ET_PLATFORM: u32 = 0x00000400;
const ET_PROFILEFLAGS: u32 = 0x00000800;
const ET_MANUFACTURER: u32 = 0x00001000;
const ET_MODEL: u32 = 0x00002000;
const ET_ATTRIBUTES: u32 = 0x00004000;
const ET_RENDERINGINTENT: u32 = 0x00008000;
const ET_CREATOR: u32 = 0x00010000;

// Forward declarations for unknown functions

// Constants assumed typical values
const LCS_SRGB: u32 = 0x73524742;            // 'sRGB'
const LCS_WINDOWS_COLOR_SPACE: u32 = 0x57696E20; // 'Win '

const PROFILE_FILENAME: u32 = 1;
const PROFILE_READ: u32 = 1;

const PROFILE_MEMBUFFER: u32 = 4;

unsafe extern "C" {
    unsafe fn header_from_file(filename: *const u16, header: *mut PROFILEHEADER) -> i32;
}
unsafe extern "C" {
    unsafe fn match_profile(record: *const ENUMTYPEW, header: *const PROFILEHEADER) -> i32;
}

const ENUM_TYPE_VERSION: u32 = 0x0201;

unsafe fn strdupW(str: LPCSTR) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();
    if !str.is_null() {
        let len = MultiByteToWideChar(CP_ACP, 0, str, -1, core::ptr::null_mut(), 0);
        ret = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as LPWSTR;
        if !ret.is_null() {
            MultiByteToWideChar(CP_ACP, 0, str, -1, ret, len);
        }
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn dbgstr_tag(tag: u32) -> *const i8 {
    wine_dbg_sprintf(
        b"'%c%c%c%c'\0".as_ptr() as *const i8,
        (tag >> 24) as u8 as i32,
        (tag >> 16) as u8 as i32,
        (tag >> 8) as u8 as i32,
        tag as u8 as i32,
    )
}

#[no_mangle]
pub unsafe extern "system" fn AssociateColorProfileWithDeviceA(
    machine: *const i8,
    profile: *const i8,
    device: *const i8,
) -> i32 {
    unsafe {
        let mut len: i32;
        let mut ret: i32 = 0;
        let profileW: *mut u16;
        let deviceW: *mut u16;

        TRACE(
            b"( %s, %s, %s )\n\0".as_ptr() as *const i8,
            debugstr_a(machine),
            debugstr_a(profile),
            debugstr_a(device),
        );

        if profile.is_null() || device.is_null() {
            SetLastError(ntdll_sys::nt::ERROR_INVALID_PARAMETER);
            return 0;
        }
        if !machine.is_null() {
            SetLastError(ntdll_sys::nt::ERROR_NOT_SUPPORTED);
            return 0;
        }

        len = MultiByteToWideChar(CP_ACP, 0, profile, -1i32, core::ptr::null_mut(), 0);
        profileW = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as *mut u16;
        if profileW.is_null() {
            return 0;
        }
        MultiByteToWideChar(CP_ACP, 0, profile, -1i32, profileW, len);

        len = MultiByteToWideChar(CP_ACP, 0, device, -1i32, core::ptr::null_mut(), 0);
        deviceW = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as *mut u16;
        if !deviceW.is_null() {
            MultiByteToWideChar(CP_ACP, 0, device, -1i32, deviceW, len);
            ret = AssociateColorProfileWithDeviceW(core::ptr::null(), profileW, deviceW);
        }
        HeapFree(GetProcessHeap(), 0, profileW as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, deviceW as *mut core::ffi::c_void);
        return ret;
    }
}

#[no_mangle]
pub unsafe extern "system" fn AssociateColorProfileWithDeviceW(
    machine: *const u16,
    profile: *const u16,
    device: *const u16,
) -> i32 {
    static DUMMY_VALUE: [u8; 12] = [0u8; 12];

    TRACE(
        b"( %s, %s, %s )\n\0".as_ptr() as *const i8,
        debugstr_w(machine),
        debugstr_w(profile),
        debugstr_w(device),
    );

    if profile.is_null() || device.is_null() {
        SetLastError(ntdll_sys::nt::ERROR_INVALID_PARAMETER);
        return 0;
    }
    if !machine.is_null() {
        SetLastError(ntdll_sys::nt::ERROR_NOT_SUPPORTED);
        return 0;
    }

    return set_profile_device_key(
        profile,
        DUMMY_VALUE.as_ptr() as *const u8,
        DUMMY_VALUE.len() as u32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DisassociateColorProfileFromDeviceA(machine: *const i8, profile: *const i8, device: *const i8) -> i32 {
    let mut len: i32;
    let mut ret: i32 = 0;
    let mut profileW: *mut u16 = core::ptr::null_mut();
    let mut deviceW: *mut u16 = core::ptr::null_mut();

    TRACE(b"( %s, %s, %s )\n\0".as_ptr() as *const i8, debugstr_a(machine), debugstr_a(profile), debugstr_a(device));

    if profile.is_null() || device.is_null() {
        SetLastError(87 /* ERROR_INVALID_PARAMETER */);
        return 0;
    }
    if !machine.is_null() {
        SetLastError(50 /* ERROR_NOT_SUPPORTED */);
        return 0;
    }

    len = MultiByteToWideChar(CP_ACP, 0, profile, -1, core::ptr::null_mut(), 0);
    profileW = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as *mut u16;
    if profileW.is_null() {
        return 0;
    }

    MultiByteToWideChar(CP_ACP, 0, profile, -1, profileW, len);

    len = MultiByteToWideChar(CP_ACP, 0, device, -1, core::ptr::null_mut(), 0);
    deviceW = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as *mut u16;
    if !deviceW.is_null() {
        MultiByteToWideChar(CP_ACP, 0, device, -1, deviceW, len);
        ret = DisassociateColorProfileFromDeviceW(core::ptr::null() as *const u16, profileW, deviceW);
    }

    HeapFree(GetProcessHeap(), 0, profileW as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, deviceW as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetColorDirectoryA(
    machine: PCSTR,
    buffer: PSTR,
    size: PDWORD,
) -> i32 {
    let mut len: i32 = 0;
    let mut bufferW: LPWSTR = core::ptr::null_mut();
    let mut ret: i32 = 0;
    let mut sizeW: u32 = 0;

    TRACE(b"( %p, %p )\n\0".as_ptr() as *const i8, buffer, size);

    if !machine.is_null() || size.is_null() {
        return 0;
    }

    if buffer.is_null() {
        ret = GetColorDirectoryW(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(sizeW));
        *size = sizeW / core::mem::size_of::<u16>() as u32;
        return ret;
    }

    sizeW = (*size as usize * core::mem::size_of::<u16>()) as u32;

    bufferW = HeapAlloc(GetProcessHeap(), 0, sizeW) as LPWSTR;
    if !bufferW.is_null() {
        ret = GetColorDirectoryW(core::ptr::null_mut(), bufferW, core::ptr::addr_of_mut!(sizeW));
        if ret != 0 {
            *size = WideCharToMultiByte(
                0, /* CP_ACP */
                0,
                bufferW,
                -1,
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as u32;
            len = WideCharToMultiByte(
                0, /* CP_ACP */
                0,
                bufferW,
                -1,
                buffer,
                *size as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            if len == 0 {
                ret = 0;
            }
        } else {
            *size = sizeW / core::mem::size_of::<u16>() as u32;
        }

        HeapFree(GetProcessHeap(), 0, bufferW as *mut core::ffi::c_void);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetColorProfileElement(
    handle: HPROFILE,
    type_: TAGTYPE,
    offset: DWORD,
    size: PDWORD,
    buffer: PVOID,
    ref_: PBOOL,
) -> BOOL {
    let mut ret: BOOL = 0;
    #[cfg(feature = "lcms2")]
    {
        TRACE(
            b"( %p, 0x%08x, %d, %p, %p, %p )\n\0".as_ptr() as *const i8,
            handle as *const core::ffi::c_void,
            type_,
            offset as i32,
            size as *const core::ffi::c_void,
            buffer as *const core::ffi::c_void,
            ref_ as *const core::ffi::c_void,
        );

        let profile: *mut profile = grab_profile(handle);

        if profile.is_null() {
            return 0;
        }

        if size.is_null() || ref_.is_null() {
            release_profile(profile);
            return 0;
        }

        if get_tag_data(profile, type_, offset, buffer, size) == 0 {
            release_profile(profile);
            return 0;
        }

        ret = get_tag_data(profile, type_, offset, buffer, size);

        *ref_ = (cmsTagLinkedTo((*profile).cmsprofile, type_) != 0) as BOOL;

        release_profile(profile);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetColorProfileElementTag(
    handle: HPROFILE,
    index: u32,
    type_: *mut u32,
) -> i32 {
    let mut ret: i32 = 0;
    let profile: *mut profile = grab_profile(handle);
    let num_tags: i32;

    TRACE(b"(%p, %d, %p)\n\0".as_ptr() as *const i8, handle, index, type_);

    if profile.is_null() {
        return 0;
    }

    if type_.is_null() {
        release_profile(profile);
        return 0;
    }

    num_tags = cmsGetTagCount((*profile).cmsprofile);
    if num_tags < 0 || index > num_tags as u32 || index < 1 {
        release_profile(profile);
        return 0;
    }

    let sig = cmsGetTagSignature((*profile).cmsprofile, index.wrapping_sub(1));
    if sig != 0 {
        *type_ = sig;
        ret = 1;
    }

    release_profile(profile);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetColorProfileFromHandle(
    handle: HPROFILE,
    buffer: *mut u8,
    size: *mut u32,
) -> i32 {
    let mut ret: i32 = 0;
    #[cfg(HAVE_LCMS2)]
    {
        let profile: *mut profile = grab_profile(handle);
        let mut header: PROFILEHEADER = core::mem::zeroed();

        TRACE(b" (%p, %p, %p)\n\0".as_ptr() as *const i8, handle as *mut u8, buffer as *mut u8, size as *mut u8);

        if profile.is_null() {
            return 0;
        }

        if size.is_null() {
            release_profile(profile);
            return 0;
        }

        get_profile_header(profile, core::ptr::addr_of_mut!(header));

        if buffer.is_null() || header.phSize > *size {
            *size = header.phSize;
            release_profile(profile);
            return 0;
        }

        /* No endian conversion needed */
        core::ptr::copy_nonoverlapping((*profile).data as *const u8, buffer, (*profile).size as usize);
        *size = (*profile).size;

        release_profile(profile);
        ret = 1;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn GetColorProfileHeader(handle: HPROFILE, header: *mut PROFILEHEADER) -> i32 {

    let profile = grab_profile(handle);
    println!("( {:p}, {:p} )", handle as *const core::ffi::c_void, header as *const core::ffi::c_void);

    if profile.is_null() {
        return 0;
    }

    if header.is_null() {
        release_profile(profile);
        return 0;
    }

    get_profile_header(profile, header);
    release_profile(profile);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetCountColorProfileElements(handle: HPROFILE, count: PDWORD) -> BOOL {
    let mut ret: BOOL = 0;
    let profile: *mut profile = grab_profile(handle);
    let num_tags: i32;

    TRACE(b"( %p, %p )\n\0".as_ptr() as *const i8, handle, count);

    if profile.is_null() {
        return 0;
    }

    if count.is_null() {
        release_profile(profile);
        return 0;
    }

    num_tags = cmsGetTagCount((*profile).cmsprofile as *mut core::ffi::c_void);
    if num_tags >= 0 {
        *count = num_tags as u32;
        ret = 1;
    }

    release_profile(profile);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetStandardColorSpaceProfileA(
    machine: *const i8,
    id: u32,
    profile: *mut i8,
    size: *mut u32,
) -> i32 {
    unsafe {
        let mut len: i32;
        let mut profileW: *mut u16;
        let mut ret: i32 = 0;
        let mut sizeW: u32;

        TRACE(
            b"( 0x%08x, %p, %p )\n\0".as_ptr() as *const i8,
            id,
            profile as *const core::ffi::c_void,
            size as *const core::ffi::c_void,
        );

        if !machine.is_null() {
            SetLastError(ERROR_NOT_SUPPORTED);
            return 0;
        }

        if size.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        sizeW = *size * (core::mem::size_of::<u16>() as u32);

        if profile.is_null() {
            ret = GetStandardColorSpaceProfileW(
                core::ptr::null_mut(),
                id,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(sizeW),
            );
            *size = sizeW / (core::mem::size_of::<u16>() as u32);
            return ret;
        }

        profileW = HeapAlloc(GetProcessHeap(), 0, sizeW as usize) as *mut u16;
        if !profileW.is_null() {
            ret = GetStandardColorSpaceProfileW(
                core::ptr::null_mut(),
                id,
                profileW,
                core::ptr::addr_of_mut!(sizeW),
            );
            if ret != 0 {
                *size = WideCharToMultiByte(
                    CP_ACP,
                    0,
                    profileW,
                    -1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                ) as u32;
                len = WideCharToMultiByte(
                    CP_ACP,
                    0,
                    profileW,
                    -1,
                    profile,
                    *size as i32,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                if len == 0 {
                    ret = 0;
                }
            } else {
                *size = sizeW / (core::mem::size_of::<u16>() as u32);
            }

            HeapFree(GetProcessHeap(), 0, profileW as *mut core::ffi::c_void);
        }
        ret
    }
}

#[no_mangle]
pub unsafe extern "system" fn EnumColorProfilesW(
    machine: PCWSTR,
    record: *mut ENUMTYPEW,
    buffer: PBYTE,
    size: PDWORD,
    number: PDWORD,
) -> i32 {
    const SPEC: [u16; 6] = [b'\\' as u16, b'*' as u16, b'i' as u16, b'c' as u16, b'm' as u16, 0];
    let mut ret: i32 = 0;
    let mut match_: i32;
    let mut colordir: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut glob: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut profiles: *mut *mut u16 = core::ptr::null_mut();
    let mut i: u32;
    let mut len: u32 = core::mem::size_of_val(&colordir) as u32;
    let mut count: u32 = 0;
    let mut totalsize: u32 = 0;
    let mut header: PROFILEHEADER = core::mem::zeroed();
    let mut data: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut find: *mut core::ffi::c_void;

    TRACE(
        b"( %p, %p, %p, %p, %p )\n\0".as_ptr() as *const i8,
        machine,
        record,
        buffer,
        size,
        number,
    );

    if !machine.is_null()
        || record.is_null()
        || size.is_null()
        || (*record).dwSize as usize != core::mem::size_of::<ENUMTYPEW>()
        || (*record).dwVersion != ENUM_TYPE_VERSION
    {
        return 0;
    }

    ret = GetColorDirectoryW(machine, colordir.as_mut_ptr(), &mut len);
    if ret == 0 || len + SPEC.len() as u32 > MAX_PATH as u32 {
        WARN(b"Can't retrieve color directory\n\0".as_ptr() as *const i8);
        return 0;
    }

    lstrcpyW(glob.as_mut_ptr(), colordir.as_ptr());
    lstrcatW(glob.as_mut_ptr(), SPEC.as_ptr());

    find = FindFirstFileW(glob.as_ptr(), &mut data);
    if find == INVALID_HANDLE_VALUE {
        return 0;
    }

    profiles = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<*mut u16>() + 1,
    ) as *mut *mut u16;
    if profiles.is_null() {
        goto exit;
    }

    ret = header_from_file(data.cFileName.as_ptr(), &mut header);
    if ret != 0 {
        match_ = match_profile(record, &header);
        if match_ != 0 {
            len = core::mem::size_of::<u16>() as u32 * (lstrlenW(data.cFileName.as_ptr()) + 1);
            *profiles.add(count as usize) = HeapAlloc(GetProcessHeap(), 0, len as usize) as *mut u16;

            if (*profiles.add(count as usize)).is_null() {
                goto exit;
            } else {
                TRACE(
                    b"matching profile: %s\n\0".as_ptr() as *const i8,
                    debugstr_w(data.cFileName.as_ptr()),
                );
                lstrcpyW(*profiles.add(count as usize), data.cFileName.as_ptr());
                totalsize += len;
                count += 1;
            }
        }
    }

    while FindNextFileW(find, &mut data) != 0 {
        ret = header_from_file(data.cFileName.as_ptr(), &mut header);
        if ret == 0 {
            continue;
        }

        match_ = match_profile(record, &header);
        if match_ != 0 {
            let tmp = HeapReAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                profiles as *mut core::ffi::c_void,
                core::mem::size_of::<*mut u16>() * (count as usize + 1),
            ) as *mut *mut u16;
            if tmp.is_null() {
                goto exit;
            } else {
                profiles = tmp;
            }

            len = core::mem::size_of::<u16>() as u32 * (lstrlenW(data.cFileName.as_ptr()) + 1);
            *profiles.add(count as usize) = HeapAlloc(GetProcessHeap(), 0, len as usize) as *mut u16;

            if (*profiles.add(count as usize)).is_null() {
                goto exit;
            } else {
                TRACE(
                    b"matching profile: %s\n\0".as_ptr() as *const i8,
                    debugstr_w(data.cFileName.as_ptr()),
                );
                lstrcpyW(*profiles.add(count as usize), data.cFileName.as_ptr());
                totalsize += len;
                count += 1;
            }
        }
    }

    totalsize += 1;
    if !buffer.is_null() && *size >= totalsize {
        let mut p = buffer as *mut u16;

        i = 0;
        while i < count {
            lstrcpyW(p, *profiles.add(i as usize));
            p = p.add(lstrlenW(*profiles.add(i as usize)) as usize + 1);
            i += 1;
        }
        *p = 0;
        ret = 1;
    } else {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        ret = 0;
    }

    *size = totalsize;
    if !number.is_null() {
        *number = count;
    }

    'exit: {
        i = 0;
        while i < count {
            HeapFree(
                GetProcessHeap(),
                0,
                *profiles.add(i as usize) as *mut core::ffi::c_void,
            );
            i += 1;
        }
        HeapFree(GetProcessHeap(), 0, profiles as *mut core::ffi::c_void);
        FindClose(find);

        return ret;
    }
}

#[no_mangle]
pub unsafe extern "system" fn InstallColorProfileA(
    machine: *const i8,
    profile: *const i8,
) -> i32 {
    let mut len: u32;
    let mut profileW: *mut u16;
    let mut ret: i32 = 0;

    TRACE(c"( %s )\n".as_ptr() as *const i8, debugstr_a(profile));

    if !machine.is_null() || profile.is_null() {
        return 0;
    }

    len = MultiByteToWideChar(
        CP_ACP,
        0,
        profile,
        -1,
        core::ptr::null_mut(),
        0,
    ) as u32;

    profileW = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize) * core::mem::size_of::<u16>(),
    ) as *mut u16;

    if !profileW.is_null() {
        MultiByteToWideChar(CP_ACP, 0, profile, -1, profileW, len as i32);

        ret = InstallColorProfileW(core::ptr::null(), profileW);
        HeapFree(GetProcessHeap(), 0, profileW as *mut core::ffi::c_void);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn IsColorProfileTagPresent(
    handle: HPROFILE,
    type_: TAGTYPE,
    present: PBOOL,
) -> i32 {
    let mut ret: i32 = 0;

    if cfg!(feature = "lcms2") {
        let profile: *mut profile = grab_profile(handle);

        TRACE(
            b"( %p, 0x%08x, %p )\n\0".as_ptr() as *const i8,
            handle as *mut core::ffi::c_void,
            type_,
            present as *mut core::ffi::c_void,
        );

        if profile.is_null() {
            return 0;
        }

        if present.is_null() {
            release_profile(profile);
            return 0;
        }

        *present = (cmsIsTag((*profile).cmsprofile, type_) != 0) as i32;
        release_profile(profile);
        ret = 1;
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn IsColorProfileValid(handle: HPROFILE, valid: PBOOL) -> BOOL {
    let mut ret: BOOL = 0; // FALSE
    if cfg!(feature = "lcms2") {
        let profile = grab_profile(handle);
        TRACE(b"( %p, %p )\n\0".as_ptr() as *const i8, handle as *const _, valid as *const _);
        if profile.is_null() {
            return 0;
        }
        if valid.is_null() {
            release_profile(profile);
            return 0;
        }
        if !(*profile).data.is_null() {
            *valid = 1;
            ret = 1;
        }
        release_profile(profile);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SetColorProfileElement(
    handle: HPROFILE,
    type_: TAGTYPE,
    offset: u32,
    size: *mut u32,
    buffer: *mut core::ffi::c_void,
) -> i32 {
    let mut ret: i32 = 0;
    // TRACE( "( %p, 0x%08x, %d, %p, %p )\n", handle, type, offset, size, buffer );
    let profile: *mut profile = grab_profile(handle);
    if profile.is_null() {
        return 0;
    }
    if size.is_null() || buffer.is_null() || ((*profile).access & PROFILE_READWRITE) == 0 {
        release_profile(profile);
        return 0;
    }
    ret = set_tag_data(profile, type_, offset, buffer, size);
    release_profile(profile);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SetColorProfileHeader(handle: HPROFILE, header: PPROFILEHEADER) -> i32 {
    if cfg!(feature = "lcms2") {
        let profile = grab_profile(handle);
        TRACE(b"( %p, %p )\n\0".as_ptr() as *const i8, handle, header);
        if profile.is_null() {
            return 0;
        }
        if header.is_null() || ((*profile).access & PROFILE_READWRITE) == 0 {
            release_profile(profile);
            return 0;
        }
        set_profile_header(profile, header);
        release_profile(profile);
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn UninstallColorProfileA(machine: *const i8, profile: *const i8, delete: i32) -> i32 {
    let mut len: u32;
    let profileW: *mut u16;
    let mut ret: i32 = 0;

    TRACE(b"( %s, %x )\n\0".as_ptr() as *const i8, debugstr_a(profile), delete);

    if !machine.is_null() || profile.is_null() {
        return 0;
    }

    len = MultiByteToWideChar(0, 0, profile, -1, core::ptr::null_mut(), 0) as u32;
    profileW = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as *mut u16;

    if !profileW.is_null() {
        MultiByteToWideChar(0, 0, profile, -1, profileW, len as i32);
        ret = UninstallColorProfileW(core::ptr::null::<u16>(), profileW, delete);
        HeapFree(GetProcessHeap(), 0, profileW as *mut core::ffi::c_void);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn UninstallColorProfileW(machine: PCWSTR, profile: PCWSTR, delete: i32) -> i32 {
    TRACE(b"( %s, %x )\n".as_ptr() as *const i8, debugstr_w(profile), delete);
    if !machine.is_null() || profile.is_null() {
        return 0;
    }
    if delete != 0 {
        return DeleteFileW(profile);
    }
    return 1;
}

const CP_ACP: u32 = 0;

#[no_mangle]
pub unsafe extern "system" fn OpenColorProfileA(
    profile: *mut PROFILE,
    access: DWORD,
    sharing: DWORD,
    creation: DWORD,
) -> HPROFILE {
    // TRACE( "( %p, 0x%08x, 0x%08x, 0x%08x )\n", profile, access, sharing, creation );
    let mut handle: HPROFILE = core::ptr::null_mut();
    let mut profileW: PROFILE = core::mem::zeroed();

    if profile.is_null() || (*profile).pProfileData.is_null() {
        return core::ptr::null_mut();
    }

    if ((*profile).dwType & PROFILE_MEMBUFFER) != 0 {
        return OpenColorProfileW(profile, access, sharing, creation);
    }

    if profile_AtoW(profile, &mut profileW as *mut PROFILE) == 0 {
        return 0; // FALSE
    }

    handle = OpenColorProfileW(&mut profileW as *mut PROFILE, access, sharing, creation);
    HeapFree(GetProcessHeap(), 0, profileW.pProfileData);
    handle
}

#[no_mangle]
pub unsafe extern "system" fn WcsGetUsePerUserProfiles(name: *const u16, class: u32, use_per_user_profile: *mut i32) -> i32 {
    FIXME(
        b"%s %s %p\n\0".as_ptr() as *const i8,
        debugstr_w(name),
        dbgstr_tag(class),
        use_per_user_profile as *const core::ffi::c_void,
    );
    SetLastError(ntdll_sys::nt::ERROR_CALL_NOT_IMPLEMENTED as u32);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn WcsEnumColorProfilesSize(scope: WCS_PROFILE_MANAGEMENT_SCOPE, record: *mut ENUMTYPEW, size: *mut DWORD) -> BOOL {
    FIXME(b"%d %p %p\n\0".as_ptr() as *const i8, scope, record, size);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn WcsOpenColorProfileA(
    cdm: *mut PROFILE,
    camp: *mut PROFILE,
    gmmp: *mut PROFILE,
    access: u32,
    sharing: u32,
    creation: u32,
    flags: u32,
) -> *mut core::ffi::c_void {
    let mut cdmW: PROFILE = core::mem::zeroed();
    let mut campW: PROFILE = core::mem::zeroed();
    let mut gmmpW: PROFILE = core::mem::zeroed();
    let mut ret: *mut core::ffi::c_void = core::ptr::null_mut();

    TRACE(
        b"%p, %p, %p, %08x, %08x, %08x, %08x\n\0".as_ptr() as *const i8,
        cdm,
        camp,
        gmmp,
        access,
        sharing,
        creation,
        flags,
    );

    if cdm.is_null() || profile_AtoW(cdm as *const PROFILE, &raw mut cdmW) == 0 {
        return core::ptr::null_mut();
    }

    'done: {
        if !camp.is_null() && profile_AtoW(camp as *const PROFILE, &raw mut campW) == 0 {
            break 'done;
        }
        if !gmmp.is_null() && profile_AtoW(gmmp as *const PROFILE, &raw mut gmmpW) == 0 {
            break 'done;
        }

        ret = WcsOpenColorProfileW(
            &raw mut cdmW,
            &raw mut campW,
            &raw mut gmmpW,
            access,
            sharing,
            creation,
            flags,
        );
    }

    HeapFree(GetProcessHeap(), 0, cdmW.pProfileData as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, campW.pProfileData as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, gmmpW.pProfileData as *mut core::ffi::c_void);

    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * MSCMS - Color Management System for Wine
//  *
//  * Copyright 2004, 2005, 2006, 2008 Hans Leidekker
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
// #include "config.h"
// #include "wine/debug.h"
// #include "wine/unicode.h"
//
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "winnls.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "winreg.h"
// #include "shlwapi.h"
// #include "icm.h"
//
// #include "mscms_priv.h"
//
// static void basename( LPCWSTR path, LPWSTR name )
// {
//     INT i = lstrlenW( path );
//
//     while (i > 0 && path[i - 1] != '\\' && path[i - 1] != '/') i--;
//     lstrcpyW( name, &path[i] );
// }
//
// static inline LPWSTR strdupW( LPCSTR str )
// {
//     LPWSTR ret = NULL;
//     if (str)
//     {
//         DWORD len = MultiByteToWideChar( CP_ACP, 0, str, -1, NULL, 0 );
//         if ((ret = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) )))
//             MultiByteToWideChar( CP_ACP, 0, str, -1, ret, len );
//     }
//     return ret;
// }
//
// const char *dbgstr_tag( DWORD tag )
// {
//     return wine_dbg_sprintf( "'%c%c%c%c'",
//         (char)(tag >> 24), (char)(tag >> 16), (char)(tag >> 8), (char)(tag) );
// }
//
// WINE_DEFAULT_DEBUG_CHANNEL(mscms);
//
// /******************************************************************************
//  * AssociateColorProfileWithDeviceA               [MSCMS.@]
//  */
// BOOL WINAPI AssociateColorProfileWithDeviceA( PCSTR machine, PCSTR profile, PCSTR device )
// {
//     int len;
//     BOOL ret = FALSE;
//     WCHAR *profileW, *deviceW;
//
//     TRACE( "( %s, %s, %s )\n", debugstr_a(machine), debugstr_a(profile), debugstr_a(device) );
//
//     if (!profile || !device)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     if (machine)
//     {
//         SetLastError( ERROR_NOT_SUPPORTED );
//         return FALSE;
//     }
//
//     len = MultiByteToWideChar( CP_ACP, 0, profile, -1, NULL, 0 );
//     if (!(profileW = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) ))) return FALSE;
//
//     MultiByteToWideChar( CP_ACP, 0, profile, -1, profileW, len );
//
//     len = MultiByteToWideChar( CP_ACP, 0, device, -1, NULL, 0 );
//     if ((deviceW = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) )))
//     {
//         MultiByteToWideChar( CP_ACP, 0, device, -1, deviceW, len );
//         ret = AssociateColorProfileWithDeviceW( NULL, profileW, deviceW );
//     }
//
//     HeapFree( GetProcessHeap(), 0, profileW );
//     HeapFree( GetProcessHeap(), 0, deviceW );
//     return ret;
// }
//
// static BOOL set_profile_device_key( PCWSTR file, const BYTE *value, DWORD size )
// {
//     static const WCHAR fmtW[] = {'%','c','%','c','%','c','%','c',0};
//     static const WCHAR icmW[] = {'S','o','f','t','w','a','r','e','\\',
//                                  'M','i','c','r','o','s','o','f','t','\\',
//                                  'W','i','n','d','o','w','s',' ','N','T','\\',
//                                  'C','u','r','r','e','n','t','V','e','r','s','i','o','n','\\',
//                                  'I','C','M',0};
//     PROFILEHEADER header;
//     PROFILE profile;
//     HPROFILE handle;
//     HKEY icm_key, class_key;
//     WCHAR basenameW[MAX_PATH], classW[5];
//
//     profile.dwType = PROFILE_FILENAME;
//     profile.pProfileData = (PVOID)file;
//     profile.cbDataSize = (lstrlenW( file ) + 1) * sizeof(WCHAR);
//
//     /* FIXME is the profile installed? */
//     if (!(handle = OpenColorProfileW( &profile, PROFILE_READ, 0, OPEN_EXISTING )))
//     {
//         SetLastError( ERROR_INVALID_PROFILE );
//         return FALSE;
//     }
//     if (!GetColorProfileHeader( handle, &header ))
//     {
//         CloseColorProfile( handle );
//         SetLastError( ERROR_INVALID_PROFILE );
//         return FALSE;
//     }
//     RegCreateKeyExW( HKEY_LOCAL_MACHINE, icmW, 0, NULL, 0, KEY_ALL_ACCESS, NULL, &icm_key, NULL );
//
//     basename( file, basenameW );
//     sprintfW( classW, fmtW, (header.phClass >> 24) & 0xff, (header.phClass >> 16) & 0xff,
//                             (header.phClass >> 8) & 0xff,  header.phClass & 0xff );
//
//     RegCreateKeyExW( icm_key, classW, 0, NULL, 0, KEY_ALL_ACCESS, NULL, &class_key, NULL );
//     if (value) RegSetValueExW( class_key, basenameW, 0, REG_BINARY, value, size );
//     else RegDeleteValueW( class_key, basenameW );
//
//     RegCloseKey( class_key );
//     RegCloseKey( icm_key );
//     CloseColorProfile( handle );
//     return TRUE;
// }
//
// /******************************************************************************
//  * AssociateColorProfileWithDeviceW               [MSCMS.@]
//  */
// BOOL WINAPI AssociateColorProfileWithDeviceW( PCWSTR machine, PCWSTR profile, PCWSTR device )
// {
//     static const BYTE dummy_value[12];
//
//     TRACE( "( %s, %s, %s )\n", debugstr_w(machine), debugstr_w(profile), debugstr_w(device) );
//
//     if (!profile || !device)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     if (machine)
//     {
//         SetLastError( ERROR_NOT_SUPPORTED );
//         return FALSE;
//     }
//
//     return set_profile_device_key( profile, dummy_value, sizeof(dummy_value) );
// }
//
// /******************************************************************************
//  * DisassociateColorProfileFromDeviceA            [MSCMS.@]
//  */
// BOOL WINAPI DisassociateColorProfileFromDeviceA( PCSTR machine, PCSTR profile, PCSTR device )
// {
//     int len;
//     BOOL ret = FALSE;
//     WCHAR *profileW, *deviceW;
//
//     TRACE( "( %s, %s, %s )\n", debugstr_a(machine), debugstr_a(profile), debugstr_a(device) );
//
//     if (!profile || !device)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     if (machine)
//     {
//         SetLastError( ERROR_NOT_SUPPORTED );
//         return FALSE;
//     }
//
//     len = MultiByteToWideChar( CP_ACP, 0, profile, -1, NULL, 0 );
//     if (!(profileW = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) ))) return FALSE;
//
//     MultiByteToWideChar( CP_ACP, 0, profile, -1, profileW, len );
//
//     len = MultiByteToWideChar( CP_ACP, 0, device, -1, NULL, 0 );
//     if ((deviceW = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) )))
//     {
//         MultiByteToWideChar( CP_ACP, 0, device, -1, deviceW, len );
//         ret = DisassociateColorProfileFromDeviceW( NULL, profileW, deviceW );
//     }
//
//     HeapFree( GetProcessHeap(), 0, profileW );
//     HeapFree( GetProcessHeap(), 0, deviceW );
//     return ret;
// }
//
// /******************************************************************************
//  * DisassociateColorProfileFromDeviceW            [MSCMS.@]
//  */
// BOOL WINAPI DisassociateColorProfileFromDeviceW( PCWSTR machine, PCWSTR profile, PCWSTR device )
// {
//     TRACE( "( %s, %s, %s )\n", debugstr_w(machine), debugstr_w(profile), debugstr_w(device) );
//
//     if (!profile || !device)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     if (machine)
//     {
//         SetLastError( ERROR_NOT_SUPPORTED );
//         return FALSE;
//     }
//
//     return set_profile_device_key( profile, NULL, 0 );
// }
//
// /******************************************************************************
//  * GetColorDirectoryA               [MSCMS.@]
//  *
//  * See GetColorDirectoryW.
//  */
// BOOL WINAPI GetColorDirectoryA( PCSTR machine, PSTR buffer, PDWORD size )
// {
//     INT len;
//     LPWSTR bufferW;
//     BOOL ret = FALSE;
//     DWORD sizeW;
//
//     TRACE( "( %p, %p )\n", buffer, size );
//
//     if (machine || !size) return FALSE;
//
//     if (!buffer)
//     {
//         ret = GetColorDirectoryW( NULL, NULL, &sizeW );
//         *size = sizeW / sizeof(WCHAR);
//         return ret;
//     }
//
//     sizeW = *size * sizeof(WCHAR);
//
//     bufferW = HeapAlloc( GetProcessHeap(), 0, sizeW );
//     if (bufferW)
//     {
//         if ((ret = GetColorDirectoryW( NULL, bufferW, &sizeW )))
//         {
//             *size = WideCharToMultiByte( CP_ACP, 0, bufferW, -1, NULL, 0, NULL, NULL );
//             len = WideCharToMultiByte( CP_ACP, 0, bufferW, -1, buffer, *size, NULL, NULL );
//             if (!len) ret = FALSE;
//         }
//         else *size = sizeW / sizeof(WCHAR);
//
//         HeapFree( GetProcessHeap(), 0, bufferW );
//     }
//     return ret;
// }
//
// /******************************************************************************
//  * GetColorDirectoryW               [MSCMS.@]
//  *
//  * Get the directory where color profiles are stored.
//  *
//  * PARAMS
//  *  machine  [I]   Name of the machine for which to get the color directory.
//  *                 Must be NULL, which indicates the local machine.
//  *  buffer   [I]   Buffer to receive the path name.
//  *  size     [I/O] Size of the buffer in bytes. On return the variable holds
//  *                 the number of bytes actually needed.
//  */
// BOOL WINAPI GetColorDirectoryW( PCWSTR machine, PWSTR buffer, PDWORD size )
// {
//     WCHAR colordir[MAX_PATH];
//     static const WCHAR colorsubdir[] =
//         {'\\','s','p','o','o','l','\\','d','r','i','v','e','r','s','\\','c','o','l','o','r',0};
//     DWORD len;
//
//     TRACE( "( %p, %p )\n", buffer, size );
//
//     if (machine || !size) return FALSE;
//
//     GetSystemDirectoryW( colordir, ARRAY_SIZE( colordir ));
//     lstrcatW( colordir, colorsubdir );
//
//     len = lstrlenW( colordir ) * sizeof(WCHAR);
//
//     if (buffer && len <= *size)
//     {
//         lstrcpyW( buffer, colordir );
//         *size = len;
//         return TRUE;
//     }
//
//     SetLastError( ERROR_MORE_DATA );
//     *size = len;
//     return FALSE;
// }
//
// /******************************************************************************
//  * GetColorProfileElement               [MSCMS.@]
//  *
//  * Retrieve data for a specified tag type.
//  *
//  * PARAMS
//  *  profile  [I]   Handle to a color profile.
//  *  type     [I]   ICC tag type. 
//  *  offset   [I]   Offset in bytes to start copying from. 
//  *  size     [I/O] Size of the buffer in bytes. On return the variable holds
//  *                 the number of bytes actually needed.
//  *  buffer   [O]   Buffer to receive the tag data.
//  *  ref      [O]   Pointer to a BOOL that specifies whether more than one tag
//  *                 references the data.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI GetColorProfileElement( HPROFILE handle, TAGTYPE type, DWORD offset, PDWORD size,
//                                     PVOID buffer, PBOOL ref )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//
//     TRACE( "( %p, 0x%08x, %d, %p, %p, %p )\n", handle, type, offset, size, buffer, ref );
//
//     if (!profile) return FALSE;
//
//     if (!size || !ref)
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     if (!get_tag_data( profile, type, offset, buffer, size ))
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     ret = get_tag_data( profile, type, offset, buffer, size );
//     *ref = cmsTagLinkedTo( profile->cmsprofile, type ) != 0;
//     release_profile( profile );
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * GetColorProfileElementTag               [MSCMS.@]
//  *
//  * Get the tag type from a color profile by index. 
//  *
//  * PARAMS
//  *  profile  [I]   Handle to a color profile.
//  *  index    [I]   Index into the tag table of the color profile.
//  *  type     [O]   Pointer to a variable that holds the ICC tag type on return.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  The tag table index starts at 1.
//  *  Use GetCountColorProfileElements to retrieve a count of tagged elements.
//  */
// BOOL WINAPI GetColorProfileElementTag( HPROFILE handle, DWORD index, PTAGTYPE type )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//     cmsInt32Number num_tags;
//     cmsTagSignature sig;
//
//     TRACE( "( %p, %d, %p )\n", handle, index, type );
//
//     if (!profile) return FALSE;
//
//     if (!type)
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     num_tags = cmsGetTagCount( profile->cmsprofile );
//     if (num_tags < 0 || index > num_tags || index < 1)
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     if ((sig = cmsGetTagSignature( profile->cmsprofile, index - 1 )))
//     {
//         *type = sig;
//         ret = TRUE;
//     }
//     release_profile( profile );
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * GetColorProfileFromHandle               [MSCMS.@]
//  *
//  * Retrieve an ICC color profile by handle.
//  *
//  * PARAMS
//  *  profile  [I]   Handle to a color profile.
//  *  buffer   [O]   Buffer to receive the ICC profile.
//  *  size     [I/O] Size of the buffer in bytes. On return the variable holds the
//  *                 number of bytes actually needed.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  The profile returned will be in big-endian format.
//  */
// BOOL WINAPI GetColorProfileFromHandle( HPROFILE handle, PBYTE buffer, PDWORD size )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//     PROFILEHEADER header;
//
//     TRACE( "( %p, %p, %p )\n", handle, buffer, size );
//
//     if (!profile) return FALSE;
//
//     if (!size)
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     get_profile_header( profile, &header );
//
//     if (!buffer || header.phSize > *size)
//     {
//         *size = header.phSize;
//         release_profile( profile );
//         return FALSE;
//     }
//
//     /* No endian conversion needed */
//     memcpy( buffer, profile->data, profile->size );
//     *size = profile->size;
//
//     release_profile( profile );
//     ret = TRUE;
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * GetColorProfileHeader               [MSCMS.@]
//  *
//  * Retrieve a color profile header by handle.
//  *
//  * PARAMS
//  *  profile  [I]   Handle to a color profile.
//  *  header   [O]   Buffer to receive the ICC profile header.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  The profile header returned will be adjusted for endianness.
//  */
// BOOL WINAPI GetColorProfileHeader( HPROFILE handle, PPROFILEHEADER header )
// {
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//
//     TRACE( "( %p, %p )\n", handle, header );
//
//     if (!profile) return FALSE;
//
//     if (!header)
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     get_profile_header( profile, header );
//     release_profile( profile );
//     return TRUE;
//
// #else
//     return FALSE;
// #endif /* HAVE_LCMS2 */
// }
//
// /******************************************************************************
//  * GetCountColorProfileElements               [MSCMS.@]
//  *
//  * Retrieve the number of elements in a color profile.
//  *
//  * PARAMS
//  *  profile  [I] Handle to a color profile.
//  *  count    [O] Pointer to a variable which is set to the number of elements
//  *               in the color profile.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI GetCountColorProfileElements( HPROFILE handle, PDWORD count )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//     cmsInt32Number num_tags;
//
//     TRACE( "( %p, %p )\n", handle, count );
//
//     if (!profile) return FALSE;
//
//     if (!count)
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     if ((num_tags = cmsGetTagCount( profile->cmsprofile )) >= 0)
//     {
//         *count = num_tags;
//         ret = TRUE;
//     }
//     release_profile( profile );
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * GetStandardColorSpaceProfileA               [MSCMS.@]
//  *
//  * See GetStandardColorSpaceProfileW.
//  */
// BOOL WINAPI GetStandardColorSpaceProfileA( PCSTR machine, DWORD id, PSTR profile, PDWORD size )
// {
//     INT len;
//     LPWSTR profileW;
//     BOOL ret = FALSE;
//     DWORD sizeW;
//
//     TRACE( "( 0x%08x, %p, %p )\n", id, profile, size );
//
//     if (machine) 
//     {
//         SetLastError( ERROR_NOT_SUPPORTED );
//         return FALSE;
//     }
//
//     if (!size) 
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//
//     sizeW = *size * sizeof(WCHAR);
//
//     if (!profile)
//     {
//         ret = GetStandardColorSpaceProfileW( NULL, id, NULL, &sizeW );
//         *size = sizeW / sizeof(WCHAR);
//         return ret;
//     }
//
//     profileW = HeapAlloc( GetProcessHeap(), 0, sizeW );
//     if (profileW)
//     {
//         if ((ret = GetStandardColorSpaceProfileW( NULL, id, profileW, &sizeW )))
//         {
//             *size = WideCharToMultiByte( CP_ACP, 0, profileW, -1, NULL, 0, NULL, NULL );
//             len = WideCharToMultiByte( CP_ACP, 0, profileW, -1, profile, *size, NULL, NULL );
//             if (!len) ret = FALSE;
//         }
//         else *size = sizeW / sizeof(WCHAR);
//
//         HeapFree( GetProcessHeap(), 0, profileW );
//     }
//     return ret;
// }
//
// /******************************************************************************
//  * GetStandardColorSpaceProfileW               [MSCMS.@]
//  *
//  * Retrieve the profile filename for a given standard color space id.
//  *
//  * PARAMS
//  *  machine  [I]   Name of the machine for which to get the standard color space.
//  *                 Must be NULL, which indicates the local machine.
//  *  id       [I]   Id of a standard color space.
//  *  profile  [O]   Buffer to receive the profile filename.
//  *  size     [I/O] Size of the filename buffer in bytes.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI GetStandardColorSpaceProfileW( PCWSTR machine, DWORD id, PWSTR profile, PDWORD size )
// {
//     static const WCHAR rgbprofilefile[] =
//         { '\\','s','r','g','b',' ','c','o','l','o','r',' ',
//           's','p','a','c','e',' ','p','r','o','f','i','l','e','.','i','c','m',0 };
//     WCHAR rgbprofile[MAX_PATH];
//     DWORD len = sizeof(rgbprofile);
//
//     TRACE( "( 0x%08x, %p, %p )\n", id, profile, size );
//
//     if (machine) 
//     {
//         SetLastError( ERROR_NOT_SUPPORTED );
//         return FALSE;
//     }
//
//     if (!size) 
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//
//     if (!profile)
//     {
//         SetLastError( ERROR_INSUFFICIENT_BUFFER );
//         return FALSE;
//     }
//
//     GetColorDirectoryW( machine, rgbprofile, &len );
//
//     switch (id)
//     {
//         case LCS_sRGB:
//         case LCS_WINDOWS_COLOR_SPACE: /* FIXME */
//             lstrcatW( rgbprofile, rgbprofilefile );
//             len = lstrlenW( rgbprofile ) * sizeof(WCHAR);
//
//             if (*size < len)
//             {
//                 *size = len;
//                 SetLastError( ERROR_MORE_DATA );
//                 return FALSE;
//             }
//
//             lstrcpyW( profile, rgbprofile );
//             break;
//
//         default:
//             SetLastError( ERROR_FILE_NOT_FOUND );
//             return FALSE;
//     }
//     return TRUE;
// }
//
// static BOOL header_from_file( LPCWSTR file, PPROFILEHEADER header )
// {
//     static const WCHAR slash[] = {'\\',0};
//     BOOL ret;
//     PROFILE profile;
//     WCHAR path[MAX_PATH];
//     DWORD size = sizeof(path);
//     HANDLE handle;
//
//     ret = GetColorDirectoryW( NULL, path, &size );
//     if (!ret)
//     {
//         WARN( "Can't retrieve color directory\n" );
//         return FALSE;
//     }
//     if (size + sizeof(slash) + sizeof(WCHAR) * lstrlenW( file ) > sizeof(path))
//     {
//         WARN( "Filename too long\n" );
//         return FALSE;
//     }
//
//     lstrcatW( path, slash );
//     lstrcatW( path, file );
//
//     profile.dwType = PROFILE_FILENAME;
//     profile.pProfileData = path;
//     profile.cbDataSize = lstrlenW( path ) + 1;
//
//     handle = OpenColorProfileW( &profile, PROFILE_READ, FILE_SHARE_READ, OPEN_EXISTING );
//     if (!handle)
//     {
//         WARN( "Can't open color profile\n" );
//         return FALSE;
//     }
//
//     ret = GetColorProfileHeader( handle, header );
//     if (!ret)
//         WARN( "Can't retrieve color profile header\n" );
//
//     CloseColorProfile( handle );
//     return ret;
// }
//
// static BOOL match_profile( PENUMTYPEW rec, PPROFILEHEADER hdr )
// {
//     if (rec->dwFields & ET_DEVICENAME)
//     {
//         FIXME( "ET_DEVICENAME: %s\n", debugstr_w(rec->pDeviceName) );
//     }
//     if (rec->dwFields & ET_MEDIATYPE)
//     {
//         FIXME( "ET_MEDIATYPE: 0x%08x\n", rec->dwMediaType );
//     }
//     if (rec->dwFields & ET_DITHERMODE)
//     {
//         FIXME( "ET_DITHERMODE: 0x%08x\n", rec->dwDitheringMode );
//     }
//     if (rec->dwFields & ET_RESOLUTION)
//     {
//         FIXME( "ET_RESOLUTION: 0x%08x, 0x%08x\n",
//                rec->dwResolution[0], rec->dwResolution[1] );
//     }
//     if (rec->dwFields & ET_DEVICECLASS)
//     {
//         FIXME( "ET_DEVICECLASS: %s\n", dbgstr_tag(rec->dwMediaType) );
//     }
//     if (rec->dwFields & ET_CMMTYPE)
//     {
//         TRACE( "ET_CMMTYPE: %s\n", dbgstr_tag(rec->dwCMMType) );
//         if (rec->dwCMMType != hdr->phCMMType) return FALSE;
//     }
//     if (rec->dwFields & ET_CLASS)
//     {
//         TRACE( "ET_CLASS: %s\n", dbgstr_tag(rec->dwClass) );
//         if (rec->dwClass != hdr->phClass) return FALSE;
//     }
//     if (rec->dwFields & ET_DATACOLORSPACE)
//     {
//         TRACE( "ET_DATACOLORSPACE: %s\n", dbgstr_tag(rec->dwDataColorSpace) );
//         if (rec->dwDataColorSpace != hdr->phDataColorSpace) return FALSE;
//     }
//     if (rec->dwFields & ET_CONNECTIONSPACE)
//     {
//         TRACE( "ET_CONNECTIONSPACE: %s\n", dbgstr_tag(rec->dwConnectionSpace) );
//         if (rec->dwConnectionSpace != hdr->phConnectionSpace) return FALSE;
//     }
//     if (rec->dwFields & ET_SIGNATURE)
//     {
//         TRACE( "ET_SIGNATURE: %s\n", dbgstr_tag(rec->dwSignature) );
//         if (rec->dwSignature != hdr->phSignature) return FALSE;
//     }
//     if (rec->dwFields & ET_PLATFORM)
//     {
//         TRACE( "ET_PLATFORM: %s\n", dbgstr_tag(rec->dwPlatform) );
//         if (rec->dwPlatform != hdr->phPlatform) return FALSE;
//     }
//     if (rec->dwFields & ET_PROFILEFLAGS)
//     {
//         TRACE( "ET_PROFILEFLAGS: 0x%08x\n", rec->dwProfileFlags );
//         if (rec->dwProfileFlags != hdr->phProfileFlags) return FALSE;
//     }
//     if (rec->dwFields & ET_MANUFACTURER)
//     {
//         TRACE( "ET_MANUFACTURER: %s\n", dbgstr_tag(rec->dwManufacturer) );
//         if (rec->dwManufacturer != hdr->phManufacturer) return FALSE;
//     }
//     if (rec->dwFields & ET_MODEL)
//     {
//         TRACE( "ET_MODEL: %s\n", dbgstr_tag(rec->dwModel) );
//         if (rec->dwModel != hdr->phModel) return FALSE;
//     }
//     if (rec->dwFields & ET_ATTRIBUTES)
//     {
//         TRACE( "ET_ATTRIBUTES: 0x%08x, 0x%08x\n",
//                rec->dwAttributes[0], rec->dwAttributes[1] );
//         if (rec->dwAttributes[0] != hdr->phAttributes[0] || 
//             rec->dwAttributes[1] != hdr->phAttributes[1]) return FALSE;
//     }
//     if (rec->dwFields & ET_RENDERINGINTENT)
//     {
//         TRACE( "ET_RENDERINGINTENT: 0x%08x\n", rec->dwRenderingIntent );
//         if (rec->dwRenderingIntent != hdr->phRenderingIntent) return FALSE;
//     }
//     if (rec->dwFields & ET_CREATOR)
//     {
//         TRACE( "ET_CREATOR: %s\n", dbgstr_tag(rec->dwCreator) );
//         if (rec->dwCreator != hdr->phCreator) return FALSE;
//     }
//     return TRUE;
// }
//
// /******************************************************************************
//  * EnumColorProfilesA               [MSCMS.@]
//  *
//  * See EnumColorProfilesW.
//  */
// BOOL WINAPI EnumColorProfilesA( PCSTR machine, PENUMTYPEA record, PBYTE buffer,
//                                 PDWORD size, PDWORD number )
// {
//     BOOL match, ret = FALSE;
//     char spec[] = "\\*.icm";
//     char colordir[MAX_PATH], glob[MAX_PATH], **profiles = NULL;
//     DWORD i, len = sizeof(colordir), count = 0, totalsize = 0;
//     PROFILEHEADER header;
//     WIN32_FIND_DATAA data;
//     ENUMTYPEW recordW;
//     WCHAR *fileW = NULL, *deviceW = NULL;
//     HANDLE find;
//
//     TRACE( "( %p, %p, %p, %p, %p )\n", machine, record, buffer, size, number );
//
//     if (machine || !record || !size ||
//         record->dwSize != sizeof(ENUMTYPEA) ||
//         record->dwVersion != ENUM_TYPE_VERSION) return FALSE;
//
//     ret = GetColorDirectoryA( machine, colordir, &len );
//     if (!ret || len + sizeof(spec) > MAX_PATH)
//     {
//         WARN( "can't retrieve color directory\n" );
//         return FALSE;
//     }
//
//     lstrcpyA( glob, colordir );
//     lstrcatA( glob, spec );
//
//     find = FindFirstFileA( glob, &data );
//     if (find == INVALID_HANDLE_VALUE) return FALSE;
//
//     profiles = HeapAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(char *) + 1 );
//     if (!profiles) goto exit;
//
//     memcpy( &recordW, record, sizeof(ENUMTYPEA) );
//     if (record->pDeviceName)
//     {
//         deviceW = strdupW( record->pDeviceName );
//         if (!(recordW.pDeviceName = deviceW)) goto exit;
//     }
//
//     fileW = strdupW( data.cFileName );
//     if (!fileW) goto exit;
//
//     ret = header_from_file( fileW, &header );
//     if (ret)
//     {
//         match = match_profile( &recordW, &header );
//         if (match)
//         {
//             len = sizeof(char) * (lstrlenA( data.cFileName ) + 1);
//             profiles[count] = HeapAlloc( GetProcessHeap(), 0, len );
//
//             if (!profiles[count]) goto exit;
//             else
//             {
//                 TRACE( "matching profile: %s\n", debugstr_a(data.cFileName) );
//                 lstrcpyA( profiles[count], data.cFileName );
//                 totalsize += len;
//                 count++;
//             }
//         }
//     }
//     HeapFree( GetProcessHeap(), 0, fileW );
//     fileW = NULL;
//
//     while (FindNextFileA( find, &data ))
//     {
//         fileW = strdupW( data.cFileName );
//         if (!fileW) goto exit;
//
//         ret = header_from_file( fileW, &header );
//         if (!ret)
//         {
//             HeapFree( GetProcessHeap(), 0, fileW );
//             continue;
//         }
//
//         match = match_profile( &recordW, &header );
//         if (match)
//         {
//             char **tmp = HeapReAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY,
//                                       profiles, sizeof(char *) * (count + 1) );
//             if (!tmp) goto exit;
//             else profiles = tmp;
//
//             len = sizeof(char) * (lstrlenA( data.cFileName ) + 1);
//             profiles[count] = HeapAlloc( GetProcessHeap(), 0, len );
//
//             if (!profiles[count]) goto exit;
//             else
//             {
//                 TRACE( "matching profile: %s\n", debugstr_a(data.cFileName) );
//                 lstrcpyA( profiles[count], data.cFileName );
//                 totalsize += len;
//                 count++;
//             }
//         }
//         HeapFree( GetProcessHeap(), 0, fileW );
//         fileW = NULL;
//     }
//
//     totalsize++;
//     if (buffer && *size >= totalsize)
//     {
//         char *p = (char *)buffer;
//
//         for (i = 0; i < count; i++)
//         {
//             lstrcpyA( p, profiles[i] );
//             p += lstrlenA( profiles[i] ) + 1;
//         }
//         *p = 0;
//         ret = TRUE;
//     }
//     else
//     {
//         SetLastError( ERROR_INSUFFICIENT_BUFFER );
//         ret = FALSE;
//     }
//
//     *size = totalsize;
//     if (number) *number = count;
//
// exit:
//     for (i = 0; i < count; i++)
//         HeapFree( GetProcessHeap(), 0, profiles[i] );
//     HeapFree( GetProcessHeap(), 0, profiles );
//     HeapFree( GetProcessHeap(), 0, deviceW );
//     HeapFree( GetProcessHeap(), 0, fileW );
//     FindClose( find );
//
//     return ret;
// }
//
// /******************************************************************************
//  * EnumColorProfilesW               [MSCMS.@]
//  *
//  * Enumerate profiles that match given criteria.
//  *
//  * PARAMS
//  *  machine  [I]   Name of the machine for which to enumerate profiles.
//  *                 Must be NULL, which indicates the local machine.
//  *  record   [I]   Record of criteria that a profile must match.
//  *  buffer   [O]   Buffer to receive a string array of profile filenames.
//  *  size     [I/O] Size of the filename buffer in bytes.
//  *  number   [O]   Number of filenames copied into buffer.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI EnumColorProfilesW( PCWSTR machine, PENUMTYPEW record, PBYTE buffer,
//                                 PDWORD size, PDWORD number )
// {
//     static const WCHAR spec[] = {'\\','*','i','c','m',0};
//     BOOL match, ret = FALSE;
//     WCHAR colordir[MAX_PATH], glob[MAX_PATH], **profiles = NULL;
//     DWORD i, len = sizeof(colordir), count = 0, totalsize = 0;
//     PROFILEHEADER header;
//     WIN32_FIND_DATAW data;
//     HANDLE find;
//
//     TRACE( "( %p, %p, %p, %p, %p )\n", machine, record, buffer, size, number );
//
//     if (machine || !record || !size ||
//         record->dwSize != sizeof(ENUMTYPEW) ||
//         record->dwVersion != ENUM_TYPE_VERSION) return FALSE;
//
//     ret = GetColorDirectoryW( machine, colordir, &len );
//     if (!ret || len + sizeof(spec) > MAX_PATH)
//     {
//         WARN( "Can't retrieve color directory\n" );
//         return FALSE;
//     }
//
//     lstrcpyW( glob, colordir );
//     lstrcatW( glob, spec );
//
//     find = FindFirstFileW( glob, &data );
//     if (find == INVALID_HANDLE_VALUE) return FALSE;
//
//     profiles = HeapAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(WCHAR *) + 1 );
//     if (!profiles) goto exit;
//
//     ret = header_from_file( data.cFileName, &header );
//     if (ret)
//     {
//         match = match_profile( record, &header );
//         if (match)
//         {
//             len = sizeof(WCHAR) * (lstrlenW( data.cFileName ) + 1);
//             profiles[count] = HeapAlloc( GetProcessHeap(), 0, len );
//
//             if (!profiles[count]) goto exit;
//             else
//             {
//                 TRACE( "matching profile: %s\n", debugstr_w(data.cFileName) );
//                 lstrcpyW( profiles[count], data.cFileName );
//                 totalsize += len;
//                 count++;
//             }
//         }
//     }
//
//     while (FindNextFileW( find, &data ))
//     {
//         ret = header_from_file( data.cFileName, &header );
//         if (!ret) continue;
//
//         match = match_profile( record, &header );
//         if (match)
//         {
//             WCHAR **tmp = HeapReAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY,
//                                        profiles, sizeof(WCHAR *) * (count + 1) );
//             if (!tmp) goto exit;
//             else profiles = tmp;
//
//             len = sizeof(WCHAR) * (lstrlenW( data.cFileName ) + 1);
//             profiles[count] = HeapAlloc( GetProcessHeap(), 0, len );
//
//             if (!profiles[count]) goto exit;
//             else
//             {
//                 TRACE( "matching profile: %s\n", debugstr_w(data.cFileName) );
//                 lstrcpyW( profiles[count], data.cFileName );
//                 totalsize += len;
//                 count++;
//             }
//         }
//     }
//
//     totalsize++;
//     if (buffer && *size >= totalsize)
//     {
//         WCHAR *p = (WCHAR *)buffer;
//
//         for (i = 0; i < count; i++)
//         {
//             lstrcpyW( p, profiles[i] );
//             p += lstrlenW( profiles[i] ) + 1;
//         }
//         *p = 0;
//         ret = TRUE;
//     }
//     else
//     {
//         SetLastError( ERROR_INSUFFICIENT_BUFFER );
//         ret = FALSE;
//     }
//
//     *size = totalsize;
//     if (number) *number = count;
//
// exit:
//     for (i = 0; i < count; i++)
//         HeapFree( GetProcessHeap(), 0, profiles[i] );
//     HeapFree( GetProcessHeap(), 0, profiles );
//     FindClose( find );
//
//     return ret;
// }
//
// /******************************************************************************
//  * InstallColorProfileA               [MSCMS.@]
//  *
//  * See InstallColorProfileW.
//  */
// BOOL WINAPI InstallColorProfileA( PCSTR machine, PCSTR profile )
// {
//     UINT len;
//     LPWSTR profileW;
//     BOOL ret = FALSE;
//
//     TRACE( "( %s )\n", debugstr_a(profile) );
//
//     if (machine || !profile) return FALSE;
//
//     len = MultiByteToWideChar( CP_ACP, 0, profile, -1, NULL, 0 );
//     profileW = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) );
//
//     if (profileW)
//     {
//         MultiByteToWideChar( CP_ACP, 0, profile, -1, profileW, len );
//
//         ret = InstallColorProfileW( NULL, profileW );
//         HeapFree( GetProcessHeap(), 0, profileW );
//     }
//     return ret;
// }
//
// /******************************************************************************
//  * InstallColorProfileW               [MSCMS.@]
//  *
//  * Install a color profile.
//  *
//  * PARAMS
//  *  machine  [I] Name of the machine to install the profile on. Must be NULL,
//  *               which indicates the local machine.
//  *  profile  [I] Full path name of the profile to install.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI InstallColorProfileW( PCWSTR machine, PCWSTR profile )
// {
//     WCHAR dest[MAX_PATH], base[MAX_PATH];
//     DWORD size = sizeof(dest);
//     static const WCHAR slash[] = { '\\', 0 };
//
//     TRACE( "( %s )\n", debugstr_w(profile) );
//
//     if (machine || !profile) return FALSE;
//
//     if (!GetColorDirectoryW( machine, dest, &size )) return FALSE;
//
//     basename( profile, base );
//
//     lstrcatW( dest, slash );
//     lstrcatW( dest, base );
//
//     /* Is source equal to destination? */
//     if (!lstrcmpW( profile, dest )) return TRUE;
//
//     return CopyFileW( profile, dest, TRUE );
// }
//
// /******************************************************************************
//  * IsColorProfileTagPresent               [MSCMS.@]
//  *
//  * Determine if a given ICC tag type is present in a color profile.
//  *
//  * PARAMS
//  *  profile  [I] Color profile handle.
//  *  tag      [I] ICC tag type.
//  *  present  [O] Pointer to a BOOL variable. Set to TRUE if tag type is present,
//  *               FALSE otherwise.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI IsColorProfileTagPresent( HPROFILE handle, TAGTYPE type, PBOOL present )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//
//     TRACE( "( %p, 0x%08x, %p )\n", handle, type, present );
//
//     if (!profile) return FALSE;
//
//     if (!present)
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     *present = (cmsIsTag( profile->cmsprofile, type ) != 0);
//     release_profile( profile );
//     ret = TRUE;
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * IsColorProfileValid               [MSCMS.@]
//  *
//  * Determine if a given color profile is valid.
//  *
//  * PARAMS
//  *  profile  [I] Color profile handle.
//  *  valid    [O] Pointer to a BOOL variable. Set to TRUE if profile is valid,
//  *               FALSE otherwise.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE 
//  */
// BOOL WINAPI IsColorProfileValid( HPROFILE handle, PBOOL valid )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//
//     TRACE( "( %p, %p )\n", handle, valid );
//
//     if (!profile) return FALSE;
//
//     if (!valid)
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     if (profile->data) ret = *valid = TRUE;
//     release_profile( profile );
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * SetColorProfileElement               [MSCMS.@]
//  *
//  * Set data for a specified tag type.
//  *
//  * PARAMS
//  *  profile  [I]   Handle to a color profile.
//  *  type     [I]   ICC tag type.
//  *  offset   [I]   Offset in bytes to start copying to.
//  *  size     [I/O] Size of the buffer in bytes. On return the variable holds the
//  *                 number of bytes actually needed.
//  *  buffer   [O]   Buffer holding the tag data.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI SetColorProfileElement( HPROFILE handle, TAGTYPE type, DWORD offset, PDWORD size,
//                                     PVOID buffer )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//
//     TRACE( "( %p, 0x%08x, %d, %p, %p )\n", handle, type, offset, size, buffer );
//
//     if (!profile) return FALSE;
//
//     if (!size || !buffer || !(profile->access & PROFILE_READWRITE))
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     ret = set_tag_data( profile, type, offset, buffer, size );
//     release_profile( profile );
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * SetColorProfileHeader               [MSCMS.@]
//  *
//  * Set header data for a given profile.
//  *
//  * PARAMS
//  *  profile  [I] Handle to a color profile.
//  *  header   [I] Buffer holding the header data.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI SetColorProfileHeader( HPROFILE handle, PPROFILEHEADER header )
// {
// #ifdef HAVE_LCMS2
//     struct profile *profile = grab_profile( handle );
//
//     TRACE( "( %p, %p )\n", handle, header );
//
//     if (!profile) return FALSE;
//
//     if (!header || !(profile->access & PROFILE_READWRITE))
//     {
//         release_profile( profile );
//         return FALSE;
//     }
//     set_profile_header( profile, header );
//     release_profile( profile );
//     return TRUE;
//
// #else
//     return FALSE;
// #endif /* HAVE_LCMS2 */
// }
//
// /******************************************************************************
//  * UninstallColorProfileA               [MSCMS.@]
//  *
//  * See UninstallColorProfileW.
//  */
// BOOL WINAPI UninstallColorProfileA( PCSTR machine, PCSTR profile, BOOL delete )
// {
//     UINT len;
//     LPWSTR profileW;
//     BOOL ret = FALSE;
//
//     TRACE( "( %s, %x )\n", debugstr_a(profile), delete );
//
//     if (machine || !profile) return FALSE;
//
//     len = MultiByteToWideChar( CP_ACP, 0, profile, -1, NULL, 0 );
//     profileW = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) );
//
//     if (profileW)
//     {
//         MultiByteToWideChar( CP_ACP, 0, profile, -1, profileW, len );
//
//         ret = UninstallColorProfileW( NULL, profileW , delete );
//
//         HeapFree( GetProcessHeap(), 0, profileW );
//     }
//     return ret;
// }
//
// /******************************************************************************
//  * UninstallColorProfileW               [MSCMS.@]
//  *
//  * Uninstall a color profile.
//  *
//  * PARAMS
//  *  machine  [I] Name of the machine to uninstall the profile on. Must be NULL,
//  *               which indicates the local machine.
//  *  profile  [I] Full path name of the profile to uninstall.
//  *  delete   [I] Bool that specifies whether the profile file should be deleted.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI UninstallColorProfileW( PCWSTR machine, PCWSTR profile, BOOL delete )
// {
//     TRACE( "( %s, %x )\n", debugstr_w(profile), delete );
//
//     if (machine || !profile) return FALSE;
//
//     if (delete) return DeleteFileW( profile );
//
//     return TRUE;
// }
//
// static BOOL profile_AtoW( const PROFILE *in, PROFILE *out )
// {
//     int len;
//     if (!in->pProfileData) return FALSE;
//     len = MultiByteToWideChar( CP_ACP, 0, in->pProfileData, -1, NULL, 0 );
//     if (!(out->pProfileData = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) ))) return FALSE;
//     out->cbDataSize = len * sizeof(WCHAR);
//     MultiByteToWideChar( CP_ACP, 0, in->pProfileData, -1, out->pProfileData, len );
//     out->dwType = in->dwType;
//     return TRUE;
// }
//
// /******************************************************************************
//  * OpenColorProfileA               [MSCMS.@]
//  *
//  * See OpenColorProfileW.
//  */
// HPROFILE WINAPI OpenColorProfileA( PPROFILE profile, DWORD access, DWORD sharing, DWORD creation )
// {
//     HPROFILE handle = NULL;
//     PROFILE profileW;
//
//     TRACE( "( %p, 0x%08x, 0x%08x, 0x%08x )\n", profile, access, sharing, creation );
//
//     if (!profile || !profile->pProfileData) return NULL;
//
//     /* No AW conversion needed for memory based profiles */
//     if (profile->dwType & PROFILE_MEMBUFFER)
//         return OpenColorProfileW( profile, access, sharing, creation );
//
//     if (!profile_AtoW( profile, &profileW )) return FALSE;
//     handle = OpenColorProfileW( &profileW, access, sharing, creation );
//     HeapFree( GetProcessHeap(), 0, profileW.pProfileData );
//     return handle;
// }
//
// /******************************************************************************
//  * OpenColorProfileW               [MSCMS.@]
//  *
//  * Open a color profile.
//  *
//  * PARAMS
//  *  profile   [I] Pointer to a color profile structure.
//  *  access    [I] Desired access.
//  *  sharing   [I] Sharing mode.
//  *  creation  [I] Creation mode.
//  *
//  * RETURNS
//  *  Success: Handle to the opened profile.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  Values for access:   PROFILE_READ or PROFILE_READWRITE.
//  *  Values for sharing:  0 (no sharing), FILE_SHARE_READ and/or FILE_SHARE_WRITE.
//  *  Values for creation: one of CREATE_NEW, CREATE_ALWAYS, OPEN_EXISTING,
//  *                       OPEN_ALWAYS, TRUNCATE_EXISTING.
//  *  Sharing and creation flags are ignored for memory based profiles.
//  */
// HPROFILE WINAPI OpenColorProfileW( PPROFILE profile, DWORD access, DWORD sharing, DWORD creation )
// {
// #ifdef HAVE_LCMS2
//     cmsHPROFILE cmsprofile = NULL;
//     char *data = NULL;
//     HANDLE handle = INVALID_HANDLE_VALUE;
//     DWORD size;
//
//     TRACE( "( %p, 0x%08x, 0x%08x, 0x%08x )\n", profile, access, sharing, creation );
//
//     if (!profile || !profile->pProfileData) return NULL;
//
//     if (profile->dwType == PROFILE_MEMBUFFER)
//     {
//         /* FIXME: access flags not implemented for memory based profiles */
//
//         if (!(data = HeapAlloc( GetProcessHeap(), 0, profile->cbDataSize ))) return NULL;
//         memcpy( data, profile->pProfileData, profile->cbDataSize );
//
//         if (!(cmsprofile = cmsOpenProfileFromMem( data, profile->cbDataSize )))
//         {
//             HeapFree( GetProcessHeap(), 0, data );
//             return FALSE;
//         }
//         size = profile->cbDataSize;
//     }
//     else if (profile->dwType == PROFILE_FILENAME)
//     {
//         DWORD read, flags = 0;
//
//         TRACE( "profile file: %s\n", debugstr_w( profile->pProfileData ) );
//
//         if (access & PROFILE_READ) flags = GENERIC_READ;
//         if (access & PROFILE_READWRITE) flags = GENERIC_READ|GENERIC_WRITE;
//
//         if (!flags) return NULL;
//         if (!sharing) sharing = FILE_SHARE_READ;
//
//         if (!PathIsRelativeW( profile->pProfileData ))
//             handle = CreateFileW( profile->pProfileData, flags, sharing, NULL, creation, 0, NULL );
//         else
//         {
//             WCHAR *path;
//
//             if (!GetColorDirectoryW( NULL, NULL, &size ) && GetLastError() == ERROR_MORE_DATA)
//             {
//                 size += (strlenW( profile->pProfileData ) + 2) * sizeof(WCHAR);
//                 if (!(path = HeapAlloc( GetProcessHeap(), 0, size ))) return NULL;
//                 GetColorDirectoryW( NULL, path, &size );
//                 PathAddBackslashW( path );
//                 strcatW( path, profile->pProfileData );
//             }
//             else return NULL;
//             handle = CreateFileW( path, flags, sharing, NULL, creation, 0, NULL );
//             HeapFree( GetProcessHeap(), 0, path );
//         }
//         if (handle == INVALID_HANDLE_VALUE)
//         {
//             WARN( "Unable to open color profile %u\n", GetLastError() );
//             return NULL;
//         }
//         if ((size = GetFileSize( handle, NULL )) == INVALID_FILE_SIZE)
//         {
//             ERR( "Unable to retrieve size of color profile\n" );
//             CloseHandle( handle );
//             return NULL;
//         }
//         if (!(data = HeapAlloc( GetProcessHeap(), 0, size )))
//         {
//             ERR( "Unable to allocate memory for color profile\n" );
//             CloseHandle( handle );
//             return NULL;
//         }
//         if (!ReadFile( handle, data, size, &read, NULL ) || read != size)
//         {
//             ERR( "Unable to read color profile\n" );
//
//             CloseHandle( handle );
//             HeapFree( GetProcessHeap(), 0, data );
//             return NULL;
//         }
//         if (!(cmsprofile = cmsOpenProfileFromMem( data, size )))
//         {
//             CloseHandle( handle );
//             HeapFree( GetProcessHeap(), 0, data );
//             return NULL;
//         }
//     }
//     else
//     {
//         ERR( "Invalid profile type %u\n", profile->dwType );
//         return NULL;
//     }
//
//     if (cmsprofile)
//     {
//         struct profile profile;
//         HPROFILE hprof;
//
//         profile.file       = handle;
//         profile.access     = access;
//         profile.data       = data;
//         profile.size       = size;
//         profile.cmsprofile = cmsprofile;
//
//         if ((hprof = create_profile( &profile ))) return hprof;
//         HeapFree( GetProcessHeap(), 0, data );
//         cmsCloseProfile( cmsprofile );
//     }
//     CloseHandle( handle );
//
// #endif /* HAVE_LCMS2 */
//     return NULL;
// }
//
// /******************************************************************************
//  * CloseColorProfile               [MSCMS.@]
//  *
//  * Close a color profile.
//  *
//  * PARAMS
//  *  profile  [I] Handle to the profile.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CloseColorProfile( HPROFILE profile )
// {
//     BOOL ret = FALSE;
// #ifdef HAVE_LCMS2
//
//     TRACE( "( %p )\n", profile );
//     ret = close_profile( profile );
//
// #endif /* HAVE_LCMS2 */
//     return ret;
// }
//
// /******************************************************************************
//  * WcsGetUsePerUserProfiles               [MSCMS.@]
//  */
// BOOL WINAPI WcsGetUsePerUserProfiles( const WCHAR* name, DWORD class, BOOL* use_per_user_profile )
// {
//     FIXME( "%s %s %p\n", debugstr_w(name), dbgstr_tag(class), use_per_user_profile );
//     SetLastError( ERROR_CALL_NOT_IMPLEMENTED );
//     return FALSE;
// }
//
// /******************************************************************************
//  * WcsEnumColorProfilesSize               [MSCMS.@]
//  */
// BOOL WINAPI WcsEnumColorProfilesSize( WCS_PROFILE_MANAGEMENT_SCOPE scope, ENUMTYPEW *record, DWORD *size )
// {
//     FIXME( "%d %p %p\n", scope, record, size );
//     SetLastError( ERROR_CALL_NOT_IMPLEMENTED );
//     return FALSE;
// }
//
// /******************************************************************************
//  * WcsOpenColorProfileA               [MSCMS.@]
//  */
// HPROFILE WINAPI WcsOpenColorProfileA( PROFILE *cdm, PROFILE *camp, PROFILE *gmmp, DWORD access, DWORD sharing,
//                                       DWORD creation, DWORD flags )
// {
//     PROFILE cdmW, campW = {0}, gmmpW = {0};
//     HPROFILE ret = NULL;
//
//     TRACE( "%p, %p, %p, %08x, %08x, %08x, %08x\n", cdm, camp, gmmp, access, sharing, creation, flags );
//
//     if (!cdm || !profile_AtoW( cdm, &cdmW )) return NULL;
//     if (camp && !profile_AtoW( camp, &campW )) goto done;
//     if (gmmp && !profile_AtoW( gmmp, &gmmpW )) goto done;
//
//     ret = WcsOpenColorProfileW( &cdmW, &campW, &gmmpW, access, sharing, creation, flags );
//
// done:
//     HeapFree( GetProcessHeap(), 0, cdmW.pProfileData );
//     HeapFree( GetProcessHeap(), 0, campW.pProfileData );
//     HeapFree( GetProcessHeap(), 0, gmmpW.pProfileData );
//     return ret;
// }
//
// /******************************************************************************
//  * WcsOpenColorProfileW               [MSCMS.@]
//  */
// HPROFILE WINAPI WcsOpenColorProfileW( PROFILE *cdm, PROFILE *camp, PROFILE *gmmp, DWORD access, DWORD sharing,
//                                       DWORD creation, DWORD flags )
// {
//     TRACE( "%p, %p, %p, %08x, %08x, %08x, %08x\n", cdm, camp, gmmp, access, sharing, creation, flags );
//     FIXME("no support for WCS profiles\n" );
//
//     return OpenColorProfileW( cdm, access, sharing, creation );
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
