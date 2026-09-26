use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::ddk::{RtlAppendUnicodeToString, RtlInitUnicodeString, ZwClose, ZwCreateFile, ZwCreateSection, ZwMapViewOfSection, ZwUnmapViewOfSection};
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

extern "system" {
    // safe_user_io.h — NTAPI probe (Length = SIZE_T), see console.rs/copy.rs.
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
}

// SetLastNtError (`@SetLastNtError@4`) and the DIB_* helpers are FASTCALL.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn DIB_FreeConvertedBitmapInfo(converted: LPBITMAPINFO, orig: LPBITMAPINFO, Usage: DWORD);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn DIB_ConvertBitmapInfo(bmi: LPBITMAPINFO, Usage: DWORD) -> LPBITMAPINFO;
    fn DIB_FreeConvertedBitmapInfo(converted: LPBITMAPINFO, orig: LPBITMAPINFO, Usage: DWORD);
}

const INVALID_HANDLE_VALUE: isize = -1; // (HANDLE)-1

// L"\\??\\" as a real static [u16] — the original transliteration built a
// temporary Vec and took .as_ptr() of it, which dangles at the statement end.
static NT_PATH_PREFIX: [u16; 5] = [0x005C, 0x003F, 0x003F, 0x005C, 0x0000];

#[inline]
unsafe fn NtCurrentProcess() -> HANDLE {
    (-1isize) as HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn W32kDosPathNameToNtPathName(
    pwszDosPathName: PCWSTR,
    pustrNtPathName: PUNICODE_STRING,
) -> BOOL {
    let mut Status;

    /* Prepend "\??\" */
    (*pustrNtPathName).Length = 0;
    Status = RtlAppendUnicodeToString(pustrNtPathName as _, NT_PATH_PREFIX.as_ptr());
    if !NT_SUCCESS(Status) {
        return FALSE as BOOL;
    }

    /* Append the dos name */
    Status = RtlAppendUnicodeToString(pustrNtPathName as _, pwszDosPathName);
    if !NT_SUCCESS(Status) {
        return FALSE as BOOL;
    }

    TRUE as BOOL
}

// Internal helpers (not referenced outside file.c) — left un-exported.
unsafe extern "system" fn W32kOpenFile(pwszFileName: PCWSTR, dwDesiredAccess: u32) -> HANDLE {
    let mut ustrFile: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE as HANDLE;
    let Status: NTSTATUS;

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ustrFile) as _, pwszFileName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.ObjectName = &mut ustrFile;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateFile(
        &mut hFile,
        dwDesiredAccess,
        core::ptr::addr_of_mut!(ObjectAttributes) as _,
        core::ptr::addr_of_mut!(IoStatusBlock) as _,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        0,
        FILE_OPEN,
        FILE_NON_DIRECTORY_FILE,
        core::ptr::null_mut(),
        0,
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        hFile = core::ptr::null_mut();
    }

    hFile
}

unsafe extern "system" fn W32kCreateFileSection(
    hFile: HANDLE,
    flAllocation: ULONG,
    flPageProtection: DWORD,
    _ullMaxSize: ULONGLONG,
) -> HANDLE {
    let Status: NTSTATUS;
    let mut hSection: HANDLE = core::ptr::null_mut();
    let mut amDesiredAccess: ACCESS_MASK;

    amDesiredAccess = (STANDARD_RIGHTS_REQUIRED | SECTION_QUERY | SECTION_MAP_READ) as ACCESS_MASK;

    if flPageProtection == PAGE_READWRITE {
        amDesiredAccess |= SECTION_MAP_WRITE as ACCESS_MASK;
    }

    Status = ZwCreateSection(
        &mut hSection,
        amDesiredAccess,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        flPageProtection,
        flAllocation,
        hFile,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        hSection = core::ptr::null_mut();
    }

    hSection
}

unsafe extern "system" fn W32kMapViewOfSection(
    hSection: HANDLE,
    dwPageProtect: DWORD,
    ulSectionOffset: ULONG_PTR,
) -> PVOID {
    let Status: NTSTATUS;
    let mut liSectionOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ulViewSize: ULONG_PTR;
    let mut pvBase: PVOID = core::ptr::null_mut();

    ulViewSize = ulSectionOffset;
    liSectionOffset.QuadPart = ulSectionOffset as i64;

    Status = ZwMapViewOfSection(
        hSection,
        NtCurrentProcess(),
        &mut pvBase,
        0,
        0,
        core::ptr::addr_of_mut!(liSectionOffset) as _,
        &mut ulViewSize,
        ViewShare,
        0,
        dwPageProtect,
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        pvBase = core::ptr::null_mut();
    }

    pvBase
}

