use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
#[allow(unused_imports)]
use core::ffi::c_void;
use crate::internal::{HIWORD, LOWORD};
use crate::FIELD_OFFSET;

// ---- debug-print macros (DbgPrint-free no-ops; mirror cursoricon.rs) ----

// ---- kernel32.dll exports (WINAPI = stdcall) ----
extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: LPCSTR, cbMultiByte: core::ffi::c_int, lpWideCharStr: LPWSTR, cchWideChar: core::ffi::c_int) -> core::ffi::c_int;
    #[allow(dead_code)]
    fn GetLastError() -> DWORD;
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
    fn SearchPathW(lpPath: LPCWSTR, lpFileName: LPCWSTR, lpExtension: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR) -> DWORD;
    fn CreateFileW(lpFileName: LPCWSTR, dwDesiredAccess: u32, dwShareMode: u32, lpSecurityAttributes: *mut core::ffi::c_void, dwCreationDisposition: u32, dwFlagsAndAttributes: u32, hTemplateFile: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn CreateFileMappingW(hFile: *mut core::ffi::c_void, lpFileMappingAttributes: *mut core::ffi::c_void, flProtect: u32, dwMaximumSizeHigh: u32, dwMaximumSizeLow: u32, lpName: LPCWSTR) -> *mut core::ffi::c_void;
    fn MapViewOfFile(hFileMappingObject: *mut core::ffi::c_void, dwDesiredAccess: u32, dwFileOffsetHigh: u32, dwFileOffsetLow: u32, dwNumberOfBytesToMap: usize) -> *mut core::ffi::c_void;
    fn UnmapViewOfFile(lpBaseAddress: *const core::ffi::c_void) -> i32;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> i32;
    fn GetFileSize(hFile: *mut core::ffi::c_void, lpFileSizeHigh: *mut u32) -> u32;
}

// ---- user32 / gdi32 own-exports (WINAPI = stdcall) ----
// CreateIconFromResourceEx / LookupIconIdFromDirectoryEx / DestroyIcon /
// get_best_icon_file_offset are own-exports defined in windows/cursoricon.rs.
extern "system" {
    fn GetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int;
    fn CreateIconFromResourceEx(pbIconBits: PBYTE, cbIconBits: DWORD, fIcon: BOOL, dwVersion: DWORD, cxDesired: core::ffi::c_int, cyDesired: core::ffi::c_int, uFlags: UINT) -> HICON;
    fn LookupIconIdFromDirectoryEx(presbits: PBYTE, fIcon: BOOL, cxDesired: core::ffi::c_int, cyDesired: core::ffi::c_int, Flags: UINT) -> core::ffi::c_int;
    fn DestroyIcon(hIcon: HICON) -> i32;
}
// get_best_icon_file_offset is exported __cdecl (see cursoricon.rs).
extern "C" {
    fn get_best_icon_file_offset(dir: LPBYTE, dwFileSize: DWORD, cxDesired: core::ffi::c_int, cyDesired: core::ffi::c_int, bIcon: BOOL, fuLoad: DWORD, ptHotSpot: *mut POINT) -> DWORD;
}

// ---- ntdll Rtl* PE-image helpers (NOT in win32k-sys; imported from ntdll) ----
extern "system" {
    fn RtlImageNtHeader(BaseAddress: PVOID) -> PIMAGE_NT_HEADERS;
    fn RtlImageDirectoryEntryToData(BaseAddress: PVOID, MappedAsImage: BOOLEAN, Directory: USHORT, Size: *mut ULONG) -> PVOID;
    fn RtlImageRvaToVa(NtHeaders: PIMAGE_NT_HEADERS, BaseAddress: PVOID, Rva: ULONG, LastRvaSection: *mut PIMAGE_SECTION_HEADER) -> PVOID;
}

// ---- CRT cdecl helpers ----
extern "C" {
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const core::ffi::c_char) -> usize;
}

/// `RtlCopyMemory`/`CopyMemory` (winnt.h macros over memcpy).
#[inline]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, n: usize) {
    memcpy(dst, src, n);
}
#[inline]
unsafe fn CopyMemory(dst: *mut c_void, src: *const c_void, n: usize) {
    memcpy(dst, src, n);
}

/// `MAKEWORD(a, b)` — winnt.h macro.
#[inline]
fn MAKEWORD(a: u8, b: u8) -> WORD {
    (a as WORD) | ((b as WORD) << 8)
}

/// `abs()` — libc cdecl; used for icon resource-id scoring.
#[inline]
fn abs(i: core::ffi::c_int) -> core::ffi::c_int {
    i.wrapping_abs()
}

// ---- winnt.h / winuser.h constants not in win32k-sys ----
const CP_ACP: u32 = 0;
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const OPEN_EXISTING: u32 = 3;
const FILE_MAP_READ: u32 = 0x0004;
// RT_* resource type ids (winuser.h MAKEINTRESOURCE) — used as plain integers here.
const RT_ICON: u32 = 3;
const RT_GROUP_ICON: u32 = 14;

