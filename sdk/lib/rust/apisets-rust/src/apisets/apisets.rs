// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_D7A3DA74DB86: [u16; 5] = [0x002Eu16, 0x0064, 0x006C, 0x006C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut API_: ULONGLONG = 0x2d004900500041 as ::core::ffi::c_longlong as ULONGLONG;

#[no_mangle]
pub static mut EXT_: ULONGLONG = 0x2d005400580045 as ::core::ffi::c_longlong as ULONGLONG;

#[no_mangle]
pub static mut PrefixSize: WORD = (::core::mem::size_of::<[::core::ffi::c_ushort; 5]>() as usize)
        .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize) as WORD;

#[no_mangle]
pub static mut ExtensionSize: WORD = (::core::mem::size_of::<[::core::ffi::c_ushort; 5]>() as usize)
        .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize) as WORD;
#[no_mangle]
pub unsafe extern "C" fn ApiSetResolveToHost(
    ApisetVersion: DWORD,
    ApiToResolve: PCUNICODE_STRING,
    Resolved: PBOOLEAN,
    Output: PUNICODE_STRING,
) -> NTSTATUS {
    if ((*ApiToResolve).Length as i32) < PrefixSize as i32 {
        *Resolved = (FALSE as u8);
        return STATUS_SUCCESS;
    }

    // Grab the first four chars from the string, converting the first 3 to uppercase
    let ApiSetNameBuffer: PWSTR = (*ApiToResolve).Buffer;
    let ApiSetNameBufferPrefix: ULONGLONG =
        *(ApiSetNameBuffer as *mut ULONGLONG).offset(0) & 0xFFFFFFDFFFDFFFDF;
    // Check if it matches either 'api-' or 'ext-'
    if !(ApiSetNameBufferPrefix == API_ || ApiSetNameBufferPrefix == EXT_) {
        *Resolved = (FALSE as u8);
        return STATUS_SUCCESS;
    }

    // If there is an extension, cut it off (we store apisets without extension)
    let mut Tmp: _UNICODE_STRING = *ApiToResolve;
    let Extension: *const WCHAR = Tmp.Buffer.offset(
        ((Tmp.Length as i32 - ExtensionSize as i32) as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
    ) as *const WCHAR;
    if _wcsnicmp(
        Extension,
        RUSTCONV_WSTR_D7A3DA74DB86.as_ptr(),
        (ExtensionSize as usize).wrapping_div(core::mem::size_of::<WCHAR>()),
    ) == 0
    {
        Tmp.Length = (Tmp.Length as i32 - ExtensionSize as i32) as USHORT;
    }

    // Binary search the apisets
    // Ideally we should use bsearch here, but that drags in another dependency and we do not want that here.
    let mut UBnd: LONG = g_ApisetsCount - 1;
    let mut LBnd: LONG = 0;
    while LBnd <= UBnd {
        let Index: LONG = (UBnd - LBnd) / 2 + LBnd;

        let result: LONG = RtlCompareUnicodeString(
            core::ptr::addr_of_mut!(Tmp) as PCUNICODE_STRING,
            core::ptr::addr_of!((*(core::ptr::addr_of!(g_Apisets) as *const ROS_APISET).offset(Index as isize)).Name)
                as PCUNICODE_STRING,
            (TRUE) as BOOLEAN,
        );
        if result == 0 {
            // Check if this version is included
            if (*(core::ptr::addr_of!(g_Apisets) as *const ROS_APISET).offset(Index as isize)).dwOsVersions
                & ApisetVersion
                != 0
            {
                // Return a static string (does not have to be freed)
                *Resolved = (TRUE as u8);
                *Output = (*(core::ptr::addr_of!(g_Apisets) as *const ROS_APISET).offset(Index as isize)).Target;
            }
            return STATUS_SUCCESS;
        } else if result < 0 {
            UBnd = Index - 1;
        } else {
            LBnd = Index + 1;
        }
    }

    // No exact row: match as Windows does, ignoring the trailing minor version (its
    // schema hashes a name only up to the last '-'). Kept in step with ntdll-rust's
    // apisets.rs, the copy the loader runs; this one is what the apisets apitest links.
    let Name = core::slice::from_raw_parts(Tmp.Buffer as *const u16, (Tmp.Length / 2) as usize);
    let is_minor = |s: &[u16]| !s.is_empty() && s.iter().all(|&c| (b'0' as u16..=b'9' as u16).contains(&c));
    if let Some(Hyphen) = Name.iter().rposition(|&c| c == b'-' as u16) {
        if is_minor(&Name[Hyphen + 1..]) {
            let mut Stem: _UNICODE_STRING = Tmp;
            Stem.Length = ((Hyphen + 1) * 2) as USHORT;
            let mut Found: *const ROS_APISET = core::ptr::null();
            for n in 0..g_ApisetsCount as isize {
                let Entry = (core::ptr::addr_of!(g_Apisets) as *const ROS_APISET).offset(n);
                let EntryName = core::slice::from_raw_parts(
                    (*Entry).Name.Buffer as *const u16,
                    ((*Entry).Name.Length / 2) as usize,
                );
                if EntryName.len() <= Hyphen + 1 || !is_minor(&EntryName[Hyphen + 1..]) {
                    continue;
                }
                let mut Head: _UNICODE_STRING = (*Entry).Name;
                Head.Length = Stem.Length;
                if RtlCompareUnicodeString(&Stem, &Head, TRUE as BOOLEAN) == 0
                    && (*Entry).dwOsVersions & ApisetVersion != 0
                {
                    Found = Entry;
                }
            }
            if !Found.is_null() {
                *Resolved = (TRUE as u8);
                *Output = (*Found).Target;
                return STATUS_SUCCESS;
            }
        }
    }
    *Resolved = (FALSE as u8);
    return STATUS_SUCCESS;
}