// UserLoadImage is NTAPI (extern "system" → stdcall on i386, cdecl elsewhere) and
// referenced from other win32k TUs, so #[no_mangle]. One body covers all arches.
#[no_mangle]
pub unsafe extern "system" fn UserLoadImage(pwszName: PCWSTR) -> HBITMAP {
    let mut Status: NTSTATUS;
    let hFile: HANDLE;
    let hSection: HANDLE;
    let pbmfh: *mut BITMAPFILEHEADER;
    let pbmi: LPBITMAPINFO;
    let pvBits: PVOID;
    let mut hbmp: HBITMAP = core::ptr::null_mut();

    /* Open the file */
    hFile = W32kOpenFile(pwszName, FILE_READ_DATA);
    if hFile.is_null() {
        return core::ptr::null_mut();
    }

    /* Create a section */
    hSection = W32kCreateFileSection(hFile, SEC_COMMIT, PAGE_READONLY, 0);
    ZwClose(hFile);
    if hSection.is_null() {
        return core::ptr::null_mut();
    }

    /* Map the section */
    pbmfh = W32kMapViewOfSection(hSection, PAGE_READONLY, 0) as *mut BITMAPFILEHEADER;
    ZwClose(hSection);
    if pbmfh.is_null() {
        return core::ptr::null_mut();
    }

    /* Get a pointer to the BITMAPINFO */
    pbmi = pbmfh.add(1) as LPBITMAPINFO;

    'leave: {
        Status = SafeProbeForRead(
            core::ptr::addr_of_mut!((*pbmfh).bfSize) as PVOID,
            core::mem::size_of::<DWORD>() as SIZE_T,
            1,
        );
        // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped the probes AND every
        // header field read in one _SEH2_TRY. pbmfh points into a section mapped into
        // the caller's address space, which another thread can unmap under us, so the
        // header is captured ONCE into kernel memory and read from there below.
        let mut SafeHdr: BITMAPFILEHEADER = core::mem::zeroed();
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(SafeHdr) as *mut core::ffi::c_void,
                pbmfh as *const core::ffi::c_void,
                core::mem::size_of::<BITMAPFILEHEADER>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(pbmfh as PVOID, SafeHdr.bfSize as SIZE_T, 1);
        }

        if !NT_SUCCESS(Status) {
            break 'leave;
        }

        if SafeHdr.bfType == 0x4D42 /* 'BM' */ {
            /* Could be BITMAPCOREINFO */
            let pConvertedInfo: *mut BITMAPINFO;
            let hdc: HDC;

            pvBits = (pbmfh as PCHAR).add(SafeHdr.bfOffBits as usize) as PVOID;

            pConvertedInfo = DIB_ConvertBitmapInfo(pbmi, DIB_RGB_COLORS) as *mut BITMAPINFO;
            if pConvertedInfo.is_null() {
                break 'leave;
            }

            hdc = IntGdiCreateDC(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
            );

            hbmp = GreCreateDIBitmapInternal(
                hdc,
                (*pConvertedInfo).bmiHeader.biWidth,
                (*pConvertedInfo).bmiHeader.biHeight,
                CBM_INIT,
                pvBits as LPBYTE,
                pConvertedInfo as PBITMAPINFO,
                DIB_RGB_COLORS,
                0,
                SafeHdr.bfSize - SafeHdr.bfOffBits,
                core::ptr::null_mut(),
            );

            NtGdiDeleteObjectApp(hdc as PVOID);
            DIB_FreeConvertedBitmapInfo(pConvertedInfo as LPBITMAPINFO, pbmi, (-1i32) as DWORD);
        }
    }

    /* Unmap our section, we don't need it anymore */
    ZwUnmapViewOfSection(NtCurrentProcess(), pbmfh as PVOID);

    hbmp
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DIB_ConvertBitmapInfo(bmi: LPBITMAPINFO, Usage: DWORD) -> LPBITMAPINFO;
}