// ---- IMAGE_OS2_HEADER (NE header) — not in win32k-sys ----
#[repr(C)]
#[allow(non_snake_case)]
pub struct IMAGE_OS2_HEADER {
    pub ne_magic: UINT16,
    pub ne_ver: core::ffi::c_char,
    pub ne_rev: core::ffi::c_char,
    pub ne_enttab: UINT16,
    pub ne_cbenttab: UINT16,
    pub ne_crc: LONG,
    pub ne_flags: UINT16,
    pub ne_autodata: UINT16,
    pub ne_heap: UINT16,
    pub ne_stack: UINT16,
    pub ne_csip: LONG,
    pub ne_sssp: LONG,
    pub ne_cseg: UINT16,
    pub ne_cmod: UINT16,
    pub ne_cbnrestab: UINT16,
    pub ne_segtab: UINT16,
    pub ne_rsrctab: UINT16,
    pub ne_restab: UINT16,
    pub ne_modtab: UINT16,
    pub ne_imptab: UINT16,
    pub ne_nrestab: LONG,
    pub ne_cmovent: UINT16,
    pub ne_align: UINT16,
    pub ne_cres: UINT16,
    pub ne_exetyp: BYTE,
    pub ne_flagsothers: BYTE,
    pub ne_pretthunks: UINT16,
    pub ne_psegrefbytes: UINT16,
    pub ne_swaparea: UINT16,
    pub ne_expver: UINT16,
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct icoICONDIRENTRY {
    pub bWidth: BYTE,
    pub bHeight: BYTE,
    pub bColorCount: BYTE,
    pub bReserved: BYTE,
    pub wPlanes: WORD,
    pub wBitCount: WORD,
    pub dwBytesInRes: DWORD,
    pub dwImageOffset: DWORD,
}
pub type LPicoICONDIRENTRY = *mut icoICONDIRENTRY;

#[repr(C)]
pub struct icoICONDIR {
    pub idReserved: WORD,
    pub idType: WORD,
    pub idCount: WORD,
    pub idEntries: [icoICONDIRENTRY; 1],
}
pub type LPicoICONDIR = *mut icoICONDIR;

#[repr(C)]
pub struct NE_NAMEINFO {
    pub offset: WORD,
    pub length: WORD,
    pub flags: WORD,
    pub id: WORD,
    pub handle: WORD,
    pub usage: WORD,
}

#[repr(C)]
pub struct NE_TYPEINFO {
    pub type_id: WORD,
    pub count: WORD,
    pub resloader: DWORD,
}

#[repr(C)]
pub struct taganiheader {
    pub cbsizeof: DWORD,
    pub cframes: DWORD,
    pub csteps: DWORD,
    pub cx: DWORD,
    pub cy: DWORD,
    pub cbitcount: DWORD,
    pub cplanes: DWORD,
    pub jifrate: DWORD,
    pub flags: DWORD,
}
pub type aniheader = taganiheader;

pub const NE_RSCTYPE_ICON: u32 = 0x8003;
pub const NE_RSCTYPE_GROUP_ICON: u32 = 0x800e;

unsafe fn find_entry_by_id(
    dir: *const IMAGE_RESOURCE_DIRECTORY,
    id: WORD,
    root: *const core::ffi::c_void,
) -> *const IMAGE_RESOURCE_DIRECTORY {
    let entry = dir.add(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut min: i32 = (*dir).NumberOfNamedEntries as i32;
    let mut max: i32 = min + (*dir).NumberOfIdEntries as i32 - 1;
    while min <= max {
        let pos = (min + max) / 2;
        let e = entry.offset(pos as isize);
        if (*e).__bindgen_anon_1.Id == id {
            return (root as *const core::ffi::c_char)
                .add((*e).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as usize)
                as *const IMAGE_RESOURCE_DIRECTORY;
        }
        if (*e).__bindgen_anon_1.Id > id {
            max = pos - 1;
        } else {
            min = pos + 1;
        }
    }
    core::ptr::null()
}

unsafe fn find_entry_default(dir: *const IMAGE_RESOURCE_DIRECTORY, root: *const core::ffi::c_void) -> *const IMAGE_RESOURCE_DIRECTORY {
    let entry = dir.add(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    (root as *const core::ffi::c_char).add((*entry).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as usize) as *const IMAGE_RESOURCE_DIRECTORY
}

unsafe fn USER32_GetResourceTable(peimage: LPBYTE, pesize: DWORD, retptr: *mut LPBYTE) -> DWORD {
    let mz_header: *mut IMAGE_DOS_HEADER = peimage as *mut IMAGE_DOS_HEADER;

    *retptr = core::ptr::null_mut();

    if (*mz_header).e_magic != IMAGE_DOS_SIGNATURE as UINT16 {
        if (*mz_header).e_cblp == 1 || (*mz_header).e_cblp == 2 {
            *retptr = (-1isize) as LPBYTE;
            return (*mz_header).e_cblp as DWORD;
        } else {
            return 0;
        }
    }
    if (*mz_header).e_lfanew as DWORD >= pesize {
        return 0;
    }
    if *(peimage.offset((*mz_header).e_lfanew as isize) as *mut DWORD) == IMAGE_NT_SIGNATURE {
        return IMAGE_NT_SIGNATURE;
    }

    if *(peimage.offset((*mz_header).e_lfanew as isize) as *mut UINT16) == IMAGE_OS2_SIGNATURE as UINT16 {
        let ne_header: *mut IMAGE_OS2_HEADER = peimage.offset((*mz_header).e_lfanew as isize) as *mut IMAGE_OS2_HEADER;

        if (*ne_header).ne_magic != IMAGE_OS2_SIGNATURE as UINT16 {
            return 0;
        }

        if ((*ne_header).ne_restab - (*ne_header).ne_rsrctab) as usize <= core::mem::size_of::<NE_TYPEINFO>() {
            *retptr = (-1isize) as LPBYTE;
        } else {
            *retptr = peimage.offset((*mz_header).e_lfanew as isize + (*ne_header).ne_rsrctab as isize);
        }

        return IMAGE_OS2_SIGNATURE as DWORD;
    }
    0
}

unsafe fn USER32_LoadResource(peimage: LPBYTE, pNInfo: *mut NE_NAMEINFO, sizeShift: WORD, uSize: *mut ULONG) -> *mut BYTE {
    *uSize = ((*pNInfo).length as DWORD) << sizeShift;
    peimage.add((((*pNInfo).offset as DWORD) << sizeShift) as usize)
}

unsafe fn ICO_LoadIcon(peimage: LPBYTE, lpiIDE: LPicoICONDIRENTRY, uSize: *mut ULONG) -> *mut BYTE {
    TRACE!(b"%p %p\n\0".as_ptr() as LPCSTR, peimage, lpiIDE);

    *uSize = (*lpiIDE).dwBytesInRes;
    peimage.add((*lpiIDE).dwImageOffset as usize)
}

unsafe fn ICO_GetIconDirectory(peimage: LPBYTE, lplpiID: *mut LPicoICONDIR, uSize: *mut ULONG) -> *mut BYTE {
    let lpcid: *mut CURSORICONDIR;
    let lpID: *mut CURSORICONDIR;
    let mut i: i32;

    TRACE!(b"%p %p\n\0".as_ptr() as LPCSTR, peimage, lplpiID);

    lpcid = peimage as *mut CURSORICONDIR;

    if (*lpcid).idReserved != 0 || (*lpcid).idType != 1 || (*lpcid).idCount == 0 {
        return core::ptr::null_mut();
    }

    *uSize = (FIELD_OFFSET!(CURSORICONDIR, idEntries)
        + (*lpcid).idCount as usize * core::mem::size_of::<CURSORICONDIRENTRY>()) as ULONG;

    lpID = HeapAlloc(GetProcessHeap(), 0, (*uSize) as SIZE_T) as *mut CURSORICONDIR;
    if !lpID.is_null() {
        (*lpID).idReserved = (*lpcid).idReserved;
        (*lpID).idType = (*lpcid).idType;
        (*lpID).idCount = (*lpcid).idCount;

        i = 0;
        while i < (*lpcid).idCount as i32 {
            let dst = (*lpID).idEntries.as_mut_ptr().offset(i as isize);
            let src = (*lpcid).idEntries.as_ptr().offset(i as isize);
            core::ptr::copy_nonoverlapping(
                src as *const u8,
                dst as *mut u8,
                core::mem::size_of::<CURSORICONDIRENTRY>() - 2,
            );
            (*dst).wResId = i as UINT16;
            i += 1;
        }

        *lplpiID = peimage as LPicoICONDIR;
        return lpID as *mut BYTE;
    }
    core::ptr::null_mut()
}

unsafe fn ICO_ExtractIconExW(
    mut lpszExeFileName: LPCWSTR,
    RetPtr: *mut HICON,
    mut nIconIndex: INT,
    mut nIcons: UINT,
    cxDesired: UINT,
    cyDesired: UINT,
    mut pIconId: *mut UINT,
    flags: UINT,
    fIconEx: BOOL,
) -> UINT {
    let mut ret: UINT = 0;
    let mut cx1: UINT;
    let mut cx2: UINT = 0;
    let mut cy1: UINT;
    let mut cy2: UINT = 0;
    let mut pData: LPBYTE = core::ptr::null_mut();
    let sig: DWORD;
    let hFile: HANDLE;
    let mut iconDirCount: UINT16 = 0;
    let mut iconCount: UINT16 = 0;
    let peimage: LPBYTE;
    let fmapping: HANDLE;
    let mut fsizeh: DWORD = 0;
    let fsizel: DWORD;
    let mut szExpandedExePath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szExePath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let dwSearchReturn: DWORD;

    if !RetPtr.is_null() {
        *RetPtr = core::ptr::null_mut();
    }

    if ExpandEnvironmentStringsW(
        lpszExeFileName,
        szExpandedExePath.as_mut_ptr(),
        szExpandedExePath.len() as DWORD,
    ) != 0
    {
        lpszExeFileName = szExpandedExePath.as_ptr();
    }

    dwSearchReturn = SearchPathW(
        core::ptr::null_mut(),
        lpszExeFileName,
        core::ptr::null_mut(),
        szExePath.len() as DWORD,
        szExePath.as_mut_ptr(),
        core::ptr::null_mut(),
    );
    if (dwSearchReturn == 0) || (dwSearchReturn > szExePath.len() as DWORD) {
        if fIconEx != 0 && RetPtr.is_null() && pIconId.is_null() {
            return 0;
        } else {
            return -1i32 as UINT;
        }
    }

    hFile = CreateFileW(
        szExePath.as_ptr(),
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return ret;
    }
    fsizel = GetFileSize(hFile, &mut fsizeh);

    if (fsizel | fsizeh) == 0 {
        CloseHandle(hFile);
        return 0;
    }

    fmapping = CreateFileMappingW(
        hFile,
        core::ptr::null_mut(),
        PAGE_READONLY | SEC_COMMIT,
        0,
        0,
        core::ptr::null_mut(),
    );
    CloseHandle(hFile);
    if fmapping.is_null() {
        WARN!(b"CreateFileMapping error %ld\n\0".as_ptr(), GetLastError());
        return 0xFFFFFFFF;
    }

    peimage = MapViewOfFile(fmapping, FILE_MAP_READ, 0, 0, 0) as LPBYTE;
    if peimage.is_null() {
        WARN!(b"MapViewOfFile error %ld\n\0".as_ptr(), GetLastError());
        CloseHandle(fmapping);
        return 0xFFFFFFFF;
    }
    CloseHandle(fmapping);

    'end: {
        if (fsizel >= 52) && memcmp(peimage as *const _, b"RIFF\0".as_ptr() as *const _, 4) == 0 {
            let anihOffset: UINT;
            let mut anihOff: UINT;
            let anihMax: UINT;
            let uSize: ULONG =
                MAKEWORD(*peimage.add(4), *peimage.add(5)) as ULONG;

            if (uSize < strlen(b"anih\0".as_ptr() as *const _) as ULONG) || (uSize > fsizel) {
                break 'end;
            }

            anihMax = uSize - strlen(b"anih\0".as_ptr() as *const _) as ULONG;
            anihOff = 0;
            while anihOff < anihMax {
                if memcmp(
                    peimage.add(anihOff as usize) as *const _,
                    b"anih\0".as_ptr() as *const _,
                    4,
                ) == 0
                {
                    break;
                }
                anihOff += 1;
            }
            anihOffset = anihOff;

            if anihOffset as ULONG + core::mem::size_of::<aniheader>() as ULONG > fsizel {
                break 'end;
            }

            ret = MAKEWORD(
                *peimage.add(anihOffset as usize + 12),
                *peimage.add(anihOffset as usize + 13),
            ) as UINT;

            cx1 = LOWORD(cxDesired) as UINT;
            cy1 = LOWORD(cyDesired) as UINT;

            if !RetPtr.is_null() {
                *RetPtr.add(0) = CreateIconFromResourceEx(
                    peimage,
                    uSize,
                    1u8 as BOOL,
                    0x00030000,
                    cx1 as INT,
                    cy1 as INT,
                    flags,
                );
            }
            break 'end;
        }

        cx1 = LOWORD(cxDesired) as UINT;
        cx2 = HIWORD(cxDesired) as UINT;
        cy1 = LOWORD(cyDesired) as UINT;
        cy2 = HIWORD(cyDesired) as UINT;

        if !pIconId.is_null() {
            *pIconId = 0xFFFFFFFF;
        }

        if pIconId.is_null() {
            pIconId = RetPtr as *mut UINT;
        }

        sig = USER32_GetResourceTable(peimage, fsizel, &mut pData);

        if sig == IMAGE_OS2_SIGNATURE {
            let mut pCIDir: *mut BYTE = core::ptr::null_mut();
            let mut pTInfo: *mut NE_TYPEINFO = (pData.add(2)) as *mut NE_TYPEINFO;
            let mut pIconStorage: *mut NE_NAMEINFO = core::ptr::null_mut();
            let mut pIconDir: *mut NE_NAMEINFO = core::ptr::null_mut();
            let mut lpiID: LPicoICONDIR = core::ptr::null_mut();
            let mut uSize: ULONG = 0;

            if pData == (-1isize) as LPBYTE {
                pCIDir = ICO_GetIconDirectory(peimage, &mut lpiID, &mut uSize);
                if !pCIDir.is_null() {
                    iconDirCount = 1;
                    iconCount = (*lpiID).idCount;
                }
            } else {
                while (*pTInfo).type_id != 0 && !(!pIconStorage.is_null() && !pIconDir.is_null()) {
                    if (*pTInfo).type_id as u32 == NE_RSCTYPE_GROUP_ICON {
                        iconDirCount = (*pTInfo).count;
                        pIconDir = pTInfo.add(1) as *mut NE_NAMEINFO;
                    }
                    if (*pTInfo).type_id as u32 == NE_RSCTYPE_ICON {
                        iconCount = (*pTInfo).count;
                        pIconStorage = pTInfo.add(1) as *mut NE_NAMEINFO;
                    }
                    pTInfo = ((pTInfo.add(1) as *mut core::ffi::c_char).add(
                        (*pTInfo).count as usize * core::mem::size_of::<NE_NAMEINFO>(),
                    )) as *mut NE_TYPEINFO;
                }
            }

            if (!pIconStorage.is_null() && !pIconDir.is_null()) || !lpiID.is_null() {
                if nIcons == 0 {
                    ret = iconDirCount as UINT;
                    if !lpiID.is_null() {
                        HeapFree(GetProcessHeap(), 0, pCIDir as *mut _);
                    }
                } else if nIconIndex < iconDirCount as INT {
                    let mut i: UINT16;
                    let mut icon: UINT16;
                    if nIcons > (iconDirCount as INT - nIconIndex) as UINT {
                        nIcons = (iconDirCount as INT - nIconIndex) as UINT;
                    }

                    i = 0;
                    while (i as UINT) < nIcons {
                        if lpiID.is_null() {
                            pCIDir = USER32_LoadResource(
                                peimage,
                                pIconDir.add(i as usize + nIconIndex as usize),
                                *(pData as *mut WORD),
                                &mut uSize,
                            );
                        }
                        *pIconId.add(i as usize) =
                            LookupIconIdFromDirectoryEx(pCIDir, 1u8 as BOOL, cx1 as INT, cy1 as INT, flags) as UINT;
                        if cx2 != 0 && cy2 != 0 {
                            i += 1;
                            *pIconId.add(i as usize) = LookupIconIdFromDirectoryEx(
                                pCIDir, 1u8 as BOOL, cx2 as INT, cy2 as INT, flags,
                            ) as UINT;
                        }
                        i += 1;
                    }
                    if !lpiID.is_null() {
                        HeapFree(GetProcessHeap(), 0, pCIDir as *mut _);
                    }

                    icon = 0;
                    while (icon as UINT) < nIcons {
                        pCIDir = core::ptr::null_mut();
                        if !lpiID.is_null() {
                            pCIDir = ICO_LoadIcon(
                                peimage,
                                (*lpiID)
                                    .idEntries
                                    .as_mut_ptr()
                                    .add(*pIconId.add(icon as usize) as INT as usize),
                                &mut uSize,
                            );
                        } else {
                            i = 0;
                            while (i as UINT) < iconCount as UINT {
                                if (*pIconStorage.add(i as usize)).id
                                    == ((*pIconId.add(icon as usize) as INT) | 0x8000) as WORD
                                {
                                    pCIDir = USER32_LoadResource(
                                        peimage,
                                        pIconStorage.add(i as usize),
                                        *(pData as *mut WORD),
                                        &mut uSize,
                                    );
                                }
                                i += 1;
                            }
                        }

                        if !pCIDir.is_null() {
                            *RetPtr.add(icon as usize) = CreateIconFromResourceEx(
                                pCIDir,
                                uSize,
                                1u8 as BOOL,
                                0x00030000,
                                cx1 as INT,
                                cy1 as INT,
                                flags,
                            );
                            if cx2 != 0 && cy2 != 0 {
                                icon += 1;
                                *RetPtr.add(icon as usize) = CreateIconFromResourceEx(
                                    pCIDir,
                                    uSize,
                                    1u8 as BOOL,
                                    0x00030000,
                                    cx2 as INT,
                                    cy2 as INT,
                                    flags,
                                );
                            }
                        } else {
                            *RetPtr.add(icon as usize) = core::ptr::null_mut();
                        }
                        icon += 1;
                    }
                    ret = icon as UINT;
                }
            }
        } else if sig == 1 || sig == 2 {
            if pData == (-1isize) as LPBYTE {
                let cx: [INT; 2] = [cx1 as INT, cx2 as INT];
                let cy: [INT; 2] = [cy1 as INT, cy2 as INT];
                let mut index: INT;

                index = 0;
                while index < (if cx2 != 0 || cy2 != 0 { 2 } else { 1 }) {
                    let dataOffset: DWORD;
                    let mut imageData: LPBYTE;
                    let mut hotSpot: POINT = core::mem::zeroed();

                    dataOffset = get_best_icon_file_offset(
                        peimage,
                        fsizel,
                        cx[index as usize],
                        cy[index as usize],
                        (sig == 1) as BOOL,
                        flags,
                        if sig == 1 {
                            core::ptr::null_mut()
                        } else {
                            &mut hotSpot
                        },
                    );

                    if dataOffset != 0 {
                        let icon: HICON;
                        let mut cursorData: *mut WORD = core::ptr::null_mut();
                        let mut bi: BITMAPINFOHEADER = core::mem::zeroed();
                        let mut cbColorTable: DWORD = 0;
                        let cbTotal: DWORD;

                        imageData = peimage.add(dataOffset as usize);

                        core::ptr::write_bytes(
                            &mut bi as *mut _ as *mut u8,
                            0,
                            core::mem::size_of::<BITMAPINFOHEADER>(),
                        );
                        CopyMemory(
                            &mut bi as *mut _ as *mut _,
                            imageData as *const _,
                            core::mem::size_of::<BITMAPCOREHEADER>(),
                        );
                        if bi.biBitCount <= 8 {
                            if bi.biSize as usize >= core::mem::size_of::<BITMAPINFOHEADER>() {
                                CopyMemory(
                                    &mut bi as *mut _ as *mut _,
                                    imageData as *const _,
                                    core::mem::size_of::<BITMAPINFOHEADER>(),
                                );
                                if bi.biClrUsed != 0 {
                                    cbColorTable =
                                        bi.biClrUsed * core::mem::size_of::<RGBQUAD>() as DWORD;
                                } else {
                                    cbColorTable = (1u32 << bi.biBitCount)
                                        * core::mem::size_of::<RGBQUAD>() as DWORD;
                                }
                            } else if bi.biSize as usize
                                == core::mem::size_of::<BITMAPCOREHEADER>()
                            {
                                cbColorTable = (1u32 << bi.biBitCount)
                                    * core::mem::size_of::<RGBTRIPLE>() as DWORD;
                            }
                        }

                        if bi.biSizeImage == 0 {
                            bi.biSizeImage = (((bi.biWidth * bi.biBitCount as i32 + 31) / 32 * 4)
                                * (bi.biHeight / 2))
                                as DWORD;
                            bi.biSizeImage += (((bi.biWidth * 1 + 31) / 32 * 4)
                                * (bi.biHeight / 2))
                                as DWORD;
                        }

                        cbTotal = bi.biSize + cbColorTable + bi.biSizeImage;

                        if sig == 2 {
                            cursorData = HeapAlloc(
                                GetProcessHeap(),
                                0,
                                (2 * core::mem::size_of::<WORD>() + (cbTotal) as usize) as SIZE_T,
                            ) as *mut WORD;

                            if cursorData.is_null() {
                                index += 1;
                                continue;
                            }

                            *cursorData.add(0) = hotSpot.x as WORD;
                            *cursorData.add(1) = hotSpot.y as WORD;

                            CopyMemory(
                                cursorData.add(2) as *mut _,
                                imageData as *const _,
                                cbTotal as usize,
                            );

                            imageData = cursorData as LPBYTE;
                        }

                        icon = CreateIconFromResourceEx(
                            imageData,
                            cbTotal,
                            (sig == 1) as BOOL,
                            0x00030000,
                            cx[index as usize],
                            cy[index as usize],
                            flags,
                        );
                        if fIconEx != 0 && sig == 1 {
                            iconCount = 1;
                        }

                        HeapFree(GetProcessHeap(), 0, cursorData as *mut _);

                        if !icon.is_null() {
                            if !RetPtr.is_null() {
                                *RetPtr.add(index as usize) = icon;
                            } else {
                                DestroyIcon(icon);
                            }

                            iconCount = 1;
                            break;
                        }
                    }
                    index += 1;
                }
            }
            ret = iconCount as UINT;
        } else if sig == IMAGE_NT_SIGNATURE {
            let mut idata: *mut BYTE;
            let mut igdata: *mut BYTE;
            let rootresdir: *const IMAGE_RESOURCE_DIRECTORY;
            let iconresdir: *const IMAGE_RESOURCE_DIRECTORY;
            let icongroupresdir: *const IMAGE_RESOURCE_DIRECTORY;
            // C declares these as plain autos; in the "icon entry not found" recovery
            // branch the cx2/cy2 path reads idataent before its own assignment (faithful
            // to the C, which would read a stale/prior value). Default to null so Rust
            // sees a defined binding.
            let mut idataent: *const IMAGE_RESOURCE_DATA_ENTRY = core::ptr::null();
            let mut igdataent: *const IMAGE_RESOURCE_DATA_ENTRY;
            let mut xresent: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
            let mut size: ULONG = 0;
            let mut i: UINT;

            rootresdir = RtlImageDirectoryEntryToData(
                peimage as HMODULE as *mut _,
                0u8,
                IMAGE_DIRECTORY_ENTRY_RESOURCE as USHORT,
                &mut size,
            ) as *const IMAGE_RESOURCE_DIRECTORY;
            if rootresdir.is_null() {
                WARN!(b"haven't found section for resource directory.\n\0".as_ptr());
                break 'end;
            }

            if ((rootresdir.add(1) as ULONG_PTR) < (rootresdir as ULONG_PTR))
                || ((rootresdir.add(1) as ULONG_PTR)
                    > (peimage as ULONG_PTR + fsizel as ULONG_PTR))
            {
                break 'end;
            }

            icongroupresdir = find_entry_by_id(
                rootresdir,
                LOWORD(RT_GROUP_ICON as DWORD),
                rootresdir as *const _,
            );
            if icongroupresdir.is_null() {
                WARN!(b"No Icongroupresourcedirectory!\n\0".as_ptr());
                break 'end;
            }
            iconDirCount = (*icongroupresdir).NumberOfNamedEntries
                + (*icongroupresdir).NumberOfIdEntries;

            if pIconId.is_null() {
                ret = iconDirCount as UINT;
                break 'end;
            }

            if nIconIndex < 0 {
                let mut n: INT = 0;
                let iId: INT = abs(nIconIndex);
                let mut xprdeTmp: *const IMAGE_RESOURCE_DIRECTORY_ENTRY =
                    icongroupresdir.add(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;

                while n < iconDirCount as INT && !xprdeTmp.is_null() {
                    if (*xprdeTmp).__bindgen_anon_1.Id as INT == iId {
                        nIconIndex = n;
                        break;
                    }
                    n += 1;
                    xprdeTmp = xprdeTmp.add(1);
                }
                if nIconIndex < 0 {
                    WARN!(b"resource id %d not found\n\0".as_ptr(), iId);
                    break 'end;
                }
            } else {
                if nIconIndex >= iconDirCount as INT {
                    WARN!(
                        b"nIconIndex %d is larger than iconDirCount %d\n\0".as_ptr(),
                        nIconIndex,
                        iconDirCount as INT,
                    );
                    break 'end;
                }
            }

            if nIcons > (iconDirCount as INT - nIconIndex) as UINT {
                nIcons = (iconDirCount as INT - nIconIndex) as UINT;
            }

            xresent = (icongroupresdir.add(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY)
                .add(nIconIndex as usize);

            i = 0;
            while i < nIcons {
                let mut resdir: *const IMAGE_RESOURCE_DIRECTORY;

                resdir = ((rootresdir as *const core::ffi::c_char)
                    .add((*xresent).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as usize))
                    as *const IMAGE_RESOURCE_DIRECTORY;

                resdir = find_entry_default(resdir, rootresdir as *const _);
                igdataent = resdir as *const IMAGE_RESOURCE_DATA_ENTRY;

                igdata = RtlImageRvaToVa(
                    RtlImageNtHeader(peimage as HMODULE as *mut _),
                    peimage as HMODULE as *mut _,
                    (*igdataent).OffsetToData,
                    core::ptr::null_mut(),
                ) as *mut BYTE;
                if igdata.is_null() {
                    FIXME!("no matching real address for icongroup!\n");
                    break 'end;
                }
                *pIconId.add(i as usize) =
                    LookupIconIdFromDirectoryEx(igdata, 1u8 as BOOL, cx1 as INT, cy1 as INT, flags) as UINT;
                if cx2 != 0 && cy2 != 0 {
                    i += 1;
                    *pIconId.add(i as usize) = LookupIconIdFromDirectoryEx(
                        igdata, 1u8 as BOOL, cx2 as INT, cy2 as INT, flags,
                    ) as UINT;
                }
                i += 1;
                xresent = xresent.add(1);
            }

            iconresdir =
                find_entry_by_id(rootresdir, LOWORD(RT_ICON as DWORD), rootresdir as *const _);
            if iconresdir.is_null() {
                WARN!(b"No Iconresourcedirectory!\n\0".as_ptr());
                break 'end;
            }

            i = 0;
            while i < nIcons {
                let mut xresdir: *const IMAGE_RESOURCE_DIRECTORY;
                xresdir = find_entry_by_id(
                    iconresdir,
                    LOWORD(*pIconId.add(i as usize)),
                    rootresdir as *const _,
                );
                if xresdir.is_null() {
                    let resdir: *const IMAGE_RESOURCE_DIRECTORY;
                    WARN!(
                        b"icon entry %d not found\n\0".as_ptr(),
                        LOWORD(*pIconId.add(i as usize)) as INT,
                    );

                    if (*iconresdir).NumberOfIdEntries > 0 {
                        xresent = (iconresdir.add(1) as ULONG_PTR)
                            as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
                    } else {
                        *RetPtr.add(i as usize) = core::ptr::null_mut();
                        i += 1;
                        continue;
                    }

                    resdir = ((rootresdir as *const core::ffi::c_char)
                        .add((*xresent).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as usize))
                        as *const IMAGE_RESOURCE_DIRECTORY;

                    if (*resdir).NumberOfIdEntries > 0 {
                        xresent = (resdir.add(1) as ULONG_PTR)
                            as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
                    } else {
                        *RetPtr.add(i as usize) = core::ptr::null_mut();
                        i += 1;
                        continue;
                    }

                    igdataent = ((rootresdir as *const core::ffi::c_char)
                        .add((*xresent).__bindgen_anon_2.OffsetToData as usize))
                        as *const IMAGE_RESOURCE_DATA_ENTRY;
                    idata = RtlImageRvaToVa(
                        RtlImageNtHeader(peimage as HMODULE as *mut _),
                        peimage as HMODULE as *mut _,
                        (*igdataent).OffsetToData,
                        core::ptr::null_mut(),
                    ) as *mut BYTE;
                    if idata.is_null() {
                        *RetPtr.add(i as usize) = core::ptr::null_mut();
                        i += 1;
                        continue;
                    }

                    if *idata.add(0) as usize == core::mem::size_of::<BITMAPINFOHEADER>() {
                        let mut bmih: BITMAPINFOHEADER = core::mem::zeroed();
                        RtlCopyMemory(
                            &mut bmih as *mut _ as *mut _,
                            idata as *const _,
                            core::mem::size_of::<BITMAPINFOHEADER>(),
                        );
                        if (bmih.biWidth * 2) == bmih.biHeight {
                            *RetPtr.add(0) = CreateIconFromResourceEx(
                                idata,
                                (*igdataent).Size,
                                1u8 as BOOL,
                                0x00030000,
                                cx1 as INT,
                                cy1 as INT,
                                flags,
                            );
                            if cx2 != 0 && cy2 != 0 {
                                *RetPtr.add(1) = CreateIconFromResourceEx(
                                    idata,
                                    (*idataent).Size,
                                    1u8 as BOOL,
                                    0x00030000,
                                    cx2 as INT,
                                    cy2 as INT,
                                    flags,
                                );
                            }
                            ret = 1;
                            break 'end;
                        }
                    } else {
                        *RetPtr.add(i as usize) = core::ptr::null_mut();
                        i += 1;
                        continue;
                    }
                    i += 1;
                    continue;
                }
                xresdir = find_entry_default(xresdir, rootresdir as *const _);
                idataent = xresdir as *const IMAGE_RESOURCE_DATA_ENTRY;

                idata = RtlImageRvaToVa(
                    RtlImageNtHeader(peimage as HMODULE as *mut _),
                    peimage as HMODULE as *mut _,
                    (*idataent).OffsetToData,
                    core::ptr::null_mut(),
                ) as *mut BYTE;
                if idata.is_null() {
                    WARN!(b"no matching real address found for icondata!\n\0".as_ptr());
                    *RetPtr.add(i as usize) = core::ptr::null_mut();
                    i += 1;
                    continue;
                }
                *RetPtr.add(i as usize) = CreateIconFromResourceEx(
                    idata,
                    (*idataent).Size,
                    1u8 as BOOL,
                    0x00030000,
                    cx1 as INT,
                    cy1 as INT,
                    flags,
                );
                if cx2 != 0 && cy2 != 0 {
                    i += 1;
                    *RetPtr.add(i as usize) = CreateIconFromResourceEx(
                        idata,
                        (*idataent).Size,
                        1u8 as BOOL,
                        0x00030000,
                        cx2 as INT,
                        cy2 as INT,
                        flags,
                    );
                }
                i += 1;
            }
            ret = i;
        }
    }

    UnmapViewOfFile(peimage as *mut _);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn PrivateExtractIconsW(
    lpwstrFile: LPCWSTR,
    nIndex: core::ffi::c_int,
    sizeX: core::ffi::c_int,
    sizeY: core::ffi::c_int,
    phicon: *mut HICON,
    pIconId: *mut UINT,
    nIcons: UINT,
    flags: UINT,
) -> UINT {
    TRACE!(
        b"%s %d %dx%d %p %p %d 0x%08x\n\0".as_ptr() as *const core::ffi::c_char,
        debugstr_w(lpwstrFile),
        nIndex,
        sizeX,
        sizeY,
        phicon,
        pIconId,
        nIcons,
        flags,
    );

    if (nIcons & 1) != 0 && HIWORD(sizeX as DWORD) != 0 && HIWORD(sizeY as DWORD) != 0 {
        WARN!(
            b"Uneven number %d of icons requested for small and large icons!\n\0".as_ptr()
                as *const core::ffi::c_char,
            nIcons,
        );
    }

    ICO_ExtractIconExW(
        lpwstrFile, phicon, nIndex, nIcons, sizeX as UINT, sizeY as UINT, pIconId, flags, 1 as BOOL,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PrivateExtractIconsA(
    lpstrFile: LPCSTR,
    nIndex: core::ffi::c_int,
    sizeX: core::ffi::c_int,
    sizeY: core::ffi::c_int,
    phicon: *mut HICON,
    piconid: *mut UINT,
    nIcons: UINT,
    flags: UINT,
) -> UINT {

    let ret: UINT;
    let len: core::ffi::c_int =
        MultiByteToWideChar(0, 0, lpstrFile, -1, core::ptr::null_mut(), 0);
    let lpwstrFile = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;

    if lpwstrFile.is_null() {
        return 0;
    }

    MultiByteToWideChar(0, 0, lpstrFile, -1, lpwstrFile, len);
    ret = PrivateExtractIconsW(
        lpwstrFile, nIndex, sizeX, sizeY, phicon, piconid, nIcons, flags,
    );

    HeapFree(GetProcessHeap(), 0, lpwstrFile as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn PrivateExtractIconExW(
    lpwstrFile: LPCWSTR,
    nIndex: core::ffi::c_int,
    phIconLarge: *mut HICON,
    phIconSmall: *mut HICON,
    nIcons: UINT,
) -> UINT {
    let cyicon: DWORD;
    let cysmicon: DWORD;
    let cxicon: DWORD;
    let cxsmicon: DWORD;
    let mut ret: UINT = 0;

    TRACE!(
        b"%s %d %p %p %d\n\0".as_ptr() as LPCSTR,
        debugstr_w(lpwstrFile),
        nIndex,
        phIconLarge,
        phIconSmall,
        nIcons,
    );

    if nIndex == -1 || (phIconSmall.is_null() && phIconLarge.is_null()) {
        return ICO_ExtractIconExW(
            lpwstrFile,
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            core::ptr::null_mut(),
            LR_DEFAULTCOLOR,
            0 as BOOL,
        );
    }

    if nIcons == 1 && !phIconSmall.is_null() && !phIconLarge.is_null() {
        let mut hIcon: [HICON; 2] = [core::ptr::null_mut(); 2];
        cxicon = GetSystemMetrics(SM_CXICON as core::ffi::c_int) as DWORD;
        cyicon = GetSystemMetrics(SM_CYICON as core::ffi::c_int) as DWORD;
        cxsmicon = GetSystemMetrics(SM_CXSMICON as core::ffi::c_int) as DWORD;
        cysmicon = GetSystemMetrics(SM_CYSMICON as core::ffi::c_int) as DWORD;

        ret = ICO_ExtractIconExW(
            lpwstrFile,
            hIcon.as_mut_ptr(),
            nIndex,
            2,
            cxicon | (cxsmicon << 16),
            cyicon | (cysmicon << 16),
            core::ptr::null_mut(),
            LR_DEFAULTCOLOR,
            0 as BOOL,
        );
        *phIconLarge = hIcon[0];
        *phIconSmall = hIcon[1];
        return ret;
    }

    if !phIconSmall.is_null() {
        cxsmicon = GetSystemMetrics(SM_CXSMICON as core::ffi::c_int) as DWORD;
        cysmicon = GetSystemMetrics(SM_CYSMICON as core::ffi::c_int) as DWORD;
        ret = ICO_ExtractIconExW(
            lpwstrFile,
            phIconSmall,
            nIndex,
            nIcons,
            cxsmicon,
            cysmicon,
            core::ptr::null_mut(),
            LR_DEFAULTCOLOR,
            0 as BOOL,
        );
    }
    if !phIconLarge.is_null() {
        cxicon = GetSystemMetrics(SM_CXICON as core::ffi::c_int) as DWORD;
        cyicon = GetSystemMetrics(SM_CYICON as core::ffi::c_int) as DWORD;
        ret = ICO_ExtractIconExW(
            lpwstrFile,
            phIconLarge,
            nIndex,
            nIcons,
            cxicon,
            cyicon,
            core::ptr::null_mut(),
            LR_DEFAULTCOLOR,
            0 as BOOL,
        );
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn PrivateExtractIconExA(
    lpstrFile: LPCSTR,
    nIndex: core::ffi::c_int,
    phIconLarge: *mut HICON,
    phIconSmall: *mut HICON,
    nIcons: UINT,
) -> UINT {
    let ret: UINT;
    let len: INT = MultiByteToWideChar(CP_ACP, 0, lpstrFile, -1, core::ptr::null_mut(), 0);
    let lpwstrFile: LPWSTR = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as LPWSTR;

    if lpwstrFile.is_null() {
        return 0;
    }

    TRACE!(
        b"%s %d %p %p %d\n\0".as_ptr() as LPCSTR,
        lpstrFile,
        nIndex,
        phIconLarge,
        phIconSmall,
        nIcons,
    );

    MultiByteToWideChar(CP_ACP, 0, lpstrFile, -1, lpwstrFile, len);
    ret = PrivateExtractIconExW(lpwstrFile, nIndex, phIconLarge, phIconSmall, nIcons);
    HeapFree(GetProcessHeap(), 0, lpwstrFile as *mut core::ffi::c_void);
    ret
}
